// SPDX-FileCopyrightText: 2025 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: MPL-2.0

//! Alone
//!
//! Check if there is already an instance of your program running on the user's machine.
//!
//! See [is_alone!].

/// Check if there is already an instance of your program running on the user's machine.
///
/// This macro will return `false` if there is a program on the user's machine that has already
/// called this macro with the **same** identifier. So use one that is **UNIQUE** (i.e. that
/// uniquely identifies your app), like a
/// [UUID](https://en.wikipedia.org/wiki/Universally_unique_identifier).
///
/// # Example
///
/// ```ignore
/// let am_i_alone = is_alone!("436a1306-cf3a-4b24-960f-dd1fe948463e").unwrap();
/// if !am_i_alone {
///     std::process::exit(0);
/// }
/// ```
#[macro_export]
macro_rules! is_alone {
    ($unique:literal) => {{
        const MUTEX_NAME_STR: &str = $crate::__private::concat!(r"Global\", $unique, "\0");
        const MUTEX_NAME_WSTR: &$crate::__private::Utf16Str =
            $crate::__private::utf16str!(MUTEX_NAME_STR);
        const MUTEX_NAME_PTR: *const u16 = MUTEX_NAME_WSTR.as_ptr();
        const MUTEX_NAME_PCWSTR: $crate::__private::PCWSTR =
            $crate::__private::PCWSTR::from_raw(MUTEX_NAME_PTR);
        $crate::__private::is_alone(MUTEX_NAME_PCWSTR)
    }};
}

#[doc(hidden)]
pub mod __private {
    // Re-export for macro.
    pub use crate::is_alone;
    pub use std::concat;
    pub use widestring::{Utf16Str, utf16str};
    pub use windows::core::PCWSTR;

    use windows::Win32::Foundation::ERROR_ALREADY_EXISTS;
    use windows::Win32::System::Threading::CreateMutexW;

    #[inline(always)]
    pub fn is_alone(unique: PCWSTR) -> Result<bool, String> {
        // We don't care about the returned handle. We just care about whether there is an error or not.
        // Windows won't destroy the mutex until we either call `CloseHandle` or the program exits.
        unsafe { CreateMutexW(None, false, unique).map_err(|err| err.message())? };

        // If the mutex exists already, `CreateMutexW` returns a valid handle and `GetLastError` returns
        // `ERROR_ALREADY_EXISTS`. If it's a new mutex, then there should be no error.
        // https://learn.microsoft.com/en-us/windows/win32/api/synchapi/nf-synchapi-createmutexw#return-value
        Ok(windows::core::Error::from_thread() != ERROR_ALREADY_EXISTS.into())
    }
}

#[cfg(all(test, target_os = "windows"))]
mod tests {
    #[test]
    fn i_am_alone() {
        let am_i_alone =
            is_alone!("436a1306-cf3a-4b24-960f-dd1fe948463e").expect("`CreateMutexW` to work");
        assert!(am_i_alone);
    }

    #[test]
    fn i_am_not_alone() {
        let _ = is_alone!("436a1306-cf3a-4b24-960f-dd1fe948463e").expect("`CreateMutexW` to work");
        let am_i_alone =
            is_alone!("436a1306-cf3a-4b24-960f-dd1fe948463e").expect("`CreateMutexW` to work");
        assert!(!am_i_alone)
    }
}
