// SPDX-FileCopyrightText: 2025-2026 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

// mod autostart;
// mod config;
// mod device;
// mod hotkey_binding;
mod device_selector;
mod error;
mod mute_listener;
mod settings;
mod tray;
mod utils;
mod windows;

use std::cell::RefCell;
use std::env::args;
use std::hint;
use std::rc::Rc;
use std::sync::LazyLock;

use anyhow::Context;
use iced::widget::text;
use iced::window::settings::PlatformSpecific;
use iced::{Element, Size, Subscription, Task, window};

use alone::is_alone;
use audio::enumerator::{DeviceEnumerator, Input};
use audio::interface::audio_controller::AudioController;

use crate::device_selector::DeviceInfo;
use crate::mute_listener::MuteListener;
use crate::settings::Settings;
use crate::tray::{State, TrayIcon};
use crate::utils::toggle_mute;
use crate::windows::Win32::{COINIT_APARTMENTTHREADED, CoInitializeEx};

fn open_window() -> (window::Id, Task<Message>) {
    // Static so we don't call `utils::get_[size]_icon` everytime.
    static DEFAULT_WINDOW_SETTINGS: LazyLock<window::Settings> =
        LazyLock::new(|| window::Settings {
            size: Size::new(800., 230.),
            platform_specific: PlatformSpecific {
                drag_and_drop: false,
                icon: Some(utils::get_small_icon()),
                taskbar_icon: Some(utils::get_big_icon()),
                ..Default::default()
            },
            ..Default::default()
        });

    let (id, task) = window::open(DEFAULT_WINDOW_SETTINGS.clone());

    (id, task.discard())
}

macro_rules! boot_maybe_bail {
    ($result:expr) => {
        match $result {
            Ok(value) => value,
            Err(err) => return (Self::Error($crate::utils::leak_error(err)), open_window().1),
        }
    };
}

macro_rules! app_maybe_bail {
    ($result:expr) => {
        match $result {
            Ok(value) => value,
            Err(err) => return Task::done(Message::Error($crate::utils::leak_error(err))),
        }
    };
}

#[inline(always)]
fn into_error_task(error: impl Into<anyhow::Error>) -> Task<Message> {
    Task::done(Message::Error(crate::utils::leak_error(error)))
}

#[derive(Debug)]
enum Message {
    WindowClosed,
    DeviceSelected(Option<DeviceInfo>),
    Tray(tray::Action),
    NewState(State),
    Settings(settings::Message),
    Error(&'static anyhow::Error),
}

struct InnerApp {
    enumerator: utils::DeviceEnumerator<Input>,
    controller: Option<AudioController>,
    mute_listener: MuteListener,
    tray: TrayIcon,
    settings: Option<(window::Id, Settings)>,
}

#[allow(clippy::large_enum_variant)]
enum App {
    App(InnerApp),
    Error(&'static anyhow::Error),
}

impl App {
    fn boot() -> (Self, Task<Message>) {
        let should_open = args().rev().find(|arg| arg == "--hidden").is_none();

        let task = if should_open {
            Task::done(Message::Tray(tray::Action::ShowSettings))
        } else {
            Task::none()
        };

        // let config_dir = AppConfig::get_directory().unwrap();

        let enumerator = boot_maybe_bail!(
            DeviceEnumerator::<Input>::try_new().context("creating device enumerator")
        );
        let tray = boot_maybe_bail!(TrayIcon::try_new());

        (
            Self::App(InnerApp {
                enumerator: Rc::new(RefCell::new(enumerator)),
                controller: None,
                mute_listener: MuteListener::new(),
                tray,
                settings: None,
            }),
            task,
        )
    }

    fn title(&self, _window_id: window::Id) -> String {
        String::from("muteit!")
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        let Self::App(this) = self else {
            hint::cold_path();

            return match message {
                Message::WindowClosed => iced::exit(),
                _ => Task::none(),
            };
        };

        match message {
            Message::WindowClosed => {
                // Drop the `Settings` so we don't keep the memory only needed for the view.
                this.settings = None;

                Task::none()
            }
            Message::DeviceSelected(info) => {
                let mut task = Task::none();

                let message = match info {
                    Some(info) => {
                        let controller = app_maybe_bail!(
                            this.enumerator
                                .borrow()
                                .get_interface(info.id)
                                .context("Getting new AudioController interface.")
                        );

                        this.controller = Some(controller);

                        mute_listener::Message::NewController(this.controller.as_mut().unwrap())
                    }
                    None => {
                        this.controller = None;

                        // `MuteListener`'s subscription doesn't give us a message with the new
                        // state when the device is inactive, so we need to set the state ourselves.
                        task = Task::done(Message::NewState(State::None));

                        mute_listener::Message::DeviceInactive
                    }
                };

                let action = this.mute_listener.update(message);

                match action {
                    mute_listener::Action::None => task,
                    mute_listener::Action::Error(err) => into_error_task(err),
                }
            }
            Message::Tray(action) => match action {
                tray::Action::None => Task::none(),
                tray::Action::ToggleMute => {
                    if let Some(controller) = &this.controller {
                        app_maybe_bail!(toggle_mute(controller).context("Toggling mute state."))
                    }

                    Task::none()
                }
                tray::Action::ShowSettings => {
                    if let Some((id, _)) = this.settings {
                        return Task::batch([window::minimize(id, false), window::gain_focus(id)]);
                    }

                    let (id, task) = open_window();

                    this.settings = Some((id, Settings::new(this.enumerator.clone())));

                    task
                }
                tray::Action::Exit => iced::exit(),
                tray::Action::Error(err) => into_error_task(err),
            },
            Message::NewState(state) => {
                let action = this.tray.update(tray::Message::NewState(state));

                match action {
                    tray::Action::None => Task::none(),
                    action => Task::done(Message::Tray(action)),
                }
            }
            Message::Settings(message) => {
                let Some((_, settings)) = &mut this.settings else {
                    return Task::none();
                };

                let action = settings.update(message);

                match action {
                    settings::Action::DeviceSelector(action) => match action {
                        device_selector::Action::NewDevice(info) => {
                            Task::done(Message::DeviceSelected(Some(info)))
                        }
                        device_selector::Action::NoDevice => {
                            Task::done(Message::DeviceSelected(None))
                        }
                        device_selector::Action::None => Task::none(),
                        device_selector::Action::Error(err) => into_error_task(err),
                    },
                }
            }
            Message::Error(err) => {
                let task = if let Some((id, _)) = this.settings {
                    window::request_user_attention(id, Some(window::UserAttention::Critical))
                } else {
                    open_window().1
                };

                *self = Self::Error(err);

                task
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        let Self::App(this) = self else {
            hint::cold_path();

            return window::close_events().map(|_| Message::WindowClosed);
        };

        Subscription::batch([
            this.tray.subscription().map(Message::Tray),
            this.mute_listener
                .subscription()
                .map(|msg| Message::NewState(msg.into())),
            error::subscription().map(Message::Error),
            window::close_events().map(|_| Message::WindowClosed),
        ])
    }

    fn view(&self, _window_id: window::Id) -> Element<'_, Message> {
        let this = match self {
            Self::App(app) => app,
            Self::Error(err) => {
                hint::cold_path();

                return text!("An error occurred: {:?}", err).into();
            }
        };

        let Some((_, settings)) = &this.settings else {
            unreachable!("Call to `view` without `settings`.");
        };

        settings.view().map(Message::Settings)
    }
}

fn main() {
    if !is_alone!(env!("APP_ID")).expect("CreateMutexW to work") {
        return;
    }

    // We need COM to be initialized in `App::boot`. `winit` does it, with
    // `COINIT_APARTMENTTHREADED`, but only on window creation, which can only happen _after_ the
    // `App:boot` call ends.
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED as _)
            .ok()
            .expect("CoInitializeEx to work");
    }

    // FIXME: `winit` might do it in a future version.
    // https://github.com/rust-windowing/winit/issues/4690
    dark::setup();

    iced::daemon(App::boot, App::update, App::view)
        .title(App::title)
        .subscription(App::subscription)
        .run()
        .unwrap();
}
