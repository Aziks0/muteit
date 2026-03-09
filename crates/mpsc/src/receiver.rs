// SPDX-FileCopyrightText: 2025 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: MPL-2.0

use std::cell::Cell;
use std::pin::Pin;
use std::task::{Context, Poll};

use crate::Channel;
use crate::loom::sync::Arc;
use crate::loom::sync::atomic::Ordering;

/// A receiver for a [`Channel`].
pub struct Receiver<T> {
    /// The channel.
    channel: Arc<Channel<T>>,
    /// Cached head of the queue.
    ///
    /// See also [`Channel::head`].
    // As this is a single consumer channel, there is no need to _load_ the atomic head.
    // Note that using a `Cell` marks `Receiver` as `!Sync`, which is what we want.
    cache_head: Cell<usize>,
}

impl<T> Receiver<T> {
    /// Create a new [`Receiver`] for a channel.
    pub(crate) fn new(channel: Arc<Channel<T>>, tail: usize) -> Self {
        Self {
            channel,
            cache_head: Cell::new(tail),
        }
    }

    // Try to receive a value.
    pub fn try_receive(&self) -> Option<T> {
        let head = self.cache_head.get();

        // Get the index of the slot.
        let index = head % self.channel.capacity;

        // SAFETY:
        // - `Self` is `!Sync` and `!Clone`, so no concurrent calls possible with itself.
        // - `Sender::try_send` only sets slots if the head as moved passed it, and we only move
        //   head after this line.
        let value = unsafe { self.channel.queue[index].try_get()? };

        // Advance the head.
        let new_head = head.wrapping_add(1);
        // This needs to release writes on the queue for `Sender::try_send`.
        self.channel.head.store(new_head, Ordering::Release);
        self.cache_head.set(new_head);

        Some(value)
    }

    // Receive a value asynchronously.
    pub async fn receive(&self) -> T {
        ReceiveAsync(self).await
    }
}

pub struct ReceiveAsync<'a, T>(&'a Receiver<T>);

impl<'a, T> Future for ReceiveAsync<'a, T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Try to get a value if the channel is not empty.
        if let Some(value) = self.0.try_receive() {
            return Poll::Ready(value);
        }

        // Channel is empty, so register the waker.
        self.0.channel.waker.register(cx.waker());

        // Try one last time to get a value, in case we got raced while registering the waker.
        self.0
            .try_receive()
            .map(Poll::Ready)
            .unwrap_or(Poll::Pending)
    }
}
