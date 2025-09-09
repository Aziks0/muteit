// SPDX-FileCopyrightText: 2025 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

mod config;

use dioxus::desktop::tao::platform::windows::WindowBuilderExtWindows;
use dioxus::desktop::tao::window::UserAttentionType;
use dioxus::desktop::{Config, WindowBuilder, WindowCloseBehaviour, window};
use dioxus::prelude::*;

use crate::config::use_config;

const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

/// The main application component.
#[component]
fn App() -> Element {
    let config = use_config()?;

    rsx! {}
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
