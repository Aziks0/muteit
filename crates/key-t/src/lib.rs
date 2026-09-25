// SPDX-FileCopyrightText: 2025-2026 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: MPL-2.0

//! key-translation
//!
//! This crates provides a way to translate a physical key to a logical key.

#[cfg(not(target_os = "windows"))]
compile_error!("This crate is only available on Windows.");

mod code;
mod windows;

use iced::keyboard::key::Code;

use crate::code::code_to_scancode;
use crate::windows::Win32::{MAPVK_VSC_TO_VK, MapVirtualKeyExW};

/// Convert a Windows scancode to a logical [`Virtual-Key`].
///
/// [`Virtual-Key`]: https://learn.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes
pub fn scancode_to_virtual_key(scancode: u16) -> Option<u32> {
    let virtual_key = unsafe { MapVirtualKeyExW(scancode as u32, MAPVK_VSC_TO_VK as _, None) };
    if virtual_key == 0 {
        return None;
    };

    Some(virtual_key)
}

/// Convert a physical key to a logical Windows [`Virtual-Key`].
///
/// The physical key should be an `iced` [`Code`] (aka a W3C [`KeyboardEvent.code`]).
///
/// [`Virtual-Key`]: https://learn.microsoft.com/en-us/windows/win32/inputdev/virtual-key-codes
/// [`KeyboardEvent.code`]: https://w3c.github.io/uievents-code/#code-value-tables
pub fn physical_key_to_virtual_key(code: Code) -> Option<u32> {
    let scancode = code_to_scancode(code)?;

    scancode_to_virtual_key(scancode)
}
