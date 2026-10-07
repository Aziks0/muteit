// SPDX-FileCopyrightText: 2026 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::LazyLock;

use anyhow::Context;
use iced::window::settings::PlatformSpecific;
use iced::window::settings::platform::IconType;
use iced::{Size, Task, window};

use audio::interface::audio_controller::AudioController;

use crate::windows::Win32::{GetSystemMetrics, SM_CXICON, SM_CXSMICON};

pub type DeviceEnumerator<F> = Rc<RefCell<audio::enumerator::DeviceEnumerator<F>>>;

#[inline]
pub fn toggle_mute(controller: &Option<AudioController>) -> anyhow::Result<()> {
    if let Some(controller) = controller {
        // TODO: replace by `try` block when stabilized.
        // https://doc.rust-lang.org/std/ops/trait.Try.html
        let inner = || -> anyhow::Result<()> {
            let muted = controller.get_mute()?;
            controller.set_mute(!muted)?;

            Ok(())
        };
        inner().context("Toggling mute state.")?;
    }

    Ok(())
}

/// Get icon from the resource's icons using the system's expected dimensions.
fn get_icon(metric: i32) -> IconType {
    let size = unsafe { GetSystemMetrics(metric) } as u32;
    assert!(size != 0, "Could not get system metrics.");

    let size = Size::new(size, size);

    IconType::Resource {
        ordinal: 1,
        size: Some(size),
    }
}

#[inline(always)]
fn get_small_icon() -> IconType {
    get_icon(SM_CXSMICON)
}

#[inline(always)]
fn get_big_icon() -> IconType {
    get_icon(SM_CXICON)
}

pub fn open_window<T>() -> (window::Id, Task<T>)
where
    T: Send + 'static,
{
    // Static so we don't call `utils::get_[size]_icon` everytime.
    static DEFAULT_WINDOW_SETTINGS: LazyLock<window::Settings> =
        LazyLock::new(|| window::Settings {
            size: Size::new(800., 230.),
            platform_specific: PlatformSpecific {
                drag_and_drop: false,
                icon: Some(get_small_icon()),
                taskbar_icon: Some(get_big_icon()),
                ..Default::default()
            },
            ..Default::default()
        });

    let (id, task) = window::open(DEFAULT_WINDOW_SETTINGS.clone());

    (id, task.discard())
}

#[cold]
pub fn leak_error(error: impl Into<anyhow::Error>) -> &'static anyhow::Error {
    let error = Box::new(error.into());
    Box::leak(error)
}
