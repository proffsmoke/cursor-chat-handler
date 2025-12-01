//! Domain layer - core business logic and types.
//!
//! This layer contains pure domain models and error types
//! without any external dependencies (DB, IO, etc.).

pub mod error;
pub mod models;

pub use error::{AppError, Result};
pub use models::{
    Bubble, BubbleType, Conversation, ExtractionStats, ModelConfig, ThinkingBlock, TokenCount,
};
