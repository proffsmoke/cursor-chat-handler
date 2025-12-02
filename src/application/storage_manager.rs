//! Storage management service.
//!
//! Handles storage limits, cleanup, and backup rotation.

use std::fs;
use std::path::{Path, PathBuf};

use chrono::{DateTime, Utc};

use crate::domain::{AppConfig, AppError, BackupMetadata, Result};

/// Service for managing storage limits and backups.
pub struct StorageManager {
    config: AppConfig,
}

impl StorageManager {
    /// Create a new storage manager.
    #[must_use]
    pub const fn new(config: AppConfig) -> Self {
        Self { config }
    }

    /// Ensure data directory exists.
    pub fn ensure_directories(&self) -> Result<()> {
        let data_dir = self.config.data_dir();

        fs::create_dir_all(&data_dir)
            .map_err(|e| AppError::io("Failed to create data directory", e))?;

        fs::create_dir_all(self.config.exports_dir())
            .map_err(|e| AppError::io("Failed to create exports directory", e))?;

        fs::create_dir_all(self.config.backups_dir())
            .map_err(|e| AppError::io("Failed to create backups directory", e))?;

        Ok(())
    }

    /// Get total storage usage in bytes.
    pub fn get_total_size(&self) -> Result<u64> {
        let data_dir = self.config.data_dir();
        if !data_dir.exists() {
            return Ok(0);
        }

        calculate_dir_size(&data_dir)
    }

    /// Check if storage is within configured limits.
    pub fn is_within_limits(&self) -> Result<bool> {
        let current = self.get_total_size()?;
        let max = self.config.max_storage_bytes();
        Ok(current < max)
    }

    /// Get storage usage as percentage.
    pub fn get_usage_percent(&self) -> Result<f64> {
        let current = self.get_total_size()?;
        let max = self.config.max_storage_bytes();

        if max == 0 {
            return Ok(0.0);
        }

        Ok((current as f64 / max as f64) * 100.0)
    }

    /// Clean up old backups based on retention policy.
    pub fn cleanup_old_backups(&self) -> Result<CleanupResult> {
        let backups_dir = self.config.backups_dir();
        if !backups_dir.exists() {
            return Ok(CleanupResult::default());
        }

        let retention_days = self.config.storage.backup_retention_days;
        let mut deleted_count = 0;
        let mut freed_bytes = 0u64;

        let entries = fs::read_dir(&backups_dir)
            .map_err(|e| AppError::io("Failed to read backups directory", e))?;

        for entry in entries.filter_map(std::result::Result::ok) {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            // Check file age
            if let Ok(metadata) = fs::metadata(&path) {
                if let Ok(modified) = metadata.modified() {
                    let age = std::time::SystemTime::now()
                        .duration_since(modified)
                        .unwrap_or_default();

                    let age_days = age.as_secs() / 86400;

                    if age_days > u64::from(retention_days) {
                        let size = metadata.len();
                        if fs::remove_file(&path).is_ok() {
                            deleted_count += 1;
                            freed_bytes += size;
                            tracing::info!(
                                path = %path.display(),
                                age_days = age_days,
                                "Deleted old backup"
                            );
                        }
                    }
                }
            }
        }

        Ok(CleanupResult {
            deleted_count,
            freed_bytes,
        })
    }

    /// Run cleanup to bring storage under limit.
    pub fn enforce_storage_limit(&self) -> Result<CleanupResult> {
        let mut total_result = CleanupResult::default();

        // First, clean up old backups
        let backup_result = self.cleanup_old_backups()?;
        total_result.deleted_count += backup_result.deleted_count;
        total_result.freed_bytes += backup_result.freed_bytes;

        // Check if still over limit
        if !self.is_within_limits()? {
            // Clean up exports by age (oldest first)
            let export_result = self.cleanup_exports_by_age()?;
            total_result.deleted_count += export_result.deleted_count;
            total_result.freed_bytes += export_result.freed_bytes;
        }

        Ok(total_result)
    }

    /// Clean up exports by age (oldest first) until under limit.
    fn cleanup_exports_by_age(&self) -> Result<CleanupResult> {
        let exports_dir = self.config.exports_dir();
        if !exports_dir.exists() {
            return Ok(CleanupResult::default());
        }

        // Collect all files with their modification times
        let mut files: Vec<(PathBuf, std::time::SystemTime, u64)> = Vec::new();

        collect_files_recursively(&exports_dir, &mut files)?;

        // Sort by modification time (oldest first)
        files.sort_by(|a, b| a.1.cmp(&b.1));

        let mut deleted_count = 0;
        let mut freed_bytes = 0u64;
        let max_bytes = self.config.max_storage_bytes();

        for (path, _, size) in files {
            // Check if we're under limit
            let current = self.get_total_size()? - freed_bytes;
            if current < max_bytes {
                break;
            }

            if fs::remove_file(&path).is_ok() {
                deleted_count += 1;
                freed_bytes += size;
                tracing::info!(
                    path = %path.display(),
                    size = size,
                    "Deleted export to free space"
                );
            }
        }

        Ok(CleanupResult {
            deleted_count,
            freed_bytes,
        })
    }

    /// List all backups with metadata.
    pub fn list_backups(&self) -> Result<Vec<BackupMetadata>> {
        let backups_dir = self.config.backups_dir();
        if !backups_dir.exists() {
            return Ok(Vec::new());
        }

        let mut backups = Vec::new();

        let entries = fs::read_dir(&backups_dir)
            .map_err(|e| AppError::io("Failed to read backups directory", e))?;

        for entry in entries.filter_map(std::result::Result::ok) {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            if let Some(metadata) = self.read_backup_metadata(&path)? {
                backups.push(metadata);
            }
        }

        // Sort by creation time (newest first)
        backups.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(backups)
    }

    /// Read backup metadata from a file.
    fn read_backup_metadata(&self, path: &Path) -> Result<Option<BackupMetadata>> {
        let metadata = fs::metadata(path)
            .map_err(|e| AppError::io("Failed to read backup metadata", e))?;

        let created_at = metadata
            .modified()
            .ok()
            .and_then(|t| {
                let duration = t.duration_since(std::time::UNIX_EPOCH).ok()?;
                DateTime::from_timestamp(duration.as_secs() as i64, 0)
            })
            .unwrap_or_else(Utc::now);

        let id = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown")
            .to_string();

        let is_compressed = path
            .extension()
            .map_or(false, |ext| ext == "gz" || ext == "zst");

        Ok(Some(BackupMetadata {
            id,
            created_at,
            size_bytes: metadata.len(),
            conversation_count: 0, // Would need to parse file to get this
            content_hash: String::new(),
            is_compressed,
            file_path: path.to_path_buf(),
        }))
    }

    /// Get storage summary.
    pub fn get_summary(&self) -> Result<StorageSummary> {
        let total_bytes = self.get_total_size()?;
        let max_bytes = self.config.max_storage_bytes();

        // Count files in each directory
        let db_size = fs::metadata(self.config.storage_db_path())
            .map(|m| m.len())
            .unwrap_or(0);

        let exports_size = calculate_dir_size(&self.config.exports_dir()).unwrap_or(0);
        let backups_size = calculate_dir_size(&self.config.backups_dir()).unwrap_or(0);

        Ok(StorageSummary {
            total_bytes,
            max_bytes,
            usage_percent: if max_bytes > 0 {
                (total_bytes as f64 / max_bytes as f64) * 100.0
            } else {
                0.0
            },
            db_size,
            exports_size,
            backups_size,
            backup_count: self.list_backups()?.len(),
        })
    }
}

/// Result of a cleanup operation.
#[derive(Debug, Clone, Default)]
pub struct CleanupResult {
    /// Number of files deleted.
    pub deleted_count: usize,
    /// Total bytes freed.
    pub freed_bytes: u64,
}

impl CleanupResult {
    /// Format freed bytes as human readable.
    #[must_use]
    pub fn freed_human(&self) -> String {
        format_bytes(self.freed_bytes)
    }
}

/// Storage summary information.
#[derive(Debug, Clone)]
pub struct StorageSummary {
    /// Total storage used in bytes.
    pub total_bytes: u64,
    /// Maximum allowed storage in bytes.
    pub max_bytes: u64,
    /// Usage percentage (0-100).
    pub usage_percent: f64,
    /// Database size in bytes.
    pub db_size: u64,
    /// Exports directory size in bytes.
    pub exports_size: u64,
    /// Backups directory size in bytes.
    pub backups_size: u64,
    /// Number of backups.
    pub backup_count: usize,
}

impl StorageSummary {
    /// Format total size as human readable.
    #[must_use]
    pub fn total_human(&self) -> String {
        format_bytes(self.total_bytes)
    }

    /// Format max size as human readable.
    #[must_use]
    pub fn max_human(&self) -> String {
        format_bytes(self.max_bytes)
    }

    /// Format database size as human readable.
    #[must_use]
    pub fn db_human(&self) -> String {
        format_bytes(self.db_size)
    }

    /// Format exports size as human readable.
    #[must_use]
    pub fn exports_human(&self) -> String {
        format_bytes(self.exports_size)
    }

    /// Format backups size as human readable.
    #[must_use]
    pub fn backups_human(&self) -> String {
        format_bytes(self.backups_size)
    }
}

/// Calculate total size of a directory recursively.
fn calculate_dir_size(path: &Path) -> Result<u64> {
    if !path.exists() {
        return Ok(0);
    }

    let mut total = 0u64;

    let entries = fs::read_dir(path)
        .map_err(|e| AppError::io(format!("Failed to read directory {}", path.display()), e))?;

    for entry in entries.filter_map(std::result::Result::ok) {
        let path = entry.path();
        let metadata = fs::metadata(&path)
            .map_err(|e| AppError::io(format!("Failed to read metadata {}", path.display()), e))?;

        if metadata.is_file() {
            total += metadata.len();
        } else if metadata.is_dir() {
            total += calculate_dir_size(&path)?;
        }
    }

    Ok(total)
}

/// Collect all files recursively with their metadata.
fn collect_files_recursively(
    path: &Path,
    files: &mut Vec<(PathBuf, std::time::SystemTime, u64)>,
) -> Result<()> {
    if !path.exists() {
        return Ok(());
    }

    let entries = fs::read_dir(path)
        .map_err(|e| AppError::io(format!("Failed to read directory {}", path.display()), e))?;

    for entry in entries.filter_map(std::result::Result::ok) {
        let path = entry.path();
        let metadata = fs::metadata(&path)
            .map_err(|e| AppError::io(format!("Failed to read metadata {}", path.display()), e))?;

        if metadata.is_file() {
            let modified = metadata.modified().unwrap_or(std::time::SystemTime::UNIX_EPOCH);
            files.push((path, modified, metadata.len()));
        } else if metadata.is_dir() {
            collect_files_recursively(&path, files)?;
        }
    }

    Ok(())
}

/// Format bytes as human readable string.
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(0), "0 B");
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1536), "1.50 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
    }

    #[test]
    fn test_cleanup_result_default() {
        let result = CleanupResult::default();
        assert_eq!(result.deleted_count, 0);
        assert_eq!(result.freed_bytes, 0);
    }
}

