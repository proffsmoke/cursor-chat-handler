//! Domain models for Cursor chat data.
//!
//! These models represent the core entities extracted from Cursor's `SQLite` database.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Type of message in a chat bubble.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub enum BubbleType {
    /// Message from the user (human).
    User = 1,
    /// Message from the AI assistant.
    Assistant = 2,
    /// Unknown or other type.
    Unknown = 0,
}

impl From<BubbleType> for u8 {
    fn from(bt: BubbleType) -> Self {
        bt as Self
    }
}

impl TryFrom<u8> for BubbleType {
    type Error = String;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::User),
            2 => Ok(Self::Assistant),
            _ => Ok(Self::Unknown),
        }
    }
}

impl std::fmt::Display for BubbleType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::User => write!(f, "User"),
            Self::Assistant => write!(f, "Assistant"),
            Self::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Thinking block from AI responses.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThinkingBlock {
    /// The thinking text content.
    pub text: String,
    /// Optional signature/verification.
    #[serde(default)]
    pub signature: Option<String>,
}

/// Token usage information.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct TokenCount {
    /// Number of input tokens.
    #[serde(default)]
    pub input_tokens: u64,
    /// Number of output tokens.
    #[serde(default)]
    pub output_tokens: u64,
}

/// A single chat message (bubble) in a conversation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bubble {
    /// Unique identifier for this bubble.
    pub bubble_id: String,
    /// Type of message (user or assistant).
    pub bubble_type: BubbleType,
    /// The actual message text content.
    pub text: String,
    /// When this message was created.
    pub created_at: Option<DateTime<Utc>>,
    /// Thinking block (for AI responses).
    #[serde(default)]
    pub thinking: Option<ThinkingBlock>,
    /// Duration of thinking in milliseconds.
    #[serde(default)]
    pub thinking_duration_ms: Option<u64>,
    /// Token usage for this message.
    #[serde(default)]
    pub token_count: TokenCount,
    /// Whether this is an agentic response.
    #[serde(default)]
    pub is_agentic: bool,
}

/// Model configuration used for a conversation.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ModelConfig {
    /// Name of the model used.
    #[serde(default)]
    pub model_name: String,
    /// Whether max mode was enabled.
    #[serde(default)]
    pub max_mode: bool,
}

/// Metadata for a conversation (composer).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    /// Unique identifier for this conversation.
    pub composer_id: String,
    /// Auto-generated title from first user message.
    #[serde(default)]
    pub title: String,
    /// When this conversation was created.
    pub created_at: Option<DateTime<Utc>>,
    /// Model configuration used.
    #[serde(default)]
    pub model_config: ModelConfig,
    /// Unified mode (agent, edit, etc.).
    #[serde(default)]
    pub unified_mode: String,
    /// All bubbles in this conversation (sorted by time).
    #[serde(default)]
    pub bubbles: Vec<Bubble>,
}

impl Conversation {
    /// Get the first message text as a preview/title.
    #[must_use]
    pub fn preview(&self) -> &str {
        self.bubbles
            .first()
            .map_or("[Empty conversation]", |b| b.text.as_str())
    }

    /// Generate a clean title from the first user message.
    #[must_use]
    pub fn generate_title(&self) -> String {
        // Find first user message
        let first_user_msg = self
            .bubbles
            .iter()
            .find(|b| b.bubble_type == BubbleType::User)
            .map(|b| b.text.as_str())
            .unwrap_or("conversa");

        // Clean and extract meaningful words
        let cleaned: String = first_user_msg
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace() || *c == '-' || *c == '_')
            .collect();

        // Take first ~50 chars, trim at word boundary
        let truncated = if cleaned.len() > 50 {
            let cut = &cleaned[..50];
            cut.rfind(' ').map_or(cut, |i| &cut[..i])
        } else {
            &cleaned
        };

        // Convert to snake_case filename
        truncated
            .split_whitespace()
            .take(8) // max 8 words
            .collect::<Vec<_>>()
            .join("_")
            .to_lowercase()
    }

    /// Get a safe filename based on title.
    #[must_use]
    pub fn filename(&self) -> String {
        let title = self.generate_title();
        let short_id = &self.composer_id[..8.min(self.composer_id.len())];
        format!("{short_id}_{title}")
    }

    /// Get total message count.
    #[must_use]
    pub const fn message_count(&self) -> usize {
        self.bubbles.len()
    }

    /// Get user message count.
    #[must_use]
    pub fn user_message_count(&self) -> usize {
        self.bubbles
            .iter()
            .filter(|b| b.bubble_type == BubbleType::User)
            .count()
    }

    /// Get assistant message count.
    #[must_use]
    pub fn assistant_message_count(&self) -> usize {
        self.bubbles
            .iter()
            .filter(|b| b.bubble_type == BubbleType::Assistant)
            .count()
    }
}

/// Summary statistics for extracted chats.
#[derive(Debug, Clone, Default, Serialize)]
pub struct ExtractionStats {
    /// Number of conversations found.
    pub conversation_count: usize,
    /// Total number of bubbles/messages.
    pub total_bubbles: usize,
    /// Total user messages.
    pub user_messages: usize,
    /// Total assistant messages.
    pub assistant_messages: usize,
    /// Database files scanned.
    pub databases_scanned: usize,
}
