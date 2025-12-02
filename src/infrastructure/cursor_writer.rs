//! Cursor database writer for restoring chat history.
//!
//! Writes chat data back to Cursor's SQLite database,
//! restoring conversations after a reset/cleanup.

use std::path::Path;

use rusqlite::{params, Connection, OpenFlags};

use crate::domain::{AppError, Bubble, BubbleType, Conversation, Result};

/// Writer for Cursor's state database.
pub struct CursorWriter {
    conn: Connection,
}

impl CursorWriter {
    /// Opens the Cursor database for writing.
    ///
    /// # Errors
    /// Returns error if database cannot be opened.
    pub fn open(path: &Path) -> Result<Self> {
        let flags = OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_NO_MUTEX;

        let conn = Connection::open_with_flags(path, flags).map_err(AppError::database)?;

        // Ensure table exists
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS cursorDiskKV (
                key TEXT PRIMARY KEY,
                value BLOB
            );",
        )
        .map_err(AppError::database)?;

        Ok(Self { conn })
    }

    /// Restore a conversation to Cursor's database.
    ///
    /// # Errors
    /// Returns error if write fails.
    pub fn restore_conversation(&self, conv: &Conversation) -> Result<()> {
        // Write composer data
        let composer_key = format!("composerData:{}", conv.composer_id);
        let composer_value = self.serialize_composer(conv)?;

        self.conn
            .execute(
                "INSERT OR REPLACE INTO cursorDiskKV (key, value) VALUES (?1, ?2)",
                params![composer_key, composer_value],
            )
            .map_err(AppError::database)?;

        // Write bubbles
        for bubble in &conv.bubbles {
            let bubble_key = format!("bubbleId:{}:{}", conv.composer_id, bubble.bubble_id);
            let bubble_value = self.serialize_bubble(bubble)?;

            self.conn
                .execute(
                    "INSERT OR REPLACE INTO cursorDiskKV (key, value) VALUES (?1, ?2)",
                    params![bubble_key, bubble_value],
                )
                .map_err(AppError::database)?;
        }

        tracing::debug!(
            composer_id = &conv.composer_id[..8],
            bubbles = conv.bubbles.len(),
            "Restored conversation"
        );

        Ok(())
    }

    /// Serialize composer data to JSON.
    fn serialize_composer(&self, conv: &Conversation) -> Result<Vec<u8>> {
        let data = serde_json::json!({
            "_v": 10,
            "composerId": conv.composer_id,
            "createdAt": conv.created_at.map(|dt| dt.timestamp_millis()),
            "modelConfig": {
                "modelName": conv.model_config.model_name,
                "maxMode": conv.model_config.max_mode
            },
            "unifiedMode": conv.unified_mode,
            "richText": "",
            "text": "",
            "hasLoaded": true,
            "status": "none",
            "context": {
                "composers": [],
                "quotes": [],
                "selectedCommits": [],
                "selectedPullRequests": [],
                "selectedImages": [],
                "folderSelections": [],
                "fileSelections": [],
                "selections": [],
                "terminalSelections": [],
                "selectedDocs": [],
                "externalLinks": [],
                "cursorRules": [],
                "cursorCommands": [],
                "uiElementSelections": [],
                "consoleLogs": []
            }
        });

        serde_json::to_vec(&data).map_err(AppError::json_parse)
    }

    /// Serialize bubble data to JSON.
    fn serialize_bubble(&self, bubble: &Bubble) -> Result<Vec<u8>> {
        let bubble_type: u8 = match bubble.bubble_type {
            BubbleType::User => 1,
            BubbleType::Assistant => 2,
            BubbleType::Unknown => 0,
        };

        let mut data = serde_json::json!({
            "_v": 10,
            "bubbleId": bubble.bubble_id,
            "type": bubble_type,
            "text": bubble.text,
            "createdAt": bubble.created_at.map(|dt| dt.to_rfc3339()),
            "isAgentic": bubble.is_agentic,
            "tokenCount": {
                "inputTokens": bubble.token_count.input_tokens,
                "outputTokens": bubble.token_count.output_tokens
            }
        });

        // Add thinking if present
        if let Some(ref thinking) = bubble.thinking {
            data["thinking"] = serde_json::json!({
                "text": thinking.text,
                "signature": thinking.signature
            });
            data["thinkingDurationMs"] = serde_json::json!(bubble.thinking_duration_ms);
        }

        serde_json::to_vec(&data).map_err(AppError::json_parse)
    }

    /// Check if database is empty (was reset).
    pub fn is_empty(&self) -> Result<bool> {
        let count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM cursorDiskKV WHERE key LIKE 'composerData:%'",
                [],
                |row| row.get(0),
            )
            .map_err(AppError::database)?;

        Ok(count == 0)
    }

    /// Get count of conversations in Cursor DB.
    pub fn conversation_count(&self) -> Result<usize> {
        let count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM cursorDiskKV WHERE key LIKE 'composerData:%'",
                [],
                |row| row.get(0),
            )
            .map_err(AppError::database)?;

        Ok(count as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_open_creates_table() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.vscdb");

        let writer = CursorWriter::open(&db_path).unwrap();
        assert!(writer.is_empty().unwrap());
    }
}

