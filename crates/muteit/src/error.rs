// SPDX-FileCopyrightText: 2025 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! [`throw_error`] from outside `dioxus`'s runtime.

use std::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::task::Poll;

use dioxus::CapturedError;
use dioxus::core::throw_error;
use dioxus::prelude::*;
use futures_util::task::AtomicWaker;

static ERROR: Error = Error {
    error: AtomicPtr::new(ptr::null_mut()),
    waker: AtomicWaker::new(),
};

struct Error {
    error: AtomicPtr<CapturedError>,
    waker: AtomicWaker,
}

impl Error {
    /// Set an error if it hasn't already been done.
    fn set(&self, error: CapturedError) {
        let error = Box::into_raw(Box::new(error));
        if self
            .error
            .compare_exchange(ptr::null_mut(), error, Ordering::AcqRel, Ordering::Relaxed)
            .is_err()
        {
            // SAFETY: We created the box and turned it into raw just above.
            let _ = unsafe { Box::from_raw(error) };
        } else {
            self.waker.wake();
        }
    }

    fn get(&self) -> Option<&'static CapturedError> {
        let ptr = self.error.load(Ordering::Acquire);
        // SAFETY: The `CapturedError` has been leaked indefinitely.
        unsafe { ptr.as_ref() }
    }

    async fn get_async(&self) -> &'static CapturedError {
        core::future::poll_fn(|cx| {
            if let Some(error) = self.get() {
                return Poll::Ready(error);
            }

            self.waker.register(cx.waker());

            self.get().map(Poll::Ready).unwrap_or(Poll::Pending)
        })
        .await
    }
}

/// Set an error if it hasn't already been done.
#[cold]
pub fn set(error: impl Into<CapturedError>) {
    ERROR.set(error.into());
}

pub fn use_setup_error() {
    // Listen for incoming error.
    use_future(|| async {
        let error = ERROR.get_async().await;
        throw_error(error.clone());
    });
}
