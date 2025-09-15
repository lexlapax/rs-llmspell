//! Configuration management commands
//!
//! Consolidates init, validate, and show config functionality into
//! config subcommands.

use crate::cli::{ConfigCommands, ConfigFormat, OutputFormat};
use anyhow::Result;
use llmspell_config::LLMSpellConfig;
use std::path::PathBuf;

/// Handle configuration management commands
pub async fn handle_config_command(
    command: ConfigCommands,
    runtime_config: LLMSpellConfig,
    output_format: OutputFormat,
) -> Result<()> {
    match command {
        ConfigCommands::Init { output, force } => init_config(output, force).await,
        ConfigCommands::Validate { file } => validate_config(file, output_format).await,
        ConfigCommands::Show { section, format } => {
            show_config(section, format, output_format, runtime_config).await
        }
    }
}

/// Initialize configuration file (moved from init.rs)
async fn init_config(output: PathBuf, force: bool) -> Result<()> {
    // Delegate to existing init module
    crate::commands::init::init_config(output, force).await
}

/// Validate configuration file (moved from validate.rs)
async fn validate_config(config: Option<PathBuf>, output_format: OutputFormat) -> Result<()> {
    // Delegate to existing validate module
    crate::commands::validate::validate_config(config, output_format).await
}

/// Show current configuration
async fn show_config(
    section: Option<String>,
    format: ConfigFormat,
    output_format: OutputFormat,
    runtime_config: LLMSpellConfig,
) -> Result<()> {
    // Convert to the desired configuration format
    let config_data = if let Some(ref section_name) = section {
        // Show specific section
        match section_name.as_str() {
            "rag" => serde_json::to_value(&runtime_config.rag)?,
            "providers" => serde_json::to_value(&runtime_config.providers)?,
            "engines" => serde_json::to_value(&runtime_config.engines)?,
            "runtime" => serde_json::to_value(&runtime_config.runtime)?,
            "tools" => serde_json::to_value(&runtime_config.tools)?,
            _ => {
                anyhow::bail!(
                    "Unknown config section '{}'. Available: rag, providers, engines, runtime, tools",
                    section_name
                );
            }
        }
    } else {
        // Show entire configuration
        serde_json::to_value(&runtime_config)?
    };

    // Format and display the configuration
    let formatted_output = match format {
        ConfigFormat::Json => serde_json::to_string_pretty(&config_data)?,
        ConfigFormat::Toml => {
            // Note: TOML serialization may not work for all complex structures
            match toml::to_string_pretty(&config_data) {
                Ok(toml_str) => toml_str,
                Err(_) => {
                    eprintln!("Warning: Cannot serialize to TOML, falling back to JSON");
                    serde_json::to_string_pretty(&config_data)?
                }
            }
        }
        ConfigFormat::Yaml => serde_yaml::to_string(&config_data)?,
    };

    match output_format {
        OutputFormat::Json if format == ConfigFormat::Json => {
            // Already in JSON format
            println!("{}", formatted_output);
        }
        OutputFormat::Json => {
            // Convert to JSON regardless of config format
            println!("{}", serde_json::to_string_pretty(&config_data)?);
        }
        OutputFormat::Text | OutputFormat::Pretty => {
            if let Some(section_name) = section {
                println!("Configuration section: {}", section_name);
                println!();
            } else {
                println!("Full configuration:");
                println!();
            }
            println!("{}", formatted_output);
        }
        OutputFormat::Yaml => {
            println!("{}", formatted_output);
        }
    }

    Ok(())
}
