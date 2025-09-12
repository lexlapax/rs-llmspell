//! State management commands
//!
//! Provides CLI commands for managing persistent state through the kernel
//! using custom protocol messages (StateRequest/StateReply).

use crate::cli::{ExportFormat, OutputFormat, StateCommands};
use anyhow::Result;
use llmspell_config::LLMSpellConfig;
use llmspell_kernel::jupyter::protocol::StateOperation;
use serde_json::Value;
use std::path::PathBuf;

/// Handle state management commands
pub async fn handle_state_command(
    command: StateCommands,
    config: LLMSpellConfig,
    output_format: OutputFormat,
    connect: Option<String>, // Connection string for external kernel
) -> Result<()> {
    // Connect to kernel instead of creating local StateManager
    // This ensures state is shared between CLI and scripts
    let kernel = super::create_kernel_connection(config, connect).await?;

    match command {
        StateCommands::Show { key } => show_state_via_kernel(kernel, key, output_format).await,
        StateCommands::Clear { key } => clear_state_via_kernel(kernel, key).await,
        StateCommands::Export { file, format } => {
            export_state_via_kernel(kernel, Some(file), format).await
        }
        StateCommands::Import { file, merge } => import_state_via_kernel(kernel, file, merge).await,
    }
}

/// Show state values via kernel
async fn show_state_via_kernel(
    mut kernel: Box<dyn crate::kernel_client::KernelConnectionTrait>,
    key: Option<String>,
    output_format: OutputFormat,
) -> Result<()> {
    // Create the state operation
    let operation = StateOperation::Show { key: key.clone() };

    // Send state request to kernel
    let reply = kernel
        .state_request(serde_json::to_value(operation)?, Some("global".to_string()))
        .await?;

    // Process the reply
    if let Some(data) = reply.get("data") {
        match output_format {
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(data)?);
            }
            OutputFormat::Text | OutputFormat::Pretty => {
                if let Some(key) = key {
                    // Single key was requested
                    if data.is_null() {
                        println!("Key '{}' not found", key);
                    } else {
                        println!("Key: {}", key);
                        println!("Value: {}", data);
                    }
                } else {
                    // All keys were requested
                    if let Some(obj) = data.as_object() {
                        if obj.is_empty() {
                            println!("No state keys found");
                        } else {
                            println!("State keys ({} total):", obj.len());
                            for (k, v) in obj {
                                println!("  {} = {}", k, v);
                            }
                        }
                    }
                }
            }
        }
    } else {
        println!("No data returned");
    }

    Ok(())
}

/// Clear state values via kernel
async fn clear_state_via_kernel(
    mut kernel: Box<dyn crate::kernel_client::KernelConnectionTrait>,
    key: Option<String>,
) -> Result<()> {
    // Create the state operation
    let operation = StateOperation::Clear { key: key.clone() };

    // Send state request to kernel
    kernel
        .state_request(serde_json::to_value(operation)?, Some("global".to_string()))
        .await?;

    match key {
        Some(key) => println!("✓ Cleared state key: {}", key),
        None => println!("✓ Cleared all state keys"),
    }

    Ok(())
}

/// Export state to file via kernel
async fn export_state_via_kernel(
    mut kernel: Box<dyn crate::kernel_client::KernelConnectionTrait>,
    file: Option<PathBuf>,
    format: ExportFormat,
) -> Result<()> {
    // Create the state operation
    let format_str = match format {
        ExportFormat::Json => "json",
        ExportFormat::Yaml => "yaml",
        ExportFormat::Toml => "toml",
    };
    let operation = StateOperation::Export {
        format: Some(format_str.to_string()),
    };

    // Send state request to kernel
    let reply = kernel
        .state_request(serde_json::to_value(operation)?, Some("global".to_string()))
        .await?;

    // Process the reply
    if let Some(data) = reply.get("data") {
        // Format the data
        let formatted = match format {
            ExportFormat::Json => serde_json::to_string_pretty(data)?,
            ExportFormat::Yaml => {
                // For now, just use JSON (YAML support can be added later)
                serde_json::to_string_pretty(data)?
            }
            ExportFormat::Toml => {
                // For now, just use JSON (TOML support can be added later)
                serde_json::to_string_pretty(data)?
            }
        };

        // Write to file or stdout
        if let Some(file) = file {
            std::fs::write(&file, formatted)?;
            println!("✓ Exported state to {}", file.display());
        } else {
            println!("{}", formatted);
        }
    } else {
        println!("No state to export");
    }

    Ok(())
}

/// Import state from file via kernel
async fn import_state_via_kernel(
    mut kernel: Box<dyn crate::kernel_client::KernelConnectionTrait>,
    file: PathBuf,
    merge: bool,
) -> Result<()> {
    // Read the file
    let content = std::fs::read_to_string(&file)?;
    let data: Value = serde_json::from_str(&content)?;

    // Create the state operation
    let operation = StateOperation::Import { data, merge };

    // Send state request to kernel
    kernel
        .state_request(serde_json::to_value(operation)?, Some("global".to_string()))
        .await?;

    println!("✓ Imported state from {}", file.display());
    if merge {
        println!("  (merged with existing state)");
    } else {
        println!("  (replaced existing state)");
    }

    Ok(())
}
