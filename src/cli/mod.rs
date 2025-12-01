//! CLI interface using clap.
//!
//! Provides command-line arguments and subcommands for the tool.

use clap::{Parser, Subcommand};

use crate::application::OutputFormat;

/// Cursor Chat Handler - Extract and display chat history from Cursor IDE.
///
/// 🤖 For AI use: cursor-chat list | show <id> --last 5 | export -c <id> -o file
#[derive(Parser, Debug)]
#[command(name = "cursor-chat-handler")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Enable verbose logging (use multiple times for more verbosity).
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Output format: markdown, json, or table.
    #[arg(short, long, default_value = "markdown")]
    pub format: String,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// List all conversations (summary table).
    List {
        /// Maximum number of conversations to show.
        #[arg(short, long, default_value = "20")]
        limit: usize,

        /// Minimum number of messages to include a conversation.
        #[arg(short, long, default_value = "1")]
        min_messages: usize,
    },

    /// Show a specific conversation in detail.
    Show {
        /// Conversation ID (full or partial).
        conversation_id: String,

        /// Include empty messages.
        #[arg(long)]
        include_empty: bool,

        /// Show only the last N messages.
        #[arg(short, long)]
        last: Option<usize>,
    },

    /// Export conversations to a file or stdout.
    Export {
        /// Output file path (stdout if not specified).
        #[arg(short, long)]
        output: Option<String>,

        /// Conversation ID to export (all if not specified).
        #[arg(short, long)]
        conversation: Option<String>,

        /// Minimum number of messages to include a conversation.
        #[arg(short, long, default_value = "1")]
        min_messages: usize,
    },

    /// Export multiple conversations to separate files with auto-generated names.
    ExportAll {
        /// Output directory for exported files.
        #[arg(short, long, default_value = "exports")]
        dir: String,

        /// Number of recent conversations to export (0 = all).
        #[arg(short, long, default_value = "0")]
        limit: usize,

        /// Minimum number of messages to include a conversation.
        #[arg(short, long, default_value = "1")]
        min_messages: usize,
    },

    /// Show statistics about stored conversations.
    Stats,

    /// Show database paths being used.
    Paths,

    /// Quick access menu - list conversations with numbers for fast selection.
    Quick {
        /// Number of recent conversations to show.
        #[arg(short, long, default_value = "5")]
        limit: usize,
    },

    /// Open conversation directly (shows last 10 messages by default).
    Open {
        /// Conversation ID or number from quick list.
        id: String,
    },
}

impl Cli {
    /// Parse the output format argument.
    pub fn output_format(&self) -> Result<OutputFormat, String> {
        self.format.parse()
    }
}
