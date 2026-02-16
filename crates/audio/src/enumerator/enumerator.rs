// SPDX-FileCopyrightText: 2025 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: MPL-2.0

use std::fmt::Display;
use std::marker::PhantomData;
use std::ops::Deref;
use std::rc::Rc;

use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_FriendlyName;
use windows::Win32::Foundation::E_INVALIDARG;
use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
use windows::Win32::Media::Audio::{IMMDeviceEnumerator, MMDeviceEnumerator};
use windows::Win32::System::Com::{CLSCTX_ALL, CoCreateInstance, CoTaskMemFree, STGM_READ};
use windows::core::{HRESULT, HSTRING, PCWSTR};

use crate::enumerator::{Flow, State};
use crate::{Device, Error, Result};

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
/// You can convert it to a [`Device`] by using either [`try_from`](TryFrom::try_from) or
/// [`try_into`](TryInto::try_into):
///
/// ```no_run
/// # use audio::*;
/// # use audio::enumerator::*;
/// # use std::ops::Index;
/// let enumerator = DeviceEnumerator::<Input>::try_new()?;
///
/// let device = Device::try_from(&enumerator[0])?;
/// let device: Device = enumerator.index(0).try_into()?;
/// # Ok::<(), Error>(())
/// ```
pub struct DeviceInformation {
    /// The unique identifier of the device.
    id: PCWSTR,
    /// The name of the device.
    name: String,
    /// The state of the device.
    state: State,
    /// A reference to the enumerator.
    enumerator: Rc<IMMDeviceEnumerator>,
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

// I'm not very fond of having an implementation for a type not in this file, but I prefer this over
// making `get_device_by_id` and attributes of `DeviceInformation` `pub(crate)`.
impl TryFrom<&DeviceInformation> for Device {
    type Error = Error;

    fn try_from(value: &DeviceInformation) -> std::result::Result<Self, Self::Error> {
        get_device_by_id(&value.enumerator, value.id)
    }
}

impl Display for DeviceInformation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.name)
    }
}

impl Drop for DeviceInformation {
    fn drop(&mut self) {
        unsafe { CoTaskMemFree(Some(self.id.0 as *const _)) };
    }
}

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
/// And you can get a device:
///
/// ```no_run
/// # use audio::*;
/// # use audio::enumerator::*;
/// # use std::ops::Index;
/// // Using the enumerator:
/// let enumerator = DeviceEnumerator::<Input>::try_new()?;
/// let device = Device::try_from(&enumerator[0])?;
/// let device: Device = enumerator.index(0).try_into()?;
///
/// // Or, if you know the ID beforehand, you can directly get it:
/// let device_id = String::from("...");
/// let device = DeviceEnumerator::<Input>::get_device_by_id(&device_id)?;
/// # Ok::<(), Error>(())
/// ```
///
/// [devices' information]: DeviceInformation
pub struct DeviceEnumerator<F: Flow> {
    enumerator: Rc<IMMDeviceEnumerator>,
    endpoints: Vec<DeviceInformation>,
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
    /// # use std::ops::Index;
    /// let enumerator = DeviceEnumerator::<Input>::try_new()?;
    ///
    /// // You can enumerate the devices' information:
    /// for information in enumerator.iter() {
    ///     // Print there friendly name:
    ///     println!("{}", information);
    /// }
    ///
    /// // And get the device you want:
    /// let device: Device = enumerator.index(0).try_into()?;
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
            unsafe { CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)? };
        Ok(Self {
            enumerator: Rc::new(enumerator),
            endpoints: Vec::new(),
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
                let property_store = device.OpenPropertyStore(STGM_READ)?;
                let name = property_store
                    .GetValue(&PKEY_Device_FriendlyName)?
                    .to_string();

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

    /// Get a device using its ID.
    pub fn get_device(&self, id: impl AsRef<str>) -> Result<Device> {
        get_device_by_id(&self.enumerator, &HSTRING::from(id.as_ref()))
    }

    /// Get a device using its ID.
    ///
    /// # Notes:
    ///
    /// COM has to be initialized for the calling thread before calling this method.
    pub fn get_device_by_id(id: impl AsRef<str>) -> Result<Device> {
        Self::try_new_with_empty_list()?.get_device(id)
    }
}

impl<F: Flow> Deref for DeviceEnumerator<F> {
    type Target = Vec<DeviceInformation>;

    fn deref(&self) -> &Self::Target {
        &self.endpoints
    }
}

fn get_device_by_id<P>(enumerator: &IMMDeviceEnumerator, id: P) -> Result<Device>
where
    P: windows::core::Param<PCWSTR>,
{
    unsafe {
        let device = enumerator.GetDevice(id).map_err(|err| {
            // It is supposed to be `E_NOTFOUND`, but I couldn't find a reference to it in the
            // source code. The value is also used for `PEER_E_NOT_FOUND` so I prefer to only
            // check for this error for the `IMMDeviceEnumerator::GetDevice` method.
            if err.code() == HRESULT(0x80070490_u32 as i32) {
                Error::DeviceNotFound
            } else if err.code() == E_INVALIDARG {
                Error::InvalidDeviceID
            } else {
                Error::UnknownWindows(err)
            }
        })?;
        let interface: IAudioEndpointVolume = device.Activate(CLSCTX_ALL, None)?;
        Ok(Device::new(interface))
    }
}
