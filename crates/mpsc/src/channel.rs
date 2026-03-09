// SPDX-FileCopyrightText: 2025 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: MPL-2.0

use crossbeam_utils::CachePadded;

use crate::loom::future::AtomicWaker;
use crate::loom::sync::Arc;
use crate::loom::sync::atomic::AtomicUsize;
use crate::{Receiver, Sender, Slot};

/// A bounded MPSC channel.
///
/// Use [channel] to create one.
///
/// See also [`Sender`] and [`Receiver`].
pub struct Channel<T> {
    /// The heap-allocated queue.
    pub(crate) queue: Box<[Slot<T>]>,
    /// The capacity of the channel.
    ///
    /// Note that this _must_ be non-zero.
    pub(crate) capacity: usize,
    /// The head of the queue.
    ///
    /// Points to the slot it tries to read.
    ///
    /// Note that this isn't the _real_ tail. `% CAPACITY` is needed to get the real one.
    pub(crate) head: CachePadded<AtomicUsize>,
    /// The tail of the queue.
    ///
    /// Points to the next slot available.
    ///
    /// Note that this isn't the _real_ tail. `% CAPACITY` is needed to get the real one.
    pub(crate) tail: CachePadded<AtomicUsize>,
    /// Waker for async operation.
    pub(crate) waker: AtomicWaker,
}

impl<T> Channel<T> {
    /// Create a new channel.
    ///
    /// The capacity must be `> 0` and cannot be changed afterwards.
    ///
    /// # Panics
    ///
    /// Panics if `capacity == 0`.
    fn new(capacity: usize) -> Self {
        // This removes the need for checking `capacity != 0` on every send and receive call.
        assert!(capacity > 0, "channel capacity must be > 0");

        let queue = (0..capacity).map(|_| Slot::new()).collect();

        Self {
            queue,
            capacity,
            head: CachePadded::new(AtomicUsize::new(0)),
            tail: CachePadded::new(AtomicUsize::new(0)),
            waker: AtomicWaker::new(),
        }
    }
}

/// Create a new bounded MPSC channel.
///
/// The capacity must be `> 0` and cannot be changed afterwards.
///
/// # Panics
///
/// Panics if `capacity == 0`.
///
/// See also [`Sender`] and [`Receiver`].
pub fn channel<T>(capacity: usize) -> (Sender<T>, Receiver<T>) {
    let channel = Arc::new(Channel::new(capacity));
    (Sender::new(channel.clone()), Receiver::new(channel, 0))
}
