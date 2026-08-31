// SPDX-FileCopyrightText: 2025 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: MPL-2.0

use windows::Win32::Media::Audio::{
    DEVICE_STATE, DEVICE_STATE_ACTIVE, DEVICE_STATE_DISABLED, DEVICE_STATE_NOTPRESENT,
    DEVICE_STATE_UNPLUGGED,
};

#[derive(Clone, Copy, PartialEq, Eq)]
/// Device state.
pub enum State {
    /// Device is active.
    Active,
    /// Device exists, but nothing is currently plugged to it.
    Unplugged,
    /// Device exists, but is disabled.
    Disabled,
    /// Device has been removed from the system.
    NotPresent,
}

impl From<DEVICE_STATE> for State {
    fn from(value: DEVICE_STATE) -> Self {
        match value {
            DEVICE_STATE_ACTIVE => Self::Active,
            DEVICE_STATE_UNPLUGGED => Self::Unplugged,
            DEVICE_STATE_NOTPRESENT => Self::NotPresent,
            DEVICE_STATE_DISABLED => Self::Disabled,
            _ => unreachable!("There are no other device state."),
        }
    }
}

impl From<State> for DEVICE_STATE {
    fn from(value: State) -> Self {
        match value {
            State::Active => DEVICE_STATE_ACTIVE,
            State::Unplugged => DEVICE_STATE_UNPLUGGED,
            State::NotPresent => DEVICE_STATE_NOTPRESENT,
            State::Disabled => DEVICE_STATE_DISABLED,
        }
    }
}
