//! Integration tests for kernel lifecycle management
//! Tests kernel state transitions, startup, and shutdown

use anyhow::Result;
use llmspell_config::{GlobalRuntimeConfig, KernelSettings, LLMSpellConfig};
use llmspell_kernel::kernel::{GenericKernel, KernelState};
use llmspell_kernel::ConnectionInfo;
use llmspell_testing::kernel_helpers::create_test_kernel_config;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

/// Test basic kernel creation and initial state
#[tokio::test(flavor = "multi_thread")]
async fn test_kernel_creation() -> Result<()> {
    let (kernel_id, port) = create_test_kernel_config().await?;
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

    let connection_info = ConnectionInfo::new(kernel_id.clone(), "127.0.0.1".to_string(), port);

    let kernel =
        GenericKernel::from_config_with_connection(kernel_id.clone(), config, connection_info)
            .await?;

    // Verify kernel was created with correct ID and initial state
    assert!(kernel.kernel_id.starts_with("test-kernel-"));
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
    let (kernel_id, port) = create_test_kernel_config().await?;
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

    let connection_info = ConnectionInfo::new(kernel_id.clone(), "127.0.0.1".to_string(), port);

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
    let (kernel_id, port) = create_test_kernel_config().await?;
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

    let connection_info = ConnectionInfo::new(kernel_id.clone(), "127.0.0.1".to_string(), port);

    let kernel =
        GenericKernel::from_config_with_connection(kernel_id, config, connection_info).await?;

    // Verify initial state
    assert_eq!(
        *kernel.execution_state.read().await,
        KernelState::Starting,
        "Kernel should start in Starting state"
    );

    // Transition to Idle
    {
        let mut state = kernel.execution_state.write().await;
        *state = KernelState::Idle;
    }
    assert_eq!(
        *kernel.execution_state.read().await,
        KernelState::Idle,
        "Kernel should transition to Idle"
    );

    // Transition to Busy
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

/// Test kernel configuration settings
#[tokio::test(flavor = "multi_thread")]
async fn test_kernel_configuration() -> Result<()> {
    let (kernel_id, port) = create_test_kernel_config().await?;
    let config = Arc::new(
        LLMSpellConfig::builder()
            .default_engine("lua")
            .runtime(
                GlobalRuntimeConfig::builder()
                    .kernel(KernelSettings {
                        max_clients: 3,
                        auth_enabled: true,
                        ..Default::default()
                    })
                    .build(),
            )
            .build(),
    );

    let connection_info = ConnectionInfo::new(kernel_id.clone(), "127.0.0.1".to_string(), port);

    let kernel =
        GenericKernel::from_config_with_connection(kernel_id, config.clone(), connection_info)
            .await?;

    // Verify configuration was applied
    assert_eq!(
        config.runtime.kernel.max_clients, 3,
        "Max clients should be configured"
    );
    assert!(config.runtime.kernel.auth_enabled, "Auth should be enabled");

    // Verify kernel has correct configuration
    assert_eq!(kernel.config.runtime.kernel.max_clients, 3);
    assert!(kernel.config.runtime.kernel.auth_enabled);

    Ok(())
}

/// Test multiple kernel instances with different ports
#[tokio::test(flavor = "multi_thread")]
async fn test_multiple_kernel_instances() -> Result<()> {
    let (kernel1_id, port1) = create_test_kernel_config().await?;
    let (kernel2_id, port2) = create_test_kernel_config().await?;

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
            .default_engine("lua") // Use lua for both, they have different configs anyway
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

    let connection1 = ConnectionInfo::new(kernel1_id.clone(), "127.0.0.1".to_string(), port1);
    let connection2 = ConnectionInfo::new(kernel2_id.clone(), "127.0.0.1".to_string(), port2);

    let kernel1 =
        GenericKernel::from_config_with_connection(kernel1_id.clone(), config1, connection1)
            .await?;
    let kernel2 =
        GenericKernel::from_config_with_connection(kernel2_id.clone(), config2, connection2)
            .await?;

    // Verify both kernels were created with different IDs
    assert_ne!(kernel1.kernel_id, kernel2.kernel_id);
    assert!(kernel1.kernel_id.starts_with("test-kernel-"));
    assert!(kernel2.kernel_id.starts_with("test-kernel-"));

    // Verify different configurations (both use lua, but with different settings)
    assert_eq!(kernel1.config.default_engine, "lua");
    assert_eq!(kernel2.config.default_engine, "lua");
    assert_eq!(kernel1.config.runtime.kernel.max_clients, 1);
    assert_eq!(kernel2.config.runtime.kernel.max_clients, 2);
    assert!(!kernel1.config.runtime.kernel.auth_enabled);
    assert!(kernel2.config.runtime.kernel.auth_enabled);

    Ok(())
}

/// Test kernel execution counter
#[tokio::test(flavor = "multi_thread")]
async fn test_kernel_execution_counter() -> Result<()> {
    let (kernel_id, port) = create_test_kernel_config().await?;
    let config = Arc::new(
        LLMSpellConfig::builder()
            .default_engine("lua")
            .runtime(
                GlobalRuntimeConfig::builder()
                    .kernel(KernelSettings::default())
                    .build(),
            )
            .build(),
    );

    let connection_info = ConnectionInfo::new(kernel_id.clone(), "127.0.0.1".to_string(), port);

    let kernel =
        GenericKernel::from_config_with_connection(kernel_id, config, connection_info).await?;

    // Verify initial execution counter
    assert_eq!(
        *kernel.execution_count.lock().await,
        0,
        "Execution counter should start at 0"
    );

    // Increment counter
    {
        let mut count = kernel.execution_count.lock().await;
        *count += 1;
    }
    assert_eq!(
        *kernel.execution_count.lock().await,
        1,
        "Execution counter should increment"
    );

    Ok(())
}

/// Test kernel with authentication enabled
#[tokio::test(flavor = "multi_thread")]
async fn test_kernel_with_auth() -> Result<()> {
    let (kernel_id, port) = create_test_kernel_config().await?;
    let config = Arc::new(
        LLMSpellConfig::builder()
            .default_engine("lua")
            .runtime(
                GlobalRuntimeConfig::builder()
                    .kernel(KernelSettings {
                        auth_enabled: true,
                        ..Default::default()
                    })
                    .build(),
            )
            .build(),
    );

    let connection_info = ConnectionInfo::new(kernel_id.clone(), "127.0.0.1".to_string(), port);

    let kernel =
        GenericKernel::from_config_with_connection(kernel_id, config, connection_info).await?;

    assert!(kernel.config.runtime.kernel.auth_enabled);

    Ok(())
}

/// Test kernel creation with invalid configuration
#[tokio::test(flavor = "multi_thread")]
async fn test_kernel_with_invalid_config() -> Result<()> {
    let (kernel_id, port) = create_test_kernel_config().await?;

    // Create config with invalid engine
    let invalid_config = Arc::new(
        LLMSpellConfig::builder()
            .default_engine("invalid_engine_xyz")
            .build(),
    );

    let connection_info = ConnectionInfo::new(kernel_id.clone(), "127.0.0.1".to_string(), port);

    // This should fail because the engine is invalid
    let result =
        GenericKernel::from_config_with_connection(kernel_id, invalid_config, connection_info)
            .await;

    // The kernel creation should fail with an invalid engine
    assert!(
        result.is_err(),
        "Kernel creation should fail with invalid engine"
    );

    Ok(())
}

/// Test kernel creation via factory method
#[tokio::test(flavor = "multi_thread")]
async fn test_kernel_from_config_factory() -> Result<()> {
    let (kernel_id, _port) = create_test_kernel_config().await?;
    let config = Arc::new(LLMSpellConfig::default());

    // This should work without explicit connection info
    let result = GenericKernel::from_config(Some(kernel_id.clone()), config).await;

    // Should succeed with auto-generated connection info
    assert!(result.is_ok());

    let kernel = result?;
    assert!(kernel.kernel_id.starts_with("test-kernel-"));

    Ok(())
}
