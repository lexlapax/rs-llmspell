//! ABOUTME: Validate command implementation for configuration validation
//! ABOUTME: Checks configuration files for errors and completeness

use crate::cli::OutputFormat;
use crate::config;
use anyhow::Result;
use serde_json::json;
use std::path::{Path, PathBuf};

/// Validate configuration file
pub async fn validate_config(
    config_path: Option<PathBuf>,
    output_format: OutputFormat,
) -> Result<()> {
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
                    if config.providers.configs.is_empty() {
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
