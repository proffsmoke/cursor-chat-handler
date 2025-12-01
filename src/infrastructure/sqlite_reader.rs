//! `SQLite` database reader for Cursor's state.vscdb files.
//!
//! Extracts chat data from the `cursorDiskKV` table.

use std::path::Path;

use rusqlite::types::ValueRef;
use rusqlite::{Connection, OpenFlags};

use crate::domain::{AppError, Result};

/// Key prefixes used in Cursor's KV store.
const BUBBLE_PREFIX: &str = "bubbleId:";
const COMPOSER_PREFIX: &str = "composerData:";

/// Raw key-value pair from the database.
#[derive(Debug)]
pub struct RawKvEntry {
    pub key: String,
    pub value: Vec<u8>,
}

/// `SQLite` reader for Cursor state databases.
pub struct StateDbReader {
    conn: Connection,
}

impl StateDbReader {
    /// Opens a state database in read-only mode.
    ///
    /// # Errors
    /// Returns error if database cannot be opened.
    pub fn open(path: &Path) -> Result<Self> {
        let flags = OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX;

        let conn = Connection::open_with_flags(path, flags).map_err(AppError::database)?;

        // Optimize for read-only access
        conn.execute_batch(
            "PRAGMA query_only = ON;
             PRAGMA temp_store = MEMORY;",
        )
        .map_err(AppError::database)?;

        Ok(Self { conn })
    }

    /// Fetches all bubble entries from the database.
    ///
    /// # Errors
    /// Returns error if query fails.
    pub fn fetch_bubbles(&self) -> Result<Vec<RawKvEntry>> {
        self.fetch_by_prefix(BUBBLE_PREFIX)
    }

    /// Fetches all composer (conversation) entries from the database.
    ///
    /// # Errors
    /// Returns error if query fails.
    pub fn fetch_composers(&self) -> Result<Vec<RawKvEntry>> {
        self.fetch_by_prefix(COMPOSER_PREFIX)
    }

    /// Fetches entries matching a key prefix.
    fn fetch_by_prefix(&self, prefix: &str) -> Result<Vec<RawKvEntry>> {
        let mut stmt = self
            .conn
            .prepare("SELECT key, value FROM cursorDiskKV WHERE key LIKE ?1")
            .map_err(AppError::database)?;

        let pattern = format!("{prefix}%");
        let rows = stmt
            .query_map([&pattern], |row| {
                let key: String = row.get(0)?;
                // Handle both TEXT and BLOB value types
                let value = match row.get_ref(1)? {
                    ValueRef::Blob(b) => b.to_vec(),
                    ValueRef::Text(t) => t.to_vec(),
                    _ => Vec::new(),
                };
                Ok(RawKvEntry { key, value })
            })
            .map_err(AppError::database)?;

        let mut entries = Vec::new();
        for row in rows {
            match row {
                Ok(entry) => entries.push(entry),
                Err(e) => {
                    tracing::warn!("Failed to read row: {}", e);
                }
            }
        }

        tracing::debug!("Fetched {} entries with prefix '{}'", entries.len(), prefix);

        Ok(entries)
    }
}
