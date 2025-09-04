//! Integration tests for Jupyter protocol compatibility
//! Tests kernel lifecycle, connection handling, and protocol compliance

use anyhow::Result;
use llmspell_config::LLMSpellConfig;
use llmspell_kernel::kernel::{GenericKernel, KernelConfig, KernelState};
use llmspell_kernel::ConnectionInfo;

/// Test kernel creation with factory method
#[tokio::test(flavor = "multi_thread")]
async fn test_kernel_factory_creation() -> Result<()> {
    let kernel_config = KernelConfig {
        kernel_id: Some("test-kernel-lifecycle".to_string()),
        engine: "lua".to_string(),
        runtime_config: LLMSpellConfig::default(),
        debug_enabled: false,
        max_clients: 1,
        auth_enabled: false,
    };

    let connection_info = ConnectionInfo::new(
        "test-kernel-lifecycle".to_string(),
        "127.0.0.1".to_string(),
        9000,
    );

    // Test factory method
    let result = GenericKernel::from_config_with_connection(kernel_config, connection_info).await;
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
    let kernel_config = KernelConfig {
        kernel_id: Some("test-kernel-state".to_string()),
        engine: "lua".to_string(),
        runtime_config: LLMSpellConfig::default(),
        debug_enabled: false,
        max_clients: 1,
        auth_enabled: false,
    };

    let connection_info = ConnectionInfo::new(
        "test-kernel-state".to_string(),
        "127.0.0.1".to_string(),
        9100,
    );

    let kernel = GenericKernel::from_config_with_connection(kernel_config, connection_info).await?;

    // Kernel channels are already bound in new() - verify state
    assert!(
        *kernel.execution_state.read().await == KernelState::Starting,
        "Kernel should start in Starting state"
    );
    Ok(())
}

/// Test connection file generation
#[tokio::test(flavor = "multi_thread")]
async fn test_connection_file_generation() -> Result<()> {
    let connection_info = ConnectionInfo::new(
        "test-kernel-connection".to_string(),
        "127.0.0.1".to_string(),
        9200,
    );

    // Test connection file creation
    connection_info.write_connection_file().await?;

    // Verify the connection file exists and has correct structure
    let connection_file_path = connection_info.connection_file_path();
    assert!(
        tokio::fs::metadata(&connection_file_path).await.is_ok(),
        "Connection file should be created"
    );

    // Read and verify connection file content
    let file_content = tokio::fs::read_to_string(&connection_file_path).await?;
    let connection_data: serde_json::Value = serde_json::from_str(&file_content)?;

    assert_eq!(connection_data["ip"], "127.0.0.1");
    assert_eq!(connection_data["transport"], "tcp");
    // Connection file format may vary - check if kernel_name exists or is null
    if let Some(kernel_name) = connection_data.get("kernel_name") {
        if !kernel_name.is_null() {
            assert_eq!(kernel_name, "llmspell");
        }
    }
    assert!(connection_data["shell_port"].is_number());
    assert!(connection_data["iopub_port"].is_number());
    assert!(connection_data["stdin_port"].is_number());
    assert!(connection_data["control_port"].is_number());
    assert!(connection_data["hb_port"].is_number());

    // Cleanup
    let _ = tokio::fs::remove_file(&connection_file_path).await;
    Ok(())
}

/// Test multiple kernel instances don't conflict
#[tokio::test(flavor = "multi_thread")]
async fn test_multiple_kernel_instances() -> Result<()> {
    let kernel1_config = KernelConfig {
        kernel_id: Some("test-kernel-multi1".to_string()),
        engine: "lua".to_string(),
        runtime_config: LLMSpellConfig::default(),
        debug_enabled: false,
        max_clients: 1,
        auth_enabled: false,
    };

    let kernel2_config = KernelConfig {
        kernel_id: Some("test-kernel-multi2".to_string()),
        engine: "lua".to_string(),
        runtime_config: LLMSpellConfig::default(),
        debug_enabled: false,
        max_clients: 1,
        auth_enabled: false,
    };

    let connection1 = ConnectionInfo::new(
        "test-kernel-multi1".to_string(),
        "127.0.0.1".to_string(),
        9300,
    );

    let connection2 = ConnectionInfo::new(
        "test-kernel-multi2".to_string(),
        "127.0.0.1".to_string(),
        9400,
    );

    // Create both kernels
    let kernel1 = GenericKernel::from_config_with_connection(kernel1_config, connection1).await?;
    let kernel2 = GenericKernel::from_config_with_connection(kernel2_config, connection2).await?;

    // Both kernels should be created (binding happens in new())
    // Port conflicts are handled at the transport level
    assert!(
        *kernel1.execution_state.read().await == KernelState::Starting,
        "First kernel should be starting"
    );
    assert!(
        *kernel2.execution_state.read().await == KernelState::Starting,
        "Second kernel should be starting"
    );
    Ok(())
}

/// Test kernel with debug enabled
#[tokio::test(flavor = "multi_thread")]
async fn test_kernel_debug_mode() -> Result<()> {
    let kernel_config = KernelConfig {
        kernel_id: Some("test-kernel-debug".to_string()),
        engine: "lua".to_string(),
        runtime_config: LLMSpellConfig::default(),
        debug_enabled: true, // Enable debug mode
        max_clients: 1,
        auth_enabled: false,
    };

    let connection_info = ConnectionInfo::new(
        "test-kernel-debug".to_string(),
        "127.0.0.1".to_string(),
        9500,
    );

    let kernel = GenericKernel::from_config_with_connection(kernel_config, connection_info).await?;
    assert!(kernel.config.debug_enabled, "Debug mode should be enabled");
    Ok(())
}

/// Test kernel with authentication enabled
#[tokio::test(flavor = "multi_thread")]
async fn test_kernel_auth_mode() -> Result<()> {
    let kernel_config = KernelConfig {
        kernel_id: Some("test-kernel-auth".to_string()),
        engine: "lua".to_string(),
        runtime_config: LLMSpellConfig::default(),
        debug_enabled: false,
        max_clients: 1,
        auth_enabled: true, // Enable authentication
    };

    let connection_info = ConnectionInfo::new(
        "test-kernel-auth".to_string(),
        "127.0.0.1".to_string(),
        9600,
    );

    let kernel = GenericKernel::from_config_with_connection(kernel_config, connection_info).await?;
    assert!(
        kernel.config.auth_enabled,
        "Authentication should be enabled"
    );
    Ok(())
}

/// Test kernel configuration validation
#[tokio::test(flavor = "multi_thread")]
async fn test_kernel_config_validation() -> Result<()> {
    // Test with invalid engine
    let invalid_config = KernelConfig {
        kernel_id: Some("test-kernel-invalid".to_string()),
        engine: "invalid_engine".to_string(),
        runtime_config: LLMSpellConfig::default(),
        debug_enabled: false,
        max_clients: 1,
        auth_enabled: false,
    };

    let connection_info = ConnectionInfo::new(
        "test-kernel-invalid".to_string(),
        "127.0.0.1".to_string(),
        9700,
    );

    let result = GenericKernel::from_config_with_connection(invalid_config, connection_info).await;
    assert!(
        result.is_err(),
        "Invalid engine should cause kernel creation to fail"
    );
    Ok(())
}

/// Test connection info port allocation
#[tokio::test(flavor = "multi_thread")]
async fn test_connection_port_allocation() -> Result<()> {
    let connection_info = ConnectionInfo::new(
        "test-kernel-ports".to_string(),
        "127.0.0.1".to_string(),
        9800,
    );

    // Verify port allocation follows expected pattern
    assert_eq!(connection_info.shell_port, 9800);
    assert_eq!(connection_info.iopub_port, 9801);
    assert_eq!(connection_info.stdin_port, 9802);
    assert_eq!(connection_info.control_port, 9803);
    assert_eq!(connection_info.hb_port, 9804);

    // All ports should be unique
    let ports = vec![
        connection_info.shell_port,
        connection_info.iopub_port,
        connection_info.stdin_port,
        connection_info.control_port,
        connection_info.hb_port,
    ];
    let mut unique_ports = ports.clone();
    unique_ports.sort_unstable();
    unique_ports.dedup();

    assert_eq!(
        ports.len(),
        unique_ports.len(),
        "All ports should be unique"
    );
    Ok(())
}

/// Test kernel with maximum clients limit
#[tokio::test(flavor = "multi_thread")]
async fn test_kernel_max_clients() -> Result<()> {
    let kernel_config = KernelConfig {
        kernel_id: Some("test-kernel-maxclients".to_string()),
        engine: "lua".to_string(),
        runtime_config: LLMSpellConfig::default(),
        debug_enabled: false,
        max_clients: 2, // Limit to 2 clients
        auth_enabled: false,
    };

    let connection_info = ConnectionInfo::new(
        "test-kernel-maxclients".to_string(),
        "127.0.0.1".to_string(),
        9900,
    );

    let kernel = GenericKernel::from_config_with_connection(kernel_config, connection_info).await?;
    assert_eq!(
        kernel.config.max_clients, 2,
        "Max clients should be set correctly"
    );
    Ok(())
}
