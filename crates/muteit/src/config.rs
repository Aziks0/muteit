// SPDX-FileCopyrightText: 2025-2026 Benjamin DIDIER <contact@aziks.aleeas.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::fs::File;
use std::hint;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::sync::OnceLock;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use smol_str::SmolStr;

use hotkey::HotKey;

/// Information about the device to control.
#[derive(Serialize, Deserialize, Clone, Default, Debug)]
pub struct DeviceInfo {
    /// Name of the device.
    pub name: String,
    /// ID of the device.
    pub id: String,
}

/// Information about the hotkey to control the selected device.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HotkeyInfo {
    /// The [`HotKey`] ready to be used.
    #[serde(flatten)]
    pub hotkey: HotKey,
    /// The hotkey key as a string (i.e. what's written on the keycap).
    pub key: SmolStr,
}

/// User configuration wrapper for the app.
#[derive(Serialize, Deserialize, Default)]
pub struct AppConfig {
    /// The config file.
    // NOTE: This is only an option to make `serde` happy when deserializing.
    #[serde(skip)]
    file: Option<File>,
    /// Information about the device to control.
    pub device: Option<DeviceInfo>,
    /// Hotkey for controlling the device.
    pub hotkey: Option<HotkeyInfo>,
}

impl AppConfig {
    /// Get the default configuration directory location.
    fn get_directory() -> Result<PathBuf> {
        let config_dir = {
            let mut config_dir = PathBuf::new();

            let appdata = std::env::var_os("LOCALAPPDATA")
                .context("`LOCALAPPDATA` environment variable is not set.")?;
            config_dir.push(appdata);

            let config_dir_name = if cfg!(feature = "dev") {
                "muteit!-dev"
            } else {
                "muteit!"
            };
            config_dir.push(config_dir_name);

            config_dir
        };

        // Don't use `create_dir_all` so that we get an error if `%LOCALAPPDATA%` points to a
        // non-existent directory.
        if !config_dir.try_exists().with_context(|| {
            format!(
                "Verifying that path `{}` exists.",
                config_dir.to_string_lossy()
            )
        })? {
            std::fs::create_dir(&config_dir).with_context(|| {
                format!(
                    "Creating config directory at `{}`.",
                    config_dir.to_string_lossy()
                )
            })?;
        };

        Ok(config_dir)
    }

    /// Get the default configuration file location.
    fn get_location() -> Result<&'static PathBuf> {
        static LOCATION: OnceLock<PathBuf> = OnceLock::new();

        if let Some(config_path) = LOCATION.get() {
            return Ok(config_path);
        }

        let mut config_path = Self::get_directory()?;
        config_path.push("config.toml");

        Ok(LOCATION.get_or_init(|| config_path))
    }

    /// Load the configuration from the default location.
    ///
    /// If the configuration file does not exist, it will be created with default values.
    pub fn load() -> Result<Self> {
        let config_path = Self::get_location()?;

        let mut file = File::options()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(config_path)
            .with_context(|| {
                format!(
                    "Opening config file at `{}`.",
                    config_path.to_string_lossy()
                )
            })?;

        let mut this = {
            let mut buf = String::new();
            let bytes = file.read_to_string(&mut buf).with_context(|| {
                format!(
                    "Reading config file at `{}`.",
                    config_path.to_string_lossy()
                )
            })?;

            // Skip deserializing if the file is empty.
            if bytes != 0 {
                toml::from_str(&buf).context("Parsing config file.")?
            } else {
                AppConfig::default()
            }
        };

        this.file = Some(file);

        Ok(this)
    }

    /// Save the configuration to the default location.
    pub fn save(&mut self) -> Result<()> {
        let config_content =
            toml::to_string(&self).expect("`AppConfig` should be serializable to `TOML`.");

        let file = self
            .file
            .as_mut()
            .expect("`AppConfig` `File` should never be `None`.");

        // TODO: replace by `try` block when stabilized.
        let mut inner = move || {
            use std::os::windows::fs::FileExt;

            // We can't write nothing, so shrink the file to 0 if the config is empty.
            if config_content.is_empty() {
                return file.set_len(0);
            }

            let buf = config_content.as_bytes();
            // Write to the start of the file.
            // NOTE: The seek and the write are combined into one unique syscall that does both.
            let written = file.seek_write(buf, 0)?;

            // The config is really tiny, so it should have been written in a single call. But just
            // in case, write the rest. The cursor has moved to the end of the previous write, no
            // need to move it manually again.
            if written != buf.len() {
                hint::cold_path();

                file.write_all(&buf[written..])?;
            }

            Ok(())
        };

        inner().with_context(|| {
            let config_path = Self::get_location()
                .expect("Config location should have been computed successfully.");
            format!(
                "Writing config file at `{}`.",
                config_path.to_string_lossy()
            )
        })
    }
}
