//! Chat extraction service.
//!
//! Orchestrates reading from databases and building conversation structures.

use std::collections::HashMap;
use std::path::Path;

use chrono::DateTime;

use crate::domain::{AppError, Conversation, ExtractionStats, ModelConfig, Result};
use crate::infrastructure::{find_state_databases, StateDbReader};

use super::parser::{extract_composer_id, extract_conversation_id, parse_bubble, parse_composer};

/// Options for chat extraction.
#[derive(Debug, Clone, Default)]
pub struct ExtractOptions {
    /// Only extract from global database (skip workspace DBs).
    pub global_only: bool,
    /// Filter to specific conversation IDs.
    pub conversation_ids: Option<Vec<String>>,
    /// Minimum number of messages to include a conversation.
    pub min_messages: usize,
    /// Include conversations with empty text.
    pub include_empty: bool,
}

/// Extracts all conversations from Cursor databases.
///
/// # Errors
/// Returns error if databases cannot be read.
pub fn extract_all_conversations(
    options: &ExtractOptions,
) -> Result<(Vec<Conversation>, ExtractionStats)> {
    let databases = find_state_databases()?;
    let mut stats = ExtractionStats::default();

    // For conversations, we primarily use the global database
    let global_db = databases
        .iter()
        .find(|p| p.to_string_lossy().contains("globalStorage"))
        .ok_or_else(|| AppError::Config {
            message: "Global storage database not found".into(),
        })?;

    let conversations = extract_from_database(global_db, options, &mut stats)?;

    stats.databases_scanned = if options.global_only {
        1
    } else {
        databases.len()
    };

    Ok((conversations, stats))
}

/// Extracts conversations from a single database file.
fn extract_from_database(
    path: &Path,
    options: &ExtractOptions,
    stats: &mut ExtractionStats,
) -> Result<Vec<Conversation>> {
    tracing::info!("Extracting from: {}", path.display());

    let reader = StateDbReader::open(path)?;

    // Load composers (conversation metadata)
    let composer_entries = reader.fetch_composers()?;
    let mut composer_map: HashMap<String, Conversation> = HashMap::new();

    for entry in composer_entries {
        if let Some(id) = extract_composer_id(&entry.key) {
            if let Some(ref filter) = options.conversation_ids {
                if !matches_any_filter(id, filter) {
                    continue;
                }
            }

            match parse_composer(&entry.value) {
                Ok(raw) => {
                    let created_at = raw
                        .created_at
                        .and_then(|ms| DateTime::from_timestamp_millis(ms as i64));

                    let model_config =
                        raw.model_config
                            .map_or_else(ModelConfig::default, |m| ModelConfig {
                                model_name: m.model_name,
                                max_mode: m.max_mode,
                            });

                    composer_map.insert(
                        id.to_string(),
                        Conversation {
                            composer_id: id.to_string(),
                            title: String::new(), // Will be filled after bubbles load
                            created_at,
                            model_config,
                            unified_mode: raw.unified_mode.unwrap_or_default(),
                            bubbles: Vec::new(),
                        },
                    );
                }
                Err(e) => {
                    tracing::debug!("Failed to parse composer {}: {}", id, e);
                }
            }
        }
    }

    // Load bubbles and attach to conversations
    let bubble_entries = reader.fetch_bubbles()?;

    for entry in bubble_entries {
        if let Some(conv_id) = extract_conversation_id(&entry.key) {
            // Skip if filtering and this conversation isn't in the filter
            if let Some(ref filter) = options.conversation_ids {
                if !matches_any_filter(conv_id, filter) {
                    continue;
                }
            }

            match parse_bubble(&entry.value) {
                Ok(bubble) => {
                    // Skip empty messages unless requested
                    if !options.include_empty && bubble.text.trim().is_empty() {
                        continue;
                    }

                    stats.total_bubbles += 1;
                    match bubble.bubble_type {
                        crate::domain::BubbleType::User => stats.user_messages += 1,
                        crate::domain::BubbleType::Assistant => stats.assistant_messages += 1,
                        crate::domain::BubbleType::Unknown => {}
                    }

                    // Create conversation if it doesn't exist (orphan bubble)
                    let conversation =
                        composer_map
                            .entry(conv_id.to_string())
                            .or_insert_with(|| Conversation {
                                composer_id: conv_id.to_string(),
                                title: String::new(),
                                created_at: bubble.created_at,
                                model_config: ModelConfig::default(),
                                unified_mode: String::new(),
                                bubbles: Vec::new(),
                            });

                    conversation.bubbles.push(bubble);
                }
                Err(e) => {
                    tracing::debug!("Failed to parse bubble: {}", e);
                }
            }
        }
    }

    // Sort bubbles by creation time, generate titles, and filter conversations
    let mut conversations: Vec<Conversation> = composer_map
        .into_values()
        .filter(|c| c.bubbles.len() >= options.min_messages)
        .map(|mut c| {
            c.bubbles.sort_by(|a, b| a.created_at.cmp(&b.created_at));
            c.title = c.generate_title();
            c
        })
        .collect();

    // Sort conversations by creation time (newest first)
    conversations.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    stats.conversation_count = conversations.len();

    tracing::info!(
        "Extracted {} conversations with {} messages",
        stats.conversation_count,
        stats.total_bubbles
    );

    Ok(conversations)
}

/// Checks if an ID matches any of the filter patterns (partial match).
fn matches_any_filter(id: &str, filters: &[String]) -> bool {
    filters.iter().any(|f| id.starts_with(f) || id.contains(f))
}
