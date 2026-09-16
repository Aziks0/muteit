// SPDX-FileCopyrightText: 2025-2026 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use iced::Element;

use audio::enumerator::Input;

use crate::device_selector::{self, DeviceSelector};
use crate::utils::DeviceEnumerator;

/// Settings action.
pub enum Action {
    /// See [`device_selector::Action`].
    DeviceSelector(device_selector::Action),
}

/// Message for the tray icon.
///
/// See [`Settings::update`].
#[derive(Debug)]
pub enum Message {
    DeviceSelector(device_selector::Message),
}

/// Container for the settings window widgets.
///
/// This only contains stuff needed for the view.
pub struct Settings {
    device_selector: DeviceSelector<Input>,
}

impl Settings {
    /// Create a new [`Settings`].
    pub fn new(enumerator: DeviceEnumerator<Input>) -> Self {
        Self {
            device_selector: DeviceSelector::new(enumerator, None),
        }
    }

    /// Update the settings.
    pub fn update(&mut self, message: Message) -> Action {
        match message {
            Message::DeviceSelector(message) => {
                let action = self.device_selector.update(message);

                Action::DeviceSelector(action)
            }
        }
    }

    /// View the settings.
    pub fn view(&self) -> Element<'_, Message> {
        self.device_selector.view().map(Message::DeviceSelector)
    }
}
