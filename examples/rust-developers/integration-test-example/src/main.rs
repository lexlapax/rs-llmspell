//! ============================================================
//! LLMSPELL RUST DEVELOPERS SHOWCASE
//! ============================================================
//! Example ID: 06 - Integration Testing v1.0.0
//! Complexity Level: ADVANCED
//! Real-World Use Case: Comprehensive testing strategies for LLMSpell components
//!
//! Purpose: Demonstrates testing patterns for tools, agents, and integration scenarios
//! Architecture: Test-driven development, mocking patterns, integration testing
//! Crates Showcased: llmspell-core (BaseAgent, Tool traits), testing patterns
//! Key Features:
//!   • Unit testing for individual components
//!   • Integration testing for component interactions
//!   • Mock objects and test fixtures
//!   • Error handling and edge case testing
//!
//! Prerequisites:
//!   • Rust 1.70+, tokio async runtime, understanding of testing patterns
//!
//! HOW TO RUN:
//! ```bash
//! cd examples/rust-developers/getting-started/integration-test-example
//! cargo build
//! cargo run
//! ```
//!
//! TO RUN TESTS:
//! ```bash
//! cargo test
//! cargo test -- --nocapture  # To see println! output
//! ```
//!
//! EXPECTED OUTPUT:
//! Testing demonstrations with unit tests, integration tests, and mocking
//!
//! Time to Complete: <5 seconds compilation + execution
//! ============================================================

use anyhow::Result;
use async_trait::async_trait;
use llmspell_core::{
    traits::tool::{ParameterDef, ParameterType, SecurityLevel, ToolCategory, ToolSchema},
    types::{AgentInput, AgentOutput},
    BaseAgent, ComponentMetadata, ExecutionContext, LLMSpellError, Tool,
};
use serde_json::json;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

// Example tool for testing
#[derive(Debug, Clone)]
pub struct EchoTool {
    metadata: ComponentMetadata,
    call_log: Arc<Mutex<Vec<String>>>, // For testing - track calls
}

impl Default for EchoTool {
    fn default() -> Self {
        Self::new()
    }
}

impl EchoTool {
    pub fn new() -> Self {
        Self {
            metadata: ComponentMetadata::new(
                "echo_tool".to_string(),
                "Echoes back input with additional metadata".to_string(),
            ),
            call_log: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Get the call log for testing purposes
    pub fn get_call_log(&self) -> Vec<String> {
        self.call_log.lock().unwrap().clone()
    }

    /// Clear the call log
    pub fn clear_call_log(&self) {
        self.call_log.lock().unwrap().clear();
    }
}

#[async_trait]
impl BaseAgent for EchoTool {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute_impl(
        &self,
        input: AgentInput,
        _context: ExecutionContext,
    ) -> Result<AgentOutput, LLMSpellError> {
        let text = if !input.text.is_empty() {
            input.text.as_str()
        } else {
            input
                .parameters
                .get("input")
                .and_then(|v| v.as_str())
                .unwrap_or("No input provided")
        };

        // Log the call for testing
        {
            let mut log = self.call_log.lock().unwrap();
            log.push(format!("echo: {}", text));
        }

        info!("EchoTool processing: {}", text);

        let result = json!({
            "echoed": text,
            "length": text.len(),
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "call_count": self.get_call_log().len(),
            "success": true
        });

        Ok(AgentOutput::text(result.to_string()))
    }

    async fn validate_input(&self, _input: &AgentInput) -> Result<(), LLMSpellError> {
        // Accept any input for this demo tool
        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput, LLMSpellError> {
        let error_response = json!({
            "error": error.to_string(),
            "tool": "echo_tool",
            "success": false
        });

        Ok(AgentOutput::text(error_response.to_string()))
    }
}

#[async_trait]
impl Tool for EchoTool {
    fn category(&self) -> ToolCategory {
        ToolCategory::Utility
    }

    fn security_level(&self) -> SecurityLevel {
        SecurityLevel::Safe
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema::new(
            self.metadata.name.clone(),
            self.metadata.description.clone(),
        )
        .with_parameter(ParameterDef {
            name: "input".to_string(),
            param_type: ParameterType::String,
            description: "Text to echo back".to_string(),
            required: false,
            default: Some(json!("default text")),
        })
        .with_returns(ParameterType::Object)
    }
}

// Mock Agent for testing
#[derive(Debug)]
pub struct MockAgent {
    metadata: ComponentMetadata,
    responses: Arc<Mutex<Vec<String>>>, // Predefined responses
    call_count: Arc<Mutex<usize>>,
}

impl MockAgent {
    pub fn new(name: String, responses: Vec<String>) -> Self {
        Self {
            metadata: ComponentMetadata::new(name, "Mock agent for testing".to_string()),
            responses: Arc::new(Mutex::new(responses)),
            call_count: Arc::new(Mutex::new(0)),
        }
    }

    pub fn get_call_count(&self) -> usize {
        *self.call_count.lock().unwrap()
    }

    pub fn add_response(&self, response: String) {
        self.responses.lock().unwrap().push(response);
    }
}

#[async_trait]
impl BaseAgent for MockAgent {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute_impl(
        &self,
        input: AgentInput,
        _context: ExecutionContext,
    ) -> Result<AgentOutput, LLMSpellError> {
        let mut count = self.call_count.lock().unwrap();
        *count += 1;
        let call_index = *count - 1;

        let responses = self.responses.lock().unwrap();
        let response = if call_index < responses.len() {
            responses[call_index].clone()
        } else {
            format!("Mock response {} to: {}", call_index + 1, input.text)
        };

        info!("MockAgent responding: {}", response);

        Ok(AgentOutput::text(response))
    }

    async fn validate_input(&self, _input: &AgentInput) -> Result<(), LLMSpellError> {
        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput, LLMSpellError> {
        Ok(AgentOutput::text(format!("Mock error: {}", error)))
    }
}

// Test fixture for setting up test environments
pub struct TestFixture {
    pub echo_tool: EchoTool,
    pub mock_agent: MockAgent,
}

impl Default for TestFixture {
    fn default() -> Self {
        Self::new()
    }
}

impl TestFixture {
    pub fn new() -> Self {
        Self {
            echo_tool: EchoTool::new(),
            mock_agent: MockAgent::new(
                "test_agent".to_string(),
                vec![
                    "Hello! I'm a mock agent.".to_string(),
                    "This is my second response.".to_string(),
                    "I have many responses.".to_string(),
                ],
            ),
        }
    }

    pub fn reset(&self) {
        self.echo_tool.clear_call_log();
    }
}

// Integration scenarios
pub struct IntegrationScenario {
    pub tools: HashMap<String, Arc<dyn Tool>>,
    pub agents: HashMap<String, Arc<dyn BaseAgent>>,
}

impl Default for IntegrationScenario {
    fn default() -> Self {
        Self::new()
    }
}

impl IntegrationScenario {
    pub fn new() -> Self {
        Self {
            tools: HashMap::new(),
            agents: HashMap::new(),
        }
    }

    pub fn add_tool(&mut self, name: String, tool: Arc<dyn Tool>) {
        self.tools.insert(name, tool);
    }

    pub fn add_agent(&mut self, name: String, agent: Arc<dyn BaseAgent>) {
        self.agents.insert(name, agent);
    }

    pub async fn execute_tool(
        &self,
        tool_name: &str,
        input: AgentInput,
    ) -> Result<AgentOutput, LLMSpellError> {
        let tool = self
            .tools
            .get(tool_name)
            .ok_or_else(|| LLMSpellError::Component {
                message: format!("Tool '{}' not found", tool_name),
                source: None,
            })?;

        let context = ExecutionContext::new();
        tool.execute_impl(input, context).await
    }

    pub async fn execute_agent(
        &self,
        agent_name: &str,
        input: AgentInput,
    ) -> Result<AgentOutput, LLMSpellError> {
        let agent = self
            .agents
            .get(agent_name)
            .ok_or_else(|| LLMSpellError::Component {
                message: format!("Agent '{}' not found", agent_name),
                source: None,
            })?;

        let context = ExecutionContext::new();
        agent.execute_impl(input, context).await
    }
}

// Main demonstration functions
async fn demonstrate_unit_testing() -> Result<()> {
    println!("=== Unit Testing Demonstrations ===");

    // Test individual tool
    println!("1. Testing EchoTool:");
    let echo_tool = EchoTool::new();

    // Test case 1: Valid input via text field
    let input1 = AgentInput::text("Hello, Unit Test!");
    let context = ExecutionContext::new();
    let result1 = echo_tool.execute_impl(input1, context).await?;

    println!("   ✅ Text input test:");
    println!("      Input: Hello, Unit Test!");
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&result1.text) {
        if let Some(echoed) = parsed.get("echoed") {
            println!("      Output: {}", echoed.as_str().unwrap_or(""));
        }
    }

    // Test case 2: Valid input via parameters
    let input2 = AgentInput::text("").with_parameter("input", json!("Parameter input test"));
    let context = ExecutionContext::new();
    let result2 = echo_tool.execute_impl(input2, context).await?;

    println!("   ✅ Parameter input test:");
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&result2.text) {
        if let Some(echoed) = parsed.get("echoed") {
            println!("      Output: {}", echoed.as_str().unwrap_or(""));
        }
    }

    // Test case 3: Empty input
    let input3 = AgentInput::text("");
    let context = ExecutionContext::new();
    let result3 = echo_tool.execute_impl(input3, context).await?;

    println!("   ✅ Empty input test:");
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&result3.text) {
        if let Some(echoed) = parsed.get("echoed") {
            println!("      Output: {}", echoed.as_str().unwrap_or(""));
        }
    }

    // Verify call log
    let calls = echo_tool.get_call_log();
    println!(
        "   ✅ Call log verification: {} calls recorded",
        calls.len()
    );

    Ok(())
}

async fn demonstrate_mock_testing() -> Result<()> {
    println!("\n=== Mock Testing Demonstrations ===");

    println!("2. Testing with MockAgent:");
    let mock_agent = MockAgent::new(
        "test_mock".to_string(),
        vec![
            "First mock response".to_string(),
            "Second mock response".to_string(),
        ],
    );

    // Test multiple calls with different responses
    for i in 1..=3 {
        let input = AgentInput::text(format!("Test message {}", i));
        let context = ExecutionContext::new();
        let response = mock_agent.execute_impl(input, context).await?;

        println!("   Call {}: {}", i, response.text);
    }

    println!(
        "   ✅ Mock agent call count: {}",
        mock_agent.get_call_count()
    );

    // Test adding responses dynamically
    mock_agent.add_response("Dynamically added response".to_string());
    let input = AgentInput::text("Another test");
    let context = ExecutionContext::new();
    let response = mock_agent.execute_impl(input, context).await?;
    println!("   ✅ Dynamic response: {}", response.text);

    Ok(())
}

async fn demonstrate_integration_testing() -> Result<()> {
    println!("\n=== Integration Testing Demonstrations ===");

    println!("3. Testing component integration:");
    let mut scenario = IntegrationScenario::new();

    // Set up components
    let echo_tool = Arc::new(EchoTool::new());
    let mock_agent = Arc::new(MockAgent::new(
        "integration_agent".to_string(),
        vec!["Agent processed your request".to_string()],
    ));

    scenario.add_tool("echo".to_string(), echo_tool.clone());
    scenario.add_agent("processor".to_string(), mock_agent);

    // Test tool execution through scenario
    let tool_input = AgentInput::text("Integration test data");
    let tool_result = scenario.execute_tool("echo", tool_input).await?;
    println!("   ✅ Tool integration result:");
    if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&tool_result.text) {
        if let Some(echoed) = parsed.get("echoed") {
            println!("      Tool echoed: {}", echoed.as_str().unwrap_or(""));
        }
    }

    // Test agent execution through scenario
    let agent_input = AgentInput::text("Process this data");
    let agent_result = scenario.execute_agent("processor", agent_input).await?;
    println!("   ✅ Agent integration result:");
    println!("      Agent response: {}", agent_result.text);

    // Test error handling
    match scenario
        .execute_tool("nonexistent", AgentInput::text("test"))
        .await
    {
        Ok(_) => println!("   ❌ Should have failed"),
        Err(e) => println!("   ✅ Error handling: {}", e),
    }

    Ok(())
}

async fn demonstrate_test_fixtures() -> Result<()> {
    println!("\n=== Test Fixture Demonstrations ===");

    println!("4. Using test fixtures:");
    let fixture = TestFixture::new();

    // Use fixture components
    let input = AgentInput::text("Fixture test");
    let context = ExecutionContext::new();

    let tool_result = fixture
        .echo_tool
        .execute_impl(input.clone(), context.clone())
        .await?;
    let agent_result = fixture.mock_agent.execute_impl(input, context).await?;

    println!(
        "   ✅ Fixture tool result: length = {} chars",
        tool_result.text.len()
    );
    println!("   ✅ Fixture agent result: {}", agent_result.text);

    // Show state tracking
    println!(
        "   ✅ Tool calls: {}",
        fixture.echo_tool.get_call_log().len()
    );
    println!("   ✅ Agent calls: {}", fixture.mock_agent.get_call_count());

    // Reset fixture state
    fixture.reset();
    println!(
        "   ✅ Fixture reset - tool calls: {}",
        fixture.echo_tool.get_call_log().len()
    );

    Ok(())
}

async fn demonstrate_schema_validation() -> Result<()> {
    println!("\n=== Schema Validation Testing ===");

    println!("5. Testing tool schema validation:");
    let echo_tool = EchoTool::new();
    let schema = echo_tool.schema();

    println!("   Tool: {}", schema.name);
    println!("   Description: {}", schema.description);
    println!("   Parameters: {} defined", schema.parameters.len());
    println!("   Required params: {:?}", schema.required_parameters());

    // Test parameter validation
    let valid_input = AgentInput::text("test").with_parameter("input", json!("valid parameter"));

    match echo_tool.validate_input(&valid_input).await {
        Ok(_) => println!("   ✅ Valid input passed validation"),
        Err(e) => println!("   ❌ Valid input failed: {}", e),
    }

    // Test tool categorization
    println!("   ✅ Tool category: {:?}", echo_tool.category());
    println!("   ✅ Security level: {:?}", echo_tool.security_level());

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    println!("=== LLMSpell Integration Testing Demo ===\n");

    // Run all demonstrations
    demonstrate_unit_testing().await?;
    demonstrate_mock_testing().await?;
    demonstrate_integration_testing().await?;
    demonstrate_test_fixtures().await?;
    demonstrate_schema_validation().await?;

    println!("\n✅ Integration Testing Demo Complete!");
    println!("\n💡 Key Testing Patterns Demonstrated:");
    println!("   - Unit testing: Testing individual components in isolation");
    println!("   - Mock objects: Controllable test doubles with predefined behavior");
    println!("   - Integration testing: Testing component interactions");
    println!("   - Test fixtures: Reusable test setup and teardown");
    println!("   - Schema validation: Verifying parameter and schema correctness");
    println!("   - Error handling: Testing failure scenarios and edge cases");

    println!("\n📚 Testing Best Practices:");
    println!("   - Write tests first (TDD) to drive API design");
    println!("   - Use mocks to isolate components under test");
    println!("   - Test both success and failure scenarios");
    println!("   - Verify state changes and side effects");
    println!("   - Use fixtures for complex test setup");
    println!("   - Test schema validation and type safety");

    println!("\n🛠️ Testing Tools Used:");
    println!("   - #[tokio::test] for async test functions");
    println!("   - Mock objects with controlled responses");
    println!("   - Test fixtures with state management");
    println!("   - Integration scenarios for component composition");
    println!("   - Call logging and state verification");

    println!("\n📋 To run the actual unit tests:");
    println!("   cargo test                    # Run all tests");
    println!("   cargo test -- --nocapture     # Show println! output");
    println!("   cargo test test_echo          # Run specific tests");

    Ok(())
}

// Unit tests - these would typically be in separate test files
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_echo_tool_basic_functionality() {
        let tool = EchoTool::new();
        let input = AgentInput::text("test message");
        let context = ExecutionContext::new();

        let result = tool.execute_impl(input, context).await.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result.text).unwrap();

        assert_eq!(parsed["echoed"], "test message");
        assert_eq!(parsed["length"], 12);
        assert_eq!(parsed["success"], true);
        assert!(parsed["timestamp"].is_string());
    }

    #[tokio::test]
    async fn test_echo_tool_parameter_input() {
        let tool = EchoTool::new();
        let input = AgentInput::text("").with_parameter("input", json!("param test"));
        let context = ExecutionContext::new();

        let result = tool.execute_impl(input, context).await.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result.text).unwrap();

        assert_eq!(parsed["echoed"], "param test");
        assert_eq!(parsed["length"], 10);
    }

    #[tokio::test]
    async fn test_echo_tool_empty_input() {
        let tool = EchoTool::new();
        let input = AgentInput::text("");
        let context = ExecutionContext::new();

        let result = tool.execute_impl(input, context).await.unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&result.text).unwrap();

        assert_eq!(parsed["echoed"], "No input provided");
    }

    #[tokio::test]
    async fn test_echo_tool_call_logging() {
        let tool = EchoTool::new();

        // Make multiple calls
        for i in 1..=3 {
            let input = AgentInput::text(format!("call {}", i));
            let context = ExecutionContext::new();
            let _ = tool.execute_impl(input, context).await.unwrap();
        }

        let log = tool.get_call_log();
        assert_eq!(log.len(), 3);
        assert!(log[0].contains("call 1"));
        assert!(log[2].contains("call 3"));
    }

    #[tokio::test]
    async fn test_mock_agent_responses() {
        let responses = vec!["Response 1".to_string(), "Response 2".to_string()];
        let agent = MockAgent::new("test".to_string(), responses);

        // First call
        let input1 = AgentInput::text("test1");
        let context1 = ExecutionContext::new();
        let result1 = agent.execute_impl(input1, context1).await.unwrap();
        assert_eq!(result1.text, "Response 1");

        // Second call
        let input2 = AgentInput::text("test2");
        let context2 = ExecutionContext::new();
        let result2 = agent.execute_impl(input2, context2).await.unwrap();
        assert_eq!(result2.text, "Response 2");

        // Third call (beyond predefined responses)
        let input3 = AgentInput::text("test3");
        let context3 = ExecutionContext::new();
        let result3 = agent.execute_impl(input3, context3).await.unwrap();
        assert!(result3.text.contains("Mock response 3"));

        assert_eq!(agent.get_call_count(), 3);
    }

    #[tokio::test]
    async fn test_integration_scenario() {
        let mut scenario = IntegrationScenario::new();
        let tool = Arc::new(EchoTool::new());
        scenario.add_tool("test_tool".to_string(), tool);

        let input = AgentInput::text("integration test");
        let result = scenario.execute_tool("test_tool", input).await.unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&result.text).unwrap();
        assert_eq!(parsed["echoed"], "integration test");
    }

    #[tokio::test]
    async fn test_integration_scenario_error() {
        let scenario = IntegrationScenario::new();
        let input = AgentInput::text("test");

        let result = scenario.execute_tool("nonexistent", input).await;
        assert!(result.is_err());

        if let Err(LLMSpellError::Component { message, .. }) = result {
            assert!(message.contains("not found"));
        }
    }

    #[tokio::test]
    async fn test_tool_schema() {
        let tool = EchoTool::new();
        let schema = tool.schema();

        assert_eq!(schema.name, "echo_tool");
        assert!(!schema.description.is_empty());
        assert_eq!(schema.parameters.len(), 1);
        assert_eq!(schema.parameters[0].name, "input");
        assert!(!schema.parameters[0].required);
    }

    #[tokio::test]
    async fn test_test_fixture() {
        let fixture = TestFixture::new();

        // Test initial state
        assert_eq!(fixture.echo_tool.get_call_log().len(), 0);
        assert_eq!(fixture.mock_agent.get_call_count(), 0);

        // Use fixture
        let input = AgentInput::text("fixture test");
        let context = ExecutionContext::new();
        let _ = fixture
            .echo_tool
            .execute_impl(input.clone(), context.clone())
            .await
            .unwrap();
        let _ = fixture
            .mock_agent
            .execute_impl(input, context)
            .await
            .unwrap();

        // Verify state changes
        assert_eq!(fixture.echo_tool.get_call_log().len(), 1);
        assert_eq!(fixture.mock_agent.get_call_count(), 1);

        // Test reset
        fixture.reset();
        assert_eq!(fixture.echo_tool.get_call_log().len(), 0);
    }
}
