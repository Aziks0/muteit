// SPDX-FileCopyrightText: 2026 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: MPL-2.0

use crate::Channel;
use crate::loom::sync::Arc;
use crate::loom::sync::atomic::Ordering;

/// A sender for a [`Channel`].
///
/// Note that this can be cloned.
pub struct Sender<T> {
    /// The channel.
    channel: Arc<Channel<T>>,
}

// Implement `Clone` manually because `derive(Clone)` only does it for `T: Clone`.
impl<T> Clone for Sender<T> {
    fn clone(&self) -> Self {
        Self {
            channel: self.channel.clone(),
        }
    }
}

impl<T> Sender<T> {
    /// Create a new [`Sender`] for a channel.
    pub(crate) fn new(channel: Arc<Channel<T>>) -> Self {
        Self { channel }
    }

    /// Send a value if the channel isn't full.
    ///
    /// This returns an error containing the given value if the channel is full.
    pub fn try_send(&self, value: T) -> Result<(), T> {
        let mut tail = self.channel.tail.load(Ordering::Relaxed);

        loop {
            // This needs to acquire any previous writes on the queue from `Receiver::try_receive`.
            let head = self.channel.head.load(Ordering::Acquire);

            // Check if it is full.
            if tail.wrapping_sub(head) >= self.channel.capacity {
                return Err(value);
            }

            let new_tail = tail.wrapping_add(1);

            // Try to take ownership of the slot.
            // NOTE: I'm not sure about the ordering.
            if let Err(real_tail) = self.channel.tail.compare_exchange(
                tail,
                new_tail,
                Ordering::AcqRel,
                Ordering::Relaxed,
            ) {
                // We need a new slot.
                tail = real_tail;
                continue;
            }

            // Get the index of the slot.
            let index = tail % self.channel.capacity;

            // Set the slot.
            // SAFETY:
            // - We checked that the queue was not full, so our slot is empty.
            // - We have exclusive access to this slot, so no concurrent calls possible.
            unsafe { self.channel.queue[index].set(value) };

            // Wake after setting the slot.
            self.channel.waker.wake();

            return Ok(());
        }
    }
}
