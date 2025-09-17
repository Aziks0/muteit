// SPDX-FileCopyrightText: 2025 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

mod config;
mod device_selector;
mod hotkey_binding;

use dioxus::core::throw_error;
use dioxus::desktop::tao::platform::windows::WindowBuilderExtWindows;
use dioxus::desktop::tao::window::UserAttentionType;
use dioxus::desktop::{
    Config, HotKeyState, ShortcutHandle, WindowBuilder, WindowCloseBehaviour, window,
};
use dioxus::prelude::*;

use audio::{Device, DeviceEnumerator};

use crate::config::{DeviceInformation, use_config};
use crate::device_selector::DeviceSelector;
use crate::hotkey_binding::HotkeyBinding;

const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

/// The main application component.
#[component]
fn App() -> Element {
    let mut config = use_config()?;
    let information = config.information.peek().clone();

    // We don't care about returning early, blah blah blah, hook, whatever. If we can't get a hold
    // to the enumerator, something is **VERY** wrong, so just fail and ask the user to restart the
    // app.
    let enumerator =
        use_hook(|| DeviceEnumerator::try_new().map(|enumerator| Signal::new(enumerator)))?;

    let mut device = use_signal(|| {
        config
            .information
            .peek()
            .as_ref()
            .map(|information| enumerator.peek().get_device(information.id.clone()))
            .transpose()
    });

    use_effect(move || {
        if let Err(ref err) = *device.read() {
            throw_error(err.clone());
        }
    });

    let mut shortcut_handle = use_signal::<Option<ShortcutHandle>>(|| None);

    use_effect(move || {
        if let Some(ref shortcut) = *shortcut_handle.peek() {
            shortcut.remove();
        }

        match *config.hotkey.read() {
            Some(ref hotkey) => {
                let res = window()
                    .create_shortcut(hotkey.hotkey, move |state| {
                        fn toggle_mute(device: &Device) -> dioxus::Result<()> {
                            let muted = device.get_mute().context("Getting device mute state.")?;
                            device
                                .set_mute(!muted)
                                .context("Setting device mute state.")?;

                            Ok(())
                        }

                        if state == HotKeyState::Pressed {
                            if let Ok(Some(ref device)) = *device.peek() {
                                if let Err(err) = toggle_mute(device) {
                                    throw_error(err);
                                }
                            }
                        }
                    })
                    .context("Creating shortcut.");

                match res {
                    Ok(shortcut) => shortcut_handle.set(Some(shortcut)),
                    Err(err) => throw_error(err),
                };
            }
            None => shortcut_handle.set(None),
        };
    });

    rsx! {
        div { class: "flex h-screen items-center justify-center bg-neutral-800 text-white",
            div { class: "flex flex-col w-5/6 gap-2",
                DeviceSelector {
                    on_select: move |i: Option<u32>| {
                        let new_device = i
                            .map(|i| Device::try_from(&enumerator.peek()[i as usize]))
                            .transpose();
                        device.set(new_device);

                        let information = i
                            .map(|i| {
                                let information = &enumerator.peek()[i as usize];
                                DeviceInformation {
                                    name: information.to_string(),
                                    id: information.get_id(),
                                }
                            });
                        config.information.set(information);
                    },
                    default: information,
                    enumerator,
                }
                HotkeyBinding { hotkey: config.hotkey }
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
    #[cfg(debug_assertions)]
    dioxus::desktop::use_global_shortcut("Ctrl+Shift+I", |state| {
        if state == dioxus::desktop::HotKeyState::Pressed {
            dioxus::desktop::window().devtool();
        }
    })
    .unwrap();

    rsx! {
        document::Stylesheet { href: TAILWIND_CSS }
        ErrorBoundary {
            handle_error: |errors: ErrorContext| {
                window().set_close_behavior(WindowCloseBehaviour::WindowCloses);
                window().set_visible(true);
                window().request_user_attention(Some(UserAttentionType::Critical));

                rsx! {
                    p { "Oops, an error has been encountered." }
                    for error in errors.errors() {
                        p { "{error}" }
                    }
                }
            },
            App {}
        }
    }
}

fn main() {
    LaunchBuilder::desktop()
        .with_cfg(
            Config::new()
                .with_window(
                    WindowBuilder::new()
                        .with_drag_and_drop(false)
                        .with_title("muteit!"),
                )
                .with_disable_context_menu(true)
                .with_disable_drag_drop_handler(true)
                .with_menu(None),
        )
        .launch(InfallibleApp);
}
