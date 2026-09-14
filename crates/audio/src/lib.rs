// SPDX-FileCopyrightText: 2025-2026 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: MPL-2.0

#[cfg(not(target_os = "windows"))]
compile_error!("This crate is only available on Windows.");

mod windows;

pub mod interface;

pub mod enumerator;

mod error;
pub use error::Error;

pub type Result<T> = std::result::Result<T, Error>;

pub use windows_core::Error as WindowsError;
pub use windows_core::Result as WindowsResult;
