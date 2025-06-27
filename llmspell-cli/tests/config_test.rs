//! ABOUTME: Tests for configuration loading and validation
//! ABOUTME: Verifies configuration discovery, parsing, and environment overrides

use llmspell_cli::config::{load_runtime_config, create_default_config, validate_config};
use tempfile::tempdir;
use std::env;
use std::fs;

#[tokio::test]
async fn test_default_config() {
    // Save and clear any existing env vars that might interfere
    let saved_engine = env::var("LLMSPELL_DEFAULT_ENGINE").ok();
    env::remove_var("LLMSPELL_DEFAULT_ENGINE");
    
    // Load default configuration
    let config = load_runtime_config(None).await.unwrap();
    
    assert_eq!(config.default_engine, "lua");
    assert!(config.runtime.enable_streaming);
    assert_eq!(config.runtime.script_timeout_seconds, 300);
    assert_eq!(config.runtime.security.max_memory_bytes, Some(50_000_000));
    
    // Restore env var if it existed
    if let Some(val) = saved_engine {
        env::set_var("LLMSPELL_DEFAULT_ENGINE", val);
    }
}

#[tokio::test]
async fn test_create_config_file() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("test.toml");
    
    // Create default config
    create_default_config(&config_path).await.unwrap();
    
    // Verify file exists and is valid TOML
    assert!(config_path.exists());
    let content = fs::read_to_string(&config_path).unwrap();
    assert!(content.contains("default_engine"));
    assert!(content.contains("lua"));
    
    // Load the created config
    let config = load_runtime_config(Some(&config_path)).await.unwrap();
    assert_eq!(config.default_engine, "lua");
}

#[tokio::test]
async fn test_environment_overrides() {
    // Set environment variables
    env::set_var("LLMSPELL_DEFAULT_ENGINE", "javascript");
    env::set_var("LLMSPELL_SCRIPT_TIMEOUT", "600");
    env::set_var("LLMSPELL_ENABLE_STREAMING", "false");
    env::set_var("LLMSPELL_ALLOW_FILE_ACCESS", "true");
    env::set_var("LLMSPELL_MAX_MEMORY_MB", "100");
    
    // Load config with environment overrides
    let config = load_runtime_config(None).await.unwrap();
    
    assert_eq!(config.default_engine, "javascript");
    assert_eq!(config.runtime.script_timeout_seconds, 600);
    assert!(!config.runtime.enable_streaming);
    assert!(config.runtime.security.allow_file_access);
    assert_eq!(config.runtime.security.max_memory_bytes, Some(100 * 1024 * 1024));
    
    // Clean up
    env::remove_var("LLMSPELL_DEFAULT_ENGINE");
    env::remove_var("LLMSPELL_SCRIPT_TIMEOUT");
    env::remove_var("LLMSPELL_ENABLE_STREAMING");
    env::remove_var("LLMSPELL_ALLOW_FILE_ACCESS");
    env::remove_var("LLMSPELL_MAX_MEMORY_MB");
}

#[tokio::test]
async fn test_config_discovery() {
    let dir = tempdir().unwrap();
    let original_dir = env::current_dir().unwrap();
    
    // Change to temp directory
    env::set_current_dir(&dir).unwrap();
    
    // Create config file in current directory
    let config_path = dir.path().join("llmspell.toml");
    create_default_config(&config_path).await.unwrap();
    
    // Should discover the config file
    let config = load_runtime_config(None).await.unwrap();
    assert_eq!(config.default_engine, "lua");
    
    // Clean up
    env::set_current_dir(original_dir).unwrap();
}

#[tokio::test]
async fn test_validate_config() {
    let config = load_runtime_config(None).await.unwrap();
    
    // Default config should be valid
    validate_config(&config).unwrap();
}

#[tokio::test]
async fn test_invalid_config_validation() {
    let mut config = load_runtime_config(None).await.unwrap();
    
    // Make config invalid
    config.default_engine = "nonexistent".to_string();
    
    // Should fail validation
    let result = validate_config(&config);
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not configured"));
}

#[tokio::test]
async fn test_missing_config_file() {
    let dir = tempdir().unwrap();
    let nonexistent = dir.path().join("nonexistent.toml");
    
    // Should fail with clear error
    let result = load_runtime_config(Some(&nonexistent)).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("not found"));
}

#[tokio::test]
async fn test_malformed_config_file() {
    let dir = tempdir().unwrap();
    let config_path = dir.path().join("bad.toml");
    
    // Write invalid TOML
    fs::write(&config_path, "this is not valid toml!").unwrap();
    
    // Should fail with parse error
    let result = load_runtime_config(Some(&config_path)).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Failed to parse"));
}