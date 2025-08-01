//! Simple performance check for hook overhead

#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_simple_hook_overhead() {
    use llmspell_core::{types::AgentInput, ExecutionContext};
    use llmspell_tools::{
        lifecycle::hook_integration::{ToolExecutor, ToolLifecycleConfig},
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
        enable_hooks: false,
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
        enable_hooks: true,
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
    let overhead_ms = duration_with_hooks.as_millis() as f64 - duration_no_hooks.as_millis() as f64;
    let overhead_percent = if duration_no_hooks.as_millis() > 0 {
        (overhead_ms / duration_no_hooks.as_millis() as f64) * 100.0
    } else {
        0.0
    };

    println!("No hooks: {:?}", duration_no_hooks);
    println!("With hooks: {:?}", duration_with_hooks);
    println!("Overhead: {:.2}%", overhead_percent);

    // Pass if overhead is reasonable (< 20% for CI environments)
    assert!(
        overhead_percent < 20.0,
        "Hook overhead too high: {:.2}%",
        overhead_percent
    );
}
