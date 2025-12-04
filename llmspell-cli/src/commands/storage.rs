//! Storage migration CLI commands for PostgreSQL ↔ SQLite data migration

use crate::cli::OutputFormat;
use anyhow::{anyhow, Context, Result};
use llmspell_config::LLMSpellConfig;
use std::path::PathBuf;
use std::sync::Arc;

/// Storage subcommands (imported from cli.rs)
pub use crate::cli::{MigrateAction, StorageCommands};

#[cfg(feature = "postgres")]
use llmspell_storage::backends::postgres::{PostgresBackend, PostgresConfig};
#[cfg(feature = "postgres")]
use llmspell_storage::export_import::PostgresExporter;
#[cfg(feature = "postgres")]
use llmspell_storage::export_import::PostgresImporter;

use llmspell_storage::backends::sqlite::{SqliteBackend, SqliteConfig};
use llmspell_storage::export_import::SqliteExporter;
use llmspell_storage::export_import::SqliteImporter;

/// Execute storage command
pub async fn handle_storage_command(
    command: StorageCommands,
    config: LLMSpellConfig,
    _output_format: OutputFormat,
) -> Result<()> {
    match command {
        StorageCommands::Export { backend, output } => {
            handle_export(&backend, output, &config).await
        }
        StorageCommands::Import { backend, input } => handle_import(&backend, input, &config).await,
        StorageCommands::Migrate { .. }
        | StorageCommands::Info { .. }
        | StorageCommands::Validate { .. } => Err(anyhow!(
            "This storage command is not yet implemented.\n\
                 \n\
                 Available commands:\n\
                 - export: Export data to JSON file\n\
                 - import: Import data from JSON file"
        )),
    }
}

/// Handle export command
async fn handle_export(backend: &str, output: PathBuf, config: &LLMSpellConfig) -> Result<()> {
    println!(
        "Exporting data from {} backend to {}...",
        backend,
        output.display()
    );

    match backend.to_lowercase().as_str() {
        "sqlite" => {
            // Use persistence path from config or default
            let db_path = config
                .rag
                .vector_storage
                .persistence_path
                .clone()
                .unwrap_or_else(|| PathBuf::from("./storage/llmspell.db"));

            let sqlite_config = SqliteConfig::new(db_path);
            let backend = Arc::new(SqliteBackend::new(sqlite_config).await?);
            let exporter = SqliteExporter::new(backend);

            println!("Starting export...");
            let export_data = exporter.export_all().await?;

            println!("Writing to file...");
            let json = serde_json::to_string_pretty(&export_data)
                .context("Failed to serialize export data")?;
            std::fs::write(&output, json)
                .with_context(|| format!("Failed to write to {}", output.display()))?;

            println!(
                "✅ Exported {} records to {}",
                count_records(&export_data),
                output.display()
            );
            Ok(())
        }
        #[cfg(feature = "postgres")]
        "postgres" | "postgresql" => {
            // Get connection string from environment variable
            let connection_string = std::env::var("DATABASE_URL")
                .context("DATABASE_URL environment variable not set for PostgreSQL export")?;

            let postgres_config = PostgresConfig::new(connection_string);
            let backend = Arc::new(PostgresBackend::new(postgres_config).await?);
            let exporter = PostgresExporter::new(backend);

            println!("Starting export...");
            let export_data = exporter.export_all().await?;

            println!("Writing to file...");
            let json = serde_json::to_string_pretty(&export_data)
                .context("Failed to serialize export data")?;
            std::fs::write(&output, json)
                .with_context(|| format!("Failed to write to {}", output.display()))?;

            println!(
                "✅ Exported {} records to {}",
                count_records(&export_data),
                output.display()
            );
            Ok(())
        }
        #[cfg(not(feature = "postgres"))]
        "postgres" | "postgresql" => Err(anyhow!(
            "PostgreSQL support not enabled. Rebuild with --features postgres"
        )),
        _ => Err(anyhow!(
            "Unknown backend: {}. Supported backends: sqlite, postgres",
            backend
        )),
    }
}

/// Handle import command
async fn handle_import(backend: &str, input: PathBuf, config: &LLMSpellConfig) -> Result<()> {
    println!(
        "Importing data into {} backend from {}...",
        backend,
        input.display()
    );

    if !input.exists() {
        return Err(anyhow!("Input file does not exist: {}", input.display()));
    }

    match backend.to_lowercase().as_str() {
        "sqlite" => {
            // Use persistence path from config or default
            let db_path = config
                .rag
                .vector_storage
                .persistence_path
                .clone()
                .unwrap_or_else(|| PathBuf::from("./storage/llmspell.db"));

            let sqlite_config = SqliteConfig::new(db_path);
            let backend = Arc::new(SqliteBackend::new(sqlite_config).await?);
            let importer = SqliteImporter::new(backend);

            println!("Reading JSON file...");
            let stats = importer
                .import_from_file(input.to_str().ok_or_else(|| anyhow!("Invalid file path"))?)
                .await?;

            println!("✅ Imported {} total records:", stats.total());
            print_import_stats(&stats);
            Ok(())
        }
        #[cfg(feature = "postgres")]
        "postgres" | "postgresql" => {
            // Get connection string from environment variable
            let connection_string = std::env::var("DATABASE_URL")
                .context("DATABASE_URL environment variable not set for PostgreSQL import")?;

            let postgres_config = PostgresConfig::new(connection_string);
            let backend = Arc::new(PostgresBackend::new(postgres_config).await?);
            let importer = PostgresImporter::new(backend);

            println!("Reading JSON file...");
            let stats = importer
                .import_from_file(input.to_str().ok_or_else(|| anyhow!("Invalid file path"))?)
                .await?;

            println!("✅ Imported {} total records:", stats.total());
            print_import_stats(&stats);
            Ok(())
        }
        #[cfg(not(feature = "postgres"))]
        "postgres" | "postgresql" => Err(anyhow!(
            "PostgreSQL support not enabled. Rebuild with --features postgres"
        )),
        _ => Err(anyhow!(
            "Unknown backend: {}. Supported backends: sqlite, postgres",
            backend
        )),
    }
}

/// Count total records in export data
fn count_records(export: &llmspell_storage::export_import::ExportFormat) -> usize {
    let mut total = 0;

    // Count vector embeddings
    for vectors in export.data.vector_embeddings.values() {
        total += vectors.len();
    }

    // Count knowledge graph
    if let Some(kg) = &export.data.knowledge_graph {
        total += kg.entities.len();
        total += kg.relationships.len();
    }

    // Count other components
    total += export.data.procedural_memory.len();
    total += export.data.agent_state.len();
    total += export.data.kv_store.len();
    total += export.data.workflow_states.len();
    total += export.data.sessions.len();

    if let Some(artifacts) = &export.data.artifacts {
        total += artifacts.content.len();
        total += artifacts.artifacts.len();
    }

    total += export.data.event_log.len();
    total += export.data.hook_history.len();

    total
}

/// Print import statistics
fn print_import_stats(stats: &llmspell_storage::export_import::ImportStats) {
    if stats.vectors > 0 {
        println!("  - Vectors: {}", stats.vectors);
    }
    if stats.entities > 0 {
        println!("  - Entities: {}", stats.entities);
    }
    if stats.relationships > 0 {
        println!("  - Relationships: {}", stats.relationships);
    }
    if stats.patterns > 0 {
        println!("  - Patterns: {}", stats.patterns);
    }
    if stats.agent_states > 0 {
        println!("  - Agent States: {}", stats.agent_states);
    }
    if stats.kv_entries > 0 {
        println!("  - KV Entries: {}", stats.kv_entries);
    }
    if stats.workflow_states > 0 {
        println!("  - Workflow States: {}", stats.workflow_states);
    }
    if stats.sessions > 0 {
        println!("  - Sessions: {}", stats.sessions);
    }
    if stats.artifact_content > 0 {
        println!("  - Artifact Content: {}", stats.artifact_content);
    }
    if stats.artifacts > 0 {
        println!("  - Artifacts: {}", stats.artifacts);
    }
    if stats.events > 0 {
        println!("  - Events: {}", stats.events);
    }
    if stats.hooks > 0 {
        println!("  - Hooks: {}", stats.hooks);
    }
}
