//! Infrastructure layer - external adapters (database, filesystem).
//!
//! This layer handles all I/O operations and external dependencies.

pub mod cursor_paths;
pub mod sqlite_reader;

pub use cursor_paths::find_state_databases;
pub use sqlite_reader::StateDbReader;
