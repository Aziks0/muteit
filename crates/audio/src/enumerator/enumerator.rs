// SPDX-FileCopyrightText: 2025-2026 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: MPL-2.0

use std::fmt::Display;
use std::marker::PhantomData;
use std::ops::Deref;
use std::rc::Rc;

use windows_core::{HSTRING, PCWSTR, WIN32_ERROR};

use crate::windows::Win32::{
    CLSCTX_ALL, CoCreateInstance, CoTaskMemFree, E_INVALIDARG, ERROR_NOT_FOUND, IMMDevice,
    IMMDeviceEnumerator, IMMNotificationClient, MMDeviceEnumerator, PKEY_Device_FriendlyName,
    PropVariantToBSTR, STGM_READ,
};

use crate::enumerator::{AudioDeviceEvents, AudioDeviceEventsProxy, Flow, State};
use crate::interface::AudioInterface;
use crate::{Error, Result};

/// Information about a device.
///
/// # Examples
///
/// Because it implements [`Display`], you can get its name easily.
///
/// ```no_run
/// # use audio::*;
/// # use audio::enumerator::*;
/// let enumerator = DeviceEnumerator::<Input>::try_new()?;
/// let information: &DeviceInformation = &enumerator[0];
///
/// // Print its name like you print any value:
/// println!("Your device is called {}", information);
///
/// // Modify a copy of its name:
/// let lower_case_name = information.to_string().to_lowercase();
///
/// // Get a copy of its id:
/// let id = information.get_id();
/// # Ok::<(), Error>(())
/// ```
///
/// You can convert it to an [`AudioInterface`] by using either [`try_from`](TryFrom::try_from) or
/// [`try_into`](TryInto::try_into):
///
/// ```no_run
/// # use audio::*;
/// # use audio::enumerator::*;
/// # use audio::interface::audio_controller::*;
/// # use std::ops::Index;
/// let enumerator = DeviceEnumerator::<Input>::try_new()?;
///
/// let controller = AudioController::try_from(&enumerator[0])?;
/// let controller: AudioController = enumerator.index(0).try_into()?;
/// # Ok::<(), Error>(())
/// ```
pub struct DeviceInformation {
    /// The unique identifier of the device.
    pub(crate) id: PCWSTR,
    /// The name of the device.
    name: String,
    /// The state of the device.
    state: State,
    /// A reference to the enumerator.
    pub(crate) enumerator: Rc<IMMDeviceEnumerator>,
}

impl DeviceInformation {
    /// Get the unique identifier of the device.
    pub fn get_id(&self) -> String {
        // PANICS: It should be impossible to get an ID from Windows with invalid UTF-16 characters.
        // SAFETY: Windows gave us the pointer when calling `IMMDevice::GetId`, which also allocates
        // the memory for us. We are responsible for freeing it, so the pointer should be valid
        // until we drop `self`.
        unsafe { self.id.to_string().unwrap() }
    }

    /// Get the state of the device.
    pub fn state(&self) -> State {
        self.state
    }
}

impl Display for DeviceInformation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)
    }
}

impl Drop for DeviceInformation {
    fn drop(&mut self) {
        unsafe { CoTaskMemFree(self.id.0 as *mut _) };
    }
}

#[derive(Clone)]
pub struct SubscriberId(IMMNotificationClient);

/// A list of audio devices.
///
/// This should be used to iterate over the available [devices' information], but also to get one
/// such device.
///
/// # Examples
///
/// You can iterate over the devices' information using `iter`:
///
/// ```no_run
/// # use audio::*;
/// # use audio::enumerator::*;
/// let enumerator = DeviceEnumerator::<Input>::try_new()?;
///
/// for information in enumerator.iter() {
///     println!("{}", information);
/// }
/// # Ok::<(), Error>(())
///
/// ```
///
/// If you want to refresh the list of devices:
///
/// ```no_run
/// # use audio::*;
/// # use audio::enumerator::*;
/// let mut enumerator = DeviceEnumerator::<Input>::try_new()?;
///
/// // Do some stuff...
///
/// enumerator.refresh();
/// # Ok::<(), Error>(())
/// ```
///
/// And you can get an interface:
///
/// ```no_run
/// # use audio::*;
/// # use audio::enumerator::*;
/// # use audio::interface::audio_controller::*;
/// # use std::ops::Index;
/// // Using the enumerator:
/// let enumerator = DeviceEnumerator::<Input>::try_new()?;
/// let controller = AudioController::try_from(&enumerator[0])?;
/// let controller: AudioController = enumerator.index(0).try_into()?;
///
/// // Or, if you know the ID beforehand, you can directly get it:
/// let device_id = String::from("...");
/// let controller: AudioController = DeviceEnumerator::<Input>::get_interface_by_id(&device_id)?;
/// # Ok::<(), Error>(())
/// ```
///
/// [devices' information]: DeviceInformation
pub struct DeviceEnumerator<F: Flow> {
    enumerator: Rc<IMMDeviceEnumerator>,
    endpoints: Vec<DeviceInformation>,
    subscribers: Vec<IMMNotificationClient>,
    _marker: PhantomData<F>,
}

impl<F: Flow> DeviceEnumerator<F> {
    /// Create a new list of audio devices.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use audio::*;
    /// # use audio::enumerator::*;
    /// # use audio::interface::audio_controller::*;
    /// # use std::ops::Index;
    /// let enumerator = DeviceEnumerator::<Input>::try_new()?;
    ///
    /// // You can enumerate the devices' information:
    /// for information in enumerator.iter() {
    ///     // Print there friendly name:
    ///     println!("{}", information);
    /// }
    ///
    /// // And get the interface you want:
    /// let controller: AudioController = enumerator.index(0).try_into()?;
    /// # Ok::<(), Error>(())
    /// ```
    ///
    /// See also [`DeviceInformation`].
    ///
    /// # Notes:
    ///
    /// COM has to be initialized for the calling thread before calling this method.
    pub fn try_new() -> Result<Self> {
        let mut result = Self::try_new_with_empty_list()?;
        result.refresh()?;
        Ok(result)
    }

    /// Create a new [`DeviceEnumerator`] without populating the list of devices.
    ///
    /// When calling this method, you'll have to call [`Self::refresh`] manually to get the list of
    /// available devices.
    ///
    /// See [`Self::try_new`] for more information.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use audio::*;
    /// # use audio::enumerator::*;
    /// let mut enumerator = DeviceEnumerator::<Input>::try_new_with_empty_list()?;
    /// assert_eq!(enumerator.len(), 0);
    ///
    /// enumerator.refresh_with_state(State::Unplugged)?;
    /// for information in enumerator.iter() {
    ///     println!("{}", information);
    /// }
    /// # Ok::<(), Error>(())
    /// ```
    ///
    /// # Notes:
    ///
    /// COM has to be initialized for the calling thread before calling this method.
    pub fn try_new_with_empty_list() -> Result<Self> {
        let enumerator: IMMDeviceEnumerator =
            unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL as _)? };
        Ok(Self {
            enumerator: Rc::new(enumerator),
            endpoints: Vec::new(),
            subscribers: Vec::new(),
            _marker: PhantomData,
        })
    }

    /// Refresh the list of audio devices.
    ///
    /// This will only fetch devices that have the specified [`State`].
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use audio::*;
    /// # use audio::enumerator::*;
    /// let mut enumerator = DeviceEnumerator::<Input>::try_new_with_empty_list()?;
    /// enumerator.refresh_with_state(State::Unplugged)?;
    /// # Ok::<_, Error>(())
    /// ```
    pub fn refresh_with_state(&mut self, state: State) -> Result<()> {
        unsafe {
            let endpoints = self
                .enumerator
                .EnumAudioEndpoints(F::DATA_FLOW, state.into())?;

            let count = endpoints.GetCount()?;
            self.endpoints.clear();
            self.endpoints.reserve_exact(count as usize);

            for device_number in 0..count {
                let device = endpoints.Item(device_number)?;

                // Get a readable name for the device.
                let property_store = device.OpenPropertyStore(STGM_READ as _)?;
                let name_property = property_store.GetValue(&PKEY_Device_FriendlyName)?;
                let name = PropVariantToBSTR(&name_property)?.display().to_string();

                let id = device.GetId()?;
                let id = PCWSTR(id.0);

                let state = device.GetState()?;

                self.endpoints.push(DeviceInformation {
                    id,
                    name,
                    state: state.into(),
                    enumerator: self.enumerator.clone(),
                });
            }
        }

        Ok(())
    }

    /// Refresh the list of audio devices.
    ///
    /// This will only fetch active devices.
    ///
    /// See also [`refresh_with_state`](DeviceEnumerator::refresh_with_state).
    pub fn refresh(&mut self) -> Result<()> {
        self.refresh_with_state(State::Active)
    }

    /// Subscribe to event notifications.
    ///
    /// When `Self` gets dropped, the subscriber will be automatically unsubscribed. The returned
    /// value can be used to [unsubscribe](Self::unsubscribe_event_notifications) before that.
    ///
    /// See [`AudioDeviceEvents`] for information on the implementation of `Subscriber`.
    pub fn subscribe_event_notifications<Subscriber>(
        &mut self,
        subscriber: Subscriber,
    ) -> Result<SubscriberId>
    where
        Subscriber: AudioDeviceEvents + 'static,
    {
        let proxy = AudioDeviceEventsProxy::new(subscriber);
        let callback: IMMNotificationClient = proxy.into();
        unsafe {
            self.enumerator
                .RegisterEndpointNotificationCallback(&callback)
                .ok()?
        }

        self.subscribers.push(callback.clone());

        Ok(SubscriberId(callback.clone()))
    }

    /// Unsubscribe from event notifications.
    pub fn unsubscribe_event_notifications(&mut self, id: SubscriberId) -> Result<()> {
        unsafe {
            self.enumerator
                .UnregisterEndpointNotificationCallback(&id.0)
                .ok()?
        }

        let index = self
            .subscribers
            .iter()
            .position(|s| *s == id.0)
            .expect("id should be in the list of subscribers.");
        self.subscribers.swap_remove(index);

        Ok(())
    }

    /// Get an interface for a device using its ID.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use audio::*;
    /// # use audio::enumerator::*;
    /// # use audio::interface::audio_controller::*;
    /// let enumerator = DeviceEnumerator::<Input>::try_new()?;
    /// let device = &enumerator[0];
    /// let controller: AudioController = enumerator.get_interface(device.get_id())?;
    /// # Ok::<(), Error>(())
    /// ```
    ///
    /// See also [`get_interface_and_state`](Self::get_interface_and_state).
    pub fn get_interface<I: AudioInterface>(&self, id: impl AsRef<str>) -> Result<I> {
        get_interface_by_id::<_, I>(&self.enumerator, &HSTRING::from(id.as_ref()))
    }

    /// Get an interface for a device, and its state, using its ID.
    pub fn get_interface_and_state<I: AudioInterface>(
        &self,
        id: impl AsRef<str>,
    ) -> Result<(I, State)> {
        unsafe {
            let device = get_device(&self.enumerator, &HSTRING::from(id.as_ref()))?;
            let state = device.GetState()?.into();
            let interface: I::Interface = device.Activate(CLSCTX_ALL as _, None)?;
            Ok((I::new(interface), state))
        }
    }

    /// Get an interface for a device using its ID.
    ///
    /// # Notes:
    ///
    /// COM has to be initialized for the calling thread before calling this method.
    ///
    /// See also [`get_interface_and_state_by_id`](Self::get_interface_and_state_by_id).
    pub fn get_interface_by_id<I: AudioInterface>(id: impl AsRef<str>) -> Result<I> {
        Self::try_new_with_empty_list()?.get_interface::<I>(id)
    }

    /// Get an interface for a device, and its state, using its ID.
    ///
    /// # Notes:
    ///
    /// COM has to be initialized for the calling thread before calling this method.
    pub fn get_interface_and_state_by_id<I: AudioInterface>(
        id: impl AsRef<str>,
    ) -> Result<(I, State)> {
        Self::try_new_with_empty_list()?.get_interface_and_state::<I>(id)
    }
}

impl<F: Flow> Deref for DeviceEnumerator<F> {
    type Target = Vec<DeviceInformation>;

    fn deref(&self) -> &Self::Target {
        &self.endpoints
    }
}

fn get_device<P>(enumerator: &IMMDeviceEnumerator, id: P) -> Result<IMMDevice>
where
    P: windows_core::Param<PCWSTR>,
{
    unsafe {
        enumerator.GetDevice(id).map_err(|err| {
            // The value is also used for `PEER_E_NOT_FOUND` so I prefer to only check for this
            // error for the `IMMDeviceEnumerator::GetDevice` method.
            if err.code() == WIN32_ERROR(ERROR_NOT_FOUND as _).into() {
                Error::DeviceNotFound
            } else if err.code() == E_INVALIDARG {
                Error::InvalidDeviceID
            } else {
                Error::UnknownWindows(err)
            }
        })
    }
}

pub(crate) fn get_interface_by_id<P, I>(enumerator: &IMMDeviceEnumerator, id: P) -> Result<I>
where
    P: windows_core::Param<PCWSTR>,
    I: AudioInterface,
{
    unsafe {
        let device = get_device(enumerator, id)?;
        let interface: I::Interface = device.Activate(CLSCTX_ALL as _, None)?;
        Ok(I::new(interface))
    }
}
