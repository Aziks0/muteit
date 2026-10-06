// SPDX-FileCopyrightText: 2026 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::hash::Hash;

use anyhow::{Context, Result};
use iced::Subscription;

use hotkey::{HotKey, HotKeyEvent};

/// Wrapper around [`hotkey::Receiver<HotKeyEvent>`]. Used in [`Subscription`].
#[derive(Clone)]
struct Receiver(hotkey::Receiver<HotKeyEvent>);

impl Hash for Receiver {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::any::TypeId::of::<Self>().hash(state);
        "hotkey_manager".as_bytes().hash(state);
    }
}

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
    rx: Receiver,
}

impl HotKeyManager {
    pub fn try_new() -> Result<Self> {
        let manager = hotkey::HotKeyManager::try_new().context("Creating `HotKeyManager`.")?;
        let rx = manager.receiver();

        Ok(Self {
            manager,
            rx: Receiver(rx),
        })
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
        let rx = self.rx.clone();

        Subscription::run_with(rx, |rx| rx.0.clone())
    }
}
