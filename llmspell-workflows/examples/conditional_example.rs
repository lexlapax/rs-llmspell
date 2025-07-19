//! ABOUTME: Example demonstrating basic conditional workflow patterns
//! ABOUTME: Shows condition evaluation, branch selection, and default branch handling

use llmspell_workflows::basic::{
    BasicCondition, BasicConditionalWorkflow, BasicStepType, BasicWorkflow, BasicWorkflowConfig,
    BasicWorkflowStep, ConditionalBranch, ConditionalWorkflowConfig,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔀 Basic Conditional Workflow Example");
    println!("=====================================\n");

    // Create workflow configuration
    let workflow_config = BasicWorkflowConfig::default();
    let conditional_config = ConditionalWorkflowConfig::default();

    let mut workflow = BasicConditionalWorkflow::new(
        "data_processor".to_string(),
        workflow_config,
        conditional_config,
    );

    println!("📋 Setting up conditional workflow branches...");

    // Branch 1: Process CSV data
    let csv_condition =
        BasicCondition::shared_data_equals("data_type".to_string(), serde_json::json!("csv"));
    let csv_step = BasicWorkflowStep::new(
        "process_csv".to_string(),
        BasicStepType::Tool {
            tool_name: "csv_analyzer".to_string(),
            parameters: serde_json::json!({"delimiter": ",", "headers": true}),
        },
    );
    let csv_branch =
        ConditionalBranch::new("CSV Processing".to_string(), csv_condition).with_step(csv_step);

    workflow.add_branch(csv_branch).await?;

    // Branch 2: Process JSON data
    let json_condition =
        BasicCondition::shared_data_equals("data_type".to_string(), serde_json::json!("json"));
    let json_step = BasicWorkflowStep::new(
        "process_json".to_string(),
        BasicStepType::Tool {
            tool_name: "json_processor".to_string(),
            parameters: serde_json::json!({"validate": true, "format": true}),
        },
    );
    let json_branch =
        ConditionalBranch::new("JSON Processing".to_string(), json_condition).with_step(json_step);

    workflow.add_branch(json_branch).await?;

    // Branch 3: Default branch for unknown data types
    let default_step = BasicWorkflowStep::new(
        "default_processing".to_string(),
        BasicStepType::Tool {
            tool_name: "file_operations".to_string(),
            parameters: serde_json::json!({"operation": "read", "encoding": "utf-8"}),
        },
    );
    let default_branch =
        ConditionalBranch::default("Default Processing".to_string()).with_step(default_step);

    workflow.add_branch(default_branch).await?;

    println!(
        "✅ Added {} branches to conditional workflow",
        workflow.branch_count()
    );

    // Test 1: Process CSV data
    println!("\n🧪 Test 1: Processing CSV data");
    workflow
        .set_shared_data("data_type".to_string(), serde_json::json!("csv"))
        .await?;

    let csv_results = workflow.execute().await?;
    println!(
        "📊 CSV processing completed: {} steps executed",
        csv_results.len()
    );
    for result in &csv_results {
        println!("  ✓ Step '{}': {}", result.step_name, result.output);
    }

    // Reset workflow for next test
    workflow.reset().await?;

    // Test 2: Process JSON data
    println!("\n🧪 Test 2: Processing JSON data");
    workflow
        .set_shared_data("data_type".to_string(), serde_json::json!("json"))
        .await?;

    let json_results = workflow.execute().await?;
    println!(
        "📊 JSON processing completed: {} steps executed",
        json_results.len()
    );
    for result in &json_results {
        println!("  ✓ Step '{}': {}", result.step_name, result.output);
    }

    // Reset workflow for next test
    workflow.reset().await?;

    // Test 3: Unknown data type (should trigger default branch)
    println!("\n🧪 Test 3: Processing unknown data type");
    workflow
        .set_shared_data("data_type".to_string(), serde_json::json!("xml"))
        .await?;

    let default_results = workflow.execute().await?;
    println!(
        "📊 Default processing completed: {} steps executed",
        default_results.len()
    );
    for result in &default_results {
        println!("  ✓ Step '{}': {}", result.step_name, result.output);
    }

    // Test 4: Complex condition with AND logic
    println!("\n🧪 Test 4: Complex condition with AND logic");
    workflow.reset().await?;

    // Add a complex branch with AND condition
    let complex_condition = BasicCondition::and(vec![
        BasicCondition::shared_data_equals("data_type".to_string(), serde_json::json!("csv")),
        BasicCondition::shared_data_exists("priority".to_string()),
    ]);
    let complex_step = BasicWorkflowStep::new(
        "priority_csv_processing".to_string(),
        BasicStepType::Tool {
            tool_name: "csv_analyzer".to_string(),
            parameters: serde_json::json!({"priority": true, "fast_mode": true}),
        },
    );
    let complex_branch =
        ConditionalBranch::new("Priority CSV Processing".to_string(), complex_condition)
            .with_step(complex_step);

    workflow.add_branch(complex_branch).await?;

    // Set conditions to match the complex branch
    workflow
        .set_shared_data("data_type".to_string(), serde_json::json!("csv"))
        .await?;
    workflow
        .set_shared_data("priority".to_string(), serde_json::json!("high"))
        .await?;

    let complex_results = workflow.execute().await?;
    println!(
        "📊 Priority CSV processing completed: {} steps executed",
        complex_results.len()
    );
    for result in &complex_results {
        println!("  ✓ Step '{}': {}", result.step_name, result.output);
    }

    println!("\n🎉 Conditional workflow example completed successfully!");

    Ok(())
}
