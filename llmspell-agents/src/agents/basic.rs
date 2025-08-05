//! ABOUTME: Basic agent implementation
//! ABOUTME: Simple agent with minimal functionality for testing and examples

use crate::factory::AgentConfig;
use crate::lifecycle::{AgentStateMachine, StateMachineConfig};
use crate::state::persistence::{StateManagerHolder, StatePersistence};
use anyhow::Result;
use async_trait::async_trait;
use llmspell_core::{
    traits::{
        agent::{Agent, AgentConfig as CoreAgentConfig, ConversationMessage},
        base_agent::BaseAgent,
    },
    types::{AgentInput, AgentOutput},
    ComponentMetadata, ExecutionContext, LLMSpellError,
};
use llmspell_state_persistence::StateManager;
use std::sync::{Arc, Mutex};
use tracing::{debug, error, info, warn};

/// Basic agent implementation
pub struct BasicAgent {
    metadata: ComponentMetadata,
    agent_id_string: String, // Cache string representation of agent ID
    config: AgentConfig,
    core_config: CoreAgentConfig,
    conversation: Arc<Mutex<Vec<ConversationMessage>>>,
    state_machine: Arc<AgentStateMachine>,
    state_manager: Arc<parking_lot::RwLock<Option<Arc<StateManager>>>>,
}

impl BasicAgent {
    /// Create a new basic agent
    ///
    /// # Errors
    ///
    /// Returns an error if agent configuration is invalid
    pub fn new(config: AgentConfig) -> Result<Self> {
        let metadata = ComponentMetadata::new(config.name.clone(), config.description.clone());
        let agent_id_string = metadata.id.to_string();

        let core_config = CoreAgentConfig {
            max_conversation_length: Some(100),
            system_prompt: Some(format!("You are {}, a basic agent.", config.name)),
            temperature: None,
            max_tokens: None,
        };

        // Create state machine configuration optimized for basic agents
        let state_config = StateMachineConfig {
            enable_logging: true,
            enable_hooks: true, // Enable hooks for basic agents
            enable_circuit_breaker: true,
            ..StateMachineConfig::default()
        };

        let state_machine = Arc::new(AgentStateMachine::new(
            format!("basic-{}", config.name),
            state_config,
        ));

        Ok(Self {
            metadata,
            agent_id_string,
            config,
            core_config,
            conversation: Arc::new(Mutex::new(Vec::new())),
            state_machine,
            state_manager: Arc::new(parking_lot::RwLock::new(None)),
        })
    }

    /// Get configuration
    #[must_use]
    pub const fn get_config(&self) -> &AgentConfig {
        &self.config
    }

    /// Get state machine for lifecycle management
    #[must_use]
    pub const fn state_machine(&self) -> &Arc<AgentStateMachine> {
        &self.state_machine
    }

    /// Initialize the agent and its state machine
    ///
    /// # Errors
    ///
    /// Returns an error if state machine initialization fails
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing BasicAgent '{}'", self.metadata.name);
        self.state_machine.initialize().await?;
        debug!(
            "BasicAgent '{}' initialization completed",
            self.metadata.name
        );
        Ok(())
    }

    /// Start the agent execution
    ///
    /// # Errors
    ///
    /// Returns an error if the state machine cannot transition to the Running state
    pub async fn start(&self) -> Result<()> {
        info!("Starting BasicAgent '{}'", self.metadata.name);

        // Note: Automatic state loading requires mutable self reference
        // Users should call load_state() explicitly before start() if needed
        if self.state_manager.read().is_some() {
            debug!("State manager available for BasicAgent '{}'. Call load_state() before start() to restore previous state.", self.metadata.name);
        }

        self.state_machine.start().await?;
        debug!("BasicAgent '{}' started successfully", self.metadata.name);
        Ok(())
    }

    /// Pause the agent execution
    ///
    /// # Errors
    ///
    /// Returns an error if the state machine cannot transition to the Paused state
    pub async fn pause(&self) -> Result<()> {
        info!("Pausing BasicAgent '{}'", self.metadata.name);
        self.state_machine.pause().await?;

        // Automatically save state when pausing if state manager is available
        if self.state_manager.read().is_some() {
            debug!(
                "Saving state for BasicAgent '{}' on pause",
                self.metadata.name
            );
            if let Err(e) = self.save_state().await {
                warn!(
                    "Failed to save state during pause for agent '{}': {}",
                    self.metadata.name, e
                );
                // Continue with pause even if state save fails
            }
        }

        debug!("BasicAgent '{}' paused", self.metadata.name);
        Ok(())
    }

    /// Resume the agent execution
    ///
    /// # Errors
    ///
    /// Returns an error if the state machine cannot transition from Paused to Running state
    pub async fn resume(&self) -> Result<()> {
        info!("Resuming BasicAgent '{}'", self.metadata.name);

        // Note: Automatic state loading requires mutable self reference
        // Users should call load_state() explicitly before resume() if needed
        if self.state_manager.read().is_some() {
            debug!("State manager available for BasicAgent '{}'. Call load_state() before resume() to restore saved state.", self.metadata.name);
        }

        self.state_machine.resume().await?;
        debug!("BasicAgent '{}' resumed", self.metadata.name);
        Ok(())
    }

    /// Stop the agent execution
    ///
    /// # Errors
    ///
    /// Returns an error if the state machine cannot transition to the Stopped state
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping BasicAgent '{}'", self.metadata.name);

        // Automatically save state before stopping if state manager is available
        if self.state_manager.read().is_some() {
            debug!(
                "Saving final state for BasicAgent '{}' before stop",
                self.metadata.name
            );
            if let Err(e) = self.save_state().await {
                warn!(
                    "Failed to save state during stop for agent '{}': {}",
                    self.metadata.name, e
                );
                // Continue with stop even if state save fails
            }
        }

        self.state_machine.stop().await?;
        debug!("BasicAgent '{}' stopped", self.metadata.name);
        Ok(())
    }

    /// Terminate the agent
    ///
    /// # Errors
    ///
    /// Returns an error if the state machine cannot transition to the Terminated state
    pub async fn terminate(&self) -> Result<()> {
        info!("Terminating BasicAgent '{}'", self.metadata.name);
        self.state_machine.terminate().await?;
        debug!("BasicAgent '{}' terminated", self.metadata.name);
        Ok(())
    }

    /// Check if agent is healthy
    pub async fn is_healthy(&self) -> bool {
        self.state_machine.is_healthy().await
    }
}

#[async_trait]
impl BaseAgent for BasicAgent {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute(
        &self,
        input: AgentInput,
        _context: ExecutionContext,
    ) -> Result<AgentOutput, LLMSpellError> {
        // Check if agent is in a state that allows execution, auto-initialize if needed
        let current_state = self.state_machine.current_state().await;

        // Auto-initialize if in Uninitialized state
        if current_state == crate::lifecycle::AgentState::Uninitialized {
            if let Err(e) = self.state_machine.initialize().await {
                return Err(LLMSpellError::Component {
                    message: format!(
                        "Failed to initialize BasicAgent '{}': {}",
                        self.metadata.name, e
                    ),
                    source: None,
                });
            }
        }

        // Re-check state after potential initialization
        let current_state = self.state_machine.current_state().await;
        if !current_state.can_execute() {
            return Err(LLMSpellError::Component {
                message: format!(
                    "BasicAgent '{}' cannot execute in state {:?}",
                    self.metadata.name, current_state
                ),
                source: None,
            });
        }

        // Ensure agent is running
        if !self
            .state_machine
            .is_state(crate::lifecycle::AgentState::Running)
            .await
        {
            // Try to start the agent if it's ready
            if self
                .state_machine
                .is_state(crate::lifecycle::AgentState::Ready)
                .await
            {
                if let Err(e) = self.state_machine.start().await {
                    return Err(LLMSpellError::Component {
                        message: format!(
                            "Failed to start BasicAgent '{}': {}",
                            self.metadata.name, e
                        ),
                        source: None,
                    });
                }
            } else {
                return Err(LLMSpellError::Component {
                    message: format!(
                        "BasicAgent '{}' is not ready for execution (current state: {:?})",
                        self.metadata.name, current_state
                    ),
                    source: None,
                });
            }
        }

        debug!(
            "BasicAgent '{}' executing with input: {}",
            self.metadata.name, input.text
        );

        // Basic implementation: echo the input
        let response = format!(
            "BasicAgent '{}' received: {}",
            self.metadata.name, input.text
        );

        // Add to conversation history
        if let Ok(mut conv) = self.conversation.lock() {
            conv.push(ConversationMessage::user(input.text.clone()));
            conv.push(ConversationMessage::assistant(response.clone()));
        }

        debug!("BasicAgent '{}' completed execution", self.metadata.name);
        Ok(AgentOutput::text(response))
    }

    async fn validate_input(&self, input: &AgentInput) -> Result<(), LLMSpellError> {
        if input.text.is_empty() {
            return Err(LLMSpellError::Validation {
                message: "Input text cannot be empty".to_string(),
                field: Some("text".to_string()),
            });
        }

        // Check resource limits
        if input.text.len()
            > (usize::try_from(self.config.resource_limits.max_memory_mb).unwrap_or(usize::MAX)
                * 1024)
        {
            return Err(LLMSpellError::Validation {
                message: "Input text exceeds memory limit".to_string(),
                field: Some("text".to_string()),
            });
        }

        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput, LLMSpellError> {
        error!(
            "BasicAgent '{}' encountered error: {}",
            self.metadata.name, error
        );

        // Transition to error state if the error is serious
        match &error {
            LLMSpellError::Component { .. } | LLMSpellError::Provider { .. } => {
                if let Err(state_error) = self
                    .state_machine
                    .error(format!("Agent error: {error}"))
                    .await
                {
                    warn!(
                        "Failed to transition BasicAgent '{}' to error state: {}",
                        self.metadata.name, state_error
                    );
                }
            }
            _ => {
                // Minor errors don't require state transition
                warn!("BasicAgent '{}' minor error: {}", self.metadata.name, error);
            }
        }

        Ok(AgentOutput::text(format!(
            "BasicAgent '{}' encountered error: {}",
            self.metadata.name, error
        )))
    }
}

#[async_trait]
impl Agent for BasicAgent {
    fn config(&self) -> &CoreAgentConfig {
        &self.core_config
    }

    async fn get_conversation(&self) -> Result<Vec<ConversationMessage>, LLMSpellError> {
        self.conversation
            .lock()
            .map(|conv| conv.clone())
            .map_err(|_| LLMSpellError::Component {
                message: "Failed to lock conversation".to_string(),
                source: None,
            })
    }

    async fn add_message(&self, message: ConversationMessage) -> Result<(), LLMSpellError> {
        self.conversation
            .lock()
            .map(|mut conv| conv.push(message))
            .map_err(|_| LLMSpellError::Component {
                message: "Failed to lock conversation".to_string(),
                source: None,
            })
    }

    async fn clear_conversation(&self) -> Result<(), LLMSpellError> {
        self.conversation
            .lock()
            .map(|mut conv| conv.clear())
            .map_err(|_| LLMSpellError::Component {
                message: "Failed to lock conversation".to_string(),
                source: None,
            })
    }
}

// Implement StateManagerHolder
impl StateManagerHolder for BasicAgent {
    fn state_manager(&self) -> Option<Arc<StateManager>> {
        self.state_manager.read().clone()
    }

    fn set_state_manager(&self, state_manager: Arc<StateManager>) {
        *self.state_manager.write() = Some(state_manager);
    }
}

// Implement StatePersistence
#[async_trait]
impl StatePersistence for BasicAgent {
    fn state_manager(&self) -> Option<Arc<StateManager>> {
        StateManagerHolder::state_manager(self)
    }

    fn set_state_manager(&self, state_manager: Arc<StateManager>) {
        StateManagerHolder::set_state_manager(self, state_manager);
    }
}

// Implement PersistentAgent using the macro
crate::impl_persistent_agent!(BasicAgent);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::AgentBuilder;
    #[tokio::test]
    async fn test_basic_agent_creation() {
        let config = AgentBuilder::basic("test-agent")
            .description("Test agent")
            .build()
            .unwrap();

        let agent = BasicAgent::new(config).unwrap();
        assert_eq!(agent.metadata().name, "test-agent");
    }
    #[tokio::test]
    async fn test_basic_agent_execution() {
        let config = AgentBuilder::basic("test-agent").build().unwrap();
        let agent = BasicAgent::new(config).unwrap();

        // Initialize and start the agent
        agent.initialize().await.unwrap();

        let input = AgentInput::text("Hello, agent!");
        let context = ExecutionContext::default();

        let output = agent.execute(input, context).await.unwrap();
        assert!(output
            .text
            .contains("BasicAgent 'test-agent' received: Hello, agent!"));
    }
    #[tokio::test]
    async fn test_basic_agent_conversation() {
        let config = AgentBuilder::basic("test-agent").build().unwrap();
        let agent = BasicAgent::new(config).unwrap();

        // Add messages
        agent
            .add_message(ConversationMessage::user("Hello".to_string()))
            .await
            .unwrap();
        agent
            .add_message(ConversationMessage::assistant("Hi there!".to_string()))
            .await
            .unwrap();

        // Get conversation
        let conv = agent.get_conversation().await.unwrap();
        assert_eq!(conv.len(), 2);

        // Clear conversation
        agent.clear_conversation().await.unwrap();
        let conv = agent.get_conversation().await.unwrap();
        assert_eq!(conv.len(), 0);
    }
    #[tokio::test]
    async fn test_basic_agent_state_machine_integration() {
        let config = AgentBuilder::basic("test-agent").build().unwrap();
        let agent = BasicAgent::new(config).unwrap();

        // Agent should start in Uninitialized state
        assert!(!agent.is_healthy().await);

        // Initialize the agent
        agent.initialize().await.unwrap();
        assert!(agent.is_healthy().await);

        // Start the agent
        agent.start().await.unwrap();

        // Agent should now be able to execute
        let input = AgentInput::text("Test execution");
        let context = ExecutionContext::default();
        let output = agent.execute(input, context).await.unwrap();
        assert!(output
            .text
            .contains("BasicAgent 'test-agent' received: Test execution"));

        // Pause the agent
        agent.pause().await.unwrap();

        // Resume the agent
        agent.resume().await.unwrap();

        // Stop the agent
        agent.stop().await.unwrap();

        // Terminate the agent
        agent.terminate().await.unwrap();
        assert!(!agent.is_healthy().await);
    }
    #[tokio::test]
    async fn test_basic_agent_execution_state_validation() {
        let config = AgentBuilder::basic("test-agent").build().unwrap();
        let agent = BasicAgent::new(config).unwrap();

        // Execute should auto-initialize and succeed (graceful handling)
        let input = AgentInput::text("Hello");
        let context = ExecutionContext::default();
        let result = agent.execute(input.clone(), context.clone()).await;
        assert!(
            result.is_ok(),
            "Agent should auto-initialize on first execution"
        );

        // Subsequent executions should also work
        let result = agent.execute(input, context).await;
        assert!(result.is_ok());
    }
    #[tokio::test]
    async fn test_basic_agent_validation() {
        let config = AgentBuilder::basic("test-agent").build().unwrap();
        let agent = BasicAgent::new(config).unwrap();

        // Empty input should fail
        let empty_input = AgentInput::text("");
        let result = agent.validate_input(&empty_input).await;
        assert!(result.is_err());

        // Valid input should pass
        let valid_input = AgentInput::text("Valid input");
        let result = agent.validate_input(&valid_input).await;
        assert!(result.is_ok());
    }
}
