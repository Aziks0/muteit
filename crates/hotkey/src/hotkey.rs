// SPDX-FileCopyrightText: 2026 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use serde::{Deserialize, Serialize};

use crate::Modifiers;

/// A Windows HotKey.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct HotKey {
    /// The Windows [`Virtual-Key`] for this hotkey.
    ///
    /// [`Virtual-Key`]: https://learn.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes
    pub(crate) virtual_key: u32,
    /// The modifiers for this hotkey.
    pub(crate) modifiers: Modifiers,
}

impl HotKey {
    /// Create a new HotKey.
    ///
    /// The [`HotKey`] need to be registered using [`HotKeyManager::register`].
    ///
    /// [`HotKeyManager::register`]: crate::HotKeyManager::register
    pub const fn new(virtual_key: u32, modifiers: Modifiers) -> Self {
        // Virtual keys are `DWORD`s because that's the standard for Windows, but, as far as I know,
        // virtual keys are all `< u16::MAX`. This is important for `Self::id`.
        assert!(virtual_key <= u16::MAX as _);

        Self {
            virtual_key,
            modifiers,
        }
    }

    /// Get the virtual key for this hotkey.
    pub const fn virtual_key(&self) -> u32 {
        self.virtual_key
    }

    /// Get the modifiers for this hotkey.
    pub const fn modifiers(&self) -> Modifiers {
        self.modifiers
    }

    /// Get this hotkey's id.
    ///
    /// This is used to identify the [`HotKey`]s associated to [`HotKeyEvent`]s.
    ///
    /// [`HotKeyEvent`]: crate::HotKeyEvent
    pub const fn id(&self) -> u32 {
        // We've asserted that `virtual_key <= u16::MAX` in `new`, so it will not leak into the
        // `modifiers` part.
        self.modifiers.bits() << 16 | self.virtual_key
    }

    /// Create a [`HotKey`] from an id.
    ///
    /// This should only be used to convert an id obtained from a previous call to [`HotKey::id`].
    pub(crate) const fn from_id(id: u32) -> Self {
        let modifiers = id >> 16;
        let virtual_key = id & u16::MAX as u32;

        Self {
            virtual_key,
            modifiers: Modifiers::from_bits_retain(modifiers),
        }
    }
}

impl PartialEq for HotKey {
    fn eq(&self, other: &Self) -> bool {
        self.id() == other.id()
    }
}

impl Eq for HotKey {}
