// Copyright Amazon.com, Inc. or its affiliates. All Rights Reserved.
// SPDX-License-Identifier: Apache-2.0

use super::c_int;

#[cfg(s2n_quic_platform_rxq_ovfl)]
mod rxq_ovfl_enabled {
    use super::*;
    use libc::{SOL_SOCKET, SO_RXQ_OVFL};

    pub const LEVEL: Option<c_int> = Some(SOL_SOCKET as _);
    pub const TYPE: Option<c_int> = Some(SO_RXQ_OVFL as _);
    pub const SOCKOPT: Option<(c_int, c_int)> = Some((SOL_SOCKET as _, SO_RXQ_OVFL as _));
    pub const CMSG_SPACE: usize = crate::message::cmsg::size_of_cmsg::<super::Cmsg>();

    #[inline]
    pub const fn is_match(level: c_int, ty: c_int) -> bool {
        level == SOL_SOCKET as c_int && ty == SO_RXQ_OVFL as c_int
    }
}

#[cfg(any(not(s2n_quic_platform_rxq_ovfl), test))]
mod rxq_ovfl_disabled {
    #![cfg_attr(test, allow(dead_code))]
    use super::*;

    pub const LEVEL: Option<c_int> = None;
    pub const TYPE: Option<c_int> = None;
    pub const SOCKOPT: Option<(c_int, c_int)> = None;
    pub const CMSG_SPACE: usize = 0;

    #[inline]
    pub const fn is_match(level: c_int, ty: c_int) -> bool {
        let _ = level;
        let _ = ty;
        false
    }
}

mod rxq_ovfl_impl {
    #[cfg(not(s2n_quic_platform_rxq_ovfl))]
    pub use super::rxq_ovfl_disabled::*;
    #[cfg(s2n_quic_platform_rxq_ovfl)]
    pub use super::rxq_ovfl_enabled::*;
}

pub use rxq_ovfl_impl::*;
pub type Cmsg = u32;
pub const IS_SUPPORTED: bool = cfg!(s2n_quic_platform_rxq_ovfl);
