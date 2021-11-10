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
