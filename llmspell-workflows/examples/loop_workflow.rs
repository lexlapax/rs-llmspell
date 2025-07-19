//! Example demonstrating loop workflow capabilities
//! Shows collection, range, and while-condition iterations with break conditions

use llmspell_workflows::{
    traits::{StepType, WorkflowStep},
    LoopWorkflowBuilder, ResultAggregation, WorkflowConfig,
};
use serde_json::json;
use std::time::Duration;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("Loop Workflow Examples");

    // Example 1: Range iteration
    range_example().await?;

    // Example 2: Collection iteration
    collection_example().await?;

    // Example 3: While condition with break
    while_condition_example().await?;

    // Example 4: Error handling and aggregation
    error_handling_example().await?;

    // Example 5: Aggregation strategies
    aggregation_strategies_example().await?;

    // Example 6: Nested data processing
    nested_data_example().await?;

    Ok(())
}

/// Example: Range iteration with accumulation
async fn range_example() -> Result<(), Box<dyn std::error::Error>> {
    info!("\n=== Range Iteration Example ===");

    let workflow = LoopWorkflowBuilder::new("range_sum")
        .description("Sum numbers from 1 to 10")
        .with_range(1, 11, 1) // 1 to 10 inclusive
        .add_step(WorkflowStep::new(
            "accumulate".to_string(),
            StepType::Custom {
                function_name: "add_to_sum".to_string(),
                parameters: json!({
                    "value": "$loop_value"
                }),
            },
        ))
        .with_aggregation(ResultAggregation::LastOnly)
        .build()?;

    let result = workflow.execute().await?;

    info!("Range workflow completed:");
    info!("  Total iterations: {}", result.total_iterations);
    info!("  Completed iterations: {}", result.completed_iterations);
    info!("  Success: {}", result.success);
    info!(
        "  Result: {:?}",
        result.aggregated_results.get("loop_metadata")
    );

    Ok(())
}

/// Example: Collection iteration
async fn collection_example() -> Result<(), Box<dyn std::error::Error>> {
    info!("\n=== Collection Iteration Example ===");

    let items = vec![
        json!({"name": "Alice", "age": 30}),
        json!({"name": "Bob", "age": 25}),
        json!({"name": "Charlie", "age": 35}),
    ];

    let workflow = LoopWorkflowBuilder::new("process_users")
        .description("Process user records")
        .with_collection(items)
        .add_step(WorkflowStep::new(
            "process_user".to_string(),
            StepType::Tool {
                tool_name: "user_processor".to_string(),
                parameters: json!({
                    "user": "$loop_value"
                }),
            },
        ))
        .add_step(WorkflowStep::new(
            "log_progress".to_string(),
            StepType::Custom {
                function_name: "log_user".to_string(),
                parameters: json!({
                    "index": "$loop_index",
                    "user": "$loop_value"
                }),
            },
        ))
        .with_aggregation(ResultAggregation::CollectAll)
        .build()?;

    let result = workflow.execute().await?;

    info!("Collection workflow completed:");
    info!("  Processed {} users", result.completed_iterations);
    info!("  Success: {}", result.success);

    Ok(())
}

/// Example: While condition with break
async fn while_condition_example() -> Result<(), Box<dyn std::error::Error>> {
    info!("\n=== While Condition Example ===");

    let workflow = LoopWorkflowBuilder::new("find_solution")
        .description("Search for solution until found")
        .with_while_condition("$found != true", 100) // Max 100 iterations
        .add_step(WorkflowStep::new(
            "search_step".to_string(),
            StepType::Custom {
                function_name: "search_solution".to_string(),
                parameters: json!({
                    "attempt": "$loop_index"
                }),
            },
        ))
        .add_break_condition(
            "$solution_quality > 0.9",
            Some("Found high-quality solution".to_string()),
        )
        .add_break_condition("$loop_index > 50", Some("Search limit reached".to_string()))
        .with_iteration_delay(Duration::from_millis(100))
        .build()?;

    let result = workflow.execute().await?;

    info!("While condition workflow completed:");
    info!("  Search attempts: {}", result.completed_iterations);
    info!("  Break reason: {:?}", result.break_reason);
    info!("  Success: {}", result.success);

    Ok(())
}

/// Example: Error handling and aggregation strategies
async fn error_handling_example() -> Result<(), Box<dyn std::error::Error>> {
    info!("\n=== Error Handling Example ===");

    let workflow = LoopWorkflowBuilder::new("resilient_processor")
        .description("Process with error resilience")
        .with_range(1, 11, 1)
        .add_step(WorkflowStep::new(
            "risky_operation".to_string(),
            StepType::Custom {
                function_name: "may_fail".to_string(),
                parameters: json!({
                    "value": "$loop_value",
                    "fail_on": [3, 7] // Simulate failures on these values
                }),
            },
        ))
        .continue_on_error(true) // Continue despite errors
        .with_aggregation(ResultAggregation::FirstN(5)) // Keep first 5 results
        .with_timeout(Duration::from_secs(30))
        .build()?;

    let result = workflow.execute().await?;

    info!("Error handling workflow completed:");
    info!("  Total iterations: {}", result.total_iterations);
    info!("  Completed iterations: {}", result.completed_iterations);
    info!("  Success: {}", result.success);
    if let Some(iterations) = result.aggregated_results.get("iterations") {
        info!("  First 5 results collected: {}", iterations);
    }

    Ok(())
}

/// Example: Complex workflow with multiple aggregation strategies
async fn aggregation_strategies_example() -> Result<(), Box<dyn std::error::Error>> {
    info!("\n=== Aggregation Strategies Example ===");

    // Test different aggregation strategies
    let strategies = vec![
        ("collect_all", ResultAggregation::CollectAll),
        ("last_only", ResultAggregation::LastOnly),
        ("first_3", ResultAggregation::FirstN(3)),
        ("last_3", ResultAggregation::LastN(3)),
        ("none", ResultAggregation::None),
    ];

    for (name, strategy) in strategies {
        info!("\nTesting aggregation strategy: {}", name);

        let workflow = LoopWorkflowBuilder::new(format!("test_{}", name))
            .with_range(1, 6, 1) // 5 iterations
            .add_step(WorkflowStep::new(
                "generate_data".to_string(),
                StepType::Custom {
                    function_name: "generate".to_string(),
                    parameters: json!({
                        "iteration": "$loop_index",
                        "value": "$loop_value"
                    }),
                },
            ))
            .with_aggregation(strategy)
            .build()?;

        let result = workflow.execute().await?;
        info!("  Results: {} keys", result.aggregated_results.len());
        for (key, _) in &result.aggregated_results {
            info!("    - {}", key);
        }
    }

    Ok(())
}

/// Example: Nested data processing
async fn nested_data_example() -> Result<(), Box<dyn std::error::Error>> {
    info!("\n=== Nested Data Processing Example ===");

    let departments = vec![
        json!({
            "name": "Engineering",
            "teams": ["Backend", "Frontend", "DevOps"]
        }),
        json!({
            "name": "Sales",
            "teams": ["Enterprise", "SMB", "Partnerships"]
        }),
    ];

    let workflow = LoopWorkflowBuilder::new("process_departments")
        .description("Process departments and their teams")
        .with_collection(departments)
        .add_step(WorkflowStep::new(
            "process_department".to_string(),
            StepType::Custom {
                function_name: "process_dept".to_string(),
                parameters: json!({
                    "department": "$loop_value",
                    "index": "$loop_index"
                }),
            },
        ))
        .add_step(WorkflowStep::new(
            "process_teams".to_string(),
            StepType::Custom {
                function_name: "process_teams".to_string(),
                parameters: json!({
                    "teams": "$loop_value.teams",
                    "department": "$loop_value.name"
                }),
            },
        ))
        .with_workflow_config(WorkflowConfig {
            max_execution_time: Some(Duration::from_secs(60)),
            ..Default::default()
        })
        .build()?;

    let result = workflow.execute().await?;

    info!("Nested data workflow completed:");
    info!("  Departments processed: {}", result.completed_iterations);
    info!("  Success: {}", result.success);

    Ok(())
}
