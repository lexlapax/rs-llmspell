//! Integration tests for Jupyter protocol compatibility
//! Tests kernel lifecycle, connection handling, and protocol compliance

use anyhow::Result;
use llmspell_config::{DebugConfig, GlobalRuntimeConfig, KernelSettings, LLMSpellConfig};
use llmspell_kernel::kernel::{GenericKernel, KernelState};
use llmspell_kernel::ConnectionInfo;
use std::sync::Arc;

/// Test kernel creation with factory method
#[tokio::test(flavor = "multi_thread")]
async fn test_kernel_factory_creation() -> Result<()> {
    let kernel_id = "test-kernel-lifecycle".to_string();
    let config = Arc::new(
        LLMSpellConfig::builder()
            .default_engine("lua")
            .runtime(
                GlobalRuntimeConfig::builder()
                    .kernel(KernelSettings {
                        max_clients: 1,
                        auth_enabled: false,
                        ..Default::default()
                    })
                    .build(),
            )
            .build(),
    );

    let connection_info = ConnectionInfo::new(kernel_id.clone(), "127.0.0.1".to_string(), 9000);

    // Test factory method
    let result =
        GenericKernel::from_config_with_connection(kernel_id, config, connection_info).await;
    assert!(
        result.is_ok(),
        "Factory method should create kernel successfully"
    );

    let kernel = result?;
    // Kernel channels are already bound in new() - just verify it was created
    assert_eq!(kernel.kernel_id, "test-kernel-lifecycle");
    Ok(())
}

/// Test kernel state management
#[tokio::test(flavor = "multi_thread")]
async fn test_kernel_state_management() -> Result<()> {
    let kernel_id = "test-kernel-state".to_string();
    let config = Arc::new(
        LLMSpellConfig::builder()
            .default_engine("lua")
            .runtime(
                GlobalRuntimeConfig::builder()
                    .kernel(KernelSettings {
                        max_clients: 1,
                        auth_enabled: false,
                        ..Default::default()
                    })
                    .build(),
            )
            .build(),
    );

    let connection_info = ConnectionInfo::new(kernel_id.clone(), "127.0.0.1".to_string(), 9100);

    let kernel =
        GenericKernel::from_config_with_connection(kernel_id, config, connection_info).await?;

    // Test state transitions
    {
        let state = kernel.execution_state.read().await;
        assert_eq!(
            *state,
            KernelState::Starting,
            "Initial state should be Starting"
        );
        drop(state); // Explicitly drop to release lock early
    }

    // Transition to Idle
    {
        let mut state = kernel.execution_state.write().await;
        *state = KernelState::Idle;
    }

    {
        let state = kernel.execution_state.read().await;
        assert_eq!(*state, KernelState::Idle, "Should transition to Idle");
        drop(state); // Explicitly drop to release lock early
    }

    // Transition to Busy
    {
        let mut state = kernel.execution_state.write().await;
        *state = KernelState::Busy;
    }

    {
        let state = kernel.execution_state.read().await;
        assert_eq!(*state, KernelState::Busy, "Should transition to Busy");
        drop(state); // Explicitly drop to release lock early
    }

    // Transition back to Idle
    {
        let mut state = kernel.execution_state.write().await;
        *state = KernelState::Idle;
    }

    {
        let state = kernel.execution_state.read().await;
        assert_eq!(*state, KernelState::Idle, "Should transition back to Idle");
        drop(state); // Explicitly drop to release lock early
    }

    // Test shutdown state
    {
        let mut state = kernel.execution_state.write().await;
        *state = KernelState::Stopping;
    }

    {
        let state = kernel.execution_state.read().await;
        assert_eq!(*state, KernelState::Stopping, "Should be in Stopping state");
        drop(state); // Explicitly drop to release lock early
    }

    Ok(())
}

/// Test kernel with multiple client support
#[tokio::test(flavor = "multi_thread")]
async fn test_kernel_multiple_clients() -> Result<()> {
    let kernel1_id = "test-multi-client-1".to_string();
    let kernel2_id = "test-multi-client-2".to_string();

    let config1 = Arc::new(
        LLMSpellConfig::builder()
            .default_engine("lua")
            .runtime(
                GlobalRuntimeConfig::builder()
                    .kernel(KernelSettings {
                        max_clients: 3,
                        auth_enabled: false,
                        ..Default::default()
                    })
                    .build(),
            )
            .build(),
    );

    let config2 = Arc::new(
        LLMSpellConfig::builder()
            .default_engine("lua")
            .runtime(
                GlobalRuntimeConfig::builder()
                    .kernel(KernelSettings {
                        max_clients: 2,
                        auth_enabled: false,
                        ..Default::default()
                    })
                    .build(),
            )
            .build(),
    );

    let connection1 = ConnectionInfo::new(kernel1_id.clone(), "127.0.0.1".to_string(), 9200);

    let connection2 = ConnectionInfo::new(kernel2_id.clone(), "127.0.0.1".to_string(), 9300);

    let kernel1 =
        GenericKernel::from_config_with_connection(kernel1_id, config1, connection1).await?;
    let kernel2 =
        GenericKernel::from_config_with_connection(kernel2_id, config2, connection2).await?;

    // Verify independent kernel configurations
    assert_eq!(
        kernel1.config.runtime.kernel.max_clients, 3,
        "First kernel should support 3 clients"
    );
    assert_eq!(
        kernel2.config.runtime.kernel.max_clients, 2,
        "Second kernel should support 2 clients"
    );

    assert_eq!(
        kernel1.config.default_engine, "lua",
        "First kernel should use lua engine"
    );
    assert_eq!(
        kernel2.config.default_engine, "lua",
        "Second kernel should use lua engine"
    );

    Ok(())
}

/// Test kernel with debug mode enabled
#[tokio::test(flavor = "multi_thread")]
async fn test_kernel_debug_mode() -> Result<()> {
    let kernel_id = "test-debug-kernel".to_string();
    let config = Arc::new(
        LLMSpellConfig::builder()
            .default_engine("lua")
            .debug(DebugConfig {
                enabled: true,
                ..Default::default()
            })
            .runtime(
                GlobalRuntimeConfig::builder()
                    .kernel(KernelSettings {
                        max_clients: 1,
                        auth_enabled: false,
                        ..Default::default()
                    })
                    .build(),
            )
            .build(),
    );

    let connection_info = ConnectionInfo::new(kernel_id.clone(), "127.0.0.1".to_string(), 9400);

    let kernel =
        GenericKernel::from_config_with_connection(kernel_id, config, connection_info).await?;
    assert!(kernel.config.debug.enabled, "Debug mode should be enabled");
    Ok(())
}

/// Test kernel with authentication enabled
#[tokio::test(flavor = "multi_thread")]
async fn test_kernel_with_authentication() -> Result<()> {
    let kernel_id = "test-auth-kernel".to_string();
    let config = Arc::new(
        LLMSpellConfig::builder()
            .default_engine("lua")
            .runtime(
                GlobalRuntimeConfig::builder()
                    .kernel(KernelSettings {
                        max_clients: 1,
                        auth_enabled: true,
                        ..Default::default()
                    })
                    .build(),
            )
            .build(),
    );

    let connection_info = ConnectionInfo::new(kernel_id.clone(), "127.0.0.1".to_string(), 9500);

    let kernel =
        GenericKernel::from_config_with_connection(kernel_id, config, connection_info).await?;
    assert!(
        kernel.config.runtime.kernel.auth_enabled,
        "Authentication should be enabled"
    );
    assert!(
        kernel.config.runtime.kernel.auth_enabled,
        "Kernel config should have auth enabled"
    );
    Ok(())
}

/// Test invalid engine configuration
#[tokio::test(flavor = "multi_thread")]
async fn test_invalid_engine_config() -> Result<()> {
    let kernel_id = "test-invalid-engine".to_string();
    let invalid_config = Arc::new(
        LLMSpellConfig::builder()
            .default_engine("invalid_engine")
            .runtime(
                GlobalRuntimeConfig::builder()
                    .kernel(KernelSettings {
                        max_clients: 1,
                        auth_enabled: false,
                        ..Default::default()
                    })
                    .build(),
            )
            .build(),
    );

    let connection_info = ConnectionInfo::new(kernel_id.clone(), "127.0.0.1".to_string(), 9600);

    let result =
        GenericKernel::from_config_with_connection(kernel_id, invalid_config, connection_info)
            .await;

    // Invalid engine should cause creation to fail
    assert!(
        result.is_err(),
        "Kernel creation should fail with invalid engine"
    );

    if let Err(e) = result {
        // Verify we get a reasonable error message
        let error_msg = e.to_string();
        assert!(
            error_msg.contains("engine") || error_msg.contains("runtime"),
            "Error should mention engine or runtime issue"
        );
    }

    Ok(())
}

/// Test kernel resource limits configuration
#[tokio::test(flavor = "multi_thread")]
async fn test_kernel_resource_limits() -> Result<()> {
    let kernel_id = "test-resource-limits".to_string();
    let config = Arc::new(
        LLMSpellConfig::builder()
            .default_engine("lua")
            .runtime(
                GlobalRuntimeConfig::builder()
                    .kernel(KernelSettings {
                        max_clients: 2,
                        auth_enabled: false,
                        heartbeat_interval_ms: 500,
                        shutdown_timeout_seconds: 5,
                        ..Default::default()
                    })
                    .build(),
            )
            .build(),
    );

    let connection_info = ConnectionInfo::new(kernel_id.clone(), "127.0.0.1".to_string(), 9700);

    let kernel =
        GenericKernel::from_config_with_connection(kernel_id, config, connection_info).await?;
    assert_eq!(
        kernel.config.runtime.kernel.max_clients, 2,
        "Max clients should be configured correctly"
    );
    assert_eq!(
        kernel.config.runtime.kernel.heartbeat_interval_ms, 500,
        "Heartbeat interval should be configured"
    );
    assert_eq!(
        kernel.config.runtime.kernel.shutdown_timeout_seconds, 5,
        "Shutdown timeout should be configured"
    );
    Ok(())
}
