// SPDX-FileCopyrightText: 2025-2026 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::cell::RefCell;
use std::rc::Rc;

use anyhow::{Context, Result};
use iced::{Element, Subscription, Task, window};

use audio::enumerator::{DeviceEnumerator, Input};
use audio::interface::audio_controller::AudioController;
use hotkey::HotKeyEvent;

use crate::config::AppConfig;
use crate::device::{self, DeviceListener};
use crate::device_selector;
use crate::hotkey_manager::{self, HotKeyManager};
use crate::mute_listener::{self, MuteListener};
use crate::settings::{self, Settings};
use crate::tray::{self, TrayIcon};
use crate::utils;

#[derive(Debug)]
pub enum Message {
    WindowClosed,
    HotKeyEvent(HotKeyEvent),
    NewDeviceState(device::Event),
    Tray(tray::Action),
    NewState(tray::State),
    Settings(settings::Message),
}

pub struct App {
    config: AppConfig,
    enumerator: utils::DeviceEnumerator<Input>,
    controller: Option<AudioController>,
    device_listener: DeviceListener,
    mute_listener: MuteListener,
    hotkey_manager: HotKeyManager,
    tray: TrayIcon,
    settings: Option<(window::Id, Settings)>,
}

impl App {
    pub fn boot() -> Result<(Self, Task<Message>)> {
        let config = AppConfig::load()?;

        let mut hotkey_manager = HotKeyManager::try_new()?;

        if let Some(info) = &config.hotkey {
            hotkey_manager.update(hotkey_manager::Message::RegisterHotKey(info.hotkey))?;
        }

        let mut enumerator =
            DeviceEnumerator::<Input>::try_new().context("creating device enumerator")?;

        let device_listener = DeviceListener::try_new(&mut enumerator)?;

        let mut mute_listener = MuteListener::new();
        let mut tray = TrayIcon::try_new()?;

        let controller = if let Some(info) = &config.device
            && let (mut controller, audio::enumerator::State::Active) = enumerator
                .get_interface_and_state::<AudioController>(&info.id)
                .context("Getting new AudioController interface.")?
        {
            mute_listener.update(mute_listener::Message::NewController(&mut controller))?;

            Some(controller)
        } else {
            // `MuteListener`'s subscription doesn't give us a message with the new state when the
            // device is inactive, so we need to set the state ourselves.
            tray.update(tray::Message::NewState(tray::State::None))?;

            None
        };

        Ok((
            Self {
                config,
                enumerator: Rc::new(RefCell::new(enumerator)),
                controller,
                device_listener,
                mute_listener,
                hotkey_manager,
                tray,
                settings: None,
            },
            Task::none(),
        ))
    }

    pub fn window_id(&self) -> Option<window::Id> {
        self.settings.as_ref().map(|(id, _)| *id)
    }

    fn update_new_controller(&mut self, id: impl AsRef<str>) -> Result<()> {
        let controller = self
            .enumerator
            .borrow()
            .get_interface(id)
            .context("Getting new AudioController interface.")?;

        self.controller = Some(controller);

        self.mute_listener
            .update(mute_listener::Message::NewController(
                self.controller.as_mut().unwrap(),
            ))?;

        Ok(())
    }

    fn update_no_controller(&mut self) -> Result<()> {
        self.controller = None;

        self.mute_listener
            .update(mute_listener::Message::DeviceUnavailable)?;

        // `MuteListener`'s subscription doesn't give us a message with the new
        // state when the device is inactive, so we need to set the state
        // ourselves.
        self.tray
            .update(tray::Message::NewState(tray::State::None))?;

        Ok(())
    }

    pub fn update(&mut self, message: Message) -> Result<Task<Message>> {
        Ok(match message {
            Message::WindowClosed => {
                // Drop the `Settings` so we don't keep the memory only needed for the view.
                self.settings = None;

                self.config.save()?;

                Task::none()
            }
            Message::HotKeyEvent(event) => {
                let HotKeyEvent::Pressed { hotkey } = event;

                if let Some(info) = &self.config.hotkey
                    && info.hotkey == hotkey
                {
                    utils::toggle_mute(&self.controller)?;
                }

                Task::none()
            }
            Message::NewDeviceState(event) => {
                let device::Event { device_id, state } = event;

                // We need `&mut self` to call `update_new_controller`, which prevents borrowing
                // `self.config.device.id` at the same time. To satisfy the borrow checker, we
                // temporarily take ownership of `self.config.device` and restore it afterward. This
                // is very hacky, but it avoids cloning the string.
                //
                // Note that the `update_*_controller` methods do not depend on
                // `self.config.device`, so this is not an issue.
                let info = self.config.device.take();

                if let Some(info) = &info
                    && info.id == device_id
                {
                    match state {
                        device::State::Available => self.update_new_controller(&info.id)?,
                        device::State::Unavailable => self.update_no_controller()?,
                    };
                }

                self.config.device = info;

                Task::none()
            }
            Message::Tray(action) => match action {
                tray::Action::ToggleMute => {
                    utils::toggle_mute(&self.controller)?;

                    Task::none()
                }
                tray::Action::ShowSettings => {
                    if let Some((id, _)) = self.settings {
                        return Ok(Task::batch([
                            window::minimize(id, false),
                            window::gain_focus(id),
                        ]));
                    }

                    let (id, task) = utils::open_window();

                    self.settings = Some((
                        id,
                        Settings::try_new(self.enumerator.clone(), self.config.device.clone())?,
                    ));

                    task
                }
                tray::Action::Exit => iced::exit(),
            },
            Message::NewState(state) => {
                self.tray.update(tray::Message::NewState(state))?;

                Task::none()
            }
            Message::Settings(message) => {
                let Some((_, settings)) = &mut self.settings else {
                    return Ok(Task::none());
                };

                let action = settings.update(message)?;

                match action {
                    settings::Action::DeviceSelector(action) => match action {
                        device_selector::Action::NewDevice(info) => {
                            self.update_new_controller(&info.id)?;
                            self.config.device = Some(info);

                            Task::none()
                        }
                        device_selector::Action::NoDevice => {
                            self.update_no_controller()?;
                            self.config.device = None;

                            Task::none()
                        }
                        device_selector::Action::None => Task::none(),
                    },
                }
            }
        })
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            self.hotkey_manager.subscription().map(Message::HotKeyEvent),
            self.device_listener
                .subscription()
                .map(Message::NewDeviceState),
            self.tray.subscription().map(Message::Tray),
            self.mute_listener
                .subscription()
                .map(|msg| Message::NewState(msg.into())),
            window::close_events().map(|_| Message::WindowClosed),
        ])
    }

    pub fn view(&self, _window_id: window::Id) -> Element<'_, Message> {
        let Some((_, settings)) = &self.settings else {
            unreachable!("Call to `view` without `settings`.");
        };

        settings.view().map(Message::Settings)
    }
}
