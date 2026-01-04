// SPDX-FileCopyrightText: 2025 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use audio::Device;
use dioxus::Result;

#[inline]
pub fn toggle_mute(device: &Device) -> Result<()> {
    let muted = device.get_mute()?;
    device.set_mute(!muted)?;
    Ok(())
}
