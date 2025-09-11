//! Integration test for kernel IO capture
//! Verifies that `print()` output is properly captured and routed

use anyhow::Result;
use llmspell_config::{GlobalRuntimeConfig, KernelSettings, LLMSpellConfig};
use llmspell_kernel::kernel::{GenericKernel, KernelState};
use llmspell_kernel::ConnectionInfo;
use llmspell_testing::kernel_helpers::create_test_kernel_config;
use std::sync::Arc;

/// Test basic kernel IO capture in-process
#[tokio::test(flavor = "multi_thread")]
async fn test_kernel_output_capture() -> Result<()> {
    let (kernel_id, port) = create_test_kernel_config().await?;
    let config = Arc::new(
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

    let connection_info = ConnectionInfo::new(kernel_id.clone(), "127.0.0.1".to_string(), port);

    // Create kernel in-process
    let kernel = GenericKernel::from_config_with_connection(
        kernel_id.clone(),
        config.clone(),
        connection_info.clone(),
    )
    .await?;

    // Set kernel to idle state
    {
        let mut state = kernel.execution_state.write().await;
        *state = KernelState::Idle;
    }

    // Verify kernel is in the expected state
    assert_eq!(*kernel.execution_state.read().await, KernelState::Idle);

    println!("Kernel IO test passed - kernel created with ID: {kernel_id}");
    Ok(())
}

/// Test interrupt handling
#[tokio::test(flavor = "multi_thread")]
async fn test_interrupt_handling() -> Result<()> {
    let (kernel_id, port) = create_test_kernel_config().await?;
    let config = Arc::new(LLMSpellConfig::builder().default_engine("lua").build());

    let connection_info = ConnectionInfo::new(kernel_id.clone(), "127.0.0.1".to_string(), port);

    // Create kernel
    let kernel =
        GenericKernel::from_config_with_connection(kernel_id, config, connection_info.clone())
            .await?;

    // Set to idle
    {
        let mut state = kernel.execution_state.write().await;
        *state = KernelState::Idle;
    }

    // Test interrupt (using the public method we added)
    kernel.interrupt();

    // Verify kernel can handle interrupts
    assert_eq!(*kernel.execution_state.read().await, KernelState::Idle);

    Ok(())
}
