// SPDX-FileCopyrightText: 2025 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

// mod autostart;
// mod config;
// mod device;
// mod device_selector;
// mod hotkey_binding;
mod error;
mod mute_listener;
mod tray_icon;
mod utils;

use std::env::args;
use std::hint;
use std::sync::LazyLock;

use alone::is_alone;
use iced::widget::text;
use iced::window::settings::PlatformSpecific;
use iced::{Element, Size, Subscription, Task, window};

use crate::mute_listener::MuteListener;
use crate::tray_icon::{State, TrayIcon};

fn open_window() -> Task<Message> {
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

    let (_, task) = window::open(DEFAULT_WINDOW_SETTINGS.clone());

    task.map(Message::WindowOpened)
}

macro_rules! app_maybe_bail {
    ($result:expr) => {
        match $result {
            Ok(value) => value,
            Err(err) => return (Self::Error($crate::utils::leak_error(err)), open_window()),
        }
    };
}

#[inline(always)]
fn into_error_task(error: impl Into<anyhow::Error>) -> Task<Message> {
    Task::done(Message::Error(crate::utils::leak_error(error)))
}

#[derive(Debug)]
enum Message {
    WindowOpened(window::Id),
    WindowClosed,
    Tray(crate::tray_icon::Action),
    NewState(State),
    Error(&'static anyhow::Error),
}

struct InnerApp {
    mute_listener: MuteListener,
    tray: TrayIcon,
    settings_window: Option<window::Id>,
}

enum App {
    App(InnerApp),
    Error(&'static anyhow::Error),
}

impl App {
    fn boot() -> (Self, Task<Message>) {
        let should_open = args().rev().find(|arg| arg == "--hidden").is_none();

        let task = if should_open {
            open_window()
        } else {
            Task::none()
        };

        // let config_dir = AppConfig::get_directory().unwrap();

        let tray = app_maybe_bail!(TrayIcon::try_new());

        (
            Self::App(InnerApp {
                mute_listener: MuteListener::new(),
                tray,
                settings_window: None,
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
            Message::WindowOpened(id) => {
                this.settings_window = Some(id);

                Task::none()
            }
            Message::WindowClosed => {
                this.settings_window = None;

                Task::none()
            }
            Message::Tray(action) => match action {
                tray_icon::Action::None => Task::none(),
                tray_icon::Action::ToggleMute => {
                    todo!()
                }
                tray_icon::Action::ShowSettings => {
                    if let Some(id) = this.settings_window {
                        return Task::batch([window::minimize(id, false), window::gain_focus(id)]);
                    }

                    open_window()
                }
                tray_icon::Action::Exit => iced::exit(),
                tray_icon::Action::Error(err) => into_error_task(err),
            },
            Message::NewState(state) => {
                let action = this.tray.update(crate::tray_icon::Message::NewState(state));

                match action {
                    tray_icon::Action::None => Task::none(),
                    action => Task::done(Message::Tray(action)),
                }
            }
            Message::Error(err) => {
                let task = if let Some(id) = this.settings_window {
                    window::request_user_attention(id, Some(window::UserAttention::Critical))
                } else {
                    open_window()
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

        text!("todo").into()
    }
}

fn main() {
    if !is_alone!(env!("APP_ID")).expect("CreateMutexW to work") {
        return;
    }

    iced::daemon(App::boot, App::update, App::view)
        .title(App::title)
        .subscription(App::subscription)
        .run()
        .unwrap();
}
