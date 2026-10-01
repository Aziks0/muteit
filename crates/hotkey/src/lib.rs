// SPDX-FileCopyrightText: 2026 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! HotKey
//!
//! Register HotKeys on Windows.
//!
//! See [`HotKeyManager`].

#[cfg(not(target_os = "windows"))]
compile_error!("This crate is only available on Windows.");

mod utils;
mod windows;

mod channel;
pub(crate) use channel::Channel;

mod event;
pub use event::HotKeyEvent;

mod hotkey;
pub use hotkey::HotKey;

mod manager;
pub use manager::HotKeyManager;

mod modifiers;
pub use modifiers::Modifiers;

mod error;
pub use error::Error;

pub type Result<T> = std::result::Result<T, Error>;

// Re-export cause we return it in `HotKeyManager`.
pub use async_channel::Receiver;
