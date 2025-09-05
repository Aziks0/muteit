// SPDX-FileCopyrightText: 2025 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: MPL-2.0

use windows::Win32::Foundation::CO_E_NOTINITIALIZED;
use windows::Win32::Media::Audio::AUDCLNT_E_DEVICE_INVALIDATED;

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
    #[error("unknown windows error ({0})")]
    UnknownWindows(windows::core::Error),
}

impl From<windows::core::Error> for Error {
    fn from(error: windows::core::Error) -> Self {
        match error.code() {
            AUDCLNT_E_DEVICE_INVALIDATED => Self::DeviceInvalidated,
            CO_E_NOTINITIALIZED => Self::ComNotInitialized,
            _ => Self::UnknownWindows(error),
        }
    }
}
