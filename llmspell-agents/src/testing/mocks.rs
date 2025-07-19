//! ABOUTME: Mock implementations for testing agent infrastructure
//! ABOUTME: Provides configurable mock agents, tools, and providers for unit and integration testing

use crate::{
    lifecycle::{
        events::{LifecycleEvent, LifecycleEventData, LifecycleEventType},
        state_machine::{AgentState, AgentStateMachine},
    },
    AgentConfig, ResourceLimits,
};
use anyhow::Result;
use async_trait::async_trait;
use llmspell_core::{
    traits::{
        agent::{Agent, AgentConfig as CoreAgentConfig, ConversationMessage},
        tool::{ParameterDef, ParameterType, SecurityLevel, Tool, ToolCategory, ToolSchema},
        tool_capable::ToolCapable,
    },
    types::{
        AgentInput, AgentOutput, ComponentId, ComponentMetadata, OutputMetadata, ToolCall,
        ToolOutput, Version,
    },
    BaseAgent, ExecutionContext, LLMSpellError,
};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::sync::broadcast;

/// Configuration for mock agent behavior
#[derive(Debug, Clone)]
pub struct MockAgentConfig {
    /// Base agent configuration
    pub agent_config: AgentConfig,
    /// Pre-programmed responses
    pub responses: Vec<MockResponse>,
    /// Simulated processing delay
    pub delay: Option<Duration>,
    /// Whether to simulate failures
    pub should_fail: bool,
    /// Failure message if should_fail is true
    pub failure_message: String,
    /// Tool calls to simulate
    pub tool_calls: Vec<ToolCall>,
    /// State transitions to simulate
    pub state_transitions: Vec<AgentState>,
    /// Events to emit
    pub events_to_emit: Vec<LifecycleEvent>,
}

impl Default for MockAgentConfig {
    fn default() -> Self {
        Self {
            agent_config: AgentConfig {
                name: "mock_agent".to_string(),
                description: "Mock agent for testing".to_string(),
                agent_type: "mock".to_string(),
                model: None,
                allowed_tools: vec![],
                custom_config: serde_json::Map::new(),
                resource_limits: ResourceLimits::default(),
            },
            responses: vec![],
            delay: None,
            should_fail: false,
            failure_message: "Mock failure".to_string(),
            tool_calls: vec![],
            state_transitions: vec![],
            events_to_emit: vec![],
        }
    }
}

/// Pre-programmed response for mock agent
#[derive(Debug, Clone)]
pub struct MockResponse {
    /// Input pattern to match (substring match)
    pub input_pattern: Option<String>,
    /// Response text
    pub text: String,
    /// Tool calls to include
    pub tool_calls: Vec<ToolCall>,
    /// Metadata to include
    pub metadata: OutputMetadata,
}

/// Mock agent implementation for testing
pub struct MockAgent {
    metadata: ComponentMetadata,
    config: Arc<Mutex<MockAgentConfig>>,
    core_config: CoreAgentConfig,
    execution_count: Arc<Mutex<usize>>,
    last_input: Arc<Mutex<Option<AgentInput>>>,
    last_context: Arc<Mutex<Option<ExecutionContext>>>,
    state_machine: Arc<AgentStateMachine>,
    event_sender: Option<broadcast::Sender<LifecycleEvent>>,
    conversation: Arc<Mutex<Vec<ConversationMessage>>>,
}

impl MockAgent {
    /// Create new mock agent
    pub fn new(config: MockAgentConfig) -> Self {
        let metadata = ComponentMetadata {
            id: ComponentId::from_name(&config.agent_config.name),
            name: config.agent_config.name.clone(),
            version: Version::new(1, 0, 0),
            description: config.agent_config.description.clone(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let state_machine =
            AgentStateMachine::new(config.agent_config.name.clone(), Default::default());

        let core_config = CoreAgentConfig {
            max_input_length: Some(10000),
            max_output_length: Some(10000),
            max_conversation_length: Some(100),
            enable_streaming: false,
            custom_config: config.agent_config.custom_config.clone(),
        };

        Self {
            metadata,
            core_config,
            config: Arc::new(Mutex::new(config)),
            execution_count: Arc::new(Mutex::new(0)),
            last_input: Arc::new(Mutex::new(None)),
            last_context: Arc::new(Mutex::new(None)),
            state_machine: Arc::new(state_machine),
            event_sender: None,
            conversation: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Set event sender for lifecycle events
    pub fn set_event_sender(&mut self, sender: broadcast::Sender<LifecycleEvent>) {
        self.event_sender = Some(sender);
    }

    /// Get execution count
    pub fn execution_count(&self) -> usize {
        *self.execution_count.lock().unwrap()
    }

    /// Get last input
    pub fn last_input(&self) -> Option<AgentInput> {
        self.last_input.lock().unwrap().clone()
    }

    /// Get last context
    pub fn last_context(&self) -> Option<ExecutionContext> {
        self.last_context.lock().unwrap().clone()
    }

    /// Add response to mock agent
    pub fn add_response(&self, response: MockResponse) {
        self.config.lock().unwrap().responses.push(response);
    }

    /// Set failure mode
    pub fn set_failure(&self, should_fail: bool, message: &str) {
        let mut config = self.config.lock().unwrap();
        config.should_fail = should_fail;
        config.failure_message = message.to_string();
    }

    /// Add state transition
    pub fn add_state_transition(&self, state: AgentState) {
        self.config.lock().unwrap().state_transitions.push(state);
    }

    /// Emit event
    fn emit_event(&self, event: LifecycleEvent) {
        if let Some(sender) = &self.event_sender {
            let _ = sender.send(event);
        }
    }
}

#[async_trait]
impl BaseAgent for MockAgent {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute(
        &self,
        input: AgentInput,
        context: ExecutionContext,
    ) -> Result<AgentOutput, LLMSpellError> {
        // Update execution tracking
        *self.execution_count.lock().unwrap() += 1;
        *self.last_input.lock().unwrap() = Some(input.clone());
        *self.last_context.lock().unwrap() = Some(context.clone());

        let config = self.config.lock().unwrap().clone();

        // Emit configured events
        for event in &config.events_to_emit {
            self.emit_event(event.clone());
        }

        // Simulate state transitions
        for state in &config.state_transitions {
            let from_state = self.state_machine.current_state().await;

            // Transition to new state
            match state {
                AgentState::Ready => {
                    let _ = self.state_machine.initialize().await;
                }
                AgentState::Running => {
                    let _ = self.state_machine.start().await;
                }
                AgentState::Paused => {
                    let _ = self.state_machine.pause().await;
                }
                AgentState::Error => {
                    let _ = self.state_machine.error("Test error".to_string()).await;
                }
                AgentState::Terminated => {
                    let _ = self.state_machine.terminate().await;
                }
                _ => {}
            }

            self.emit_event(LifecycleEvent::new(
                LifecycleEventType::StateChanged,
                self.metadata.id.to_string(),
                LifecycleEventData::StateTransition {
                    from: from_state,
                    to: state.clone(),
                    duration: None,
                    reason: None,
                },
                "mock_agent".to_string(),
            ));
        }

        // Simulate delay if configured
        if let Some(delay) = config.delay {
            tokio::time::sleep(delay).await;
        }

        // Check if should fail
        if config.should_fail {
            return Err(LLMSpellError::Execution {
                message: config.failure_message.clone(),
                source: None,
            });
        }

        // Find matching response
        let response = config
            .responses
            .iter()
            .find(|r| {
                if let Some(pattern) = &r.input_pattern {
                    input.text.contains(pattern)
                } else {
                    true // Match any input if no pattern specified
                }
            })
            .cloned();

        // Build output
        let output = if let Some(response) = response {
            AgentOutput {
                text: response.text,
                media: vec![],
                tool_calls: response.tool_calls,
                metadata: response.metadata,
                transfer_to: None,
            }
        } else {
            // Default response
            AgentOutput {
                text: format!("Mock response to: {}", input.text),
                media: vec![],
                tool_calls: config.tool_calls.clone(),
                metadata: OutputMetadata::default(),
                transfer_to: None,
            }
        };

        Ok(output)
    }

    async fn validate_input(&self, _input: &AgentInput) -> Result<(), LLMSpellError> {
        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput, LLMSpellError> {
        Ok(AgentOutput {
            text: format!("Mock error handling: {}", error),
            media: vec![],
            tool_calls: vec![],
            metadata: OutputMetadata::default(),
            transfer_to: None,
        })
    }
}

#[async_trait]
impl Agent for MockAgent {
    fn config(&self) -> &CoreAgentConfig {
        &self.core_config
    }

    async fn get_conversation(&self) -> Result<Vec<ConversationMessage>, LLMSpellError> {
        Ok(self.conversation.lock().unwrap().clone())
    }

    async fn add_message(&mut self, message: ConversationMessage) -> Result<(), LLMSpellError> {
        self.conversation.lock().unwrap().push(message);
        Ok(())
    }

    async fn clear_conversation(&mut self) -> Result<(), LLMSpellError> {
        self.conversation.lock().unwrap().clear();
        Ok(())
    }
}

#[async_trait]
impl ToolCapable for MockAgent {
    async fn list_available_tools(&self) -> Result<Vec<String>, LLMSpellError> {
        Ok(self
            .config
            .lock()
            .unwrap()
            .agent_config
            .allowed_tools
            .clone())
    }

    async fn tool_available(&self, tool_name: &str) -> bool {
        let config = self.config.lock().unwrap();
        config
            .agent_config
            .allowed_tools
            .contains(&tool_name.to_string())
    }
}

/// Mock tool implementation for testing
pub struct MockTool {
    metadata: ComponentMetadata,
    responses: Arc<Mutex<HashMap<String, ToolOutput>>>,
    execution_count: Arc<Mutex<usize>>,
    delay: Option<Duration>,
    should_fail: bool,
}

impl MockTool {
    /// Create new mock tool
    pub fn new(name: &str) -> Self {
        let metadata = ComponentMetadata {
            id: ComponentId::from_name(name),
            name: name.to_string(),
            version: Version::new(1, 0, 0),
            description: format!("Mock tool: {}", name),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        Self {
            metadata,
            responses: Arc::new(Mutex::new(HashMap::new())),
            execution_count: Arc::new(Mutex::new(0)),
            delay: None,
            should_fail: false,
        }
    }

    /// Set response for specific input
    pub fn set_response(&self, input_key: &str, output: ToolOutput) {
        self.responses
            .lock()
            .unwrap()
            .insert(input_key.to_string(), output);
    }

    /// Set execution delay
    pub fn set_delay(&mut self, delay: Duration) {
        self.delay = Some(delay);
    }

    /// Set failure mode
    pub fn set_failure(&mut self, should_fail: bool) {
        self.should_fail = should_fail;
    }

    /// Get execution count
    pub fn execution_count(&self) -> usize {
        *self.execution_count.lock().unwrap()
    }
}

#[async_trait]
impl BaseAgent for MockTool {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute(
        &self,
        input: AgentInput,
        _context: ExecutionContext,
    ) -> Result<AgentOutput, LLMSpellError> {
        *self.execution_count.lock().unwrap() += 1;

        // Simulate delay
        if let Some(delay) = self.delay {
            tokio::time::sleep(delay).await;
        }

        // Check failure mode
        if self.should_fail {
            return Err(LLMSpellError::Tool {
                tool_name: self.metadata.name.clone(),
                message: "Mock tool failure".to_string(),
                source: None,
            });
        }

        // Extract parameters from input
        let params = &input.parameters;

        // Check for pre-programmed response
        let input_str = serde_json::to_string(params).unwrap_or_default();
        let responses = self.responses.lock().unwrap();

        let tool_output = if let Some(response) = responses.get(&input_str) {
            response.clone()
        } else {
            // Default response
            ToolOutput {
                success: true,
                data: serde_json::json!({
                    "mock": true,
                    "tool": self.metadata.name,
                    "input": params,
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                }),
                error: None,
                execution_time_ms: Some(10),
            }
        };

        // Convert tool output to agent output
        Ok(AgentOutput {
            text: serde_json::to_string_pretty(&tool_output.data).unwrap_or_default(),
            media: vec![],
            tool_calls: vec![],
            metadata: OutputMetadata::default(),
            transfer_to: None,
        })
    }

    async fn validate_input(&self, _input: &AgentInput) -> Result<(), LLMSpellError> {
        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput, LLMSpellError> {
        Ok(AgentOutput::text(format!("Tool error: {}", error)))
    }
}

#[async_trait]
impl Tool for MockTool {
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
            name: "mock_input".to_string(),
            param_type: ParameterType::String,
            description: "Mock input parameter".to_string(),
            required: false,
            default: None,
        })
    }
}

/// Builder for creating mock agents with specific behaviors
pub struct MockAgentBuilder {
    config: MockAgentConfig,
}

impl MockAgentBuilder {
    /// Create new mock agent builder
    pub fn new(name: &str) -> Self {
        let mut config = MockAgentConfig::default();
        config.agent_config.name = name.to_string();

        Self { config }
    }

    /// Set agent type
    pub fn agent_type(mut self, agent_type: &str) -> Self {
        self.config.agent_config.agent_type = agent_type.to_string();
        self
    }

    /// Add allowed tool
    pub fn with_tool(mut self, tool_name: &str) -> Self {
        self.config
            .agent_config
            .allowed_tools
            .push(tool_name.to_string());
        self
    }

    /// Add response
    pub fn with_response(mut self, pattern: Option<String>, text: &str) -> Self {
        self.config.responses.push(MockResponse {
            input_pattern: pattern,
            text: text.to_string(),
            tool_calls: vec![],
            metadata: OutputMetadata::default(),
        });
        self
    }

    /// Add response with tool calls
    pub fn with_tool_response(
        mut self,
        pattern: Option<String>,
        text: &str,
        tool_calls: Vec<ToolCall>,
    ) -> Self {
        self.config.responses.push(MockResponse {
            input_pattern: pattern,
            text: text.to_string(),
            tool_calls,
            metadata: OutputMetadata::default(),
        });
        self
    }

    /// Set delay
    pub fn with_delay(mut self, delay: Duration) -> Self {
        self.config.delay = Some(delay);
        self
    }

    /// Set failure mode
    pub fn will_fail(mut self, message: &str) -> Self {
        self.config.should_fail = true;
        self.config.failure_message = message.to_string();
        self
    }

    /// Add state transition
    pub fn with_state_transition(mut self, state: AgentState) -> Self {
        self.config.state_transitions.push(state);
        self
    }

    /// Set resource limits
    pub fn with_resource_limits(mut self, limits: ResourceLimits) -> Self {
        self.config.agent_config.resource_limits = limits;
        self
    }

    /// Build the mock agent
    pub fn build(self) -> MockAgent {
        MockAgent::new(self.config)
    }
}

/// Test doubles for various agent types
pub struct TestDoubles;

impl TestDoubles {
    /// Create a simple echo agent
    pub fn echo_agent(name: &str) -> MockAgent {
        MockAgentBuilder::new(name)
            .agent_type("echo")
            .with_response(None, "Echo: {input}")
            .build()
    }

    /// Create an agent that always fails
    pub fn failing_agent(name: &str, error_message: &str) -> MockAgent {
        MockAgentBuilder::new(name)
            .agent_type("failing")
            .will_fail(error_message)
            .build()
    }

    /// Create an agent with tool capabilities
    pub fn tool_agent(name: &str, tools: Vec<&str>) -> MockAgent {
        let mut builder = MockAgentBuilder::new(name).agent_type("tool_capable");

        for tool in tools {
            builder = builder.with_tool(tool);
        }

        builder.build()
    }

    /// Create a slow agent
    pub fn slow_agent(name: &str, delay: Duration) -> MockAgent {
        MockAgentBuilder::new(name)
            .agent_type("slow")
            .with_delay(delay)
            .with_response(None, "Slow response")
            .build()
    }

    /// Create a stateful agent
    pub fn stateful_agent(name: &str, states: Vec<AgentState>) -> MockAgent {
        let mut builder = MockAgentBuilder::new(name).agent_type("stateful");

        for state in states {
            builder = builder.with_state_transition(state);
        }

        builder.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_agent_basic() {
        let agent = MockAgentBuilder::new("test")
            .with_response(Some("hello".to_string()), "Hi there!")
            .build();

        let input = AgentInput::text("hello world");
        let output = agent
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();

        assert_eq!(output.text, "Hi there!");
        assert_eq!(agent.execution_count(), 1);
    }

    #[tokio::test]
    async fn test_mock_agent_failure() {
        let agent = MockAgentBuilder::new("test")
            .will_fail("Test failure")
            .build();

        let input = AgentInput::text("test");
        let result = agent.execute(input, ExecutionContext::default()).await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_mock_tool() {
        let tool = MockTool::new("test_tool");

        let input = AgentInput::builder()
            .text("test")
            .parameter("test", "value")
            .build();

        let output = tool
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        assert!(output.text.contains("mock"));
        assert_eq!(tool.execution_count(), 1);
    }

    #[tokio::test]
    async fn test_test_doubles() {
        let echo = TestDoubles::echo_agent("echo");
        let input = AgentInput::text("test");
        let output = echo
            .execute(input, ExecutionContext::default())
            .await
            .unwrap();
        assert!(output.text.contains("Echo"));

        let failing = TestDoubles::failing_agent("fail", "Error");
        let input = AgentInput::text("test");
        let result = failing.execute(input, ExecutionContext::default()).await;
        assert!(result.is_err());
    }
}
