//! Comprehensive DAP (Debug Adapter Protocol) tests
//!
//! Tests launch, configuration, breakpoints, stepping, and event handling

#[cfg(test)]
mod dap_tests {
    use llmspell_core::traits::debug_context::DebugContext;
    use llmspell_kernel::debug::{
        dap::{DAPBridge, InitializeArguments, Source as DapSource, SourceBreakpoint},
        execution_bridge::{ExecutionManager, StepMode, StoppedEvent},
    };
    use serde_json::json;
    use std::sync::Arc;
    use tokio::sync::mpsc;

    /// Helper to create a test DAP bridge
    fn create_test_dap() -> DAPBridge {
        DAPBridge::new("test-session".to_string())
    }

    /// Helper to create a test execution manager
    fn create_test_execution_manager() -> Arc<ExecutionManager> {
        Arc::new(ExecutionManager::new("test-session".to_string()))
    }

    /// Test: Launch with debug=true enables debug mode
    #[test]
    fn test_launch_with_debug_true() {
        let mut dap = create_test_dap();
        let exec_mgr = create_test_execution_manager();
        dap.connect_execution_manager(Arc::clone(&exec_mgr));

        // Launch with debug=true
        let launch_args = json!({
            "request": "launch",
            "program": "/path/to/script.lua",
            "args": ["arg1", "arg2"],
            "noDebug": false,
            "stopOnEntry": false
        });

        let result = dap.handle_launch(&launch_args);
        assert!(result.is_ok(), "Launch should succeed");

        // Verify debug mode is enabled
        let debug_ctx: Arc<dyn DebugContext> = exec_mgr.clone();
        assert!(debug_ctx.is_debug_enabled(), "Debug mode should be enabled");
    }

    /// Test: Launch with noDebug=true disables debug mode
    #[test]
    fn test_launch_with_no_debug() {
        let mut dap = create_test_dap();
        let exec_mgr = create_test_execution_manager();
        dap.connect_execution_manager(Arc::clone(&exec_mgr));

        // Launch with noDebug=true
        let launch_args = json!({
            "request": "launch",
            "program": "/path/to/script.lua",
            "noDebug": true,
            "stopOnEntry": false
        });

        let result = dap.handle_launch(&launch_args);
        assert!(result.is_ok(), "Launch should succeed");

        // Verify debug mode is disabled
        let debug_ctx: Arc<dyn DebugContext> = exec_mgr.clone();
        assert!(
            !debug_ctx.is_debug_enabled(),
            "Debug mode should be disabled"
        );
    }

    /// Test: stopOnEntry sets breakpoint at line 1
    #[tokio::test]
    async fn test_stop_on_entry() {
        let mut dap = create_test_dap();
        let exec_mgr = create_test_execution_manager();
        dap.connect_execution_manager(Arc::clone(&exec_mgr));

        // Launch with stopOnEntry=true
        let launch_args = json!({
            "request": "launch",
            "program": "/path/to/script.lua",
            "stopOnEntry": true
        });

        dap.handle_launch(&launch_args).unwrap();

        // ConfigurationDone should set breakpoint at line 1
        let result = dap.handle_configuration_done();
        assert!(result.is_ok(), "ConfigurationDone should succeed");

        // Verify breakpoint was set
        let breakpoints = exec_mgr.get_breakpoints("/path/to/script.lua");
        assert!(
            !breakpoints.is_empty(),
            "Should have at least one breakpoint"
        );
        assert_eq!(
            breakpoints[0].line, 1,
            "Breakpoint should be at line 1 for stopOnEntry"
        );
    }

    /// Test: Arguments are passed correctly
    #[test]
    fn test_arguments_passed_correctly() {
        let mut dap = create_test_dap();
        let exec_mgr = create_test_execution_manager();
        dap.connect_execution_manager(Arc::clone(&exec_mgr));

        // Launch with various arguments
        let launch_args = json!({
            "request": "launch",
            "program": "/path/to/script.lua",
            "args": ["--verbose", "--input", "data.txt"],
            "env": {
                "DEBUG": "true",
                "LOG_LEVEL": "info"
            },
            "cwd": "/project/dir"
        });

        let result = dap.handle_launch(&launch_args);
        assert!(result.is_ok(), "Launch should succeed");

        // Retrieve stored arguments
        let stored_args = dap.get_launch_args();
        assert!(stored_args.is_some(), "Launch args should be stored");

        let args = stored_args.unwrap();
        assert_eq!(args["program"].as_str().unwrap(), "/path/to/script.lua");
        assert_eq!(
            args["args"].as_array().unwrap().len(),
            3,
            "Should have 3 arguments"
        );
        assert_eq!(args["env"]["DEBUG"].as_str().unwrap(), "true");
    }

    /// Test: Stopped event is sent when hitting breakpoint
    #[tokio::test]
    async fn test_stopped_event_on_breakpoint() {
        let mut dap = create_test_dap();
        let mut exec_mgr = ExecutionManager::new("test-session".to_string());

        // Create channel for stopped events
        let (tx, _rx) = mpsc::channel::<StoppedEvent>(10);
        exec_mgr.set_stopped_event_sender(tx);

        dap.connect_execution_manager(Arc::new(exec_mgr));

        // Set a breakpoint
        let source = DapSource {
            name: Some("test.lua".to_string()),
            path: Some("/test.lua".to_string()),
            source_reference: None,
            presentation_hint: None,
            origin: None,
        };
        let breakpoints = vec![SourceBreakpoint {
            line: 10,
            column: None,
            condition: None,
            hit_condition: None,
            log_message: None,
        }];
        dap.handle_set_breakpoints(&source, &breakpoints).unwrap();

        // Simulate hitting the breakpoint
        let event = StoppedEvent {
            reason: "breakpoint".to_string(),
            thread_id: 1,
            breakpoint_id: Some("bp-1".to_string()),
            file: "/test.lua".to_string(),
            line: 10,
        };

        // Send event through DAP (returns the JSON value)
        let _json_event = dap.send_stopped_event(
            &event.reason,
            event.thread_id,
            event.breakpoint_id.as_deref(),
        );

        // Verify event was sent (in real impl, this would go via IOPub)
        // For now, we just verify the method doesn't panic
    }

    /// Test: Initialize returns correct capabilities
    #[test]
    fn test_initialize_capabilities() {
        let dap = create_test_dap();

        let init_args = InitializeArguments {
            adapter_id: "llmspell".to_string(),
            locale: "en-US".to_string(),
            lines_start_at1: true,
            columns_start_at1: true,
            path_format: "path".to_string(),
            supports_variable_type: true,
            supports_variable_paging: false,
            supports_run_in_terminal_request: false,
        };

        let result = dap.handle_initialize(init_args);
        assert!(result.is_ok(), "Initialize should succeed");

        let capabilities = result.unwrap();
        assert!(capabilities.supports_configuration_done_request);
        assert!(capabilities.supports_conditional_breakpoints);
        assert!(capabilities.supports_evaluate_for_hovers);
        assert!(capabilities.supports_set_variable);
        assert!(capabilities.supports_terminate_request);
    }

    /// Test: Continue command resumes execution
    #[test]
    fn test_continue_command() {
        let mut dap = create_test_dap();
        let exec_mgr = create_test_execution_manager();
        dap.connect_execution_manager(Arc::clone(&exec_mgr));

        // Pause execution first
        exec_mgr.pause();
        assert!(exec_mgr.is_paused(), "Should be paused");

        // Continue command
        let result = dap.handle_continue();
        assert!(result.is_ok(), "Continue should succeed");

        // Verify execution resumed
        assert!(!exec_mgr.is_paused(), "Should not be paused after continue");
    }

    /// Test: Step commands work correctly
    #[test]
    fn test_step_commands() {
        let mut dap = create_test_dap();
        let exec_mgr = create_test_execution_manager();
        dap.connect_execution_manager(Arc::clone(&exec_mgr));

        // Test stepIn
        exec_mgr.pause();
        dap.handle_step_in().unwrap();
        assert_eq!(exec_mgr.get_step_mode(), Some(StepMode::StepIn));

        // Test stepOver (next)
        exec_mgr.pause();
        dap.handle_next().unwrap();
        assert_eq!(exec_mgr.get_step_mode(), Some(StepMode::StepOver));

        // Test stepOut
        exec_mgr.pause();
        dap.handle_step_out().unwrap();
        assert_eq!(exec_mgr.get_step_mode(), Some(StepMode::StepOut));
    }

    /// Test: Stack trace returns current frames
    #[test]
    fn test_stack_trace() {
        let mut dap = create_test_dap();
        let exec_mgr = create_test_execution_manager();

        // Add some test stack frames
        exec_mgr.push_frame("main".to_string(), "/test.lua".to_string(), 1, None);
        exec_mgr.push_frame(
            "function1".to_string(),
            "/test.lua".to_string(),
            10,
            Some(5),
        );

        dap.connect_execution_manager(Arc::clone(&exec_mgr));

        // Get stack trace
        let result = dap.handle_stack_trace(1);
        assert!(result.is_ok(), "Stack trace should succeed");

        let frames = result.unwrap();
        assert_eq!(frames.len(), 2, "Should have 2 frames");
        assert_eq!(frames[0].name, "function1", "Top frame should be function1");
        assert_eq!(frames[1].name, "main", "Bottom frame should be main");
    }

    /// Test: Disconnect cleans up properly
    #[test]
    fn test_disconnect() {
        let mut dap = create_test_dap();
        let exec_mgr = create_test_execution_manager();
        dap.connect_execution_manager(Arc::clone(&exec_mgr));

        // Enable debug mode
        let debug_ctx: Arc<dyn DebugContext> = exec_mgr.clone();
        debug_ctx.enable_debug_mode();

        // Mark as initialized to simulate connection
        dap.handle_initialize(InitializeArguments {
            adapter_id: "llmspell".to_string(),
            locale: "en-US".to_string(),
            lines_start_at1: true,
            columns_start_at1: true,
            path_format: "path".to_string(),
            supports_variable_type: true,
            supports_variable_paging: false,
            supports_run_in_terminal_request: false,
        })
        .unwrap();

        // Disconnect
        let result = dap.handle_disconnect();
        assert!(result.is_ok(), "Disconnect should succeed");

        // Verify cleanup
        assert!(
            !debug_ctx.is_debug_enabled(),
            "Debug mode should be disabled"
        );
        assert!(!dap.is_connected(), "DAP should not be connected");
    }

    /// Test: Late connection of ExecutionManager
    #[test]
    fn test_late_execution_manager_connection() {
        let mut dap = create_test_dap();

        // Launch before ExecutionManager is connected
        let launch_args = json!({
            "request": "launch",
            "program": "/path/to/script.lua",
            "noDebug": false
        });

        let result = dap.handle_launch(&launch_args);
        assert!(
            result.is_ok(),
            "Launch should succeed even without ExecutionManager"
        );

        // Connect ExecutionManager later
        let exec_mgr = create_test_execution_manager();
        dap.connect_execution_manager(Arc::clone(&exec_mgr));

        // Debug mode should be enabled on connection
        let debug_ctx: Arc<dyn DebugContext> = exec_mgr.clone();
        assert!(
            debug_ctx.is_debug_enabled(),
            "Debug mode should be enabled after connection"
        );
    }

    /// Test: Concurrent event handling
    #[tokio::test]
    async fn test_concurrent_events() {
        let dap = Arc::new(create_test_dap());
        let mut handles = vec![];

        // Spawn multiple tasks sending events
        for i in 0..5 {
            let dap_clone = Arc::clone(&dap);
            let handle = tokio::spawn(async move {
                dap_clone.send_event(
                    "output",
                    &json!({
                        "category": "stdout",
                        "output": format!("Message {}\n", i)
                    }),
                );
            });
            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }

        // No panics = success (events are thread-safe)
    }

    /// Test: Variables request with scopes
    #[test]
    fn test_variables_with_scopes() {
        let mut dap = create_test_dap();
        let exec_mgr = create_test_execution_manager();

        // Push a frame first so we can add variables to it
        exec_mgr.push_frame("test_frame".to_string(), "/test.lua".to_string(), 1, None);

        // Add variables to the frame
        exec_mgr.add_variable("local", "x", "42", "number");
        exec_mgr.add_variable("local", "name", "test", "string");

        dap.connect_execution_manager(Arc::clone(&exec_mgr));

        // Request scopes with frame ID 0
        let scopes_result = dap.handle_scopes(0);
        assert!(scopes_result.is_ok(), "Scopes request should succeed");

        let scopes = scopes_result.unwrap();
        assert!(!scopes.is_empty(), "Should have at least one scope");

        // Request variables for the scope
        let var_ref = scopes[0].variables_reference;
        let vars_result = dap.handle_variables(var_ref);
        assert!(vars_result.is_ok(), "Variables request should succeed");

        let vars = vars_result.unwrap();
        assert_eq!(vars.len(), 2, "Should have 2 variables");
        assert!(vars.iter().any(|v| v.name == "x" && v.value == "42"));
        assert!(vars.iter().any(|v| v.name == "name" && v.value == "test"));
    }

    /// Test: Evaluate expression
    #[test]
    fn test_evaluate_expression() {
        let mut dap = create_test_dap();
        let exec_mgr = create_test_execution_manager();
        dap.connect_execution_manager(Arc::clone(&exec_mgr));

        // Evaluate a simple expression
        let result = dap.handle_evaluate("2 + 2", &Some(0));
        assert!(result.is_ok(), "Evaluate should succeed");

        // In real implementation, this would call into the script engine
        // For now, just verify it doesn't panic
    }

    /// Test: Breakpoint conditions
    #[test]
    fn test_conditional_breakpoints() {
        let mut dap = create_test_dap();
        let exec_mgr = create_test_execution_manager();
        dap.connect_execution_manager(Arc::clone(&exec_mgr));

        // Set conditional breakpoint
        let source = DapSource {
            name: Some("test.lua".to_string()),
            path: Some("/test.lua".to_string()),
            source_reference: None,
            presentation_hint: None,
            origin: None,
        };
        let breakpoints = vec![SourceBreakpoint {
            line: 10,
            column: None,
            condition: Some("x > 5".to_string()),
            hit_condition: Some("3".to_string()),
            log_message: None,
        }];

        let result = dap.handle_set_breakpoints(&source, &breakpoints);
        assert!(
            result.is_ok(),
            "Setting conditional breakpoint should succeed"
        );

        let bps = result.unwrap();
        assert_eq!(bps.len(), 1, "Should have one breakpoint");
        assert!(bps[0].verified, "Breakpoint should be verified");
    }

    /// Test: Request sequence numbers increment
    #[test]
    fn test_request_sequence_numbers() {
        let mut dap = create_test_dap();
        let exec_mgr = create_test_execution_manager();
        dap.connect_execution_manager(Arc::clone(&exec_mgr));

        // Make multiple requests with proper arguments
        let req1 = json!({
            "command": "initialize",
            "seq": 1,
            "arguments": {
                "adapterId": "llmspell",
                "locale": "en-US",
                "linesStartAt1": true,
                "columnsStartAt1": true,
                "pathFormat": "path",
                "supportsVariableType": true,
                "supportsVariablePaging": false,
                "supportsRunInTerminalRequest": false
            }
        });
        let req2 = json!({
            "command": "launch",
            "seq": 2,
            "arguments": {
                "request": "launch",
                "program": "/test.lua"
            }
        });
        let req3 = json!({ "command": "continue", "seq": 3 });

        let resp1 = dap.handle_request(&req1).unwrap();
        let resp2 = dap.handle_request(&req2).unwrap();
        let resp3 = dap.handle_request(&req3).unwrap();

        // Responses should have matching sequence numbers
        assert_eq!(resp1["request_seq"].as_i64(), Some(1));
        assert_eq!(resp2["request_seq"].as_i64(), Some(2));
        assert_eq!(resp3["request_seq"].as_i64(), Some(3));
    }
}
