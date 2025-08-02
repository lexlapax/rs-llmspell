//! ABOUTME: Tool orchestrator agent example demonstrating multi-tool coordination
//! ABOUTME: Shows how agents can discover, select, and chain tools for complex tasks

use llmspell_agents::templates::{
    AgentTemplate, OrchestratorAgentTemplate, TemplateInstantiationParams,
};
use llmspell_core::types::AgentInput;
use llmspell_core::ExecutionContext;
use std::time::Instant;
use tracing::{info, Level};

/// Example demonstrating a tool orchestrator agent that coordinates multiple tools
/// to analyze data files and generate reports.
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("Starting Tool Orchestrator Agent Example");

    // For this example, we'll create a mock orchestrator agent
    // In a real implementation, this would create an actual agent with tool management capabilities

    // Create the orchestrator agent template
    let template = OrchestratorAgentTemplate::new();

    // Create instantiation parameters
    let params = TemplateInstantiationParams::new("tool-orchestrator-001".to_string())
        .with_parameter("agent_name", "Tool Orchestrator".into())
        .with_parameter("orchestration_strategy", "sequential".into())
        .with_parameter("max_workflow_time", 3600.into())
        .with_parameter("enable_health_monitoring", true.into());

    // Instantiate the agent
    let result = template.instantiate(params).await?;
    let agent = result.agent;

    info!("Created Tool Orchestrator Agent");

    // Example 1: Simple tool chain - Read file, process JSON, validate data
    println!("\n=== Example 1: Simple Tool Chain ===");

    let input = AgentInput::text(
        "Read the file 'data/config.json', validate its structure, and summarize the contents.",
    );

    let start = Instant::now();
    let output = agent.execute(input, ExecutionContext::default()).await?;
    let duration = start.elapsed();

    println!("Result: {}", output.text);
    println!("Execution time: {duration:?}");

    if !output.tool_calls.is_empty() {
        println!("\nTools used:");
        for (i, call) in output.tool_calls.iter().enumerate() {
            println!("  {}. {} - {:?}", i + 1, call.tool_name, call.parameters);
        }
    }

    // Example 2: Complex analysis pipeline
    println!("\n=== Example 2: Complex Analysis Pipeline ===");

    let input = AgentInput::text(
        "Analyze the CSV file 'sales_data.csv': \
         1. Read and parse the CSV \
         2. Calculate basic statistics \
         3. Identify any data quality issues \
         4. Generate a summary report \
         5. Save the report as 'sales_analysis.json'",
    );

    let start = Instant::now();
    let output = agent.execute(input, ExecutionContext::default()).await?;
    let duration = start.elapsed();

    println!("Result: {}", output.text);
    println!("Execution time: {duration:?}");

    if !output.tool_calls.is_empty() {
        println!("\nTool execution sequence:");
        for (i, call) in output.tool_calls.iter().enumerate() {
            println!("  Step {}: {}", i + 1, call.tool_name);
            if let Some(input) = call.parameters.get("input") {
                if let Some(s) = input.as_str() {
                    println!("    Input: {}", &s[..s.len().min(50)]);
                }
            }
        }
    }

    // Example 3: Error handling and recovery
    println!("\n=== Example 3: Error Handling and Recovery ===");

    let input = AgentInput::text(
        "Try to read 'nonexistent.txt'. If it fails, create a new file with default content \
         and then process it.",
    );

    let output = agent.execute(input, ExecutionContext::default()).await?;
    println!("Result: {}", output.text);

    // Example 4: Parallel tool execution
    println!("\n=== Example 4: Parallel Tool Execution ===");

    let input = AgentInput::text(
        "Simultaneously: \
         1. Calculate hash of 'file1.txt' \
         2. Get system information \
         3. Check current time \
         Then combine all results into a status report.",
    );

    let start = Instant::now();
    let output = agent.execute(input, ExecutionContext::default()).await?;
    let duration = start.elapsed();

    println!("Result: {}", output.text);
    println!("Execution time: {duration:?}");
    println!("Note: Parallel execution should be faster than sequential");

    // Example 5: Tool selection based on context
    println!("\n=== Example 5: Intelligent Tool Selection ===");

    let input = AgentInput::text(
        "I have some data that might be in different formats. \
         Figure out what format the file 'mystery_data' is in and process it appropriately.",
    );

    let output = agent.execute(input, ExecutionContext::default()).await?;
    println!("Result: {}", output.text);

    // Show agent's decision-making process
    if let Some(confidence) = output.metadata.confidence {
        println!("Agent confidence: {:.2}%", confidence * 100.0);
    }

    // Performance summary
    println!("\n=== Performance Summary ===");
    println!("The tool orchestrator agent demonstrates:");
    println!("- Automatic tool discovery and selection");
    println!("- Efficient tool chaining and pipelining");
    println!("- Error handling and recovery strategies");
    println!("- Parallel execution capabilities");
    println!("- Context-aware tool selection");

    // Advanced usage tips
    println!("\n=== Advanced Usage Tips ===");
    println!("1. Use specific tool names when you know what you need");
    println!("2. Let the agent discover tools for general tasks");
    println!("3. Provide clear success criteria for complex pipelines");
    println!("4. Use the agent's metadata to understand its decisions");
    println!("5. Monitor tool_calls to see the execution flow");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_agents::templates::AgentTemplate;
    use llmspell_testing::fixtures::create_test_context;

    #[tokio::test]
    async fn test_tool_orchestrator_basic() {
        let template = OrchestratorAgentTemplate::new();
        let params = TemplateInstantiationParams::new("test-orchestrator".to_string())
            .with_parameter("agent_name", "Test Orchestrator".into())
            .with_parameter("orchestration_strategy", "sequential".into());

        let result = template.instantiate(params).await.unwrap();
        let agent = result.agent;

        let input = AgentInput::text("List available tools");
        let output = agent.execute(input, create_test_context()).await.unwrap();

        assert!(!output.text.is_empty());
    }

    #[tokio::test]
    async fn test_tool_chain_execution() {
        let template = OrchestratorAgentTemplate::new();
        let params = TemplateInstantiationParams::new("test-orchestrator".to_string())
            .with_parameter("agent_name", "Test Orchestrator".into())
            .with_parameter("orchestration_strategy", "sequential".into());

        let result = template.instantiate(params).await.unwrap();
        let agent = result.agent;

        let input = AgentInput::text("Generate a UUID and then encode it in base64");
        let output = agent.execute(input, create_test_context()).await.unwrap();

        // Mock agent returns fixed output for now
        assert!(!output.text.is_empty());
    }

    #[tokio::test]
    async fn test_error_recovery() {
        let template = OrchestratorAgentTemplate::new();
        let params = TemplateInstantiationParams::new("test-orchestrator".to_string())
            .with_parameter("agent_name", "Test Orchestrator".into())
            .with_parameter("orchestration_strategy", "sequential".into())
            .with_parameter("enable_rollback", true.into());

        let result = template.instantiate(params).await.unwrap();
        let agent = result.agent;

        let input =
            AgentInput::text("Try to read a non-existent file and handle the error gracefully");
        let output = agent.execute(input, create_test_context()).await.unwrap();

        // Should complete successfully despite the error
        assert!(!output.text.is_empty());
    }
}
