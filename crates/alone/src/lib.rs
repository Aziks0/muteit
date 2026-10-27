// SPDX-FileCopyrightText: 2025-2026 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: MPL-2.0

//! Alone
//!
//! Check if there is already an instance of your program running on the user's machine.
//!
//! See [is_alone!].

#[cfg(not(target_os = "windows"))]
compile_error!("This crate is only available on Windows.");

mod windows;

/// Check if there is already an instance of your program running on the user's machine.
///
/// This macro will return `false` if there is a program on the user's machine that has already
/// called this macro with the **same** identifier. So use one that is **UNIQUE** (i.e. that
/// uniquely identifies your app), like a
/// [UUID](https://en.wikipedia.org/wiki/Universally_unique_identifier).
///
/// # Examples
///
/// ```
/// # use alone::*;
/// let am_i_alone = is_alone!("436a1306-cf3a-4b24-960f-dd1fe948463a").unwrap();
/// if !am_i_alone {
///     std::process::exit(0);
/// }
/// ```
///
/// The macro also accepts constant values and variables:
///
/// ```
/// # use alone::*;
/// let am_i_alone = is_alone!(env!("APP_ID"));
/// let am_i_alone = is_alone!(concat!("it", " ", "works"));
/// const APP_ID: &str = "436a1306-cf3a-4b24-960f-dd1fe948463c";
/// let am_i_alone = is_alone!(APP_ID);
/// ```
#[macro_export]
macro_rules! is_alone {
    ($unique:expr) => {{
        const MUTEX_NAME: $crate::__private::PCWSTR = {
            const C_STR: &str = $crate::__private::concat!($unique, "\0");
            const WSTR: &$crate::__private::Utf16Str = $crate::__private::utf16str!(C_STR);

            $crate::__private::PCWSTR::from_raw(WSTR.as_ptr())
        };

        $crate::__private::is_alone(MUTEX_NAME)
    }};
}

// Export for macro.
#[doc(hidden)]
pub mod __private {
    #[doc(hidden)]
    pub use constcat::concat;
    #[doc(hidden)]
    pub use widestring::{Utf16Str, utf16str};
    #[doc(hidden)]
    pub use windows_core::PCWSTR;

    #[doc(hidden)]
    #[inline]
    pub fn is_alone(unique: PCWSTR) -> Result<bool, String> {
        use crate::windows::Win32::{CreateMutexW, ERROR_ALREADY_EXISTS};
        use windows_core::{HRESULT, WIN32_ERROR};

        // We don't care about the returned handle. We just care about whether there is an error or not.
        // Windows won't destroy the mutex until we either call `CloseHandle` or the program exits.
        let failed = unsafe { CreateMutexW(None, false, unique).0.is_null() };

        // Call `GetLastError`.
        let err = HRESULT::from_thread();

        if failed {
            return Err(err.message());
        }

        // If the mutex exists already, `CreateMutexW` returns a valid handle and `GetLastError` returns
        // `ERROR_ALREADY_EXISTS`. If it's a new mutex, then there should be no error.
        // https://learn.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-createmutexw#return-value
        Ok(err != WIN32_ERROR(ERROR_ALREADY_EXISTS as _).into())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn i_am_alone() {
        let am_i_alone =
            is_alone!("436a1306-cf3a-4b24-960f-dd1fe948463d").expect("`CreateMutexW` should work.");
        assert!(am_i_alone);
    }

    #[test]
    fn i_am_not_alone() {
        let _ =
            is_alone!("436a1306-cf3a-4b24-960f-dd1fe948463e").expect("`CreateMutexW` should work.");
        let am_i_alone =
            is_alone!("436a1306-cf3a-4b24-960f-dd1fe948463e").expect("`CreateMutexW` should work.");
        assert!(!am_i_alone)
    }

    #[test]
    fn accept_constant() {
        _ = is_alone!(env!("CARGO_PKG_NAME"));
        _ = is_alone!(concat!("it", "should", "work"));
        const APP_ID: &str = "436a1306-cf3a-4b24-960f-dd1fe948463f";
        _ = is_alone!(APP_ID);
    }
}
