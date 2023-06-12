use std::sync::Arc;

use serenity::prelude::TypeMapKey;

/// Errors that can occur with a config file
#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    /// Returned when a config file could
    /// not be read from file
    #[error("Could not read config file")]
    ConfigReadError,

    /// Returned when a config file could
    /// not be parsed
    #[error("Could not parse config file")]
    ConfigParseError
}

/// A collection of configuration values
/// for Yukimi
#[derive(Default, serde::Serialize, serde::Deserialize)]
pub struct Config {
    /// Login token for Discord
    token: String,

    /// Command prefixes
    prefixes: Vec<String>,

    #[serde(default)]
    dev_guild_id: Option<u64>,

    secret_key: String,
}

pub struct ConfigData;

impl TypeMapKey for ConfigData {
    type Value = Arc<Config>;
}

impl Config {
    /// Attempts to read a file into a string, then parse it
    /// using toml.
    /// 
    /// Returns a `ConfigError` if either step fails.
    pub fn from_file(path: &str) -> Result<Self, ConfigError> {
        if let Ok(contents) = std::fs::read_to_string(path) {
            if let Ok(config) = toml::from_str::<Self>(contents.as_str()) {
                Ok(config)
            } else {
                Err(ConfigError::ConfigParseError)
            }
        } else {
            Err(ConfigError::ConfigReadError)
        }
    }

    /// Returns the value of `token`
    pub fn get_token(&self) -> &String { &self.token }

    /// Returns a list of default prefixes
    pub fn get_prefixes(&self) -> &Vec<String> { &self.prefixes }

    // Returns the Optional dev guild id
    pub fn get_dev_guild_id(&self) -> &Option<u64> {
        &self.dev_guild_id
    }

    pub fn get_secret_key(&self) -> &String { &self.secret_key }
}