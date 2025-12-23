// SPDX-FileCopyrightText: 2025 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: MPL-2.0

use std::ffi::{OsStr, c_void};

use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_SUCCESS};
use windows::Win32::System::Registry::{
    HKEY_CURRENT_USER, REG_SZ, RRF_RT_REG_SZ, RegDeleteKeyValueW, RegGetValueW, RegSetKeyValueW,
};
use windows::core::{HSTRING, PCWSTR, w};

use crate::{Error, Result};

const SUB_KEY_PATH: PCWSTR = w!(r"Software\Microsoft\Windows\CurrentVersion\Run");

pub struct AutoStart {
    app_name: HSTRING,
    command: HSTRING,
}

/// Construct a new [`AutoStart`].
///
/// This macro accepts a different number of patterns:
///
/// ```ignore
/// // With no argument, the app name will be the environment variable `CARGO_BIN_NAME`
/// // (i.e. the name of the binary currently being compiled), and there will be no args.
/// let autostart = AutoStart!()?;
///
/// // With one or more string literal, the app name will be the same as without argument, and the
/// // literal(s) will be passed as args.
/// let autostart = AutoStart!("--hidden")?;
/// let autostart = AutoStart!("--hidden", "--silent")?;
///
/// // With the `with` keyword, the app name will be the first string literal, and the args the
/// // subsequent string literal(s).
/// let autostart = AutoStart!(with "MyApp")?;
/// let autostart = AutoStart!(with "MyApp", "--hidden")?;
/// let autostart = AutoStart!(with "MyApp", "--hidden", "--silent")?;
/// ```
#[macro_export]
macro_rules! AutoStart {
    () => {
        $crate::AutoStart::new::<_, &str>(Some(env!("CARGO_BIN_NAME")), None)
    };
    ($($arg:literal),+) => {
        $crate::AutoStart::new(Some(env!("CARGO_BIN_NAME")), Some(&[$($arg),+]))
    };
    (with $name:literal$(, $arg:literal)*) => {
        $crate::AutoStart::new::<_, &str>(Some($name), Some(&[$($arg),*]))
    };
}

impl AutoStart {
    /// Construct a new [`AutoStart`].
    ///
    /// If `app_name` is not specified, the app name will be the current executable name without the
    /// `.exe`.
    ///
    /// See also the [`AutoStart!`](AutoStart!) macro.
    pub fn new<Name: AsRef<str>, Arg: AsRef<OsStr>>(
        app_name: Option<Name>,
        args: Option<&[Arg]>,
    ) -> Result<Self> {
        let exe_path = std::env::current_exe().map_err(|err| Error::GettingExePath(err))?;

        let app_name: HSTRING = if let Some(app_name) = app_name {
            app_name.as_ref().into()
        } else {
            exe_path
                .file_prefix()
                .expect("path to exe should have a filename")
                .into()
        };

        let mut command = exe_path.into_os_string();
        if let Some(args) = args {
            for arg in args {
                let arg = arg.as_ref();
                command.reserve(arg.len() + 1);
                command.push(" ");
                command.push(arg);
            }
        }

        Ok(Self {
            app_name,
            command: command.into(),
        })
    }

    /// Check if the program will open on startup.
    pub fn is_set(&self) -> Result<bool> {
        match unsafe {
            RegGetValueW(
                HKEY_CURRENT_USER,
                SUB_KEY_PATH,
                &self.app_name,
                RRF_RT_REG_SZ,
                None,
                None,
                None,
            )
        } {
            ERROR_SUCCESS => Ok(true),
            ERROR_FILE_NOT_FOUND => Ok(false),
            err => Err(Error::GettingRegistryValue(err.into())),
        }
    }

    /// Set if the program should open on startup.
    pub fn set(&self, should_open: bool) -> Result<()> {
        unsafe {
            match if should_open {
                // This is here to denote that every method call to `self.command` is a call on a
                // slice of `u16`. So use `command` instead to make it explicit.
                let command: &[u16] = &*self.command;

                // SAFETY: `lpdata` requires the string to be null terminated. Even though the slice
                // given to us above stops before the null char, the char is still present in the
                // next memory slot, and `c_void` doesn't care about slices' length.
                let lpdata = Some(command.as_ptr() as *const c_void);

                // SAFETY: `cbdata` requires the size of `*lpdata` **in bytes**, including the
                // terminating null char.
                let len = (command.len() + 1) * std::mem::size_of::<u16>();

                RegSetKeyValueW(
                    HKEY_CURRENT_USER,
                    SUB_KEY_PATH,
                    &self.app_name,
                    REG_SZ.0,
                    lpdata,
                    len as _,
                )
            } else {
                RegDeleteKeyValueW(HKEY_CURRENT_USER, SUB_KEY_PATH, &self.app_name)
            } {
                ERROR_SUCCESS => Ok(()),
                err => Err(Error::SettingRegistryValue(err.into())),
            }
        }
    }
}
