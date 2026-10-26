// SPDX-FileCopyrightText: 2026 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::{Context, Result};
use iced::Subscription;

use hotkey::{HotKey, HotKeyEvent, Receiver};

/// Message for [`HotKeyManager`].
///
/// See [`HotKeyManager::update`].
pub enum Message {
    /// Register a new [`HotKey`].
    RegisterHotKey(HotKey),
    /// Unregister a [`HotKey`].
    UnregisterHotKey(HotKey),
}

/// Register hotkeys and listen to hotkeys event.
pub struct HotKeyManager {
    manager: hotkey::HotKeyManager,
    rx: Receiver<HotKeyEvent>,
}

impl HotKeyManager {
    pub fn try_new() -> Result<Self> {
        let manager = hotkey::HotKeyManager::try_new().context("Creating `HotKeyManager`.")?;
        let rx = manager.receiver();

        Ok(Self { manager, rx })
    }

    /// Update the hotkey manager.
    pub fn update(&mut self, message: Message) -> Result<()> {
        match message {
            Message::RegisterHotKey(hotkey) => self.manager.register(hotkey)?,
            Message::UnregisterHotKey(hotkey) => self.manager.unregister(hotkey)?,
        };

        Ok(())
    }

    /// Subscribe to [`HotKeyEvent`]s.
    pub fn subscription(&self) -> Subscription<HotKeyEvent> {
        use std::hash::{Hash, Hasher};

        #[derive(Clone)]
        struct Receiver(hotkey::Receiver<HotKeyEvent>);

        impl Hash for Receiver {
            fn hash<H: Hasher>(&self, state: &mut H) {
                std::any::TypeId::of::<Self>().hash(state);
                "hotkey_manager".as_bytes().hash(state);
            }
        }

        let rx = Receiver(self.rx.clone());

        Subscription::run_with(rx, |rx| rx.0.clone())
    }
}
