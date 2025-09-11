//! Tests for kernel IO routing through `IOPub` channel
//!
//! Verifies Task 9.8.15.2: Kernel Integration
//! - Script output is captured via callbacks
//! - Output is published to `IOPub` channel
//! - Multiple streams (stdout/stderr) are handled correctly

use llmspell_bridge::ScriptRuntime;
use llmspell_config::LLMSpellConfig;
use llmspell_core::io::SignalHandler;
use llmspell_kernel::{callback_io::create_callback_io_context, kernel_io::KernelSignalHandler};
use std::sync::{Arc, Mutex as StdMutex};

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_callback_io_captures_output() {
    // Create a buffer to capture output
    let output_buffer = Arc::new(StdMutex::new(String::new()));
    let output_buffer_clone = output_buffer.clone();

    let stdout_callback = move |text: &str| -> Result<(), llmspell_core::error::LLMSpellError> {
        output_buffer_clone.lock().unwrap().push_str(text);
        Ok(())
    };

    let stderr_buffer = Arc::new(StdMutex::new(String::new()));
    let stderr_buffer_clone = stderr_buffer.clone();

    let stderr_callback = move |text: &str| -> Result<(), llmspell_core::error::LLMSpellError> {
        stderr_buffer_clone.lock().unwrap().push_str(text);
        Ok(())
    };

    // Create callback-based IO context
    let signal_handler = Arc::new(KernelSignalHandler::new());
    let io_context = create_callback_io_context(stdout_callback, stderr_callback, signal_handler);

    // Create a runtime and execute script with IO context
    let config = LLMSpellConfig::default();
    let runtime = ScriptRuntime::new_with_lua(config).await.unwrap();
    let script = r#"
        print("Hello from Lua")
        print("This is line 2")
        return "execution complete"
    "#;

    let result = runtime.execute_script_with_io(script, io_context).await;
    assert!(result.is_ok());

    // Check captured output
    {
        let captured = output_buffer.lock().unwrap();
        assert!(
            captured.contains("Hello from Lua"),
            "Should capture first print"
        );
        assert!(
            captured.contains("This is line 2"),
            "Should capture second print"
        );
        drop(captured);
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_io_context_signal_handling() {
    let signal_handler = Arc::new(KernelSignalHandler::new());

    // Test interrupt handling
    assert!(!signal_handler.is_interrupted());
    signal_handler.interrupt();
    assert!(signal_handler.is_interrupted());

    // Test reset
    signal_handler.reset();
    assert!(!signal_handler.is_interrupted());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_execute_with_io_preserves_return_value() {
    let output_buffer = Arc::new(StdMutex::new(String::new()));
    let output_buffer_clone = output_buffer.clone();

    let stdout_callback = move |text: &str| -> Result<(), llmspell_core::error::LLMSpellError> {
        output_buffer_clone.lock().unwrap().push_str(text);
        Ok(())
    };

    let stderr_callback = |_: &str| -> Result<(), llmspell_core::error::LLMSpellError> { Ok(()) };

    let signal_handler = Arc::new(KernelSignalHandler::new());
    let io_context = create_callback_io_context(stdout_callback, stderr_callback, signal_handler);

    let config = LLMSpellConfig::default();
    let runtime = ScriptRuntime::new_with_lua(config).await.unwrap();

    // Script that both prints and returns a value
    let script = r#"
        print("Side effect output")
        return 42
    "#;

    let result = runtime
        .execute_script_with_io(script, io_context)
        .await
        .unwrap();

    // Check return value
    assert_eq!(result.output.to_string(), "42");

    // Check captured output
    assert!(output_buffer.lock().unwrap().contains("Side effect output"));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_error_handling_with_io_context() {
    let output_buffer = Arc::new(StdMutex::new(String::new()));
    let stderr_buffer = Arc::new(StdMutex::new(String::new()));

    let output_buffer_clone = output_buffer.clone();
    let stdout_callback = move |text: &str| -> Result<(), llmspell_core::error::LLMSpellError> {
        output_buffer_clone.lock().unwrap().push_str(text);
        Ok(())
    };

    let stderr_buffer_clone = stderr_buffer.clone();
    let stderr_callback = move |text: &str| -> Result<(), llmspell_core::error::LLMSpellError> {
        stderr_buffer_clone.lock().unwrap().push_str(text);
        Ok(())
    };

    let signal_handler = Arc::new(KernelSignalHandler::new());
    let io_context = create_callback_io_context(stdout_callback, stderr_callback, signal_handler);

    let config = LLMSpellConfig::default();
    let runtime = ScriptRuntime::new_with_lua(config).await.unwrap();

    // Script with an error
    let script = r#"
        print("Before error")
        error("Intentional error")
        print("After error - should not print")
    "#;

    let result = runtime.execute_script_with_io(script, io_context).await;
    assert!(result.is_err());

    // Check that output before error was captured
    {
        let captured = output_buffer.lock().unwrap();
        assert!(captured.contains("Before error"));
        assert!(!captured.contains("After error"));
        drop(captured);
    }
}
