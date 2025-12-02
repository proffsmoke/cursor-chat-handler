//! Systemd service integration.
//!
//! Handles generation and installation of systemd user service
//! for the sync daemon.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use crate::domain::{AppConfig, AppError, Result};

/// Service unit file name.
const SERVICE_NAME: &str = "cursor-chat-sync.service";

/// Systemd service manager.
pub struct SystemdService {
    config: AppConfig,
}

impl SystemdService {
    /// Create a new systemd service manager.
    #[must_use]
    pub const fn new(config: AppConfig) -> Self {
        Self { config }
    }

    /// Get the systemd user directory path.
    fn user_systemd_dir() -> Result<PathBuf> {
        let home = dirs::home_dir().ok_or_else(|| AppError::Config {
            message: "Could not determine home directory".into(),
        })?;

        Ok(home.join(".config/systemd/user"))
    }

    /// Get the service file path.
    fn service_file_path() -> Result<PathBuf> {
        Ok(Self::user_systemd_dir()?.join(SERVICE_NAME))
    }

    /// Generate the systemd unit file content.
    fn generate_unit_file(&self) -> Result<String> {
        // Find the binary path
        let binary_path = std::env::current_exe()
            .map_err(|e| AppError::io("Failed to get executable path", e))?;

        let interval_secs = self.config.sync.interval_secs;

        let unit = format!(
            r#"[Unit]
Description=Cursor Chat Handler Sync Daemon
Documentation=https://github.com/user/cursor-chat-handler
After=network.target

[Service]
Type=simple
ExecStart={binary} daemon --interval {interval}
Restart=on-failure
RestartSec=30
Environment=RUST_LOG=info

# Resource limits
MemoryMax=256M
CPUQuota=25%

# Security
ProtectSystem=strict
ProtectHome=read-only
ReadWritePaths={data_dir}
NoNewPrivileges=yes
PrivateTmp=yes

[Install]
WantedBy=default.target
"#,
            binary = binary_path.display(),
            interval = interval_secs,
            data_dir = self.config.data_dir().display(),
        );

        Ok(unit)
    }

    /// Install the systemd service.
    pub fn install(&self) -> Result<InstallResult> {
        // Ensure systemd user directory exists
        let systemd_dir = Self::user_systemd_dir()?;
        fs::create_dir_all(&systemd_dir)
            .map_err(|e| AppError::io("Failed to create systemd user directory", e))?;

        // Write service file
        let service_path = Self::service_file_path()?;
        let unit_content = self.generate_unit_file()?;

        fs::write(&service_path, &unit_content)
            .map_err(|e| AppError::io("Failed to write service file", e))?;

        tracing::info!(path = %service_path.display(), "Service file written");

        // Reload systemd daemon
        let reload_status = Command::new("systemctl")
            .args(["--user", "daemon-reload"])
            .status()
            .map_err(|e| AppError::io("Failed to reload systemd daemon", e))?;

        if !reload_status.success() {
            return Err(AppError::Config {
                message: "Failed to reload systemd daemon".into(),
            });
        }

        Ok(InstallResult {
            service_path,
            unit_content,
        })
    }

    /// Enable and start the service.
    pub fn enable_and_start(&self) -> Result<()> {
        // Enable service
        let enable_status = Command::new("systemctl")
            .args(["--user", "enable", SERVICE_NAME])
            .status()
            .map_err(|e| AppError::io("Failed to enable service", e))?;

        if !enable_status.success() {
            return Err(AppError::Config {
                message: "Failed to enable service".into(),
            });
        }

        // Start service
        let start_status = Command::new("systemctl")
            .args(["--user", "start", SERVICE_NAME])
            .status()
            .map_err(|e| AppError::io("Failed to start service", e))?;

        if !start_status.success() {
            return Err(AppError::Config {
                message: "Failed to start service".into(),
            });
        }

        tracing::info!("Service enabled and started");

        Ok(())
    }

    /// Stop and disable the service.
    pub fn stop_and_disable(&self) -> Result<()> {
        // Stop service (ignore errors if not running)
        let _ = Command::new("systemctl")
            .args(["--user", "stop", SERVICE_NAME])
            .status();

        // Disable service
        let disable_status = Command::new("systemctl")
            .args(["--user", "disable", SERVICE_NAME])
            .status()
            .map_err(|e| AppError::io("Failed to disable service", e))?;

        if !disable_status.success() {
            tracing::warn!("Service may not have been fully disabled");
        }

        tracing::info!("Service stopped and disabled");

        Ok(())
    }

    /// Get service status.
    pub fn get_status(&self) -> Result<ServiceStatus> {
        // Check if service file exists
        let service_path = Self::service_file_path()?;
        let is_installed = service_path.exists();

        if !is_installed {
            return Ok(ServiceStatus {
                is_installed: false,
                is_enabled: false,
                is_running: false,
                status_text: "not installed".into(),
            });
        }

        // Check if enabled
        let enabled_output = Command::new("systemctl")
            .args(["--user", "is-enabled", SERVICE_NAME])
            .output()
            .map_err(|e| AppError::io("Failed to check service enabled status", e))?;

        let is_enabled = enabled_output.status.success();

        // Check if running
        let active_output = Command::new("systemctl")
            .args(["--user", "is-active", SERVICE_NAME])
            .output()
            .map_err(|e| AppError::io("Failed to check service active status", e))?;

        let is_running = active_output.status.success();

        // Get full status
        let status_output = Command::new("systemctl")
            .args(["--user", "status", SERVICE_NAME, "--no-pager"])
            .output()
            .map_err(|e| AppError::io("Failed to get service status", e))?;

        let status_text = String::from_utf8_lossy(&status_output.stdout).to_string();

        Ok(ServiceStatus {
            is_installed,
            is_enabled,
            is_running,
            status_text,
        })
    }

    /// Uninstall the service.
    pub fn uninstall(&self) -> Result<()> {
        // Stop and disable first
        let _ = self.stop_and_disable();

        // Remove service file
        let service_path = Self::service_file_path()?;
        if service_path.exists() {
            fs::remove_file(&service_path)
                .map_err(|e| AppError::io("Failed to remove service file", e))?;
        }

        // Reload daemon
        let _ = Command::new("systemctl")
            .args(["--user", "daemon-reload"])
            .status();

        tracing::info!("Service uninstalled");

        Ok(())
    }

    /// View service logs.
    pub fn view_logs(&self, lines: usize) -> Result<String> {
        let output = Command::new("journalctl")
            .args([
                "--user",
                "-u",
                SERVICE_NAME,
                "-n",
                &lines.to_string(),
                "--no-pager",
            ])
            .output()
            .map_err(|e| AppError::io("Failed to get service logs", e))?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    }
}

/// Result of installing the service.
#[derive(Debug)]
pub struct InstallResult {
    /// Path where service file was written.
    pub service_path: PathBuf,
    /// Content of the unit file.
    pub unit_content: String,
}

/// Service status information.
#[derive(Debug, Clone)]
pub struct ServiceStatus {
    /// Whether the service file is installed.
    pub is_installed: bool,
    /// Whether the service is enabled to start on boot.
    pub is_enabled: bool,
    /// Whether the service is currently running.
    pub is_running: bool,
    /// Full status text from systemctl.
    pub status_text: String,
}

impl ServiceStatus {
    /// Get a short status string.
    #[must_use]
    pub fn short_status(&self) -> &'static str {
        match (self.is_installed, self.is_enabled, self.is_running) {
            (false, _, _) => "not installed",
            (true, false, false) => "installed, disabled",
            (true, true, false) => "enabled, stopped",
            (true, false, true) => "running (not enabled)",
            (true, true, true) => "running",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_status_short() {
        let status = ServiceStatus {
            is_installed: true,
            is_enabled: true,
            is_running: true,
            status_text: String::new(),
        };
        assert_eq!(status.short_status(), "running");

        let status = ServiceStatus {
            is_installed: false,
            is_enabled: false,
            is_running: false,
            status_text: String::new(),
        };
        assert_eq!(status.short_status(), "not installed");
    }
}

