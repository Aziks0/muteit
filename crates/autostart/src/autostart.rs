// SPDX-FileCopyrightText: 2025 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: MPL-2.0

use std::ffi::{OsStr, c_void};

use windows::Win32::Foundation::{ERROR_FILE_NOT_FOUND, ERROR_MORE_DATA, ERROR_SUCCESS};
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
    /// If `app_name` is `None`, the app name will be the _runtime_ executable name without the
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
    ///
    /// See also [`is_set_strict`](Self::is_set_strict) and
    /// [`is_set_autocorrect`](Self::is_set_autocorrect).
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

    fn is_set_strict_with_autocorrect_optional(&self, autocorrect: bool) -> Result<bool> {
        // We want the buffer to be the same size as our command. If the buffer is not large enough,
        // we get a `ERROR_MORE_DATA`, and so we know for sure it won't match with our command.
        // NOTE: The null char is not counted in `.len()`.
        let buffer_capacity = self.command.len() + 1;
        let mut buffer = Vec::<u16>::with_capacity(buffer_capacity);
        // `pcbdata` wants the size in **bytes**.
        let mut buffer_capacity_bytes = (buffer_capacity * std::mem::size_of::<u16>()) as u32;

        // SAFETY: The buffer can be written to for at least `buffer_capacity_bytes` bytes.
        match unsafe {
            RegGetValueW(
                HKEY_CURRENT_USER,
                SUB_KEY_PATH,
                &self.app_name,
                RRF_RT_REG_SZ,
                None,
                Some(buffer.as_mut_ptr() as _),
                Some(&mut buffer_capacity_bytes),
            )
        } {
            ERROR_SUCCESS => {
                // `buffer_capacity_bytes` is now the len of the buffer, still in bytes, including
                // the null char.
                let buffer_len = buffer_capacity_bytes as usize / std::mem::size_of::<u16>();
                if buffer_len == buffer_capacity {
                    // SAFETY: `buffer_len == buffer_capacity` and `RegGetValueW` wrote `buffer_len`
                    // UTF-16 chars.
                    // NOTE: We remove the null char from the len. It won't overflow because we will
                    // always have a non-empty command.
                    unsafe { buffer.set_len(buffer_len - 1) };

                    if *buffer == *self.command {
                        // We've got a match, no need to autocorrect anything.
                        return Ok(true);
                    }
                }

                // Getting here means that the key exists, but the value did not match ours.

                if autocorrect {
                    self.set(true)?;
                }

                // We've either autocorrected it and now it's a match, or we didn't and so it's not
                // a match. So autocorrect is equal to match.
                Ok(autocorrect)
            }
            ERROR_FILE_NOT_FOUND => Ok(false),
            ERROR_MORE_DATA => Ok(false),
            err => Err(Error::GettingRegistryValue(err.into())),
        }
    }

    /// Check if the program will open on startup.
    ///
    /// The path and args will be checked as well. If they do not match the current `AutoStart`,
    /// this method will return `false`.
    ///
    /// See also [`is_set`](Self::is_set) and [`is_set_autocorrect`](Self::is_set_autocorrect).
    #[inline]
    pub fn is_set_strict(&self) -> Result<bool> {
        self.is_set_strict_with_autocorrect_optional(false)
    }

    /// Check if the program will open on startup.
    ///
    /// The path and args will be checked as well. If they do not match the current `AutoStart`,
    /// this method will correct the error and return `true`.
    ///
    /// See also [`is_set`](Self::is_set) and [`is_set_strict`](Self::is_set_strict).
    #[inline]
    pub fn is_set_autocorrect(&self) -> Result<bool> {
        self.is_set_strict_with_autocorrect_optional(true)
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
