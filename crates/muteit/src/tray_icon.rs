// SPDX-FileCopyrightText: 2025 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::cell::RefCell;
use std::time::Duration;

use dioxus::core::throw_error;
use dioxus::desktop::muda::{Menu, MenuItem, PredefinedMenuItem};
use dioxus::desktop::trayicon::{
    Icon, MouseButton, MouseButtonState, TrayIcon as DioxusTrayIcon, TrayIconBuilder, TrayIconEvent,
};
use dioxus::desktop::{use_muda_event_handler, use_tray_icon_event_handler, window};
use dioxus::prelude::*;

use crate::toggle_mute;

const SHOW_SETTINGS_EVENT_ID: &str = "muteit-show-settings";

const MIC_ICON_RGBA: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/mic.bin"));
const MIC_OFF_ICON_RGBA: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/mic-off.bin"));
const MIC_NONE_ICON_RGBA: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/mic-none.bin"));

pub struct TrayIcon {
    tray: DioxusTrayIcon,
    mic_icon: Icon,
    mic_off_icon: Icon,
    mic_none_icon: Icon,
    muted: RefCell<Option<bool>>,
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
            muted: RefCell::new(None),
        })
    }

    pub fn set_none(&self) -> Result<()> {
        self.tray.set_icon(Some(self.mic_none_icon.clone()))?;
        self.tray.set_tooltip(Some("Device unavailable"))?;
        self.muted.replace(None);
        Ok(())
    }

    pub fn set_mute(&self, muted: bool) -> Result<()> {
        let (icon, tooltip) = if muted {
            (self.mic_off_icon.clone(), "Left click to unmute")
        } else {
            (self.mic_icon.clone(), "Left click to mute")
        };
        self.tray.set_icon(Some(icon))?;
        self.tray.set_tooltip(Some(tooltip))?;
        self.muted.replace(Some(muted));
        Ok(())
    }
}

pub fn use_setup_tray(
    device: Signal<std::result::Result<Option<audio::Device>, audio::Error>>,
) -> Result<Signal<TrayIcon>> {
    let tray = use_hook(|| TrayIcon::try_new().map(|tray| Signal::new(tray)))?;

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
        } = event
            && let Ok(Some(ref device)) = *device.peek()
        {
            if let Err(err) = toggle_mute(device, &*tray.peek()) {
                throw_error(err);
            };
        }
    });

    use_future(move || async move {
        // TODO: replace by `try` block when stabilized.
        let mute_check = move |device: &audio::Device| {
            let tray = tray.peek();

            let muted = device.get_mute()?;
            // `tray.set_mute()` mutably borrows the ref, so we clone this one to drop the borrow.
            let was_muted = tray.muted.borrow().clone();

            if matches!(was_muted, Some(was_muted) if was_muted != muted) {
                tray.set_mute(muted)?;
            }

            dioxus::Ok(())
        };

        loop {
            if let Ok(Some(ref device)) = *device.peek() {
                if let Err(err) = mute_check(device) {
                    throw_error(err);
                }
            }

            tokio::time::sleep(Duration::from_secs(1)).await;
        }
    });

    Ok(tray)
}
