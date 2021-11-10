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
    ConfigurationError(ConfigError),
    FileSystemError(std::io::Error),
    ExitError(subprocess::ExitStatus),
    ProcessError(PopenError),
    ChannelError(async_std::channel::TrySendError<crate::tui_state::TuiEvent>),
    CannotGetStderr,
    CannotGetStdout,
}

impl std::fmt::Display for RunnerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            RunnerError::MissingConfiguration => write!(f, "Missing configuration file argument!"),
            RunnerError::ConfigurationError(err) => {
                write!(f, "Error parsing configuration: {}", err)
            }
            RunnerError::FileSystemError(err) => write!(f, "File system error: {}", err),
            RunnerError::ExitError(err) => write!(f, "Process exited with: {:#?}", err),
            RunnerError::ProcessError(err) => write!(f, "Error creating process: {}", err),
            RunnerError::ChannelError(err) => write!(f, "Unexpected channel error: {}", err),
            RunnerError::CannotGetStderr => write!(f, "Could not get Stderr for a process!"),
            RunnerError::CannotGetStdout => write!(f, "Could not get Stdout for a process!"),
        }
    }
}

// configuration error conversion
impl std::convert::From<ConfigError> for RunnerError {
    fn from(config_error: ConfigError) -> Self {
        RunnerError::ConfigurationError(config_error)
    }
}

// io error conversion
impl std::convert::From<std::io::Error> for RunnerError {
    fn from(file_error: std::io::Error) -> Self {
        RunnerError::FileSystemError(file_error)
    }
}

// process error conversion
impl std::convert::From<PopenError> for RunnerError {
    fn from(process_error: PopenError) -> Self {
        RunnerError::ProcessError(process_error)
    }
}

// channel error conversion
impl std::convert::From<async_std::channel::TrySendError<crate::tui_state::TuiEvent>>
    for RunnerError
{
    fn from(channel_error: async_std::channel::TrySendError<crate::tui_state::TuiEvent>) -> Self {
        RunnerError::ChannelError(channel_error)
    }
}
