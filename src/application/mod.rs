//! Application layer - use cases and orchestration.
//!
//! This layer contains the main business logic for extracting
//! and formatting chat data.

pub mod extractor;
pub mod formatter;
pub mod parser;
pub mod restore_service;
pub mod storage_manager;
pub mod sync_service;

pub use extractor::{extract_all_conversations, ExtractOptions};
pub use formatter::{
    format_conversation_markdown, format_conversations_json, format_conversations_table,
    format_stats, OutputFormat,
};
pub use restore_service::{RestoreResult, RestoreService};
pub use storage_manager::{CleanupResult, StorageManager, StorageSummary};
pub use sync_service::{StorageInfo, SyncService};
