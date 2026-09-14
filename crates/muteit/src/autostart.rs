// SPDX-FileCopyrightText: 2026 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::rc::Rc;
use std::sync::Arc;

use dioxus::prelude::*;

#[component]
pub fn AutoStart() -> Element {
    let autostart = use_hook(|| {
        autostart::AutoStart!(with "muteit!", "--hidden")
            .map(Rc::new)
            .map_err(Arc::new)
    })?;
    let mut is_checked = use_hook(|| {
        autostart
            .is_set_autocorrect()
            .map(Signal::new)
            .map_err(Arc::new)
    })?;

    rsx! {
        div { class: "mt-2 flex justify-between",
            label { class: "font-medium", r#for: "input-autostart", "Open on system startup" }
            label {
                class: "relative cursor-pointer group",
                r#for: "input-autostart",
                input {
                    class: "sr-only peer",
                    r#type: "checkbox",
                    checked: is_checked,
                    id: "input-autostart",
                    onchange: move |_| {
                        let checked = !is_checked();
                        is_checked.set(checked);
                        autostart.set(checked)?;
                        Ok(())
                    },
                }
                div {
                    class: "h-10 w-18 rounded-full border-2 border-gray-900 bg-black",
                    class: "group-hover:border-gray-700",
                    class: "peer-focus-visible:border-white",
                }
                div {
                    class: "absolute top-1.5 left-2 h-7 w-7 rounded-full bg-neutral-700 transition",
                    class: if is_checked() { "translate-x-full bg-white!" },
                }
            }
        }
    }
}
