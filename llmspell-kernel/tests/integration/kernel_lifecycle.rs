//! Integration tests for kernel lifecycle management
//! Tests kernel state transitions, startup, and shutdown

use anyhow::Result;
use llmspell_config::{GlobalRuntimeConfig, KernelSettings, LLMSpellConfig};
use llmspell_kernel::kernel::{GenericKernel, KernelState};
use llmspell_kernel::ConnectionInfo;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

/// Test basic kernel creation and initial state
#[tokio::test(flavor = "multi_thread")]
async fn test_kernel_creation() -> Result<()> {
    let kernel_id = "test-kernel-creation".to_string();
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

    let connection_info = ConnectionInfo::new(kernel_id.clone(), "127.0.0.1".to_string(), 8000);

    let kernel =
        GenericKernel::from_config_with_connection(kernel_id, config, connection_info).await?;

    // Verify kernel was created with correct ID and initial state
    assert_eq!(kernel.kernel_id, "test-kernel-creation");
    assert_eq!(
        *kernel.execution_state.read().await,
        KernelState::Starting,
        "Kernel should start in Starting state"
    );
    Ok(())
}

/// Test kernel shutdown
#[tokio::test(flavor = "multi_thread")]
async fn test_kernel_shutdown() -> Result<()> {
    let kernel_id = "test-kernel-shutdown".to_string();
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

    let connection_info = ConnectionInfo::new(kernel_id.clone(), "127.0.0.1".to_string(), 8100);

    let kernel =
        GenericKernel::from_config_with_connection(kernel_id, config, connection_info).await?;

    // Test shutdown
    let shutdown_result = timeout(Duration::from_secs(5), kernel.shutdown()).await;
    assert!(
        shutdown_result.is_ok(),
        "Shutdown should complete within timeout"
    );
    assert!(shutdown_result?.is_ok(), "Shutdown should succeed");
    Ok(())
}

/// Test kernel state transitions
#[tokio::test(flavor = "multi_thread")]
async fn test_kernel_state_transitions() -> Result<()> {
    let kernel_id = "test-state-transitions".to_string();
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

    let connection_info = ConnectionInfo::new(kernel_id.clone(), "127.0.0.1".to_string(), 8200);

    let kernel =
        GenericKernel::from_config_with_connection(kernel_id, config, connection_info).await?;

    // Verify initial state
    assert_eq!(
        *kernel.execution_state.read().await,
        KernelState::Starting,
        "Initial state should be Starting"
    );

    // Transition to idle
    {
        let mut state = kernel.execution_state.write().await;
        *state = KernelState::Idle;
    }

    assert_eq!(
        *kernel.execution_state.read().await,
        KernelState::Idle,
        "Kernel should transition to Idle"
    );

    // Transition to busy
    {
        let mut state = kernel.execution_state.write().await;
        *state = KernelState::Busy;
    }

    assert_eq!(
        *kernel.execution_state.read().await,
        KernelState::Busy,
        "Kernel should transition to Busy"
    );
    Ok(())
}

/// Test kernel configuration properties
#[tokio::test(flavor = "multi_thread")]
async fn test_kernel_configuration() -> Result<()> {
    let kernel_id = "test-kernel-config".to_string();
    let config = Arc::new(
        LLMSpellConfig::builder()
            .default_engine("lua")
            .debug(llmspell_config::DebugConfig {
                enabled: true,
                ..Default::default()
            })
            .runtime(
                GlobalRuntimeConfig::builder()
                    .kernel(KernelSettings {
                        max_clients: 5,
                        auth_enabled: true,
                        ..Default::default()
                    })
                    .build(),
            )
            .build(),
    );

    let connection_info = ConnectionInfo::new(kernel_id.clone(), "127.0.0.1".to_string(), 8300);

    let kernel =
        GenericKernel::from_config_with_connection(kernel_id, config, connection_info).await?;

    // Verify configuration is preserved
    assert_eq!(kernel.config.default_engine, "lua");
    assert!(kernel.config.debug.enabled, "Debug should be enabled");
    assert_eq!(
        kernel.config.runtime.kernel.max_clients, 5,
        "Max clients should be 5"
    );
    assert!(
        kernel.config.runtime.kernel.auth_enabled,
        "Auth should be enabled"
    );
    Ok(())
}

/// Test multiple kernel instances with different ports
#[tokio::test(flavor = "multi_thread")]
async fn test_multiple_kernel_instances() -> Result<()> {
    let kernel1_id = "test-kernel-multi-1".to_string();
    let kernel2_id = "test-kernel-multi-2".to_string();

    let config1 = Arc::new(
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

    let config2 = Arc::new(
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

    let connection1 = ConnectionInfo::new(kernel1_id.clone(), "127.0.0.1".to_string(), 8400);

    let connection2 = ConnectionInfo::new(kernel2_id.clone(), "127.0.0.1".to_string(), 8500);

    // Create both kernels
    let kernel1 =
        GenericKernel::from_config_with_connection(kernel1_id, config1, connection1).await?;
    let kernel2 =
        GenericKernel::from_config_with_connection(kernel2_id, config2, connection2).await?;

    // Verify both kernels are created independently
    assert_eq!(kernel1.kernel_id, "test-kernel-multi-1");
    assert_eq!(kernel2.kernel_id, "test-kernel-multi-2");

    assert_eq!(
        *kernel1.execution_state.read().await,
        KernelState::Starting,
        "First kernel should be starting"
    );
    assert_eq!(
        *kernel2.execution_state.read().await,
        KernelState::Starting,
        "Second kernel should be starting"
    );
    Ok(())
}

/// Test kernel execution counter
#[tokio::test(flavor = "multi_thread")]
async fn test_kernel_execution_counter() -> Result<()> {
    let kernel_id = "test-kernel-counter".to_string();
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

    let connection_info = ConnectionInfo::new(kernel_id.clone(), "127.0.0.1".to_string(), 8600);

    let kernel =
        GenericKernel::from_config_with_connection(kernel_id, config, connection_info).await?;

    // Verify initial count is zero
    assert_eq!(
        *kernel.execution_count.lock().await,
        0,
        "Initial count should be 0"
    );

    // Increment counter manually (normally done during execution)
    {
        let mut count = kernel.execution_count.lock().await;
        *count += 1;
    }

    assert_eq!(*kernel.execution_count.lock().await, 1, "Count should be 1");
    Ok(())
}

/// Test kernel with authentication enabled
#[tokio::test(flavor = "multi_thread")]
async fn test_kernel_with_auth() -> Result<()> {
    let kernel_id = "test-kernel-auth".to_string();
    let config = Arc::new(
        LLMSpellConfig::builder()
            .default_engine("lua")
            .runtime(
                GlobalRuntimeConfig::builder()
                    .kernel(KernelSettings {
                        max_clients: 2,
                        auth_enabled: true,
                        ..Default::default()
                    })
                    .build(),
            )
            .build(),
    );

    let connection_info = ConnectionInfo::new(kernel_id.clone(), "127.0.0.1".to_string(), 8700);

    let kernel =
        GenericKernel::from_config_with_connection(kernel_id, config, connection_info).await?;

    // Verify auth is enabled through config
    assert!(
        kernel.config.runtime.kernel.auth_enabled,
        "Auth should be enabled"
    );
    Ok(())
}

/// Test invalid configuration (should compile but runtime may handle)
#[tokio::test(flavor = "multi_thread")]
async fn test_kernel_with_invalid_config() -> Result<()> {
    let kernel_id = "test-invalid-config".to_string();
    let config = Arc::new(
        LLMSpellConfig::builder()
            .default_engine("") // Empty engine name
            .runtime(
                GlobalRuntimeConfig::builder()
                    .kernel(KernelSettings {
                        max_clients: 0, // Zero clients
                        auth_enabled: false,
                        ..Default::default()
                    })
                    .build(),
            )
            .build(),
    );

    let connection_info = ConnectionInfo::new(kernel_id.clone(), "127.0.0.1".to_string(), 8800);

    // This might fail but we're testing configuration handling
    let result =
        GenericKernel::from_config_with_connection(kernel_id, config, connection_info).await;

    // The kernel creation might fail with empty engine name
    // Just verify we can handle the error gracefully
    if result.is_err() {
        // Expected - empty engine name should fail
        println!("Kernel creation failed as expected with invalid config");
    }
    Ok(())
}

/// Test kernel `from_config` factory method
#[tokio::test(flavor = "multi_thread")]
async fn test_kernel_from_config_factory() -> Result<()> {
    let kernel_id = Some("test-factory".to_string());
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

    // Use factory method that creates default connection info
    let result = GenericKernel::from_config(kernel_id, config).await;

    // Should succeed and create kernel with auto-generated connection info
    assert!(result.is_ok(), "Factory method should succeed");

    let kernel = result?;
    assert_eq!(kernel.config.default_engine, "lua");
    Ok(())
}
