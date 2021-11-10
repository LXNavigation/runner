use std::num::TryFromIntError;

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

// Configuration error. All possible errors that can happen during the parsing of configuration
#[derive(Debug)]
pub(crate) enum ConfigError {
    FileSystemError(std::io::Error),
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

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ConfigError::FileSystemError(err) => write!(f, "File system error - {}", err),
            ConfigError::FileSerializationError(err) => {
                write!(f, "File serilization error - {}", err)
            }
            ConfigError::MissingApplicationName => write!(f, "Missing application name!"),
            ConfigError::WrongApplicationName(name) => {
                write!(f, "Wrong application name, got {} instead!", name)
            }
            ConfigError::MissingVersion => write!(f, "Missing intended version!"),
            ConfigError::WrongVersion(version) => {
                write!(f, "Wrong application version, got {} instead!", version)
            }
            ConfigError::MissingCommandsArray => write!(f, "Missing commands array!"),
            ConfigError::WrongCommandsFormat => write!(f, "Wrong commands array format!"),
            ConfigError::BadCommandConfig(field, config) => write!(
                f,
                "Bad command config! Could not parse {} in {}",
                field, config
            ),
            ConfigError::UnsupportedSystem(_) => write!(f, "Unsuported operatign system!"),
        }
    }
}

// IO error conversion
impl std::convert::From<std::io::Error> for ConfigError {
    fn from(io_error: std::io::Error) -> Self {
        ConfigError::FileSystemError(io_error)
    }
}

// conversion error conversion
impl std::convert::From<TryFromIntError> for ConfigError {
    fn from(from_int_error: TryFromIntError) -> Self {
        ConfigError::UnsupportedSystem(from_int_error)
    }
}

// json error conversion
impl std::convert::From<serde_json::Error> for ConfigError {
    fn from(json_error: serde_json::Error) -> Self {
        ConfigError::FileSerializationError(json_error)
    }
}
