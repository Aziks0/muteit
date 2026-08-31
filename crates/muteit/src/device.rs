// SPDX-FileCopyrightText: 2025 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::rc::Rc;

use dioxus::core::{anyhow, throw_error};
use dioxus::prelude::*;

use audio::enumerator::AudioDeviceEvents;
use audio::enumerator::{DeviceEnumerator, Input};
use audio::interface::audio_controller::AudioController;
use mpsc::Sender;

use crate::config::DeviceInformation;

#[derive(Clone, Copy)]
enum State {
    Active,
    Inactive,
}

impl From<audio::enumerator::State> for State {
    fn from(value: audio::enumerator::State) -> Self {
        match value {
            audio::enumerator::State::Active => Self::Active,
            _ => Self::Inactive,
        }
    }
}

/// The message sent by our [`Messenger`].
struct Message {
    /// The ID of the device that had its state changed.
    device_id: String,
    /// The new state of the device.
    state: State,
}

/// Messenger to send device state changes to update our controller.
struct Messenger {
    tx: Sender<Message>,
}

impl AudioDeviceEvents for Messenger {
    fn on_device_state_change(
        &self,
        device_id: String,
        new_state: audio::enumerator::State,
    ) -> audio::WindowsResult<()> {
        if self
            .tx
            .try_send(Message {
                device_id,
                state: new_state.into(),
            })
            .is_err()
        {
            crate::error::set(anyhow!("on_device_state_change channel is full"));
        };

        Ok(())
    }
}

fn get_controller(
    enumerator: &DeviceEnumerator<Input>,
    information: Option<&DeviceInformation>,
) -> Result<Option<AudioController>> {
    let controller = information
        .map(|information| enumerator.get_interface_and_state(&information.id))
        .transpose()
        .map(|controller_and_state| match controller_and_state {
            Some((controller, audio::enumerator::State::Active)) => Some(controller),
            _ => None,
        })?;
    Ok(controller)
}

#[allow(clippy::type_complexity)]
pub fn use_setup_device(
    information: Signal<Option<DeviceInformation>>,
) -> Result<(
    Signal<DeviceEnumerator<Input>>,
    Signal<Option<AudioController>>,
)> {
    let mut enumerator = use_hook(|| {
        DeviceEnumerator::<Input>::try_new().map(|enumerator| Signal::new(enumerator))
    })?;

    let mut controller = use_hook(|| {
        get_controller(&enumerator.peek(), information.peek().as_ref()).map(Signal::new)
    })?;

    let rx = use_hook(|| {
        // The capacity of the channel is rather arbitrary. It just needs to be enough to avoid
        // getting full.
        let (tx, rx) = mpsc::channel::<Message>(32);

        if let Err(err) = enumerator
            .write()
            .subscribe_event_notifications(Messenger { tx })
        {
            throw_error(err);
        }

        Rc::new(rx)
    });

    use_future(move || {
        let rx = rx.clone();
        async move {
            loop {
                let Message { state, device_id } = rx.receive().await;

                // Discard messages that are not for our device.
                if let Some(information) = information.peek().as_ref()
                    && information.id == device_id
                {
                    match state {
                        State::Inactive => controller.set(None),
                        State::Active => {
                            match get_controller(&enumerator.peek(), Some(information)) {
                                Ok(new_controller) => controller.set(new_controller),
                                Err(err) => throw_error(err),
                            }
                        }
                    }
                }
            }
        }
    });

    Ok((enumerator, controller))
}
