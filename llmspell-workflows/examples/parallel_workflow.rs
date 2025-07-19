//! Example demonstrating parallel workflow capabilities
//! Shows fork-join pattern with concurrent branch execution

use llmspell_workflows::{
    traits::{StepType, WorkflowStep},
    ParallelBranch, ParallelWorkflowBuilder, WorkflowConfig,
};
use serde_json::json;
use std::time::Duration;
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("Parallel Workflow Examples");

    // Example 1: Simple parallel execution
    simple_parallel_example().await?;

    // Example 2: Mixed required and optional branches
    mixed_branches_example().await?;

    // Example 3: Fail-fast behavior
    fail_fast_example().await?;

    // Example 4: Concurrency limits
    concurrency_limits_example().await?;

    // Example 5: Complex multi-branch workflow
    complex_workflow_example().await?;

    // Example 6: Error handling and recovery
    error_handling_example().await?;

    Ok(())
}

/// Example: Simple parallel execution
async fn simple_parallel_example() -> Result<(), Box<dyn std::error::Error>> {
    info!("\n=== Simple Parallel Execution Example ===");

    let branch1 = ParallelBranch::new("data_fetch".to_string())
        .with_description("Fetch data from external API".to_string())
        .add_step(WorkflowStep::new(
            "fetch_step".to_string(),
            StepType::Tool {
                tool_name: "http_request".to_string(),
                parameters: json!({
                    "url": "https://api.example.com/data",
                    "method": "GET"
                }),
            },
        ));

    let branch2 = ParallelBranch::new("data_process".to_string())
        .with_description("Process local data".to_string())
        .add_step(WorkflowStep::new(
            "load_data".to_string(),
            StepType::Tool {
                tool_name: "file_operations".to_string(),
                parameters: json!({
                    "operation": "read",
                    "path": "./data.json"
                }),
            },
        ))
        .add_step(WorkflowStep::new(
            "transform_data".to_string(),
            StepType::Tool {
                tool_name: "json_processor".to_string(),
                parameters: json!({
                    "operation": "transform"
                }),
            },
        ));

    let workflow = ParallelWorkflowBuilder::new("simple_parallel")
        .description("Simple parallel workflow with two branches")
        .add_branch(branch1)
        .add_branch(branch2)
        .build()?;

    let result = workflow.execute().await?;

    info!("Simple parallel workflow completed:");
    info!("  Success: {}", result.success);
    info!("  Total branches: {}", result.branch_results.len());
    info!("  Successful branches: {}", result.successful_branches);
    info!("  Duration: {:?}", result.duration);

    Ok(())
}

/// Example: Mixed required and optional branches
async fn mixed_branches_example() -> Result<(), Box<dyn std::error::Error>> {
    info!("\n=== Mixed Branches Example ===");

    let required_branch = ParallelBranch::new("critical_task".to_string())
        .with_description("Critical task that must succeed".to_string())
        .add_step(WorkflowStep::new(
            "critical_step".to_string(),
            StepType::Tool {
                tool_name: "calculator".to_string(),
                parameters: json!({
                    "expression": "100 + 200"
                }),
            },
        ));

    let optional_branch1 = ParallelBranch::new("optional_task1".to_string())
        .with_description("Optional enrichment task".to_string())
        .optional()
        .add_step(WorkflowStep::new(
            "enrichment_step".to_string(),
            StepType::Custom {
                function_name: "enrich_data".to_string(),
                parameters: json!({}),
            },
        ));

    let optional_branch2 = ParallelBranch::new("optional_task2".to_string())
        .with_description("Optional validation task".to_string())
        .optional()
        .add_step(WorkflowStep::new(
            "validation_step".to_string(),
            StepType::Custom {
                function_name: "validate_data".to_string(),
                parameters: json!({}),
            },
        ));

    let workflow = ParallelWorkflowBuilder::new("mixed_branches")
        .description("Workflow with required and optional branches")
        .add_branch(required_branch)
        .add_branch(optional_branch1)
        .add_branch(optional_branch2)
        .continue_on_optional_failure(true)
        .build()?;

    let result = workflow.execute().await?;

    info!("Mixed branches workflow completed:");
    info!("  Success: {}", result.success);
    info!("  Branch results:");
    for branch_result in &result.branch_results {
        info!(
            "    - {} ({}): {}",
            branch_result.branch_name,
            if branch_result.required {
                "required"
            } else {
                "optional"
            },
            if branch_result.success { "✓" } else { "✗" }
        );
    }

    Ok(())
}

/// Example: Fail-fast behavior
async fn fail_fast_example() -> Result<(), Box<dyn std::error::Error>> {
    info!("\n=== Fail-Fast Example ===");

    let branch1 = ParallelBranch::new("quick_task".to_string()).add_step(WorkflowStep::new(
        "quick_step".to_string(),
        StepType::Custom {
            function_name: "quick_operation".to_string(),
            parameters: json!({"delay_ms": 100}),
        },
    ));

    let branch2 = ParallelBranch::new("failing_task".to_string()).add_step(WorkflowStep::new(
        "fail_step".to_string(),
        StepType::Tool {
            tool_name: "".to_string(), // Empty tool name causes failure
            parameters: json!({}),
        },
    ));

    let branch3 = ParallelBranch::new("slow_task".to_string()).add_step(WorkflowStep::new(
        "slow_step".to_string(),
        StepType::Custom {
            function_name: "slow_operation".to_string(),
            parameters: json!({"delay_ms": 5000}),
        },
    ));

    let workflow = ParallelWorkflowBuilder::new("fail_fast_demo")
        .description("Demonstrates fail-fast behavior")
        .add_branch(branch1)
        .add_branch(branch2)
        .add_branch(branch3)
        .fail_fast(true)
        .build()?;

    let result = workflow.execute().await?;

    info!("Fail-fast workflow completed:");
    info!("  Success: {}", result.success);
    info!("  Stopped early: {}", result.stopped_early);
    info!("  Branches executed: {}", result.branch_results.len());
    if let Some(error) = &result.error {
        info!("  Error: {}", error);
    }

    Ok(())
}

/// Example: Concurrency limits
async fn concurrency_limits_example() -> Result<(), Box<dyn std::error::Error>> {
    info!("\n=== Concurrency Limits Example ===");

    let mut branches = Vec::new();

    // Create 10 branches
    for i in 0..10 {
        let branch = ParallelBranch::new(format!("task_{}", i))
            .with_description(format!("Task number {}", i))
            .add_step(WorkflowStep::new(
                format!("step_{}", i),
                StepType::Custom {
                    function_name: "process_item".to_string(),
                    parameters: json!({
                        "item_id": i,
                        "delay_ms": 500
                    }),
                },
            ));
        branches.push(branch);
    }

    let mut builder = ParallelWorkflowBuilder::new("concurrency_limited")
        .description("Workflow with concurrency limits")
        .with_max_concurrency(3); // Only 3 branches run concurrently

    for branch in branches {
        builder = builder.add_branch(branch);
    }

    let workflow = builder.build()?;

    let start = std::time::Instant::now();
    let result = workflow.execute().await?;
    let elapsed = start.elapsed();

    info!("Concurrency limited workflow completed:");
    info!("  Total branches: 10");
    info!("  Max concurrency: 3");
    info!("  Total duration: {:?}", elapsed);
    info!("  Expected minimum: ~1.7s (10 tasks * 500ms / 3 concurrent)");
    info!("  Success: {}", result.success);

    Ok(())
}

/// Example: Complex multi-branch workflow
async fn complex_workflow_example() -> Result<(), Box<dyn std::error::Error>> {
    info!("\n=== Complex Multi-Branch Workflow Example ===");

    // Data ingestion branch
    let ingestion_branch = ParallelBranch::new("data_ingestion".to_string())
        .with_description("Ingest data from multiple sources".to_string())
        .with_timeout(Duration::from_secs(30))
        .add_step(WorkflowStep::new(
            "fetch_api_data".to_string(),
            StepType::Tool {
                tool_name: "http_request".to_string(),
                parameters: json!({
                    "url": "https://api.example.com/data",
                    "method": "GET"
                }),
            },
        ))
        .add_step(WorkflowStep::new(
            "fetch_db_data".to_string(),
            StepType::Tool {
                tool_name: "database_connector".to_string(),
                parameters: json!({
                    "query": "SELECT * FROM metrics"
                }),
            },
        ));

    // Validation branch
    let validation_branch = ParallelBranch::new("data_validation".to_string())
        .with_description("Validate data quality".to_string())
        .add_step(WorkflowStep::new(
            "schema_validation".to_string(),
            StepType::Tool {
                tool_name: "data_validation".to_string(),
                parameters: json!({
                    "schema": "metrics_schema.json"
                }),
            },
        ))
        .add_step(WorkflowStep::new(
            "business_rules".to_string(),
            StepType::Custom {
                function_name: "check_business_rules".to_string(),
                parameters: json!({}),
            },
        ));

    // Processing branch
    let processing_branch = ParallelBranch::new("data_processing".to_string())
        .with_description("Process and transform data".to_string())
        .add_step(WorkflowStep::new(
            "clean_data".to_string(),
            StepType::Tool {
                tool_name: "data_cleaner".to_string(),
                parameters: json!({
                    "remove_nulls": true,
                    "normalize": true
                }),
            },
        ))
        .add_step(WorkflowStep::new(
            "aggregate_data".to_string(),
            StepType::Tool {
                tool_name: "aggregator".to_string(),
                parameters: json!({
                    "group_by": ["category", "date"],
                    "metrics": ["sum", "avg", "count"]
                }),
            },
        ))
        .add_step(WorkflowStep::new(
            "enrich_data".to_string(),
            StepType::Custom {
                function_name: "enrich_with_metadata".to_string(),
                parameters: json!({}),
            },
        ));

    // Export branch (optional)
    let export_branch = ParallelBranch::new("data_export".to_string())
        .with_description("Export processed data".to_string())
        .optional()
        .add_step(WorkflowStep::new(
            "export_csv".to_string(),
            StepType::Tool {
                tool_name: "file_operations".to_string(),
                parameters: json!({
                    "operation": "write",
                    "format": "csv",
                    "path": "./output/results.csv"
                }),
            },
        ))
        .add_step(WorkflowStep::new(
            "upload_s3".to_string(),
            StepType::Tool {
                tool_name: "s3_uploader".to_string(),
                parameters: json!({
                    "bucket": "results-bucket",
                    "key": "processed/results.csv"
                }),
            },
        ));

    let workflow = ParallelWorkflowBuilder::new("data_pipeline")
        .description("Complex data processing pipeline")
        .add_branch(ingestion_branch)
        .add_branch(validation_branch)
        .add_branch(processing_branch)
        .add_branch(export_branch)
        .with_max_concurrency(3)
        .with_timeout(Duration::from_secs(120))
        .with_workflow_config(WorkflowConfig {
            max_execution_time: Some(Duration::from_secs(120)),
            ..Default::default()
        })
        .build()?;

    let result = workflow.execute().await?;

    info!("Complex workflow completed:");
    info!("{}", result.generate_report());

    Ok(())
}

/// Example: Error handling and recovery
async fn error_handling_example() -> Result<(), Box<dyn std::error::Error>> {
    info!("\n=== Error Handling Example ===");

    let reliable_branch =
        ParallelBranch::new("reliable_task".to_string()).add_step(WorkflowStep::new(
            "reliable_step".to_string(),
            StepType::Tool {
                tool_name: "calculator".to_string(),
                parameters: json!({"expression": "42 * 2"}),
            },
        ));

    let unreliable_branch = ParallelBranch::new("unreliable_task".to_string())
        .optional() // Make it optional so workflow can succeed
        .add_step(WorkflowStep::new(
            "flaky_step".to_string(),
            StepType::Custom {
                function_name: "flaky_operation".to_string(),
                parameters: json!({"fail_probability": 0.5}),
            },
        ));

    let recovery_branch = ParallelBranch::new("recovery_task".to_string())
        .optional()
        .with_description("Attempt recovery if other tasks fail".to_string())
        .add_step(WorkflowStep::new(
            "recovery_step".to_string(),
            StepType::Custom {
                function_name: "recover_state".to_string(),
                parameters: json!({}),
            },
        ));

    let workflow = ParallelWorkflowBuilder::new("error_handling")
        .description("Demonstrates error handling patterns")
        .add_branch(reliable_branch)
        .add_branch(unreliable_branch)
        .add_branch(recovery_branch)
        .fail_fast(false) // Continue even if branches fail
        .continue_on_optional_failure(true)
        .build()?;

    let result = workflow.execute().await?;

    info!("Error handling workflow completed:");
    info!("  Overall success: {}", result.success);
    info!("  Branch details:");
    for branch in &result.branch_results {
        info!(
            "    - {} ({}): {} - Duration: {:?}",
            branch.branch_name,
            if branch.required {
                "required"
            } else {
                "optional"
            },
            if branch.success { "Success" } else { "Failed" },
            branch.duration
        );
        if let Some(error) = &branch.error {
            info!("      Error: {}", error);
        }
    }

    Ok(())
}
