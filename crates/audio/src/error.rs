// SPDX-FileCopyrightText: 2025-2026 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: MPL-2.0

use windows_core::{HRESULT, WIN32_ERROR};

use crate::windows::Win32::{
    AUDCLNT_E_DEVICE_INVALIDATED as AUDCLNT_E_DEVICE_INVALIDATED_WIN32, CO_E_NOTINITIALIZED,
    ERROR_DEVICE_REMOVED as ERROR_DEVICE_REMOVED_WIN32,
    ERROR_NO_SUCH_DEVICE_INTERFACE as ERROR_NO_SUCH_DEVICE_INTERFACE_WIN32,
};

#[derive(thiserror::Error, Debug, Clone)]
pub enum Error {
    #[error("COM has not been initialized for use by the current thread")]
    ComNotInitialized,
    #[error("invalid volume level (expected value between 0.0 and 1.0, found {0})")]
    InvalidVolume(f32),
    #[error(
        "the audio device has been unplugged, or the audio hardware has been \
        reconfigured, disabled, remove, or otherwise made unavailable for use"
    )]
    DeviceInvalidated,
    #[error("the device ID does not identify an audio device that is in this system")]
    DeviceNotFound,
    #[error("the ID is not a device ID")]
    InvalidDeviceID,
    #[error("unknown windows error ({0})")]
    UnknownWindows(windows_core::Error),
}

impl From<windows_core::Error> for Error {
    fn from(error: windows_core::Error) -> Self {
        const ERROR_DEVICE_REMOVED: HRESULT =
            WIN32_ERROR(ERROR_DEVICE_REMOVED_WIN32 as _).to_hresult();
        const ERROR_NO_SUCH_DEVICE_INTERFACE: HRESULT =
            WIN32_ERROR(ERROR_NO_SUCH_DEVICE_INTERFACE_WIN32 as _).to_hresult();
        const AUDCLNT_E_DEVICE_INVALIDATED: HRESULT =
            WIN32_ERROR(AUDCLNT_E_DEVICE_INVALIDATED_WIN32 as _).to_hresult();

        match error.code() {
            AUDCLNT_E_DEVICE_INVALIDATED => Self::DeviceInvalidated,
            ERROR_DEVICE_REMOVED => Self::DeviceInvalidated,
            CO_E_NOTINITIALIZED => Self::ComNotInitialized,
            ERROR_NO_SUCH_DEVICE_INTERFACE => Self::DeviceNotFound,
            _ => Self::UnknownWindows(error),
        }
    }
}
