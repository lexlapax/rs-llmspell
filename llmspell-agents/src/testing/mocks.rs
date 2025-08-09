//! ABOUTME: Mock implementations for testing agent infrastructure
//! ABOUTME: Provides configurable mock agents, tools, and providers for unit and integration testing

#![allow(clippy::significant_drop_tightening)]

use crate::factory::{AgentConfig, ResourceLimits};
use crate::lifecycle::{
    events::{LifecycleEvent, LifecycleEventData, LifecycleEventType},
    state_machine::{AgentState, AgentStateMachine},
};
use crate::state::{StateManagerHolder, StatePersistence};
use crate::StateMachineConfig;
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
use llmspell_state_persistence::ToolUsageStats;
use llmspell_state_persistence::{PersistentAgentState, StateManager};
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
    /// Failure message if `should_fail` is true
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
    state_manager: Arc<parking_lot::RwLock<Option<Arc<StateManager>>>>,
    agent_id_string: String,
}

impl MockAgent {
    /// Create new mock agent
    #[must_use]
    pub fn new(config: MockAgentConfig) -> Self {
        let metadata = ComponentMetadata {
            id: ComponentId::from_name(&config.agent_config.name),
            name: config.agent_config.name.clone(),
            version: Version::new(1, 0, 0),
            description: config.agent_config.description.clone(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        let state_machine = AgentStateMachine::new(
            config.agent_config.name.clone(),
            StateMachineConfig::default(),
        );

        let core_config = CoreAgentConfig {
            max_conversation_length: Some(100),
            system_prompt: None,
            temperature: Some(0.7),
            max_tokens: Some(2000),
        };

        let agent_id_string = metadata.id.to_string();

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
            state_manager: Arc::new(parking_lot::RwLock::new(None)),
            agent_id_string,
        }
    }

    /// Set event sender for lifecycle events
    pub fn set_event_sender(&mut self, sender: broadcast::Sender<LifecycleEvent>) {
        self.event_sender = Some(sender);
    }

    /// Get execution count
    ///
    /// # Panics
    ///
    /// Panics if the Mutex is poisoned
    #[must_use]
    pub fn execution_count(&self) -> usize {
        *self.execution_count.lock().unwrap()
    }

    /// Get last input
    ///
    /// # Panics
    ///
    /// Panics if the Mutex is poisoned
    #[must_use]
    pub fn last_input(&self) -> Option<AgentInput> {
        self.last_input.lock().unwrap().clone()
    }

    /// Get last context
    ///
    /// # Panics
    ///
    /// Panics if the Mutex is poisoned
    #[must_use]
    pub fn last_context(&self) -> Option<ExecutionContext> {
        self.last_context.lock().unwrap().clone()
    }

    /// Add response to mock agent
    ///
    /// # Panics
    ///
    /// Panics if the Mutex is poisoned
    pub fn add_response(&self, response: MockResponse) {
        self.config.lock().unwrap().responses.push(response);
    }

    /// Set failure mode
    ///
    /// # Panics
    ///
    /// Panics if the Mutex is poisoned
    pub fn set_failure(&self, should_fail: bool, message: &str) {
        let mut config = self.config.lock().unwrap();
        config.should_fail = should_fail;
        config.failure_message = message.to_string();
    }

    /// Add state transition
    ///
    /// # Panics
    ///
    /// Panics if the Mutex is poisoned
    pub fn add_state_transition(&self, state: AgentState) {
        self.config.lock().unwrap().state_transitions.push(state);
    }

    /// Emit event
    fn emit_event(&self, event: LifecycleEvent) {
        if let Some(sender) = &self.event_sender {
            let _ = sender.send(event);
        }
    }

    /// Initialize the agent
    ///
    /// # Errors
    ///
    /// Returns an error if state machine initialization fails
    pub async fn initialize(&mut self) -> Result<()> {
        self.state_machine
            .initialize()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to initialize agent: {}", e))
    }

    /// Start the agent
    ///
    /// # Errors
    ///
    /// Returns an error if state machine start fails
    pub async fn start(&mut self) -> Result<()> {
        self.state_machine
            .start()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to start agent: {}", e))
    }

    /// Stop the agent
    ///
    /// # Errors
    ///
    /// Returns an error if state machine stop fails
    pub async fn stop(&mut self) -> Result<()> {
        self.state_machine
            .stop()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to stop agent: {}", e))
    }

    /// Terminate the agent
    ///
    /// # Errors
    ///
    /// Returns an error if state machine termination fails
    pub async fn terminate(&mut self) -> Result<()> {
        self.state_machine
            .terminate()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to terminate agent: {}", e))
    }
}

#[async_trait]
impl BaseAgent for MockAgent {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    /// # Panics
    ///
    /// Panics if any Mutex is poisoned
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
                    to: *state,
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
            return Err(LLMSpellError::Component {
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

        // Add messages to conversation history
        {
            let mut conversation = self.conversation.lock().unwrap();
            // Add user message
            conversation.push(ConversationMessage {
                role: llmspell_core::traits::agent::MessageRole::User,
                content: input.text.clone(),
                timestamp: chrono::Utc::now(),
            });
            // Add assistant response
            conversation.push(ConversationMessage {
                role: llmspell_core::traits::agent::MessageRole::Assistant,
                content: output.text.clone(),
                timestamp: chrono::Utc::now(),
            });
        }

        Ok(output)
    }

    async fn validate_input(&self, _input: &AgentInput) -> Result<(), LLMSpellError> {
        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput, LLMSpellError> {
        Ok(AgentOutput {
            text: format!("Mock error handling: {error}"),
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

    async fn add_message(&self, message: ConversationMessage) -> Result<(), LLMSpellError> {
        self.conversation.lock().unwrap().push(message);
        Ok(())
    }

    async fn clear_conversation(&self) -> Result<(), LLMSpellError> {
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

impl StateManagerHolder for MockAgent {
    fn state_manager(&self) -> Option<Arc<StateManager>> {
        self.state_manager.read().clone()
    }

    fn set_state_manager(&self, state_manager: Arc<StateManager>) {
        *self.state_manager.write() = Some(state_manager);
    }
}

#[async_trait]
impl StatePersistence for MockAgent {
    fn state_manager(&self) -> Option<Arc<StateManager>> {
        StateManagerHolder::state_manager(self)
    }

    fn set_state_manager(&self, state_manager: Arc<StateManager>) {
        StateManagerHolder::set_state_manager(self, state_manager);
    }

    async fn save_state(&self) -> Result<()> {
        if let Some(state_manager) = StateManagerHolder::state_manager(self) {
            let state = self.create_persistent_state().await?;
            state_manager
                .save_agent_state(&state)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to save state: {}", e))?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("No state manager configured"))
        }
    }

    async fn load_state(&self) -> Result<bool> {
        if let Some(state_manager) = StateManagerHolder::state_manager(self) {
            if let Some(state) = state_manager
                .load_agent_state(&self.agent_id_string)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to load state: {}", e))?
            {
                self.restore_from_persistent_state(state).await?;
                Ok(true)
            } else {
                Ok(false)
            }
        } else {
            Err(anyhow::anyhow!("No state manager configured"))
        }
    }

    async fn create_persistent_state(&self) -> Result<PersistentAgentState> {
        let conversation = self.conversation.lock().unwrap().clone();
        let conversation_history = conversation
            .into_iter()
            .map(
                |msg| llmspell_state_persistence::agent_state::ConversationMessage {
                    role: match msg.role {
                        llmspell_core::traits::agent::MessageRole::System => {
                            llmspell_state_persistence::agent_state::MessageRole::System
                        }
                        llmspell_core::traits::agent::MessageRole::User => {
                            llmspell_state_persistence::agent_state::MessageRole::User
                        }
                        llmspell_core::traits::agent::MessageRole::Assistant => {
                            llmspell_state_persistence::agent_state::MessageRole::Assistant
                        }
                    },
                    content: msg.content,
                    timestamp: std::time::SystemTime::from(msg.timestamp),
                    metadata: None,
                },
            )
            .collect();

        let state_data = llmspell_state_persistence::agent_state::AgentStateData {
            conversation_history,
            context_variables: HashMap::new(),
            tool_usage_stats: ToolUsageStats::default(),
            execution_state: llmspell_state_persistence::agent_state::ExecutionState::Idle,
            custom_data: HashMap::new(),
        };

        let metadata = llmspell_state_persistence::agent_state::AgentMetadata {
            name: self.metadata.name.clone(),
            description: Some(self.metadata.description.clone()),
            version: self.metadata.version.to_string(),
            capabilities: vec![],
            provider_config: Some(serde_json::to_value(&self.core_config).unwrap_or_default()),
            tags: vec![],
        };

        Ok(PersistentAgentState {
            agent_id: self.agent_id_string.clone(),
            agent_type: "mock".to_string(),
            state: state_data,
            metadata,
            creation_time: std::time::SystemTime::now(),
            last_modified: std::time::SystemTime::now(),
            schema_version: 1,
            hook_registrations: vec![],
            last_hook_execution: None,
            correlation_context: None,
        })
    }

    async fn restore_from_persistent_state(&self, state: PersistentAgentState) -> Result<()> {
        let mut conversation = self.conversation.lock().unwrap();
        conversation.clear();

        for entry in state.state.conversation_history {
            let role = match entry.role {
                llmspell_state_persistence::agent_state::MessageRole::System => {
                    llmspell_core::traits::agent::MessageRole::System
                }
                llmspell_state_persistence::agent_state::MessageRole::User => {
                    llmspell_core::traits::agent::MessageRole::User
                }
                llmspell_state_persistence::agent_state::MessageRole::Assistant
                | llmspell_state_persistence::agent_state::MessageRole::Tool => {
                    llmspell_core::traits::agent::MessageRole::Assistant
                } // Map Tool to Assistant
            };

            conversation.push(ConversationMessage {
                role,
                content: entry.content,
                timestamp: chrono::DateTime::<chrono::Utc>::from(entry.timestamp),
            });
        }

        Ok(())
    }
}

// Implement the PersistentAgent trait using our macro
crate::impl_persistent_agent!(MockAgent);

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
    #[must_use]
    pub fn new(name: &str) -> Self {
        let metadata = ComponentMetadata {
            id: ComponentId::from_name(name),
            name: name.to_string(),
            version: Version::new(1, 0, 0),
            description: format!("Mock tool: {name}"),
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
    ///
    /// # Panics
    ///
    /// Panics if the Mutex is poisoned.
    pub fn set_response(&self, input_key: &str, output: ToolOutput) {
        self.responses
            .lock()
            .unwrap()
            .insert(input_key.to_string(), output);
    }

    /// Set execution delay
    pub const fn set_delay(&mut self, delay: Duration) {
        self.delay = Some(delay);
    }

    /// Set failure mode
    pub const fn set_failure(&mut self, should_fail: bool) {
        self.should_fail = should_fail;
    }

    /// Get execution count
    ///
    /// # Panics
    ///
    /// Panics if the Mutex is poisoned
    #[must_use]
    pub fn execution_count(&self) -> usize {
        *self.execution_count.lock().unwrap()
    }
}

#[async_trait]
impl BaseAgent for MockTool {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    /// # Panics
    ///
    /// Panics if any Mutex is poisoned
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
                tool_name: Some(self.metadata.name.clone()),
                message: "Mock tool failure".to_string(),
                source: None,
            });
        }

        // Extract parameters from input
        let params = &input.parameters;

        // Check for pre-programmed response
        let input_str = serde_json::to_string(params).unwrap_or_default();
        let responses = self.responses.lock().unwrap();

        let tool_output = responses.get(&input_str).map_or_else(
            || {
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
            },
            Clone::clone,
        );

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
        Ok(AgentOutput::text(format!("Tool error: {error}")))
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
    #[must_use]
    pub fn new(name: &str) -> Self {
        let mut config = MockAgentConfig::default();
        config.agent_config.name = name.to_string();

        Self { config }
    }

    /// Set agent type
    #[must_use]
    pub fn agent_type(mut self, agent_type: &str) -> Self {
        self.config.agent_config.agent_type = agent_type.to_string();
        self
    }

    /// Add allowed tool
    #[must_use]
    pub fn with_tool(mut self, tool_name: &str) -> Self {
        self.config
            .agent_config
            .allowed_tools
            .push(tool_name.to_string());
        self
    }

    /// Add response
    #[must_use]
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
    #[must_use]
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
    #[must_use]
    pub const fn with_delay(mut self, delay: Duration) -> Self {
        self.config.delay = Some(delay);
        self
    }

    /// Set failure mode
    #[must_use]
    pub fn will_fail(mut self, message: &str) -> Self {
        self.config.should_fail = true;
        self.config.failure_message = message.to_string();
        self
    }

    /// Add state transition
    #[must_use]
    pub fn with_state_transition(mut self, state: AgentState) -> Self {
        self.config.state_transitions.push(state);
        self
    }

    /// Set resource limits
    #[must_use]
    pub const fn with_resource_limits(mut self, limits: ResourceLimits) -> Self {
        self.config.agent_config.resource_limits = limits;
        self
    }

    /// Build the mock agent
    #[must_use]
    pub fn build(self) -> MockAgent {
        MockAgent::new(self.config)
    }
}

/// Test doubles for various agent types
pub struct TestDoubles;

impl TestDoubles {
    /// Create a simple echo agent
    #[must_use]
    pub fn echo_agent(name: &str) -> MockAgent {
        MockAgentBuilder::new(name)
            .agent_type("echo")
            .with_response(None, "Echo: {input}")
            .build()
    }

    /// Create an agent that always fails
    #[must_use]
    pub fn failing_agent(name: &str, error_message: &str) -> MockAgent {
        MockAgentBuilder::new(name)
            .agent_type("failing")
            .will_fail(error_message)
            .build()
    }

    /// Create an agent with tool capabilities
    #[must_use]
    pub fn tool_agent(name: &str, tools: Vec<&str>) -> MockAgent {
        let mut builder = MockAgentBuilder::new(name).agent_type("tool_capable");

        for tool in tools {
            builder = builder.with_tool(tool);
        }

        builder.build()
    }

    /// Create a slow agent
    #[must_use]
    pub fn slow_agent(name: &str, delay: Duration) -> MockAgent {
        MockAgentBuilder::new(name)
            .agent_type("slow")
            .with_delay(delay)
            .with_response(None, "Slow response")
            .build()
    }

    /// Create a stateful agent
    #[must_use]
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
