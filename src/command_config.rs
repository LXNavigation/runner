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

use std::path::Path;

use crate::config::ConfigError;

// default number of lines to store for stdout history
const DEFAULT_HISTORY: u64 = 1000u64;

// default mode for application if none specified
const DEFAULT_MODE: CommandMode = CommandMode::RunUntilSuccess;

// enum indicating whether app should be restarted
#[derive(Debug, Clone)]
pub(crate) enum CommandMode {
    // run once, never repeat
    RunOnce,

    // run once, wait for finish
    RunOnceAndWait,

    // run until exits successfully
    RunUntilSuccess,

    // run until exits successfully, wait for finished
    RunUntilSuccessAndWait,

    // restart no matter what
    KeepAlive,
}

// single app configuration
//
// this struct holds all information needed to successfully run a process
#[derive(Debug, Clone)]
pub(crate) struct CommandConfig {
    // command to execute
    pub(crate) command: String,

    // arguments to pass to command
    pub(crate) args: Vec<String>,

    // number of lines to store for stdout
    pub(crate) stdout_history: usize,

    // mode to run application in
    pub(crate) mode: CommandMode,

    // name given to application
    pub(crate) name: String,
}

impl CommandConfig {
    // parses given app configuration, returning AppConfig on success, or error on failure
    pub(crate) fn parse_config(json: &serde_json::Value) -> Result<CommandConfig, ConfigError> {
        let command = CommandConfig::parse_command(json)?;
        Ok(CommandConfig {
            command: command.clone(),
            args: CommandConfig::parse_args(json)?,
            stdout_history: CommandConfig::parse_history(json)?,
            mode: CommandConfig::parse_mode(json)?,
            name: CommandConfig::parse_name(json)
                .unwrap_or_else(|| CommandConfig::get_name(&command)),
        })
    }

    // parses command part of configuration. This field must be present in configuration
    fn parse_command(json: &serde_json::Value) -> Result<String, ConfigError> {
        let command = match json.get("command") {
            Some(cmd) => cmd.as_str(),
            None => {
                return Err(ConfigError::BadCommandConfig(
                    String::from("command"),
                    json.to_string(),
                ))
            }
        };
        match command {
            Some(command) => Ok(String::from(command)),
            None => Err(ConfigError::BadCommandConfig(
                String::from("command"),
                json.to_string(),
            )),
        }
    }

    // parses command arguments. This field must be present but can be empty array
    fn parse_args(json: &serde_json::Value) -> Result<Vec<String>, ConfigError> {
        let args = match json.get("args") {
            Some(args) => args,
            None => return Ok(Vec::new()),
        };
        match args.as_array() {
            Some(args) => {
                let args: Option<Vec<String>> = args
                    .iter()
                    .map(|e| e.as_str().map(|e| e.to_owned()))
                    .collect();
                match args {
                    Some(args) => Ok(args),
                    None => Err(ConfigError::BadCommandConfig(
                        String::from("args"),
                        json.to_string(),
                    )),
                }
            }
            None => Err(ConfigError::BadCommandConfig(
                String::from("args"),
                json.to_string(),
            )),
        }
    }

    // parses history (number of lines for stdout)
    fn parse_history(json: &serde_json::Value) -> Result<usize, ConfigError> {
        let mut history = DEFAULT_HISTORY;
        if let Some(value) = json.get("stdout history") {
            history = match value.as_u64() {
                Some(val) => val,
                None => {
                    return Err(ConfigError::BadCommandConfig(
                        String::from("stdout history"),
                        json.to_string(),
                    ))
                }
            };
        };
        Ok(history
            .try_into()
            .expect("Could not convert u64 to usize on this system"))
    }

    // parses mode. Has 5 valid values, everything else should be reported as error
    fn parse_mode(json: &serde_json::Value) -> Result<CommandMode, ConfigError> {
        let mode = match json.get("mode") {
            Some(mode) => mode.as_str(),
            None => return Ok(DEFAULT_MODE),
        };
        let mode = match mode {
            Some(mode) => mode,
            None => {
                return Err(ConfigError::BadCommandConfig(
                    String::from("mode"),
                    json.to_string(),
                ))
            }
        };
        match mode {
            "run once" => Ok(CommandMode::RunOnce),
            "run once and wait" => Ok(CommandMode::RunOnceAndWait),
            "run until success" => Ok(CommandMode::RunUntilSuccess),
            "run until success and wait" => Ok(CommandMode::RunUntilSuccessAndWait),
            "keep alive" => Ok(CommandMode::KeepAlive),
            _ => Err(ConfigError::BadCommandConfig(
                String::from("mode"),
                json.to_string(),
            )),
        }
    }

    // parses name if given. Name will be calculated from command if it is not
    fn parse_name(json: &serde_json::Value) -> Option<String> {
        json.get("name")
            .and_then(|val| val.as_str())
            .map(|s| s.to_owned())
    }

    // get name from command. should extract file name from executable path
    fn get_name(command: &str) -> String {
        Path::new(&command)
            .file_stem()
            .unwrap_or_else(|| panic!("'{}' is not a valid command!", &command))
            .to_str()
            .unwrap()
            .to_owned()
    }
}

#[cfg(test)]
mod tests {

    use serde_json::json;

    use super::*;

    #[test]
    fn test_parse_config() {
        let json = json!({
            "command": "./updater/updater",
            "args": [ "-all" ],
            "mode": "run until success",
            "stdout history": 100
        });
        CommandConfig::parse_config(&json).unwrap();

        let json = json!({
            "command": "./updater/updater",
            "args": [ ],
            "mode": "run until success",
            "stdout history": 100
        });
        CommandConfig::parse_config(&json).unwrap();

        let json = json!({
            "command": "./updater/updater"
        });

        CommandConfig::parse_config(&json).unwrap();

        let json = json!({
            "args": [ "-all" ],
            "mode": "run until success",
            "stdout history": 100
        });
        CommandConfig::parse_config(&json).unwrap_err();
    }

    #[test]
    fn test_get_name() {
        assert_eq!(
            CommandConfig::get_name(&String::from("ls")),
            String::from("ls")
        );
        assert_eq!(
            CommandConfig::get_name(&String::from("test.exe")),
            String::from("test")
        );
        assert_eq!(
            CommandConfig::get_name(&String::from("path/test.exe")),
            String::from("test")
        );
        assert_eq!(
            CommandConfig::get_name(&String::from("path/test")),
            String::from("test")
        );
    }
}
