// SPDX-FileCopyrightText: 2025 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: MPL-2.0

#[cfg(not(target_os = "windows"))]
compile_error!("This crate is only available on Windows.");

mod device;
pub use device::Device;

pub mod enumerator;

mod error;
pub use error::Error;

pub mod events;

pub type Result<T> = std::result::Result<T, Error>;
