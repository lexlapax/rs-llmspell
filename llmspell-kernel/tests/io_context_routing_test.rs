//! Unit tests for IO context routing through the execution stack

use llmspell_bridge::runtime::ScriptRuntime;
use llmspell_config::LLMSpellConfig;
use llmspell_core::io::{IOContext, IOStream};
use std::sync::Arc;

// Test helper struct for capturing IO output
struct TestStream {
    buffer: Arc<std::sync::Mutex<Vec<String>>>,
}

impl IOStream for TestStream {
    fn write(&self, data: &str) -> Result<(), llmspell_core::error::LLMSpellError> {
        self.buffer.lock().unwrap().push(data.to_string());
        Ok(())
    }

    fn write_line(&self, line: &str) -> Result<(), llmspell_core::error::LLMSpellError> {
        self.buffer.lock().unwrap().push(format!("{line}\n"));
        Ok(())
    }

    fn flush(&self) -> Result<(), llmspell_core::error::LLMSpellError> {
        Ok(())
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_io_context_routing() {
    // Create test IO collectors
    let stdout_buffer = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
    let stderr_buffer = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));

    let stdout_clone = stdout_buffer.clone();
    let stderr_clone = stderr_buffer.clone();

    // Create custom IO streams that collect output
    let io_context = Arc::new(IOContext::new(
        Arc::new(TestStream {
            buffer: stdout_clone,
        }),
        Arc::new(TestStream {
            buffer: stderr_clone,
        }),
        Arc::new(llmspell_core::io::NullInput),
        Arc::new(llmspell_core::io::NoOpSignalHandler),
        llmspell_core::io::IOPerformanceHints::default(),
    ));

    // Create runtime with Lua engine
    let config = LLMSpellConfig::default();
    let runtime = ScriptRuntime::new_with_lua(config).await.unwrap();

    // Execute script with IO context
    let script = r#"
        print("Hello from Lua")
        io.write("Direct write")
    "#;

    let result = runtime
        .execute_script_with_io(script, io_context.clone())
        .await;
    assert!(result.is_ok(), "Script execution should succeed");

    // Verify stdout output
    let stdout_text = stdout_buffer.lock().unwrap().join("");
    assert!(
        stdout_text.contains("Hello from Lua"),
        "Should capture print output"
    );
    assert!(
        stdout_text.contains("Direct write"),
        "Should capture io.write output"
    );

    // Note: stderr routing is tested in test_io_context_error_handling
}

#[tokio::test(flavor = "multi_thread")]
async fn test_io_context_isolation() {
    // Test that each execution gets its own IO context

    let config = LLMSpellConfig::default();
    let runtime = ScriptRuntime::new_with_lua(config).await.unwrap();

    // First execution
    let buffer1 = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
    let io1 = Arc::new(IOContext::new(
        Arc::new(TestStream {
            buffer: buffer1.clone(),
        }),
        Arc::new(TestStream {
            buffer: Arc::new(std::sync::Mutex::new(Vec::new())),
        }),
        Arc::new(llmspell_core::io::NullInput),
        Arc::new(llmspell_core::io::NoOpSignalHandler),
        llmspell_core::io::IOPerformanceHints::default(),
    ));

    let result1 = runtime
        .execute_script_with_io("print('execution 1')", io1.clone())
        .await;
    assert!(result1.is_ok());

    // Second execution with different context
    let buffer2 = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
    let io2 = Arc::new(IOContext::new(
        Arc::new(TestStream {
            buffer: buffer2.clone(),
        }),
        Arc::new(TestStream {
            buffer: Arc::new(std::sync::Mutex::new(Vec::new())),
        }),
        Arc::new(llmspell_core::io::NullInput),
        Arc::new(llmspell_core::io::NoOpSignalHandler),
        llmspell_core::io::IOPerformanceHints::default(),
    ));

    let result2 = runtime
        .execute_script_with_io("print('execution 2')", io2.clone())
        .await;
    assert!(result2.is_ok());

    // Verify isolation - io1 should not have execution 2's output
    let io1_text = buffer1.lock().unwrap().join("");
    assert!(
        io1_text.contains("execution 1"),
        "IO1 should have execution 1 output"
    );
    assert!(
        !io1_text.contains("execution 2"),
        "IO1 should not have execution 2 output"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_io_context_error_handling() {
    struct TestStream {
        buffer: Arc<std::sync::Mutex<Vec<String>>>,
    }

    impl IOStream for TestStream {
        fn write(&self, data: &str) -> Result<(), llmspell_core::error::LLMSpellError> {
            self.buffer.lock().unwrap().push(data.to_string());
            Ok(())
        }

        fn write_line(&self, line: &str) -> Result<(), llmspell_core::error::LLMSpellError> {
            self.buffer.lock().unwrap().push(format!("{line}\n"));
            Ok(())
        }

        fn flush(&self) -> Result<(), llmspell_core::error::LLMSpellError> {
            Ok(())
        }
    }

    let stdout_buffer = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
    let stderr_buffer = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));

    let io_context = Arc::new(IOContext::new(
        Arc::new(TestStream {
            buffer: stdout_buffer.clone(),
        }),
        Arc::new(TestStream {
            buffer: stderr_buffer.clone(),
        }),
        Arc::new(llmspell_core::io::NullInput),
        Arc::new(llmspell_core::io::NoOpSignalHandler),
        llmspell_core::io::IOPerformanceHints::default(),
    ));

    let config = LLMSpellConfig::default();
    let runtime = ScriptRuntime::new_with_lua(config).await.unwrap();

    // Execute script that produces an error
    let script = r#"
        print("Before error")
        error("Something went wrong")
        print("After error")  -- Should not execute
    "#;

    let result = runtime.execute_script_with_io(script, io_context).await;
    assert!(result.is_err(), "Script with error should fail");

    // Verify output before error was captured
    let stdout_text = stdout_buffer.lock().unwrap().join("");
    assert!(
        stdout_text.contains("Before error"),
        "Should capture output before error"
    );
    assert!(
        !stdout_text.contains("After error"),
        "Should not have output after error"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_io_context_with_return_value() {
    struct TestStream {
        buffer: Arc<std::sync::Mutex<Vec<String>>>,
    }

    impl IOStream for TestStream {
        fn write(&self, data: &str) -> Result<(), llmspell_core::error::LLMSpellError> {
            self.buffer.lock().unwrap().push(data.to_string());
            Ok(())
        }

        fn write_line(&self, line: &str) -> Result<(), llmspell_core::error::LLMSpellError> {
            self.buffer.lock().unwrap().push(format!("{line}\n"));
            Ok(())
        }

        fn flush(&self) -> Result<(), llmspell_core::error::LLMSpellError> {
            Ok(())
        }
    }

    let stdout_buffer = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
    let io_context = Arc::new(IOContext::new(
        Arc::new(TestStream {
            buffer: stdout_buffer.clone(),
        }),
        Arc::new(TestStream {
            buffer: Arc::new(std::sync::Mutex::new(Vec::new())),
        }),
        Arc::new(llmspell_core::io::NullInput),
        Arc::new(llmspell_core::io::NoOpSignalHandler),
        llmspell_core::io::IOPerformanceHints::default(),
    ));

    let config = LLMSpellConfig::default();
    let runtime = ScriptRuntime::new_with_lua(config).await.unwrap();

    let script = r#"
        print("Computing result...")
        local result = 42
        print("Result computed: " .. result)
        return result
    "#;

    let result = runtime.execute_script_with_io(script, io_context).await;
    assert!(result.is_ok(), "Script should execute successfully");

    // Verify both output and return value
    let output = result.unwrap();
    assert!(
        output.output.to_string().contains("42"),
        "Should return the correct value"
    );

    let stdout_text = stdout_buffer.lock().unwrap().join("");
    assert!(
        stdout_text.contains("Computing result"),
        "Should capture print statements"
    );
    assert!(
        stdout_text.contains("Result computed: 42"),
        "Should capture computed output"
    );
}

#[tokio::test(flavor = "multi_thread")]
async fn test_io_context_multiline_output() {
    struct TestStream {
        buffer: Arc<std::sync::Mutex<Vec<String>>>,
    }

    impl IOStream for TestStream {
        fn write(&self, data: &str) -> Result<(), llmspell_core::error::LLMSpellError> {
            self.buffer.lock().unwrap().push(data.to_string());
            Ok(())
        }

        fn write_line(&self, line: &str) -> Result<(), llmspell_core::error::LLMSpellError> {
            self.buffer.lock().unwrap().push(format!("{line}\n"));
            Ok(())
        }

        fn flush(&self) -> Result<(), llmspell_core::error::LLMSpellError> {
            Ok(())
        }
    }

    let stdout_buffer = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
    let io_context = Arc::new(IOContext::new(
        Arc::new(TestStream {
            buffer: stdout_buffer.clone(),
        }),
        Arc::new(TestStream {
            buffer: Arc::new(std::sync::Mutex::new(Vec::new())),
        }),
        Arc::new(llmspell_core::io::NullInput),
        Arc::new(llmspell_core::io::NoOpSignalHandler),
        llmspell_core::io::IOPerformanceHints::default(),
    ));

    let config = LLMSpellConfig::default();
    let runtime = ScriptRuntime::new_with_lua(config).await.unwrap();

    let script = r#"
        for i = 1, 5 do
            print("Line " .. i)
        end
    "#;

    let result = runtime.execute_script_with_io(script, io_context).await;
    assert!(result.is_ok());

    let stdout_text = stdout_buffer.lock().unwrap().join("");

    for i in 1..=5 {
        assert!(
            stdout_text.contains(&format!("Line {i}")),
            "Should have line {i}"
        );
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_io_context_flush_behavior() {
    struct TestStream {
        buffer: Arc<std::sync::Mutex<Vec<String>>>,
    }

    impl IOStream for TestStream {
        fn write(&self, data: &str) -> Result<(), llmspell_core::error::LLMSpellError> {
            self.buffer.lock().unwrap().push(data.to_string());
            Ok(())
        }

        fn write_line(&self, line: &str) -> Result<(), llmspell_core::error::LLMSpellError> {
            self.buffer.lock().unwrap().push(format!("{line}\n"));
            Ok(())
        }

        fn flush(&self) -> Result<(), llmspell_core::error::LLMSpellError> {
            Ok(())
        }
    }

    let stdout_buffer = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
    let io_context = Arc::new(IOContext::new(
        Arc::new(TestStream {
            buffer: stdout_buffer.clone(),
        }),
        Arc::new(TestStream {
            buffer: Arc::new(std::sync::Mutex::new(Vec::new())),
        }),
        Arc::new(llmspell_core::io::NullInput),
        Arc::new(llmspell_core::io::NoOpSignalHandler),
        llmspell_core::io::IOPerformanceHints::default(),
    ));

    let config = LLMSpellConfig::default();
    let runtime = ScriptRuntime::new_with_lua(config).await.unwrap();

    let script = r#"
        io.write("Part 1")
        io.flush()
        io.write(" Part 2")
        io.flush()
    "#;

    let result = runtime.execute_script_with_io(script, io_context).await;
    assert!(result.is_ok());

    let full_output = stdout_buffer.lock().unwrap().join("");
    assert!(
        full_output.contains("Part 1") && full_output.contains("Part 2"),
        "Should capture both parts with explicit flush"
    );
}
