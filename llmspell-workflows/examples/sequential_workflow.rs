//! Sequential workflow example
//!
//! This example demonstrates how to create and execute a sequential workflow
//! that processes data through multiple steps in order.

use llmspell_workflows::{
    ErrorStrategy, SequentialWorkflow, StepType, WorkflowConfig, WorkflowStep,
};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for logging
    tracing_subscriber::fmt::init();

    println!("🚀 Starting Sequential Workflow Example");

    // Create workflow configuration
    let mut config = WorkflowConfig::default();
    config.max_execution_time = Some(Duration::from_secs(60));
    config.default_step_timeout = Duration::from_secs(10);
    config.continue_on_error = false; // Fail fast for this example

    println!("📋 Creating workflow steps...");

    // Step 1: Load data
    let load_step = WorkflowStep::new(
        "load_data".to_string(),
        StepType::Tool {
            tool_name: "file_operations".to_string(),
            parameters: serde_json::json!({
                "operation": "read",
                "path": "input_data.json"
            }),
        },
    )
    .with_timeout(Duration::from_secs(5));

    // Step 2: Validate data
    let validate_step = WorkflowStep::new(
        "validate_data".to_string(),
        StepType::Custom {
            function_name: "validation".to_string(),
            parameters: serde_json::json!({
                "schema": "data_schema.json",
                "strict": true
            }),
        },
    )
    .with_retry(2);

    // Step 3: Transform data
    let transform_step = WorkflowStep::new(
        "transform_data".to_string(),
        StepType::Tool {
            tool_name: "json_processor".to_string(),
            parameters: serde_json::json!({
                "operation": "transform",
                "rules": ["normalize", "enrich"]
            }),
        },
    )
    .with_timeout(Duration::from_secs(15));

    // Step 4: Aggregate results
    let aggregate_step = WorkflowStep::new(
        "aggregate_results".to_string(),
        StepType::Custom {
            function_name: "aggregation".to_string(),
            parameters: serde_json::json!({
                "type": "sum",
                "group_by": "category"
            }),
        },
    );

    // Step 5: Save results
    let save_step = WorkflowStep::new(
        "save_results".to_string(),
        StepType::Tool {
            tool_name: "file_operations".to_string(),
            parameters: serde_json::json!({
                "operation": "write",
                "path": "output_results.json"
            }),
        },
    );

    // Create the workflow with all steps
    let workflow = SequentialWorkflow::builder("data_processing_pipeline".to_string())
        .with_config(config)
        .add_step(load_step)
        .add_step(validate_step)
        .add_step(transform_step)
        .add_step(aggregate_step)
        .add_step(save_step)
        .build();

    println!("✅ Added {} steps to workflow", workflow.step_count());

    // Set some initial shared data
    workflow
        .set_shared_data("batch_id".to_string(), serde_json::json!("batch_2024_001"))
        .await?;
    workflow
        .set_shared_data(
            "timestamp".to_string(),
            serde_json::json!(chrono::Utc::now().to_rfc3339()),
        )
        .await?;

    // Execute the workflow
    println!("🔄 Executing workflow...");
    let start_time = std::time::Instant::now();

    match workflow.execute().await {
        Ok(result) => {
            let execution_time = start_time.elapsed();
            println!("🎉 Workflow completed successfully in {:?}", execution_time);
            println!("📊 Workflow Report:");
            println!("{}", result.generate_report());

            println!("\n📋 Step Details:");
            for (index, step_result) in result.successful_steps.iter().enumerate() {
                println!(
                    "  {}. ✅ {} - Duration: {:?}",
                    index + 1,
                    step_result.step_name,
                    step_result.duration
                );
            }

            for (index, step_result) in result.failed_steps.iter().enumerate() {
                println!(
                    "  {}. ❌ {} - Duration: {:?}",
                    index + 1 + result.successful_steps.len(),
                    step_result.step_name,
                    step_result.duration
                );
                if let Some(error) = &step_result.error {
                    println!("     Error: {}", error);
                }
                if step_result.retry_count > 0 {
                    println!("     Retries: {}", step_result.retry_count);
                }
            }

            // Show execution statistics
            println!("\n📈 Execution Statistics:");
            let stats = workflow.get_execution_stats().await?;
            println!("{}", stats.generate_report());

            // Show final shared data
            println!("\n💾 Final Shared Data:");
            let batch_id = workflow.get_shared_data("batch_id").await?;
            let timestamp = workflow.get_shared_data("timestamp").await?;
            if let Some(id) = batch_id {
                println!("  batch_id: {}", id);
            }
            if let Some(ts) = timestamp {
                println!("  timestamp: {}", ts);
            }
        }
        Err(error) => {
            let execution_time = start_time.elapsed();
            println!("❌ Workflow failed after {:?}: {}", execution_time, error);

            // Show execution statistics even on failure
            let stats = workflow.get_execution_stats().await?;
            println!("\n📈 Partial Execution Statistics:");
            println!("{}", stats.generate_report());

            return Err(error.into());
        }
    }

    println!("\n🏁 Example completed successfully!");
    Ok(())
}

/// Example showing workflow with error handling strategies
#[allow(dead_code)]
async fn example_with_retry_strategy() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔄 Example: Workflow with Retry Strategy");

    let config = WorkflowConfig::default();
    let error_strategy = ErrorStrategy::Retry {
        max_attempts: 3,
        backoff_ms: 1000,
    };

    // Add a potentially failing step
    let unstable_step = WorkflowStep::new(
        "unstable_operation".to_string(),
        StepType::Tool {
            tool_name: "unreliable_service".to_string(),
            parameters: serde_json::json!({
                "operation": "fetch_data",
                "endpoint": "https://api.example.com/data"
            }),
        },
    )
    .with_retry(3);

    let workflow = SequentialWorkflow::builder("retry_example".to_string())
        .with_config(config)
        .with_error_strategy(error_strategy)
        .add_step(unstable_step)
        .build();

    match workflow.execute().await {
        Ok(result) => {
            println!("✅ Workflow with retry completed");
            if let Some(step_result) = result.successful_steps.first() {
                if step_result.retry_count > 0 {
                    println!(
                        "🔄 Step succeeded after {} retries",
                        step_result.retry_count
                    );
                }
            }
        }
        Err(error) => {
            println!("❌ Workflow failed even with retries: {}", error);
        }
    }

    Ok(())
}

/// Example showing workflow with continue-on-error strategy
#[allow(dead_code)]
async fn example_with_continue_strategy() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🚧 Example: Workflow with Continue-on-Error Strategy");

    let mut config = WorkflowConfig::default();
    config.continue_on_error = true;

    // Add steps, some of which might fail
    let steps = vec![
        (
            "stable_step1",
            "calculator",
            serde_json::json!({"expression": "2 + 2"}),
        ),
        ("failing_step", "nonexistent_tool", serde_json::json!({})),
        (
            "stable_step2",
            "json_processor",
            serde_json::json!({"input": {"test": true}}),
        ),
    ];

    let mut workflow_builder = SequentialWorkflow::builder("resilient_workflow".to_string())
        .with_config(config)
        .with_error_strategy(ErrorStrategy::Continue);

    for (name, tool, params) in steps {
        let step = WorkflowStep::new(
            name.to_string(),
            StepType::Tool {
                tool_name: tool.to_string(),
                parameters: params,
            },
        );
        workflow_builder = workflow_builder.add_step(step);
    }

    let workflow = workflow_builder.build();
    let result = workflow.execute().await?;

    println!(
        "📊 Results: {} successful, {} failed",
        result.successful_steps.len(),
        result.failed_steps.len()
    );
    println!(
        "🎯 Workflow completed despite {} failures",
        result.failed_steps.len()
    );

    Ok(())
}
