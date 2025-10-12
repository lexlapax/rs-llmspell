//! ABOUTME: Configuration loading and management for CLI
//! ABOUTME: Delegates to llmspell-config for all configuration operations

use anyhow::{Context, Result};
use llmspell_config::LLMSpellConfig;
use std::path::Path;
use tokio::fs;
use tracing::info;

/// Load runtime configuration from profile, file, or use defaults
/// Delegates to llmspell-config which handles:
/// - Builtin profile loading
/// - Configuration file discovery
/// - TOML parsing and validation
/// - Environment variable overrides
///
/// Precedence: profile > config_path > discovery > default
pub async fn load_runtime_config(
    config_path: Option<&Path>,
    profile: Option<&str>,
) -> Result<LLMSpellConfig> {
    // Delegate to llmspell-config's comprehensive load_with_profile
    // This handles profile loading, discovery, environment overrides, and validation
    let config = LLMSpellConfig::load_with_profile(config_path, profile)
        .await
        .map_err(|e| anyhow::anyhow!("Configuration error: {}", e))?;

    // Validation is already done in load_with_profile
    Ok(config)
}

/// Create default configuration file
pub async fn create_default_config(path: &Path) -> Result<()> {
    let default_config = LLMSpellConfig::default();
    let toml_content = toml::to_string_pretty(&default_config)
        .context("Failed to serialize default configuration")?;

    // Create parent directories if needed
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .await
            .with_context(|| format!("Failed to create config directory: {}", parent.display()))?;
    }

    // Write the configuration file
    fs::write(path, toml_content)
        .await
        .with_context(|| format!("Failed to write config file: {}", path.display()))?;

    info!("Created default configuration at: {}", path.display());
    Ok(())
}

/// Validate configuration
/// Delegates to llmspell-config's comprehensive validation
pub fn validate_config(config: &LLMSpellConfig) -> Result<()> {
    // Delegate to config's own validation method
    config
        .validate()
        .map_err(|e| anyhow::anyhow!("Configuration validation failed: {}", e))
}
