// SPDX-FileCopyrightText: 2025 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use dioxus::Result;
use dioxus::desktop::tao::platform::windows::IconExtWindows;
use dioxus::desktop::tao::window::Icon;
use dioxus::desktop::wry::dpi::PhysicalSize;
use windows::Win32::UI::WindowsAndMessaging::{
    GetSystemMetrics, SM_CXICON, SM_CXSMICON, SYSTEM_METRICS_INDEX,
};

use audio::Device;

#[inline]
pub fn toggle_mute(device: &Device) -> Result<()> {
    let muted = device.get_mute()?;
    device.set_mute(!muted)?;
    Ok(())
}

/// Get icon from the resource's icons using the system's expected dimensions.
fn get_icon(metric: SYSTEM_METRICS_INDEX) -> Icon {
    let size = unsafe { GetSystemMetrics(metric) } as u32;
    assert!(size != 0, "Could not get system metrics.");
    let size = PhysicalSize::new(size, size);
    Icon::from_resource(1, Some(size)).unwrap()
}

#[inline(always)]
pub fn get_small_icon() -> Icon {
    get_icon(SM_CXSMICON)
}

#[inline(always)]
pub fn get_big_icon() -> Icon {
    get_icon(SM_CXICON)
}
