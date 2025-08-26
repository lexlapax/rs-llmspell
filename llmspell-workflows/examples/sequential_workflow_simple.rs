//! ABOUTME: Simple example demonstrating sequential workflows as agents
//! ABOUTME: Shows how to use workflows through the BaseAgent interface

use llmspell_core::execution_context::ExecutionContext;
use llmspell_core::traits::base_agent::BaseAgent;
use llmspell_core::types::AgentInput;
use llmspell_workflows::{
    traits::{StepType, WorkflowStep},
    SequentialWorkflowBuilder,
};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("Sequential Workflow as Agent Example\n");

    // Create a simple sequential workflow
    let workflow = SequentialWorkflowBuilder::new("data_pipeline".to_string())
        .add_step(WorkflowStep::new(
            "load_data".to_string(),
            StepType::Tool {
                tool_name: "file_reader".to_string(),
                parameters: json!({"path": "data.json"}),
            },
        ))
        .add_step(WorkflowStep::new(
            "validate_data".to_string(),
            StepType::Tool {
                tool_name: "data_validator".to_string(),
                parameters: json!({"schema": "customer_schema"}),
            },
        ))
        .add_step(WorkflowStep::new(
            "transform_data".to_string(),
            StepType::Tool {
                tool_name: "data_transformer".to_string(),
                parameters: json!({"format": "csv"}),
            },
        ))
        .build();

    // Execute the workflow as an agent
    let input = AgentInput::text("Process customer data file");
    let context = ExecutionContext::default();

    println!("Executing workflow with input: {}", input.text);
    let result = workflow.execute(input, context).await?;

    // The result is an AgentOutput with a text summary
    println!("\nWorkflow Result:");
    println!("{}", result.text);

    // Execution time is available in metadata
    if let Some(exec_time) = result.metadata.execution_time_ms {
        println!("\nExecution time: {}ms", exec_time);
    }

    Ok(())
}
