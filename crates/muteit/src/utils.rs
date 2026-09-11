// SPDX-FileCopyrightText: 2025 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use iced::Size;
use iced::window::settings::platform::IconType;
use windows::Win32::UI::WindowsAndMessaging::{
    GetSystemMetrics, SM_CXICON, SM_CXSMICON, SYSTEM_METRICS_INDEX,
};

use audio::interface::audio_controller::AudioController;

#[inline]
pub fn toggle_mute(controller: &AudioController) -> audio::Result<()> {
    let muted = controller.get_mute()?;
    controller.set_mute(!muted)?;
    Ok(())
}

/// Get icon from the resource's icons using the system's expected dimensions.
fn get_icon(metric: SYSTEM_METRICS_INDEX) -> IconType {
    let size = unsafe { GetSystemMetrics(metric) } as u32;
    assert!(size != 0, "Could not get system metrics.");

    let size = Size::new(size, size);

    IconType::Resource {
        ordinal: 1,
        size: Some(size),
    }
}

#[inline(always)]
pub fn get_small_icon() -> IconType {
    get_icon(SM_CXSMICON)
}

#[inline(always)]
pub fn get_big_icon() -> IconType {
    get_icon(SM_CXICON)
}

#[cold]
pub fn leak_error(error: impl Into<anyhow::Error>) -> &'static anyhow::Error {
    let error = Box::new(error.into());
    Box::leak(error)
}

/// Return an `Action::Error` if there is an error.
///
/// # Examples
///
/// ```no_run
/// use std::fs::File;
///
/// # pub enum Message {}
/// #
/// pub enum Action {
///     None,
///     Error(anyhow::Error),
/// }
///
/// pub fn update(message: Message) -> Action {
///     let file = maybe_bail!(File::create("foo.txt"));
///
///     // Do things...
///
///     Action::None
/// }
/// ```
#[macro_export]
macro_rules! maybe_bail {
    ($result:expr) => {
        match $result {
            Ok(value) => value,
            Err(err) => {
                std::hint::cold_path();
                return Action::Error(err.into());
            }
        }
    };
}
