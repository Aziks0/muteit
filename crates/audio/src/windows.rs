// SPDX-FileCopyrightText: 2026 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: CC0-1.0

#![allow(
    non_camel_case_types,
    non_snake_case,
    non_upper_case_globals,
    clippy::upper_case_acronyms,
    clippy::missing_transmute_annotations,
    clippy::useless_transmute
)]

include!(concat!(env!("OUT_DIR"), "/windows.rs"));

// FIXME: Those can't be filtered out, so this removes clippy warning about dead code.
const _: Windows::Win32::EDataFlow = Windows::Win32::EDataFlow_enum_count;
const _: Windows::Win32::ERole = Windows::Win32::ERole_enum_count;

pub use Windows::*;

pub type DeviceState = u32;
