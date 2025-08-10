//! Simple performance check for hook overhead
#[tokio::test]
async fn test_simple_hook_overhead() {
    use llmspell_core::{types::AgentInput, ExecutionContext};
    use llmspell_tools::{
        lifecycle::hook_integration::{HookFeatures, ToolExecutor, ToolLifecycleConfig},
        util::calculator::CalculatorTool,
    };
    use serde_json::json;
    use std::time::Instant;

    let calculator = CalculatorTool::new();
    let input = AgentInput::text("test").with_parameter(
        "parameters",
        json!({"operation": "evaluate", "input": "2 + 2"}),
    );

    // Without hooks
    let config_no_hooks = ToolLifecycleConfig {
        features: HookFeatures {
            hooks_enabled: false,
            ..Default::default()
        },
        ..Default::default()
    };
    let executor_no_hooks = ToolExecutor::new(config_no_hooks, None, None);

    let start = Instant::now();
    for _ in 0..10 {
        let _ = executor_no_hooks
            .execute_tool_with_hooks(&calculator, input.clone(), ExecutionContext::default())
            .await
            .unwrap();
    }
    let duration_no_hooks = start.elapsed();

    // With hooks
    let config_with_hooks = ToolLifecycleConfig {
        features: HookFeatures {
            hooks_enabled: true,
            ..Default::default()
        },
        ..Default::default()
    };
    let executor_with_hooks = ToolExecutor::new(config_with_hooks, None, None);

    let start = Instant::now();
    for _ in 0..10 {
        let _ = executor_with_hooks
            .execute_tool_with_hooks(&calculator, input.clone(), ExecutionContext::default())
            .await
            .unwrap();
    }
    let duration_with_hooks = start.elapsed();

    // Calculate overhead
    let with_hooks_ms = duration_with_hooks.as_millis();
    let no_hooks_ms = duration_no_hooks.as_millis();
    let overhead_ms = with_hooks_ms.saturating_sub(no_hooks_ms);
    let overhead_percent = if no_hooks_ms > 0 {
        let overhead_ms_f64 = u64::try_from(overhead_ms).unwrap_or(u64::MAX) as f64;
        let no_hooks_ms_f64 = u64::try_from(no_hooks_ms).unwrap_or(u64::MAX) as f64;
        (overhead_ms_f64 / no_hooks_ms_f64) * 100.0
    } else {
        0.0
    };

    println!("No hooks: {duration_no_hooks:?}");
    println!("With hooks: {duration_with_hooks:?}");
    println!("Overhead: {overhead_percent:.2}%");

    // Pass if overhead is reasonable (< 25% for CI environments)
    assert!(
        overhead_percent < 25.0,
        "Hook overhead too high: {overhead_percent:.2}%"
    );
}
