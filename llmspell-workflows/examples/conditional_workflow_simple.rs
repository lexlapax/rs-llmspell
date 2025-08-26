//! ABOUTME: Simple example demonstrating conditional workflows as agents
//! ABOUTME: Shows how to use workflows through the BaseAgent interface

use llmspell_core::execution_context::ExecutionContext;
use llmspell_core::traits::base_agent::BaseAgent;
use llmspell_core::types::agent_io::AgentInput;
use llmspell_workflows::{
    conditional::{ConditionalBranch, ConditionalWorkflowBuilder},
    conditions::Condition,
    traits::{StepType, WorkflowStep},
};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("Conditional Workflow as Agent Example\n");

    // Create branches for different conditions
    let csv_branch = ConditionalBranch::new(
        "csv_processor".to_string(),
        Condition::SharedDataEquals {
            key: "file_type".to_string(),
            expected_value: json!("csv"),
        },
    )
    .with_step(WorkflowStep::new(
        "parse_csv".to_string(),
        StepType::Tool {
            tool_name: "csv_parser".to_string(),
            parameters: json!({"delimiter": ","}),
        },
    ));

    let json_branch = ConditionalBranch::new(
        "json_processor".to_string(),
        Condition::SharedDataEquals {
            key: "file_type".to_string(),
            expected_value: json!("json"),
        },
    )
    .with_step(WorkflowStep::new(
        "parse_json".to_string(),
        StepType::Tool {
            tool_name: "json_parser".to_string(),
            parameters: json!({"strict": true}),
        },
    ));

    let default_branch =
        ConditionalBranch::default("default_processor".to_string()).with_step(WorkflowStep::new(
            "parse_text".to_string(),
            StepType::Tool {
                tool_name: "text_parser".to_string(),
                parameters: json!({}),
            },
        ));

    // Build the conditional workflow
    let workflow = ConditionalWorkflowBuilder::new("file_processor".to_string())
        .add_branch(csv_branch)
        .add_branch(json_branch)
        .add_branch(default_branch)
        .build();

    // Execute the workflow as an agent
    let input = AgentInput::text("Process uploaded file");
    let context = ExecutionContext::default();

    println!("Executing workflow with input: {}", input.text);
    let result = workflow.execute(input, context).await?;

    // The result is an AgentOutput with a text summary
    println!("\nWorkflow Result:");
    println!("{}", result.text);

    Ok(())
}
