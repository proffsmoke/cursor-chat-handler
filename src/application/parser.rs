//! JSON parsing for Cursor chat data.
//!
//! Handles conversion from raw database values to domain models.

use chrono::{DateTime, Utc};
use serde::Deserialize;

use crate::domain::{AppError, Bubble, BubbleType, Result, ThinkingBlock, TokenCount};

/// Raw bubble data as stored in the database (JSON format).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawBubble {
    #[serde(rename = "_v")]
    _version: Option<u8>,
    #[serde(rename = "type", default)]
    bubble_type: u8,
    #[serde(rename = "bubbleId")]
    bubble_id: String,
    #[serde(default)]
    text: String,
    #[serde(default)]
    created_at: Option<String>,
    #[serde(default)]
    thinking: Option<RawThinking>,
    #[serde(default)]
    thinking_duration_ms: Option<u64>,
    #[serde(default)]
    token_count: Option<RawTokenCount>,
    #[serde(default)]
    is_agentic: bool,
}

#[derive(Debug, Deserialize, Default)]
struct RawThinking {
    #[serde(default)]
    text: String,
    #[serde(default)]
    signature: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
struct RawTokenCount {
    #[serde(default)]
    input_tokens: u64,
    #[serde(default)]
    output_tokens: u64,
}

/// Raw composer data as stored in the database.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RawComposer {
    #[serde(rename = "_v")]
    pub _version: Option<u8>,
    #[serde(default)]
    pub created_at: Option<u64>,
    #[serde(default)]
    pub model_config: Option<RawModelConfig>,
    #[serde(default)]
    pub unified_mode: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct RawModelConfig {
    #[serde(default)]
    pub model_name: String,
    #[serde(default)]
    pub max_mode: bool,
}

/// Parses a bubble from raw JSON bytes.
///
/// # Errors
/// Returns error if JSON parsing fails.
pub fn parse_bubble(data: &[u8]) -> Result<Bubble> {
    let raw: RawBubble = serde_json::from_slice(data).map_err(AppError::json_parse)?;

    let created_at = parse_datetime(&raw.created_at);
    let bubble_type = BubbleType::try_from(raw.bubble_type).unwrap_or(BubbleType::Unknown);

    let thinking = raw.thinking.map(|t| ThinkingBlock {
        text: t.text,
        signature: t.signature,
    });

    let token_count = raw
        .token_count
        .map_or_else(TokenCount::default, |t| TokenCount {
            input_tokens: t.input_tokens,
            output_tokens: t.output_tokens,
        });

    Ok(Bubble {
        bubble_id: raw.bubble_id,
        bubble_type,
        text: raw.text,
        created_at,
        thinking,
        thinking_duration_ms: raw.thinking_duration_ms,
        token_count,
        is_agentic: raw.is_agentic,
    })
}

/// Parses a composer from raw JSON bytes.
///
/// # Errors
/// Returns error if JSON parsing fails.
pub fn parse_composer(data: &[u8]) -> Result<RawComposer> {
    serde_json::from_slice(data).map_err(AppError::json_parse)
}

/// Parses datetime from various formats used by Cursor.
fn parse_datetime(value: &Option<String>) -> Option<DateTime<Utc>> {
    let s = value.as_ref()?;

    // Try ISO 8601 format first
    if let Ok(dt) = s.parse::<DateTime<Utc>>() {
        return Some(dt);
    }

    // Try parsing as milliseconds timestamp
    if let Ok(millis) = s.parse::<i64>() {
        return DateTime::from_timestamp_millis(millis);
    }

    None
}

/// Extracts conversation ID from a bubble key.
///
/// Key format: `bubbleId:{composer_id}:{bubble_id}`
pub fn extract_conversation_id(key: &str) -> Option<&str> {
    let stripped = key.strip_prefix("bubbleId:")?;
    stripped.split(':').next()
}

/// Extracts composer ID from a composer key.
///
/// Key format: `composerData:{composer_id}`
pub fn extract_composer_id(key: &str) -> Option<&str> {
    key.strip_prefix("composerData:")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_conversation_id() {
        let key = "bubbleId:abc-123:def-456";
        assert_eq!(extract_conversation_id(key), Some("abc-123"));
    }

    #[test]
    fn test_extract_composer_id() {
        let key = "composerData:abc-123";
        assert_eq!(extract_composer_id(key), Some("abc-123"));
    }

    #[test]
    fn test_parse_datetime_iso() {
        let dt = parse_datetime(&Some("2025-12-01T16:25:48.612Z".to_string()));
        assert!(dt.is_some());
    }

    #[test]
    fn test_parse_datetime_millis() {
        let dt = parse_datetime(&Some("1764561943374".to_string()));
        assert!(dt.is_some());
    }
}
