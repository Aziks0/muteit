// SPDX-FileCopyrightText: 2026 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: MPL-2.0

use crate::windows::DeviceState;
use crate::windows::Win32::{
    DEVICE_STATE_ACTIVE, DEVICE_STATE_DISABLED, DEVICE_STATE_NOTPRESENT, DEVICE_STATE_UNPLUGGED,
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

impl From<DeviceState> for State {
    fn from(value: DeviceState) -> Self {
        let value = value as _;
        match value {
            DEVICE_STATE_ACTIVE => Self::Active,
            DEVICE_STATE_UNPLUGGED => Self::Unplugged,
            DEVICE_STATE_NOTPRESENT => Self::NotPresent,
            DEVICE_STATE_DISABLED => Self::Disabled,
            _ => unreachable!("There are no other device state."),
        }
    }
}

impl From<State> for DeviceState {
    fn from(value: State) -> Self {
        let res = match value {
            State::Active => DEVICE_STATE_ACTIVE,
            State::Unplugged => DEVICE_STATE_UNPLUGGED,
            State::NotPresent => DEVICE_STATE_NOTPRESENT,
            State::Disabled => DEVICE_STATE_DISABLED,
        };
        res as _
    }
}
