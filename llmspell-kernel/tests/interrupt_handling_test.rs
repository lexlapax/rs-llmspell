//! Tests for interrupt handling in kernel execution
//!
//! Verifies Task 9.8.15.3: Signal Handling
//! - Interrupt requests stop script execution
//! - ExecutionInterrupted error is properly propagated
//! - Signal handler state is correctly managed

use llmspell_bridge::ScriptRuntime;
use llmspell_config::LLMSpellConfig;
use llmspell_core::io::SignalHandler;
use llmspell_kernel::{callback_io::create_callback_io_context, kernel_io::KernelSignalHandler};
use std::sync::{Arc, Mutex as StdMutex};
use std::time::Duration;
use tokio::time::timeout;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_interrupt_stops_execution() {
    // Create a buffer to capture output
    let output_buffer = Arc::new(StdMutex::new(String::new()));
    let output_buffer_clone = output_buffer.clone();

    let stdout_callback = move |text: &str| -> Result<(), llmspell_core::error::LLMSpellError> {
        let mut buffer = output_buffer_clone.lock().unwrap();
        buffer.push_str(text);
        Ok(())
    };

    let stderr_callback = |_: &str| -> Result<(), llmspell_core::error::LLMSpellError> { Ok(()) };

    // Create signal handler
    let signal_handler = Arc::new(KernelSignalHandler::new());
    let io_context =
        create_callback_io_context(stdout_callback, stderr_callback, signal_handler.clone());

    // Create runtime
    let config = LLMSpellConfig::default();
    let runtime = ScriptRuntime::new_with_lua(config).await.unwrap();

    // Script with a long-running loop
    let script = r#"
        local count = 0
        while true do
            count = count + 1
            if count % 100000 == 0 then
                print("Still running: " .. count)
            end
            -- This loop will run forever unless interrupted
        end
        return "should not reach here"
    "#;

    // Start execution in a separate task
    let io_context_clone = io_context.clone();
    let execution_task = tokio::spawn(async move {
        runtime
            .execute_script_with_io(script, io_context_clone)
            .await
    });

    // Wait a bit for execution to start
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Trigger interrupt
    signal_handler.interrupt();

    // Wait for execution to complete (should be interrupted)
    let result = timeout(Duration::from_secs(2), execution_task).await;

    // Verify execution was interrupted
    assert!(result.is_ok(), "Execution should complete within timeout");
    let execution_result = result.unwrap().unwrap();
    assert!(
        execution_result.is_err(),
        "Execution should return an error"
    );

    let error = execution_result.unwrap_err();
    match error {
        llmspell_core::error::LLMSpellError::ExecutionInterrupted { .. } => {
            // Expected error type
        }
        _ => panic!("Expected ExecutionInterrupted error, got: {:?}", error),
    }

    // Check that some output was captured before interruption
    let captured = output_buffer.lock().unwrap();
    assert!(
        captured.contains("Still running") || captured.is_empty(),
        "Should have captured some output or been interrupted immediately"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_signal_handler_reset() {
    let signal_handler = Arc::new(KernelSignalHandler::new());

    // Initially not interrupted
    assert!(!signal_handler.is_interrupted());

    // Set interrupt
    signal_handler.interrupt();
    assert!(signal_handler.is_interrupted());

    // Reset clears the interrupt
    signal_handler.reset();
    assert!(!signal_handler.is_interrupted());

    // Can interrupt again
    signal_handler.interrupt();
    assert!(signal_handler.is_interrupted());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_normal_execution_not_affected() {
    let output_buffer = Arc::new(StdMutex::new(String::new()));
    let output_buffer_clone = output_buffer.clone();

    let stdout_callback = move |text: &str| -> Result<(), llmspell_core::error::LLMSpellError> {
        let mut buffer = output_buffer_clone.lock().unwrap();
        buffer.push_str(text);
        Ok(())
    };

    let stderr_callback = |_: &str| -> Result<(), llmspell_core::error::LLMSpellError> { Ok(()) };

    // Create signal handler but don't interrupt
    let signal_handler = Arc::new(KernelSignalHandler::new());
    let io_context =
        create_callback_io_context(stdout_callback, stderr_callback, signal_handler.clone());

    let config = LLMSpellConfig::default();
    let runtime = ScriptRuntime::new_with_lua(config).await.unwrap();

    // Normal script that completes quickly
    let script = r#"
        print("Starting")
        local sum = 0
        for i = 1, 100 do
            sum = sum + i
        end
        print("Sum: " .. sum)
        return sum
    "#;

    let result = runtime.execute_script_with_io(script, io_context).await;

    // Should complete successfully
    assert!(result.is_ok(), "Normal execution should succeed");
    let output = result.unwrap();
    assert_eq!(
        output.output.to_string(),
        "5050",
        "Should return correct sum"
    );

    // Check output was captured
    let captured = output_buffer.lock().unwrap();
    assert!(captured.contains("Starting"));
    assert!(captured.contains("Sum: 5050"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_interrupt_timing() {
    let output_buffer = Arc::new(StdMutex::new(String::new()));
    let output_buffer_clone = output_buffer.clone();

    let stdout_callback = move |text: &str| -> Result<(), llmspell_core::error::LLMSpellError> {
        let mut buffer = output_buffer_clone.lock().unwrap();
        buffer.push_str(text);
        Ok(())
    };

    let stderr_callback = |_: &str| -> Result<(), llmspell_core::error::LLMSpellError> { Ok(()) };

    let signal_handler = Arc::new(KernelSignalHandler::new());
    let io_context =
        create_callback_io_context(stdout_callback, stderr_callback, signal_handler.clone());

    let config = LLMSpellConfig::default();
    let runtime = ScriptRuntime::new_with_lua(config).await.unwrap();

    // Script with controlled iterations
    let script = r#"
        for i = 1, 10000000 do
            if i == 1 then
                print("Loop started")
            end
            -- Do some work
            local x = i * 2
        end
        print("Loop completed")
        return "done"
    "#;

    // Start execution
    let io_context_clone = io_context.clone();
    let execution_task = tokio::spawn(async move {
        runtime
            .execute_script_with_io(script, io_context_clone)
            .await
    });

    // Interrupt after a short delay
    tokio::time::sleep(Duration::from_millis(50)).await;
    signal_handler.interrupt();

    // Wait for result
    let result = timeout(Duration::from_secs(2), execution_task).await;

    // Should be interrupted before completion
    assert!(result.is_ok(), "Should complete within timeout");
    let execution_result = result.unwrap().unwrap();
    assert!(execution_result.is_err(), "Should be interrupted");

    // Verify output shows start but not completion
    let captured = output_buffer.lock().unwrap();
    if !captured.is_empty() {
        assert!(
            captured.contains("Loop started"),
            "Should have started the loop"
        );
        assert!(
            !captured.contains("Loop completed"),
            "Should not have completed the loop"
        );
    }
}
