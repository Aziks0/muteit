// SPDX-FileCopyrightText: 2026 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use bitflags::bitflags;
use serde::{Deserialize, Serialize};

use crate::windows::Windows::Win32::{MOD_ALT, MOD_CONTROL, MOD_SHIFT, MOD_WIN};

bitflags! {
    /// The state of the keyboard modifiers.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
    pub struct Modifiers: u32 {
        /// No modifiers.
        const NONE = 0;

        // Use the same values as the Windows API so translation is transparent.
        //
        // NOTE: Windows modifier values aren't going to change overnight. We can assume they will
        // never be `> u16::MAX` and this is important for `HotKey::id`. Keep them as `u32` tho, as
        // this avoids having to cast them back everywhere (and zero-extending them).

        /// The `Windows` modifier.
        const WIN = MOD_WIN as _;
        /// The `Control` modifier.
        const CTRL = MOD_CONTROL as _;
        /// The `Alt` modifier.
        const ALT = MOD_ALT as _;
        /// The `Shift` modifier.
        const SHIFT = MOD_SHIFT as _;
    }
}
