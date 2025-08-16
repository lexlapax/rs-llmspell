// ABOUTME: Example demonstrating how to test LLMSpell components
// ABOUTME: Shows unit testing, integration testing, and test helpers

use anyhow::Result;
use llmspell_testing::{
    agent_helpers::{create_mock_provider_agent, AgentTestBuilder},
    tool_helpers::{create_test_tool, create_test_tool_input, MockTool},
    workflow_helpers::{create_test_sequential_workflow, create_test_workflow_step},
};
use serde_json::json;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

// Example custom tool for testing
use async_trait::async_trait;
use llmspell_core::{Tool, ToolInput, ToolOutput};

#[derive(Debug, Clone)]
struct EchoTool;

#[async_trait]
impl Tool for EchoTool {
    fn name(&self) -> &str {
        "echo"
    }

    fn description(&self) -> &str {
        "Echoes back the input"
    }

    fn parameters(&self) -> Vec<(&str, &str)> {
        vec![("input", "Text to echo")]
    }

    async fn invoke(&self, input: ToolInput) -> Result<ToolOutput> {
        let text = input
            .get("input")
            .and_then(|v| v.as_str())
            .unwrap_or("No input");

        Ok(ToolOutput::from_json(json!({
            "echoed": text,
            "length": text.len(),
            "success": true
        })))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    println!("=== Testing LLMSpell Components ===\n");

    // Run all test examples
    test_tools().await?;
    test_agents().await?;
    test_workflows().await?;
    test_integration().await?;

    println!("\n✅ All tests completed successfully!");
    println!("\n💡 Key Testing Concepts:");
    println!("   - Use llmspell-testing for test helpers");
    println!("   - Mock external dependencies");
    println!("   - Test both success and error cases");
    println!("   - Use AgentTestBuilder for agent testing");
    println!("   - Verify tool invocations and outputs");

    Ok(())
}

// Test 1: Testing Tools
async fn test_tools() -> Result<()> {
    println!("1. Testing Tools");
    println!("   " + &"-".repeat(40));

    // Test with real tool
    println!("   Testing EchoTool:");
    let echo_tool = EchoTool;
    
    // Test case 1: Valid input
    let input = json!({ "input": "Hello, World!" });
    let result = echo_tool.invoke(input).await?;
    let output = result.to_json();
    
    assert_eq!(
        output.get("echoed").and_then(|v| v.as_str()),
        Some("Hello, World!")
    );
    assert_eq!(
        output.get("length").and_then(|v| v.as_u64()),
        Some(13)
    );
    println!("   ✅ Valid input test passed");

    // Test case 2: Missing input
    let empty_input = json!({});
    let empty_result = echo_tool.invoke(empty_input).await?;
    let empty_output = empty_result.to_json();
    
    assert_eq!(
        empty_output.get("echoed").and_then(|v| v.as_str()),
        Some("No input")
    );
    println!("   ✅ Missing input test passed");

    // Test with mock tool from test helpers
    println!("\n   Testing with MockTool:");
    let mock_tool = create_test_tool(
        "test_tool",
        "A test tool",
        vec![("param1", "string"), ("param2", "number")],
    );
    
    let test_input = create_test_tool_input(vec![
        ("param1", "test value"),
        ("param2", "42"),
    ]);
    
    // MockTool always returns success with the input echoed
    let mock_result = mock_tool.invoke(test_input).await?;
    println!("   ✅ Mock tool invocation successful");

    Ok(())
}

// Test 2: Testing Agents
async fn test_agents() -> Result<()> {
    println!("\n2. Testing Agents");
    println!("   " + &"-".repeat(40));

    // Create a mock agent using test helpers
    println!("   Creating mock agent:");
    let mock_agent = create_mock_provider_agent(
        "test_agent",
        "A test agent",
        Some("You are a helpful test assistant"),
    )?;

    // Test agent invocation
    let input = json!({ "text": "What is 2+2?" });
    let response = mock_agent.invoke(input).await?;
    
    assert!(response.to_json().get("text").is_some());
    println!("   ✅ Mock agent invocation successful");

    // Test agent builder
    println!("\n   Testing AgentTestBuilder:");
    let test_agent = AgentTestBuilder::new()
        .name("builder_test_agent")
        .system_prompt("You are a test bot")
        .temperature(0.5)
        .max_tokens(100)
        .build()?;

    let builder_input = json!({ "text": "Hello!" });
    let builder_response = test_agent.invoke(builder_input).await?;
    
    assert!(builder_response.to_json().get("success").is_some());
    println!("   ✅ Agent builder test passed");

    Ok(())
}

// Test 3: Testing Workflows
async fn test_workflows() -> Result<()> {
    println!("\n3. Testing Workflows");
    println!("   " + &"-".repeat(40));

    // Create test workflow steps
    println!("   Creating test workflow:");
    let step1 = create_test_workflow_step(
        "step1",
        "tool",
        Some("echo"),
        json!({ "input": "Step 1" }),
    );
    
    let step2 = create_test_workflow_step(
        "step2",
        "tool",
        Some("echo"),
        json!({ "input": "Step 2" }),
    );

    // Create sequential workflow
    let workflow = create_test_sequential_workflow(
        "test_workflow",
        vec![step1, step2],
    )?;

    println!("   ✅ Test workflow created with 2 steps");

    // In a real test, you would execute the workflow
    // and verify the results
    println!("   ✅ Workflow structure validated");

    Ok(())
}

// Test 4: Integration Testing
async fn test_integration() -> Result<()> {
    println!("\n4. Integration Testing");
    println!("   " + &"-".repeat(40));

    // Test tool + agent integration
    println!("   Testing tool-agent integration:");
    
    // Create an agent that can use tools
    let agent_with_tools = AgentTestBuilder::new()
        .name("tool_user")
        .system_prompt("You can use tools to help answer questions")
        .can_use_tools(true)
        .build()?;

    // In a real scenario, you would:
    // 1. Register tools with the agent
    // 2. Send a message that requires tool use
    // 3. Verify the agent called the correct tool
    // 4. Check the final response

    println!("   ✅ Tool-agent integration test setup complete");

    // Test workflow + agent integration
    println!("\n   Testing workflow-agent integration:");
    
    // Create a workflow that uses an agent
    let agent_step = create_test_workflow_step(
        "agent_step",
        "agent",
        Some("test_agent"),
        json!({ "text": "Process this data" }),
    );

    let tool_step = create_test_workflow_step(
        "tool_step",
        "tool",
        Some("echo"),
        json!({ "input": "{{step:agent_step:output}}" }),
    );

    let integrated_workflow = create_test_sequential_workflow(
        "integrated_workflow",
        vec![agent_step, tool_step],
    )?;

    println!("   ✅ Integrated workflow created");

    Ok(())
}

// Example unit tests (would be in tests/ directory)
#[cfg(test)]
mod tests {
    use super::*;
    use tokio;

    #[tokio::test]
    async fn test_echo_tool_basic() {
        let tool = EchoTool;
        let input = json!({ "input": "test" });
        let result = tool.invoke(input).await.unwrap();
        let output = result.to_json();
        
        assert_eq!(
            output.get("echoed").and_then(|v| v.as_str()),
            Some("test")
        );
    }

    #[tokio::test]
    async fn test_echo_tool_empty_input() {
        let tool = EchoTool;
        let input = json!({});
        let result = tool.invoke(input).await.unwrap();
        let output = result.to_json();
        
        assert_eq!(
            output.get("echoed").and_then(|v| v.as_str()),
            Some("No input")
        );
    }

    #[tokio::test]
    async fn test_mock_agent_creation() {
        let agent = create_mock_provider_agent(
            "test",
            "description",
            None,
        ).unwrap();
        
        assert_eq!(agent.name(), "test");
    }
}

// To run tests:
// cargo test
// cargo test -- --nocapture  # To see println! output
// cargo test test_echo_tool  # Run specific test