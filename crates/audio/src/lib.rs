// SPDX-FileCopyrightText: 2025 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: MPL-2.0

#[cfg(not(target_os = "windows"))]
compile_error!("This crate is only available on Windows.");

mod device;
pub use device::Device;

mod enumerator;
pub use enumerator::DeviceEnumerator;

mod error;
pub use error::Error;

pub type Result<T> = std::result::Result<T, Error>;
