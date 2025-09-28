// SPDX-FileCopyrightText: 2025 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use audio::DeviceEnumerator;
use dioxus::prelude::*;

use crate::config::DeviceInformation;

#[component]
pub fn DeviceSelector(
    enumerator: WriteSignal<DeviceEnumerator>,
    on_select: Callback<Option<u32>>,
    default: Option<DeviceInformation>,
) -> Element {
    let mut found_selected = default.is_none();
    let default = default.unwrap_or_default();

    let mut elements = Vec::with_capacity(enumerator.read().len());

    for (value, information) in enumerator.read().iter().enumerate() {
        let id = information.get_id();
        let initial_selected = id == default.id;
        if initial_selected {
            found_selected = true;
        }

        let element = rsx! {
            option { key: "{id}", value, initial_selected, "{information}" }

        };
        elements.push(element);
    }

    // We want the selected device to be in the list, even if it is not. We just disable it so the
    // user cannot select it.
    if !found_selected {
        let DeviceInformation { id, name } = default;
        let element = rsx! {
            option {
                key: "{id}",
                value: "default",
                initial_selected: true,
                disabled: true,
                "{name} (Unavailable)"
            }
        };
        elements.push(element);
    }

    rsx! {
        div {
            label { class: "ml-1 text-sm font-medium", r#for: "device-select", "Input device" }
            div { class: "flex w-full gap-1",
                select {
                    id: "device-select",
                    class: "grow rounded-md border-2 border-gray-900 bg-black px-3 py-2",
                    class: "hover:border-gray-700",
                    class: "focus-visible:border-white focus-visible:outline-none",
                    onchange: move |event| {
                        let value = event.value().parse::<u32>().ok();
                        on_select.call(value)
                    },
                    option { value: "none", "None" }
                    {elements.iter()}
                }
                button {
                    class: "cursor-pointer rounded-md border-2 border-gray-900 bg-black px-3 py-2",
                    class: "hover:border-gray-700",
                    class: "focus-visible:border-white focus-visible:outline-none",
                    onclick: move |_| {
                        enumerator.write().refresh()?;
                        Ok(())
                    },
                    "Refresh"
                }
            }
        }
    }
}
