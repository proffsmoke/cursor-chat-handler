//! Output formatting for extracted chat data.
//!
//! Supports multiple output formats: Markdown, JSON, and table view.

use colored::Colorize;
use comfy_table::{presets::UTF8_FULL, Table};

use crate::domain::{BubbleType, Conversation, ExtractionStats};

/// Output format options.
#[derive(Debug, Clone, Copy, Default)]
pub enum OutputFormat {
    /// Human-readable Markdown format.
    #[default]
    Markdown,
    /// JSON format for programmatic use.
    Json,
    /// Compact table listing.
    Table,
}

impl std::str::FromStr for OutputFormat {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "markdown" | "md" => Ok(Self::Markdown),
            "json" => Ok(Self::Json),
            "table" => Ok(Self::Table),
            _ => Err(format!("Unknown format: {s}. Use: markdown, json, table")),
        }
    }
}

/// Formats a single conversation as Markdown.
pub fn format_conversation_markdown(conv: &Conversation) -> String {
    let mut out = String::new();

    // Header with title
    let title = if conv.title.is_empty() {
        conv.composer_id.clone()
    } else {
        format!("{} ({})", conv.title, &conv.composer_id[..8])
    };
    out.push_str(&format!("# {title}\n\n"));

    if let Some(dt) = conv.created_at {
        out.push_str(&format!(
            "**Created:** {}\n",
            dt.format("%Y-%m-%d %H:%M:%S UTC")
        ));
    }

    if !conv.model_config.model_name.is_empty() {
        out.push_str(&format!("**Model:** {}\n", conv.model_config.model_name));
    }

    if !conv.unified_mode.is_empty() {
        out.push_str(&format!("**Mode:** {}\n", conv.unified_mode));
    }

    out.push_str(&format!(
        "**Messages:** {} ({} user, {} assistant)\n\n",
        conv.message_count(),
        conv.user_message_count(),
        conv.assistant_message_count()
    ));

    out.push_str("---\n\n");

    // Messages
    for bubble in &conv.bubbles {
        let role = match bubble.bubble_type {
            BubbleType::User => "👤 **User**",
            BubbleType::Assistant => "🤖 **Assistant**",
            BubbleType::Unknown => "❓ **Unknown**",
        };

        out.push_str(&format!("### {role}\n\n"));

        if let Some(dt) = bubble.created_at {
            out.push_str(&format!("*{}*\n\n", dt.format("%H:%M:%S")));
        }

        // Thinking block (if present)
        if let Some(ref thinking) = bubble.thinking {
            if !thinking.text.is_empty() {
                out.push_str("<details>\n<summary>💭 Thinking</summary>\n\n");
                out.push_str(&thinking.text);
                out.push_str("\n\n</details>\n\n");
            }
        }

        // Message content
        out.push_str(&bubble.text);
        out.push_str("\n\n");

        // Token info
        if bubble.token_count.input_tokens > 0 || bubble.token_count.output_tokens > 0 {
            out.push_str(&format!(
                "*Tokens: {} in / {} out*\n\n",
                bubble.token_count.input_tokens, bubble.token_count.output_tokens
            ));
        }

        out.push_str("---\n\n");
    }

    out
}

/// Formats multiple conversations as JSON.
///
/// # Errors
/// Returns error if serialization fails.
pub fn format_conversations_json(
    conversations: &[Conversation],
) -> Result<String, serde_json::Error> {
    serde_json::to_string_pretty(conversations)
}

/// Formats a table listing of conversations.
pub fn format_conversations_table(conversations: &[Conversation]) -> String {
    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec!["ID", "Created", "Model", "Msgs", "Title"]);

    for conv in conversations {
        let created = conv.created_at.map_or_else(
            || "-".to_string(),
            |dt| dt.format("%Y-%m-%d %H:%M").to_string(),
        );

        let model = if conv.model_config.model_name.is_empty() {
            "-".to_string()
        } else {
            truncate(&conv.model_config.model_name, 18)
        };

        let title = if conv.title.is_empty() {
            truncate(conv.preview(), 35)
        } else {
            truncate(&conv.title, 35)
        };

        table.add_row(vec![
            &conv.composer_id[..8],
            &created,
            &model,
            &conv.message_count().to_string(),
            &title,
        ]);
    }

    table.to_string()
}

/// Formats extraction statistics for display.
pub fn format_stats(stats: &ExtractionStats) -> String {
    format!(
        "{}\n  Conversations: {}\n  Total messages: {}\n  User messages: {}\n  Assistant messages: {}\n  Databases scanned: {}",
        "📊 Statistics".bold(),
        stats.conversation_count.to_string().cyan(),
        stats.total_bubbles.to_string().cyan(),
        stats.user_messages.to_string().green(),
        stats.assistant_messages.to_string().blue(),
        stats.databases_scanned.to_string().yellow()
    )
}

/// Truncates a string to max length with ellipsis.
fn truncate(s: &str, max_len: usize) -> String {
    let s = s.lines().next().unwrap_or(s);
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate() {
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("hello world!", 8), "hello...");
    }

    #[test]
    fn test_output_format_from_str() {
        assert!(matches!(
            "markdown".parse::<OutputFormat>(),
            Ok(OutputFormat::Markdown)
        ));
        assert!(matches!(
            "json".parse::<OutputFormat>(),
            Ok(OutputFormat::Json)
        ));
        assert!(matches!(
            "table".parse::<OutputFormat>(),
            Ok(OutputFormat::Table)
        ));
        assert!("invalid".parse::<OutputFormat>().is_err());
    }
}
