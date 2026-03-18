// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

#![allow(clippy::unnecessary_cast)] // some platforms encode lengths as `u32` so we cast everything to be safe

use crate::features;
use core::mem::size_of;
use libc::cmsghdr;

pub mod decode;
pub mod encode;
pub mod storage;

#[cfg(test)]
mod tests;

pub use encode::Encoder;
pub use storage::Storage;

pub const fn size_of_cmsg<T: Copy + Sized>() -> usize {
    unsafe { libc::CMSG_SPACE(size_of::<T>() as _) as _ }
}

/// Extracts the `SO_RXQ_OVFL` dropped packet count from a `msghdr`'s ancillary data.
///
/// Returns `None` if the cmsg is not present or the feature is not supported.
///
/// # Safety
///
/// The `msghdr` must have valid `msg_control` and `msg_controllen` fields.
#[inline]
pub unsafe fn dropped_packets_from_msghdr(msg: &libc::msghdr) -> Option<u32> {
    if msg.msg_control.is_null() || msg.msg_controllen == 0 {
        return None;
    }

    let iter = decode::Iter::from_msghdr(msg);
    for (cmsg, value) in iter {
        if features::rxq_ovfl::is_match(cmsg.cmsg_level, cmsg.cmsg_type) {
            return decode::value_from_bytes::<features::rxq_ovfl::Cmsg>(value);
        }
    }

    None
}

const fn const_max(a: usize, b: usize) -> usize {
    if a > b {
        a
    } else {
        b
    }
}

/// The maximum number of bytes allocated for cmsg data
///
/// This should be enough for UDP_SEGMENT + IP_TOS + IP_PKTINFO. It may need to be increased
/// to allow for future control messages.
pub const MAX_LEN: usize = {
    let tos_v4_size = features::tos_v4::CMSG_SPACE;
    let tos_v6_size = features::tos_v6::CMSG_SPACE;

    let tos_size = const_max(tos_v4_size, tos_v6_size);

    let gso_size = features::gso::CMSG_SPACE;
    let gro_size = features::gro::CMSG_SPACE;

    let segment_offload_size = const_max(gso_size, gro_size);

    // rather than taking the max, we add these in case the OS gives us both
    let pktinfo_size = features::pktinfo_v4::CMSG_SPACE + features::pktinfo_v6::CMSG_SPACE;

    let rxq_ovfl_size = features::rxq_ovfl::CMSG_SPACE;

    // This is currently needed due to how we detect if CMSG data has been written or not.
    //
    // TODO remove this once we split the `reset` traits into TX and RX types
    let padding = size_of::<cmsghdr>();

    tos_size + segment_offload_size + pktinfo_size + rxq_ovfl_size + padding
};

#[cfg(test)]
mod tests_ {}
