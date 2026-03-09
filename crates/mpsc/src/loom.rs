// SPDX-FileCopyrightText: 2025 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: CC0-1.0

#![allow(dead_code)]

#[cfg(loom)]
mod inner {
    pub mod cell {
        pub use loom::cell::UnsafeCell;
    }

    pub mod sync {
        pub use loom::sync::Arc;

        pub mod atomic {
            pub use std::sync::atomic::Ordering;

            pub use loom::sync::atomic::{AtomicBool, AtomicUsize};
        }
    }

    pub mod future {
        use std::task::Waker;

        pub struct AtomicWaker(loom::future::AtomicWaker);

        impl AtomicWaker {
            pub fn new() -> Self {
                Self(loom::future::AtomicWaker::new())
            }

            pub fn register(&self, waker: &Waker) {
                self.0.register_by_ref(waker);
            }

            pub fn wake(&self) {
                self.0.wake();
            }
        }
    }
}

#[cfg(not(loom))]
mod inner {
    pub mod cell {
        pub struct UnsafeCell<T>(std::cell::UnsafeCell<T>);

        impl<T> UnsafeCell<T> {
            pub const fn new(data: T) -> UnsafeCell<T> {
                UnsafeCell(std::cell::UnsafeCell::new(data))
            }

            pub fn with<R>(&self, f: impl FnOnce(*const T) -> R) -> R {
                f(self.0.get())
            }

            pub fn with_mut<R>(&self, f: impl FnOnce(*mut T) -> R) -> R {
                f(self.0.get())
            }
        }
    }

    pub mod sync {
        pub use std::sync::Arc;

        pub mod atomic {
            pub use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
        }
    }

    pub mod future {
        pub use futures_util::task::AtomicWaker;
    }
}

pub(crate) use self::inner::*;
