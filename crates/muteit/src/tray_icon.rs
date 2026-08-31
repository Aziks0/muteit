// SPDX-FileCopyrightText: 2025 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::cell::Cell;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};

use dioxus::core::{anyhow, throw_error};
use dioxus::desktop::muda::{Menu, MenuItem, PredefinedMenuItem};
use dioxus::desktop::trayicon::{
    Icon, MouseButton, MouseButtonState, TrayIcon as DioxusTrayIcon, TrayIconBuilder, TrayIconEvent,
};
use dioxus::desktop::{use_muda_event_handler, use_tray_icon_event_handler, window};
use dioxus::prelude::*;

use audio::enumerator::{DeviceEnumerator, Input};
use audio::interface::audio_controller::{AudioController, AudioControllerEvents, AudioData};
use mpsc::Sender;

use crate::utils::toggle_mute;

const SHOW_SETTINGS_EVENT_ID: &str = "muteit-show-settings";

const MIC_ICON_RGBA: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/mic.bin"));
const MIC_OFF_ICON_RGBA: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/mic-off.bin"));
const MIC_NONE_ICON_RGBA: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/mic-none.bin"));

const CHANNEL_FULL_ERROR: &str = "tray_icon channel is full";

/// Tray icon state.
#[derive(Clone, Copy, Debug)]
enum State {
    /// Device is muted.
    Mute,
    /// Device is not muted.
    Unmute,
    /// No device selected.
    None,
}

impl From<bool> for State {
    fn from(value: bool) -> Self {
        match value {
            true => Self::Mute,
            false => Self::Unmute,
        }
    }
}

impl From<State> for bool {
    fn from(value: State) -> Self {
        match value {
            State::Mute => true,
            State::Unmute => false,
            State::None => unreachable!("Tried to convert `State::None` into bool."),
        }
    }
}

/// Message to update our tray icon.
#[derive(Clone, Copy, Debug)]
struct Message {
    /// The device ID of the messenger.
    device_id: usize,
    /// The state of the device.
    state: State,
    /// Is this an initialization message?
    is_initial: bool,
}

/// Messenger to send update to our tray.
struct Messenger {
    /// The device ID associated with the messenger.
    device_id: usize,
    /// The last known mute state.
    last_muted: AtomicBool,
    /// The channel `Sender` to notify our tray.
    tx: Sender<Message>,
}

impl AudioControllerEvents for Messenger {
    fn on_new_data(&self, data: AudioData) -> audio::WindowsResult<()> {
        // Check if the event is related to a mute state change.
        if data.muted == self.last_muted.swap(data.muted, Ordering::Relaxed) {
            return Ok(());
        }

        if self
            .tx
            .try_send(Message {
                device_id: self.device_id,
                state: data.muted.into(),
                is_initial: false,
            })
            .is_err()
        {
            crate::error::set(anyhow!(CHANNEL_FULL_ERROR));
        };

        Ok(())
    }
}

pub struct TrayIcon {
    tray: DioxusTrayIcon,
    mic_icon: Icon,
    mic_off_icon: Icon,
    mic_none_icon: Icon,
}

impl TrayIcon {
    fn try_new() -> Result<Self> {
        let mic_icon = Icon::from_rgba(MIC_ICON_RGBA.to_vec(), 32, 32)
            .expect("mic.bin should be a correct icon");
        let mic_off_icon = Icon::from_rgba(MIC_OFF_ICON_RGBA.to_vec(), 32, 32)
            .expect("mic-off.bin should be a correct icon");
        let mic_none_icon = Icon::from_rgba(MIC_NONE_ICON_RGBA.to_vec(), 32, 32)
            .expect("mic-none.bin should be a correct icon");

        let menu = Menu::new();
        menu.append(&MenuItem::with_id(
            SHOW_SETTINGS_EVENT_ID,
            "Settings",
            true,
            None,
        ))?;
        menu.append(&PredefinedMenuItem::quit(Some("Exit")))?;

        let tray = TrayIconBuilder::new()
            .with_menu_on_left_click(false)
            .with_menu(Box::new(menu))
            .with_icon(mic_none_icon.clone())
            .with_tooltip("Initializing...")
            .build()?;

        Ok(Self {
            tray,
            mic_icon,
            mic_off_icon,
            mic_none_icon,
        })
    }

    fn set_icon(&self, state: State) -> Result<()> {
        let (icon, tooltip) = match state {
            State::None => (&self.mic_none_icon, "Device unavailable"),
            State::Mute => (&self.mic_off_icon, "Left click to unmute"),
            State::Unmute => (&self.mic_icon, "Left click to mute"),
        };

        self.tray.set_icon(Some(icon.clone()))?;
        self.tray.set_tooltip(Some(tooltip))?;

        Ok(())
    }
}

pub fn use_setup_tray(
    mut enumerator: Signal<DeviceEnumerator<Input>>,
    mut controller: Signal<Option<AudioController>>,
) -> Result<()> {
    let tray = use_hook(|| TrayIcon::try_new().map(Rc::new))?;

    use_muda_event_handler(move |event| {
        if event.id() == SHOW_SETTINGS_EVENT_ID {
            // Refresh the list of devices before showing the window.
            if let Err(err) = enumerator.write().refresh() {
                throw_error(err);
                // Let the `ErrorBoundary` show the window if there is an error.
                return;
            }
            window().set_visible(true);
            window().set_focus();
        }
    });

    use_tray_icon_event_handler(move |event| {
        if let TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Down,
            ..
        }
        | TrayIconEvent::DoubleClick {
            button: MouseButton::Left,
            ..
        } = event
            && let Some(ref controller) = *controller.peek()
            && let Err(err) = toggle_mute(controller)
        {
            throw_error(err);
        };
    });

    let (tx, rx) = use_hook(|| {
        // The capacity of the channel is rather arbitrary. It just needs to be enough to avoid
        // getting full. For the channel to get full, the user would need to spam the mute button
        // EXTREMELY rapidly—I consider it a user error.
        let (tx, rx) = mpsc::channel(32);
        (tx, Rc::new(rx))
    });

    // Change the tray icon when we receive a message.
    use_future(move || {
        let tray = tray.clone();
        let rx = rx.clone();
        async move {
            let mut last_device_id = 0;
            loop {
                let Message {
                    device_id,
                    state,
                    is_initial,
                } = rx.receive().await;

                // Avoid race between previous device messenger and new one.
                if device_id < last_device_id {
                    continue;
                }

                // Avoid race between initial send and new device messenger.
                if is_initial && device_id == last_device_id {
                    continue;
                }

                last_device_id = device_id;

                if let Err(err) = tray.set_icon(state) {
                    throw_error(err);
                }
            }
        }
    });

    // It doesn't need to be reactive.
    let current_device_id = use_hook(|| Rc::new(Cell::new(0usize)));

    // Called when `device` changes. We need to change the tray icon accordingly and subscribe to
    // event notifications.
    use_effect(move || {
        let device_id = current_device_id.get() + 1;
        current_device_id.set(device_id);

        // TODO: replace by `try` block when stabilized.
        // https://doc.rust-lang.org/std/ops/trait.Try.html
        let mut inner = || -> Result<()> {
            match *controller.write() {
                Some(ref mut controller) => {
                    let muted = controller.get_mute()?;

                    controller.subscribe_event_notifications(Messenger {
                        device_id,
                        last_muted: AtomicBool::new(muted),
                        tx: tx.clone(),
                    })?;

                    tx.try_send(Message {
                        device_id,
                        state: muted.into(),
                        is_initial: true,
                    })
                    .map_err(|_| anyhow!(CHANNEL_FULL_ERROR))?;

                    Ok(())
                }
                None => {
                    tx.try_send(Message {
                        device_id,
                        state: State::None,
                        is_initial: true,
                    })
                    .map_err(|_| anyhow!(CHANNEL_FULL_ERROR))?;
                    Ok(())
                }
            }
        };

        if let Err(err) = inner() {
            throw_error(err);
        };

        // Subscribe to the `device` **after** the call to `write`. If we subscribe _before_, we
        // cause an infinite loop.
        let _ = controller.read();
    });

    Ok(())
}
