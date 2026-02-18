// SPDX-FileCopyrightText: 2025 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: MPL-2.0

use std::cell::UnsafeCell;
use std::ptr;

use windows::Win32::Foundation::E_INVALIDARG;
use windows::Win32::Media::Audio::{IMMNotificationClient, IMMNotificationClient_Impl};
use windows::core::implement;

use crate::WindowsResult;
use crate::enumerator::{FlowVariant, Role, State};

/// Implementations must follow these rules:
///     - The methods must be **non-blocking**. They should never wait on a synchronization
///     primitives.
///     - The methods should never call their respective subscribe and unsubscribe method.
///     - The methods should never drop the final reference of their respective API.
///
/// The implemented methods will be called by **different threads** than the one used for the
/// subscription.
#[allow(unused_variables)]
pub trait AudioDeviceEvents: Send + Sync {
    /// Called when the default input or output device changes.
    ///
    /// Note that there might be no default device available.
    fn on_default_device_change(
        &mut self,
        flow: FlowVariant,
        role: Role,
        device_id: Option<String>,
    ) -> WindowsResult<()> {
        Ok(())
    }

    /// Called when a new device is added.
    fn on_device_added(&mut self, device_id: String) -> WindowsResult<()> {
        Ok(())
    }

    /// Called when a device is removed.
    fn on_device_removed(&mut self, device_id: String) -> WindowsResult<()> {
        Ok(())
    }

    /// Called when the state of a device changes.
    fn on_device_state_change(&mut self, device_id: String, new_state: State) -> WindowsResult<()> {
        Ok(())
    }
}

pub(super) struct InnerAudioDeviceEventsProxy<Subscriber>
where
    Subscriber: AudioDeviceEvents + 'static,
{
    subscriber: Subscriber,
}

#[implement(IMMNotificationClient)]
pub(super) struct AudioDeviceEventsProxy<Subscriber>(
    UnsafeCell<InnerAudioDeviceEventsProxy<Subscriber>>,
)
where
    Subscriber: AudioDeviceEvents + 'static;

// NOTE: Calls to the callbacks are put in a queue by Windows. They are **not** called
// concurrently. So no data race possible with `self`.
impl<Subscriber> IMMNotificationClient_Impl for AudioDeviceEventsProxy_Impl<Subscriber>
where
    Subscriber: AudioDeviceEvents + 'static,
{
    fn OnDefaultDeviceChanged(
        &self,
        flow: windows::Win32::Media::Audio::EDataFlow,
        role: windows::Win32::Media::Audio::ERole,
        device_id: &windows_core::PCWSTR,
    ) -> windows_core::Result<()> {
        // SAFETY: The `UnsafeCell` got initialized by rust.
        let mut this = unsafe { ptr::read(self.0.get()) };

        // There might be no device available, which means no default device, so this method can
        // give a null pointer as a device ID.
        let device_id = (!device_id.is_null())
            // SAFETY: This was given to us by the Windows API, it should be valid to read.
            .then(|| unsafe { device_id.to_string().expect("Device ID to be valid UTF-8.") });

        let result = this
            .subscriber
            .on_default_device_change(flow.into(), role.into(), device_id);

        // SAFETY: We write to the same memory location we previously read from.
        unsafe { ptr::write(self.0.get(), this) };

        result
    }

    fn OnDeviceAdded(&self, device_id: &windows_core::PCWSTR) -> windows_core::Result<()> {
        // SAFETY: The `UnsafeCell` got initialized by rust.
        let mut this = unsafe { ptr::read(self.0.get()) };

        if device_id.is_null() {
            return Err(E_INVALIDARG.into());
        }
        // SAFETY: This was given to us by the Windows API, it should be valid to read.
        let device_id = unsafe { device_id.to_string().expect("Device ID to be valid UTF-8.") };

        let result = this.subscriber.on_device_added(device_id);

        // SAFETY: We write to the same memory location we previously read from.
        unsafe { ptr::write(self.0.get(), this) };

        result
    }

    fn OnDeviceRemoved(&self, device_id: &windows_core::PCWSTR) -> windows_core::Result<()> {
        // SAFETY: The `UnsafeCell` got initialized by rust.
        let mut this = unsafe { ptr::read(self.0.get()) };

        if device_id.is_null() {
            return Err(E_INVALIDARG.into());
        }
        // SAFETY: This was given to us by the Windows API, it should be valid to read.
        let device_id = unsafe { device_id.to_string().expect("Device ID to be valid UTF-8.") };

        let result = this.subscriber.on_device_removed(device_id);

        // SAFETY: We write to the same memory location we previously read from.
        unsafe { ptr::write(self.0.get(), this) };

        result
    }

    fn OnDeviceStateChanged(
        &self,
        device_id: &windows_core::PCWSTR,
        new_state: windows::Win32::Media::Audio::DEVICE_STATE,
    ) -> windows_core::Result<()> {
        // SAFETY: The `UnsafeCell` got initialized by rust.
        let mut this = unsafe { ptr::read(self.0.get()) };

        if device_id.is_null() {
            return Err(E_INVALIDARG.into());
        }
        // SAFETY: This was given to us by the Windows API, it should be valid to read.
        let device_id = unsafe { device_id.to_string().expect("Device ID to be valid UTF-8.") };

        let result = this
            .subscriber
            .on_device_state_change(device_id, new_state.into());

        // SAFETY: We write to the same memory location we previously read from.
        unsafe { ptr::write(self.0.get(), this) };

        result
    }

    fn OnPropertyValueChanged(
        &self,
        _device_id: &windows_core::PCWSTR,
        _key: &windows::Win32::Foundation::PROPERTYKEY,
    ) -> windows_core::Result<()> {
        Ok(())
    }
}

impl<Subscriber> AudioDeviceEventsProxy<Subscriber>
where
    Subscriber: AudioDeviceEvents + 'static,
{
    pub(super) fn new(subscriber: Subscriber) -> Self {
        Self(UnsafeCell::new(InnerAudioDeviceEventsProxy { subscriber }))
    }
}
