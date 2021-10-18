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
        let file = match std::fs::File::open(&path) {
            Ok(file) => file,
            Err(error) => return Err(ConfigError::FileOpenError(error)),
        };
        let json: serde_json::Value = match serde_json::from_reader(file) {
            Ok(json) => json,
            Err(error) => return Err(ConfigError::FileSerializationError(error)),
        };
        Config::verify_config(&json)?;
        Config::parse_config(&json)
    }

    // verifies that config fits application name and version
    fn verify_config(json: &serde_json::Value) -> Result<(), ConfigError> {
        let application = match json.get("application") {
            Some(app) => app,
            None => return Err(ConfigError::MissingApplicationName),
        };
        if application != "runner" {
            return Err(ConfigError::WrongApplicationName(application.to_string()));
        }
        let version = match json.get("version") {
            Some(ver) => ver,
            None => return Err(ConfigError::MissingVersion),
        };
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
        let commands = match json.get("commands") {
            Some(commands) => commands,
            None => return Err(ConfigError::MissingCommandsArray),
        };
        let commands = match commands.as_array() {
            Some(commands) => commands,
            None => return Err(ConfigError::WrongCommandsFormat),
        };
        commands.iter()
            .map(CommandConfig::parse_config)
            .into_iter()
            .collect::<Result<Vec<CommandConfig>, ConfigError>>()
    }

    // parses crash path specified in config file
    fn parse_crash_path(json: &serde_json::Value) -> Result<String, ConfigError> {
        match json.get("crash path") {
            Some(path) => match path.as_str() {
                Some(path) => Ok(path.to_owned()),
                None => Err(ConfigError::BadCommandConfig(
                    String::from("crash path"),
                    json.to_string(),
                )),
            },
            None => Err(ConfigError::BadCommandConfig(
                String::from("crash path"),
                json.to_string(),
            )),
        }
    }
}
