// SPDX-FileCopyrightText: 2025 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: MPL-2.0

use windows_core::implement;

use crate::WindowsResult;
use crate::windows::Win32::{
    AUDIO_VOLUME_NOTIFICATION_DATA, E_INVALIDARG, IAudioEndpointVolumeCallback,
    IAudioEndpointVolumeCallback_Impl,
};

/// Device audio data.
pub struct AudioData {
    /// Is the device muted?
    pub muted: bool,
}

impl From<AUDIO_VOLUME_NOTIFICATION_DATA> for AudioData {
    fn from(value: AUDIO_VOLUME_NOTIFICATION_DATA) -> Self {
        Self {
            muted: value.bMuted.into(),
        }
    }
}

/// Implementations must follow these rules:
///     - The methods must be **non-blocking**. They should never wait on a synchronization
///     primitives.
///     - The methods should never call their respective subscribe and unsubscribe method.
///     - The methods should never drop the final reference of their respective API.
///
/// The implemented methods will be called by **different threads** than the one used for the
/// subscription.
#[allow(unused_variables)]
pub trait AudioControllerEvents: Send + Sync {
    /// Called when the device audio state changes.
    ///
    /// See [`AudioData`].
    fn on_new_data(&self, new_data: AudioData) -> WindowsResult<()> {
        Ok(())
    }
}

#[implement(IAudioEndpointVolumeCallback)]
pub(super) struct AudioControllerEventProxy<Subscriber>(Subscriber)
where
    Subscriber: AudioControllerEvents + 'static;

impl<Subscriber> IAudioEndpointVolumeCallback_Impl for AudioControllerEventProxy_Impl<Subscriber>
where
    Subscriber: AudioControllerEvents + 'static,
{
    fn OnNotify(&self, data: *mut AUDIO_VOLUME_NOTIFICATION_DATA) -> windows_core::Result<()> {
        if data.is_null() {
            return Err(E_INVALIDARG.into());
        }

        // SAFETY:
        // - We checked that the pointer was not null
        // - AUDIO_VOLUME_NOTIFICATION_DATA C struct is not packed
        // - We expect Windows to properly initialize the struct it gave us
        let new_data = unsafe { data.read() };

        self.0.on_new_data(new_data.into())
    }
}

impl<Subscriber> AudioControllerEventProxy<Subscriber>
where
    Subscriber: AudioControllerEvents + 'static,
{
    pub(super) fn new(subscriber: Subscriber) -> Self {
        Self(subscriber)
    }
}
