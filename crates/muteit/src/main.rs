// SPDX-FileCopyrightText: 2025 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use dioxus::desktop::tao::platform::windows::WindowBuilderExtWindows;
use dioxus::desktop::{Config, WindowBuilder};
use dioxus::prelude::*;

const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

#[component]
fn App() -> Element {
    #[cfg(debug_assertions)]
    dioxus::desktop::use_global_shortcut("Ctrl+Shift+I", |state| {
        if state == dioxus::desktop::HotKeyState::Pressed {
            dioxus::desktop::window().devtool();
        }
    })
    .unwrap();

    rsx! {
        document::Stylesheet { href: TAILWIND_CSS }
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
        .launch(App);
}
