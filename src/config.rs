use crate::guard_clause;
use config::{Config, ConfigError, File};
use dirs_next::home_dir;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub default_icon_path: String,
    pub desktop_name_to_icon_path: HashMap<String, String>,
    pub desktop_index_to_icon_path: HashMap<u32, String>,
}

#[derive(Debug)]
#[allow(unused)]
pub enum SettingsError {
    ConfigError(ConfigError),
    NoHomeDirError,
}

impl Settings {
    pub fn new() -> Result<Self, SettingsError> {
        let Some(home_dir) = home_dir() else {
            return Err(SettingsError::NoHomeDirError);
        };
        let settings = guard_clause!(
            Config::builder()
                .add_source(File::from(home_dir.join("desktop-indicator.yaml")))
                .build(),
            error,
            {
                return Err(SettingsError::ConfigError(error));
            }
        );
        match settings.try_deserialize() {
            Ok(result) => Ok(result),
            Err(error) => Err(SettingsError::ConfigError(error)),
        }
    }
}
