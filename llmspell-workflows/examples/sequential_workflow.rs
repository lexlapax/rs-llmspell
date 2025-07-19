//! Basic sequential workflow example
//!
//! This example demonstrates how to create and execute a basic sequential workflow
//! that processes data through multiple steps in order.

use llmspell_workflows::{
    basic::{
        BasicErrorStrategy, BasicSequentialWorkflow, BasicStepType, BasicWorkflowConfig,
        BasicWorkflowStep,
    },
    BasicWorkflow,
};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing for logging
    tracing_subscriber::fmt::init();

    println!("🚀 Starting Basic Sequential Workflow Example");

    // Create workflow configuration
    let mut config = BasicWorkflowConfig::default();
    config.max_execution_time = Some(Duration::from_secs(60));
    config.default_step_timeout = Duration::from_secs(10);
    config.continue_on_error = false; // Fail fast for this example

    // Create the workflow
    let mut workflow = BasicSequentialWorkflow::new("data_processing_pipeline".to_string(), config);

    println!("📋 Creating workflow steps...");

    // Step 1: Load data
    let load_step = BasicWorkflowStep::new(
        "load_data".to_string(),
        BasicStepType::Tool {
            tool_name: "file_operations".to_string(),
            parameters: serde_json::json!({
                "operation": "read",
                "path": "input_data.json"
            }),
        },
    )
    .with_timeout(Duration::from_secs(5));

    // Step 2: Validate data
    let validate_step = BasicWorkflowStep::new(
        "validate_data".to_string(),
        BasicStepType::Custom {
            function_name: "validation".to_string(),
            parameters: serde_json::json!({
                "schema": "data_schema.json",
                "strict": true
            }),
        },
    )
    .with_retry(2);

    // Step 3: Transform data
    let transform_step = BasicWorkflowStep::new(
        "transform_data".to_string(),
        BasicStepType::Tool {
            tool_name: "json_processor".to_string(),
            parameters: serde_json::json!({
                "operation": "transform",
                "rules": ["normalize", "enrich"]
            }),
        },
    )
    .with_timeout(Duration::from_secs(15));

    // Step 4: Aggregate results
    let aggregate_step = BasicWorkflowStep::new(
        "aggregate_results".to_string(),
        BasicStepType::Custom {
            function_name: "aggregation".to_string(),
            parameters: serde_json::json!({
                "type": "sum",
                "group_by": "category"
            }),
        },
    );

    // Step 5: Save results
    let save_step = BasicWorkflowStep::new(
        "save_results".to_string(),
        BasicStepType::Tool {
            tool_name: "file_operations".to_string(),
            parameters: serde_json::json!({
                "operation": "write",
                "path": "output_results.json"
            }),
        },
    );

    // Add all steps to the workflow
    workflow.add_step(load_step).await?;
    workflow.add_step(validate_step).await?;
    workflow.add_step(transform_step).await?;
    workflow.add_step(aggregate_step).await?;
    workflow.add_step(save_step).await?;

    println!("✅ Added {} steps to workflow", workflow.step_count());

    // Validate the workflow before execution
    println!("🔍 Validating workflow...");
    workflow.validate().await?;
    println!("✅ Workflow validation passed");

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
        Ok(results) => {
            let execution_time = start_time.elapsed();
            println!("🎉 Workflow completed successfully in {:?}", execution_time);
            println!("📊 Step Results:");

            for (index, result) in results.iter().enumerate() {
                let status = if result.success { "✅" } else { "❌" };
                println!(
                    "  {}. {} {} - Duration: {:?}",
                    index + 1,
                    status,
                    result.step_name,
                    result.duration
                );

                if !result.success {
                    if let Some(error) = &result.error {
                        println!("     Error: {}", error);
                    }
                    if result.retry_count > 0 {
                        println!("     Retries: {}", result.retry_count);
                    }
                }
            }

            // Show execution statistics
            println!("\n📈 Execution Statistics:");
            let stats = workflow.get_stats().await?;
            println!("{}", stats.generate_report());

            // Show final shared data
            println!("\n💾 Final Shared Data:");
            let state_snapshot = workflow.get_state_snapshot().await?;
            for (key, value) in &state_snapshot.shared_data {
                println!("  {}: {}", key, value);
            }
        }
        Err(error) => {
            let execution_time = start_time.elapsed();
            println!("❌ Workflow failed after {:?}: {}", execution_time, error);

            // Show partial results
            let results = workflow.get_results().await?;
            if !results.is_empty() {
                println!("\n📊 Partial Results (before failure):");
                for (index, result) in results.iter().enumerate() {
                    let status = if result.success { "✅" } else { "❌" };
                    println!(
                        "  {}. {} {} - Duration: {:?}",
                        index + 1,
                        status,
                        result.step_name,
                        result.duration
                    );
                }
            }

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

    let config = BasicWorkflowConfig::default();
    let error_strategy = BasicErrorStrategy::Retry {
        max_attempts: 3,
        backoff_ms: 1000,
    };

    let mut workflow = BasicSequentialWorkflow::with_error_strategy(
        "retry_example".to_string(),
        config,
        error_strategy,
    );

    // Add a potentially failing step
    let unstable_step = BasicWorkflowStep::new(
        "unstable_operation".to_string(),
        BasicStepType::Tool {
            tool_name: "unreliable_service".to_string(),
            parameters: serde_json::json!({
                "operation": "fetch_data",
                "endpoint": "https://api.example.com/data"
            }),
        },
    )
    .with_retry(3);

    workflow.add_step(unstable_step).await?;

    match workflow.execute().await {
        Ok(results) => {
            println!("✅ Workflow with retry completed");
            if let Some(result) = results.first() {
                if result.retry_count > 0 {
                    println!("🔄 Step succeeded after {} retries", result.retry_count);
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

    let mut config = BasicWorkflowConfig::default();
    config.continue_on_error = true;

    let mut workflow = BasicSequentialWorkflow::new("resilient_workflow".to_string(), config);

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

    for (name, tool, params) in steps {
        let step = BasicWorkflowStep::new(
            name.to_string(),
            BasicStepType::Tool {
                tool_name: tool.to_string(),
                parameters: params,
            },
        );
        workflow.add_step(step).await?;
    }

    let results = workflow.execute().await?;
    let successful = results.iter().filter(|r| r.success).count();
    let failed = results.len() - successful;

    println!("📊 Results: {} successful, {} failed", successful, failed);
    println!("🎯 Workflow completed despite {} failures", failed);

    Ok(())
}
