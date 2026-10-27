// SPDX-FileCopyrightText: 2025-2026 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

// mod autostart;
// mod device;
mod app;
mod config;
mod device_listener;
mod error;
mod hotkey_manager;
mod mute_listener;
mod tray;
mod utils;
mod windows;

mod views;
use views::*;

use std::env::args;
use std::hint;

use iced::theme::Style;
use iced::widget::text;
use iced::{Element, Subscription, Task, Theme, window};

use alone::is_alone;

use crate::app::App;

use crate::windows::Win32::{COINIT_APARTMENTTHREADED, CoInitializeEx};

#[derive(Debug)]
enum Message {
    App(app::Message),
    Error(&'static anyhow::Error),
    WindowClosed,
}

#[allow(clippy::large_enum_variant)]
enum InfallibleApp {
    App(App),
    Error(&'static anyhow::Error),
}

impl InfallibleApp {
    fn boot() -> (Self, Task<Message>) {
        let (this, task) = match App::boot() {
            Ok((app, task)) => (Self::App(app), task.map(Message::App)),
            Err(err) => (Self::Error(utils::leak_error(err)), utils::open_window().1),
        };

        let should_open = args().rev().find(|arg| arg == "--hidden").is_none();

        let open_task = if should_open {
            Task::done(Message::App(app::Message::Tray(tray::Action::ShowSettings)))
        } else {
            Task::none()
        };

        (this, task.chain(open_task))
    }

    fn title(&self, _window_id: window::Id) -> String {
        String::from("muteit!")
    }

    fn style(&self, _theme: &Theme) -> Style {
        style()
    }

    fn update(&mut self, message: Message) -> Task<Message> {
        match (&mut *self, message) {
            (Self::App(app), Message::App(message)) => match app.update(message) {
                // NOTE: This is the 99.99% path.
                Ok(task) => task.map(Message::App),
                Err(err) => Task::done(Message::Error(utils::leak_error(err))),
            },
            (Self::App(app), Message::Error(err)) => {
                hint::cold_path();

                let task = if let Some(id) = app.window_id() {
                    window::request_user_attention(id, Some(window::UserAttention::Critical))
                } else {
                    utils::open_window().1
                };

                *self = Self::Error(err);

                task
            }
            (Self::Error(_), Message::WindowClosed) => {
                hint::cold_path();

                iced::exit()
            }
            _ => {
                hint::cold_path();

                Task::none()
            }
        }
    }

    fn subscription(&self) -> Subscription<Message> {
        match self {
            Self::App(app) => Subscription::batch([
                app.subscription().map(Message::App),
                error::subscription().map(Message::Error),
            ]),
            Self::Error(_) => {
                hint::cold_path();

                window::close_events().map(|_| Message::WindowClosed)
            }
        }
    }

    fn view(&self, window_id: window::Id) -> Element<'_, Message> {
        match self {
            Self::App(app) => app.view(window_id).map(Message::App),
            Self::Error(err) => {
                hint::cold_path();

                text!("An error occurred: {:?}", err).into()
            }
        }
    }
}

fn main() {
    if !is_alone!(env!("APP_ID")).expect("`CreateMutexW` should work.") {
        return;
    }

    // We need COM to be initialized in `App::boot`. `winit` does it, with
    // `COINIT_APARTMENTTHREADED`, but only on window creation, which can only happen _after_ the
    // `App:boot` call ends.
    unsafe {
        CoInitializeEx(None, COINIT_APARTMENTTHREADED as _)
            .ok()
            .expect("`CoInitializeEx` should work.");
    }

    // FIXME: `winit` might do it in a future version.
    // https://github.com/rust-windowing/winit/issues/4690
    dark::setup();

    iced::daemon(
        InfallibleApp::boot,
        InfallibleApp::update,
        InfallibleApp::view,
    )
    .title(InfallibleApp::title)
    .style(InfallibleApp::style)
    .subscription(InfallibleApp::subscription)
    .run()
    .unwrap();
}
