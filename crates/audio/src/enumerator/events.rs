// SPDX-FileCopyrightText: 2026 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: MPL-2.0

use windows_core::implement;

use crate::windows::Win32::{E_INVALIDARG, IMMNotificationClient, IMMNotificationClient_Impl};

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
        &self,
        flow: FlowVariant,
        role: Role,
        device_id: Option<String>,
    ) -> WindowsResult<()> {
        Ok(())
    }

    /// Called when a new device is added.
    fn on_device_added(&self, device_id: String) -> WindowsResult<()> {
        Ok(())
    }

    /// Called when a device is removed.
    fn on_device_removed(&self, device_id: String) -> WindowsResult<()> {
        Ok(())
    }

    /// Called when the state of a device changes.
    fn on_device_state_change(&self, device_id: String, new_state: State) -> WindowsResult<()> {
        Ok(())
    }
}

#[implement(IMMNotificationClient)]
pub(super) struct AudioDeviceEventsProxy<Subscriber>(Subscriber)
where
    Subscriber: AudioDeviceEvents + 'static;

impl<Subscriber> IMMNotificationClient_Impl for AudioDeviceEventsProxy_Impl<Subscriber>
where
    Subscriber: AudioDeviceEvents + 'static,
{
    fn OnDefaultDeviceChanged(
        &self,
        flow: crate::windows::Win32::EDataFlow,
        role: crate::windows::Win32::ERole,
        device_id: &windows_core::PCWSTR,
    ) -> windows_core::Result<()> {
        // There might be no device available, which means no default device, so this method can
        // give a null pointer as a device ID.
        let device_id = (!device_id.is_null())
            // SAFETY: This was given to us by the Windows API, it should be valid to read.
            .then(|| unsafe {
                device_id
                    .to_string()
                    .expect("Device ID should be valid UTF-8.")
            });

        self.0
            .on_default_device_change(flow.into(), role.into(), device_id)
    }

    fn OnDeviceAdded(&self, device_id: &windows_core::PCWSTR) -> windows_core::Result<()> {
        if device_id.is_null() {
            return Err(E_INVALIDARG.into());
        }
        // SAFETY: This was given to us by the Windows API, it should be valid to read.
        let device_id = unsafe {
            device_id
                .to_string()
                .expect("Device ID should be valid UTF-8.")
        };

        self.0.on_device_added(device_id)
    }

    fn OnDeviceRemoved(&self, device_id: &windows_core::PCWSTR) -> windows_core::Result<()> {
        if device_id.is_null() {
            return Err(E_INVALIDARG.into());
        }
        // SAFETY: This was given to us by the Windows API, it should be valid to read.
        let device_id = unsafe {
            device_id
                .to_string()
                .expect("Device ID should be valid UTF-8.")
        };

        self.0.on_device_removed(device_id)
    }

    fn OnDeviceStateChanged(
        &self,
        device_id: &windows_core::PCWSTR,
        new_state: crate::windows::DeviceState,
    ) -> windows_core::Result<()> {
        if device_id.is_null() {
            return Err(E_INVALIDARG.into());
        }
        // SAFETY: This was given to us by the Windows API, it should be valid to read.
        let device_id = unsafe {
            device_id
                .to_string()
                .expect("Device ID should be valid UTF-8.")
        };

        self.0.on_device_state_change(device_id, new_state.into())
    }

    fn OnPropertyValueChanged(
        &self,
        _device_id: &windows_core::PCWSTR,
        _key: &crate::windows::Win32::PROPERTYKEY,
    ) -> windows_core::Result<()> {
        Ok(())
    }
}

impl<Subscriber> AudioDeviceEventsProxy<Subscriber>
where
    Subscriber: AudioDeviceEvents + 'static,
{
    pub(super) fn new(subscriber: Subscriber) -> Self {
        Self(subscriber)
    }
}
