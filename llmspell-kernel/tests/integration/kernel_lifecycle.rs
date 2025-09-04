//! Integration tests for kernel lifecycle management
//! Tests kernel state transitions, startup, and shutdown

use anyhow::Result;
use llmspell_config::LLMSpellConfig;
use llmspell_kernel::kernel::{GenericKernel, KernelConfig, KernelState};
use llmspell_kernel::ConnectionInfo;
use std::time::Duration;
use tokio::time::timeout;

/// Test basic kernel creation and initial state
#[tokio::test(flavor = "multi_thread")]
async fn test_kernel_creation() -> Result<()> {
    let kernel_config = KernelConfig {
        kernel_id: Some("test-kernel-creation".to_string()),
        engine: "lua".to_string(),
        runtime_config: LLMSpellConfig::default(),
        debug_enabled: false,
        max_clients: 1,
        auth_enabled: false,
        state_dir: None,
    };

    let connection_info = ConnectionInfo::new(
        "test-kernel-creation".to_string(),
        "127.0.0.1".to_string(),
        8000,
    );

    let kernel = GenericKernel::from_config_with_connection(kernel_config, connection_info).await?;

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
    let kernel_config = KernelConfig {
        kernel_id: Some("test-kernel-shutdown".to_string()),
        engine: "lua".to_string(),
        runtime_config: LLMSpellConfig::default(),
        debug_enabled: false,
        max_clients: 1,
        auth_enabled: false,
        state_dir: None,
    };

    let connection_info = ConnectionInfo::new(
        "test-kernel-shutdown".to_string(),
        "127.0.0.1".to_string(),
        8100,
    );

    let kernel = GenericKernel::from_config_with_connection(kernel_config, connection_info).await?;

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
    let kernel_config = KernelConfig {
        kernel_id: Some("test-kernel-states".to_string()),
        engine: "lua".to_string(),
        runtime_config: LLMSpellConfig::default(),
        debug_enabled: false,
        max_clients: 1,
        auth_enabled: false,
        state_dir: None,
    };

    let connection_info = ConnectionInfo::new(
        "test-kernel-states".to_string(),
        "127.0.0.1".to_string(),
        8200,
    );

    let kernel = GenericKernel::from_config_with_connection(kernel_config, connection_info).await?;

    // Initial state should be Starting
    let initial_state = kernel.execution_state.read().await;
    assert_eq!(
        *initial_state,
        KernelState::Starting,
        "Kernel should start in Starting state"
    );
    drop(initial_state);

    // Simulate setting state to Idle (this would normally happen in serve())
    {
        let mut state = kernel.execution_state.write().await;
        *state = KernelState::Idle;
    }

    let idle_state = kernel.execution_state.read().await;
    assert_eq!(
        *idle_state,
        KernelState::Idle,
        "Kernel should transition to Idle"
    );
    drop(idle_state);

    // Simulate setting state to Busy
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
    let kernel_config = KernelConfig {
        kernel_id: Some("test-kernel-config".to_string()),
        engine: "lua".to_string(),
        runtime_config: LLMSpellConfig::default(),
        debug_enabled: true,
        max_clients: 5,
        auth_enabled: true,
        state_dir: None,
    };

    let connection_info = ConnectionInfo::new(
        "test-kernel-config".to_string(),
        "127.0.0.1".to_string(),
        8300,
    );

    let kernel = GenericKernel::from_config_with_connection(kernel_config, connection_info).await?;

    // Verify configuration is preserved
    assert_eq!(kernel.config.engine, "lua");
    assert!(kernel.config.debug_enabled, "Debug should be enabled");
    assert_eq!(kernel.config.max_clients, 5, "Max clients should be 5");
    assert!(kernel.config.auth_enabled, "Auth should be enabled");
    Ok(())
}

/// Test multiple kernel instances with different ports
#[tokio::test(flavor = "multi_thread")]
async fn test_multiple_kernel_instances() -> Result<()> {
    let kernel1_config = KernelConfig {
        kernel_id: Some("test-kernel-multi-1".to_string()),
        engine: "lua".to_string(),
        runtime_config: LLMSpellConfig::default(),
        debug_enabled: false,
        max_clients: 1,
        auth_enabled: false,
        state_dir: None,
    };

    let kernel2_config = KernelConfig {
        kernel_id: Some("test-kernel-multi-2".to_string()),
        engine: "lua".to_string(),
        runtime_config: LLMSpellConfig::default(),
        debug_enabled: false,
        max_clients: 1,
        auth_enabled: false,
        state_dir: None,
    };

    let connection1 = ConnectionInfo::new(
        "test-kernel-multi-1".to_string(),
        "127.0.0.1".to_string(),
        8400,
    );

    let connection2 = ConnectionInfo::new(
        "test-kernel-multi-2".to_string(),
        "127.0.0.1".to_string(),
        8500,
    );

    // Create both kernels
    let kernel1 = GenericKernel::from_config_with_connection(kernel1_config, connection1).await?;
    let kernel2 = GenericKernel::from_config_with_connection(kernel2_config, connection2).await?;

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
    let kernel_config = KernelConfig {
        kernel_id: Some("test-kernel-counter".to_string()),
        engine: "lua".to_string(),
        runtime_config: LLMSpellConfig::default(),
        debug_enabled: false,
        max_clients: 1,
        auth_enabled: false,
        state_dir: None,
    };

    let connection_info = ConnectionInfo::new(
        "test-kernel-counter".to_string(),
        "127.0.0.1".to_string(),
        8600,
    );

    let kernel = GenericKernel::from_config_with_connection(kernel_config, connection_info).await?;

    // Verify execution counter starts at 0
    let initial_count = kernel.execution_count.lock().await;
    assert_eq!(*initial_count, 0, "Execution count should start at 0");
    drop(initial_count);

    // Simulate incrementing execution count
    {
        let mut count = kernel.execution_count.lock().await;
        *count += 1;
    }

    assert_eq!(
        *kernel.execution_count.lock().await,
        1,
        "Execution count should increment"
    );
    Ok(())
}

/// Test kernel resource limits
#[tokio::test(flavor = "multi_thread")]
async fn test_kernel_resource_limits() -> Result<()> {
    let kernel_config = KernelConfig {
        kernel_id: Some("test-kernel-limits".to_string()),
        engine: "lua".to_string(),
        runtime_config: LLMSpellConfig::default(),
        debug_enabled: false,
        max_clients: 1,
        auth_enabled: false,
        state_dir: None,
    };

    let connection_info = ConnectionInfo::new(
        "test-kernel-limits".to_string(),
        "127.0.0.1".to_string(),
        8700,
    );

    let kernel = GenericKernel::from_config_with_connection(kernel_config, connection_info).await?;

    // Verify resource limits have default values
    assert_eq!(
        kernel.resource_limits.max_execution_time_ms, 30000,
        "Default execution timeout should be 30 seconds"
    );
    assert_eq!(
        kernel.resource_limits.max_memory_bytes,
        100 * 1024 * 1024,
        "Default memory limit should be 100MB"
    );
    assert_eq!(
        kernel.resource_limits.max_concurrent_executions, 5,
        "Default concurrent executions should be 5"
    );
    Ok(())
}

/// Test kernel with invalid engine
#[tokio::test(flavor = "multi_thread")]
async fn test_kernel_invalid_engine() -> Result<()> {
    let kernel_config = KernelConfig {
        kernel_id: Some("test-kernel-invalid".to_string()),
        engine: "nonexistent_engine".to_string(),
        runtime_config: LLMSpellConfig::default(),
        debug_enabled: false,
        max_clients: 1,
        auth_enabled: false,
        state_dir: None,
    };

    let connection_info = ConnectionInfo::new(
        "test-kernel-invalid".to_string(),
        "127.0.0.1".to_string(),
        8800,
    );

    let result = GenericKernel::from_config_with_connection(kernel_config, connection_info).await;
    assert!(
        result.is_err(),
        "Kernel creation with invalid engine should fail"
    );
    Ok(())
}

/// Test kernel factory method from config
#[tokio::test(flavor = "multi_thread")]
async fn test_kernel_from_config() -> Result<()> {
    let kernel_config = KernelConfig {
        kernel_id: Some("test-kernel-factory".to_string()),
        engine: "lua".to_string(),
        runtime_config: LLMSpellConfig::default(),
        debug_enabled: false,
        max_clients: 1,
        auth_enabled: false,
        state_dir: None,
    };

    let result = GenericKernel::from_config(kernel_config).await;
    assert!(result.is_ok(), "Kernel factory method should succeed");

    let kernel = result?;
    assert_eq!(kernel.kernel_id, "test-kernel-factory");
    assert_eq!(kernel.config.engine, "lua");
    Ok(())
}
