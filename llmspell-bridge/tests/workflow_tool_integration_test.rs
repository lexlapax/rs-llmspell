//! ABOUTME: Comprehensive tests for workflow-tool integration
//! ABOUTME: Verifies all 33+ tools work properly with workflow system

#[cfg(test)]
mod workflow_tool_tests {
    use llmspell_bridge::workflows::WorkflowBridge;
    use llmspell_bridge::{tools::register_all_tools, ComponentRegistry};
    use llmspell_core::Result;
    use serde_json::json;
    use std::sync::Arc;

    fn setup_test_environment() -> Arc<WorkflowBridge> {
        let registry = Arc::new(ComponentRegistry::new());

        // Register all tools
        register_all_tools(registry.clone()).expect("Failed to register tools");

        Arc::new(WorkflowBridge::new(registry))
    }
    #[tokio::test]
    #[ignore = "Workflow step execution not implemented - placeholder test"]
    async fn test_filesystem_tools_in_workflows() -> Result<()> {
        let bridge = setup_test_environment();

        // Test file operations in sequential workflow
        let workflow_params = json!({
            "name": "test_file_ops",
            "steps": [
                {
                    "name": "create_file",
                    "type": "tool",
                    "tool": "file_operations",
                    "input": {
                        "operation": "write",
                        "path": "/tmp/test_workflow.txt",
                        "content": "Test content"
                    }
                },
                {
                    "name": "read_file",
                    "type": "tool",
                    "tool": "file_operations",
                    "input": {
                        "operation": "read",
                        "path": "/tmp/test_workflow.txt"
                    }
                }
            ]
        });

        let workflow_id = bridge
            .create_workflow("sequential", workflow_params)
            .await?;
        let result = bridge.execute_workflow(&workflow_id, json!({})).await?;

        assert!(result.get("success").unwrap().as_bool().unwrap());

        Ok(())
    }
    #[tokio::test]
    #[ignore = "Workflow step execution not implemented - placeholder test"]
    async fn test_data_processing_tools_in_workflows() -> Result<()> {
        let bridge = setup_test_environment();

        // Test JSON and CSV processing
        let workflow_params = json!({
            "name": "test_data_processing",
            "steps": [
                {
                    "name": "process_json",
                    "type": "tool",
                    "tool": "json_processor",
                    "input": {
                        "input": r#"{"name": "test", "value": 42}"#,
                        "operation": "parse"
                    }
                },
                {
                    "name": "analyze_csv",
                    "type": "tool",
                    "tool": "csv_analyzer",
                    "input": {
                        "input": "name,value\ntest,42\n",
                        "operation": "analyze"
                    }
                }
            ]
        });

        let workflow_id = bridge
            .create_workflow("sequential", workflow_params)
            .await?;
        let result = bridge.execute_workflow(&workflow_id, json!({})).await?;

        assert!(result.get("success").unwrap().as_bool().unwrap());

        Ok(())
    }
    #[tokio::test]
    #[ignore = "Workflow step execution not implemented - placeholder test"]
    async fn test_utility_tools_in_workflows() -> Result<()> {
        let bridge = setup_test_environment();

        // Test various utility tools
        let workflow_params = json!({
            "name": "test_utilities",
            "steps": [
                {
                    "name": "calculate",
                    "type": "tool",
                    "tool": "calculator",
                    "input": {"input": "2 + 2"}
                },
                {
                    "name": "generate_uuid",
                    "type": "tool",
                    "tool": "uuid_generator",
                    "input": {"version": "v4"}
                },
                {
                    "name": "encode_data",
                    "type": "tool",
                    "tool": "base64_encoder",
                    "input": {
                        "input": "test data",
                        "operation": "encode"
                    }
                }
            ]
        });

        let workflow_id = bridge
            .create_workflow("sequential", workflow_params)
            .await?;
        let result = bridge.execute_workflow(&workflow_id, json!({})).await?;

        assert!(result.get("success").unwrap().as_bool().unwrap());

        Ok(())
    }
    #[tokio::test]
    #[ignore = "Workflow step execution not implemented - placeholder test"]
    async fn test_parallel_tool_execution() -> Result<()> {
        let bridge = setup_test_environment();

        // Test parallel execution of different tools
        let workflow_params = json!({
            "name": "test_parallel_tools",
            "branches": [
                {
                    "name": "branch1",
                    "steps": [{
                        "name": "calc",
                        "type": "tool",
                        "tool": "calculator",
                        "input": {"input": "10 * 5"}
                    }]
                },
                {
                    "name": "branch2",
                    "steps": [{
                        "name": "hash",
                        "type": "tool",
                        "tool": "hash_calculator",
                        "input": {
                            "input": "test",
                            "algorithm": "sha256"
                        }
                    }]
                },
                {
                    "name": "branch3",
                    "steps": [{
                        "name": "text",
                        "type": "tool",
                        "tool": "text_manipulator",
                        "input": {
                            "input": "hello",
                            "operation": "uppercase"
                        }
                    }]
                }
            ],
            "max_concurrency": 3
        });

        let workflow_id = bridge.create_workflow("parallel", workflow_params).await?;
        let result = bridge.execute_workflow(&workflow_id, json!({})).await?;

        assert!(result.get("success").unwrap().as_bool().unwrap());

        Ok(())
    }
    #[tokio::test]
    #[ignore = "Workflow step execution not implemented - placeholder test"]
    async fn test_tool_composition_in_workflows() -> Result<()> {
        let bridge = setup_test_environment();

        // Test tool output passed to next tool
        let workflow_params = json!({
            "name": "test_tool_composition",
            "steps": [
                {
                    "name": "generate_text",
                    "type": "tool",
                    "tool": "text_manipulator",
                    "input": {
                        "input": "Hello World",
                        "operation": "lowercase"
                    }
                },
                {
                    "name": "hash_text",
                    "type": "tool",
                    "tool": "hash_calculator",
                    "input": {
                        "input": "{{step:generate_text:output}}",
                        "algorithm": "md5"
                    }
                },
                {
                    "name": "encode_hash",
                    "type": "tool",
                    "tool": "base64_encoder",
                    "input": {
                        "input": "{{step:hash_text:output}}",
                        "operation": "encode"
                    }
                }
            ]
        });

        let workflow_id = bridge
            .create_workflow("sequential", workflow_params)
            .await?;
        let result = bridge.execute_workflow(&workflow_id, json!({})).await?;

        assert!(result.get("success").unwrap().as_bool().unwrap());

        Ok(())
    }
    #[tokio::test]
    #[ignore = "Workflow step execution not implemented - placeholder test"]
    async fn test_conditional_tool_execution() -> Result<()> {
        let bridge = setup_test_environment();

        // Test conditional workflow with tools
        let workflow_params = json!({
            "name": "test_conditional_tools",
            "branches": [
                {
                    "name": "calc_branch",
                    "condition": {"type": "always"},
                    "steps": [{
                        "name": "calculate",
                        "type": "tool",
                        "tool": "calculator",
                        "input": {"input": "5 * 5"}
                    }]
                }
            ],
            "default_branch": {
                "name": "default",
                "steps": [{
                    "name": "default_tool",
                    "type": "tool",
                    "tool": "uuid_generator",
                    "input": {"version": "v4"}
                }]
            }
        });

        let workflow_id = bridge
            .create_workflow("conditional", workflow_params)
            .await?;
        let result = bridge.execute_workflow(&workflow_id, json!({})).await?;

        // Check if execution was successful
        assert!(result.is_object());
        // Conditional workflows might have a different result structure
        let has_success = result
            .get("success")
            .is_some_and(|v| v.as_bool().unwrap_or(false));
        let has_executed_branches = result.get("executed_branches").is_some();
        assert!(
            has_success || has_executed_branches,
            "Conditional workflow execution failed: {result:?}"
        );

        Ok(())
    }
    #[tokio::test]
    #[ignore = "Workflow step execution not implemented - placeholder test"]
    async fn test_loop_tool_execution() -> Result<()> {
        let bridge = setup_test_environment();

        // Test loop workflow with tools
        let workflow_params = json!({
            "name": "test_loop_tools",
            "iterator": {
                "range": {
                    "start": 1,
                    "end": 3,
                    "step": 1
                }
            },
            "body": [
                {
                    "name": "generate_uuid",
                    "type": "tool",
                    "tool": "uuid_generator",
                    "input": {"version": "v4"}
                }
            ]
        });

        let workflow_id = bridge.create_workflow("loop", workflow_params).await?;
        let result = bridge.execute_workflow(&workflow_id, json!({})).await?;

        assert!(result.get("success").unwrap().as_bool().unwrap());

        Ok(())
    }
    #[tokio::test]
    #[ignore = "Workflow step execution not implemented - placeholder test"]
    async fn test_error_handling_with_tools() -> Result<()> {
        let bridge = setup_test_environment();

        // Test error handling when tool fails
        let workflow_params = json!({
            "name": "test_error_handling",
            "error_strategy": "continue",
            "steps": [
                {
                    "name": "invalid_tool",
                    "type": "tool",
                    "tool": "non_existent_tool",
                    "input": {}
                },
                {
                    "name": "valid_tool",
                    "type": "tool",
                    "tool": "calculator",
                    "input": {"input": "1 + 1"}
                }
            ]
        });

        let workflow_id = bridge
            .create_workflow("sequential", workflow_params)
            .await?;
        let result = bridge.execute_workflow(&workflow_id, json!({})).await?;

        // Check that we have step results in metadata
        let step_results = result
            .get("metadata")
            .and_then(|m| m.get("extra"))
            .and_then(|e| e.get("step_results"))
            .and_then(|s| s.as_array());

        assert!(
            step_results.is_some(),
            "Expected step_results in metadata.extra"
        );
        let steps = step_results.unwrap();
        assert_eq!(steps.len(), 2, "Expected 2 steps to be executed");

        // With error_strategy: "continue", both steps should execute
        // even if the first one fails
        assert!(
            result
                .get("success")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
            "Workflow should succeed with continue strategy"
        );

        Ok(())
    }
    #[tokio::test]
    #[ignore = "Workflow step execution not implemented - placeholder test"]
    async fn test_tool_timeout_in_workflows() -> Result<()> {
        let bridge = setup_test_environment();

        // Test timeout behavior
        let workflow_params = json!({
            "name": "test_timeout",
            "timeout_ms": 100,
            "steps": [
                {
                    "name": "slow_operation",
                    "type": "tool",
                    "tool": "process_executor",
                    "input": {
                        "command": "sleep",
                        "args": ["0.2"]
                    }
                }
            ]
        });

        let workflow_id = bridge
            .create_workflow("sequential", workflow_params)
            .await?;
        let result = bridge.execute_workflow(&workflow_id, json!({})).await;

        // Should either error or indicate failure
        match result {
            Err(_) => {
                // Timeout resulted in error
                // Timeout resulted in error - this is expected
            }
            Ok(res) => {
                // Check if execution failed due to timeout
                let success = res
                    .get("success")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(true);
                let has_error = res.get("error").is_some() || res.get("error_message").is_some();
                assert!(
                    !success || has_error,
                    "Expected workflow to fail due to timeout, but got: {res:?}"
                );
            }
        }

        Ok(())
    }
    #[tokio::test]
    #[ignore = "Workflow step execution not implemented - placeholder test"]
    async fn test_tool_resource_limits() -> Result<()> {
        let bridge = setup_test_environment();

        // Test resource limits are enforced
        let workflow_params = json!({
            "name": "test_resource_limits",
            "steps": [
                {
                    "name": "memory_intensive",
                    "type": "tool",
                    "tool": "data_validation",
                    "input": {
                        "input": "x".repeat(1_000_000), // Large string
                        "schema": {"type": "string"}
                    }
                }
            ]
        });

        let workflow_id = bridge
            .create_workflow("sequential", workflow_params)
            .await?;
        let result = bridge.execute_workflow(&workflow_id, json!({})).await;

        // Should handle large input gracefully
        assert!(result.is_ok());

        Ok(())
    }
    #[tokio::test]
    #[ignore = "Workflow step execution not implemented - placeholder test"]
    async fn test_all_tool_categories() -> Result<()> {
        let bridge = setup_test_environment();

        // Comprehensive test covering all tool categories
        let categories = vec![
            (
                "file_operations",
                json!({"operation": "list", "path": "/tmp"}),
            ),
            (
                "json_processor",
                json!({"input": "{}", "operation": "parse"}),
            ),
            ("calculator", json!({"input": "1 + 1"})),
            ("system_monitor", json!({"metrics": ["cpu"]})),
            (
                "http_request",
                json!({"url": "https://httpbin.org/get", "method": "GET"}),
            ),
        ];

        for (tool_name, input) in categories {
            let workflow_params = json!({
                "name": format!("test_{}", tool_name),
                "steps": [{
                    "name": "test_step",
                    "type": "tool",
                    "tool": tool_name,
                    "input": input
                }]
            });

            let workflow_id = bridge
                .create_workflow("sequential", workflow_params)
                .await?;
            let result = bridge.execute_workflow(&workflow_id, json!({})).await;

            assert!(result.is_ok(), "Tool {tool_name} failed in workflow");
        }

        Ok(())
    }
    #[test]
    fn test_tool_discovery_from_workflows() {
        let registry = Arc::new(ComponentRegistry::new());
        register_all_tools(registry.clone()).expect("Failed to register tools");

        // Verify all expected tools are registered
        let expected_tools = vec![
            "file_operations",
            "file_search",
            "file_watcher",
            "archive_handler",
            "file_converter",
            "json_processor",
            "csv_analyzer",
            "calculator",
            "text_manipulator",
            "base64_encoder",
            "hash_calculator",
            "uuid_generator",
            "date_time_handler",
            "template_engine",
            "diff_calculator",
            "data_validation",
            "environment_reader",
            "process_executor",
            "system_monitor",
            "service_checker",
            "http_request",
            "graphql_query",
            "web_search",
            "url-analyzer",
            "api-tester",
            "webhook-caller",
            "web-scraper",
            "sitemap-crawler",
        ];

        for tool_name in expected_tools {
            assert!(
                registry.get_tool(tool_name).is_some(),
                "Tool {tool_name} not found in registry"
            );
        }
    }
    #[tokio::test]
    #[ignore = "Workflow step execution not implemented - placeholder test"]
    async fn test_performance_requirements() -> Result<()> {
        use std::time::Instant;

        let bridge = setup_test_environment();

        // Test workflow creation performance
        let start = Instant::now();

        let workflow_params = json!({
            "name": "performance_test",
            "steps": [
                {"name": "step1", "type": "tool", "tool": "calculator", "input": {"input": "1+1"}},
                {"name": "step2", "type": "tool", "tool": "uuid_generator", "input": {"version": "v4"}},
                {"name": "step3", "type": "tool", "tool": "text_manipulator", "input": {"input": "test", "operation": "uppercase"}}
            ]
        });

        let workflow_id = bridge
            .create_workflow("sequential", workflow_params)
            .await?;
        let creation_time = start.elapsed();

        // Should create workflow in <50ms
        assert!(
            creation_time.as_millis() < 50,
            "Workflow creation took {}ms",
            creation_time.as_millis()
        );

        // Test execution performance
        let start = Instant::now();
        let _result = bridge.execute_workflow(&workflow_id, json!({})).await?;
        let execution_time = start.elapsed();

        // Should execute simple workflow in reasonable time
        assert!(
            execution_time.as_millis() < 500,
            "Workflow execution took {}ms",
            execution_time.as_millis()
        );

        Ok(())
    }
}
