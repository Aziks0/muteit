// SPDX-FileCopyrightText: 2025 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: MPL-2.0

//! Autostart
//!
//! Set your application to automatically start on computer startup.
//!
//! # Examples
//!
//! ```no_run
//! # use autostart::Error;
//! use autostart::AutoStart;
//!
//! let autostart = AutoStart!("--hidden")?;
//! autostart.set(true)?;
//! # Ok::<(), Error>(())
//! ```
//!
//! See [`AutoStart`] and the [`AutoStart!`](AutoStart!) macro for more information.

#[cfg(not(target_os = "windows"))]
compile_error!("This crate is only available on Windows.");

mod autostart;
pub use autostart::AutoStart;

mod error;
pub use error::Error;

pub type Result<T> = std::result::Result<T, Error>;
