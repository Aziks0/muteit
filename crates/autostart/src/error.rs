// SPDX-FileCopyrightText: 2025 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: MPL-2.0

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("getting executable path ({0})")]
    GettingExePath(std::io::Error),
    #[error("getting registry key ({0})")]
    GettingRegistryValue(windows_core::Error),
    #[error("setting registry key ({0})")]
    SettingRegistryValue(windows_core::Error),
}
