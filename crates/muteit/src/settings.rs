// SPDX-FileCopyrightText: 2025-2026 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use iced::Element;
use iced::widget::text;

/// Settings action.
pub enum Action {
    /// Nothing to do.
    None,
}

/// Message for the tray icon.
///
/// See [`Settings::update`].
#[derive(Debug)]
pub enum Message {}

/// Container for the settings window widgets.
///
/// This only contains stuff needed for the view.
pub struct Settings {}

impl Settings {
    /// Create a new [`Settings`].
    pub fn new() -> Self {
        Self {}
    }

    /// Update the settings.
    pub fn update(&mut self, message: Message) -> Action {
        Action::None
    }

    /// View the settings.
    pub fn view(&self) -> Element<'_, Message> {
        text!("todo").into()
    }
}
