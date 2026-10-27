// SPDX-FileCopyrightText: 2026 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::Result;
use iced::widget::column;
use iced::{Element, Subscription};

use audio::enumerator::Input;

use crate::config::{DeviceInfo, HotkeyInfo};
use crate::device_selector::{self, DeviceSelector};
use crate::hotkey_binding::{self, HotkeyBinding};
use crate::utils::DeviceEnumerator;

/// Settings action.
pub enum Action {
    /// See [`device_selector::Action`].
    DeviceSelector(device_selector::Action),
    /// See [`hotkey_binding::Action`].
    HotKeyBinding(hotkey_binding::Action),
}

/// Message for the tray icon.
///
/// See [`Settings::update`].
#[derive(Debug)]
pub enum Message {
    DeviceSelector(device_selector::Message),
    HotkeyBinding(hotkey_binding::Message),
}

/// Container for the settings window widgets.
///
/// This only contains stuff needed for the view.
pub struct Settings {
    device_selector: DeviceSelector<Input>,
    hotkey_binding: HotkeyBinding,
}

impl Settings {
    /// Create a new [`Settings`].
    pub fn try_new(
        enumerator: DeviceEnumerator<Input>,
        device_info: Option<DeviceInfo>,
        hotkey_info: Option<HotkeyInfo>,
    ) -> Result<Self> {
        Ok(Self {
            device_selector: DeviceSelector::try_new(enumerator, device_info)?,
            hotkey_binding: HotkeyBinding::new(hotkey_info),
        })
    }

    /// Update the settings.
    pub fn update(&mut self, message: Message) -> Result<Action> {
        Ok(match message {
            Message::DeviceSelector(message) => {
                let action = self.device_selector.update(message)?;

                Action::DeviceSelector(action)
            }
            Message::HotkeyBinding(message) => {
                let action = self.hotkey_binding.update(message);

                Action::HotKeyBinding(action)
            }
        })
    }

    /// Subscribe to settings events.
    pub fn subscription(&self) -> Subscription<Message> {
        self.hotkey_binding
            .subscription()
            .map(Message::HotkeyBinding)
    }

    /// View the settings.
    pub fn view(&self) -> Element<'_, Message> {
        column![
            self.device_selector.view().map(Message::DeviceSelector),
            self.hotkey_binding.view().map(Message::HotkeyBinding)
        ]
        .into()
    }
}
