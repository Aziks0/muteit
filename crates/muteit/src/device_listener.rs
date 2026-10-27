// SPDX-FileCopyrightText: 2026 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;

use anyhow::{Context as ResultExt, Result, anyhow};
use iced::Subscription;

use audio::enumerator::{AudioDeviceEvents, DeviceEnumerator, Flow};
use mpsc::{ReceiverSync, Sender};

/// State of the device, aka "is it available?".
#[derive(Clone, Copy, Debug)]
pub enum State {
    /// Device is available.
    Available,
    /// Device is unavailable. Changing mute state might give an error.
    Unavailable,
}

impl From<audio::enumerator::State> for State {
    fn from(value: audio::enumerator::State) -> Self {
        match value {
            audio::enumerator::State::Active => Self::Available,
            _ => Self::Unavailable,
        }
    }
}

/// Device state event.
#[derive(Debug)]
pub struct Event {
    /// The ID of the device that had its state changed.
    pub device_id: String,
    /// The new state of the device.
    pub state: State,
}

/// Messenger to send device state changes to [`DeviceListener`].
struct Messenger {
    /// The channel `Sender` to notify the runtime.
    tx: Sender<Event>,
}

impl AudioDeviceEvents for Messenger {
    fn on_device_state_change(
        &self,
        device_id: String,
        new_state: audio::enumerator::State,
    ) -> audio::WindowsResult<()> {
        if self
            .tx
            .try_send(Event {
                device_id,
                state: new_state.into(),
            })
            .is_err()
        {
            crate::error::set(anyhow!("on_device_state_change channel is full."));
        };

        Ok(())
    }
}

/// Listen to audio device state changes.
pub struct DeviceListener {
    rx: Arc<ReceiverSync<Event>>,
}

impl DeviceListener {
    pub fn try_new<F: Flow>(enumerator: &mut DeviceEnumerator<F>) -> Result<Self> {
        // The capacity of the channel is rather arbitrary. It just needs to be enough to avoid
        // getting full. For the channel to get full, the user would need to connect/disconnect
        // audio devices EXTREMELY rapidly—I consider it a user error.
        let (tx, rx) = mpsc::channel(32);

        enumerator
            .subscribe_event_notifications(Messenger { tx })
            .context("Subscribing to audio device's state changes events.")?;

        Ok(Self {
            rx: Arc::new(rx.into()),
        })
    }

    /// Subscribe to audio device state changes events.
    pub fn subscription(&self) -> Subscription<Event> {
        use std::hash::{Hash, Hasher};
        use std::pin::{Pin, pin};
        use std::task::{Context, Poll};

        use iced::futures::Stream;

        #[derive(Clone)]
        struct Receiver(Arc<ReceiverSync<Event>>);

        impl Hash for Receiver {
            fn hash<H: Hasher>(&self, state: &mut H) {
                std::any::TypeId::of::<Self>().hash(state);
                "device_listener".as_bytes().hash(state);
            }
        }

        impl Stream for Receiver {
            type Item = Event;

            fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
                pin!(self.0.receive()).poll(cx).map(Some)
            }
        }

        let rx = Receiver(self.rx.clone());

        Subscription::run_with(rx, |rx| rx.clone())
    }
}
