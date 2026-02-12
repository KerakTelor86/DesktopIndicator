use config::{Config, ConfigError, File};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct Settings {
    default_icon_path: String,
    desktop_name_to_icon_path: HashMap<String, String>,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let settings = Config::builder()
            .add_source(File::with_name("%userprofile%/desktop-indicator.yaml"))
            .build()?;
        settings.try_deserialize()
    }
}
