// SPDX-FileCopyrightText: 2026 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use crate::HotKey;

/// [`HotKeyManager`]'s [`HotKey`] events.
///
/// [`HotKeyManager`]: crate::HotKeyManager
/// [`HotKey`]: crate::HotKey
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HotKeyEvent {
    /// A [`HotKey`] got pressed.
    Pressed {
        /// The [`HotKey`] that got pressed.
        hotkey: HotKey,
    },
}
