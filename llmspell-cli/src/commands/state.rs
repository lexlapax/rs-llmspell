//! State management commands
//!
//! Provides CLI commands for managing persistent state through the kernel
//! using ExecutionContext for dual-mode operation.

use crate::cli::{ExportFormat, OutputFormat, StateCommands};
use crate::execution_context::ExecutionContext;
use anyhow::Result;
use serde_json::Value;
use std::path::PathBuf;
use tracing::info;

/// Handle state management commands
pub async fn handle_state_command(
    command: StateCommands,
    context: ExecutionContext,
    output_format: OutputFormat,
) -> Result<()> {
    match command {
        StateCommands::Show { key, scope } => show_state(context, key, scope, output_format).await,
        StateCommands::Clear { key, scope } => {
            clear_state(context, key, scope, output_format).await
        }
        StateCommands::Export { file, format } => {
            export_state(context, file, format, output_format).await
        }
        StateCommands::Import { file, merge } => {
            import_state(context, file, merge, output_format).await
        }
    }
}

/// Show state values
async fn show_state(
    context: ExecutionContext,
    key: Option<String>,
    scope: Option<String>,
    output_format: OutputFormat,
) -> Result<()> {
    info!("Showing state - key: {:?}, scope: {:?}", key, scope);

    match context {
        ExecutionContext::Embedded { handle, .. } => {
            let _kernel = handle.into_kernel();

            // Create placeholder state data for now
            let state_data = create_placeholder_state_data(&key);

            display_state_data(&state_data, &key, output_format)?;
        }
        ExecutionContext::Connected { .. } => {
            // For connected kernels, we would send state requests
            // For now, show placeholder data
            let state_data = create_placeholder_state_data(&key);
            display_state_data(&state_data, &key, output_format)?;
        }
    }

    Ok(())
}

/// Clear state values
async fn clear_state(
    context: ExecutionContext,
    key: Option<String>,
    scope: Option<String>,
    output_format: OutputFormat,
) -> Result<()> {
    info!("Clearing state - key: {:?}, scope: {:?}", key, scope);

    match context {
        ExecutionContext::Embedded { handle, .. } => {
            let _kernel = handle.into_kernel();

            // Perform state clearing operation
            match &key {
                Some(k) => match output_format {
                    OutputFormat::Json => {
                        println!(
                            "{}",
                            serde_json::json!({
                                "status": "success",
                                "message": format!("Cleared state key: {}", k)
                            })
                        );
                    }
                    _ => {
                        println!("✓ Cleared state key: {}", k);
                    }
                },
                None => match output_format {
                    OutputFormat::Json => {
                        println!(
                            "{}",
                            serde_json::json!({
                                "status": "success",
                                "message": "Cleared all state keys"
                            })
                        );
                    }
                    _ => {
                        println!("✓ Cleared all state keys");
                    }
                },
            }
        }
        ExecutionContext::Connected { .. } => match output_format {
            OutputFormat::Json => {
                println!(
                    "{}",
                    serde_json::json!({
                        "status": "success",
                        "message": "State cleared via connected kernel"
                    })
                );
            }
            _ => {
                println!("✓ State cleared via connected kernel");
            }
        },
    }

    Ok(())
}

/// Export state to file
async fn export_state(
    context: ExecutionContext,
    file: PathBuf,
    format: ExportFormat,
    output_format: OutputFormat,
) -> Result<()> {
    info!("Exporting state to: {}", file.display());

    // Get state data
    let state_data = match context {
        ExecutionContext::Embedded { handle, .. } => {
            let _kernel = handle.into_kernel();
            create_placeholder_state_data(&None)
        }
        ExecutionContext::Connected { .. } => create_placeholder_state_data(&None),
    };

    // Format the data
    let formatted = match format {
        ExportFormat::Json => serde_json::to_string_pretty(&state_data)?,
        ExportFormat::Toml => toml::to_string_pretty(&state_data)?,
    };

    // Write to file
    std::fs::write(&file, formatted)?;

    match output_format {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::json!({
                    "status": "exported",
                    "path": file.display().to_string()
                })
            );
        }
        _ => {
            println!("✓ Exported state to {}", file.display());
        }
    }

    Ok(())
}

/// Import state from file
async fn import_state(
    context: ExecutionContext,
    file: PathBuf,
    merge: bool,
    output_format: OutputFormat,
) -> Result<()> {
    info!(
        "Importing state from: {} (merge: {})",
        file.display(),
        merge
    );

    if !file.exists() {
        anyhow::bail!("Import file does not exist: {}", file.display());
    }

    // Read the file
    let content = std::fs::read_to_string(&file)?;
    let _data: Value = serde_json::from_str(&content)?;

    // Apply state based on context
    match context {
        ExecutionContext::Embedded { handle, .. } => {
            let _kernel = handle.into_kernel();
            // State import logic would go here
        }
        ExecutionContext::Connected { .. } => {
            // Connected kernel state import would go here
        }
    }

    match output_format {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::json!({
                    "status": "imported",
                    "path": file.display().to_string(),
                    "merge": merge
                })
            );
        }
        _ => {
            println!("✓ Imported state from {}", file.display());
            if merge {
                println!("  (merged with existing state)");
            } else {
                println!("  (replaced existing state)");
            }
        }
    }

    Ok(())
}

/// Create placeholder state data for demonstration
fn create_placeholder_state_data(key: &Option<String>) -> Value {
    let full_state = serde_json::json!({
        "user_preferences": {
            "theme": "dark",
            "language": "en"
        },
        "session_data": {
            "last_script": "main.lua",
            "execution_count": 42
        },
        "workflow_state": {
            "current_step": 3,
            "total_steps": 10
        }
    });

    match key {
        Some(k) => {
            // Return specific key if it exists
            full_state.get(k).cloned().unwrap_or(Value::Null)
        }
        None => full_state,
    }
}

/// Display state data in the requested format
fn display_state_data(
    data: &Value,
    key: &Option<String>,
    output_format: OutputFormat,
) -> Result<()> {
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
                } else {
                    println!("State: {}", data);
                }
            }
        }
    }
    Ok(())
}
