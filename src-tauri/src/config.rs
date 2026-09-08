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
    load_from(&config_path())
}

/// Loads the configuration from the given path, falling back to the default
/// configuration when the file is missing or unreadable.
fn load_from(path: &std::path::Path) -> Config {
    match std::fs::read_to_string(path) {
        Ok(contents) => serde_json::from_str::<Config>(&contents).unwrap_or_default(),
        Err(_) => Config::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn default_config_uses_start_directory() {
        let config = Config::default();
        assert_eq!(
            config.workspace_root,
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        );
    }

    #[test]
    fn data_dir_ends_with_parkplatz() {
        assert!(data_dir().ends_with("parkplatz"));
    }

    #[test]
    fn load_from_missing_file_returns_default() {
        let missing = std::env::temp_dir().join("parkplatz-nonexistent-config.json");
        let config = load_from(&missing);
        assert_eq!(
            config.workspace_root,
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        );
    }

    #[test]
    fn load_from_valid_file_parses_workspace_root() {
        let path = std::env::temp_dir().join("parkplatz-test-config.json");
        std::fs::write(&path, "{\"workspace_root\":\"/tmp/myproject\"}").unwrap();
        let config = load_from(&path);
        assert_eq!(config.workspace_root, PathBuf::from("/tmp/myproject"));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn load_from_invalid_file_returns_default() {
        let path = std::env::temp_dir().join("parkplatz-invalid-config.json");
        std::fs::write(&path, "{ not valid json ").unwrap();
        let config = load_from(&path);
        assert_eq!(
            config.workspace_root,
            std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
        );
        let _ = std::fs::remove_file(&path);
    }
}
