//! ABOUTME: Backup and restore CLI commands for state management
//! ABOUTME: Provides operational commands for backup creation, listing, and recovery

use anyhow::{Context, Result};
use clap::{Args, Subcommand};
use llmspell_bridge::RuntimeConfig;
use llmspell_state_persistence::backup::{BackupManager, RestoreOptions};
use llmspell_state_persistence::manager::StateManager;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::cli::OutputFormat;

/// Backup management commands
#[derive(Debug, Args)]
pub struct BackupCommand {
    #[command(subcommand)]
    pub command: BackupSubcommand,
}

/// Backup subcommands
#[derive(Debug, Subcommand)]
pub enum BackupSubcommand {
    /// Create a new backup
    Create {
        /// Create incremental backup (default: full backup)
        #[arg(short, long)]
        incremental: bool,

        /// Add custom description
        #[arg(short, long)]
        description: Option<String>,
    },

    /// List available backups
    List {
        /// Maximum number of backups to show
        #[arg(short, long, default_value = "20")]
        limit: usize,

        /// Show detailed information
        #[arg(short, long)]
        verbose: bool,
    },

    /// Restore from a backup
    Restore {
        /// Backup ID to restore from
        backup_id: String,

        /// Perform dry run without making changes
        #[arg(short, long)]
        dry_run: bool,

        /// Skip checksum verification
        #[arg(long)]
        skip_verify: bool,

        /// Don't create a backup before restoring
        #[arg(long)]
        no_backup: bool,
    },

    /// Validate backup integrity
    Validate {
        /// Backup ID to validate
        backup_id: String,
    },

    /// Show backup details
    Info {
        /// Backup ID to inspect
        backup_id: String,
    },
}

/// Execute backup command
pub async fn execute_backup(
    cmd: BackupCommand,
    config: &RuntimeConfig,
    output_format: OutputFormat,
) -> Result<()> {
    // Initialize state infrastructure if needed
    let (_state_manager, backup_manager) = initialize_backup_infrastructure(config).await?;

    match cmd.command {
        BackupSubcommand::Create {
            incremental,
            description,
        } => create_backup(backup_manager, incremental, description, output_format).await,
        BackupSubcommand::List { limit, verbose } => {
            list_backups(backup_manager, limit, verbose, output_format).await
        }
        BackupSubcommand::Restore {
            backup_id,
            dry_run,
            skip_verify,
            no_backup,
        } => {
            restore_backup(
                backup_manager,
                backup_id,
                dry_run,
                skip_verify,
                no_backup,
                output_format,
            )
            .await
        }
        BackupSubcommand::Validate { backup_id } => {
            validate_backup(backup_manager, backup_id, output_format).await
        }
        BackupSubcommand::Info { backup_id } => {
            show_backup_info(backup_manager, backup_id, output_format).await
        }
    }
}

/// Initialize backup infrastructure
async fn initialize_backup_infrastructure(
    _config: &RuntimeConfig,
) -> Result<(Arc<RwLock<StateManager>>, Arc<BackupManager>)> {
    use llmspell_state_persistence::config::{BackupConfig, StorageBackendType};

    // For now, we'll use default state config

    // Create state manager with memory backend
    let state_manager = Arc::new(RwLock::new(
        StateManager::with_backend(StorageBackendType::Memory, Default::default())
            .await
            .context("Failed to create state manager")?,
    ));

    // Create backup manager
    let backup_config = BackupConfig {
        backup_dir: std::path::PathBuf::from("./backups"),
        compression_enabled: true,
        compression_type: llmspell_state_persistence::config::CompressionType::Zstd,
        compression_level: 3,
        encryption_enabled: false,
        max_backups: Some(10),
        max_backup_age: None,
        incremental_enabled: true,
        full_backup_interval: std::time::Duration::from_secs(3600),
    };

    let backup_manager = Arc::new(
        BackupManager::new(backup_config, state_manager.clone())
            .context("Failed to create backup manager")?,
    );

    Ok((state_manager, backup_manager))
}

/// Create a backup
async fn create_backup(
    backup_manager: Arc<BackupManager>,
    incremental: bool,
    description: Option<String>,
    output_format: OutputFormat,
) -> Result<()> {
    eprintln!(
        "Creating {} backup{}...",
        if incremental { "incremental" } else { "full" },
        if let Some(ref desc) = description {
            format!(" ({})", desc)
        } else {
            String::new()
        }
    );

    let start_time = std::time::Instant::now();
    let backup_status = backup_manager.create_backup(incremental).await?;
    let duration = start_time.elapsed();

    eprintln!(
        "✅ Backup created successfully in {:.2}s",
        duration.as_secs_f64()
    );

    let result = json!({
        "backup_id": backup_status.id,
        "type": if incremental { "incremental" } else { "full" },
        "size_bytes": backup_status.size_bytes,
        "entry_count": backup_status.entry_count,
        "created_at": backup_status.created_at,
        "parent_id": backup_status.parent_id,
        "duration_ms": duration.as_millis(),
        "description": description,
    });

    match output_format {
        OutputFormat::Json | OutputFormat::Pretty => {
            println!("{}", serde_json::to_string_pretty(&result)?)
        }
        OutputFormat::Text => {}
    }
    Ok(())
}

/// List backups
async fn list_backups(
    backup_manager: Arc<BackupManager>,
    limit: usize,
    verbose: bool,
    output_format: OutputFormat,
) -> Result<()> {
    let backups = backup_manager.list_backups().await?;
    let display_backups: Vec<_> = backups.into_iter().take(limit).collect();

    if display_backups.is_empty() {
        eprintln!("No backups found");
        return Ok(());
    }

    eprintln!(
        "Found {} backups (showing {})",
        display_backups.len(),
        display_backups.len().min(limit)
    );

    if verbose || matches!(output_format, OutputFormat::Json | OutputFormat::Pretty) {
        let result = json!(display_backups);
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        // Simple table format
        println!("ID                          Type          Size       Entries   Created");
        println!(
            "-------------------------   -----------   --------   -------   -------------------"
        );

        for backup in display_backups {
            let backup_type = if backup.is_incremental {
                "Incremental"
            } else {
                "Full"
            };
            let size = format_bytes(backup.size_bytes);
            let created = chrono::DateTime::<chrono::Local>::from(backup.created_at)
                .format("%Y-%m-%d %H:%M:%S")
                .to_string();

            println!(
                "{:<25}   {:<11}   {:>8}   {:>7}   {}",
                truncate_string(&backup.id, 25),
                backup_type,
                size,
                backup.entry_count,
                created
            );
        }
    }

    Ok(())
}

/// Restore from backup
async fn restore_backup(
    backup_manager: Arc<BackupManager>,
    backup_id: String,
    dry_run: bool,
    skip_verify: bool,
    no_backup: bool,
    _output_format: OutputFormat,
) -> Result<()> {
    let options = RestoreOptions {
        verify_checksums: !skip_verify,
        backup_current: !no_backup,
        target_version: None,
        dry_run,
    };

    eprintln!(
        "Restoring from backup: {} {}",
        backup_id,
        if dry_run { "(dry run)" } else { "" }
    );

    if !skip_verify {
        eprintln!("Verifying backup integrity...");
    }

    if !no_backup && !dry_run {
        eprintln!("Creating backup of current state...");
    }

    let start_time = std::time::Instant::now();
    backup_manager.restore_backup(&backup_id, options).await?;
    let duration = start_time.elapsed();

    eprintln!(
        "✅ Restore {} successfully in {:.2}s",
        if dry_run {
            "simulation completed"
        } else {
            "completed"
        },
        duration.as_secs_f64()
    );

    Ok(())
}

/// Validate backup
async fn validate_backup(
    backup_manager: Arc<BackupManager>,
    backup_id: String,
    output_format: OutputFormat,
) -> Result<()> {
    eprintln!("Validating backup: {}", backup_id);

    let validation = backup_manager.validate_backup(&backup_id).await?;

    if validation.is_valid {
        eprintln!("✅ Backup is valid");
    } else {
        eprintln!("❌ Backup validation failed");
    }

    let result = json!({
        "backup_id": backup_id,
        "is_valid": validation.is_valid,
        "checksum_valid": validation.checksum_valid,
        "integrity_valid": validation.integrity_valid,
        "validated_at": validation.validated_at,
        "errors": validation.errors,
        "warnings": validation.warnings,
    });

    match output_format {
        OutputFormat::Json | OutputFormat::Pretty => {
            println!("{}", serde_json::to_string_pretty(&result)?)
        }
        OutputFormat::Text => {}
    }
    Ok(())
}

/// Show backup information
async fn show_backup_info(
    backup_manager: Arc<BackupManager>,
    backup_id: String,
    output_format: OutputFormat,
) -> Result<()> {
    let backups = backup_manager.list_backups().await?;
    let backup = backups
        .into_iter()
        .find(|b| b.id == backup_id)
        .ok_or_else(|| anyhow::anyhow!("Backup not found: {}", backup_id))?;

    eprintln!("Backup Information: {}", backup_id);

    match output_format {
        OutputFormat::Json | OutputFormat::Pretty => {
            println!("{}", serde_json::to_string_pretty(&json!(backup))?)
        }
        OutputFormat::Text => {
            println!("ID: {}", backup.id);
            println!(
                "Type: {}",
                if backup.is_incremental {
                    "Incremental"
                } else {
                    "Full"
                }
            );
            println!("Size: {}", format_bytes(backup.size_bytes));
            println!("Entries: {}", backup.entry_count);
            println!(
                "Created: {}",
                chrono::DateTime::<chrono::Local>::from(backup.created_at)
                    .format("%Y-%m-%d %H:%M:%S")
            );
            if let Some(parent) = backup.parent_id {
                println!("Parent: {}", parent);
            }
        }
    }

    Ok(())
}

/// Format bytes to human-readable string
fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", size as u64, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Truncate string to specified length
fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}
