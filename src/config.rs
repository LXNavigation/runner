/*
This file is part of the Everdream Runner (https://gitlab.com/everdream/runner).
Copyright (c) 2021 Everdream.

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, version 3.

This program is distributed in the hope that it will be useful, but
WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program. If not, see <http://www.gnu.org/licenses/>.
*/

use std::num::TryFromIntError;

use clap::crate_version;

use crate::command_config::CommandConfig;

// Configuration error. All possible errors that can happen during the parsing of configuration
#[derive(Debug)]
pub(crate) enum ConfigError {
    FileOpenError(std::io::Error),
    FileSerializationError(serde_json::error::Error),
    MissingApplicationName,
    WrongApplicationName(String),
    MissingVersion,
    WrongVersion(String),
    MissingCommandsArray,
    WrongCommandsFormat,
    BadCommandConfig(String, String),
    UnsupportedSystem(TryFromIntError),
}

impl std::convert::From<std::io::Error> for ConfigError {
    fn from(io_error: std::io::Error) -> Self {
        ConfigError::FileOpenError(io_error)
    }
}

impl std::convert::From<TryFromIntError> for ConfigError {
    fn from(from_int_error: TryFromIntError) -> Self {
        ConfigError::UnsupportedSystem(from_int_error)
    }
}

impl std::convert::From<serde_json::Error> for ConfigError {
    fn from(json_error: serde_json::Error) -> Self {
        ConfigError::FileSerializationError(json_error)
    }
}

// All config data parsed out
#[derive(Debug)]
pub(crate) struct Config {
    pub(crate) commands: Vec<CommandConfig>,
    pub(crate) crash_path: String,
}

impl Config {
    // creates parsed out configuration from a path to configuration file and reports on any errors
    pub(crate) fn create(path: String) -> Result<Config, ConfigError> {
        let file = std::fs::File::open(&path)?;
        let json = serde_json::from_reader(file)?;
        Config::verify_config(&json)?;
        Config::parse_config(&json)
    }

    // verifies that config fits application name and version
    fn verify_config(json: &serde_json::Value) -> Result<(), ConfigError> {
        let application = json
            .get("application")
            .ok_or(ConfigError::MissingApplicationName)?;
        if application != "runner" {
            return Err(ConfigError::WrongApplicationName(application.to_string()));
        }
        let version = json.get("version").ok_or(ConfigError::MissingVersion)?;
        if version != crate_version!() {
            return Err(ConfigError::WrongVersion(version.to_string()));
        }
        Ok(())
    }

    // parses config from json value
    fn parse_config(json: &serde_json::Value) -> Result<Config, ConfigError> {
        Ok(Config {
            commands: Config::parse_commands(json)?,
            crash_path: Config::parse_crash_path(json)?,
        })
    }

    // parses commands part of configuration file. Passes to CommandConfig
    fn parse_commands(json: &serde_json::Value) -> Result<Vec<CommandConfig>, ConfigError> {
        json.get("commands")
            .ok_or(ConfigError::MissingCommandsArray)?
            .as_array()
            .ok_or(ConfigError::WrongCommandsFormat)?
            .iter()
            .map(CommandConfig::parse_config)
            .into_iter()
            .collect::<Result<Vec<CommandConfig>, ConfigError>>()
    }

    // parses crash path specified in config file
    fn parse_crash_path(json: &serde_json::Value) -> Result<String, ConfigError> {
        json.get("crash path")
            .ok_or_else(|| {
                ConfigError::BadCommandConfig(String::from("crash path"), json.to_string())
            })?
            .as_str()
            .ok_or_else(|| {
                ConfigError::BadCommandConfig(String::from("crash path"), json.to_string())
            })
            .map(|path| path.to_owned())
    }
}
