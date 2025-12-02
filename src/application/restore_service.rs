//! Restore service for recovering chat history.
//!
//! Detects when Cursor's database was cleared and restores
//! chat history from local backup storage.

use std::path::PathBuf;

use crate::domain::{AppConfig, AppError, Result};
use crate::infrastructure::{find_cursor_config_dir, CursorWriter, LocalStorage};

/// Service for restoring chat history to Cursor.
pub struct RestoreService {
    config: AppConfig,
}

impl RestoreService {
    /// Create a new restore service.
    #[must_use]
    pub const fn new(config: AppConfig) -> Self {
        Self { config }
    }

    /// Get the path to Cursor's global state database.
    fn cursor_db_path() -> Result<PathBuf> {
        let config_dir = find_cursor_config_dir()?;
        Ok(config_dir.join("User/globalStorage/state.vscdb"))
    }

    /// Check if Cursor's database appears to have been reset.
    ///
    /// Returns true if local storage has more conversations than Cursor.
    pub fn needs_restore(&self) -> Result<bool> {
        let storage_path = self.config.storage_db_path();
        if !storage_path.exists() {
            return Ok(false);
        }

        let cursor_db = Self::cursor_db_path()?;
        if !cursor_db.exists() {
            return Ok(true); // Cursor DB doesn't exist, needs restore
        }

        // Compare conversation counts
        let local_storage = LocalStorage::open(&storage_path)?;
        let local_count = local_storage.get_conversation_count()?;

        let cursor_writer = CursorWriter::open(&cursor_db)?;
        let cursor_count = cursor_writer.conversation_count()?;

        // If local has significantly more, Cursor was probably reset
        let needs = local_count > 0 && cursor_count < local_count / 2;

        if needs {
            tracing::info!(
                local = local_count,
                cursor = cursor_count,
                "Cursor appears to have been reset"
            );
        }

        Ok(needs)
    }

    /// Check if Cursor's database is completely empty.
    pub fn cursor_is_empty(&self) -> Result<bool> {
        let cursor_db = Self::cursor_db_path()?;
        if !cursor_db.exists() {
            return Ok(true);
        }

        let cursor_writer = CursorWriter::open(&cursor_db)?;
        cursor_writer.is_empty()
    }

    /// Restore all conversations from local storage to Cursor.
    ///
    /// # Errors
    /// Returns error if restore fails.
    pub fn restore_all(&self) -> Result<RestoreResult> {
        let storage_path = self.config.storage_db_path();
        if !storage_path.exists() {
            return Err(AppError::Config {
                message: "No local storage found. Run 'cursor-chat sync now' first.".into(),
            });
        }

        let cursor_db = Self::cursor_db_path()?;

        tracing::info!(
            cursor_db = %cursor_db.display(),
            "Starting restore to Cursor"
        );

        let local_storage = LocalStorage::open(&storage_path)?;
        let cursor_writer = CursorWriter::open(&cursor_db)?;

        // Get all conversations from local storage
        let conversations = local_storage.get_conversations(None)?;

        let mut restored_count = 0;
        let mut message_count = 0;

        for conv in &conversations {
            match cursor_writer.restore_conversation(conv) {
                Ok(()) => {
                    restored_count += 1;
                    message_count += conv.bubbles.len();
                }
                Err(e) => {
                    tracing::warn!(
                        composer_id = &conv.composer_id[..8],
                        error = %e,
                        "Failed to restore conversation"
                    );
                }
            }
        }

        tracing::info!(
            restored = restored_count,
            messages = message_count,
            "Restore completed"
        );

        Ok(RestoreResult {
            restored_conversations: restored_count,
            restored_messages: message_count,
            cursor_db_path: cursor_db,
        })
    }

    /// Restore specific conversations by ID.
    pub fn restore_by_ids(&self, ids: &[String]) -> Result<RestoreResult> {
        let storage_path = self.config.storage_db_path();
        if !storage_path.exists() {
            return Err(AppError::Config {
                message: "No local storage found.".into(),
            });
        }

        let cursor_db = Self::cursor_db_path()?;
        let local_storage = LocalStorage::open(&storage_path)?;
        let cursor_writer = CursorWriter::open(&cursor_db)?;

        let conversations = local_storage.get_conversations(None)?;

        let mut restored_count = 0;
        let mut message_count = 0;

        for conv in &conversations {
            // Check if this conversation matches any of the requested IDs
            let matches = ids.iter().any(|id| {
                conv.composer_id.starts_with(id) || conv.composer_id.contains(id)
            });

            if matches {
                match cursor_writer.restore_conversation(conv) {
                    Ok(()) => {
                        restored_count += 1;
                        message_count += conv.bubbles.len();
                    }
                    Err(e) => {
                        tracing::warn!(
                            composer_id = &conv.composer_id[..8],
                            error = %e,
                            "Failed to restore conversation"
                        );
                    }
                }
            }
        }

        Ok(RestoreResult {
            restored_conversations: restored_count,
            restored_messages: message_count,
            cursor_db_path: cursor_db,
        })
    }

    /// Auto-restore if needed (called by daemon).
    ///
    /// Returns true if restore was performed.
    pub fn auto_restore_if_needed(&self) -> Result<bool> {
        if !self.needs_restore()? {
            return Ok(false);
        }

        tracing::info!("Auto-restore triggered - Cursor database appears reset");

        let result = self.restore_all()?;

        tracing::info!(
            conversations = result.restored_conversations,
            messages = result.restored_messages,
            "Auto-restore completed"
        );

        Ok(true)
    }
}

/// Result of a restore operation.
#[derive(Debug)]
pub struct RestoreResult {
    /// Number of conversations restored.
    pub restored_conversations: usize,
    /// Number of messages restored.
    pub restored_messages: usize,
    /// Path to Cursor's database.
    pub cursor_db_path: PathBuf,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cursor_db_path() {
        // Just verify it doesn't panic
        let _ = RestoreService::cursor_db_path();
    }
}

