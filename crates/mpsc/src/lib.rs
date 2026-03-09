// SPDX-FileCopyrightText: 2025 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: MPL-2.0

//! Wait-free multiproducer with single asynchronous consumer channel. Optimized for
//! low-contention.

mod loom;

mod slot;
use slot::*;

mod receiver;
pub use receiver::*;

mod sender;
pub use sender::*;

mod channel;
pub use channel::*;
