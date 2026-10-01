// SPDX-FileCopyrightText: Copyright 2022-2026 Tauri Programme within The Commons Conservancy
// SPDX-FileCopyrightText: 2026 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: Apache-2.0 OR MIT
// SPDX-License-Identifier: GPL-3.0-or-later
//
// SPDX-FileComment: Initially taken from the `global-hotkey` git repository
// <https://github.com/tauri-apps/global-hotkey.git>, revision
// `35b81e58ffb6b7c333851ee4b75121616ea0ab3b`, file
// `global-hotkey/src/platform_impl/windows/mod.rs`, licensed under `Apache-2.0 OR MIT`.
// Modifications apported are licensed under `GPL-3.0-or-later`.

use std::marker::PhantomData;
use std::{mem, ptr};

use windows_core::{PCWSTR, w};

use crate::windows::Win32::{
    CW_USEDEFAULT, CreateWindowExW, DefWindowProcW, DestroyWindow, ERROR_HOTKEY_ALREADY_REGISTERED,
    HMODULE, HWND, LPARAM, LRESULT, MOD_NOREPEAT, RegisterClassW, RegisterHotKey, UnregisterHotKey,
    WM_HOTKEY, WNDCLASSW, WPARAM, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW,
    WS_EX_TRANSPARENT, WS_OVERLAPPED,
};

use crate::{Channel, Error, HotKey, HotKeyEvent, Receiver, Result, utils};

/// Global hotkey manager.
///
/// This is used to register [`HotKey`]s and handle [`HotKeyEvent`]s.
///
/// An event loop must be running on the thread used to create [`HotKeyManager`], otherwise no
/// [`HotKeyEvent`] will be received.
///
/// # Examples
///
/// ```no_run
/// # use hotkey::{HotKey, HotKeyEvent, HotKeyManager, Modifiers, Result};
/// #
/// # fn main() -> Result<()> {
/// #
/// const hk: HotKey = HotKey::new('A' as _, Modifiers::SHIFT);
///
/// let manager = HotKeyManager::try_new()?;
/// manager.register(hk)?;
///
/// let rx = manager.receiver();
/// let event = rx.recv_blocking().unwrap();
///
/// match event {
///     HotKeyEvent::Pressed { hotkey } => {
///         println!("{:?} was pressed. Matches hk: {}", hotkey, hk == hotkey);
///     }
/// }
/// #
/// #   Ok(())
/// # }
/// ```
pub struct HotKeyManager {
    /// The window handle of the window that receives raw hotkey events.
    hwnd: HWND,
    /// The channel to send [`HotKeyEvent`]s.
    channel: Channel,
    // NOTE: Do not rely on `HWND` being `!Send`.
    // TODO: replace by `!Send` implementation when stabilized.
    // https://github.com/rust-lang/rust/issues/68318
    /// Marker for `!Send`.
    _not_send: PhantomData<*mut ()>,
}

impl Drop for HotKeyManager {
    fn drop(&mut self) {
        let _success = unsafe { DestroyWindow(self.hwnd) };

        #[cfg(debug_assertions)]
        _success
            .ok()
            .expect("`HotKeyManager` window to get destroyed.");
    }
}

impl HotKeyManager {
    /// Try to create a new [`HotKeyManager`].
    ///
    /// An event loop must be running on the thread used to call this method, otherwise no
    /// [`HotKeyEvent`] will be received.
    ///
    /// # Errors
    ///
    /// This will return an error if a previously created [`HotKeyManager`] is still live.
    pub fn try_new() -> Result<Self> {
        let channel = Channel::try_new()?;

        const INSTANCE_HANDLE: HMODULE = utils::get_instance_handle();
        const WINDOW_CLASS_NAME: PCWSTR = w!("hotkey_manager");

        const WINDOW_CLASS: WNDCLASSW = unsafe {
            WNDCLASSW {
                lpfnWndProc: Some(hotkey_manager_proc),
                lpszClassName: WINDOW_CLASS_NAME,
                hInstance: INSTANCE_HANDLE,
                // Cannot use `Default::default()`` in const context. Zeroing does the same so... anyway.
                ..mem::zeroed()
            }
        };

        let success = unsafe { RegisterClassW(&WINDOW_CLASS) };
        if success.0 == 0 {
            return Err(Error::RegisteringClass(windows_core::Error::from_thread()));
        }

        // WS_EX_TOOLWINDOW prevents this window from ever showing up in the taskbar, which we want
        // to avoid. If you remove this style, this window won't show up in the taskbar *initially*,
        // but it can show up at some later point. This can sometimes happen on its own after
        // several hours have passed, although this has proven difficult to reproduce.
        // Alternatively, it can be manually triggered by killing `explorer.exe` and then starting
        // the process back up. It is unclear why the bug is triggered by waiting for several hours.
        let window_style = WS_EX_NOACTIVATE | WS_EX_TRANSPARENT | WS_EX_LAYERED | WS_EX_TOOLWINDOW;

        let window_handle = unsafe {
            CreateWindowExW(
                window_style as _,
                WINDOW_CLASS_NAME,
                PCWSTR::null(),
                WS_OVERLAPPED as _,
                CW_USEDEFAULT,
                0,
                CW_USEDEFAULT,
                0,
                None,
                None,
                Some(INSTANCE_HANDLE),
                None,
            )
        };

        if ptr::eq(window_handle.0, ptr::null_mut()) {
            return Err(Error::CreatingWindow(windows_core::Error::from_thread()));
        }

        Ok(Self {
            hwnd: window_handle,
            channel,
            _not_send: PhantomData,
        })
    }

    /// Register a [`HotKey`].
    ///
    /// Use the receiver given by [`HotKeyManager::receiver`] to receive [`HotKeyEvent`]s.
    pub fn register(&self, hotkey: HotKey) -> Result<()> {
        let mods = hotkey.modifiers.bits() | MOD_NOREPEAT as u32;

        let success =
            unsafe { RegisterHotKey(Some(self.hwnd), hotkey.id() as _, mods, hotkey.virtual_key) };

        success.ok().map_err(|err| match err.code().0 {
            ERROR_HOTKEY_ALREADY_REGISTERED => Error::HotKeyAlreadyRegistered,
            _ => Error::RegisteringHotKey(err),
        })
    }

    /// Unregister a previously registered [`HotKey`].
    pub fn unregister(&self, hotkey: HotKey) -> Result<()> {
        let success = unsafe { UnregisterHotKey(Some(self.hwnd), hotkey.id() as _) };

        success.ok().map_err(Error::UnregisteringHotKey)
    }

    /// Get a receiver for this hotkey manager.
    ///
    /// It will receive [`HotKeyEvent`]s for every registered [`HotKey`].
    ///
    /// # Channel closing
    ///
    /// The receiver will get closed when its associated [`HotKeyManager`] is dropped.
    pub fn receiver(&self) -> Receiver<HotKeyEvent> {
        self.channel.receiver()
    }
}

unsafe extern "system" fn hotkey_manager_proc(
    window_handle: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message as _ {
        WM_HOTKEY => {
            let hotkey = HotKey::from_id(wparam.0 as _);
            let event = HotKeyEvent::Pressed { hotkey };

            // SAFETY: The window and `Channel` are owned—and created—by `HotKeyManager`. The window
            // is destroyed in the drop implementation of `HotKeyManager`, which happens before
            // `Channel` is dropped. So this procedure can only be called while `Channel` is
            // instantiated.
            unsafe { Channel::send_blocking(event) };

            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(window_handle, message, wparam, lparam) },
    }
}
