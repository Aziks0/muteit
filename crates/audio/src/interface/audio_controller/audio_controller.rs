// SPDX-FileCopyrightText: 2025-2026 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: MPL-2.0

use crate::windows::Win32::{IAudioEndpointVolume, IAudioEndpointVolumeCallback};

use crate::enumerator::{DeviceInformation, get_interface_by_id};
use crate::interface::AudioInterface;
use crate::interface::audio_controller::{AudioControllerEventProxy, AudioControllerEvents};
use crate::{Error, Result};

#[derive(Clone)]
pub struct SubscriberId(IAudioEndpointVolumeCallback);

/// A wrapper around the [`IAudioEndpointVolume`] interface to provide a more convenient API for
/// interacting with muting state of audio devices.
///
/// [`IAudioEndpointVolume`]:
///     https://learn.microsoft.com/en-us/windows/win32/api/endpointvolume/nn-endpointvolume-iaudioendpointvolume
pub struct AudioController {
    interface: IAudioEndpointVolume,
    subscribers: Vec<IAudioEndpointVolumeCallback>,
}

impl AudioController {
    /// Get whether the device is muted or not.
    pub fn get_mute(&self) -> Result<bool> {
        let result = unsafe { self.interface.GetMute()? }.as_bool();
        Ok(result)
    }

    /// Set whether the device is muted or not.
    pub fn set_mute(&self, mute: bool) -> Result<()> {
        unsafe { self.interface.SetMute(mute, std::ptr::null()).ok()? }
        Ok(())
    }

    /// Subscribe to event notifications.
    ///
    /// When `Self` gets dropped, the subscriber will be automatically unsubscribed. The returned
    /// value can be used to [unsubscribe](Self::unsubscribe_event_notifications) before that.
    ///
    /// See [`AudioControllerEvents`] for information on the implementation of `Subscriber`.
    pub fn subscribe_event_notifications<Subscriber>(
        &mut self,
        subscriber: Subscriber,
    ) -> Result<SubscriberId>
    where
        Subscriber: AudioControllerEvents + 'static,
    {
        let proxy = AudioControllerEventProxy::new(subscriber);
        let callback: IAudioEndpointVolumeCallback = proxy.into();
        unsafe { self.interface.RegisterControlChangeNotify(&callback).ok()? }

        self.subscribers.push(callback.clone());

        Ok(SubscriberId(callback.clone()))
    }

    /// Unsubscribe from event notifications.
    pub fn unsubscribe_event_notifications(&mut self, id: SubscriberId) -> Result<()> {
        unsafe { self.interface.UnregisterControlChangeNotify(&id.0).ok()? }

        let index = self
            .subscribers
            .iter()
            .position(|s| *s == id.0)
            .expect("id should be in the list of subscribers");
        self.subscribers.swap_remove(index);

        Ok(())
    }
}

impl Drop for AudioController {
    fn drop(&mut self) {
        for subscriber in &self.subscribers {
            let _ = unsafe { self.interface.UnregisterControlChangeNotify(subscriber) };
        }
    }
}

impl TryFrom<&DeviceInformation> for AudioController {
    type Error = Error;

    fn try_from(value: &DeviceInformation) -> std::result::Result<Self, Self::Error> {
        get_interface_by_id::<_, Self>(&value.enumerator, value.id)
    }
}

impl AudioInterface for AudioController {
    type Interface = IAudioEndpointVolume;

    fn new(interface: Self::Interface) -> Self {
        Self {
            interface,
            subscribers: Vec::new(),
        }
    }
}
