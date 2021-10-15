use crate::app_config::AppConfig;
use clap::crate_version;

#[derive(Debug)]
pub(crate) enum ConfigError {
    FileOpenError(std::io::Error),
    FileSerializationError(serde_json::error::Error),
    MissingApplicationName,
    WrongApplicationName(String),
    MissingVersion,
    WrongVersion(String),
    MissingAppsArray,
    WrongAppsFormat,
    BadAppConfig(String, String),
}

// All config data parsed out
#[derive(Debug)]
pub(crate) struct Config {
    pub(crate) apps: Vec<AppConfig>,
    pub(crate) crash_path: String,
}

impl Config {
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

    fn parse_config(json: &serde_json::Value) -> Result<Config, ConfigError> {
        Ok(Config {
            apps: Config::parse_apps(json)?,
            crash_path: Config::parse_crash_path(json)?,
        })
    }

    fn parse_apps(json: &serde_json::Value) -> Result<Vec<AppConfig>, ConfigError> {
        let apps = match json.get("apps") {
            Some(apps) => apps,
            None => return Err(ConfigError::MissingAppsArray),
        };
        let apps = match apps.as_array() {
            Some(apps) => apps,
            None => return Err(ConfigError::WrongAppsFormat),
        };
        apps
            .iter()
            .map(AppConfig::parse_config)
            .into_iter()
            .collect::<Result<Vec<AppConfig>, ConfigError>>()
    }

    fn parse_crash_path(json: &serde_json::Value) -> Result<String, ConfigError> {
        match json.get("crash path") {
            Some(path) => match path.as_str() {
                Some(path) => Ok(path.to_owned()),
                None => Err(ConfigError::BadAppConfig(
                        "crash path".to_owned(),
                        json.to_string(),
                    ))
            },
            None => Err(ConfigError::BadAppConfig(
                    "crash path".to_owned(),
                    json.to_string(),
                ))
        }
    }
}
