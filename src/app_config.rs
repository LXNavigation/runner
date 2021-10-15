/* 
This file is part of the Everdream Runner (https://gitlab.com/everdream/runner).
Copyright (c) 2021 Kyoko.
 
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

const DEFAULT_HISTORY: u64 = 1000u64;
const DEFAULT_MODE: AppMode = AppMode::RunUntilSuccess;

// enum indicating whether app should be restarted
#[derive(Debug, Clone)]
pub(crate) enum AppMode {
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
#[derive(Debug, Clone)]
pub(crate) struct AppConfig {
    pub(crate) path: String,
    pub(crate) args: Vec<String>,
    pub(crate) hist: usize,
    pub(crate) mode: AppMode,
}

impl AppConfig {
    pub(crate) fn parse_config(json: &serde_json::Value) -> Result<AppConfig, ConfigError> {
        Ok(AppConfig {
            path: AppConfig::parse_path(json)?,
            args: AppConfig::parse_args(json)?,
            hist: AppConfig::parse_history(json)?,
            mode: AppConfig::parse_mode(json)?,
        })
    }

    fn parse_path(json: &serde_json::Value) -> Result<String, ConfigError> {
        let path = match json.get("path") {
            Some(cmd) => cmd.as_str(),
            None => {
                return Err(ConfigError::BadAppConfig(
                    "path".to_owned(),
                    json.to_string(),
                ))
            }
        };
        match path {
            Some(path) => Ok(path.to_owned()),
            None => Err(ConfigError::BadAppConfig(
                    "path".to_owned(),
                    json.to_string(),
                ))
        }
    }

    fn parse_args(json: &serde_json::Value) -> Result<Vec<String>, ConfigError> {
        let args = match json.get("args") {
            Some(args) => args,
            None => {
                return Err(ConfigError::BadAppConfig(
                    "args".to_owned(),
                    json.to_string(),
                ))
            }
        };
        match args.as_array() {
            Some(args) => {
                let args: Option<Vec<String>> = args
                    .iter()
                    .map(|e| e.as_str().map(|e| e.to_owned()))
                    .collect();
                match args {
                    Some(args) => Ok(args),
                    None => Err(ConfigError::BadAppConfig(
                        "args array".to_owned(),
                        json.to_string(),
                    )),
                }
            }
            None => Err(ConfigError::BadAppConfig(
                    "args array".to_owned(),
                    json.to_string(),
                ))
        }
    }

    fn parse_history(json: &serde_json::Value) -> Result<usize, ConfigError> {
        let mut history = DEFAULT_HISTORY;
        if let Some(value) = json.get("hist") {
            history = match value.as_u64() {
                Some(val) => val,
                None => {
                    return Err(ConfigError::BadAppConfig(
                        "hist".to_owned(),
                        json.to_string(),
                    ))
                }
            };
        };
        Ok(history
            .try_into()
            .expect("Could not convert u64 to usize on this system"))
    }

    fn parse_mode(json: &serde_json::Value) -> Result<AppMode, ConfigError> {
        let mode = match json.get("mode") {
            Some(mode) => mode.as_str(),
            None => return Ok(DEFAULT_MODE),
        };
        let mode = match mode {
            Some(mode) => mode,
            None => {
                return Err(ConfigError::BadAppConfig(
                    "mode".to_owned(),
                    json.to_string(),
                ))
            }
        };
        match mode {
            "run once" => Ok(AppMode::RunOnce),
            "run once and wait" => Ok(AppMode::RunOnceAndWait),
            "run until success" => Ok(AppMode::RunUntilSuccess),
            "run until success and wait" => Ok(AppMode::RunUntilSuccessAndWait),
            "keep alive" => Ok(AppMode::KeepAlive),
            _ => Err(ConfigError::BadAppConfig(
                "mode".to_owned(),
                json.to_string(),
            )),
        }
    }
}
