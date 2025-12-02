//! Synchronization service for chat data.
//!
//! Handles incremental sync from Cursor's database to local storage,
//! with change detection and workspace extraction.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use chrono::Utc;

use crate::domain::{AppConfig, Conversation, Result, SyncState, WorkspaceInfo};
use crate::infrastructure::{find_state_databases, LocalStorage, StateDbReader};

use super::parser::{extract_composer_id, extract_conversation_id, parse_bubble, parse_composer};
use super::restore_service::RestoreService;

/// Service for synchronizing chat data from Cursor to local storage.
pub struct SyncService {
    config: AppConfig,
    storage: LocalStorage,
}

impl SyncService {
    /// Create a new sync service with the given configuration.
    ///
    /// # Errors
    /// Returns error if local storage cannot be opened.
    pub fn new(config: AppConfig) -> Result<Self> {
        let storage_path = config.storage_db_path();
        let storage = LocalStorage::open(&storage_path)?;

        Ok(Self { config, storage })
    }

    /// Create with an existing storage instance.
    #[must_use]
    pub const fn with_storage(config: AppConfig, storage: LocalStorage) -> Self {
        Self { config, storage }
    }

    /// Perform a full sync from Cursor database to local storage.
    ///
    /// # Errors
    /// Returns error if sync fails.
    pub fn sync(&self) -> Result<SyncState> {
        tracing::info!("Starting sync...");

        // Mark sync as in progress
        let mut state = self.storage.get_sync_state()?.syncing();
        self.storage.update_sync_state(&state)?;

        // Find and read Cursor databases
        let databases = find_state_databases()?;
        let global_db = databases
            .iter()
            .find(|p| p.to_string_lossy().contains("globalStorage"))
            .ok_or_else(|| crate::domain::AppError::Config {
                message: "Global storage database not found".into(),
            })?;

        // Extract conversations
        let start = std::time::Instant::now();
        let (conversations, workspace_map) = self.extract_conversations(global_db)?;

        let mut synced_count = 0;
        let mut message_count = 0;

        for conv in &conversations {
            // Calculate content hash for change detection
            let content_hash = self.calculate_hash(conv);

            // Check if conversation changed
            let existing_hash = self.storage.get_conversation_hash(&conv.composer_id)?;
            if existing_hash.as_ref() == Some(&content_hash) {
                tracing::debug!("Skipping unchanged conversation: {}", &conv.composer_id[..8]);
                continue;
            }

            // Get or create workspace
            let workspace_id = workspace_map
                .get(&conv.composer_id)
                .map(|ws| self.storage.get_or_create_workspace(ws))
                .transpose()?;

            // Upsert conversation
            let conv_id = self
                .storage
                .upsert_conversation(conv, workspace_id, Some(&content_hash))?;

            // Upsert bubbles
            for bubble in &conv.bubbles {
                self.storage.upsert_bubble(bubble, conv_id)?;
            }

            synced_count += 1;
            message_count += conv.bubbles.len();
        }

        let elapsed = start.elapsed();
        tracing::info!(
            synced = synced_count,
            messages = message_count,
            duration_ms = elapsed.as_millis(),
            "Sync completed"
        );

        // Update sync state
        state = SyncState {
            last_sync: Some(Utc::now()),
            last_hash: None, // Could calculate global hash here
            conversation_count: self.storage.get_conversation_count()?,
            message_count: self.storage.get_message_count()?,
            storage_bytes: self.storage.get_storage_size()?,
            is_syncing: false,
            last_error: None,
        };

        self.storage.update_sync_state(&state)?;

        Ok(state)
    }

    /// Extract conversations from a Cursor database.
    fn extract_conversations(
        &self,
        db_path: &std::path::Path,
    ) -> Result<(Vec<Conversation>, std::collections::HashMap<String, WorkspaceInfo>)> {
        use std::collections::HashMap;

        let reader = StateDbReader::open(db_path)?;
        let mut conversations: HashMap<String, Conversation> = HashMap::new();
        let mut workspace_map: HashMap<String, WorkspaceInfo> = HashMap::new();

        // Load composers
        for entry in reader.fetch_composers()? {
            if let Some(id) = extract_composer_id(&entry.key) {
                if let Ok(raw) = parse_composer(&entry.value) {
                    let created_at = raw.created_at.and_then(|ms| {
                        chrono::DateTime::from_timestamp_millis(ms as i64)
                    });

                    let model_config = raw.model_config.map_or_else(
                        crate::domain::ModelConfig::default,
                        |m| crate::domain::ModelConfig {
                            model_name: m.model_name,
                            max_mode: m.max_mode,
                        },
                    );

                    conversations.insert(
                        id.to_string(),
                        Conversation {
                            composer_id: id.to_string(),
                            title: String::new(),
                            created_at,
                            model_config,
                            unified_mode: raw.unified_mode.unwrap_or_default(),
                            bubbles: Vec::new(),
                        },
                    );
                }
            }
        }

        // Load bubbles and extract workspace info
        for entry in reader.fetch_bubbles()? {
            if let Some(conv_id) = extract_conversation_id(&entry.key) {
                if let Ok(bubble) = parse_bubble(&entry.value) {
                    // Skip empty messages
                    if bubble.text.trim().is_empty() {
                        continue;
                    }

                    // Try to extract workspace info from bubble JSON
                    if let Ok(json) = serde_json::from_slice::<serde_json::Value>(&entry.value) {
                        if let Some(workspace_uri) = json
                            .get("workspaceUris")
                            .and_then(|v| v.as_array())
                            .and_then(|arr| arr.first())
                            .and_then(|v| v.as_str())
                        {
                            let mut ws_info = WorkspaceInfo::from_uri(workspace_uri);
                            ws_info.cursor_path = json
                                .get("workspaceProjectDir")
                                .and_then(|v| v.as_str())
                                .map(String::from);

                            workspace_map.insert(conv_id.to_string(), ws_info);
                        }
                    }

                    // Get or create conversation
                    let conv = conversations
                        .entry(conv_id.to_string())
                        .or_insert_with(|| Conversation {
                            composer_id: conv_id.to_string(),
                            title: String::new(),
                            created_at: bubble.created_at,
                            model_config: crate::domain::ModelConfig::default(),
                            unified_mode: String::new(),
                            bubbles: Vec::new(),
                        });

                    conv.bubbles.push(bubble);
                }
            }
        }

        // Sort bubbles and generate titles
        let mut result: Vec<Conversation> = conversations
            .into_values()
            .filter(|c| !c.bubbles.is_empty())
            .map(|mut c| {
                c.bubbles.sort_by(|a, b| a.created_at.cmp(&b.created_at));
                c.title = c.generate_title();
                c
            })
            .collect();

        // Sort by creation time (newest first)
        result.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok((result, workspace_map))
    }

    /// Calculate a hash for change detection.
    fn calculate_hash(&self, conv: &Conversation) -> String {
        let mut hasher = DefaultHasher::new();

        conv.composer_id.hash(&mut hasher);
        conv.title.hash(&mut hasher);
        conv.bubbles.len().hash(&mut hasher);

        // Hash last few bubbles for efficiency
        for bubble in conv.bubbles.iter().rev().take(5) {
            bubble.bubble_id.hash(&mut hasher);
            bubble.text.hash(&mut hasher);
        }

        format!("{:x}", hasher.finish())
    }

    /// Get current sync state.
    pub fn get_state(&self) -> Result<SyncState> {
        self.storage.get_sync_state()
    }

    /// Get conversations from local storage.
    pub fn get_conversations(&self, workspace: Option<&str>) -> Result<Vec<Conversation>> {
        self.storage.get_conversations(workspace)
    }

    /// Get all workspaces.
    pub fn get_workspaces(&self) -> Result<Vec<WorkspaceInfo>> {
        self.storage.get_workspaces()
    }

    /// Check if storage is within limits.
    pub fn check_storage_limits(&self) -> Result<bool> {
        let current_size = self.storage.get_storage_size()?;
        let max_size = self.config.max_storage_bytes();
        Ok(current_size < max_size)
    }

    /// Get storage usage info.
    pub fn get_storage_info(&self) -> Result<StorageInfo> {
        let current_bytes = self.storage.get_storage_size()?;
        let max_bytes = self.config.max_storage_bytes();

        Ok(StorageInfo {
            current_bytes,
            max_bytes,
            usage_percent: (current_bytes as f64 / max_bytes as f64) * 100.0,
            conversation_count: self.storage.get_conversation_count()?,
            message_count: self.storage.get_message_count()?,
        })
    }
}

/// Storage usage information.
#[derive(Debug, Clone)]
pub struct StorageInfo {
    /// Current storage size in bytes.
    pub current_bytes: u64,
    /// Maximum allowed storage in bytes.
    pub max_bytes: u64,
    /// Usage percentage (0-100).
    pub usage_percent: f64,
    /// Total conversation count.
    pub conversation_count: usize,
    /// Total message count.
    pub message_count: usize,
}

impl StorageInfo {
    /// Format current size as human readable.
    #[must_use]
    pub fn current_size_human(&self) -> String {
        format_bytes(self.current_bytes)
    }

    /// Format max size as human readable.
    #[must_use]
    pub fn max_size_human(&self) -> String {
        format_bytes(self.max_bytes)
    }
}

/// Format bytes as human readable string.
fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_bytes() {
        assert_eq!(format_bytes(500), "500 B");
        assert_eq!(format_bytes(1024), "1.00 KB");
        assert_eq!(format_bytes(1024 * 1024), "1.00 MB");
        assert_eq!(format_bytes(1024 * 1024 * 1024), "1.00 GB");
    }
}

