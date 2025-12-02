//! Sync-related domain models and configuration.
//!
//! Contains types for managing synchronization state, configuration,
//! and backup metadata for the auto-sync system.

use std::path::PathBuf;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Configuration for the sync daemon.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    /// Interval between sync operations in seconds.
    #[serde(default = "default_interval")]
    pub interval_secs: u64,

    /// Whether sync is enabled.
    #[serde(default = "default_enabled")]
    pub enabled: bool,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            interval_secs: default_interval(),
            enabled: default_enabled(),
        }
    }
}

const fn default_interval() -> u64 {
    120 // 2 minutes
}

const fn default_enabled() -> bool {
    true
}

/// Configuration for storage management.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Maximum storage size in gigabytes.
    #[serde(default = "default_max_size_gb")]
    pub max_size_gb: u64,

    /// Number of days to retain backups.
    #[serde(default = "default_retention_days")]
    pub backup_retention_days: u32,

    /// Whether to compress backups.
    #[serde(default = "default_compression")]
    pub compression: bool,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            max_size_gb: default_max_size_gb(),
            backup_retention_days: default_retention_days(),
            compression: default_compression(),
        }
    }
}

const fn default_max_size_gb() -> u64 {
    10
}

const fn default_retention_days() -> u32 {
    30
}

const fn default_compression() -> bool {
    true
}

/// Path configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PathConfig {
    /// Base data directory.
    #[serde(default)]
    pub data_dir: Option<PathBuf>,
}

impl Default for PathConfig {
    fn default() -> Self {
        Self { data_dir: None }
    }
}

/// Complete application configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    /// Sync daemon configuration.
    #[serde(default)]
    pub sync: SyncConfig,

    /// Storage management configuration.
    #[serde(default)]
    pub storage: StorageConfig,

    /// Path configuration.
    #[serde(default)]
    pub paths: PathConfig,
}

impl AppConfig {
    /// Get the data directory, using default if not configured.
    #[must_use]
    pub fn data_dir(&self) -> PathBuf {
        self.paths
            .data_dir
            .clone()
            .unwrap_or_else(Self::default_data_dir)
    }

    /// Get the default data directory path.
    #[must_use]
    pub fn default_data_dir() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".cursor-chat-handler")
    }

    /// Get the storage database path.
    #[must_use]
    pub fn storage_db_path(&self) -> PathBuf {
        self.data_dir().join("storage.db")
    }

    /// Get the config file path.
    #[must_use]
    pub fn config_file_path(&self) -> PathBuf {
        self.data_dir().join("config.toml")
    }

    /// Get the lock file path.
    #[must_use]
    pub fn lock_file_path(&self) -> PathBuf {
        self.data_dir().join("sync.lock")
    }

    /// Get the exports directory path.
    #[must_use]
    pub fn exports_dir(&self) -> PathBuf {
        self.data_dir().join("exports")
    }

    /// Get the backups directory path.
    #[must_use]
    pub fn backups_dir(&self) -> PathBuf {
        self.data_dir().join("backups")
    }

    /// Maximum storage size in bytes.
    #[must_use]
    pub const fn max_storage_bytes(&self) -> u64 {
        self.storage.max_size_gb * 1024 * 1024 * 1024
    }
}

/// Current state of synchronization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncState {
    /// Last successful sync timestamp.
    pub last_sync: Option<DateTime<Utc>>,

    /// Hash of last synced data (for change detection).
    pub last_hash: Option<String>,

    /// Number of conversations synced.
    pub conversation_count: usize,

    /// Number of messages synced.
    pub message_count: usize,

    /// Total storage used in bytes.
    pub storage_bytes: u64,

    /// Whether a sync is currently in progress.
    pub is_syncing: bool,

    /// Last error message if any.
    pub last_error: Option<String>,
}

impl Default for SyncState {
    fn default() -> Self {
        Self {
            last_sync: None,
            last_hash: None,
            conversation_count: 0,
            message_count: 0,
            storage_bytes: 0,
            is_syncing: false,
            last_error: None,
        }
    }
}

impl SyncState {
    /// Create a new sync state with current timestamp.
    #[must_use]
    pub fn with_sync_time(mut self) -> Self {
        self.last_sync = Some(Utc::now());
        self
    }

    /// Mark sync as in progress.
    #[must_use]
    pub const fn syncing(mut self) -> Self {
        self.is_syncing = true;
        self
    }

    /// Mark sync as completed.
    #[must_use]
    pub const fn completed(mut self) -> Self {
        self.is_syncing = false;
        self
    }

    /// Set error state.
    #[must_use]
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.last_error = Some(error.into());
        self.is_syncing = false;
        self
    }

    /// Clear error state.
    #[must_use]
    pub fn clear_error(mut self) -> Self {
        self.last_error = None;
        self
    }
}

/// Metadata for a backup file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupMetadata {
    /// Unique identifier for this backup.
    pub id: String,

    /// When this backup was created.
    pub created_at: DateTime<Utc>,

    /// Size of backup in bytes.
    pub size_bytes: u64,

    /// Number of conversations in this backup.
    pub conversation_count: usize,

    /// Hash of backup contents.
    pub content_hash: String,

    /// Whether backup is compressed.
    pub is_compressed: bool,

    /// Path to backup file.
    pub file_path: PathBuf,
}

impl BackupMetadata {
    /// Create new backup metadata.
    #[must_use]
    pub fn new(
        id: impl Into<String>,
        size_bytes: u64,
        conversation_count: usize,
        content_hash: impl Into<String>,
        file_path: PathBuf,
    ) -> Self {
        Self {
            id: id.into(),
            created_at: Utc::now(),
            size_bytes,
            conversation_count,
            content_hash: content_hash.into(),
            is_compressed: false,
            file_path,
        }
    }

    /// Mark as compressed.
    #[must_use]
    pub const fn compressed(mut self) -> Self {
        self.is_compressed = true;
        self
    }

    /// Check if backup is expired based on retention days.
    #[must_use]
    pub fn is_expired(&self, retention_days: u32) -> bool {
        let age = Utc::now() - self.created_at;
        age.num_days() > i64::from(retention_days)
    }
}

/// Information about a workspace/project.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkspaceInfo {
    /// Project name (derived from path).
    pub name: String,

    /// Full path to the project.
    pub path: Option<PathBuf>,

    /// Cursor's internal workspace path.
    pub cursor_path: Option<String>,
}

impl WorkspaceInfo {
    /// Create workspace info from a URI like "file:///path/to/project".
    #[must_use]
    pub fn from_uri(uri: &str) -> Self {
        let path = uri
            .strip_prefix("file://")
            .map(PathBuf::from);

        let name = path
            .as_ref()
            .and_then(|p| p.file_name())
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        Self {
            name,
            path,
            cursor_path: None,
        }
    }

    /// Create from path and cursor internal path.
    #[must_use]
    pub fn new(path: PathBuf, cursor_path: Option<String>) -> Self {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        Self {
            name,
            path: Some(path),
            cursor_path,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.sync.interval_secs, 120);
        assert!(config.sync.enabled);
        assert_eq!(config.storage.max_size_gb, 10);
    }

    #[test]
    fn test_sync_state_transitions() {
        let state = SyncState::default()
            .syncing()
            .with_sync_time()
            .completed();

        assert!(!state.is_syncing);
        assert!(state.last_sync.is_some());
    }

    #[test]
    fn test_workspace_from_uri() {
        let ws = WorkspaceInfo::from_uri("file:///home/user/projects/my-app");
        assert_eq!(ws.name, "my-app");
        assert_eq!(
            ws.path,
            Some(PathBuf::from("/home/user/projects/my-app"))
        );
    }

    #[test]
    fn test_backup_expiry() {
        let backup = BackupMetadata::new(
            "test",
            1000,
            5,
            "abc123",
            PathBuf::from("/tmp/backup.db"),
        );

        // Fresh backup should not be expired
        assert!(!backup.is_expired(30));
    }
}

