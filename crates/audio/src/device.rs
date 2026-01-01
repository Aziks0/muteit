// SPDX-FileCopyrightText: 2025 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: MPL-2.0

use windows::Win32::Media::Audio::Endpoints::{IAudioEndpointVolume, IAudioEndpointVolumeCallback};

use crate::Result;
use crate::events::{AudioEventProxy, AudioEvents};

#[derive(Clone)]
pub struct SubscriberId(IAudioEndpointVolumeCallback);

/// A wrapper around the [`IAudioEndpointVolume`] interface to provide a more convenient API for
/// interacting with muting state of audio devices.
///
/// [`IAudioEndpointVolume`]:
///     https://learn.microsoft.com/en-us/windows/win32/api/endpointvolume/nn-endpointvolume-iaudioendpointvolume
pub struct Device {
    interface: IAudioEndpointVolume,
    subscribers: Vec<IAudioEndpointVolumeCallback>,
}

impl Device {
    pub(crate) fn new(interface: IAudioEndpointVolume) -> Self {
        Self {
            interface,
            subscribers: Vec::new(),
        }
    }

    /// Get whether the device is muted or not.
    pub fn get_mute(&self) -> Result<bool> {
        let result = unsafe { self.interface.GetMute()? }.as_bool();
        Ok(result)
    }

    /// Set whether the device is muted or not.
    pub fn set_mute(&self, mute: bool) -> Result<()> {
        unsafe { self.interface.SetMute(mute, std::ptr::null())? }
        Ok(())
    }

    /// Subscribe to event notifications.
    ///
    /// When `Self` gets dropped, the subscriber will be automatically unsubscribed. The returned
    /// value can be used to [unsubscribe](Self::unsubscribe_event_notifications) before that.
    ///
    /// See [`AudioEvents`] for information on the implementation of `Subscriber`.
    pub fn subscribe_event_notifications<Subscriber>(
        &mut self,
        subscriber: Subscriber,
    ) -> Result<SubscriberId>
    where
        Subscriber: AudioEvents + 'static,
    {
        let mute = unsafe { self.interface.GetMute()? };
        let proxy = AudioEventProxy::new(mute, subscriber);
        let callback: IAudioEndpointVolumeCallback = proxy.into();
        unsafe { self.interface.RegisterControlChangeNotify(&callback)? }

        self.subscribers.push(callback.clone());

        Ok(SubscriberId(callback.clone()))
    }

    /// Unsubscribe from event notifications.
    pub fn unsubscribe_event_notifications(&mut self, id: SubscriberId) -> Result<()> {
        unsafe { self.interface.UnregisterControlChangeNotify(&id.0)? }

        let index = self
            .subscribers
            .iter()
            .position(|s| *s == id.0)
            .expect("id should be in the list of subscribers");
        self.subscribers.swap_remove(index);

        Ok(())
    }
}

impl Drop for Device {
    fn drop(&mut self) {
        for subscriber in &self.subscribers {
            let _ = unsafe { self.interface.UnregisterControlChangeNotify(subscriber) };
        }
    }
}
