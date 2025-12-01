//! Application layer - use cases and orchestration.
//!
//! This layer contains the main business logic for extracting
//! and formatting chat data.

pub mod extractor;
pub mod formatter;
pub mod parser;

pub use extractor::{extract_all_conversations, ExtractOptions};
pub use formatter::{
    format_conversation_markdown, format_conversations_json, format_conversations_table,
    format_stats, OutputFormat,
};
