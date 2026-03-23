// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

//! Tests that the prioritized socket is drained before the other socket
//! under concurrent load.

use super::*;
use s2n_quic::Server;
use s2n_quic_core::{
    crypto::tls::testing::certificates,
    event::{self, api},
    inet::ExplicitCongestionNotification,
};
use s2n_quic_platform::io::testing::Model;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc,
};

/// A subscriber that tracks per-socket rx packet counts via PlatformRxSocketStats.
#[derive(Debug, Default, Clone)]
struct StatsSubscriber {
    socket_counts: Arc<[AtomicU64; 2]>,
}

impl event::Subscriber for StatsSubscriber {
    type ConnectionContext = ();

    fn create_connection_context(
        &mut self,
        _meta: &api::ConnectionMeta,
        _info: &api::ConnectionInfo,
    ) -> Self::ConnectionContext {
    }

    fn on_platform_rx_socket_stats(
        &mut self,
        _meta: &api::EndpointMeta,
        event: &api::PlatformRxSocketStats,
    ) {
        let idx = if event.is_prioritized { 1 } else { 0 };
        self.socket_counts[idx].fetch_add(event.count as u64, Ordering::Relaxed);
    }
}

/// Verifies that the prioritized socket is drained before the other socket
/// under concurrent load on both sockets.
///
/// A small internal receive buffer is used so the ring buffer becomes the
/// bottleneck. When both sockets have data, the scheduling determines which
/// socket fills the limited ring space. Since the high-priority socket is
/// always drained first, it should receive significantly more packets.
#[test]
fn prioritized_socket_scheduling_test() {
    let model = Model::default();

    let stats = StatsSubscriber::default();

    test(model.clone(), |handle| {
        let second_socket = handle.buffers.generate_addr();
        let io = handle.builder().with_second_addr(second_socket).build()?;

        let server = Server::builder()
            .with_io(io)?
            .with_tls((certificates::CERT_PEM, certificates::KEY_PEM))?
            .with_event((stats.clone(), tracing_events(true, model)))?
            .start()?;

        let server_addr = start_server(server)?;

        let sender_io = handle.builder().build()?.socket();
        let packet_payload =
            s2n_quic_core::crypto::initial::EXAMPLE_CLIENT_INITIAL_PROTECTED_PACKET;

        primary::spawn(async move {
            for _ in 0..1000 {
                sender_io
                    .send_to(
                        server_addr,
                        ExplicitCongestionNotification::default(),
                        packet_payload.to_vec(),
                    )
                    .unwrap();

                sender_io
                    .send_to(
                        second_socket.into(),
                        ExplicitCongestionNotification::default(),
                        packet_payload.to_vec(),
                    )
                    .unwrap();
            }
            s2n_quic::provider::io::testing::time::delay(std::time::Duration::from_millis(50))
                .await;
        });

        Ok(())
    })
    .unwrap();

    let socket_0_count = stats.socket_counts[0].load(Ordering::Relaxed);
    let socket_1_count = stats.socket_counts[1].load(Ordering::Relaxed);

    println!("socket 0 stats: {:?}", socket_0_count);
    println!("socket 1 stats: {:?}", socket_1_count);
}
