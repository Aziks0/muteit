// SPDX-FileCopyrightText: 2026 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::cell::UnsafeCell;
use std::marker::PhantomData;
use std::ptr;
use std::sync::atomic::{AtomicBool, Ordering};

use async_channel::{Receiver, Sender};

use crate::{Error, HotKeyEvent, Result};

/// Wrapper around a `*mut`.
struct Ptr<T>(UnsafeCell<*mut T>);

// SAFETY: It's our job to make it safe while cousuming the ptr. Every method is marked as `unsafe`.
unsafe impl<T> Sync for Ptr<T> {}

impl<T> Ptr<T> {
    #[inline(always)]
    const fn new() -> Self {
        Ptr(UnsafeCell::new(ptr::null_mut()))
    }

    #[inline(always)]
    unsafe fn get(&self) -> *mut T {
        unsafe { *self.0.get() }
    }

    #[inline(always)]
    unsafe fn set(&self, ptr: *mut T) {
        unsafe { *self.0.get() = ptr };
    }

    #[inline(always)]
    unsafe fn replace(&self, new_ptr: *mut T) -> *mut T {
        let ptr = unsafe { *self.0.get() };
        unsafe { *self.0.get() = new_ptr }
        ptr
    }

    #[inline(always)]
    unsafe fn take(&self) -> *mut T {
        unsafe { self.replace(ptr::null_mut()) }
    }
}

struct ChannelInner {
    tx: Sender<HotKeyEvent>,
    rx: Receiver<HotKeyEvent>,
}

/// Is [`Channel`] currently instantiated?
static INSTANTIATED: AtomicBool = AtomicBool::new(false);

/// The global channel.
static CHANNEL: Ptr<ChannelInner> = Ptr::new();

/// Global channel instantiation.
#[derive(Debug)]
pub struct Channel {
    /// We want [`Channel`] to only be constructed using [`Channel::try_new`].
    _private: PhantomData<()>,
}

impl Drop for Channel {
    fn drop(&mut self) {
        // Take the ptr so we can drop it.
        // SAFETY: In `Drop`, so we're the only ones that can access the channel.
        let previous_channel = unsafe { CHANNEL.take() };
        debug_assert!(!previous_channel.is_null());

        // This needs to release the write to `CHANNEL`.
        let previous_instantiated = INSTANTIATED.swap(false, Ordering::Release);
        debug_assert!(previous_instantiated);

        // NOTE: This is ordered after the `INSTANTIATED` swap so this can get reordered by the
        // compiler/cpu.
        // SAFETY: We've created the box ourselves and there's no reason for the ptr to be `null`
        // cause we're in `Drop`.
        let channel = unsafe { Box::from_raw(previous_channel) };
        drop(channel);
    }
}

impl Channel {
    /// Try to create a new channel.
    ///
    /// # Errors
    ///
    /// Returns an error if an instance has already been created, and still hasn't been dropped.
    pub fn try_new() -> Result<Self> {
        // Acquire any previous write to `CHANNEL`.
        INSTANTIATED
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Acquire)
            .map_err(|_| Error::InstanceAlreadyCreated)?;

        // Create a new channel and leak it.
        let (tx, rx) = async_channel::unbounded();
        let channel = Box::new(ChannelInner { tx, rx });
        let channel_ptr = Box::into_raw(channel);

        // Set the new channel.
        // SAFETY: This is synced with `INSTANTIATED`, so the ptr can only be `null` here, and we
        // are the only ones accessing it.
        unsafe {
            debug_assert!(CHANNEL.get().is_null());
            CHANNEL.set(channel_ptr);
        }

        Ok(Self {
            _private: PhantomData,
        })
    }

    /// Get a receiver for this channel.
    ///
    /// # Channel closing
    ///
    /// The receiver might get closed at anytime.
    pub fn receiver(&self) -> Receiver<HotKeyEvent> {
        // SAFETY: This takes `&self` and `CHANNEL` is only modified in `Drop`.
        let channel = unsafe { CHANNEL.get() };
        debug_assert!(!channel.is_null());

        // SAFETY:
        //   - `CHANNEL` can only be `null` when `self` has been dropped.
        //   - We've made the ptr ourselves. It's a rust struct so it's aligned.
        let rx = unsafe { &(*channel).rx };

        rx.clone()
    }

    /// Send a message to this channel by blocking until the operation is done.
    ///
    /// # Safety
    ///
    /// The caller must unsure that the channel is instantiated, in other words, that
    /// `Channel::try_new()` got called and the returned channel has not been dropped (or in the
    /// process of being dropped).
    pub unsafe fn send_blocking(event: HotKeyEvent) {
        // SAFETY: `CHANNEL` is only modified in `Drop` and, by the caller safety contract, this
        // method cannot be called while `Self` is being dropped.
        let channel = unsafe { CHANNEL.get() };
        debug_assert!(!channel.is_null());

        // SAFETY:
        //   - By the caller safety contract, `CHANNEL` cannot be `null`.
        //   - We've made the ptr ourselves. It's a rust struct so it's aligned.
        let tx = unsafe { &(*channel).tx };

        // NOTE: The channel cannot be closed, otherwise our ptr would be dangling.
        let _ = tx.send_blocking(event);
    }
}

#[cfg(test)]
#[serial_test::serial]
mod tests {
    use std::assert_matches;

    use crate::{HotKey, Modifiers};

    use super::*;

    #[test]
    fn channel_created_on_first_instance() {
        let channel = Channel::try_new();
        assert_matches!(channel, Ok(_));
    }

    #[test]
    fn channel_error_on_multiple_instances() {
        let _channel = Channel::try_new().unwrap();

        let not_channel1 = Channel::try_new();
        assert_matches!(not_channel1, Err(Error::InstanceAlreadyCreated));

        let not_channel2 = Channel::try_new();
        assert_matches!(not_channel2, Err(Error::InstanceAlreadyCreated));
    }

    #[test]
    fn channel_created_after_previous_dropped() {
        let channel = Channel::try_new().unwrap();
        drop(channel);

        let channel = Channel::try_new();
        assert_matches!(channel, Ok(_));
    }

    #[test]
    fn inner_channel_closed_after_drop() {
        let channel = Channel::try_new().unwrap();
        let rx = channel.receiver();

        drop(channel);

        assert!(rx.is_closed());
    }

    #[test]
    fn send_works() {
        const EVENT: HotKeyEvent = HotKeyEvent::Pressed {
            hotkey: HotKey::new(0, Modifiers::NONE),
        };

        let channel = Channel::try_new().unwrap();

        unsafe { Channel::send_blocking(EVENT) };

        let rx = channel.receiver();
        assert!(rx.recv_blocking().unwrap() == EVENT);
    }

    #[test]
    #[cfg(miri)]
    fn no_leak_on_drop() {
        let _ = Channel::try_new().unwrap();
    }
}
