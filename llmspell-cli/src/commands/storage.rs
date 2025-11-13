//! ABOUTME: Storage migration CLI commands (deprecated - Phase 13c)
//! ABOUTME: SQLite-based storage implemented, use SQLite or PostgreSQL backends directly

use crate::cli::OutputFormat;
use anyhow::{anyhow, Result};
use llmspell_config::LLMSpellConfig;

/// Storage subcommands (imported from cli.rs)
pub use crate::cli::{MigrateAction, StorageCommands};

/// Execute storage command
///
/// # Errors
///
/// Returns error with migration deprecation message
pub async fn handle_storage_command(
    _command: StorageCommands,
    _config: LLMSpellConfig,
    _output_format: OutputFormat,
) -> Result<()> {
    Err(anyhow!(
        "Storage migration commands removed in Phase 13c.\n\
         \n\
         Sled backend has been removed. Use SQLite or PostgreSQL backends directly:\n\
         \n\
         For SQLite:\n\
         - Set backend_type = \"Sqlite\" in config\n\
         - Use libsql for embedded storage\n\
         \n\
         For PostgreSQL:\n\
         - Set backend_type = \"Postgres\" in config\n\
         - Configure connection string\n\
         \n\
         See docs/user-guide/05-storage.md for migration guide."
    ))
}
