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

use crate::config::ConfigError;

#[derive(Debug)]
pub(crate) enum RunnerError {
    ConfigurationError(ConfigError),
    FileSystemError(std::io::Error),
}

impl std::convert::From<ConfigError> for RunnerError {
    fn from(config_error: ConfigError) -> Self {
        RunnerError::ConfigurationError(config_error)
    }
}

impl std::convert::From<std::io::Error> for RunnerError {
    fn from(file_error: std::io::Error) -> Self {
        RunnerError::FileSystemError(file_error)
    }
}