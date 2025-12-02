//! CLI interface using clap.
//!
//! Provides command-line arguments and subcommands for the tool.

use clap::{Parser, Subcommand};

use crate::application::OutputFormat;

const LONG_ABOUT: &str = r#"
🤖 GUIA RÁPIDO PARA IA
══════════════════════════════════════════════════════════════

📋 RECUPERAR CONTEXTO DE CHAT ANTERIOR:
  cursor-chat quick              # Menu com números
  cursor-chat open 1             # Abrir última conversa
  cursor-chat show <ID> --last 10  # Ver últimas 10 msgs

💾 SALVAR CONTEXTO ATUAL:
  cursor-chat export -c <ID> -o chat.md
  cursor-chat export-all --limit 3

🔄 AUTO-SYNC + AUTO-RESTORE:
  cursor-chat sync start         # Iniciar daemon (salva a cada 2min)
  cursor-chat sync restore       # Restaurar chats após limpar Cursor
  cursor-chat sync status        # Ver status

📁 VER POR PROJETO/WORKSPACE:
  cursor-chat storage workspaces  # Listar projetos

⚙️ STORAGE LOCAL (~/.cursor-chat-handler/):
  cursor-chat storage stats      # Ver uso (limite 10GB)

🔥 LIMPOU O CURSOR? Use: cursor-chat sync restore
   Detecta e restaura automaticamente quando Cursor resetar!

══════════════════════════════════════════════════════════════
"#;

/// Cursor Chat Handler - Extract and display chat history from Cursor IDE.
#[derive(Parser, Debug)]
#[command(name = "cursor-chat-handler")]
#[command(author, version, about, long_about = LONG_ABOUT)]
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

        /// Filter by workspace/project name.
        #[arg(short, long)]
        workspace: Option<String>,
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

    /// Sync management commands.
    #[command(subcommand)]
    Sync(SyncCommands),

    /// Storage management commands.
    #[command(subcommand)]
    Storage(StorageCommands),

    /// Run as daemon (used by systemd service).
    Daemon {
        /// Sync interval in seconds.
        #[arg(short, long, default_value = "120")]
        interval: u64,
    },

    /// Restore chat history to Cursor after clearing/reset.
    Restore {
        /// Restore specific conversation IDs only.
        #[arg(short, long)]
        ids: Vec<String>,

        /// Force restore even if Cursor has chats.
        #[arg(long)]
        force: bool,
    },
}

/// Sync subcommands.
#[derive(Subcommand, Debug)]
pub enum SyncCommands {
    /// Install and start the sync service.
    Start,

    /// Stop and disable the sync service.
    Stop,

    /// Show sync service status.
    Status,

    /// Run a sync immediately.
    Now,

    /// Show sync logs.
    Logs {
        /// Number of log lines to show.
        #[arg(short, long, default_value = "50")]
        lines: usize,
    },

    /// Uninstall the sync service.
    Uninstall,

    /// Restore chats to Cursor (when reset detected).
    Restore {
        /// Restore specific conversation IDs (all if not specified).
        #[arg(short, long)]
        ids: Vec<String>,

        /// Force restore even if not needed.
        #[arg(short, long)]
        force: bool,
    },
}

/// Storage subcommands.
#[derive(Subcommand, Debug)]
pub enum StorageCommands {
    /// Show storage usage statistics.
    Stats,

    /// Clean up old backups and enforce storage limits.
    Cleanup,

    /// List all workspaces/projects.
    Workspaces,

    /// Show storage configuration.
    Config,
}

impl Cli {
    /// Parse the output format argument.
    pub fn output_format(&self) -> Result<OutputFormat, String> {
        self.format.parse()
    }
}
