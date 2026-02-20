// SPDX-FileCopyrightText: 2025 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::rc::Rc;
use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering;

use dioxus::core::throw_error;
use dioxus::desktop::muda::{Menu, MenuItem, PredefinedMenuItem};
use dioxus::desktop::trayicon::{
    Icon, MouseButton, MouseButtonState, TrayIcon as DioxusTrayIcon, TrayIconBuilder, TrayIconEvent,
};
use dioxus::desktop::{use_muda_event_handler, use_tray_icon_event_handler, window};
use dioxus::prelude::*;
use tokio::sync::watch;

use audio::interface::audio_controller::AudioControllerEvents;

use crate::device::Controller;
use crate::utils::toggle_mute;

const SHOW_SETTINGS_EVENT_ID: &str = "muteit-show-settings";

const MIC_ICON_RGBA: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/mic.bin"));
const MIC_OFF_ICON_RGBA: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/mic-off.bin"));
const MIC_NONE_ICON_RGBA: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/mic-none.bin"));

static DEVICE_COUNTER: AtomicUsize = AtomicUsize::new(usize::MAX);

/// Tray icon state.
#[derive(Clone, Copy)]
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

/// Message to update our tray icon.
struct Message {
    /// The device ID of the messenger.
    device_id: usize,
    /// The state of the device.
    state: State,
}

/// Messenger to send update to our tray.
struct Messenger {
    /// The device ID associated with the notifier.
    device_id: usize,
    /// The channel `Sender` to notify our tray.
    tx: watch::Sender<Message>,
}

impl AudioControllerEvents for Messenger {
    fn on_mute_changed(&mut self, new_mute: bool) -> audio::WindowsResult<()> {
        // `Sender` uses a `RwLock` internally, so using it here is _technically_ forbidden...
        // We live dangerously though.
        self.tx.send_if_modified(|message| {
            // Only send the message if we are the **last device** that was subscribed. It should be
            // next to impossible to get a `mute_changed` event **before** we get unsubscribed **AND
            // after** the init message of the new device, but we check anyway.
            if message.device_id != self.device_id {
                return false;
            }

            *message = Message {
                device_id: self.device_id,
                state: new_mute.into(),
            };

            true
        });

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

pub fn use_setup_tray(mut controller: Signal<Controller>) -> Result<()> {
    let tray = use_hook(|| TrayIcon::try_new().map(|tray| Rc::new(tray)))?;

    use_muda_event_handler(|event| {
        if event.id() == SHOW_SETTINGS_EVENT_ID {
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
            && let Ok(Some(ref device)) = *controller.peek()
        {
            if let Err(err) = toggle_mute(device) {
                throw_error(err);
            };
        }
    });

    let (tx, rx) = use_hook(|| {
        watch::channel(Message {
            device_id: 0,
            state: State::None,
        })
    });

    // Change the tray icon when we receive a message.
    use_future(move || {
        let tray = tray.clone();
        let mut rx = rx.clone();
        async move {
            loop {
                if rx.changed().await.is_err() {
                    break;
                }

                let Message { state, .. } = *rx.borrow_and_update();

                if let Err(err) = tray.set_icon(state) {
                    throw_error(err);
                }
            }
        }
    });

    // Called when `device` changes. We need to change the tray icon accordingly and subscribe to
    // event notifications.
    use_effect(move || {
        let device_id = DEVICE_COUNTER.fetch_add(1, Ordering::Relaxed);

        // TODO: replace by `try` block when stabilized.
        // https://doc.rust-lang.org/std/ops/trait.Try.html
        let mut inner = || match *controller.write() {
            Ok(Some(ref mut controller)) => {
                controller.subscribe_event_notifications(Messenger {
                    device_id,
                    tx: tx.clone(),
                })?;

                let muted = controller.get_mute()?;
                tx.send_if_modified(|state| {
                    // Only set the new icon if the `Messenger` we just got subscribed hasn't
                    // already done so. It should be next to impossible, but we check anyway.
                    if state.device_id == device_id {
                        return false;
                    }

                    *state = Message {
                        device_id,
                        state: muted.into(),
                    };

                    true
                });

                Ok(())
            }
            Ok(None) => {
                tx.send_replace(Message {
                    device_id,
                    state: State::None,
                });
                Ok(())
            }
            Err(ref err) => Err(err.clone()),
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
