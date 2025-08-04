//! ABOUTME: Simple example demonstrating loop workflows as agents
//! ABOUTME: Shows how to use workflows through the BaseAgent interface

use llmspell_core::execution_context::ExecutionContext;
use llmspell_core::traits::base_agent::BaseAgent;
use llmspell_core::types::agent_io::AgentInput;
use llmspell_workflows::{
    r#loop::LoopWorkflowBuilder,
    traits::{StepType, WorkflowStep},
};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("Loop Workflow as Agent Example\n");

    // Create a simple loop workflow that processes items
    let workflow = LoopWorkflowBuilder::new("item_processor")
        .description("Process items in a loop")
        .with_range(0, 5, 1) // Loop from 0 to 5 with step 1
        .add_step(WorkflowStep::new(
            "process_item".to_string(),
            StepType::Tool {
                tool_name: "item_processor".to_string(),
                parameters: json!({"operation": "transform"}),
            },
        ))
        .continue_on_error(false)
        .build()?;

    // Execute the workflow as an agent
    let input = AgentInput::text("Process all pending items");
    let context = ExecutionContext::default();

    println!("Executing workflow with input: {}", input.text);
    let result = workflow.execute(input, context).await?;

    // The result is an AgentOutput with a text summary
    println!("\nWorkflow Result:");
    println!("{}", result.text);

    Ok(())
}
