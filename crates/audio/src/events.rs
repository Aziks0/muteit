// SPDX-FileCopyrightText: 2025 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: MPL-2.0

use std::cell::UnsafeCell;
use std::ptr;

use windows::Win32::Foundation::E_INVALIDARG;
use windows::Win32::Media::Audio::AUDIO_VOLUME_NOTIFICATION_DATA;
use windows::Win32::Media::Audio::Endpoints::{
    IAudioEndpointVolumeCallback, IAudioEndpointVolumeCallback_Impl,
};
use windows::core::implement;

// Re-export for `AudioEvents`.
pub use windows::core::Error as WindowsError;
pub use windows::core::Result as WindowsResult;

/// Implementations must follow these rules:
///     - The methods must be **non-blocking**. They should never wait on a synchronization
///     primitives.
///     - The methods should never call their respective subscribe and unsubscribe method.
///     - The methods should never drop the final reference of their respective API.
///
/// The implemented methods will be called by **different threads** than the one used for the
/// subscription.
#[allow(unused_variables)]
pub trait AudioEvents: Send + Sync {
    /// Called when mute state changes.
    fn on_mute_changed(&mut self, new_mute: bool) -> WindowsResult<()> {
        Ok(())
    }
}

struct InnerAudioEventProxy<Subscriber>
where
    Subscriber: AudioEvents + 'static,
{
    mute: windows::core::BOOL,
    subscriber: Subscriber,
}

#[implement(IAudioEndpointVolumeCallback)]
pub(crate) struct AudioEventProxy<Subscriber>(UnsafeCell<InnerAudioEventProxy<Subscriber>>)
where
    Subscriber: AudioEvents + 'static;

impl<Subscriber> IAudioEndpointVolumeCallback_Impl for AudioEventProxy_Impl<Subscriber>
where
    Subscriber: AudioEvents + 'static,
{
    fn OnNotify(&self, data: *mut AUDIO_VOLUME_NOTIFICATION_DATA) -> windows::core::Result<()> {
        // NOTE: Calls to this callback are put in a queue by Windows. They are **not** called
        // concurrently. So no data race possible with `self`.

        // SAFETY: The `UnsafeCell` got initialized by rust.
        let mut this = unsafe { ptr::read(self.0.get()) };

        if data.is_null() {
            return Err(E_INVALIDARG.into());
        }

        // SAFETY:
        // - We checked that the pointer was not null
        // - AUDIO_VOLUME_NOTIFICATION_DATA C struct is not packed
        // - We expect Windows to properly initialize the struct it gave us
        let data = unsafe { data.read() };

        let result = if this.mute != data.bMuted {
            this.mute = data.bMuted;
            this.subscriber.on_mute_changed(data.bMuted.as_bool())
        } else {
            Ok(())
        };

        // SAFETY: We write to the same memory location we previously read from.
        unsafe { ptr::write(self.0.get(), this) };

        result
    }
}

impl<Subscriber> AudioEventProxy<Subscriber>
where
    Subscriber: AudioEvents + 'static,
{
    pub(crate) fn new(mute: windows::core::BOOL, subscriber: Subscriber) -> Self {
        Self(UnsafeCell::new(InnerAudioEventProxy { mute, subscriber }))
    }
}
