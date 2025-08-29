// SPDX-FileCopyrightText: 2025 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: MPL-2.0

use windows::Win32::Media::Audio::ISimpleAudioVolume;

use crate::Result;

/// A wrapper around the [`ISimpleAudioVolume`] interface to provide a more convenient API for
/// interacting with audio devices.
///
/// [`ISimpleAudioVolume`]:
///     https://learn.microsoft.com/en-us/windows/win32/api/audioclient/nn-audioclient-isimpleaudiovolume
pub struct Device(ISimpleAudioVolume);

impl Device {
    pub(crate) fn new(interface: ISimpleAudioVolume) -> Self {
        Self(interface)
    }

    /// Get the current volume level of the device as a float between 0 and 1.0.
    pub fn get_volume(&self) -> Result<f32> {
        let result = unsafe { self.0.GetMasterVolume()? };
        Ok(result)
    }

    /// Set the current volume level of the device as a float between 0 and 1.0.
    pub fn set_volume(&self, volume: f32) -> Result<()> {
        unsafe { self.0.SetMasterVolume(volume, std::ptr::null())? }
        Ok(())
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
