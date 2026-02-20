// SPDX-FileCopyrightText: 2025 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use dioxus::prelude::*;

use audio::enumerator::{DeviceEnumerator, Input};
use audio::interface::audio_controller::AudioController;

use crate::config::DeviceInformation;

pub fn use_setup_device(
    information: Signal<Option<DeviceInformation>>,
) -> Result<(
    Signal<DeviceEnumerator<Input>>,
    Signal<Option<AudioController>>,
)> {
    let enumerator = use_hook(|| {
        DeviceEnumerator::<Input>::try_new().map(|enumerator| Signal::new(enumerator))
    })?;

    let controller = use_hook(|| {
        information
            .peek()
            .as_ref()
            .map(|information| enumerator.peek().get_interface(information.id.clone()))
            .transpose()
            .map(|controller| Signal::new(controller))
    })?;

    Ok((enumerator, controller))
}
