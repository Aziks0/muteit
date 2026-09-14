// SPDX-FileCopyrightText: 2026 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: MPL-2.0

#![allow(clippy::needless_doctest_main)]

//! Setup your app to use dark mode.
//!
//! # Examples
//!
//! ```
//! fn main() {
//!     dark::setup();
//! }
//! ```

#[cfg(not(target_os = "windows"))]
compile_error!("This crate is only available on Windows.");

use std::mem;

use windows_core::{PCSTR, w};

use crate::windows::Win32::{
    FreeLibrary, GetProcAddress, LoadLibraryW, OSVERSIONINFOW, RtlGetVersion, STATUS_SUCCESS,
};

mod windows;

fn is_dark_mode_available() -> bool {
    let mut version_info = OSVERSIONINFOW::default();

    let res = unsafe { RtlGetVersion(&mut version_info as *mut _) };

    res == STATUS_SUCCESS
        // For Windows 12 :)
        && (version_info.dwMajorVersion > 10
            // Windows 10 1903 and onward (including Windows 11).
            || (version_info.dwMajorVersion == 10 && version_info.dwBuildNumber >= 18362))
}

/// Setup the app to use dark mode.
pub fn setup() {
    if !is_dark_mode_available() {
        return;
    }

    #[allow(dead_code)]
    #[repr(C)]
    enum PreferredAppMode {
        Default,
        AllowDark,
        ForceDark,
        ForceLight,
        Max,
    }

    const SET_PREFERRED_APP_MODE_ORDINAL: PCSTR = PCSTR(135 as *const _);

    type SetPreferredAppModeFn = unsafe extern "system" fn(PreferredAppMode) -> PreferredAppMode;

    unsafe {
        let module = LoadLibraryW(w!("uxtheme.dll"));

        if module.0.is_null() {
            return;
        }

        let Some(func) = GetProcAddress(module, SET_PREFERRED_APP_MODE_ORDINAL) else {
            return;
        };

        let set_preferred_app_mode: SetPreferredAppModeFn = mem::transmute(func);

        set_preferred_app_mode(PreferredAppMode::AllowDark);

        let _ = FreeLibrary(module);
    }
}
