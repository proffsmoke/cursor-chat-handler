//! Local SQLite storage for synced chat data.
//!
//! Provides persistent storage that survives Cursor resets,
//! with support for incremental sync and workspace organization.

use std::path::Path;

use chrono::{DateTime, Utc};
use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::{
    AppError, Bubble, BubbleType, Conversation, ModelConfig, Result, SyncState, WorkspaceInfo,
};

/// Local storage repository using SQLite.
pub struct LocalStorage {
    conn: Connection,
}

impl LocalStorage {
    /// Opens or creates the local storage database.
    ///
    /// # Errors
    /// Returns error if database cannot be opened or schema creation fails.
    pub fn open(path: &Path) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| AppError::io("Failed to create storage directory", e))?;
        }

        let conn = Connection::open(path).map_err(AppError::database)?;

        // Enable WAL mode for better concurrent access
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA foreign_keys = ON;",
        )
        .map_err(AppError::database)?;

        let storage = Self { conn };
        storage.init_schema()?;

        Ok(storage)
    }

    /// Initialize database schema.
    fn init_schema(&self) -> Result<()> {
        self.conn
            .execute_batch(
                r"
            -- Workspaces/Projects table
            CREATE TABLE IF NOT EXISTS workspaces (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                path TEXT UNIQUE,
                cursor_path TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            -- Conversations table
            CREATE TABLE IF NOT EXISTS conversations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                composer_id TEXT UNIQUE NOT NULL,
                workspace_id INTEGER REFERENCES workspaces(id),
                title TEXT NOT NULL DEFAULT '',
                model_name TEXT NOT NULL DEFAULT '',
                max_mode INTEGER NOT NULL DEFAULT 0,
                unified_mode TEXT NOT NULL DEFAULT '',
                created_at TEXT,
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                content_hash TEXT
            );

            -- Bubbles/Messages table
            CREATE TABLE IF NOT EXISTS bubbles (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                bubble_id TEXT UNIQUE NOT NULL,
                conversation_id INTEGER NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
                bubble_type INTEGER NOT NULL,
                text TEXT NOT NULL DEFAULT '',
                created_at TEXT,
                thinking_text TEXT,
                thinking_signature TEXT,
                thinking_duration_ms INTEGER,
                input_tokens INTEGER NOT NULL DEFAULT 0,
                output_tokens INTEGER NOT NULL DEFAULT 0,
                is_agentic INTEGER NOT NULL DEFAULT 0,
                workspace_uri TEXT,
                workspace_project_dir TEXT
            );

            -- Sync state table
            CREATE TABLE IF NOT EXISTS sync_state (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                last_sync TEXT,
                last_hash TEXT,
                conversation_count INTEGER NOT NULL DEFAULT 0,
                message_count INTEGER NOT NULL DEFAULT 0,
                storage_bytes INTEGER NOT NULL DEFAULT 0,
                is_syncing INTEGER NOT NULL DEFAULT 0,
                last_error TEXT
            );

            -- Initialize sync state if not exists
            INSERT OR IGNORE INTO sync_state (id) VALUES (1);

            -- Indexes for common queries
            CREATE INDEX IF NOT EXISTS idx_conversations_workspace 
                ON conversations(workspace_id);
            CREATE INDEX IF NOT EXISTS idx_conversations_created 
                ON conversations(created_at DESC);
            CREATE INDEX IF NOT EXISTS idx_bubbles_conversation 
                ON bubbles(conversation_id);
            CREATE INDEX IF NOT EXISTS idx_bubbles_created 
                ON bubbles(created_at);
            ",
            )
            .map_err(AppError::database)?;

        Ok(())
    }

    /// Get or create a workspace by path.
    pub fn get_or_create_workspace(&self, info: &WorkspaceInfo) -> Result<i64> {
        // Try to find existing
        if let Some(path) = &info.path {
            let path_str = path.to_string_lossy();
            if let Some(id) = self
                .conn
                .query_row(
                    "SELECT id FROM workspaces WHERE path = ?1",
                    [path_str.as_ref()],
                    |row| row.get::<_, i64>(0),
                )
                .optional()
                .map_err(AppError::database)?
            {
                return Ok(id);
            }
        }

        // Create new
        self.conn
            .execute(
                "INSERT INTO workspaces (name, path, cursor_path) VALUES (?1, ?2, ?3)",
                params![
                    &info.name,
                    info.path.as_ref().map(|p| p.to_string_lossy().to_string()),
                    &info.cursor_path,
                ],
            )
            .map_err(AppError::database)?;

        Ok(self.conn.last_insert_rowid())
    }

    /// Upsert a conversation.
    pub fn upsert_conversation(
        &self,
        conv: &Conversation,
        workspace_id: Option<i64>,
        content_hash: Option<&str>,
    ) -> Result<i64> {
        self.conn
            .execute(
                r"
            INSERT INTO conversations 
                (composer_id, workspace_id, title, model_name, max_mode, unified_mode, created_at, content_hash)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(composer_id) DO UPDATE SET
                workspace_id = COALESCE(excluded.workspace_id, workspace_id),
                title = excluded.title,
                model_name = excluded.model_name,
                max_mode = excluded.max_mode,
                unified_mode = excluded.unified_mode,
                updated_at = datetime('now'),
                content_hash = excluded.content_hash
            ",
                params![
                    &conv.composer_id,
                    workspace_id,
                    &conv.title,
                    &conv.model_config.model_name,
                    conv.model_config.max_mode as i32,
                    &conv.unified_mode,
                    conv.created_at.map(|dt| dt.to_rfc3339()),
                    content_hash,
                ],
            )
            .map_err(AppError::database)?;

        // Get the ID
        self.conn
            .query_row(
                "SELECT id FROM conversations WHERE composer_id = ?1",
                [&conv.composer_id],
                |row| row.get(0),
            )
            .map_err(AppError::database)
    }

    /// Upsert a bubble/message.
    pub fn upsert_bubble(&self, bubble: &Bubble, conversation_id: i64) -> Result<()> {
        self.conn
            .execute(
                r"
            INSERT INTO bubbles 
                (bubble_id, conversation_id, bubble_type, text, created_at,
                 thinking_text, thinking_signature, thinking_duration_ms,
                 input_tokens, output_tokens, is_agentic)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            ON CONFLICT(bubble_id) DO UPDATE SET
                text = excluded.text,
                thinking_text = excluded.thinking_text,
                thinking_signature = excluded.thinking_signature,
                thinking_duration_ms = excluded.thinking_duration_ms,
                input_tokens = excluded.input_tokens,
                output_tokens = excluded.output_tokens,
                is_agentic = excluded.is_agentic
            ",
                params![
                    &bubble.bubble_id,
                    conversation_id,
                    bubble.bubble_type as u8,
                    &bubble.text,
                    bubble.created_at.map(|dt| dt.to_rfc3339()),
                    bubble.thinking.as_ref().map(|t| &t.text),
                    bubble.thinking.as_ref().and_then(|t| t.signature.as_ref()),
                    bubble.thinking_duration_ms,
                    bubble.token_count.input_tokens as i64,
                    bubble.token_count.output_tokens as i64,
                    bubble.is_agentic as i32,
                ],
            )
            .map_err(AppError::database)?;

        Ok(())
    }

    /// Get all conversations, optionally filtered by workspace.
    pub fn get_conversations(&self, workspace_name: Option<&str>) -> Result<Vec<Conversation>> {
        let query = if workspace_name.is_some() {
            r"
            SELECT c.composer_id, c.title, c.model_name, c.max_mode, c.unified_mode, c.created_at
            FROM conversations c
            JOIN workspaces w ON c.workspace_id = w.id
            WHERE w.name = ?1
            ORDER BY c.created_at DESC
            "
        } else {
            r"
            SELECT composer_id, title, model_name, max_mode, unified_mode, created_at
            FROM conversations
            ORDER BY created_at DESC
            "
        };

        let mut stmt = self.conn.prepare(query).map_err(AppError::database)?;

        let rows = if let Some(name) = workspace_name {
            stmt.query_map([name], Self::row_to_conversation)
        } else {
            stmt.query_map([], Self::row_to_conversation)
        }
        .map_err(AppError::database)?;

        let mut conversations = Vec::new();
        for row in rows {
            if let Ok(mut conv) = row {
                // Load bubbles for this conversation
                conv.bubbles = self.get_bubbles(&conv.composer_id)?;
                conversations.push(conv);
            }
        }

        Ok(conversations)
    }

    /// Convert a row to a Conversation.
    fn row_to_conversation(row: &rusqlite::Row) -> rusqlite::Result<Conversation> {
        let created_at_str: Option<String> = row.get(5)?;
        let created_at = created_at_str.and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        Ok(Conversation {
            composer_id: row.get(0)?,
            title: row.get(1)?,
            model_config: ModelConfig {
                model_name: row.get(2)?,
                max_mode: row.get::<_, i32>(3)? != 0,
            },
            unified_mode: row.get(4)?,
            created_at,
            bubbles: Vec::new(),
        })
    }

    /// Get bubbles for a conversation.
    pub fn get_bubbles(&self, composer_id: &str) -> Result<Vec<Bubble>> {
        let mut stmt = self
            .conn
            .prepare(
                r"
            SELECT b.bubble_id, b.bubble_type, b.text, b.created_at,
                   b.thinking_text, b.thinking_signature, b.thinking_duration_ms,
                   b.input_tokens, b.output_tokens, b.is_agentic
            FROM bubbles b
            JOIN conversations c ON b.conversation_id = c.id
            WHERE c.composer_id = ?1
            ORDER BY b.created_at ASC
            ",
            )
            .map_err(AppError::database)?;

        let rows = stmt
            .query_map([composer_id], |row| {
                let created_at_str: Option<String> = row.get(3)?;
                let created_at = created_at_str
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc));

                let thinking_text: Option<String> = row.get(4)?;
                let thinking_signature: Option<String> = row.get(5)?;
                let thinking = thinking_text.map(|text| crate::domain::ThinkingBlock {
                    text,
                    signature: thinking_signature,
                });

                Ok(Bubble {
                    bubble_id: row.get(0)?,
                    bubble_type: BubbleType::try_from(row.get::<_, u8>(1)?)
                        .unwrap_or(BubbleType::Unknown),
                    text: row.get(2)?,
                    created_at,
                    thinking,
                    thinking_duration_ms: row.get(6)?,
                    token_count: crate::domain::TokenCount {
                        input_tokens: row.get::<_, i64>(7)? as u64,
                        output_tokens: row.get::<_, i64>(8)? as u64,
                    },
                    is_agentic: row.get::<_, i32>(9)? != 0,
                })
            })
            .map_err(AppError::database)?;

        let mut bubbles = Vec::new();
        for row in rows {
            if let Ok(bubble) = row {
                bubbles.push(bubble);
            }
        }

        Ok(bubbles)
    }

    /// Get sync state.
    pub fn get_sync_state(&self) -> Result<SyncState> {
        self.conn
            .query_row(
                r"
            SELECT last_sync, last_hash, conversation_count, message_count,
                   storage_bytes, is_syncing, last_error
            FROM sync_state WHERE id = 1
            ",
                [],
                |row| {
                    let last_sync_str: Option<String> = row.get(0)?;
                    let last_sync = last_sync_str
                        .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                        .map(|dt| dt.with_timezone(&Utc));

                    Ok(SyncState {
                        last_sync,
                        last_hash: row.get(1)?,
                        conversation_count: row.get::<_, i64>(2)? as usize,
                        message_count: row.get::<_, i64>(3)? as usize,
                        storage_bytes: row.get::<_, i64>(4)? as u64,
                        is_syncing: row.get::<_, i32>(5)? != 0,
                        last_error: row.get(6)?,
                    })
                },
            )
            .map_err(AppError::database)
    }

    /// Update sync state.
    pub fn update_sync_state(&self, state: &SyncState) -> Result<()> {
        self.conn
            .execute(
                r"
            UPDATE sync_state SET
                last_sync = ?1,
                last_hash = ?2,
                conversation_count = ?3,
                message_count = ?4,
                storage_bytes = ?5,
                is_syncing = ?6,
                last_error = ?7
            WHERE id = 1
            ",
                params![
                    state.last_sync.map(|dt| dt.to_rfc3339()),
                    &state.last_hash,
                    state.conversation_count as i64,
                    state.message_count as i64,
                    state.storage_bytes as i64,
                    state.is_syncing as i32,
                    &state.last_error,
                ],
            )
            .map_err(AppError::database)?;

        Ok(())
    }

    /// Get total storage size in bytes.
    pub fn get_storage_size(&self) -> Result<u64> {
        let path = match self.conn.path() {
            Some(p) => Path::new(p),
            None => return Ok(0),
        };
        let metadata = std::fs::metadata(path)
            .map_err(|e| AppError::io("Failed to get storage size", e))?;
        Ok(metadata.len())
    }

    /// Get conversation count.
    pub fn get_conversation_count(&self) -> Result<usize> {
        self.conn
            .query_row("SELECT COUNT(*) FROM conversations", [], |row| {
                row.get::<_, i64>(0)
            })
            .map(|c| c as usize)
            .map_err(AppError::database)
    }

    /// Get message count.
    pub fn get_message_count(&self) -> Result<usize> {
        self.conn
            .query_row("SELECT COUNT(*) FROM bubbles", [], |row| {
                row.get::<_, i64>(0)
            })
            .map(|c| c as usize)
            .map_err(AppError::database)
    }

    /// Check if a conversation exists and get its hash.
    pub fn get_conversation_hash(&self, composer_id: &str) -> Result<Option<String>> {
        self.conn
            .query_row(
                "SELECT content_hash FROM conversations WHERE composer_id = ?1",
                [composer_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(AppError::database)
    }

    /// Get all workspace names.
    pub fn get_workspaces(&self) -> Result<Vec<WorkspaceInfo>> {
        let mut stmt = self
            .conn
            .prepare("SELECT name, path, cursor_path FROM workspaces ORDER BY name")
            .map_err(AppError::database)?;

        let rows = stmt
            .query_map([], |row| {
                let path_str: Option<String> = row.get(1)?;
                Ok(WorkspaceInfo {
                    name: row.get(0)?,
                    path: path_str.map(std::path::PathBuf::from),
                    cursor_path: row.get(2)?,
                })
            })
            .map_err(AppError::database)?;

        let mut workspaces = Vec::new();
        for row in rows {
            if let Ok(ws) = row {
                workspaces.push(ws);
            }
        }

        Ok(workspaces)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_open_creates_schema() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");

        let storage = LocalStorage::open(&db_path).unwrap();

        // Check tables exist
        let count: i64 = storage
            .conn
            .query_row(
                "SELECT COUNT(*) FROM sqlite_master WHERE type='table'",
                [],
                |row| row.get(0),
            )
            .unwrap();

        assert!(count >= 4); // At least 4 tables
    }

    #[test]
    fn test_sync_state_roundtrip() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let storage = LocalStorage::open(&db_path).unwrap();

        let state = SyncState::default()
            .with_sync_time()
            .completed();

        storage.update_sync_state(&state).unwrap();
        let loaded = storage.get_sync_state().unwrap();

        assert!(!loaded.is_syncing);
        assert!(loaded.last_sync.is_some());
    }
}

