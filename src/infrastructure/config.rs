//! Configuration file management.
//!
//! Handles loading and saving TOML configuration files.

use std::fs;
use std::path::Path;

use crate::domain::{AppConfig, AppError, Result};

/// Default configuration file content.
const DEFAULT_CONFIG: &str = r#"# Cursor Chat Handler Configuration
# Auto-generated - edit as needed

[sync]
# Interval between syncs in seconds (default: 120 = 2 minutes)
interval_secs = 120

# Whether sync is enabled
enabled = true

[storage]
# Maximum storage size in GB (default: 10)
max_size_gb = 10

# Number of days to keep backups (default: 30)
backup_retention_days = 30

# Whether to compress backups
compression = true

[paths]
# Custom data directory (optional, defaults to ~/.cursor-chat-handler)
# data_dir = "/custom/path"
"#;

/// Load configuration from file or create default.
///
/// # Errors
/// Returns error if file exists but cannot be read or parsed.
pub fn load_config() -> Result<AppConfig> {
    let config_path = AppConfig::default_data_dir().join("config.toml");

    if config_path.exists() {
        load_config_from_file(&config_path)
    } else {
        Ok(AppConfig::default())
    }
}

/// Load configuration from a specific file.
///
/// # Errors
/// Returns error if file cannot be read or parsed.
pub fn load_config_from_file(path: &Path) -> Result<AppConfig> {
    let content = fs::read_to_string(path)
        .map_err(|e| AppError::io(format!("Failed to read config file: {}", path.display()), e))?;

    toml::from_str(&content).map_err(|e| AppError::Config {
        message: format!("Failed to parse config file: {e}"),
    })
}

/// Save configuration to file.
///
/// # Errors
/// Returns error if file cannot be written.
pub fn save_config(config: &AppConfig) -> Result<()> {
    let config_path = config.config_file_path();

    // Ensure parent directory exists
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| AppError::io("Failed to create config directory", e))?;
    }

    let content = toml::to_string_pretty(config).map_err(|e| AppError::Config {
        message: format!("Failed to serialize config: {e}"),
    })?;

    fs::write(&config_path, content)
        .map_err(|e| AppError::io(format!("Failed to write config file: {}", config_path.display()), e))?;

    tracing::info!(path = %config_path.display(), "Configuration saved");

    Ok(())
}

/// Create default configuration file if it doesn't exist.
///
/// # Errors
/// Returns error if file cannot be created.
pub fn ensure_config_exists() -> Result<()> {
    let config_path = AppConfig::default_data_dir().join("config.toml");

    if !config_path.exists() {
        // Ensure parent directory exists
        if let Some(parent) = config_path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| AppError::io("Failed to create config directory", e))?;
        }

        fs::write(&config_path, DEFAULT_CONFIG)
            .map_err(|e| AppError::io("Failed to create default config", e))?;

        tracing::info!(path = %config_path.display(), "Created default configuration");
    }

    Ok(())
}

/// Get the path to the configuration file.
#[must_use]
pub fn config_file_path() -> std::path::PathBuf {
    AppConfig::default_data_dir().join("config.toml")
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_config_parses() {
        let config: AppConfig = toml::from_str(DEFAULT_CONFIG).unwrap();
        assert_eq!(config.sync.interval_secs, 120);
        assert_eq!(config.storage.max_size_gb, 10);
    }

    #[test]
    fn test_save_and_load() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("config.toml");

        let config = AppConfig::default();

        // Save
        let content = toml::to_string_pretty(&config).unwrap();
        fs::write(&config_path, content).unwrap();

        // Load
        let loaded = load_config_from_file(&config_path).unwrap();

        assert_eq!(loaded.sync.interval_secs, config.sync.interval_secs);
        assert_eq!(loaded.storage.max_size_gb, config.storage.max_size_gb);
    }
}

