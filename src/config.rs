use crate::guard_clause;
use config::{Config, ConfigError, File};
use dirs_next::home_dir;
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Eq, PartialEq, Hash, Debug, Deserialize)]
pub struct HotKey {
    pub modifier_keys: Vec<String>,
    pub trigger_key: String,
    pub target_desktop_index: u32,
}

#[derive(Debug, Deserialize)]
pub struct Settings {
    pub default_icon_path: String,
    pub desktop_index_to_icon_path: HashMap<u32, String>,
    pub switch_desktop_hotkeys: Vec<HotKey>,
    pub move_window_hotkeys: Vec<HotKey>,
    pub follow_moved_windows: bool,
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
