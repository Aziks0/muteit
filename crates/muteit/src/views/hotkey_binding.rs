// SPDX-FileCopyrightText: 2025-2026 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::debug_assert_matches;

use iced::alignment::Vertical;
use iced::font::Weight;
use iced::keyboard::key::{Code, NativeCode, Physical as IcedPhysicalKey};
use iced::keyboard::{Key, Modifiers as IcedModifiers};
use iced::widget::{column, container, row, space, text};
use iced::{Background, Border, Color, Element, Font, Length, Padding, Subscription, keyboard};
use smol_str::{SmolStr, StrExt, format_smolstr};

use hotkey::{HotKey, Modifiers};

use crate::config::HotkeyInfo;
use crate::utils;
use crate::views::theme::{
    BORDER, BORDER_RADIUS, HEIGHT, HOVER_BORDER, PADDING, PRESSED_BORDER, color,
};
use crate::views::widgets::{button, kbd, label};

/// Error when trying to get a bind for a hotkey.
///
/// Use [`Self::as_str`] to get the description of the error.
#[derive(Clone, Copy, Debug)]
enum HotKeyBindingError {
    UnidentifiedKey,
    NoKey,
    TooManyKeys,
}

impl HotKeyBindingError {
    /// Get a description of the error.
    const fn as_str(&self) -> &'static str {
        match self {
            HotKeyBindingError::UnidentifiedKey => "The pressed key couldn't be identified.",
            HotKeyBindingError::NoKey => "A key that is not a modifier is required.",
            HotKeyBindingError::TooManyKeys => "Only one key that is not a modifier is possible.",
        }
    }
}

/// A physical key.
enum PhysicalKey {
    /// A W3C [`KeyboardEvent.code`].
    ///
    /// [`KeyboardEvent.code`]: https://w3c.github.io/uievents-code/#code-value-tables
    Code(Code),
    /// A Windows scancode.
    Scancode(u16),
}

impl TryFrom<IcedPhysicalKey> for PhysicalKey {
    type Error = HotKeyBindingError;

    fn try_from(physical_key: IcedPhysicalKey) -> Result<Self, Self::Error> {
        Ok(match physical_key {
            IcedPhysicalKey::Code(code) => Self::Code(code),
            IcedPhysicalKey::Unidentified(NativeCode::Windows(scancode)) => {
                Self::Scancode(scancode)
            }
            _ => Err(HotKeyBindingError::UnidentifiedKey)?,
        })
    }
}

/// A single key press.
struct KeyPress {
    /// The physical key.
    physical_key: PhysicalKey,
    /// The key as a string (i.e. what's written on the keycap).
    text: SmolStr,
}

impl KeyPress {
    fn try_new(key: Key, physical_key: IcedPhysicalKey) -> Result<Self, HotKeyBindingError> {
        const UNIDENTIFIED_TEXT: SmolStr = SmolStr::new_inline("???");

        let physical_key = physical_key.try_into()?;

        let text = match key {
            Key::Character(text) => text.to_ascii_uppercase_smolstr(),
            Key::Named(name) => SmolStr::new_inline(utils::named_key_as_str(name)),
            Key::Unidentified => match physical_key {
                PhysicalKey::Scancode(scancode) => format_smolstr!("{:#06x}", scancode),
                // This case should never happen, as `Key::Named` would be used, but just in case.
                PhysicalKey::Code(_) => UNIDENTIFIED_TEXT,
            },
        };

        Ok(Self { physical_key, text })
    }
}

/// A series of key presses.
struct KeyPresses {
    /// The keys that were pressed, which are not modifiers.
    keys: Vec<KeyPress>,
    /// The modifiers that were pressed.
    modifiers: Modifiers,
}

impl KeyPresses {
    fn new() -> Self {
        Self {
            keys: Vec::with_capacity(2),
            modifiers: Modifiers::NONE,
        }
    }

    /// Clear the key presses.
    fn clear(&mut self) {
        self.keys.clear();
        self.modifiers = Modifiers::NONE;
    }

    /// Whether there are keypresses or not.
    const fn is_empty(&self) -> bool {
        self.keys.is_empty() && self.modifiers.is_empty()
    }

    /// Get the key presses as an element.
    fn as_element(&self) -> Option<Element<'_, Message>> {
        if self.is_empty() {
            return None;
        }

        let mut elements = Vec::with_capacity(12);

        for (name, _) in self.modifiers.iter_names() {
            elements.push(kbd(name));
        }

        for key in &self.keys {
            elements.push(kbd(key.text.as_str()));
        }

        Some(row(elements).spacing(4).into())
    }
}

impl From<HotkeyInfo> for KeyPresses {
    fn from(info: HotkeyInfo) -> Self {
        let mut this = Self::new();

        this.keys.push(KeyPress {
            // Use a dummy `PhysicalKey` because it doesn't matter, we won't be using it.
            physical_key: PhysicalKey::Scancode(0),
            text: info.key,
        });

        this.modifiers = info.hotkey.modifiers();

        this
    }
}

/// Hotkey binding action.
pub enum Action {
    /// Register the new hotkey.
    RegisterHotkey(HotkeyInfo),
    /// Unregister the current hotkey.
    UnregisterHotkey,
    /// Nothing to do.
    None,
}

/// Message for [`HotkeyBinding`].
///
/// See [`HotkeyBinding::update`].
#[derive(Debug, Clone)]
pub enum Message {
    /// Start the binding process.
    StartBinding,
    /// Stop the binding process.
    StopBinding,
    /// A key was received.
    KeyReceived {
        /// The key, as expressed by `iced`.
        key: Key,
        /// The physical key, as expressed by `iced`.
        physical_key: IcedPhysicalKey,
        /// The modifiers applied while the key was pressed, as expressed by `iced`.
        modifiers: IcedModifiers,
    },
    /// Remove the current bind.
    Unbind,
}

/// Component to show and bind a hotkey.
pub struct HotkeyBinding {
    key_presses: KeyPresses,
    is_binding: bool,
    error: Option<HotKeyBindingError>,
}

impl HotkeyBinding {
    pub fn new(info: Option<HotkeyInfo>) -> Self {
        Self {
            key_presses: info.map_or_else(KeyPresses::new, KeyPresses::from),
            is_binding: false,
            error: None,
        }
    }

    /// Update the component.
    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::StartBinding => {
                self.is_binding = true;
                self.error = None;
                self.key_presses.clear();

                Action::UnregisterHotkey
            }
            Message::StopBinding => {
                // Possible race between the subscription tracking/canceling and keyboard events?
                // Just check to be sure.
                if !self.is_binding {
                    debug_assert!(self.is_binding);
                    return Action::None;
                }

                self.is_binding = false;

                self.error = match self.key_presses.keys.len() {
                    0 => Some(HotKeyBindingError::NoKey),
                    1 => {
                        debug_assert_matches!(self.error, None);
                        // Keep `self.error` in case it is already `Some`. Shouldn't happen as
                        // `self.is_binding` would be `false` and so we wouldn't be here, but it
                        // doesn't cost anything compared to using `None` directly.
                        self.error
                    }
                    _ => Some(HotKeyBindingError::TooManyKeys),
                };

                if self.error.is_some() {
                    return Action::None;
                };

                let hotkey_info = {
                    let virtual_key = match self.key_presses.keys[0].physical_key {
                        PhysicalKey::Code(code) => key_t::physical_key_to_virtual_key(code),
                        PhysicalKey::Scancode(scancode) => key_t::scancode_to_virtual_key(scancode),
                    };

                    let Some(virtual_key) = virtual_key else {
                        self.error = Some(HotKeyBindingError::UnidentifiedKey);

                        return Action::None;
                    };

                    HotkeyInfo {
                        hotkey: HotKey::new(virtual_key, self.key_presses.modifiers),
                        key: self.key_presses.keys[0].text.clone(),
                    }
                };

                Action::RegisterHotkey(hotkey_info)
            }
            Message::KeyReceived {
                key,
                physical_key,
                modifiers,
            } => {
                // Possible race between the subscription tracking/canceling and keyboard events?
                // Just check to be sure.
                if !self.is_binding {
                    debug_assert!(self.is_binding);
                    return Action::None;
                }

                let modifiers = utils::translate_modifiers(modifiers);

                if self.key_presses.modifiers != modifiers {
                    self.key_presses.modifiers = modifiers;

                    // We got a modifier, so no reason to check the key, just stop there.
                    return Action::None;
                }

                match KeyPress::try_new(key, physical_key) {
                    Ok(key_press) => self.key_presses.keys.push(key_press),
                    Err(err) => {
                        self.is_binding = false;
                        self.error = Some(err)
                    }
                }

                Action::None
            }
            Message::Unbind => {
                self.is_binding = false;
                self.error = None;
                self.key_presses.clear();

                Action::UnregisterHotkey
            }
        }
    }

    /// Subscribe to hotkey binding events.
    pub fn subscription(&self) -> Subscription<Message> {
        if !self.is_binding {
            return Subscription::none();
        }

        keyboard::listen().filter_map(|event| match event {
            keyboard::Event::KeyPressed {
                repeat: false,
                physical_key,
                key,
                modifiers,
                ..
            } => Some(Message::KeyReceived {
                key,
                physical_key,
                modifiers,
            }),
            keyboard::Event::KeyReleased { .. } => Some(Message::StopBinding),
            _ => None,
        })
    }

    /// Create a new hotkey binder element.
    fn binder<'a>(&'a self) -> iced::widget::Button<'a, Message> {
        use iced::widget::button::{Status, Style};

        let content = self.key_presses.as_element().unwrap_or_else(|| {
            let content = if self.is_binding {
                "Press a key..."
            } else {
                "Click to start binding"
            };

            text(content)
                .height(Length::Fill)
                .align_y(Vertical::Center)
                .into()
        });

        iced::widget::button(content)
            .style(|_, status| {
                let background = Some(Background::Color(Color::TRANSPARENT));

                if self.error.is_some() {
                    return Style {
                        background,
                        text_color: color::TEXT,
                        border: Border {
                            color: color::DANGER,
                            radius: BORDER_RADIUS.bottom_left(0),
                            ..BORDER
                        },
                        ..Default::default()
                    };
                }

                if self.is_binding {
                    return Style {
                        background,
                        text_color: color::TEXT,
                        border: BORDER.color(Color::WHITE),
                        ..Default::default()
                    };
                }

                match status {
                    Status::Active | Status::Disabled => Style {
                        background,
                        text_color: color::TEXT,
                        border: BORDER,
                        ..Default::default()
                    },
                    Status::Hovered => Style {
                        background,
                        text_color: color::TEXT,
                        border: HOVER_BORDER,
                        ..Default::default()
                    },
                    Status::Pressed => Style {
                        background,
                        text_color: color::TEXT,
                        border: PRESSED_BORDER,
                        ..Default::default()
                    },
                }
            })
            .width(Length::Fill)
            .height(HEIGHT)
            .padding(PADDING.vertical(6))
    }

    /// View the component.
    pub fn view(&self) -> Element<'_, Message> {
        let error_message: Element<_> = self.error.map_or_else(
            || space().into(),
            |err| {
                container(text(err.as_str()).size(14).font(Font {
                    weight: Weight::Bold,
                    ..Font::DEFAULT
                }))
                .style(|_| iced::widget::container::Style {
                    background: Some(Background::Color(color::DANGER)),
                    border: Border {
                        radius: BORDER_RADIUS.top(0),
                        ..Default::default()
                    },
                    text_color: Some(color::TEXT),
                    ..Default::default()
                })
                .padding(Padding::default().horizontal(4).vertical(1))
                .into()
            },
        );

        column![
            label("Hotkey"),
            column![
                row![
                    self.binder().on_press(Message::StartBinding),
                    button("Unbind").on_press(Message::Unbind)
                ]
                .spacing(4)
                .width(Length::Fill),
                error_message
            ]
            .spacing(-2.)
            .width(Length::Fill)
        ]
        .width(Length::Fill)
        .into()
    }
}
