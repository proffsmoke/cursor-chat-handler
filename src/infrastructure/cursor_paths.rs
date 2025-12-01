//! Cursor IDE path discovery.
//!
//! Handles locating Cursor's data directories across different platforms.

use std::path::PathBuf;

use crate::domain::{AppError, Result};

/// Known Cursor data directory locations by platform.
const CURSOR_CONFIG_PATHS: &[&str] = &[
    // Linux
    ".config/Cursor",
    // macOS
    "Library/Application Support/Cursor",
    // Alternative locations
    ".cursor",
];

/// Subdirectory containing state databases.
const GLOBAL_STORAGE_PATH: &str = "User/globalStorage";
const WORKSPACE_STORAGE_PATH: &str = "User/workspaceStorage";
const STATE_DB_NAME: &str = "state.vscdb";

/// Discovers the Cursor configuration directory.
///
/// # Errors
/// Returns error if home directory cannot be determined or Cursor is not installed.
pub fn find_cursor_config_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| AppError::Config {
        message: "Could not determine home directory".into(),
    })?;

    for path in CURSOR_CONFIG_PATHS {
        let full_path = home.join(path);
        if full_path.exists() && full_path.is_dir() {
            tracing::debug!("Found Cursor config at: {}", full_path.display());
            return Ok(full_path);
        }
    }

    Err(AppError::Config {
        message: format!("Cursor config directory not found. Searched: {CURSOR_CONFIG_PATHS:?}"),
    })
}

/// Finds all state.vscdb files in Cursor's data directories.
///
/// # Errors
/// Returns error if Cursor config directory cannot be found.
pub fn find_state_databases() -> Result<Vec<PathBuf>> {
    let config_dir = find_cursor_config_dir()?;
    let mut databases = Vec::new();

    // Global storage database
    let global_db = config_dir.join(GLOBAL_STORAGE_PATH).join(STATE_DB_NAME);
    if global_db.exists() {
        tracing::debug!("Found global state DB: {}", global_db.display());
        databases.push(global_db);
    }

    // Workspace storage databases
    let workspace_dir = config_dir.join(WORKSPACE_STORAGE_PATH);
    if workspace_dir.exists() {
        match std::fs::read_dir(&workspace_dir) {
            Ok(entries) => {
                for entry in entries.filter_map(std::result::Result::ok) {
                    let db_path = entry.path().join(STATE_DB_NAME);
                    if db_path.exists() {
                        tracing::debug!("Found workspace state DB: {}", db_path.display());
                        databases.push(db_path);
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Failed to read workspace directory: {}", e);
            }
        }
    }

    if databases.is_empty() {
        return Err(AppError::DatabaseNotFound {
            path: config_dir.join(GLOBAL_STORAGE_PATH).join(STATE_DB_NAME),
        });
    }

    Ok(databases)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_cursor_config_returns_result() {
        // This test just ensures the function doesn't panic
        let _ = find_cursor_config_dir();
    }
}
