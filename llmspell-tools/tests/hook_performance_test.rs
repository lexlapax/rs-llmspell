//! Performance tests for hook integration overhead
//! Measures overhead and ensures it's under 2% threshold

use llmspell_core::{types::AgentInput, ExecutionContext};
use llmspell_tools::{
    lifecycle::hook_integration::{AuditConfig, HookFeatures, ToolExecutor, ToolLifecycleConfig},
    util::calculator::CalculatorTool,
};
use serde_json::json;
use std::time::{Duration, Instant};
#[tokio::test]
async fn test_hook_overhead_under_5_percent() {
    // Test configuration
    const ITERATIONS: usize = 100;
    const MAX_OVERHEAD_PERCENT: f64 = 5.0;

    // Create calculator tool
    let calculator = CalculatorTool::new();

    // Test cases with varying complexity
    let test_cases = vec![
        ("simple", json!({"operation": "evaluate", "input": "2 + 2"})),
        (
            "medium",
            json!({"operation": "evaluate", "input": "10 * 20 + 30 / 40"}),
        ),
        (
            "complex",
            json!({
                "operation": "evaluate",
                "input": "sin(pi()/2) + cos(0) * log(10, 100)"
            }),
        ),
        (
            "variables",
            json!({
                "operation": "evaluate",
                "input": "x^2 + y^2",
                "variables": {"x": 3, "y": 4}
            }),
        ),
    ];

    for (name, params) in test_cases {
        println!("\nTesting hook overhead for: {name}");

        // Create executors
        let config_no_hooks = ToolLifecycleConfig {
            features: HookFeatures {
                hooks_enabled: false,
                ..Default::default()
            },
            ..Default::default()
        };
        let executor_no_hooks = ToolExecutor::new(config_no_hooks, None, None);

        let config_with_hooks = ToolLifecycleConfig {
            features: HookFeatures {
                hooks_enabled: true,
                security_validation_enabled: true,
                circuit_breaker_enabled: true,
            },
            audit: AuditConfig {
                enabled: true,
                log_parameters: false,
            },
            ..Default::default()
        };
        let executor_with_hooks = ToolExecutor::new(config_with_hooks, None, None);

        // Warm up
        for _ in 0..10 {
            let input = AgentInput::text("warmup").with_parameter("parameters", params.clone());
            let _ = executor_no_hooks
                .execute_tool_with_hooks(&calculator, input.clone(), ExecutionContext::default())
                .await;
            let _ = executor_with_hooks
                .execute_tool_with_hooks(&calculator, input, ExecutionContext::default())
                .await;
        }

        // Measure without hooks
        let mut total_no_hooks = Duration::ZERO;
        for _ in 0..ITERATIONS {
            let input = AgentInput::text("test").with_parameter("parameters", params.clone());
            let start = Instant::now();
            let _ = executor_no_hooks
                .execute_tool_with_hooks(&calculator, input, ExecutionContext::default())
                .await
                .unwrap();
            total_no_hooks += start.elapsed();
        }
        let avg_no_hooks =
            total_no_hooks / u32::try_from(ITERATIONS).expect("ITERATIONS should fit in u32");

        // Measure with hooks
        let mut total_with_hooks = Duration::ZERO;
        for _ in 0..ITERATIONS {
            let input = AgentInput::text("test").with_parameter("parameters", params.clone());
            let start = Instant::now();
            let _ = executor_with_hooks
                .execute_tool_with_hooks(&calculator, input, ExecutionContext::default())
                .await
                .unwrap();
            total_with_hooks += start.elapsed();
        }
        let avg_with_hooks =
            total_with_hooks / u32::try_from(ITERATIONS).expect("ITERATIONS should fit in u32");

        // Calculate overhead
        let overhead_micros = avg_with_hooks
            .as_micros()
            .saturating_sub(avg_no_hooks.as_micros());
        let overhead_percent = if avg_no_hooks.as_micros() > 0 {
            (f64::from(u32::try_from(overhead_micros).unwrap_or(u32::MAX))
                / f64::from(u32::try_from(avg_no_hooks.as_micros()).unwrap_or(u32::MAX)))
                * 100.0
        } else {
            0.0
        };

        println!(
            "  No hooks: {avg_no_hooks:?}, With hooks: {avg_with_hooks:?}, Overhead: {overhead_percent:.2}%"
        );

        // Assert overhead is under threshold
        assert!(
            overhead_percent < MAX_OVERHEAD_PERCENT,
            "Hook overhead {overhead_percent:.2}% exceeds {MAX_OVERHEAD_PERCENT}% threshold for test case: {name}"
        );
    }
}
#[tokio::test]
async fn test_circuit_breaker_performance() {
    let config = ToolLifecycleConfig {
        features: HookFeatures {
            hooks_enabled: true,
            circuit_breaker_enabled: true,
            ..Default::default()
        },
        circuit_breaker_failure_threshold: 3,
        circuit_breaker_recovery_time: Duration::from_millis(100),
        ..Default::default()
    };
    let executor = ToolExecutor::new(config, None, None);
    let calculator = CalculatorTool::new();

    // Measure circuit breaker overhead with healthy requests
    let iterations = 50;
    let start = Instant::now();

    for i in 0..iterations {
        let input = AgentInput::text("test").with_parameter(
            "parameters",
            json!({"operation": "evaluate", "input": format!("{} + {}", i, i)}),
        );
        let _ = executor
            .execute_tool_with_hooks(&calculator, input, ExecutionContext::default())
            .await
            .unwrap();
    }

    let total_duration = start.elapsed();
    let avg_duration =
        total_duration / u32::try_from(iterations).expect("iterations should be positive");

    println!("\nCircuit breaker performance:");
    println!("  Average execution time: {avg_duration:?}");
    println!("  Total time for {iterations} requests: {total_duration:?}");

    // Ensure circuit breaker doesn't add significant overhead
    assert!(
        avg_duration < Duration::from_millis(10),
        "Circuit breaker adds too much overhead: {avg_duration:?}"
    );
}
#[tokio::test]
async fn test_resource_tracking_overhead() {
    let config = ToolLifecycleConfig {
        features: HookFeatures {
            hooks_enabled: true,
            ..Default::default()
        },
        ..Default::default()
    };

    let executor = ToolExecutor::new(config, None, None);
    let calculator = CalculatorTool::new();

    // Test resource tracking overhead
    let iterations = 50;
    let start = Instant::now();

    for _ in 0..iterations {
        let input = AgentInput::text("test").with_parameter(
            "parameters",
            json!({"operation": "evaluate", "input": "100 * 200 + 300"}),
        );
        let _ = executor
            .execute_tool_with_hooks(&calculator, input, ExecutionContext::default())
            .await
            .unwrap();
    }

    let total_duration = start.elapsed();
    let avg_duration =
        total_duration / u32::try_from(iterations).expect("iterations should be positive");

    println!("\nResource tracking performance:");
    println!("  Average execution time: {avg_duration:?}");

    // Resource tracking should have minimal overhead
    assert!(
        avg_duration < Duration::from_millis(5),
        "Resource tracking adds too much overhead: {avg_duration:?}"
    );
}
#[tokio::test]
async fn test_hook_execution_time_limit() {
    let config = ToolLifecycleConfig {
        features: HookFeatures {
            hooks_enabled: true,
            ..Default::default()
        },
        max_hook_execution_time: Duration::from_millis(50), // Very short timeout
        ..Default::default()
    };
    let executor = ToolExecutor::new(config, None, None);
    let calculator = CalculatorTool::new();

    // Execute with hook timeout
    let start = Instant::now();
    let input = AgentInput::text("test").with_parameter(
        "parameters",
        json!({"operation": "evaluate", "input": "42"}),
    );
    let result = executor
        .execute_tool_with_hooks(&calculator, input, ExecutionContext::default())
        .await;
    let duration = start.elapsed();

    // Should succeed even with short timeout
    assert!(result.is_ok());

    // Total execution should not be significantly impacted by hook timeout
    assert!(
        duration < Duration::from_millis(100),
        "Hook timeout handling takes too long: {duration:?}"
    );
}
#[tokio::test]
async fn test_audit_logging_performance_impact() {
    let iterations = 50;

    // Test with audit logging disabled
    let config_no_audit = ToolLifecycleConfig {
        features: HookFeatures {
            hooks_enabled: true,
            ..Default::default()
        },
        audit: AuditConfig {
            enabled: false,
            log_parameters: false,
        },
        ..Default::default()
    };
    let executor_no_audit = ToolExecutor::new(config_no_audit, None, None);

    // Test with audit logging enabled
    let config_with_audit = ToolLifecycleConfig {
        features: HookFeatures {
            hooks_enabled: true,
            ..Default::default()
        },
        audit: AuditConfig {
            enabled: true,
            log_parameters: true, // Full logging
        },
        ..Default::default()
    };
    let executor_with_audit = ToolExecutor::new(config_with_audit, None, None);

    let calculator = CalculatorTool::new();

    // Measure without audit logging
    let start_no_audit = Instant::now();
    for i in 0..iterations {
        let input = AgentInput::text("test").with_parameter(
            "parameters",
            json!({"operation": "evaluate", "input": format!("{} * 2", i)}),
        );
        let _ = executor_no_audit
            .execute_tool_with_hooks(&calculator, input, ExecutionContext::default())
            .await
            .unwrap();
    }
    let duration_no_audit = start_no_audit.elapsed();

    // Measure with audit logging
    let start_with_audit = Instant::now();
    for i in 0..iterations {
        let input = AgentInput::text("test").with_parameter(
            "parameters",
            json!({"operation": "evaluate", "input": format!("{} * 2", i)}),
        );
        let _ = executor_with_audit
            .execute_tool_with_hooks(&calculator, input, ExecutionContext::default())
            .await
            .unwrap();
    }
    let duration_with_audit = start_with_audit.elapsed();

    // Calculate audit logging overhead
    let audit_millis = duration_with_audit.as_millis();
    let no_audit_millis = duration_no_audit.as_millis();
    let overhead_millis = audit_millis.saturating_sub(no_audit_millis);
    let overhead_percent = if no_audit_millis > 0 {
        ((overhead_millis as f64) / (no_audit_millis as f64)) * 100.0
    } else {
        0.0
    };

    println!("\nAudit logging performance:");
    println!("  Without audit: {duration_no_audit:?}");
    println!("  With audit: {duration_with_audit:?}");
    println!("  Overhead: {overhead_percent:.2}%");

    // Audit logging should have minimal impact
    assert!(
        overhead_percent < 5.0,
        "Audit logging overhead {overhead_percent:.2}% exceeds 5% threshold"
    );
}
