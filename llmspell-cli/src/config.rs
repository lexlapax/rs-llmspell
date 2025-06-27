//! ABOUTME: Configuration loading and management for CLI
//! ABOUTME: Handles loading runtime configuration from files and environment

use llmspell_bridge::RuntimeConfig;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::env;
use tokio::fs;

/// Configuration file discovery order
const CONFIG_SEARCH_PATHS: &[&str] = &[
    "llmspell.toml",
    ".llmspell.toml",
    "config/llmspell.toml",
    ".config/llmspell.toml",
];

/// Environment variable prefix
const ENV_PREFIX: &str = "LLMSPELL_";

/// Load runtime configuration from file or use defaults
pub async fn load_runtime_config(config_path: Option<&Path>) -> Result<RuntimeConfig> {
    // If explicit path provided, use it
    if let Some(path) = config_path {
        if path.exists() {
            return load_from_file(path).await;
        } else {
            anyhow::bail!("Configuration file not found: {}", path.display());
        }
    }
    
    // Try to discover config file
    if let Some(path) = discover_config_file().await? {
        tracing::info!("Found configuration file: {}", path.display());
        let mut config = load_from_file(&path).await?;
        
        // Apply environment overrides
        apply_environment_overrides(&mut config)?;
        
        return Ok(config);
    }
    
    // No config file found, use defaults with environment overrides
    let mut config = RuntimeConfig::default();
    apply_environment_overrides(&mut config)?;
    
    Ok(config)
}

/// Discover configuration file in standard locations
async fn discover_config_file() -> Result<Option<PathBuf>> {
    // Check current directory first
    for path in CONFIG_SEARCH_PATHS {
        let path = PathBuf::from(path);
        if path.exists() {
            return Ok(Some(path));
        }
    }
    
    // Check home directory
    if let Ok(home_dir) = env::var("HOME").or_else(|_| env::var("USERPROFILE")) {
        let home_path = PathBuf::from(home_dir);
        
        for filename in &[".llmspell.toml", ".config/llmspell.toml"] {
            let path = home_path.join(filename);
            if path.exists() {
                return Ok(Some(path));
            }
        }
    }
    
    // Check system config directories
    if let Some(config_dir) = dirs::config_dir() {
        let path = config_dir.join("llmspell").join("config.toml");
        if path.exists() {
            return Ok(Some(path));
        }
    }
    
    Ok(None)
}

/// Load configuration from TOML file
async fn load_from_file(path: &Path) -> Result<RuntimeConfig> {
    let content = fs::read_to_string(path)
        .await
        .with_context(|| format!("Failed to read config file: {}", path.display()))?;
    
    let config = toml::from_str(&content)
        .with_context(|| format!("Failed to parse config file: {}", path.display()))?;
    
    Ok(config)
}

/// Apply environment variable overrides
fn apply_environment_overrides(config: &mut RuntimeConfig) -> Result<()> {
    // Override default engine
    if let Ok(engine) = env::var(format!("{}DEFAULT_ENGINE", ENV_PREFIX)) {
        config.default_engine = engine;
    }
    
    // Provider API keys are handled via api_key_env in provider config
    // Set default provider if specified
    if let Ok(provider) = env::var(format!("{}DEFAULT_PROVIDER", ENV_PREFIX)) {
        config.providers.default_provider = Some(provider);
    }
    
    // Override security settings
    if let Ok(val) = env::var(format!("{}ALLOW_FILE_ACCESS", ENV_PREFIX)) {
        config.runtime.security.allow_file_access = val.parse()
            .with_context(|| "Invalid boolean value for LLMSPELL_ALLOW_FILE_ACCESS")?;
    }
    
    if let Ok(val) = env::var(format!("{}ALLOW_NETWORK_ACCESS", ENV_PREFIX)) {
        config.runtime.security.allow_network_access = val.parse()
            .with_context(|| "Invalid boolean value for LLMSPELL_ALLOW_NETWORK_ACCESS")?;
    }
    
    if let Ok(val) = env::var(format!("{}MAX_MEMORY_MB", ENV_PREFIX)) {
        let mb: usize = val.parse()
            .with_context(|| "Invalid value for LLMSPELL_MAX_MEMORY_MB")?;
        config.runtime.security.max_memory_bytes = Some(mb * 1024 * 1024);
    }
    
    // Override runtime settings
    if let Ok(val) = env::var(format!("{}SCRIPT_TIMEOUT", ENV_PREFIX)) {
        config.runtime.script_timeout_seconds = val.parse()
            .with_context(|| "Invalid value for LLMSPELL_SCRIPT_TIMEOUT")?;
    }
    
    if let Ok(val) = env::var(format!("{}ENABLE_STREAMING", ENV_PREFIX)) {
        config.runtime.enable_streaming = val.parse()
            .with_context(|| "Invalid boolean value for LLMSPELL_ENABLE_STREAMING")?;
    }
    
    Ok(())
}

/// Create default configuration file
pub async fn create_default_config(path: &Path) -> Result<()> {
    let default_config = RuntimeConfig::default();
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
    
    tracing::info!("Created default configuration at: {}", path.display());
    Ok(())
}

/// Validate configuration
pub fn validate_config(config: &RuntimeConfig) -> Result<()> {
    // Validate engine is supported
    if !config.supports_engine(&config.default_engine) {
        anyhow::bail!("Default engine '{}' is not configured", config.default_engine);
    }
    
    // Validate security settings
    if config.runtime.security.max_memory_bytes == Some(0) {
        anyhow::bail!("Invalid max_memory_bytes: cannot be zero");
    }
    
    if config.runtime.security.max_execution_time_ms == Some(0) {
        anyhow::bail!("Invalid max_execution_time_ms: cannot be zero");
    }
    
    // Validate runtime settings
    if config.runtime.max_concurrent_scripts == 0 {
        anyhow::bail!("Invalid max_concurrent_scripts: cannot be zero");
    }
    
    if config.runtime.script_timeout_seconds == 0 {
        anyhow::bail!("Invalid script_timeout_seconds: cannot be zero");
    }
    
    Ok(())
}