// SPDX-FileCopyrightText: 2025-2026 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fmt::Display;

use anyhow::{Context, Result};
use iced::Element;
use iced::widget::{button, pick_list, row};

use audio::enumerator::Flow;

use crate::config::DeviceInfo;
use crate::utils::DeviceEnumerator;

/// Device availability.
#[derive(Debug, PartialEq, Clone, Copy)]
enum Availability {
    /// Device is in the [`DeviceEnumerator`](audio::enumerator::DeviceEnumerator) list.
    Available,
    /// Device is not in the [`DeviceEnumerator`](audio::enumerator::DeviceEnumerator) list.
    Unavailable,
    /// Not a device.
    None,
}

/// A device option for [`pick_list`].
#[derive(Debug, Clone)]
struct DeviceOption {
    info: DeviceInfo,
    availability: Availability,
}

impl From<DeviceInfo> for DeviceOption {
    fn from(value: DeviceInfo) -> Self {
        Self {
            info: value,
            availability: Availability::Available,
        }
    }
}

impl PartialEq for DeviceOption {
    fn eq(&self, other: &Self) -> bool {
        // Only compare the ids, as this will only be used by `pick_list` and not by us.
        self.info.id == other.info.id
    }
}

impl Display for DeviceOption {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.availability {
            Availability::Available => f.write_str(self.info.name.as_str()),
            Availability::Unavailable => {
                f.write_fmt(format_args!("(Unavailable) {}", self.info.name))
            }
            Availability::None => f.write_str("None"),
        }
    }
}

/// Device selector action.
///
/// See [`DeviceSelector::update`].
pub enum Action {
    /// Handle the newly selected device.
    NewDevice(DeviceInfo),
    /// No device selected.
    NoDevice,
    /// Nothing to do.
    None,
}

/// Message for [`DeviceSelector`].
///
/// See [`DeviceSelector::update`].
#[derive(Debug, Clone)]
pub enum Message {
    /// A device has been selected from the list.
    #[allow(
        private_interfaces,
        reason = "This message is not meant to be constructed outside this module."
    )]
    DeviceSelected(DeviceOption),
    /// Refresh the list of devices.
    Refresh,
}

/// Widget to select an audio device.
pub struct DeviceSelector<F: Flow> {
    /// The device enumerator to create the list of options.
    enumerator: DeviceEnumerator<F>,
    /// The currently selected option.
    selected: DeviceOption,
    /// The list of options.
    options: Vec<DeviceOption>,
}

impl<F: Flow> DeviceSelector<F> {
    /// Create a new widget.
    pub fn try_new(enumerator: DeviceEnumerator<F>, selected: Option<DeviceInfo>) -> Result<Self> {
        let mut this = Self {
            enumerator,
            selected: selected
                .map(DeviceOption::from)
                .unwrap_or_else(|| DeviceOption {
                    info: Default::default(),
                    availability: Availability::None,
                }),
            options: Vec::new(),
        };

        this.update(Message::Refresh)?;

        Ok(this)
    }

    /// Update the widget.
    pub fn update(&mut self, message: Message) -> Result<Action> {
        Ok(match message {
            Message::DeviceSelected(option) => {
                // Don't send any action if the new selected device is the same as the previous one.
                if (self.selected.info.id == option.info.id
                    && self.selected.availability == option.availability)
                    // And don't select unavailable devices.
                    || option.availability == Availability::Unavailable
                {
                    return Ok(Action::None);
                }

                self.selected = option;

                match self.selected.availability {
                    Availability::None => Action::NoDevice,
                    Availability::Available => Action::NewDevice(self.selected.info.clone()),
                    Availability::Unavailable => {
                        unreachable!("We should never select unavailable devices.")
                    }
                }
            }
            Message::Refresh => {
                self.enumerator
                    .borrow_mut()
                    .refresh()
                    .context("Refreshing audio devices list.")?;

                // If there is no selected device, the selected one will be "None", so no need to
                // search for it.
                let mut found_selected = self.selected.availability == Availability::None;

                self.options.clear();
                // Reserve enough space for all the devices + the `None` option + the possible
                // `Unavailable` option.
                self.options
                    .reserve_exact(self.enumerator.borrow().len() + 2);

                self.options.push(DeviceOption {
                    info: DeviceInfo::default(),
                    availability: Availability::None,
                });

                for info in self.enumerator.borrow().iter() {
                    let id = info.get_id();

                    if self.selected.info.id == id {
                        found_selected = true;

                        // We need to update the availability of our selected option in case it's
                        // different, otherwise its name won't be changed.
                        self.selected.availability = Availability::Available;
                    }

                    self.options.push(DeviceOption {
                        // NOTE: Don't use `DeviceInfo::into` as we've already paid to get the id.
                        info: DeviceInfo {
                            name: info.to_string(),
                            id,
                        },
                        availability: Availability::Available,
                    });
                }

                // We want the selected device to be in the list, even if it is not available.
                if !found_selected {
                    // We need to update the availability of our selected option in case it's
                    // different, otherwise its name won't be changed.
                    self.selected.availability = Availability::Unavailable;

                    self.options.push(self.selected.clone());
                }

                Action::None
            }
        })
    }

    /// View the widget.
    pub fn view(&self) -> Element<'_, Message> {
        row![
            pick_list(
                self.options.as_slice(),
                Some(&self.selected),
                Message::DeviceSelected,
            ),
            button("Refresh").on_press(Message::Refresh)
        ]
        .into()
    }
}
