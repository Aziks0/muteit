// SPDX-FileCopyrightText: 2026 Benjamin DIDIER <contact@aziks.aleeas.com>
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
//! use autostart::AutoStarter;
//!
//! let autostart = AutoStarter!("--hidden")?;
//! autostart.set(true)?;
//! # Ok::<(), Error>(())
//! ```
//!
//! See [`AutoStarter`] and the [`AutoStarter!`](AutoStarter!) macro for more information.

#[cfg(not(target_os = "windows"))]
compile_error!("This crate is only available on Windows.");

mod windows;

mod autostart;
pub use autostart::AutoStarter;

mod error;
pub use error::Error;

pub type Result<T> = std::result::Result<T, Error>;
