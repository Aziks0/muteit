// SPDX-FileCopyrightText: 2025 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::PathBuf;
use std::sync::OnceLock;

use dioxus::Result;
use dioxus::core::{AnyhowContext, use_drop, use_hook};
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

/// Information about the hotkey to control the selected device.
#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct HotkeyInformation {
    /// The [`HotKey`] ready to be used.
    pub hotkey: HotKey,
    /// The display character(s) of the key in the hotkey.
    pub key: String,
}

/// Configuration for the app.
#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct AppConfig {
    /// Information about the device to control.
    #[serde(flatten)]
    pub information: Signal<Option<DeviceInformation>>,
    /// Hotkey for controlling the device.
    #[serde(flatten)]
    pub hotkey: Signal<Option<HotkeyInformation>>,
}

impl AppConfig {
    /// Create a new configuration with default values.
    fn new() -> Self {
        Self {
            information: Signal::new(None),
            hotkey: Signal::new(None),
        }
    }

    /// Get the default configuration directory location.
    pub fn get_directory() -> Result<&'static PathBuf> {
        static DIRECTORY: OnceLock<PathBuf> = OnceLock::new();

        if let Some(path) = DIRECTORY.get() {
            return Ok(path);
        }

        let appdata = std::env::var_os("LOCALAPPDATA")
            .context("LOCALAPPDATA environment variable is not set.")?;
        let appdata = PathBuf::from(appdata);

        // Don't use `create_dir_all` so that we get an error if `%LOCALAPPDATA%` points to a
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
        };

        Ok(DIRECTORY.get_or_init(|| config_dir))
    }

    /// Get the default configuration file location.
    fn get_location() -> Result<&'static PathBuf> {
        static LOCATION: OnceLock<PathBuf> = OnceLock::new();

        let config_dir = Self::get_directory()?;
        let config_path = config_dir.join("config.toml");

        Ok(LOCATION.get_or_init(|| config_path))
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
                return Err(err)
                    .context(format!(
                        "Could not read config file at `{}`.",
                        config_path.to_string_lossy()
                    ))
                    .map_err(|err| err.into());
            }
        };

        let config = toml::from_slice(&config_content).context("Could not parse config file.")?;
        Ok(config)
    }

    /// Save the configuration to the default location.
    pub fn save(&self) -> Result<()> {
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
