// SPDX-FileCopyrightText: 2025 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::str::FromStr;

use dioxus::core::use_hook_with_cleanup;
use dioxus::desktop::wry::http::Response;
use dioxus::desktop::{Code, HotKey, window};
use dioxus::prelude::*;
use serde::Deserialize;

use key_t::LogicalKey;

use crate::config::HotkeyInformation;

#[cfg(not(feature = "bundle"))]
const SCRIPT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/hotkey_binding.js"
));
#[cfg(feature = "bundle")]
const SCRIPT: &str = include_str!(concat!(env!("OUT_DIR"), "/hotkey_binding.js"));

const FONT_NAME: &str = "JetBrainsMono-Regular.woff2";
const FONT: &[u8] = include_bytes!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/assets/JetBrainsMono-Regular.woff2",
));

/// The [keyboard event](https://developer.mozilla.org/en-US/docs/Web/API/KeyboardEvent) given to us
/// by our JS script.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct KeyPress {
    ctrl: bool,
    shift: bool,
    alt: bool,
    meta: bool,
    code: String,
    key: String,
}

/// Message received from our JS script.
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
enum Message {
    /// A key press.
    KeyPress(KeyPress),
    /// No more messages.
    Finished,
}

/// A single key.
struct Key {
    /// The display character(s).
    key: String,
    /// The key code.
    code: Code,
}

/// A series of key presses.
struct KeyPresses {
    modifiers: Modifiers,
    keys: Vec<Key>,
}

impl KeyPresses {
    fn new() -> Self {
        Self {
            modifiers: Modifiers::empty(),
            keys: Vec::with_capacity(2),
        }
    }

    fn push(&mut self, key_press: KeyPress) {
        let mut modifiers = Modifiers::empty();
        if key_press.ctrl {
            modifiers |= Modifiers::CONTROL;
        }
        if key_press.shift {
            modifiers |= Modifiers::SHIFT;
        }
        if key_press.alt {
            modifiers |= Modifiers::ALT;
        }
        if key_press.meta {
            modifiers |= Modifiers::SUPER;
        }

        if !self.modifiers.contains(modifiers) {
            self.modifiers = modifiers;
            // We got a modifier, so no reason to check the key, just stop there.
            return;
        }

        // The JS `KeyboardEvent.code` maps to a physical key but Windows expect a logical key when
        // binding `HotKey`s (e.g. `A` maps to `KeyQ` on AZERTY, but Windows still expect `VK_A`).
        let (code, key) = match LogicalKey::from_physical_key(&key_press.code) {
            Some(logical_key) => (
                logical_key.code,
                logical_key
                    .key
                    .map(|c| match c {
                        '\t' => String::from("Tab"),
                        '\n' => String::from("Enter"),
                        '\r' => String::from("Enter"),
                        '\x08' => String::from("Backspace"),
                        '\x1B' => String::from("Escape"),
                        '\x7F' => String::from("Delete"),
                        ' ' => String::from("Space"),
                        _ => c.to_string(),
                    })
                    .unwrap_or(key_press.key),
            ),
            None => (
                // It seems like, on Edge, some keys give an empty `code` and in those cases the
                // `key` is set instead. e.g. `MediaTrackNext`, `AudioVolumeMute`.
                if key_press.code.len() != 0 {
                    Code::from_str(&key_press.code)
                        .expect("`KeyboardEvent.code` to be W3C compliant")
                } else {
                    Code::from_str(&key_press.key).expect("")
                },
                key_press.key,
            ),
        };

        self.keys.push(Key { key, code });
    }

    fn clear(&mut self) {
        self.modifiers = Modifiers::empty();
        self.keys.clear();
    }

    fn into_elements(&self) -> Element {
        #[component]
        fn Kbd(hidden: bool, children: Element) -> Element {
            rsx! {
                kbd {
                    class: "cursor-default bg-black px-2.5 py-1.5 text-sm",
                    class: "rounded-md border border-b-3 border-gray-600",
                    class: if hidden { "hidden" },
                    {children}
                }
            }
        }

        rsx! {
            Kbd { hidden: !self.modifiers.contains(Modifiers::SUPER), "Win" }
            Kbd { hidden: !self.modifiers.contains(Modifiers::CONTROL), "Ctrl" }
            Kbd { hidden: !self.modifiers.contains(Modifiers::SHIFT), "Shift" }
            Kbd { hidden: !self.modifiers.contains(Modifiers::ALT), "Alt" }
            for key in &self.keys {
                Kbd { hidden: false, "{key.key}" }
            }
        }
    }

    fn into_hotkey(&self) -> Result<HotkeyInformation, &'static str> {
        const NO_KEY: &str = "A key that is not a modifier is required.";
        const TOO_MANY_KEYS: &str = "Only one key that is not a modifier possible.";

        match self.keys.len() {
            0 => Err(NO_KEY),
            1 => Ok(HotkeyInformation {
                hotkey: HotKey::new(Some(self.modifiers), self.keys[0].code),
                key: self.keys[0].key.clone(),
            }),
            _ => Err(TOO_MANY_KEYS),
        }
    }
}

impl From<&HotkeyInformation> for KeyPresses {
    fn from(value: &HotkeyInformation) -> Self {
        let mut ret = Self {
            modifiers: value.hotkey.mods,
            keys: Vec::with_capacity(2),
        };
        ret.keys.push(Key {
            key: value.key.clone(),
            code: value.hotkey.key,
        });
        ret
    }
}

#[component]
pub fn HotkeyBinding(hotkey: WriteSignal<Option<HotkeyInformation>>) -> Element {
    use_hook_with_cleanup(
        || {
            window().register_asset_handler(String::from(FONT_NAME), |_asset, responder| {
                responder.respond(
                    Response::builder()
                        .header("Content-Type", "application/font-woff")
                        .header("Access-Control-Allow-Origin", "*")
                        .body(FONT)
                        .unwrap(),
                )
            })
        },
        |_| _ = window().remove_asset_handler(FONT_NAME),
    );

    let mut is_binding = use_signal(|| false);
    let mut key_presses = use_signal(|| match *hotkey.peek() {
        Some(ref hotkey) => KeyPresses::from(hotkey),
        None => KeyPresses::new(),
    });
    let mut error = use_signal::<Option<&'static str>>(|| None);

    rsx! {
        div {
            label { class: "ml-1 text-sm font-medium", id: "hotkey-label", "Hotkey" }
            div { class: "flex w-full gap-1",
                div {
                    class: "flex items-center gap-1 grow px-1",
                    class: "rounded-md border-2 border-transparent outline-none",
                    class: if is_binding() { "border-white" },
                    class: if error() != None { "rounded-bl-none border-red-600!" },
                    aria_labelledby: "hotkey-label",
                    {key_presses.read().into_elements()}
                }
                button {
                    class: "cursor-pointer rounded-md border-2 border-gray-900 bg-black px-3 py-2",
                    class: "hover:border-gray-700",
                    class: "focus-visible:border-white focus-visible:outline-none",
                    onclick: move |_| {
                        key_presses.write().clear();
                        error.set(None);
                        hotkey.set(None);
                    },
                    "Unbind"
                }
                button {
                    class: "cursor-pointer rounded-md border-2 border-gray-900 bg-black px-3 py-2",
                    class: "hover:border-gray-700",
                    class: "focus-visible:border-white focus-visible:outline-none",
                    id: "bind-button",
                    onclick: move |_| async move {
                        if is_binding() {
                            return;
                        }

                        is_binding.set(true);

                        hotkey.set(None);
                        key_presses.write().clear();
                        error.set(None);

                        let mut eval = document::eval(SCRIPT);

                        loop {
                            let message: Message = eval
                                .recv()
                                .await
                                .expect("Message to be deserialized");
                            match message {
                                Message::KeyPress(key_press) => key_presses.write().push(key_press),
                                Message::Finished => break,
                            }
                        }

                        match key_presses.read().into_hotkey() {
                            Err(message) => error.set(Some(message)),
                            Ok(new_hotkey) => hotkey.set(Some(new_hotkey)),
                        };

                        is_binding.set(false);
                    },
                    "Bind"
                }
            }
            if let Some(message) = error() {
                div { class: "absolute w-fit rounded-b-md bg-red-600 px-1",
                    p { class: "-mt-0.5 text-sm font-medium", role: "alert", {message} }
                }
            }
        }
    }
}
