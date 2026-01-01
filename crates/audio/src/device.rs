// SPDX-FileCopyrightText: 2025 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: MPL-2.0

use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;

use crate::Result;

/// A wrapper around the [`IAudioEndpointVolume`] interface to provide a more convenient API for
/// interacting with muting state of audio devices.
///
/// [`IAudioEndpointVolume`]:
///     https://learn.microsoft.com/en-us/windows/win32/api/endpointvolume/nn-endpointvolume-iaudioendpointvolume
pub struct Device(IAudioEndpointVolume);

impl Device {
    pub(crate) fn new(interface: IAudioEndpointVolume) -> Self {
        Self(interface)
    }

    /// Get whether the device is muted or not.
    pub fn get_mute(&self) -> Result<bool> {
        let result = unsafe { self.0.GetMute()? }.as_bool();
        Ok(result)
    }

    /// Set whether the device is muted or not.
    pub fn set_mute(&self, mute: bool) -> Result<()> {
        unsafe { self.0.SetMute(mute, std::ptr::null())? }
        Ok(())
    }
}
