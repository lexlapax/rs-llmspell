//! State management commands
//!
//! Provides CLI commands for managing persistent state using the StateManager
//! from llmspell-state-persistence crate.

use crate::cli::{ExportFormat, OutputFormat, StateCommands};
use anyhow::Result;
use llmspell_config::LLMSpellConfig;
use llmspell_state_persistence::{StateManager, StateScope};
use serde_json::Value;
use std::path::PathBuf;

/// Handle state management commands
pub async fn handle_state_command(
    command: StateCommands,
    _config: LLMSpellConfig,
    output_format: OutputFormat,
) -> Result<()> {
    // Create state manager using the configuration
    let state_manager = StateManager::new().await?;

    match command {
        StateCommands::Show { key } => show_state(&state_manager, key, output_format).await,
        StateCommands::Clear { key } => clear_state(&state_manager, key).await,
        StateCommands::Export { file, format } => {
            export_state(&state_manager, file, format).await
        }
        StateCommands::Import { file, merge } => import_state(&state_manager, file, merge).await,
    }
}

/// Show state values
async fn show_state(
    state_manager: &StateManager,
    key: Option<String>,
    output_format: OutputFormat,
) -> Result<()> {
    match key {
        Some(key) => {
            // Show specific key
            let value = state_manager.get(StateScope::Global, &key).await?;
            match value {
                Some(value) => {
                    match output_format {
                        OutputFormat::Json => {
                            println!("{}", serde_json::to_string_pretty(&value)?);
                        }
                        OutputFormat::Text | OutputFormat::Pretty => {
                            println!("Key: {}", key);
                            println!("Value: {}", value);
                        }
                    }
                }
                None => {
                    if matches!(output_format, OutputFormat::Text | OutputFormat::Pretty) {
                        println!("Key '{}' not found", key);
                    }
                }
            }
        }
        None => {
            // Show all state
            println!("State listing not yet implemented - please specify a key");
            // TODO: Implement list_all functionality when StateManager supports it
        }
    }

    Ok(())
}

/// Clear state values
async fn clear_state(state_manager: &StateManager, key: Option<String>) -> Result<()> {
    match key {
        Some(key) => {
            // Clear specific key
            state_manager.delete(StateScope::Global, &key).await?;
            println!("✓ Cleared state key: {}", key);
        }
        None => {
            // Clear all state
            println!("Clear all state not yet implemented - please specify a key");
            // TODO: Implement clear_all functionality when StateManager supports it
        }
    }

    Ok(())
}

/// Export state to file
async fn export_state(
    _state_manager: &StateManager,
    file: PathBuf,
    format: ExportFormat,
) -> Result<()> {
    println!("State export not yet fully implemented");
    println!("Would export to: {} (format: {:?})", file.display(), format);
    
    // TODO: Implement state export when StateManager supports listing all keys
    // let all_state = state_manager.export_all(StateScope::Global).await?;
    // let content = match format {
    //     ExportFormat::Json => serde_json::to_string_pretty(&all_state)?,
    //     ExportFormat::Yaml => serde_yaml::to_string(&all_state)?,
    //     ExportFormat::Toml => toml::to_string(&all_state)?,
    // };
    // tokio::fs::write(&file, content).await?;
    // println!("✓ Exported state to {}", file.display());

    Ok(())
}

/// Import state from file
async fn import_state(
    state_manager: &StateManager,
    file: PathBuf,
    merge: bool,
) -> Result<()> {
    let content = tokio::fs::read_to_string(&file).await?;
    
    // Parse the file content based on extension  
    let data: Value = if file.extension().and_then(|s| s.to_str()) == Some("yaml") {
        serde_yaml::from_str(&content)?
    } else if file.extension().and_then(|s| s.to_str()) == Some("toml") {
        toml::from_str(&content)?
    } else {
        serde_json::from_str(&content)?
    };

    // Import the data
    if let Value::Object(map) = data {
        for (key, value) in map {
            if merge {
                // Only set if key doesn't exist
                if state_manager.get(StateScope::Global, &key).await?.is_none() {
                    state_manager.set(StateScope::Global, &key, value).await?;
                }
            } else {
                // Always set (overwrite existing)
                state_manager.set(StateScope::Global, &key, value).await?;
            }
        }
        
        let action = if merge { "merged" } else { "imported" };
        println!("✓ {} state from {}", action, file.display());
    } else {
        anyhow::bail!("State file must contain an object at root level");
    }

    Ok(())
}