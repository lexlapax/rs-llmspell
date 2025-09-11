//! Integration tests for kernel IO operations
//! Tests the full stack: kernel -> IOContext -> script -> output

use llmspell_config::{GlobalRuntimeConfig, KernelSettings, LLMSpellConfig};
use llmspell_kernel::kernel::{GenericKernel, KernelState};
use llmspell_kernel::ConnectionInfo;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test]
async fn test_kernel_stdout_routing() {
    // Create kernel with test configuration
    let kernel_id = "test-io-stdout".to_string();
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

    let connection_info = ConnectionInfo::new(kernel_id.clone(), "127.0.0.1".to_string(), 50000);

    // Create kernel
    let kernel = GenericKernel::from_config_with_connection(kernel_id, config, connection_info)
        .await
        .expect("Failed to create kernel");

    // Verify kernel was created with correct state
    assert_eq!(
        *kernel.execution_state.read().await,
        KernelState::Starting,
        "Kernel should start in Starting state"
    );

    // Set kernel to idle state
    {
        let mut state = kernel.execution_state.write().await;
        *state = KernelState::Idle;
    }

    // Execute code that produces output
    let script = r#"print("Test output from kernel")"#;

    // Collect output through callback
    let output_buffer = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
    let buffer_clone = output_buffer.clone();

    let output_callback = move |msg: String| {
        buffer_clone.lock().unwrap().push(msg);
    };

    // Execute the script through the kernel (using execute_code_streaming)
    let result = kernel
        .execute_code_streaming(
            script,
            Arc::new(output_callback),
            "test_session".to_string(),
            None,
        )
        .await;

    assert!(result.is_ok(), "Execution should succeed");

    // Verify output was routed correctly
    let output = output_buffer.lock().unwrap();
    let output_text = output.join("");
    assert!(
        output_text.contains("Test output from kernel"),
        "Should capture print output"
    );

    // Shutdown kernel
    let _ = timeout(Duration::from_secs(1), kernel.shutdown()).await;
}

#[tokio::test]
async fn test_kernel_stderr_separation() {
    let kernel_id = "test-io-stderr".to_string();
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

    let connection_info = ConnectionInfo::new(kernel_id.clone(), "127.0.0.1".to_string(), 51000);

    // Create kernel
    let kernel = GenericKernel::from_config_with_connection(kernel_id, config, connection_info)
        .await
        .expect("Failed to create kernel");

    // Set kernel to idle state
    {
        let mut state = kernel.execution_state.write().await;
        *state = KernelState::Idle;
    }

    // Execute code that produces both stdout and stderr
    let script = r#"
print("Normal output")
io.stderr:write("Error output\n")
print("More normal output")
"#;

    // Collect output
    let output_buffer = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
    let buffer_clone = output_buffer.clone();

    let output_callback = move |msg: String| {
        buffer_clone.lock().unwrap().push(msg);
    };

    let result = kernel
        .execute_code_streaming(
            script,
            Arc::new(output_callback),
            "test_session".to_string(),
            None,
        )
        .await;

    assert!(result.is_ok(), "Execution should succeed");

    // Verify both stdout and stderr were captured
    let output = output_buffer.lock().unwrap();
    let output_text = output.join("");
    assert!(
        output_text.contains("Normal output"),
        "Should capture stdout"
    );
    assert!(
        output_text.contains("Error output"),
        "Should capture stderr"
    );
    assert!(
        output_text.contains("More normal output"),
        "Should continue after stderr"
    );

    let _ = timeout(Duration::from_secs(1), kernel.shutdown()).await;
}

#[tokio::test]
async fn test_kernel_interrupt_handling() {
    let kernel_id = "test-io-interrupt".to_string();
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

    let connection_info = ConnectionInfo::new(kernel_id.clone(), "127.0.0.1".to_string(), 52000);

    // Create kernel
    let kernel = GenericKernel::from_config_with_connection(kernel_id, config, connection_info)
        .await
        .expect("Failed to create kernel");

    // Set kernel to idle state
    {
        let mut state = kernel.execution_state.write().await;
        *state = KernelState::Idle;
    }

    // Start a long-running script
    let script = r#"
for i = 1, 1000000 do
    if i % 10000 == 0 then
        print("Iteration " .. i)
    end
end
"#;

    // Collect output
    let output_buffer = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
    let buffer_clone = output_buffer.clone();

    let output_callback = move |msg: String| {
        buffer_clone.lock().unwrap().push(msg);
    };

    // Execute in background
    let kernel_clone = kernel.clone();
    let exec_task = tokio::spawn(async move {
        kernel_clone
            .execute_code_streaming(
                script,
                Arc::new(output_callback),
                "test_session".to_string(),
                None,
            )
            .await
    });

    // Give it time to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Send interrupt signal
    kernel.signal_handler.interrupt();

    // Execution should be interrupted quickly
    let exec_result = timeout(Duration::from_secs(2), exec_task).await;
    assert!(
        exec_result.is_ok(),
        "Execution should complete (be interrupted) quickly"
    );

    let _ = timeout(Duration::from_secs(1), kernel.shutdown()).await;
}

#[tokio::test]
async fn test_kernel_multiple_executions() {
    let kernel_id = "test-io-multiple".to_string();
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

    let connection_info = ConnectionInfo::new(kernel_id.clone(), "127.0.0.1".to_string(), 53000);

    // Create kernel
    let kernel = GenericKernel::from_config_with_connection(kernel_id, config, connection_info)
        .await
        .expect("Failed to create kernel");

    // Set kernel to idle state
    {
        let mut state = kernel.execution_state.write().await;
        *state = KernelState::Idle;
    }

    // First execution - set a variable
    let script1 = r#"x = 1; print('First execution set x=' .. x)"#;

    let output_buffer1 = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
    let buffer_clone1 = output_buffer1.clone();

    let output_callback1 = move |msg: String| {
        buffer_clone1.lock().unwrap().push(msg);
    };

    let result1 = kernel
        .execute_code_streaming(
            script1,
            Arc::new(output_callback1),
            "test_session".to_string(),
            None,
        )
        .await;
    assert!(result1.is_ok(), "First execution should succeed");

    // Second execution - read and modify the variable
    let script2 = r#"x = x + 1; print('Second execution sees x=' .. x)"#;

    let output_buffer2 = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
    let buffer_clone2 = output_buffer2.clone();

    let output_callback2 = move |msg: String| {
        buffer_clone2.lock().unwrap().push(msg);
    };

    let result2 = kernel
        .execute_code_streaming(
            script2,
            Arc::new(output_callback2),
            "test_session".to_string(),
            None,
        )
        .await;
    assert!(result2.is_ok(), "Second execution should succeed");

    // Verify both executions produced output
    let output1 = output_buffer1.lock().unwrap();
    let output1_text = output1.join("");
    assert!(
        output1_text.contains("First execution set x=1"),
        "First execution should set x=1"
    );

    let output2 = output_buffer2.lock().unwrap();
    let output2_text = output2.join("");
    assert!(
        output2_text.contains("Second execution sees x=2"),
        "Second execution should see x=2"
    );

    let _ = timeout(Duration::from_secs(1), kernel.shutdown()).await;
}

#[tokio::test]
async fn test_kernel_io_performance() {
    let kernel_id = "test-io-performance".to_string();
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

    let connection_info = ConnectionInfo::new(kernel_id.clone(), "127.0.0.1".to_string(), 54000);

    // Create kernel
    let kernel = GenericKernel::from_config_with_connection(kernel_id, config, connection_info)
        .await
        .expect("Failed to create kernel");

    // Set kernel to idle state
    {
        let mut state = kernel.execution_state.write().await;
        *state = KernelState::Idle;
    }

    // Generate script with many output lines
    let script = r#"
for i = 1, 100 do
    print("Line " .. i .. ": This is a test line with some content")
end
"#;

    let output_buffer = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
    let buffer_clone = output_buffer.clone();

    let output_callback = move |msg: String| {
        buffer_clone.lock().unwrap().push(msg);
    };

    let start = std::time::Instant::now();
    let result = kernel
        .execute_code_streaming(
            script,
            Arc::new(output_callback),
            "test_session".to_string(),
            None,
        )
        .await;
    let duration = start.elapsed();

    assert!(result.is_ok(), "Execution should succeed");

    // Verify all lines were captured
    let output = output_buffer.lock().unwrap();
    let output_text = output.join("");
    for i in 1..=100 {
        assert!(
            output_text.contains(&format!("Line {i}")),
            "Should have line {i}"
        );
    }

    // Performance check - should complete quickly even with many lines
    assert!(
        duration < Duration::from_secs(2),
        "100 output lines should complete within 2 seconds, took {:?}",
        duration
    );

    let _ = timeout(Duration::from_secs(1), kernel.shutdown()).await;
}
