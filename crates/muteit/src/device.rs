// SPDX-FileCopyrightText: 2025 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later
use std::sync::{Arc, RwLock};

use dioxus::core::throw_error;
use dioxus::prelude::*;
use tokio::sync::watch;

use audio::enumerator::AudioDeviceEvents;
use audio::enumerator::{DeviceEnumerator, Input};
use audio::interface::audio_controller::AudioController;

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

struct Message {
    state: State,
}

struct Messenger {
    /// The ID of the currently chosen device.
    device_id: Arc<RwLock<String>>,
    tx: watch::Sender<Message>,
}

impl AudioDeviceEvents for Messenger {
    fn on_device_state_change(
        &self,
        device_id: String,
        new_state: audio::enumerator::State,
    ) -> audio::WindowsResult<()> {
        if *self.device_id.read().expect("device id to not be poisoned") == device_id {
            let _ = self.tx.send(Message {
                state: new_state.into(),
            });
        }

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

    let (tx, rx) = use_hook(|| {
        // Dummy initial message. We don't care about it.
        let (tx, mut rx) = watch::channel(Message {
            state: State::Active,
        });
        rx.mark_unchanged();

        (tx, rx)
    });

    use_future(move || {
        let mut rx = rx.clone();
        async move {
            loop {
                if rx.changed().await.is_err() {
                    break;
                }

                let Message { state } = *rx.borrow_and_update();

                match state {
                    State::Inactive => controller.set(None),
                    State::Active => {
                        match get_controller(&enumerator.peek(), information.peek().as_ref()) {
                            Ok(new_controller) => controller.set(new_controller),
                            Err(err) => throw_error(err),
                        }
                    }
                }
            }
        }
    });

    // Setup the messenger.
    let device_id = use_hook(|| {
        let device_id = information
            .peek()
            .as_ref()
            .map(|information| information.id.clone())
            .unwrap_or_default();
        let device_id = Arc::new(RwLock::new(device_id));

        if let Err(err) = enumerator.write().subscribe_event_notifications(Messenger {
            device_id: device_id.clone(),
            tx,
        }) {
            throw_error(err);
        }

        device_id
    });

    // Change the messenger's device ID when a new device is selected.
    use_effect(move || {
        let new_device_id = information.read();
        let new_device_id = new_device_id
            .as_ref()
            .map(|information| information.id.as_str());

        // We use an empty device ID as a way to signal that none is chosen.
        let mut device_id = device_id.write().expect("`device_id` to not be poisoned");
        device_id.clear();
        if let Some(new_device_id) = new_device_id {
            device_id.push_str(new_device_id);
        }
    });

    Ok((enumerator, controller))
}
