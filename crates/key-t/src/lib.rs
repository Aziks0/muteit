// SPDX-FileCopyrightText: 2025-2026 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: MPL-2.0

//! key-translation
//!
//! This crates provides a way to translate a physical key to a logical key, the physical key being
//! a [W3C `KeyboardEvent.code`](https://w3c.github.io/uievents-code/#code-value-tables).

#[cfg(not(target_os = "windows"))]
compile_error!("This crate is only available on Windows.");

mod windows;

use global_hotkey::hotkey::Code;

use crate::windows::Win32::*;
use crate::windows::*;

include!(concat!(env!("OUT_DIR"), "/code_to_scancode.rs"));

/// One-one mapping of Windows Virtual-Keys to [`Code`].
fn virtual_key_to_code(virtual_key: u32) -> Option<Code> {
    let virtual_key = virtual_key as _;

    let code = match virtual_key {
        VK_A => Code::KeyA,
        VK_B => Code::KeyB,
        VK_C => Code::KeyC,
        VK_D => Code::KeyD,
        VK_E => Code::KeyE,
        VK_F => Code::KeyF,
        VK_G => Code::KeyG,
        VK_H => Code::KeyH,
        VK_I => Code::KeyI,
        VK_J => Code::KeyJ,
        VK_K => Code::KeyK,
        VK_L => Code::KeyL,
        VK_M => Code::KeyM,
        VK_N => Code::KeyN,
        VK_O => Code::KeyO,
        VK_P => Code::KeyP,
        VK_Q => Code::KeyQ,
        VK_R => Code::KeyR,
        VK_S => Code::KeyS,
        VK_T => Code::KeyT,
        VK_U => Code::KeyU,
        VK_V => Code::KeyV,
        VK_W => Code::KeyW,
        VK_X => Code::KeyX,
        VK_Y => Code::KeyY,
        VK_Z => Code::KeyZ,
        VK_0 => Code::Digit0,
        VK_1 => Code::Digit1,
        VK_2 => Code::Digit2,
        VK_3 => Code::Digit3,
        VK_4 => Code::Digit4,
        VK_5 => Code::Digit5,
        VK_6 => Code::Digit6,
        VK_7 => Code::Digit7,
        VK_8 => Code::Digit8,
        VK_9 => Code::Digit9,
        VK_OEM_PLUS => Code::Equal,
        VK_OEM_COMMA => Code::Comma,
        VK_OEM_MINUS => Code::Minus,
        VK_OEM_PERIOD => Code::Period,
        VK_OEM_1 => Code::Semicolon,
        VK_OEM_2 => Code::Slash,
        VK_OEM_3 => Code::Backquote,
        VK_OEM_4 => Code::BracketLeft,
        VK_OEM_5 => Code::Backslash,
        VK_OEM_6 => Code::BracketRight,
        VK_OEM_7 => Code::Quote,
        VK_BACK => Code::Backspace,
        VK_TAB => Code::Tab,
        VK_SPACE => Code::Space,
        VK_RETURN => Code::Enter,
        VK_CAPITAL => Code::CapsLock,
        VK_ESCAPE => Code::Escape,
        VK_PRIOR => Code::PageUp,
        VK_NEXT => Code::PageDown,
        VK_END => Code::End,
        VK_HOME => Code::Home,
        VK_LEFT => Code::ArrowLeft,
        VK_UP => Code::ArrowUp,
        VK_RIGHT => Code::ArrowRight,
        VK_DOWN => Code::ArrowDown,
        VK_SNAPSHOT => Code::PrintScreen,
        VK_INSERT => Code::Insert,
        VK_DELETE => Code::Delete,
        VK_F1 => Code::F1,
        VK_F2 => Code::F2,
        VK_F3 => Code::F3,
        VK_F4 => Code::F4,
        VK_F5 => Code::F5,
        VK_F6 => Code::F6,
        VK_F7 => Code::F7,
        VK_F8 => Code::F8,
        VK_F9 => Code::F9,
        VK_F10 => Code::F10,
        VK_F11 => Code::F11,
        VK_F12 => Code::F12,
        VK_F13 => Code::F13,
        VK_F14 => Code::F14,
        VK_F15 => Code::F15,
        VK_F16 => Code::F16,
        VK_F17 => Code::F17,
        VK_F18 => Code::F18,
        VK_F19 => Code::F19,
        VK_F20 => Code::F20,
        VK_F21 => Code::F21,
        VK_F22 => Code::F22,
        VK_F23 => Code::F23,
        VK_F24 => Code::F24,
        VK_NUMLOCK => Code::NumLock,
        VK_NUMPAD0 => Code::Numpad0,
        VK_NUMPAD1 => Code::Numpad1,
        VK_NUMPAD2 => Code::Numpad2,
        VK_NUMPAD3 => Code::Numpad3,
        VK_NUMPAD4 => Code::Numpad4,
        VK_NUMPAD5 => Code::Numpad5,
        VK_NUMPAD6 => Code::Numpad6,
        VK_NUMPAD7 => Code::Numpad7,
        VK_NUMPAD8 => Code::Numpad8,
        VK_NUMPAD9 => Code::Numpad9,
        VK_ADD => Code::NumpadAdd,
        VK_DECIMAL => Code::NumpadDecimal,
        VK_DIVIDE => Code::NumpadDivide,
        VK_MULTIPLY => Code::NumpadMultiply,
        VK_SUBTRACT => Code::NumpadSubtract,
        VK_SCROLL => Code::ScrollLock,
        VK_VOLUME_DOWN => Code::AudioVolumeDown,
        VK_VOLUME_UP => Code::AudioVolumeUp,
        VK_VOLUME_MUTE => Code::AudioVolumeMute,
        VK_PLAY => Code::MediaPlay,
        VK_MEDIA_PLAY_PAUSE => Code::MediaPlayPause,
        VK_MEDIA_STOP => Code::MediaStop,
        VK_MEDIA_NEXT_TRACK => Code::MediaTrackNext,
        VK_MEDIA_PREV_TRACK => Code::MediaTrackPrevious,
        VK_LAUNCH_MEDIA_SELECT => Code::MediaSelect,
        VK_LAUNCH_MAIL => Code::LaunchMail,
        VK_LAUNCH_APP1 => Code::LaunchApp1,
        VK_LAUNCH_APP2 => Code::LaunchApp2,
        VK_PAUSE => Code::Pause,
        VK_SLEEP => Code::Sleep,
        VK_BROWSER_BACK => Code::BrowserBack,
        VK_BROWSER_FORWARD => Code::BrowserForward,
        VK_BROWSER_REFRESH => Code::BrowserRefresh,
        VK_BROWSER_STOP => Code::BrowserStop,
        VK_BROWSER_SEARCH => Code::BrowserSearch,
        VK_BROWSER_FAVORITES => Code::BrowserFavorites,
        VK_BROWSER_HOME => Code::BrowserHome,
        VK_CONTROL => Code::ControlLeft,
        VK_LCONTROL => Code::ControlLeft,
        VK_RCONTROL => Code::ControlRight,
        VK_SHIFT => Code::ShiftLeft,
        VK_LSHIFT => Code::ShiftLeft,
        VK_RSHIFT => Code::ShiftRight,
        VK_MENU => Code::AltLeft,
        VK_LMENU => Code::AltLeft,
        VK_RMENU => Code::AltRight,
        VK_LWIN => Code::MetaLeft,
        VK_RWIN => Code::MetaRight,
        _ => return None,
    };
    Some(code)
}

/// A logical key.
#[derive(Debug, PartialEq, Clone)]
pub struct LogicalKey {
    /// The code of the key.
    ///
    /// Note that, because it is a logical key, it does not respect the W3C specification.
    pub code: Code,
    /// The character attached to the logical key.
    pub key: Option<char>,
}

impl LogicalKey {
    /// Create a logical key from a physical key.
    ///
    /// The physical key should be a [`KeyboardEvent.code`] from the W3C specification.
    ///
    /// [`KeyboardEvent.code`]: https://w3c.github.io/uievents-code/#code-value-tables
    pub fn from_physical_key(code: &str) -> Option<Self> {
        let scancode = code_to_scancode(code)?;

        let virtual_key = unsafe { MapVirtualKeyExW(scancode as u32, MAPVK_VSC_TO_VK as _, None) };
        if virtual_key == 0 {
            return None;
        };

        let code = virtual_key_to_code(virtual_key)?;

        // Get the character associated with the virtual key.
        let key = unsafe { MapVirtualKeyExW(virtual_key, MAPVK_VK_TO_CHAR as _, None) };
        let key = match key {
            0 => None,
            _ => char::from_u32(key & 0xffff),
        };

        Some(Self { code, key })
    }
}
