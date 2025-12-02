//! Cursor reset operations.
//!
//! Handles killing processes, resetting machine ID, and cleaning config directories
//! for a complete Cursor trial reset.

use std::path::PathBuf;
use std::process::Command;

use crate::domain::{AppError, Result};

/// Configuration directories to clean during reset.
const CONFIG_DIRS: &[&str] = &[
    ".config/Cursor",
    ".cache/Cursor",
    ".local/share/Cursor",
    ".cursor",
    ".cursor-server",
];

/// Desktop entry patterns to clean.
const DESKTOP_PATTERNS: &[&str] = &[
    "cursor*.desktop",
    "co.anysphere.cursor*.desktop",
];

/// Icon patterns to clean.
const ICON_PATTERNS: &[&str] = &[
    "cursor*.*",
    "co.anysphere.cursor*.*",
];

/// Cursor reset service.
pub struct CursorReset {
    /// Whether to clean AppImage files.
    clean_appimage: bool,
    /// Downloads directory for AppImage cleanup.
    downloads_dir: PathBuf,
}

impl CursorReset {
    /// Create a new cursor reset service.
    #[must_use]
    pub fn new(clean_appimage: bool) -> Self {
        let downloads_dir = dirs::download_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap_or_default().join("Downloads"));

        Self {
            clean_appimage,
            downloads_dir,
        }
    }

    /// Kill all running Cursor processes.
    pub fn kill_cursor(&self) -> Result<KillResult> {
        tracing::info!("Killing Cursor processes...");

        let mut killed = 0;

        // Try killall first
        let status = Command::new("killall")
            .arg("cursor")
            .status();

        if status.is_ok() {
            killed += 1;
        }

        // Also try pkill for any remaining
        let _ = Command::new("pkill")
            .args(["-f", "cursor"])
            .status();

        // Give processes time to terminate
        std::thread::sleep(std::time::Duration::from_millis(500));

        Ok(KillResult { killed })
    }

    /// Reset the machine ID (requires sudo).
    pub fn reset_machine_id(&self) -> Result<MachineIdResult> {
        tracing::info!("Resetting machine ID...");

        // Remove existing machine-id files
        let rm_result = Command::new("sudo")
            .args(["rm", "-f", "/etc/machine-id", "/var/lib/dbus/machine-id"])
            .status()
            .map_err(|e| AppError::io("Failed to remove machine-id", e))?;

        if !rm_result.success() {
            return Err(AppError::Config {
                message: "Failed to remove machine-id files (sudo required)".into(),
            });
        }

        // Generate new machine-id
        let setup_result = Command::new("sudo")
            .arg("systemd-machine-id-setup")
            .status()
            .map_err(|e| AppError::io("Failed to setup new machine-id", e))?;

        if !setup_result.success() {
            return Err(AppError::Config {
                message: "Failed to generate new machine-id".into(),
            });
        }

        // Read new machine-id
        let new_id = std::fs::read_to_string("/etc/machine-id")
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|_| "unknown".to_string());

        Ok(MachineIdResult { new_id })
    }

    /// Clean Cursor configuration directories.
    pub fn clean_config_dirs(&self) -> Result<CleanupStats> {
        tracing::info!("Cleaning configuration directories...");

        let home = dirs::home_dir().ok_or_else(|| AppError::Config {
            message: "Could not determine home directory".into(),
        })?;

        let mut stats = CleanupStats::default();

        for dir_name in CONFIG_DIRS {
            let dir_path = home.join(dir_name);

            if dir_path.exists() {
                match std::fs::remove_dir_all(&dir_path) {
                    Ok(()) => {
                        stats.dirs_removed += 1;
                        stats.paths_cleaned.push(dir_path.display().to_string());
                        tracing::debug!(path = %dir_path.display(), "Removed directory");
                    }
                    Err(e) => {
                        tracing::warn!(
                            path = %dir_path.display(),
                            error = %e,
                            "Failed to remove directory"
                        );
                    }
                }
            } else {
                stats.paths_skipped.push(dir_path.display().to_string());
            }
        }

        Ok(stats)
    }

    /// Clean desktop entries and icons.
    pub fn clean_desktop_entries(&self) -> Result<CleanupStats> {
        tracing::info!("Cleaning desktop entries and icons...");

        let home = dirs::home_dir().ok_or_else(|| AppError::Config {
            message: "Could not determine home directory".into(),
        })?;

        let mut stats = CleanupStats::default();

        // User desktop entries
        let user_apps = home.join(".local/share/applications");
        self.clean_by_patterns(&user_apps, DESKTOP_PATTERNS, &mut stats);

        // User icons
        let user_icons = home.join(".local/share/icons");
        self.clean_by_patterns(&user_icons, ICON_PATTERNS, &mut stats);

        // System desktop entries (requires sudo)
        self.clean_system_files(&mut stats);

        // Update desktop database
        let _ = Command::new("update-desktop-database")
            .arg(user_apps)
            .status();

        Ok(stats)
    }

    /// Clean files matching patterns in a directory.
    fn clean_by_patterns(&self, dir: &PathBuf, patterns: &[&str], stats: &mut CleanupStats) {
        if !dir.exists() {
            return;
        }

        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.filter_map(std::result::Result::ok) {
                let path = entry.path();
                let name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");

                for pattern in patterns {
                    if matches_pattern(name, pattern) {
                        if std::fs::remove_file(&path).is_ok() {
                            stats.files_removed += 1;
                            stats.paths_cleaned.push(path.display().to_string());
                        }
                        break;
                    }
                }
            }
        }
    }

    /// Clean system-level files (requires sudo).
    fn clean_system_files(&self, stats: &mut CleanupStats) {
        let system_paths = [
            "/usr/share/applications/cursor*.desktop",
            "/usr/share/applications/co.anysphere.cursor*.desktop",
            "/usr/share/icons/hicolor/*/apps/cursor.png",
            "/usr/share/icons/hicolor/*/apps/co.anysphere.cursor.*",
            "/usr/share/pixmaps/cursor*.*",
            "/usr/share/pixmaps/co.anysphere.cursor.*",
        ];

        for pattern in system_paths {
            // Use sudo rm with glob pattern via sh
            let result = Command::new("sudo")
                .args(["sh", "-c", &format!("rm -f {}", pattern)])
                .status();

            if result.is_ok() {
                stats.files_removed += 1;
            }
        }

        // Update system desktop database
        let _ = Command::new("sudo")
            .args(["update-desktop-database", "/usr/share/applications"])
            .status();
    }

    /// Clean AppImage files from downloads directory.
    pub fn clean_appimages(&self) -> Result<CleanupStats> {
        if !self.clean_appimage {
            return Ok(CleanupStats::default());
        }

        tracing::info!("Cleaning AppImage files...");

        let mut stats = CleanupStats::default();

        if !self.downloads_dir.exists() {
            return Ok(stats);
        }

        if let Ok(entries) = std::fs::read_dir(&self.downloads_dir) {
            for entry in entries.filter_map(std::result::Result::ok) {
                let path = entry.path();
                let name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("");

                // Match Cursor-*.AppImage pattern
                if name.starts_with("Cursor-") && name.ends_with(".AppImage") {
                    if std::fs::remove_file(&path).is_ok() {
                        stats.files_removed += 1;
                        stats.paths_cleaned.push(path.display().to_string());
                        tracing::info!(path = %path.display(), "Removed AppImage");
                    }
                }
            }
        }

        // Also clean extracted squashfs-root
        let home = dirs::home_dir().unwrap_or_default();
        let squashfs = home.join("squashfs-root");
        if squashfs.exists() {
            if std::fs::remove_dir_all(&squashfs).is_ok() {
                stats.dirs_removed += 1;
                stats.paths_cleaned.push(squashfs.display().to_string());
            }
        }

        Ok(stats)
    }

    /// Run the complete reset process.
    pub fn run_full_reset(&self) -> Result<ResetResult> {
        let kill_result = self.kill_cursor()?;
        let config_stats = self.clean_config_dirs()?;
        let desktop_stats = self.clean_desktop_entries()?;
        let appimage_stats = self.clean_appimages()?;
        let machine_id = self.reset_machine_id()?;

        Ok(ResetResult {
            kill_result,
            config_stats,
            desktop_stats,
            appimage_stats,
            machine_id,
        })
    }
}

/// Simple glob pattern matching (supports * wildcard).
fn matches_pattern(name: &str, pattern: &str) -> bool {
    if pattern.contains('*') {
        let parts: Vec<&str> = pattern.split('*').collect();

        if parts.len() == 2 {
            // Pattern like "cursor*" or "*cursor" or "cursor*.desktop"
            let (prefix, suffix) = (parts[0], parts[1]);
            return name.starts_with(prefix) && name.ends_with(suffix);
        }
    }

    name == pattern
}

/// Result of killing processes.
#[derive(Debug, Default)]
pub struct KillResult {
    /// Number of processes signaled.
    pub killed: usize,
}

/// Result of machine ID reset.
#[derive(Debug)]
pub struct MachineIdResult {
    /// New machine ID.
    pub new_id: String,
}

/// Statistics from cleanup operations.
#[derive(Debug, Default)]
pub struct CleanupStats {
    /// Number of directories removed.
    pub dirs_removed: usize,
    /// Number of files removed.
    pub files_removed: usize,
    /// Paths that were cleaned.
    pub paths_cleaned: Vec<String>,
    /// Paths that were skipped (not found).
    pub paths_skipped: Vec<String>,
}

impl CleanupStats {
    /// Merge another stats into this one.
    pub fn merge(&mut self, other: CleanupStats) {
        self.dirs_removed += other.dirs_removed;
        self.files_removed += other.files_removed;
        self.paths_cleaned.extend(other.paths_cleaned);
        self.paths_skipped.extend(other.paths_skipped);
    }
}

/// Complete reset result.
#[derive(Debug)]
pub struct ResetResult {
    /// Process kill result.
    pub kill_result: KillResult,
    /// Config cleanup stats.
    pub config_stats: CleanupStats,
    /// Desktop entries cleanup stats.
    pub desktop_stats: CleanupStats,
    /// AppImage cleanup stats.
    pub appimage_stats: CleanupStats,
    /// Machine ID reset result.
    pub machine_id: MachineIdResult,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_pattern() {
        assert!(matches_pattern("cursor.desktop", "cursor*.desktop"));
        assert!(matches_pattern("cursor-app.desktop", "cursor*.desktop"));
        assert!(!matches_pattern("other.desktop", "cursor*.desktop"));

        assert!(matches_pattern("cursor.png", "cursor*.*"));
        assert!(matches_pattern("cursor-icon.svg", "cursor*.*"));
    }

    #[test]
    fn test_cleanup_stats_merge() {
        let mut stats1 = CleanupStats {
            dirs_removed: 2,
            files_removed: 5,
            paths_cleaned: vec!["a".into()],
            paths_skipped: vec![],
        };

        let stats2 = CleanupStats {
            dirs_removed: 1,
            files_removed: 3,
            paths_cleaned: vec!["b".into()],
            paths_skipped: vec!["c".into()],
        };

        stats1.merge(stats2);

        assert_eq!(stats1.dirs_removed, 3);
        assert_eq!(stats1.files_removed, 8);
        assert_eq!(stats1.paths_cleaned.len(), 2);
    }
}

