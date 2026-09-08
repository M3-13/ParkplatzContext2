use std::path::PathBuf;

use serde::{Deserialize, Serialize};

/// Application configuration, loaded from a local JSON file.
///
/// No environment variables are used. The config file lives at
/// `dirs::data_dir()/parkplatz/config.json`. When it is missing or unreadable,
/// the default configuration is used (workspace_root = the directory the app
/// was started from).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub workspace_root: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            workspace_root: std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
        }
    }
}

/// Returns the directory under which all Parkplatz data is stored.
pub fn data_dir() -> PathBuf {
    dirs::data_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("parkplatz")
}

/// Returns the path to the config file.
pub fn config_path() -> PathBuf {
    data_dir().join("config.json")
}

/// Loads the configuration from disk, falling back to defaults on any error.
pub fn load() -> Config {
    match std::fs::read_to_string(config_path()) {
        Ok(contents) => match serde_json::from_str::<Config>(&contents) {
            Ok(config) => config,
            Err(_) => Config::default(),
        },
        Err(_) => Config::default(),
    }
}
