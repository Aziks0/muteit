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
use std::sync::LazyLock;

use alone::is_alone;
use iced::widget::text;
use iced::window::settings::PlatformSpecific;
use iced::{Element, Size, Subscription, Task, window};

use crate::mute_listener::MuteListener;
use crate::tray_icon::{State, TrayIcon};

static DEFAULT_WINDOW_SETTINGS: LazyLock<window::Settings> = LazyLock::new(|| window::Settings {
    size: Size::new(800., 230.),
    platform_specific: PlatformSpecific {
        drag_and_drop: false,
        icon: Some(utils::get_small_icon()),
        taskbar_icon: Some(utils::get_big_icon()),
        ..Default::default()
    },
    ..Default::default()
});

#[derive(Debug)]
enum Message {
    WindowOpened(window::Id),
    WindowClosed,
    Tray(crate::tray_icon::Action),
    NewState(State),
    Error(&'static anyhow::Error),
}

struct App {
    mute_listener: MuteListener,
    tray: TrayIcon,
    settings_window: Option<window::Id>,
    error: Option<&'static anyhow::Error>,
}

impl App {
    fn boot() -> (Self, Task<Message>) {
        let should_open = args().rev().find(|arg| arg == "--hidden").is_none();

        let task = if should_open {
            window::open(DEFAULT_WINDOW_SETTINGS.clone())
                .1
                .map(Message::WindowOpened)
        } else {
            Task::none()
        };

        (
            Self {
                mute_listener: MuteListener::new(),
                tray: TrayIcon::try_new().unwrap(),
                settings_window: None,
                error: None,
            },
            task,
        )
    }

    fn title(&self, _window_id: window::Id) -> String {
        String::from("muteit!")
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::WindowOpened(id) => {
                self.settings_window = Some(id);

                Task::none()
            }
            Message::WindowClosed => {
                self.settings_window = None;

                Task::none()
            }
            Message::Tray(action) => match action {
                tray_icon::Action::ToggleMute => {
                    todo!()
                }
                tray_icon::Action::ShowSettings => {
                    if let Some(id) = self.settings_window {
                        return Task::batch([window::minimize(id, false), window::gain_focus(id)]);
                    }

                    let (_, task) = window::open(DEFAULT_WINDOW_SETTINGS.clone());

                    task.map(Message::WindowOpened)
                }
                tray_icon::Action::Exit => iced::exit(),
            },
            Message::NewState(state) => {
                self.tray.update(crate::tray_icon::Message::NewState(state));

                Task::none()
            }
            Message::Error(error) => {
                self.error = Some(error);

                Task::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        Subscription::batch([
            self.tray.subscription().map(Message::Tray),
            self.mute_listener
                .subscription()
                .map(|msg| Message::NewState(msg.into())),
            error::subscription().map(Message::Error),
            window::close_events().map(|_| Message::WindowClosed),
        ])
    }

    fn view(&self, _window_id: window::Id) -> Element<'_, Message> {
        if let Some(error) = self.error.as_ref() {
            return text!("An error occurred: {:?}", error).into();
        }

        text!("todo").into()
    }
}

fn main() {
    if !is_alone!(env!("APP_ID")).expect("CreateMutexW to work") {
        return;
    }

    // TODO: show an error message using `MessageBoxW`?
    // let config_dir = AppConfig::get_directory().unwrap();

    iced::daemon(App::boot, App::update, App::view)
        .title(App::title)
        .subscription(App::subscription)
        .run()
        .unwrap();
}
