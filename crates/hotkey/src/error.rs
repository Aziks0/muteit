// SPDX-FileCopyrightText: 2026 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

/// [`HotKeyManager`]'s errors.
///
/// [`HotKeyManager`]: crate::HotKeyManager
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("only one instance of `HotKeyManager` is allowed")]
    InstanceAlreadyCreated,
    #[error("registering class for hotkey manager proc window ({0})")]
    RegisteringClass(windows_core::Error),
    #[error("creating window for hotkey manager proc ({0})")]
    CreatingWindow(windows_core::Error),
    #[error("the hotkey is already registered")]
    HotKeyAlreadyRegistered,
    #[error("registering hotkey ({0})")]
    RegisteringHotKey(windows_core::Error),
    #[error("unregistering hotkey ({0})")]
    UnregisteringHotKey(windows_core::Error),
}
