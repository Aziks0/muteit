// SPDX-FileCopyrightText: 2025-2026 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::cell::RefCell;
use std::rc::Rc;

use anyhow::{Context, Result};
use iced::{Element, Subscription, Task, window};

use audio::enumerator::{DeviceEnumerator, Input};
use audio::interface::audio_controller::AudioController;

use crate::device_selector::{self, DeviceInfo};
use crate::mute_listener::{self, MuteListener};
use crate::settings::{self, Settings};
use crate::tray::{self, State, TrayIcon};
use crate::utils::{self, toggle_mute};

#[derive(Debug)]
pub enum Message {
    WindowClosed,
    DeviceSelected(Option<DeviceInfo>),
    Tray(tray::Action),
    NewState(State),
    Settings(settings::Message),
}

pub struct App {
    enumerator: utils::DeviceEnumerator<Input>,
    controller: Option<AudioController>,
    mute_listener: MuteListener,
    tray: TrayIcon,
    settings: Option<(window::Id, Settings)>,
}

impl App {
    pub fn boot() -> Result<(Self, Task<Message>)> {
        let enumerator =
            DeviceEnumerator::<Input>::try_new().context("creating device enumerator")?;
        let tray = TrayIcon::try_new()?;

        Ok((
            Self {
                enumerator: Rc::new(RefCell::new(enumerator)),
                controller: None,
                mute_listener: MuteListener::new(),
                tray,
                settings: None,
            },
            Task::none(),
        ))
    }

    pub fn window_id(&self) -> Option<window::Id> {
        self.settings.as_ref().map(|(id, _)| *id)
    }

    pub fn update(&mut self, message: Message) -> Result<Task<Message>> {
        Ok(match message {
            Message::WindowClosed => {
                // Drop the `Settings` so we don't keep the memory only needed for the view.
                self.settings = None;

                Task::none()
            }
            Message::DeviceSelected(info) => {
                let mut task = Task::none();

                let message = match info {
                    Some(info) => {
                        let controller = self
                            .enumerator
                            .borrow()
                            .get_interface(info.id)
                            .context("Getting new AudioController interface.")?;

                        self.controller = Some(controller);

                        mute_listener::Message::NewController(self.controller.as_mut().unwrap())
                    }
                    None => {
                        self.controller = None;

                        // `MuteListener`'s subscription doesn't give us a message with the new
                        // state when the device is inactive, so we need to set the state ourselves.
                        task = Task::done(Message::NewState(State::None));

                        mute_listener::Message::DeviceInactive
                    }
                };

                self.mute_listener.update(message)?;

                task
            }
            Message::Tray(action) => match action {
                tray::Action::ToggleMute => {
                    if let Some(controller) = &self.controller {
                        toggle_mute(controller).context("Toggling mute state.")?;
                    }

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

                    self.settings = Some((id, Settings::try_new(self.enumerator.clone())?));

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
                            Task::done(Message::DeviceSelected(Some(info)))
                        }
                        device_selector::Action::NoDevice => {
                            Task::done(Message::DeviceSelected(None))
                        }
                        device_selector::Action::None => Task::none(),
                    },
                }
            }
        })
    }

    pub fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
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
