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

use subprocess::PopenError;

use crate::config_error::ConfigError;

pub(crate) type Result<T> = std::result::Result<T, RunnerError>;

// Top level error for entire application
#[derive(Debug)]
pub(crate) enum RunnerError {
    MissingConfiguration,
    Configuration(ConfigError),
    FileSystem(std::io::Error),
    Exit(subprocess::ExitStatus),
    Process(PopenError),
    Channel(async_std::channel::TrySendError<crate::tui_state::TuiEvent>),
    CannotGetStderr,
    CannotGetStdout,
}

// configuration error conversion
impl std::convert::From<ConfigError> for RunnerError {
    fn from(config_error: ConfigError) -> Self {
        RunnerError::Configuration(config_error)
    }
}

// io error conversion
impl std::convert::From<std::io::Error> for RunnerError {
    fn from(file_error: std::io::Error) -> Self {
        RunnerError::FileSystem(file_error)
    }
}

// process error conversion
impl std::convert::From<PopenError> for RunnerError {
    fn from(process_error: PopenError) -> Self {
        RunnerError::Process(process_error)
    }
}

// channel error conversion
impl std::convert::From<async_std::channel::TrySendError<crate::tui_state::TuiEvent>>
    for RunnerError
{
    fn from(channel_error: async_std::channel::TrySendError<crate::tui_state::TuiEvent>) -> Self {
        RunnerError::Channel(channel_error)
    }
}
