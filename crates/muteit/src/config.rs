// SPDX-FileCopyrightText: 2025 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::PathBuf;
use std::sync::OnceLock;

use dioxus::Result;
use dioxus::core::{Context, use_drop, use_hook};
use dioxus::desktop::HotKey;
use dioxus_signals::Signal;
use serde::{Deserialize, Serialize};

/// Information about the device to control.
#[derive(Serialize, Deserialize, Clone, Default, PartialEq)]
pub struct DeviceInformation {
    /// Name of the device.
    pub name: String,
    /// ID of the device.
    pub id: String,
}

/// Configuration for the app.
#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct AppConfig {
    /// Information about the device to control.
    #[serde(flatten)]
    pub information: Signal<Option<DeviceInformation>>,
    /// Shortcut for controlling the device.
    pub shortcut: Signal<Option<HotKey>>,
}

impl AppConfig {
    /// Create a new configuration with default values.
    fn new() -> Self {
        Self {
            information: Signal::new(None),
            shortcut: Signal::new(None),
        }
    }

    /// Get the default location of the configuration file.
    fn get_location() -> Result<PathBuf> {
        static LOCATION: OnceLock<PathBuf> = OnceLock::new();

        if let Some(path) = LOCATION.get() {
            return Ok(path.to_owned());
        }

        let appdata =
            std::env::var_os("APPDATA").context("APPDATA environment variable is not set.")?;
        let appdata = PathBuf::from(appdata);

        // Don't use `create_dir_all` so that we get an error if `%APPDATA%` points to a
        // non-existent directory.
        let config_dir = appdata.join("muteit!");
        if !config_dir.try_exists().with_context(|| {
            format!(
                "Could not verify that path `{}` exists.",
                config_dir.to_string_lossy()
            )
        })? {
            std::fs::create_dir(&config_dir).with_context(|| {
                format!(
                    "Could not create dir at `{}`.",
                    config_dir.to_string_lossy()
                )
            })?;
        }

        let config_path = config_dir.join("config.toml");
        Ok(LOCATION.get_or_init(|| config_path).to_owned())
    }

    /// Load the configuration from the default location.
    ///
    /// If the configuration file does not exist, it will be created with default values.
    fn load() -> Result<Self> {
        let config_path = Self::get_location()?;

        let config_content = match std::fs::read(&config_path) {
            Ok(content) => content,
            Err(err) => {
                if err.kind() == std::io::ErrorKind::NotFound {
                    return Ok(AppConfig::new());
                }
                return Err(err).with_context(|| {
                    format!(
                        "Could not read config file at `{}`.",
                        config_path.to_string_lossy()
                    )
                });
            }
        };

        let config = toml::from_slice(&config_content).context("Could not parse config file.")?;
        Ok(config)
    }

    /// Save the configuration to the default location.
    fn save(&self) -> Result<()> {
        let config_path = Self::get_location()?;
        let config_content = toml::to_string(self)?;
        std::fs::write(config_path, config_content)?;
        Ok(())
    }
}

/// Hook to get the config.
///
/// # Notes:
///
/// Only one component should call this hook, because they will each load and get a different
/// [`AppConfig`] when first calling it (i.e. modifying one will **not** update the others).
pub fn use_config() -> Result<AppConfig> {
    let config = use_hook::<Result<_>>(|| Ok(AppConfig::load()?))?;
    use_drop(move || {
        // This closure will be called in a `Drop::drop` so we should avoid a "double panic".
        let res = config.save();
        if !std::thread::panicking() {
            res.unwrap();
        }
    });
    Ok(config)
}
