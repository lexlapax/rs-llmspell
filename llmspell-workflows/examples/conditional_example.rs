//! ABOUTME: Example demonstrating conditional workflow patterns
//! ABOUTME: Shows condition evaluation, branch selection, and default branch handling

use llmspell_core::traits::base_agent::BaseAgent;
use llmspell_core::types::agent_io::AgentInput;
use llmspell_core::execution_context::ExecutionContext;
use llmspell_workflows::{
    Condition, ConditionalBranch, ConditionalWorkflow, ConditionalWorkflowConfig, StepType,
    WorkflowConfig, WorkflowStep,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔀 Conditional Workflow Example");
    
    // Create reusable input and context for examples
    let create_execution_params = || {
        let input = AgentInput {
            prompt: "Execute workflow".to_string(),
            context: Default::default(),
        };
        let context = ExecutionContext::default();
        (input, context)
    };
    println!("===============================\n");

    // Initialize tracing for logging
    tracing_subscriber::fmt::init();

    // Create workflow configuration
    let workflow_config = WorkflowConfig::default();
    let conditional_config = ConditionalWorkflowConfig::default();

    println!("📋 Setting up conditional workflow branches...");

    // Branch 1: Process CSV data
    let csv_condition =
        Condition::shared_data_equals("data_type".to_string(), serde_json::json!("csv"));
    let csv_step = WorkflowStep::new(
        "process_csv".to_string(),
        StepType::Tool {
            tool_name: "csv_analyzer".to_string(),
            parameters: serde_json::json!({"delimiter": ",", "headers": true}),
        },
    );
    let csv_branch =
        ConditionalBranch::new("CSV Processing".to_string(), csv_condition).with_step(csv_step);

    // Branch 2: Process JSON data
    let json_condition =
        Condition::shared_data_equals("data_type".to_string(), serde_json::json!("json"));
    let json_step = WorkflowStep::new(
        "process_json".to_string(),
        StepType::Tool {
            tool_name: "json_processor".to_string(),
            parameters: serde_json::json!({"validate": true, "format": true}),
        },
    );
    let json_branch =
        ConditionalBranch::new("JSON Processing".to_string(), json_condition).with_step(json_step);

    // Branch 3: Default branch for unknown data types
    let default_step = WorkflowStep::new(
        "default_processing".to_string(),
        StepType::Tool {
            tool_name: "file_operations".to_string(),
            parameters: serde_json::json!({"operation": "read", "encoding": "utf-8"}),
        },
    );
    let default_branch =
        ConditionalBranch::default("Default Processing".to_string()).with_step(default_step);

    // Build the conditional workflow
    let workflow = ConditionalWorkflow::builder("data_processor".to_string())
        .with_workflow_config(workflow_config)
        .with_conditional_config(conditional_config)
        .add_branch(csv_branch)
        .add_branch(json_branch)
        .add_branch(default_branch)
        .build();

    println!(
        "✅ Added {} branches to conditional workflow",
        workflow.branch_count()
    );

    // Test 1: Process CSV data
    println!("\n🧪 Test 1: Processing CSV data");
    workflow
        .set_shared_data("data_type".to_string(), serde_json::json!("csv"))
        .await?;

    let (input, context) = create_execution_params();
    let csv_result = workflow.execute(input, context).await?;
    println!("📊 CSV processing completed!");
    println!("{}", csv_result.generate_report());

    for branch_result in &csv_result.executed_branches {
        println!("\n📋 Branch '{}' executed:", branch_result.branch_name);
        for step_result in &branch_result.step_results {
            println!(
                "  ✓ Step '{}': {}",
                step_result.step_name, step_result.output
            );
        }
    }

    // Reset workflow for next test
    workflow.reset().await?;

    // Test 2: Process JSON data
    println!("\n🧪 Test 2: Processing JSON data");
    workflow
        .set_shared_data("data_type".to_string(), serde_json::json!("json"))
        .await?;

    let (input, context) = create_execution_params();
    let json_result = workflow.execute(input, context).await?;
    println!("📊 JSON processing completed!");
    println!("{}", json_result.generate_report());

    for branch_result in &json_result.executed_branches {
        println!("\n📋 Branch '{}' executed:", branch_result.branch_name);
        for step_result in &branch_result.step_results {
            println!(
                "  ✓ Step '{}': {}",
                step_result.step_name, step_result.output
            );
        }
    }

    // Reset workflow for next test
    workflow.reset().await?;

    // Test 3: Unknown data type (should trigger default branch)
    println!("\n🧪 Test 3: Processing unknown data type");
    workflow
        .set_shared_data("data_type".to_string(), serde_json::json!("xml"))
        .await?;

    let (input, context) = create_execution_params();
    let default_result = workflow.execute(input, context).await?;
    println!("📊 Default processing completed!");
    println!("{}", default_result.generate_report());

    for branch_result in &default_result.executed_branches {
        println!("\n📋 Branch '{}' executed:", branch_result.branch_name);
        for step_result in &branch_result.step_results {
            println!(
                "  ✓ Step '{}': {}",
                step_result.step_name, step_result.output
            );
        }
    }

    // Test 4: Complex condition with AND logic
    println!("\n🧪 Test 4: Complex condition with AND logic");
    workflow.reset().await?;

    // Create a new workflow with complex condition
    let complex_condition = Condition::and(vec![
        Condition::shared_data_equals("data_type".to_string(), serde_json::json!("csv")),
        Condition::shared_data_exists("priority".to_string()),
    ]);
    let complex_step = WorkflowStep::new(
        "priority_csv_processing".to_string(),
        StepType::Tool {
            tool_name: "csv_analyzer".to_string(),
            parameters: serde_json::json!({"priority": true, "fast_mode": true}),
        },
    );
    let complex_branch =
        ConditionalBranch::new("Priority CSV Processing".to_string(), complex_condition)
            .with_step(complex_step);

    // Rebuild workflow with the additional complex branch
    let complex_workflow = ConditionalWorkflow::builder("data_processor_complex".to_string())
        .with_workflow_config(WorkflowConfig::default())
        .with_conditional_config(ConditionalWorkflowConfig::default())
        .add_branch(
            ConditionalBranch::new(
                "CSV Processing".to_string(),
                Condition::shared_data_equals("data_type".to_string(), serde_json::json!("csv")),
            )
            .with_step(WorkflowStep::new(
                "process_csv".to_string(),
                StepType::Tool {
                    tool_name: "csv_analyzer".to_string(),
                    parameters: serde_json::json!({"delimiter": ",", "headers": true}),
                },
            )),
        )
        .add_branch(complex_branch)
        .add_branch(
            ConditionalBranch::default("Default Processing".to_string()).with_step(
                WorkflowStep::new(
                    "default_processing".to_string(),
                    StepType::Tool {
                        tool_name: "file_operations".to_string(),
                        parameters: serde_json::json!({"operation": "read", "encoding": "utf-8"}),
                    },
                ),
            ),
        )
        .build();

    // Set conditions to match the complex branch
    complex_workflow
        .set_shared_data("data_type".to_string(), serde_json::json!("csv"))
        .await?;
    complex_workflow
        .set_shared_data("priority".to_string(), serde_json::json!("high"))
        .await?;

    let (input, context) = create_execution_params();
    let complex_result = complex_workflow.execute(input, context).await?;
    println!("📊 Priority CSV processing completed!");
    println!("{}", complex_result.generate_report());

    for branch_result in &complex_result.executed_branches {
        println!("\n📋 Branch '{}' executed:", branch_result.branch_name);
        for step_result in &branch_result.step_results {
            println!(
                "  ✓ Step '{}': {}",
                step_result.step_name, step_result.output
            );
        }
    }

    println!("\n🎉 Conditional workflow example completed successfully!");

    Ok(())
}
