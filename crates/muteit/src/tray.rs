// SPDX-FileCopyrightText: 2025-2026 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::Arc;

use anyhow::{Context, Result, anyhow};
use iced::Subscription;
use tray_icon::menu::{Menu, MenuEvent, MenuItem};
use tray_icon::{Icon, MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};

use mpsc::ReceiverSync;

const SHOW_SETTINGS_EVENT_ID: &str = "muteit-show-settings";
const EXIT_EVENT_ID: &str = "muteit-exit";

const MIC_ICON_RGBA: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/mic.bin"));
const MIC_OFF_ICON_RGBA: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/mic-off.bin"));
const MIC_NONE_ICON_RGBA: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/mic-none.bin"));

/// Error message used when the channel is full.
const CHANNEL_FULL_ERROR: &str = "tray_icon channel is full.";

/// Tray icon/audio device state.
#[derive(Clone, Copy, Debug)]
pub enum State {
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

/// Tray icon action.
#[derive(Debug)]
pub enum Action {
    /// Toggle the mute state.
    ToggleMute,
    /// Show the settings window.
    ShowSettings,
    /// Exit the application.
    Exit,
}

/// Message for the tray icon.
///
/// See [`TrayIcon::update`].
pub enum Message {
    /// New device state.
    NewState(State),
}

/// Use a tray icon to display audio device's state.
pub struct TrayIcon {
    tray: tray_icon::TrayIcon,
    mic_icon: Icon,
    mic_off_icon: Icon,
    mic_none_icon: Icon,
    rx: Arc<ReceiverSync<Action>>,
}

impl TrayIcon {
    pub fn try_new() -> Result<Self> {
        let (tx, rx) = mpsc::channel(100);

        let tx_tray = tx.clone();
        TrayIconEvent::set_event_handler(Some(move |event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Down,
                ..
            }
            | TrayIconEvent::DoubleClick {
                button: MouseButton::Left,
                ..
            } = event
            {
                #[allow(
                    clippy::collapsible_if,
                    reason = "Sending the action is the intent, not setting the error."
                )]
                if tx_tray.try_send(Action::ToggleMute).is_err() {
                    crate::error::set(anyhow!(CHANNEL_FULL_ERROR));
                }
            }
        }));

        MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
            let action = match event.id().as_ref() {
                SHOW_SETTINGS_EVENT_ID => Action::ShowSettings,
                EXIT_EVENT_ID => Action::Exit,
                _ => return,
            };

            if tx.try_send(action).is_err() {
                crate::error::set(anyhow!(CHANNEL_FULL_ERROR));
            }
        }));

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
        menu.append(&MenuItem::with_id(EXIT_EVENT_ID, "Exit", true, None))?;

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
            rx: Arc::new(rx.into()),
        })
    }

    fn set_icon(&self, state: State) -> Result<()> {
        let (icon, tooltip) = match state {
            State::None => (&self.mic_none_icon, "Device unavailable"),
            State::Mute => (&self.mic_off_icon, "Left click to unmute"),
            State::Unmute => (&self.mic_icon, "Left click to mute"),
        };

        self.tray
            .set_icon(Some(icon.clone()))
            .context("Setting tray icon.")?;
        self.tray
            .set_tooltip(Some(tooltip))
            .context("Setting tray tooltip.")?;

        Ok(())
    }

    /// Update the tray icon.
    pub fn update(&mut self, message: Message) -> Result<()> {
        let Message::NewState(state) = message;

        self.set_icon(state)
    }

    /// Subscribe to tray icon events.
    pub fn subscription(&self) -> Subscription<Action> {
        use std::hash::{Hash, Hasher};
        use std::pin::{Pin, pin};
        use std::task::{Context, Poll};

        use iced::futures::Stream;

        #[derive(Clone)]
        struct Receiver(Arc<ReceiverSync<Action>>);

        impl Hash for Receiver {
            fn hash<H: Hasher>(&self, state: &mut H) {
                std::any::TypeId::of::<Self>().hash(state);
                "tray_icon".as_bytes().hash(state);
            }
        }

        impl Stream for Receiver {
            type Item = Action;

            fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
                pin!(self.0.receive()).poll(cx).map(Some)
            }
        }

        let rx = Receiver(self.rx.clone());

        Subscription::run_with(rx, |rx| rx.clone())
    }
}
