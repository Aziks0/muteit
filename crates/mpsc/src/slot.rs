// SPDX-FileCopyrightText: 2026 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: MPL-2.0

use std::mem::{self, MaybeUninit};

use crate::loom::cell::UnsafeCell;
use crate::loom::sync::atomic::{AtomicBool, Ordering};

/// A slot for a channel.
pub(crate) struct Slot<T> {
    /// The value of this slot, which might be uninitialized.
    value: UnsafeCell<MaybeUninit<T>>,
    /// Is the slot empty?
    is_empty: AtomicBool,
}

unsafe impl<T> Sync for Slot<T> {}

impl<T> Slot<T> {
    /// Create a new uninitialized slot.
    pub(crate) fn new() -> Self {
        Self {
            value: UnsafeCell::new(MaybeUninit::uninit()),
            is_empty: AtomicBool::new(true),
        }
    }

    /// Get the value in the slot, if there is one.
    ///
    /// # SAFETY:
    ///
    /// The caller has the responsibility for the following guarantees:
    /// - There must be no concurrent calls to this method.
    /// - If the slot is not empty, there must be no concurrent calls between this method and
    ///   [`set`].
    ///
    /// [`set`]: Self::set
    pub(crate) unsafe fn try_get(&self) -> Option<T> {
        // Acquire the release from `set`.
        let is_empty = self.is_empty.load(Ordering::Acquire);
        if is_empty {
            return None;
        }

        let value = self.value.with_mut(|value| {
            // SAFETY:
            // The aliasing rules are followed, because:
            // 1. The caller guarantees that there are no concurrent calls.
            // 2. We only have 1 reference in this function.
            let value = unsafe { &mut *value };
            mem::replace(value, MaybeUninit::uninit())
        });

        // No ordering required because `is_empty` is only read by this method and the caller
        // guarantees there are no concurrent calls.
        self.is_empty.store(true, Ordering::Relaxed);

        // SAFETY: We checked that the slot was set.
        let value = unsafe { value.assume_init() };

        Some(value)
    }

    /// Set the value in the slot.
    ///
    /// # SAFETY:
    ///
    /// The caller has the responsibility for the following guarantees:
    /// - The slot must be empty.
    /// - There must be no concurrent calls to this method.
    ///
    /// [`try_get`]: Self::try_get
    pub(crate) unsafe fn set(&self, new_value: T) {
        self.value.with_mut(|value| {
            // SAFETY: The caller guarantees that there are no concurrent calls.
            unsafe { *value = MaybeUninit::new(new_value) };
        });

        // Release the write for `try_get`.
        self.is_empty.store(false, Ordering::Release);
    }
}

impl<T> Drop for Slot<T> {
    fn drop(&mut self) {
        // No memory ordering needed in drop.
        if self.is_empty.load(Ordering::Relaxed) {
            return;
        }

        self.value.with_mut(|value| {
            // SAFETY: We checked that the slot was not empty.
            unsafe {
                (*value).assume_init_drop();
            }
        });
    }
}

#[cfg(all(test, not(loom)))]
mod tests {
    use super::*;

    #[test]
    fn no_get_if_empty() {
        let slot: Slot<()> = Slot::new();
        unsafe {
            assert_eq!(slot.try_get(), None);
        }
    }

    #[test]
    fn get_after_set() {
        let slot = Slot::new();
        unsafe {
            slot.set(0);
            assert_eq!(slot.try_get(), Some(0));
        }
    }

    #[test]
    fn get_clears_slot() {
        let slot = Slot::new();
        unsafe {
            slot.set(0);
            assert_eq!(slot.try_get(), Some(0));
            assert_eq!(slot.try_get(), None);
        }
    }

    #[test]
    #[cfg(miri)]
    fn no_leak() {
        let slot = Slot::new();
        unsafe {
            let _ = slot.set(String::from("drop"));
        }
    }

    #[test]
    #[cfg(miri)]
    fn no_drop_if_empty() {
        let _slot: Slot<String> = Slot::new();
    }

    #[test]
    #[cfg(miri)]
    fn drop_on_get() {
        let slot = Slot::new();
        unsafe {
            // We use a string 'cause we need something that gets dropped.
            slot.set(String::from("drop"));
            let _ = slot.try_get();
        }
    }
}
