//! Infrastructure layer - external adapters (database, filesystem).
//!
//! This layer handles all I/O operations and external dependencies.

pub mod config;
pub mod cursor_paths;
pub mod cursor_reset;
pub mod cursor_writer;
pub mod local_storage;
pub mod sqlite_reader;
pub mod systemd;

pub use config::{ensure_config_exists, load_config, save_config};
pub use cursor_paths::{find_cursor_config_dir, find_state_databases};
pub use cursor_reset::{CursorReset, ResetResult};
pub use cursor_writer::CursorWriter;
pub use local_storage::LocalStorage;
pub use sqlite_reader::StateDbReader;
pub use systemd::{InstallResult, ServiceStatus, SystemdService};
