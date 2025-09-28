//! Configuration management commands
//!
//! Consolidates init, validate, and show config functionality into
//! config subcommands.

use crate::cli::{ConfigCommands, ConfigFormat, OutputFormat};
use crate::config;
use anyhow::Result;
use llmspell_config::LLMSpellConfig;
use serde_json::json;
use std::path::{Path, PathBuf};

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
    // Check if file already exists
    if output.exists() && !force {
        anyhow::bail!(
            "Configuration file already exists: {}. Use --force to overwrite.",
            output.display()
        );
    }

    // Create the configuration file
    config::create_default_config(&output).await?;

    println!("✓ Created configuration file: {}", output.display());
    println!();
    println!("Next steps:");
    println!("  1. Edit {} to configure your settings", output.display());
    println!("  2. Set API keys:");
    println!("     - OPENAI_API_KEY for OpenAI provider");
    println!("     - ANTHROPIC_API_KEY for Anthropic provider");
    println!("     - COHERE_API_KEY for Cohere provider");
    println!("  3. Run 'llmspell config validate' to check your configuration");
    println!("  4. Run 'llmspell run <script>' to execute scripts");

    Ok(())
}

/// Validate configuration file (moved from validate.rs)
async fn validate_config(config_path: Option<PathBuf>, output_format: OutputFormat) -> Result<()> {
    let path = config_path.as_deref();
    let mut warnings = Vec::new();
    let mut errors = Vec::new();

    // Try to load the configuration
    let (config_result, actual_path) = match path {
        Some(p) => {
            let result = config::load_runtime_config(Some(p)).await;
            (result, p.to_string_lossy().to_string())
        }
        None => {
            // Try to discover config file
            let result = config::load_runtime_config(None).await;
            let discovered_path = discover_actual_path().await;
            (result, discovered_path)
        }
    };

    // Check if configuration loaded successfully
    let valid = match config_result {
        Ok(config) => {
            // Validate the loaded configuration
            match config::validate_config(&config) {
                Ok(_) => {
                    // Additional checks
                    if config.providers.providers.is_empty() {
                        warnings.push("No providers configured".to_string());
                    }

                    if !config.runtime.security.allow_network_access {
                        warnings.push(
                            "Network access is disabled - LLM providers won't work".to_string(),
                        );
                    }

                    true
                }
                Err(e) => {
                    errors.push(format!("Validation error: {}", e));
                    false
                }
            }
        }
        Err(e) => {
            errors.push(format!("Failed to load configuration: {}", e));
            false
        }
    };

    let validation_result = json!({
        "valid": valid,
        "path": actual_path,
        "warnings": warnings,
        "errors": errors
    });

    match output_format {
        OutputFormat::Json => {
            println!("{}", serde_json::to_string_pretty(&validation_result)?);
        }
        OutputFormat::Yaml => {
            println!("{}", serde_yaml::to_string(&validation_result)?);
        }
        OutputFormat::Text | OutputFormat::Pretty => {
            println!("Configuration validation:");
            println!("  File: {}", actual_path);

            if valid {
                println!("  Status: ✓ Valid");
            } else {
                println!("  Status: ✗ Invalid");
            }

            if !warnings.is_empty() {
                println!("\nWarnings:");
                for warning in &warnings {
                    println!("  ⚠ {}", warning);
                }
            }

            if !errors.is_empty() {
                println!("\nErrors:");
                for error in &errors {
                    println!("  ✗ {}", error);
                }
            }
        }
    }

    if !valid {
        anyhow::bail!("Configuration validation failed");
    }

    Ok(())
}

/// Try to discover which config file would be used
async fn discover_actual_path() -> String {
    // Check standard paths
    for path in &["llmspell.toml", ".llmspell.toml", "config/llmspell.toml"] {
        if Path::new(path).exists() {
            return path.to_string();
        }
    }

    // Check home directory
    if let Ok(home) = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE")) {
        for filename in &[".llmspell.toml", ".config/llmspell.toml"] {
            let path = PathBuf::from(&home).join(filename);
            if path.exists() {
                return path.display().to_string();
            }
        }
    }

    "(no config file found - using defaults)".to_string()
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
