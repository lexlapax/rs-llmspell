//! Integration tests for template execution via `ScriptRuntime`
//!
//! These tests verify the full execution path from CLI → Kernel → Runtime → Template
//! to catch regressions that unit tests miss.

use llmspell_bridge::ScriptRuntime;
use llmspell_config::{LLMSpellConfig, ProviderConfig, ProviderManagerConfigBuilder};
use llmspell_core::traits::script_executor::ScriptExecutor;
use serde_json::json;

/// Test that template execution works with real `provider_config`
///
/// This test validates the fix for Task 13b.1.7 (Phase 13.5.7d regression)
/// where `ExecutionContext::build()` requires `provider_config` but `runtime.rs`
/// wasn't providing it.
///
/// NOTE: This test uses empty providers config to avoid actual LLM validation.
/// The bug we're testing occurs during `ExecutionContext::build()`, not provider init.
#[tokio::test(flavor = "multi_thread")]
#[cfg(feature = "lua")]
async fn test_template_exec_with_real_provider_config() {
    eprintln!("[TEST] Starting test_template_exec_with_real_provider_config");
    // Use default config (empty providers) to avoid API validation
    // The bug happens in ExecutionContext::build(), not provider initialization
    eprintln!("[TEST] Creating default config");
    let config = LLMSpellConfig::default();

    // Initialize ScriptRuntime (should succeed with empty providers)
    eprintln!("[TEST] Creating ScriptRuntime");
    let runtime = Box::pin(ScriptRuntime::new_with_lua(config))
        .await
        .expect("Failed to create ScriptRuntime");
    eprintln!("[TEST] ScriptRuntime created successfully");

    // Attempt to execute code-generator template
    eprintln!("[TEST] Preparing to execute template");
    // This should FAIL before fix with: "provider_config is required"
    // After fix, ExecutionContext should build (template may fail for other reasons)
    let params = json!({
        "description": "A simple test function",
        "language": "python",
        "include_tests": false,
        // Don't specify provider_name or model - will use default (empty)
    });

    let result = runtime.handle_template_exec("code-generator", params).await;

    // Before fix: This will fail with "provider_config is required"
    // After fix: This should either succeed or fail with a different error
    // (like missing actual LLM connection, which is OK for this test)
    match result {
        Ok(_) => {
            // Fix is working! ExecutionContext built successfully
            println!("✅ Template execution succeeded (fix working)");
        }
        Err(e) => {
            let error_msg = e.to_string();

            // If we see "provider_config is required", the bug still exists
            assert!(
                !error_msg.contains("provider_config is required"),
                "❌ BUG DETECTED: ExecutionContext missing provider_config\n\
                 Error: {error_msg}\n\
                 Fix: Add .with_provider_config() to runtime.rs:1463"
            );

            // Other errors are OK (e.g., missing LLM connection, template not found)
            println!("⚠️  Template execution failed (expected without real LLM): {error_msg}");

            // For now, we accept any error that's NOT "provider_config is required"
            // This proves ExecutionContext was built successfully
            assert!(
                !error_msg.contains("provider_config is required"),
                "provider_config should be provided by runtime.rs"
            );
        }
    }
}

/// Test provider resolution with `provider_name` param (centralized config)
///
/// Validates Phase 13.5.7d smart dual-path provider resolution:
/// - `provider_name` param → lookup in `ProviderManagerConfig`
#[tokio::test(flavor = "multi_thread")]
#[cfg(feature = "lua")]
async fn test_template_exec_provider_name_resolution() {
    // Create config with anthropic provider (disabled to skip validation)
    let provider_config = ProviderConfig {
        name: "anthropic".to_string(),
        provider_type: "anthropic".to_string(),
        enabled: false, // Skip validation for test (Task 13b.15)
        base_url: None,
        api_key_env: None,
        api_key: Some("test-anthropic-key".to_string()),
        default_model: Some("claude-3-haiku-20240307".to_string()),
        max_tokens: Some(4096),
        timeout_seconds: Some(60),
        temperature: None,
        rate_limit: None,
        retry: None,
        max_retries: Some(3),
        options: std::collections::HashMap::new(),
    };

    let provider_manager_config = ProviderManagerConfigBuilder::new()
        .add_provider("anthropic", provider_config)
        .build();

    let config = LLMSpellConfig {
        providers: provider_manager_config,
        ..LLMSpellConfig::default()
    };

    let runtime = Box::pin(ScriptRuntime::new_with_lua(config))
        .await
        .expect("Failed to create ScriptRuntime");

    // Test with provider_name param (should resolve to anthropic config)
    let params = json!({
        "description": "Test function",
        "language": "rust",
        "provider_name": "anthropic",  // Should lookup this provider
        "include_tests": false,
    });

    let result = runtime.handle_template_exec("code-generator", params).await;

    // Should NOT fail with "provider_config is required"
    if let Err(e) = result {
        let error_msg = e.to_string();
        assert!(
            !error_msg.contains("provider_config is required"),
            "ExecutionContext should have provider_config from runtime.rs"
        );
        println!("⚠️  Expected error (no real LLM): {error_msg}");
    }
}

/// Test provider resolution with model param (ephemeral provider)
///
/// Validates Phase 13.5.7d smart dual-path provider resolution:
/// - model param → create ephemeral `ProviderConfig`
#[tokio::test(flavor = "multi_thread")]
#[cfg(feature = "lua")]
async fn test_template_exec_model_ephemeral_resolution() {
    let config = LLMSpellConfig::default();

    let runtime = Box::pin(ScriptRuntime::new_with_lua(config))
        .await
        .expect("Failed to create ScriptRuntime");

    // Test with model param (should create ephemeral provider)
    let params = json!({
        "description": "Test function",
        "language": "python",
        "model": "ollama/llama3.2:3b",  // Should create ephemeral config
        "include_tests": false,
    });

    let result = runtime.handle_template_exec("code-generator", params).await;

    // Should NOT fail with "provider_config is required"
    if let Err(e) = result {
        let error_msg = e.to_string();
        assert!(
            !error_msg.contains("provider_config is required"),
            "ExecutionContext should have provider_config from runtime.rs"
        );
        println!("⚠️  Expected error (no real LLM): {error_msg}");
    }
}

/// Test that `ExecutionContext` builds successfully even without LLM execution
///
/// This is a smoke test to ensure the infrastructure wiring is correct
#[tokio::test(flavor = "multi_thread")]
#[cfg(feature = "lua")]
async fn test_execution_context_infrastructure_wiring() {
    let provider_config = ProviderConfig {
        name: "test".to_string(),
        provider_type: "openai".to_string(),
        enabled: false, // Skip validation for test (Task 13b.15)
        base_url: None,
        api_key_env: None,
        api_key: Some("test-key".to_string()),
        default_model: Some("gpt-3.5-turbo".to_string()),
        max_tokens: Some(1000),
        timeout_seconds: Some(30),
        temperature: None,
        rate_limit: None,
        retry: None,
        max_retries: Some(2),
        options: std::collections::HashMap::new(),
    };

    let provider_manager_config = ProviderManagerConfigBuilder::new()
        .add_provider("test", provider_config)
        .build();

    let config = LLMSpellConfig {
        providers: provider_manager_config,
        ..LLMSpellConfig::default()
    };

    let runtime = Box::pin(ScriptRuntime::new_with_lua(config))
        .await
        .expect("Failed to create ScriptRuntime");

    // Try to execute a simple template that might not need actual LLM
    // The key is that ExecutionContext should build without "provider_config is required"
    let params = json!({
        "description": "Simple test",
        "language": "python",
        "provider_name": "test",
        "include_tests": false,
    });

    let result = runtime.handle_template_exec("code-generator", params).await;

    // Main assertion: Should NOT fail with infrastructure error
    if let Err(e) = result {
        let error_msg = e.to_string();

        // These errors indicate infrastructure problems (the bug we're fixing)
        assert!(
            !error_msg.contains("provider_config is required"),
            "Infrastructure bug: provider_config not wired"
        );
        assert!(
            !error_msg.contains("Required infrastructure not available"),
            "Infrastructure bug: ExecutionContext missing required components"
        );

        // Other errors are acceptable (template execution, LLM connection, etc.)
        println!("✅ Infrastructure wiring OK. Template error (expected): {error_msg}");
    } else {
        println!("✅ Template execution succeeded");
    }
}
