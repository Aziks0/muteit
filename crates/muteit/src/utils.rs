// SPDX-FileCopyrightText: 2025 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use dioxus::Result;

use audio::Device;

#[inline]
pub fn toggle_mute(device: &Device) -> Result<()> {
    let muted = device.get_mute()?;
    device.set_mute(!muted)?;
    Ok(())
}
