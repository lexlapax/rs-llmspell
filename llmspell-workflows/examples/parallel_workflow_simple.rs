//! ABOUTME: Simple example demonstrating parallel workflows as agents
//! ABOUTME: Shows how to use workflows through the BaseAgent interface

use llmspell_core::execution_context::ExecutionContext;
use llmspell_core::traits::base_agent::BaseAgent;
use llmspell_core::types::agent_io::AgentInput;
use llmspell_workflows::{
    traits::{StepType, WorkflowStep},
    ParallelBranch, ParallelWorkflowBuilder,
};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("Parallel Workflow as Agent Example\n");

    // Create a simple parallel workflow with two branches
    let data_branch = ParallelBranch::new("fetch_data".to_string())
        .with_description("Fetch data from API".to_string())
        .add_step(WorkflowStep::new(
            "api_call".to_string(),
            StepType::Tool {
                tool_name: "http_request".to_string(),
                parameters: json!({"url": "https://api.example.com/data"}),
            },
        ));

    let process_branch = ParallelBranch::new("process_data".to_string())
        .with_description("Process local data".to_string())
        .add_step(WorkflowStep::new(
            "transform".to_string(),
            StepType::Tool {
                tool_name: "data_processor".to_string(),
                parameters: json!({"operation": "transform"}),
            },
        ));

    // Build the workflow
    let workflow = ParallelWorkflowBuilder::new("data_workflow")
        .add_branch(data_branch)
        .add_branch(process_branch)
        .build()?;

    // Execute the workflow as an agent
    let input = AgentInput::text("Process customer data for report");
    let context = ExecutionContext::default();

    println!("Executing workflow with input: {}", input.text);
    let result = workflow.execute(input, context).await?;

    // The result is an AgentOutput with a text summary
    println!("\nWorkflow Result:");
    println!("{}", result.text);

    // If you need execution details, they're in metadata
    if let Some(exec_time) = result.metadata.execution_time_ms {
        println!("\nExecution time: {}ms", exec_time);
    }

    // For advanced use: access workflow-specific metadata
    if let Some(success) = result.metadata.extra.get("workflow_success") {
        println!("Workflow success: {}", success);
    }

    Ok(())
}
