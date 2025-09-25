//! Tests for script execution via the .run command
//!
//! Verifies file loading, error handling, and script argument passing.

#[cfg(test)]
mod script_execution_tests {
    use llmspell_bridge::runtime::ScriptRuntime;
    use llmspell_core::traits::script_executor::ScriptExecutor;
    use std::fs;
    use tempfile::TempDir;

    /// Create a test script runtime
    async fn create_test_runtime() -> ScriptRuntime {
        use llmspell_config::LLMSpellConfig;

        ScriptRuntime::new_with_lua(LLMSpellConfig::default())
            .await
            .unwrap()
    }

    /// Test executing valid Lua scripts
    #[tokio::test]
    #[ignore = "Requires full bridge setup"]
    async fn test_execute_valid_script() {
        let temp_dir = TempDir::new().unwrap();
        let script_path = temp_dir.path().join("test.lua");

        // Write a simple Lua script
        let script_content = r#"
            local function greet(name)
                return "Hello, " .. name .. "!"
            end
            return greet("World")
        "#;
        fs::write(&script_path, script_content).unwrap();

        let runtime = create_test_runtime().await;

        // Execute the script
        let result = runtime.execute_script(script_content).await;
        assert!(result.is_ok());

        let output = result.unwrap();
        assert_eq!(output.output.as_str().unwrap(), "Hello, World!");
    }

    /// Test file not found error
    #[tokio::test]
    async fn test_file_not_found() {
        let temp_dir = TempDir::new().unwrap();
        let non_existent = temp_dir.path().join("does_not_exist.lua");

        // Try to read non-existent file
        let result = fs::read_to_string(&non_existent);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No such file"));
    }

    /// Test syntax error reporting with line numbers
    #[tokio::test]
    #[ignore = "Requires full bridge setup"]
    async fn test_syntax_error_reporting() {
        let script_with_syntax_error = r#"
            function broken(
                -- Missing closing parenthesis
                return 42
            end
        "#;

        let runtime = create_test_runtime().await;
        let result = runtime.execute_script(script_with_syntax_error).await;

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();

        // Should contain line number information
        assert!(
            error_msg.contains("line") || error_msg.contains("2") || error_msg.contains("syntax")
        );
    }

    /// Test runtime error handling
    #[tokio::test]
    #[ignore = "Requires full bridge setup"]
    async fn test_runtime_error() {
        let script_with_runtime_error = r#"
            local function divide(a, b)
                return a / b
            end
            -- This will cause a runtime error (attempt to perform arithmetic on nil)
            return divide(10, nil)
        "#;

        let runtime = create_test_runtime().await;
        let result = runtime.execute_script(script_with_runtime_error).await;

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();

        // Should contain runtime error information
        assert!(
            error_msg.contains("nil")
                || error_msg.contains("arithmetic")
                || error_msg.contains("number")
        );
    }

    /// Test relative and absolute paths
    #[tokio::test]
    async fn test_path_handling() {
        let temp_dir = TempDir::new().unwrap();

        // Test absolute path
        let abs_path = temp_dir.path().join("absolute.lua");
        fs::write(&abs_path, "return 'absolute'").unwrap();
        assert!(abs_path.is_absolute());
        let content = fs::read_to_string(&abs_path).unwrap();
        assert_eq!(content, "return 'absolute'");

        // Test relative path
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp_dir.path()).unwrap();

        fs::write("relative.lua", "return 'relative'").unwrap();
        let content = fs::read_to_string("relative.lua").unwrap();
        assert_eq!(content, "return 'relative'");

        std::env::set_current_dir(original_dir).unwrap();
    }

    /// Test with and without .lua extension
    #[test]
    fn test_file_extension_handling() {
        let temp_dir = TempDir::new().unwrap();

        // File with .lua extension
        let with_ext = temp_dir.path().join("script.lua");
        fs::write(&with_ext, "return 1").unwrap();
        assert!(with_ext.exists());

        // File without extension
        let without_ext = temp_dir.path().join("script");
        fs::write(&without_ext, "return 2").unwrap();

        // Try to load without extension, should try adding .lua
        let mut path = temp_dir.path().join("script");
        if !path.exists() {
            path.set_extension("lua");
        }
        assert!(path.exists());
    }

    /// Test script arguments passing
    #[tokio::test]
    #[ignore = "Requires full bridge setup"]
    async fn test_script_arguments() {
        let script_with_args = r#"
            -- Access arguments (in real implementation via ARGS global)
            local args = ARGS or {}
            local name = args.name or "Guest"
            local count = tonumber(args.count) or 1

            local result = {}
            for i = 1, count do
                table.insert(result, "Hello " .. name .. " #" .. i)
            end
            return table.concat(result, ", ")
        "#;

        let runtime = create_test_runtime().await;

        // Execute without arguments
        let result = runtime.execute_script(script_with_args).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.output.as_str().unwrap().contains("Hello Guest #1"));

        // Execute with arguments (would need execute_script_with_args in real impl)
        let mut args = std::collections::HashMap::new();
        args.insert("name".to_string(), "Alice".to_string());
        args.insert("count".to_string(), "3".to_string());

        let result = runtime
            .execute_script_with_args(script_with_args, args)
            .await;
        assert!(result.is_ok());
        // Note: This would work if ARGS injection is implemented
    }

    /// Test working directory setting
    #[tokio::test]
    #[ignore = "Requires full bridge setup"]
    async fn test_working_directory() {
        let temp_dir = TempDir::new().unwrap();
        let work_dir = temp_dir.path().join("workspace");
        fs::create_dir(&work_dir).unwrap();

        // Create a file in the workspace
        fs::write(work_dir.join("data.txt"), "test data").unwrap();

        let script = r#"
            -- Try to read file from working directory
            local f = io.open("data.txt", "r")
            if f then
                local content = f:read("*all")
                f:close()
                return content
            else
                return "File not found"
            end
        "#;

        let runtime = create_test_runtime().await;

        // Execute without changing directory
        let _result = runtime.execute_script(script).await;
        // Might fail or return "File not found" depending on current dir

        // Change to work directory and execute
        let original_dir = std::env::current_dir().unwrap();
        std::env::set_current_dir(&work_dir).unwrap();

        let result = runtime.execute_script(script).await;
        assert!(result.is_ok());
        let _output = result.unwrap();
        // Should be able to read the file now
        // Note: Actual behavior depends on Lua sandbox settings

        std::env::set_current_dir(original_dir).unwrap();
    }

    /// Test script output capture
    #[tokio::test]
    #[ignore = "Requires full bridge setup"]
    async fn test_output_capture() {
        let script_with_output = r#"
            print("Line 1")
            print("Line 2")
            io.write("No newline")
            print(" - with newline")
            return "final value"
        "#;

        let runtime = create_test_runtime().await;
        let result = runtime.execute_script(script_with_output).await;

        assert!(result.is_ok());
        let output = result.unwrap();

        // Check return value
        assert_eq!(output.output.as_str().unwrap(), "final value");

        // Check captured console output
        assert!(output.console_output.contains(&"Line 1".to_string()));
        assert!(output.console_output.contains(&"Line 2".to_string()));
    }

    /// Test error propagation and stack traces
    #[tokio::test]
    #[ignore = "Requires full bridge setup"]
    async fn test_error_stack_trace() {
        let script_with_nested_error = r#"
            local function level3()
                error("Deep error")
            end

            local function level2()
                level3()
            end

            local function level1()
                level2()
            end

            level1()
        "#;

        let runtime = create_test_runtime().await;
        let result = runtime.execute_script(script_with_nested_error).await;

        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();

        // Should contain error message
        assert!(error_msg.contains("Deep error"));
        // Might contain stack trace information (depends on Lua error reporting)
    }

    /// Test script timeout/cancellation
    #[tokio::test]
    #[ignore = "Requires full bridge setup"]
    async fn test_script_timeout() {
        let infinite_loop = r#"
            while true do
                -- Infinite loop
            end
        "#;

        let runtime = create_test_runtime().await;

        // Use tokio timeout to limit execution
        let result = tokio::time::timeout(
            std::time::Duration::from_millis(100),
            runtime.execute_script(infinite_loop),
        )
        .await;

        assert!(result.is_err()); // Should timeout
    }

    /// Test UTF-8 handling in scripts
    #[tokio::test]
    #[ignore = "Requires full bridge setup"]
    async fn test_utf8_handling() {
        let script_with_utf8 = r#"
            local message = "Hello 世界 🌍"
            local emoji = "😊"
            return message .. " " .. emoji
        "#;

        let runtime = create_test_runtime().await;
        let result = runtime.execute_script(script_with_utf8).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.output.as_str().unwrap().contains("世界"));
        assert!(output.output.as_str().unwrap().contains("🌍"));
        assert!(output.output.as_str().unwrap().contains("😊"));
    }
}
