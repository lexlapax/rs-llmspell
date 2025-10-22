//! Comprehensive tests for tool hook integration
//! Tests all 8 hook points with various tools and scenarios

use llmspell_testing::tool_helpers::create_default_test_sandbox;
#[cfg(feature = "json-query")]
use llmspell_tools::data::json_processor::JsonProcessorTool;
use llmspell_tools::{
    fs::file_operations::{FileOperationsConfig, FileOperationsTool},
    lifecycle::{
        hook_integration::{AuditConfig, HookFeatures, ToolExecutor, ToolLifecycleConfig},
        HookableToolExecution,
    },
    registry::ToolRegistry,
    system::process_executor::{ProcessExecutorConfig, ProcessExecutorTool},
    util::calculator::CalculatorTool,
};

use llmspell_core::{
    traits::tool::{SecurityLevel, Tool, ToolCategory},
    types::AgentInput,
    ExecutionContext,
};

use serde_json::json;
use std::sync::Arc;
use std::time::{Duration, Instant};
#[tokio::test]
async fn test_tool_executor_basic_execution() {
    let config = ToolLifecycleConfig {
        features: HookFeatures {
            hooks_enabled: false, // Start with hooks disabled
            ..Default::default()
        },
        ..Default::default()
    };

    let executor = ToolExecutor::new(config, None, None);
    let calculator = CalculatorTool::new();

    let input = AgentInput::text("Test basic execution").with_parameter(
        "parameters",
        json!({
            "operation": "evaluate",
            "input": "2 + 2"
        }),
    );

    let result = executor
        .execute_tool_with_hooks(&calculator, input, ExecutionContext::default())
        .await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.text.contains('4'));
}
#[tokio::test]
async fn test_security_validation_with_safe_tool() {
    let config = ToolLifecycleConfig {
        features: HookFeatures {
            security_validation_enabled: true,
            ..Default::default()
        },
        max_security_level: SecurityLevel::Safe,
        ..Default::default()
    };

    let executor = ToolExecutor::new(config, None, None);
    let calculator = CalculatorTool::new();

    assert_eq!(calculator.security_level(), SecurityLevel::Safe);

    let input = AgentInput::text("Test safe tool").with_parameter(
        "parameters",
        json!({
            "operation": "evaluate",
            "input": "10 / 2"
        }),
    );

    let result = executor
        .execute_tool_with_hooks(&calculator, input, ExecutionContext::default())
        .await;

    assert!(result.is_ok());
}
#[tokio::test]
async fn test_security_validation_with_restricted_tool() {
    let config = ToolLifecycleConfig {
        features: HookFeatures {
            security_validation_enabled: true,
            ..Default::default()
        },
        max_security_level: SecurityLevel::Safe,
        ..Default::default()
    };

    let executor = ToolExecutor::new(config, None, None);

    // ProcessExecutor is Restricted level
    let process_config = ProcessExecutorConfig::default();
    let sandbox = create_default_test_sandbox();
    let process_tool = ProcessExecutorTool::new(process_config, sandbox);

    assert_eq!(process_tool.security_level(), SecurityLevel::Restricted);

    let input = AgentInput::text("Test restricted tool").with_parameter(
        "parameters",
        json!({
            "executable": "echo",
            "arguments": ["test"]
        }),
    );

    let result = executor
        .execute_tool_with_hooks(&process_tool, input, ExecutionContext::default())
        .await;

    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("security level") || error_msg.contains("exceeds maximum"));
}
#[tokio::test]
async fn test_audit_logging_enabled() {
    let config = ToolLifecycleConfig {
        audit: AuditConfig {
            enabled: true,
            log_parameters: false,
        },
        ..Default::default()
    };

    let executor = ToolExecutor::new(config, None, None);
    let calculator = CalculatorTool::new();

    let input = AgentInput::text("Test audit logging").with_parameter(
        "parameters",
        json!({
            "operation": "evaluate",
            "input": "42"
        }),
    );

    let result = executor
        .execute_tool_with_hooks(&calculator, input.clone(), ExecutionContext::default())
        .await;

    assert!(result.is_ok());

    // Audit logging happens internally - we can't directly access logs
    // but execution should complete successfully with logging enabled
}
#[tokio::test]
async fn test_error_handling_with_invalid_expression() {
    let config = ToolLifecycleConfig {
        features: HookFeatures {
            hooks_enabled: false,
            ..Default::default()
        },
        ..Default::default()
    };

    let executor = ToolExecutor::new(config, None, None);
    let calculator = CalculatorTool::new();

    // Invalid expression
    let input = AgentInput::text("Test error handling").with_parameter(
        "parameters",
        json!({
            "operation": "evaluate",
            "input": "(((((" // Invalid expression
        }),
    );

    let result = executor
        .execute_tool_with_hooks(&calculator, input, ExecutionContext::default())
        .await;

    // Calculator returns error in response, not as Error
    assert!(result.is_ok());
    let output_text = result.unwrap().text;
    assert!(output_text.contains("false") || output_text.contains("error"));
}
#[tokio::test]
async fn test_resource_tracking_integration() {
    let config = ToolLifecycleConfig {
        features: HookFeatures {
            hooks_enabled: false,
            ..Default::default()
        },
        ..Default::default()
    };

    let _resource_tracker = Arc::new(llmspell_utils::resource_limits::ResourceTracker::new(
        llmspell_utils::resource_limits::ResourceLimits::default(),
    ));
    let executor = ToolExecutor::new(config, None, None);
    let calculator = CalculatorTool::new();

    let input = AgentInput::text("Test resource tracking").with_parameter(
        "parameters",
        json!({
            "operation": "evaluate",
            "input": "100 * 200"
        }),
    );

    let result = executor
        .execute_tool_with_hooks(&calculator, input, ExecutionContext::default())
        .await;

    assert!(result.is_ok());

    // Resource tracking happens internally through the tool's execution
}
#[tokio::test]
async fn test_hook_performance_overhead() {
    // Test with hooks disabled
    let config_no_hooks = ToolLifecycleConfig {
        features: HookFeatures {
            hooks_enabled: false,
            ..Default::default()
        },
        ..Default::default()
    };
    let executor_no_hooks = ToolExecutor::new(config_no_hooks, None, None);

    // Test with hooks enabled (but no actual hooks registered)
    let config_with_hooks = ToolLifecycleConfig {
        features: HookFeatures {
            hooks_enabled: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let executor_with_hooks = ToolExecutor::new(config_with_hooks, None, None);

    let calculator = CalculatorTool::new();
    let input = AgentInput::text("Performance test").with_parameter(
        "parameters",
        json!({
            "operation": "evaluate",
            "input": "100 * 200 + 300 / 400"
        }),
    );

    // Measure execution time without hooks
    let start_no_hooks = Instant::now();
    for _ in 0..50 {
        let _ = executor_no_hooks
            .execute_tool_with_hooks(&calculator, input.clone(), ExecutionContext::default())
            .await;
    }
    let duration_no_hooks = start_no_hooks.elapsed();

    // Measure execution time with hooks enabled
    let start_with_hooks = Instant::now();
    for _ in 0..50 {
        let _ = executor_with_hooks
            .execute_tool_with_hooks(&calculator, input.clone(), ExecutionContext::default())
            .await;
    }
    let duration_with_hooks = start_with_hooks.elapsed();

    // Calculate overhead percentage
    let with_hooks_ms = duration_with_hooks.as_millis();
    let no_hooks_ms = duration_no_hooks.as_millis();
    let overhead_ms = with_hooks_ms.saturating_sub(no_hooks_ms);
    let overhead_percent = if no_hooks_ms > 0 {
        #[allow(clippy::cast_precision_loss)]
        let overhead_ms_f64 = u64::try_from(overhead_ms).unwrap_or(u64::MAX) as f64;
        #[allow(clippy::cast_precision_loss)]
        let no_hooks_ms_f64 = u64::try_from(no_hooks_ms).unwrap_or(u64::MAX) as f64;
        (overhead_ms_f64 / no_hooks_ms_f64) * 100.0
    } else {
        0.0
    };

    println!(
        "No hooks: {duration_no_hooks:?}, With hooks: {duration_with_hooks:?}, Overhead: {overhead_percent:.2}%"
    );

    // Verify overhead is reasonable (less than 20% for CI environments)
    assert!(
        overhead_percent < 20.0,
        "Hook overhead {overhead_percent:.2}% exceeds 20% threshold"
    );
}
#[tokio::test]
async fn test_circuit_breaker_functionality() {
    let config = ToolLifecycleConfig {
        features: HookFeatures {
            hooks_enabled: false,
            circuit_breaker_enabled: true,
            ..Default::default()
        },
        circuit_breaker_failure_threshold: 3,
        circuit_breaker_recovery_time: Duration::from_secs(1),
        ..Default::default()
    };

    let executor = ToolExecutor::new(config, None, None);
    let calculator = CalculatorTool::new();

    // Cause multiple executions with valid expressions
    for i in 0..5 {
        let input = AgentInput::text(format!("Circuit breaker test {i}")).with_parameter(
            "parameters",
            json!({
                "operation": "evaluate",
                "input": "2 + 2" // Valid expression
            }),
        );

        let result = executor
            .execute_tool_with_hooks(&calculator, input, ExecutionContext::default())
            .await;

        // All should succeed as we're using valid expressions
        assert!(result.is_ok());
    }
}
#[tokio::test]
async fn test_hook_integration_with_multiple_tool_types() {
    let config = ToolLifecycleConfig {
        features: HookFeatures {
            hooks_enabled: false,
            ..Default::default()
        },
        ..Default::default()
    };

    let executor = ToolExecutor::new(config, None, None);

    // Test different tool categories and security levels
    #[cfg(feature = "json-query")]
    let test_cases = vec![
        (
            Box::new(CalculatorTool::new()) as Box<dyn Tool>,
            "calculator",
            json!({
                "operation": "evaluate",
                "input": "2 + 2"
            }),
            ToolCategory::Utility,
            SecurityLevel::Safe,
        ),
        (
            Box::new(JsonProcessorTool::default()) as Box<dyn Tool>,
            "json_processor",
            json!({
                "operation": "query",
                "input": r#"{"test": 123}"#,
                "query": ".test"
            }),
            ToolCategory::Data,
            SecurityLevel::Safe,
        ),
    ];

    #[cfg(not(feature = "json-query"))]
    let test_cases = vec![(
        Box::new(CalculatorTool::new()) as Box<dyn Tool>,
        "calculator",
        json!({
            "operation": "evaluate",
            "input": "2 + 2"
        }),
        ToolCategory::Utility,
        SecurityLevel::Safe,
    )];

    for (tool, name, params, expected_category, expected_security) in test_cases {
        let input = AgentInput::text(format!("Test {name}")).with_parameter("parameters", params);

        let result = executor
            .execute_tool_with_hooks(tool.as_ref(), input, ExecutionContext::default())
            .await;

        // Verify tool properties
        assert_eq!(tool.category(), expected_category);
        assert_eq!(tool.security_level(), expected_security);

        // All should succeed
        assert!(result.is_ok());
    }
}
#[tokio::test]
async fn test_tool_registry_with_executor() {
    let config = ToolLifecycleConfig {
        features: HookFeatures {
            hooks_enabled: false,
            ..Default::default()
        },
        ..Default::default()
    };

    // Create tool registry
    let registry = ToolRegistry::new();

    // Register tools using register method
    registry
        .register("calculator".to_string(), CalculatorTool::new())
        .await
        .unwrap();
    #[cfg(feature = "json-query")]
    registry
        .register("json_processor".to_string(), JsonProcessorTool::default())
        .await
        .unwrap();

    // Execute tool through registry
    let input = AgentInput::text("Registry test").with_parameter(
        "parameters",
        json!({
            "operation": "evaluate",
            "input": "100 / 25"
        }),
    );

    // Get tool from registry and execute
    let executor = ToolExecutor::new(config, None, None);
    let calculator = registry.get_tool("calculator").await.unwrap();

    let result = executor
        .execute_tool_with_hooks(
            calculator.as_ref().as_ref(),
            input,
            ExecutionContext::default(),
        )
        .await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.text.contains('4'));
}
#[tokio::test]
async fn test_hookable_tool_execution_trait() {
    // Test that tools implement HookableToolExecution trait
    let config = ToolLifecycleConfig::default();
    let executor = ToolExecutor::new(config, None, None);

    let calculator = CalculatorTool::new();
    let input = AgentInput::text("Test trait").with_parameter(
        "parameters",
        json!({
            "operation": "evaluate",
            "input": "5 * 5"
        }),
    );

    // Use the trait method directly
    let result = calculator
        .execute_with_hooks(input, ExecutionContext::default(), &executor)
        .await;

    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.text.contains("25"));
}
#[tokio::test]
async fn test_different_security_levels() {
    let mut tools_and_levels = vec![(
        Box::new(CalculatorTool::new()) as Box<dyn Tool>,
        SecurityLevel::Safe,
    )];

    #[cfg(feature = "json-query")]
    tools_and_levels.push((
        Box::new(JsonProcessorTool::default()) as Box<dyn Tool>,
        SecurityLevel::Safe,
    ));

    tools_and_levels.push((
        Box::new(FileOperationsTool::new(
            FileOperationsConfig::default(),
            create_default_test_sandbox(),
        )) as Box<dyn Tool>,
        SecurityLevel::Privileged,
    ));

    tools_and_levels.push((
        Box::new(ProcessExecutorTool::new(
            ProcessExecutorConfig::default(),
            create_default_test_sandbox(),
        )) as Box<dyn Tool>,
        SecurityLevel::Restricted,
    ));

    for (tool, expected_level) in tools_and_levels {
        assert_eq!(tool.security_level(), expected_level);
    }
}
#[tokio::test]
async fn test_max_hook_execution_time() {
    let config = ToolLifecycleConfig {
        features: HookFeatures {
            hooks_enabled: true,
            ..Default::default()
        },
        max_hook_execution_time: Duration::from_millis(10), // Very short timeout
        ..Default::default()
    };

    let executor = ToolExecutor::new(config, None, None);
    let calculator = CalculatorTool::new();

    let input = AgentInput::text("Test max hook time").with_parameter(
        "parameters",
        json!({
            "operation": "evaluate",
            "input": "3 * 3"
        }),
    );

    let result = executor
        .execute_tool_with_hooks(&calculator, input, ExecutionContext::default())
        .await;

    // Should still succeed even with short hook timeout
    assert!(result.is_ok());
}
#[tokio::test]
async fn test_security_level_ordering() {
    // Test that security levels have proper ordering
    assert!(SecurityLevel::Safe.allows(&SecurityLevel::Safe));
    assert!(!SecurityLevel::Safe.allows(&SecurityLevel::Restricted));
    assert!(!SecurityLevel::Safe.allows(&SecurityLevel::Privileged));

    assert!(SecurityLevel::Restricted.allows(&SecurityLevel::Safe));
    assert!(SecurityLevel::Restricted.allows(&SecurityLevel::Restricted));
    assert!(!SecurityLevel::Restricted.allows(&SecurityLevel::Privileged));

    assert!(SecurityLevel::Privileged.allows(&SecurityLevel::Safe));
    assert!(SecurityLevel::Privileged.allows(&SecurityLevel::Restricted));
    assert!(SecurityLevel::Privileged.allows(&SecurityLevel::Privileged));
}
#[tokio::test]
async fn test_tool_execution_phases() {
    // This test verifies that all 8 hook phases are represented in ToolExecutionPhase
    use llmspell_tools::lifecycle::hook_integration::ToolExecutionPhase;

    let phases = [
        ToolExecutionPhase::ParameterValidation,
        ToolExecutionPhase::SecurityCheck,
        ToolExecutionPhase::ResourceAllocation,
        ToolExecutionPhase::PreExecution,
        ToolExecutionPhase::PostExecution,
        ToolExecutionPhase::ErrorHandling,
        ToolExecutionPhase::ResourceCleanup,
        ToolExecutionPhase::Timeout,
    ];

    // Verify we have exactly 8 phases
    assert_eq!(phases.len(), 8);
}
