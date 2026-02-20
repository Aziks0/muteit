// SPDX-FileCopyrightText: 2025 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

#![cfg_attr(feature = "bundle", windows_subsystem = "windows")]

mod autostart;
mod config;
mod device;
mod device_selector;
mod hotkey_binding;
mod tray_icon;
mod utils;

use std::env::args;

use dioxus::core::throw_error;
use dioxus::desktop::tao::platform::windows::WindowBuilderExtWindows;
use dioxus::desktop::tao::window::UserAttentionType;
use dioxus::desktop::{
    Config, HotKeyState, LogicalSize, ShortcutHandle, WindowBuilder, WindowCloseBehaviour, window,
};
use dioxus::prelude::*;

use alone::is_alone;
use audio::interface::audio_controller::AudioController;

use crate::autostart::AutoStart;
use crate::config::{AppConfig, DeviceInformation, use_config};
use crate::device::use_setup_device;
use crate::device_selector::DeviceSelector;
use crate::hotkey_binding::HotkeyBinding;
use crate::tray_icon::use_setup_tray;
use crate::utils::toggle_mute;

// Bake tailwind into the binary only in release mode. We want to keep using dx `Asset` in debug to
// get hotreloading.
#[cfg(not(feature = "bundle"))]
#[component]
fn Tailwind() -> Element {
    rsx! {
        document::Stylesheet { href: asset!("/assets/tailwind.css") }
    }
}
#[cfg(feature = "bundle")]
#[component]
fn Tailwind() -> Element {
    const TAILWIND: &str = include_str!(concat!(env!("OUT_DIR"), "/tailwind.css",));

    rsx! {
        document::Style { {TAILWIND} }
    }
}

/// The main application component.
#[component]
fn App() -> Element {
    let mut config = use_config()?;
    let information = config.information.peek().clone();

    let (enumerator, mut controller) = use_setup_device(config.information)?;

    use_setup_tray(controller)?;

    let mut shortcut_handle = use_signal::<Option<ShortcutHandle>>(|| None);

    use_effect(move || {
        if let Some(ref shortcut) = *shortcut_handle.peek() {
            shortcut.remove();
        }

        match *config.hotkey.read() {
            Some(ref hotkey) => {
                match window().create_shortcut(hotkey.hotkey, move |state| {
                    if state == HotKeyState::Pressed
                        && let Some(ref controller) = *controller.peek()
                    {
                        if let Err(err) = toggle_mute(controller) {
                            throw_error(err);
                        }
                    }
                }) {
                    Ok(shortcut) => shortcut_handle.set(Some(shortcut)),
                    Err(err) => throw_error(err),
                };
            }
            None => shortcut_handle.set(None),
        };
    });

    rsx! {
        div { class: "flex h-screen items-center justify-center bg-neutral-800 text-white pb-1",
            div { class: "flex flex-col w-5/6 gap-2",
                DeviceSelector {
                    on_select: move |i: Option<u32>| {
                        let new_controller = i
                            .map(|i| AudioController::try_from(&enumerator.peek()[i as usize]))
                            .transpose()?;
                        controller.set(new_controller);

                        let information = i
                            .map(|i| {
                                let information = &enumerator.peek()[i as usize];
                                DeviceInformation {
                                    name: information.to_string(),
                                    id: information.get_id(),
                                }
                            });
                        config.information.set(information);

                        Ok(())
                    },
                    default: information,
                    enumerator,
                }
                HotkeyBinding { hotkey: config.hotkey }
                AutoStart {}
            }
        }
    }
}

/// A version of the `App` component wrapped in an `ErrorBoundary`.
///
/// Everything that can fail should be put in [`App`] to ensure we can display a user-friendly error
/// message.
#[component]
fn InfallibleApp() -> Element {
    #[cfg(not(feature = "bundle"))]
    dioxus::desktop::use_global_shortcut("Ctrl+Shift+I", |state| {
        if state == dioxus::desktop::HotKeyState::Pressed {
            dioxus::desktop::window().devtool();
        }
    })
    .unwrap();

    rsx! {
        Tailwind {}
        ErrorBoundary {
            handle_error: |error_context: ErrorContext| {
                window().set_close_behavior(WindowCloseBehaviour::WindowCloses);
                window().set_visible(true);
                window().request_user_attention(Some(UserAttentionType::Critical));

                let error = error_context
                    .error()
                    .expect("error should be present inside the handler");

                rsx! {
                    p { "Oops, an error has been encountered." }
                    p { "{error:#}" }
                }
            },
            App {}
        }
    }
}

fn main() {
    if !is_alone!(env!("APP_ID")).expect("CreateMutexW to work") {
        return;
    }

    // TODO: show an error message using `MessageBoxW`?
    let config_dir = AppConfig::get_directory().unwrap();
    let is_hidden = args().rev().find(|arg| arg == "--hidden").is_some();

    let small_icon = utils::get_small_icon();
    let big_icon = utils::get_big_icon();

    LaunchBuilder::desktop()
        .with_cfg(
            Config::new()
                .with_window(
                    WindowBuilder::new()
                        .with_drag_and_drop(false)
                        .with_title("muteit!")
                        .with_inner_size(LogicalSize::new(800, 230))
                        .with_window_icon(Some(small_icon))
                        .with_taskbar_icon(Some(big_icon))
                        .with_visible(!is_hidden),
                )
                .with_disable_context_menu(true)
                .with_disable_drag_drop_handler(true)
                .with_menu(None)
                .with_data_directory(config_dir)
                .with_close_behaviour(WindowCloseBehaviour::WindowHides),
        )
        .launch(InfallibleApp);
}
