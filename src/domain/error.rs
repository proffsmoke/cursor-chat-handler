//! Domain-level error types for cursor-chat-handler.
//!
//! All errors are typed with `thiserror` and provide meaningful context
//! without exposing internal details to end users.

use std::path::PathBuf;
use thiserror::Error;

/// Application-level errors with proper HTTP status mapping (for future API use).
#[derive(Error, Debug)]
pub enum AppError {
    /// Database file not found at expected location.
    #[error("Cursor database not found at: {path}")]
    DatabaseNotFound { path: PathBuf },

    /// Failed to open or query the database.
    #[error("Database error: {message}")]
    Database {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Invalid or corrupted data in database.
    #[error("Invalid data: {message}")]
    InvalidData { message: String },

    /// JSON parsing failed.
    #[error("JSON parse error: {message}")]
    JsonParse {
        message: String,
        #[source]
        source: Option<serde_json::Error>,
    },

    /// Configuration or environment error.
    #[error("Configuration error: {message}")]
    Config { message: String },

    /// IO operation failed.
    #[error("IO error: {message}")]
    Io {
        message: String,
        #[source]
        source: Option<std::io::Error>,
    },
}

impl AppError {
    /// Create a database error from rusqlite error.
    pub fn database(err: rusqlite::Error) -> Self {
        Self::Database {
            message: err.to_string(),
            source: Some(Box::new(err)),
        }
    }

    /// Create a JSON parse error.
    pub fn json_parse(err: serde_json::Error) -> Self {
        Self::JsonParse {
            message: err.to_string(),
            source: Some(err),
        }
    }

    /// Create an IO error with context.
    pub fn io(message: impl Into<String>, err: std::io::Error) -> Self {
        Self::Io {
            message: message.into(),
            source: Some(err),
        }
    }
}

/// Result type alias using `AppError`.
pub type Result<T> = std::result::Result<T, AppError>;
