// SPDX-FileCopyrightText: 2025 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::hash::Hash;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use anyhow::anyhow;
use iced::Subscription;
use iced::futures::SinkExt;

use audio::interface::audio_controller::{AudioController, AudioControllerEvents, AudioData};
use mpsc::{ReceiverSync, Sender};

/// Error message used when the channel is full.
const CHANNEL_FULL_ERROR: &str = "mute_listener channel is full.";

/// Wrapper around [`ReceiverSync<Event>`]. Used in [`Subscription`].
#[derive(Clone)]
struct Receiver(Arc<ReceiverSync<Event>>);

impl Hash for Receiver {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        std::any::TypeId::of::<Self>().hash(state);
        "mute_listener".as_bytes().hash(state);
    }
}

/// Device mute state event.
#[derive(Clone, Copy, Debug)]
struct Event {
    /// The device ID of the messenger.
    device_id: usize,
    /// The mute state of the device.
    muted: bool,
    /// Is this an initialization message?
    initial: bool,
}

/// Messenger to send update to the [`MuteListener`].
struct Messenger {
    /// The device ID associated with the messenger.
    device_id: usize,
    /// The last known mute state.
    last_muted: AtomicBool,
    /// The channel `Sender` to notify the runtime.
    tx: Sender<Event>,
}

impl AudioControllerEvents for Messenger {
    fn on_new_data(&self, data: AudioData) -> audio::WindowsResult<()> {
        // Check if the event is related to a mute state change.
        if data.muted == self.last_muted.swap(data.muted, Ordering::Relaxed) {
            return Ok(());
        }

        if self
            .tx
            .try_send(Event {
                device_id: self.device_id,
                muted: data.muted,
                initial: false,
            })
            .is_err()
        {
            crate::error::set(anyhow!(CHANNEL_FULL_ERROR));
        };

        Ok(())
    }
}

/// Message for [`MuteListener`].
///
/// See [`MuteListener::update`].
pub enum Message<'a> {
    NewController(&'a mut AudioController),
    DeviceInactive,
}

/// Listen to an audio device's mute events.
pub struct MuteListener {
    device_id: usize,
    tx: Sender<Event>,
    rx: Receiver,
}

impl MuteListener {
    pub fn new() -> Self {
        // The capacity of the channel is rather arbitrary. It just needs to be enough to avoid
        // getting full. For the channel to get full, the user would need to spam the mute button
        // EXTREMELY rapidly—I consider it a user error.
        let (tx, rx) = mpsc::channel(32);

        Self {
            device_id: 0,
            rx: Receiver(Arc::new(rx.into())),
            tx,
        }
    }

    /// Update the mute listener.
    pub fn update(&mut self, message: Message) {
        // Increment `device_id`.
        let device_id = self.device_id + 1;
        self.device_id = device_id;

        let Message::NewController(controller) = message else {
            // NOTE: We still need the `device_id` increment when the device becomes inactive.
            return;
        };

        let muted = controller.get_mute().unwrap();

        controller
            .subscribe_event_notifications(Messenger {
                device_id,
                last_muted: AtomicBool::new(muted),
                tx: self.tx.clone(),
            })
            .unwrap();

        self.tx
            .try_send(Event {
                device_id,
                muted,
                initial: true,
            })
            .map_err(|_| anyhow!(CHANNEL_FULL_ERROR))
            .unwrap();
    }

    /// Subscribe to mute events.
    pub fn subscription(&self) -> Subscription<bool> {
        let rx = self.rx.clone();

        Subscription::run_with(rx, |rx| {
            let rx = rx.clone();
            iced::stream::channel(1, async move |mut sender| {
                let mut last_device_id = 0;

                loop {
                    let Event {
                        device_id,
                        muted,
                        initial,
                    } = rx.0.receive().await;

                    // Avoid race between previous device messenger and new one.
                    if device_id < last_device_id {
                        continue;
                    }

                    // Avoid race between initial send and new device messenger.
                    if initial && device_id == last_device_id {
                        continue;
                    }

                    last_device_id = device_id;

                    // Channel cannot be disconnected, no error can happen.
                    let _ = sender.send(muted).await;
                }
            })
        })
    }
}
