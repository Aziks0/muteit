// SPDX-FileCopyrightText: 2026 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

//! Throw error from outside [`iced`]'s runtime.

use std::ptr;
use std::sync::atomic::{AtomicPtr, Ordering};
use std::task::Poll;

use iced::Subscription;
use iced::futures::stream;
use iced::futures::task::AtomicWaker;

/// The global error tracker.
static ERROR: Error = Error {
    error: AtomicPtr::new(ptr::null_mut()),
    waker: AtomicWaker::new(),
};

/// Track the first error that happened outside of [`iced`]'s runtime.
struct Error {
    error: AtomicPtr<anyhow::Error>,
    waker: AtomicWaker,
}

impl Error {
    /// Set an error if it hasn't already been done.
    fn set(&self, error: anyhow::Error) {
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

    fn get(&self) -> Option<&'static anyhow::Error> {
        let ptr = self.error.load(Ordering::Acquire);
        // SAFETY: The `Error` has been leaked indefinitely.
        unsafe { ptr.as_ref() }
    }

    async fn get_async(&self) -> &'static anyhow::Error {
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

/// Set the global error if it hasn't already been done.
#[cold]
pub fn set(error: impl Into<anyhow::Error>) {
    ERROR.set(error.into());
}

/// Subscribe to global error events.
pub fn subscription() -> Subscription<&'static anyhow::Error> {
    Subscription::run(|| stream::once(async { ERROR.get_async().await }))
}
