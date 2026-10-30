// SPDX-FileCopyrightText: 2026 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::rc::Rc;

use anyhow::Result;

use autostart::AutoStarter;
use iced::Element;

use crate::views::widgets::option;

/// Message for [`AutoStart`].
///
/// See [`AutoStart::update`].
#[derive(Debug)]
pub enum Message {
    Toggled(bool),
}

/// Switch to change application's automatic start behaviour.
pub struct AutoStart {
    is_toggled: bool,
    autostart: Rc<AutoStarter>,
}

impl AutoStart {
    pub fn try_new(autostart: Rc<autostart::AutoStarter>) -> Result<Self> {
        Ok(Self {
            is_toggled: autostart.is_set()?,
            autostart,
        })
    }

    /// Update the component.
    pub fn update(&mut self, message: Message) -> Result<()> {
        match message {
            Message::Toggled(is_checked) => {
                self.is_toggled = is_checked;

                self.autostart.set(is_checked)?;

                Ok(())
            }
        }
    }

    /// View the component.
    pub fn view(&self) -> Element<'_, Message> {
        option("Open on system startup", self.is_toggled, Message::Toggled).into()
    }
}
