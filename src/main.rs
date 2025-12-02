//! Cursor Chat Handler - Extract and display chat history from Cursor IDE.
//!
//! This tool reads Cursor's `SQLite` databases to extract conversation history,
//! providing various output formats (Markdown, JSON, table) for review and export.
//!
//! 🤖 QUICK START FOR AI USE:
//!   cursor-chat quick                   # Interactive menu with numbers
//!   cursor-chat open 1                  # Open first chat directly
//!   cursor-chat list                    # See all available chats
//!   cursor-chat show <id> --last 5      # View last 5 messages from chat
//!   cursor-chat export -c <id> -o file  # Save chat for later reference
//!   cursor-chat sync start              # Start auto-sync daemon

mod application;
mod cli;
mod domain;
mod infrastructure;

use std::io::Write;
use std::time::Duration;

use clap::Parser;
use colored::Colorize;
use tracing_subscriber::{fmt, prelude::*, EnvFilter};

use application::{
    extract_all_conversations, format_conversation_markdown, format_conversations_json,
    format_conversations_table, format_stats, ExtractOptions, OutputFormat,
    RestoreService, StorageManager, SyncService,
};
use cli::{Cli, Commands, StorageCommands, SyncCommands};
use domain::AppConfig;
use infrastructure::{find_state_databases, SystemdService};

fn main() {
    let cli = Cli::parse();

    // Setup logging based on verbosity
    setup_logging(cli.verbose);

    if let Err(e) = run(cli) {
        eprintln!("{} {}", "Error:".red().bold(), e);
        std::process::exit(1);
    }
}

/// Main application logic.
fn run(cli: Cli) -> domain::Result<()> {
    let format = cli
        .output_format()
        .map_err(|e| domain::AppError::Config { message: e })?;

    match cli.command {
        Commands::List {
            limit,
            min_messages,
            workspace,
        } => {
            cmd_list(limit, min_messages, workspace.as_deref())?;
        }
        Commands::Show {
            conversation_id,
            include_empty,
            last,
        } => {
            cmd_show(&conversation_id, include_empty, last, format)?;
        }
        Commands::Export {
            output,
            conversation,
            min_messages,
        } => {
            cmd_export(
                output.as_deref(),
                conversation.as_deref(),
                min_messages,
                format,
            )?;
        }
        Commands::ExportAll {
            dir,
            limit,
            min_messages,
        } => {
            cmd_export_all(&dir, limit, min_messages, format)?;
        }
        Commands::Stats => {
            cmd_stats()?;
        }
        Commands::Paths => {
            cmd_paths()?;
        }
        Commands::Quick { limit } => {
            cmd_quick(limit)?;
        }
        Commands::Open { id } => {
            cmd_open(&id)?;
        }
        Commands::Sync(sync_cmd) => {
            cmd_sync(sync_cmd)?;
        }
        Commands::Storage(storage_cmd) => {
            cmd_storage(storage_cmd)?;
        }
        Commands::Daemon { interval } => {
            cmd_daemon(interval)?;
        }
        Commands::Restore { ids, force } => {
            cmd_restore(&ids, force)?;
        }
    }

    Ok(())
}

/// List conversations command.
fn cmd_list(limit: usize, min_messages: usize, _workspace: Option<&str>) -> domain::Result<()> {
    let options = ExtractOptions {
        min_messages,
        ..Default::default()
    };

    let (mut conversations, stats) = extract_all_conversations(&options)?;
    conversations.truncate(limit);

    println!("{}", format_conversations_table(&conversations));
    println!();
    println!("{}", format_stats(&stats));

    Ok(())
}

/// Show a single conversation.
fn cmd_show(
    id: &str,
    include_empty: bool,
    last: Option<usize>,
    format: OutputFormat,
) -> domain::Result<()> {
    let options = ExtractOptions {
        include_empty,
        ..Default::default()
    };

    let (conversations, _) = extract_all_conversations(&options)?;

    // Find conversation by ID (partial match)
    let conv = conversations
        .iter()
        .find(|c| c.composer_id.starts_with(id) || c.composer_id.contains(id))
        .ok_or_else(|| domain::AppError::InvalidData {
            message: format!("Conversation not found: {id}"),
        })?;

    // Apply --last filter if specified
    let conv = if let Some(n) = last {
        let mut filtered = conv.clone();
        let len = filtered.bubbles.len();
        if n < len {
            filtered.bubbles = filtered.bubbles.into_iter().skip(len - n).collect();
        }
        filtered
    } else {
        conv.clone()
    };

    let output = match format {
        OutputFormat::Markdown => format_conversation_markdown(&conv),
        OutputFormat::Json => format_conversations_json(std::slice::from_ref(&conv))
            .map_err(domain::AppError::json_parse)?,
        OutputFormat::Table => format_conversations_table(std::slice::from_ref(&conv)),
    };

    println!("{output}");
    Ok(())
}

/// Export conversations to file or stdout.
fn cmd_export(
    output_path: Option<&str>,
    conversation_id: Option<&str>,
    min_messages: usize,
    format: OutputFormat,
) -> domain::Result<()> {
    let options = ExtractOptions {
        min_messages,
        conversation_ids: conversation_id.map(|id| vec![id.to_string()]),
        ..Default::default()
    };

    let (conversations, stats) = extract_all_conversations(&options)?;

    let content = match format {
        OutputFormat::Markdown => {
            let mut out = String::new();
            for conv in &conversations {
                out.push_str(&format_conversation_markdown(conv));
                out.push_str("\n\n");
            }
            out
        }
        OutputFormat::Json => {
            format_conversations_json(&conversations).map_err(domain::AppError::json_parse)?
        }
        OutputFormat::Table => format_conversations_table(&conversations),
    };

    match output_path {
        Some(path) => {
            let mut file = std::fs::File::create(path)
                .map_err(|e| domain::AppError::io(format!("Failed to create {path}"), e))?;
            file.write_all(content.as_bytes())
                .map_err(|e| domain::AppError::io("Failed to write file", e))?;
            println!(
                "{} Exported {} conversations to {}",
                "✓".green().bold(),
                stats.conversation_count,
                path
            );
        }
        None => {
            println!("{content}");
        }
    }

    Ok(())
}

/// Export all conversations to separate files with auto-generated names.
fn cmd_export_all(
    dir: &str,
    limit: usize,
    min_messages: usize,
    format: OutputFormat,
) -> domain::Result<()> {
    let options = ExtractOptions {
        min_messages,
        ..Default::default()
    };

    let (mut conversations, _) = extract_all_conversations(&options)?;

    if limit > 0 {
        conversations.truncate(limit);
    }

    // Create output directory
    std::fs::create_dir_all(dir)
        .map_err(|e| domain::AppError::io(format!("Failed to create directory {dir}"), e))?;

    let ext = match format {
        OutputFormat::Markdown => "md",
        OutputFormat::Json => "json",
        OutputFormat::Table => "txt",
    };

    for conv in &conversations {
        let filename = format!("{}/{}.{}", dir, conv.filename(), ext);

        let content = match format {
            OutputFormat::Markdown => format_conversation_markdown(conv),
            OutputFormat::Json => format_conversations_json(std::slice::from_ref(conv))
                .map_err(domain::AppError::json_parse)?,
            OutputFormat::Table => format_conversations_table(std::slice::from_ref(conv)),
        };

        let mut file = std::fs::File::create(&filename)
            .map_err(|e| domain::AppError::io(format!("Failed to create {filename}"), e))?;
        file.write_all(content.as_bytes())
            .map_err(|e| domain::AppError::io("Failed to write file", e))?;

        println!("{} {} → {}", "✓".green(), conv.title.cyan(), filename);
    }

    println!(
        "\n{} Exported {} conversations to {}/",
        "📁".bold(),
        conversations.len(),
        dir
    );

    Ok(())
}

/// Show statistics command.
fn cmd_stats() -> domain::Result<()> {
    let options = ExtractOptions {
        include_empty: true,
        ..Default::default()
    };

    let (_, stats) = extract_all_conversations(&options)?;
    println!("{}", format_stats(&stats));

    Ok(())
}

/// Show database paths command.
fn cmd_paths() -> domain::Result<()> {
    let databases = find_state_databases()?;

    println!("{}", "📂 Cursor Database Paths".bold());
    println!();

    for (i, path) in databases.iter().enumerate() {
        let label = if path.to_string_lossy().contains("globalStorage") {
            "global".green()
        } else {
            "workspace".blue()
        };

        println!("  {}. [{}] {}", i + 1, label, path.display());
    }

    println!();
    println!("Total: {} database(s)", databases.len());

    Ok(())
}

/// Quick access menu command.
fn cmd_quick(limit: usize) -> domain::Result<()> {
    let options = ExtractOptions {
        min_messages: 1,
        ..Default::default()
    };

    let (mut conversations, _) = extract_all_conversations(&options)?;
    conversations.truncate(limit);

    println!("🚀 Quick Access Menu");
    println!("==================");
    println!();

    for (i, conv) in conversations.iter().enumerate() {
        let model = if conv.model_config.model_name.is_empty() {
            "unknown".to_string()
        } else {
            conv.model_config.model_name[..15.min(conv.model_config.model_name.len())].to_string()
        };

        let title = if conv.title.is_empty() {
            "Untitled".to_string()
        } else {
            conv.title[..40.min(conv.title.len())].to_string()
        };

        println!(
            "  {:2}. {} | {} | {} msgs | {}",
            i + 1,
            &conv.composer_id[..8],
            model,
            conv.message_count(),
            title
        );
    }

    println!();
    println!("💡 Quick commands:");
    println!(
        "   cursor-chat open {}        # Open conversation by number",
        conversations.first().map(|_| "1").unwrap_or("N")
    );
    println!(
        "   cursor-chat show {}         # Show full conversation",
        conversations
            .first()
            .map(|c| &c.composer_id[..8])
            .unwrap_or("ID")
    );
    println!(
        "   cursor-chat export -c {}    # Export conversation",
        conversations
            .first()
            .map(|c| &c.composer_id[..8])
            .unwrap_or("ID")
    );

    Ok(())
}

/// Open conversation directly command.
fn cmd_open(id_or_number: &str) -> domain::Result<()> {
    let options = ExtractOptions {
        min_messages: 1,
        ..Default::default()
    };

    let (conversations, _) = extract_all_conversations(&options)?;

    // Try to parse as number first (1-based index)
    let conv = if let Ok(number) = id_or_number.parse::<usize>() {
        if number == 0 || number > conversations.len() {
            return Err(domain::AppError::InvalidData {
                message: format!(
                    "Number {} is out of range (1-{})",
                    number,
                    conversations.len()
                ),
            });
        }
        &conversations[number - 1]
    } else {
        // Try as ID (partial match)
        conversations
            .iter()
            .find(|c| {
                c.composer_id.starts_with(id_or_number) || c.composer_id.contains(id_or_number)
            })
            .ok_or_else(|| domain::AppError::InvalidData {
                message: format!("Conversation '{}' not found", id_or_number),
            })?
    };

    // Show the conversation with last 10 messages
    cmd_show(&conv.composer_id, false, Some(10), OutputFormat::Markdown)?;

    println!();
    println!("💡 Pro tips:");
    println!(
        "   cursor-chat export -c {} -o current.md    # Save this chat",
        &conv.composer_id[..8]
    );
    println!(
        "   cursor-chat show {} --last 20            # See more messages",
        &conv.composer_id[..8]
    );

    Ok(())
}

/// Handle sync subcommands.
fn cmd_sync(cmd: SyncCommands) -> domain::Result<()> {
    let config = AppConfig::default();
    let systemd = SystemdService::new(config.clone());

    match cmd {
        SyncCommands::Start => {
            println!("{}", "🚀 Installing sync service...".bold());

            // Ensure directories exist
            let storage_mgr = StorageManager::new(config.clone());
            storage_mgr.ensure_directories()?;

            // Install and start service
            let result = systemd.install()?;
            println!("  {} Service file: {}", "✓".green(), result.service_path.display());

            systemd.enable_and_start()?;
            println!("  {} Service enabled and started", "✓".green());

            println!();
            println!("Sync daemon is now running! Check status with:");
            println!("  cursor-chat sync status");
        }
        SyncCommands::Stop => {
            println!("{}", "⏹️  Stopping sync service...".bold());
            systemd.stop_and_disable()?;
            println!("  {} Service stopped and disabled", "✓".green());
        }
        SyncCommands::Status => {
            let status = systemd.get_status()?;

            println!("{}", "📊 Sync Service Status".bold());
            println!();
            println!("  Installed: {}", if status.is_installed { "Yes".green() } else { "No".red() });
            println!("  Enabled:   {}", if status.is_enabled { "Yes".green() } else { "No".yellow() });
            println!("  Running:   {}", if status.is_running { "Yes".green() } else { "No".red() });
            println!();

            if status.is_installed {
                // Show sync state
                let sync_service = SyncService::new(config)?;
                let state = sync_service.get_state()?;

                println!("  Last sync:      {}", state.last_sync
                    .map(|dt| dt.format("%Y-%m-%d %H:%M:%S").to_string())
                    .unwrap_or_else(|| "Never".to_string()));
                println!("  Conversations:  {}", state.conversation_count);
                println!("  Messages:       {}", state.message_count);

                if let Some(err) = &state.last_error {
                    println!("  Last error:     {}", err.red());
                }
            }
        }
        SyncCommands::Now => {
            println!("{}", "🔄 Running sync...".bold());

            let sync_service = SyncService::new(config)?;
            let state = sync_service.sync()?;

            println!();
            println!("  {} Sync completed!", "✓".green());
            println!("  Conversations: {}", state.conversation_count);
            println!("  Messages:      {}", state.message_count);
        }
        SyncCommands::Logs { lines } => {
            let logs = systemd.view_logs(lines)?;
            println!("{}", logs);
        }
        SyncCommands::Uninstall => {
            println!("{}", "🗑️  Uninstalling sync service...".bold());
            systemd.uninstall()?;
            println!("  {} Service uninstalled", "✓".green());
        }
        SyncCommands::Restore { ids, force } => {
            use application::RestoreService;

            println!("{}", "🔄 Restoring chats to Cursor...".bold());

            let restore_service = RestoreService::new(config);

            // Check if restore is needed
            if !force {
                if !restore_service.needs_restore()? {
                    println!("  {} Cursor database looks fine, no restore needed", "ℹ️".blue());
                    println!("  Use --force to restore anyway");
                    return Ok(());
                }
            }

            // Perform restore
            let result = if ids.is_empty() {
                restore_service.restore_all()?
            } else {
                restore_service.restore_by_ids(&ids)?
            };

            println!();
            println!("  {} Restore completed!", "✓".green());
            println!("  Conversations: {}", result.restored_conversations);
            println!("  Messages:      {}", result.restored_messages);
            println!();
            println!("  Database: {}", result.cursor_db_path.display());
            println!();
            println!("  {} Restart Cursor to see restored chats", "💡".bold());
        }
    }

    Ok(())
}

/// Handle storage subcommands.
fn cmd_storage(cmd: StorageCommands) -> domain::Result<()> {
    let config = AppConfig::default();
    let storage_mgr = StorageManager::new(config.clone());

    match cmd {
        StorageCommands::Stats => {
            storage_mgr.ensure_directories()?;
            let summary = storage_mgr.get_summary()?;

            println!("{}", "💾 Storage Statistics".bold());
            println!();
            println!("  Total used:     {} / {} ({:.1}%)",
                summary.total_human(),
                summary.max_human(),
                summary.usage_percent);
            println!();
            println!("  Database:       {}", summary.db_human());
            println!("  Exports:        {}", summary.exports_human());
            println!("  Backups:        {} ({} files)", summary.backups_human(), summary.backup_count);
            println!();
            println!("  Data directory: {}", config.data_dir().display());
        }
        StorageCommands::Cleanup => {
            println!("{}", "🧹 Running cleanup...".bold());

            let result = storage_mgr.enforce_storage_limit()?;

            if result.deleted_count > 0 {
                println!("  {} Deleted {} files, freed {}",
                    "✓".green(),
                    result.deleted_count,
                    result.freed_human());
            } else {
                println!("  {} Nothing to clean up", "✓".green());
            }
        }
        StorageCommands::Workspaces => {
            let sync_service = SyncService::new(config)?;
            let workspaces = sync_service.get_workspaces()?;

            println!("{}", "📁 Workspaces".bold());
            println!();

            if workspaces.is_empty() {
                println!("  No workspaces found. Run 'cursor-chat sync now' first.");
            } else {
                for ws in &workspaces {
                    let path = ws.path.as_ref()
                        .map(|p| p.display().to_string())
                        .unwrap_or_else(|| "(unknown)".to_string());
                    println!("  {} → {}", ws.name.cyan(), path);
                }
            }
        }
        StorageCommands::Config => {
            println!("{}", "⚙️  Storage Configuration".bold());
            println!();
            println!("  Max storage:       {} GB", config.storage.max_size_gb);
            println!("  Backup retention:  {} days", config.storage.backup_retention_days);
            println!("  Sync interval:     {} seconds", config.sync.interval_secs);
            println!("  Sync enabled:      {}", config.sync.enabled);
            println!();
            println!("  Data directory:    {}", config.data_dir().display());
            println!("  Config file:       {}", config.config_file_path().display());
        }
    }

    Ok(())
}

/// Restore chat history to Cursor.
fn cmd_restore(ids: &[String], force: bool) -> domain::Result<()> {
    let config = AppConfig::default();
    let restore_service = RestoreService::new(config);

    println!("{}", "🔄 Checking restore status...".bold());

    // Check if restore is needed
    let cursor_empty = restore_service.cursor_is_empty()?;
    let needs_restore = restore_service.needs_restore()?;

    if !cursor_empty && !needs_restore && !force {
        println!();
        println!("  {} Cursor has chats - restore not needed", "ℹ".blue());
        println!();
        println!("  Use --force to restore anyway");
        return Ok(());
    }

    if cursor_empty {
        println!("  {} Cursor database is empty", "!".yellow());
    } else if needs_restore {
        println!("  {} Cursor appears to have been reset", "!".yellow());
    }

    println!();
    println!("{}", "📥 Restoring chats from backup...".bold());

    let result = if ids.is_empty() {
        restore_service.restore_all()?
    } else {
        restore_service.restore_by_ids(ids)?
    };

    println!();
    println!("  {} Restore completed!", "✓".green());
    println!("  Conversations: {}", result.restored_conversations);
    println!("  Messages:      {}", result.restored_messages);
    println!();
    println!("  Cursor DB: {}", result.cursor_db_path.display());
    println!();
    println!("{}", "💡 Reinicie o Cursor para ver os chats restaurados".cyan());

    Ok(())
}

/// Run as daemon (background sync service).
fn cmd_daemon(interval_secs: u64) -> domain::Result<()> {
    let config = AppConfig::default();

    println!("{}", "🔄 Starting sync daemon...".bold());
    println!("  Interval: {} seconds", interval_secs);
    println!("  Data dir: {}", config.data_dir().display());
    println!("  Auto-restore: enabled");
    println!();

    // Ensure directories exist
    let storage_mgr = StorageManager::new(config.clone());
    storage_mgr.ensure_directories()?;

    let sync_service = SyncService::new(config.clone())?;
    let restore_service = RestoreService::new(config);

    loop {
        tracing::info!("Starting sync cycle...");

        // First, check if restore is needed (Cursor was cleared)
        match restore_service.auto_restore_if_needed() {
            Ok(true) => {
                tracing::info!("Auto-restore completed");
            }
            Ok(false) => {}
            Err(e) => {
                tracing::warn!(error = %e, "Auto-restore check failed");
            }
        }

        // Then sync from Cursor to local storage
        match sync_service.sync() {
            Ok(state) => {
                tracing::info!(
                    conversations = state.conversation_count,
                    messages = state.message_count,
                    "Sync completed successfully"
                );
            }
            Err(e) => {
                tracing::error!(error = %e, "Sync failed");
            }
        }

        // Check and enforce storage limits
        if let Err(e) = storage_mgr.enforce_storage_limit() {
            tracing::warn!(error = %e, "Failed to enforce storage limits");
        }

        // Sleep until next sync
        std::thread::sleep(Duration::from_secs(interval_secs));
    }
}

/// Setup tracing/logging based on verbosity level.
fn setup_logging(verbosity: u8) {
    let filter = match verbosity {
        0 => "warn",
        1 => "info",
        2 => "debug",
        _ => "trace",
    };

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(filter));

    tracing_subscriber::registry()
        .with(fmt::layer().with_target(false).without_time())
        .with(filter)
        .init();
}
