//! ABOUTME: LLM agent implementation that uses language model providers
//! ABOUTME: The fundamental agent type that powers intelligent behavior through LLMs

use crate::factory::AgentConfig;
use crate::lifecycle::{AgentStateMachine, StateMachineConfig};
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
use llmspell_providers::{ModelSpecifier, ProviderInstance, ProviderManager};
use std::sync::{Arc, Mutex};
use tracing::{debug, error, info, warn};

/// LLM-powered agent implementation
pub struct LLMAgent {
    metadata: ComponentMetadata,
    #[allow(dead_code)]
    config: AgentConfig,
    core_config: CoreAgentConfig,
    conversation: Arc<Mutex<Vec<ConversationMessage>>>,
    provider: Arc<Box<dyn ProviderInstance>>,
    state_machine: Arc<AgentStateMachine>,
}

impl LLMAgent {
    /// Create a new LLM agent
    pub async fn new(config: AgentConfig, provider_manager: Arc<ProviderManager>) -> Result<Self> {
        let metadata = ComponentMetadata::new(config.name.clone(), config.description.clone());

        // Extract model configuration
        let model_config = config
            .model
            .as_ref()
            .ok_or_else(|| LLMSpellError::Configuration {
                message: "LLM agent requires model configuration".to_string(),
                source: None,
            })?;

        // Parse model specification
        let model_spec = if model_config.provider.is_empty() {
            // Try to parse from model_id as "provider/model" format
            ModelSpecifier::parse(&model_config.model_id)?
        } else {
            // Use explicit provider and model
            ModelSpecifier {
                provider: Some(model_config.provider.clone()),
                model: model_config.model_id.clone(),
                base_url: None,
            }
        };

        // Get or create provider instance
        let provider = provider_manager
            .create_agent_from_spec(
                model_spec, None, // base_url_override
                None, // api_key
            )
            .await?;

        // Build core agent configuration
        let core_config = CoreAgentConfig {
            max_conversation_length: config
                .custom_config
                .get("max_conversation_length")
                .and_then(|v| v.as_u64())
                .map(|v| v as usize)
                .or(Some(100)),
            system_prompt: config
                .custom_config
                .get("system_prompt")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .or_else(|| {
                    Some(format!(
                        "You are {}, an AI assistant. {}",
                        config.name, config.description
                    ))
                }),
            temperature: model_config.temperature,
            max_tokens: model_config.max_tokens.map(|v| v as usize),
        };

        // Create state machine configuration optimized for LLM agents
        let state_config = StateMachineConfig {
            enable_logging: true,
            enable_hooks: true,           // Enable hooks for LLM agents
            enable_circuit_breaker: true, // Critical for LLM reliability
            max_transition_time: std::time::Duration::from_secs(30), // Longer for LLM operations
            ..StateMachineConfig::default()
        };

        let state_machine = Arc::new(AgentStateMachine::new(
            format!("llm-{}", config.name),
            state_config,
        ));

        Ok(Self {
            metadata,
            config,
            core_config,
            conversation: Arc::new(Mutex::new(Vec::new())),
            provider,
            state_machine,
        })
    }

    /// Build messages for provider including conversation history
    fn build_messages(&self, input: &str) -> Result<Vec<ConversationMessage>, LLMSpellError> {
        let mut messages = Vec::new();

        // Add system prompt if configured
        if let Some(ref system_prompt) = self.core_config.system_prompt {
            messages.push(ConversationMessage::system(system_prompt.clone()));
        }

        // Add conversation history
        if let Ok(conv) = self.conversation.lock() {
            // Respect max conversation length
            let max_len = self.core_config.max_conversation_length.unwrap_or(100);
            let start = if conv.len() > max_len {
                conv.len() - max_len
            } else {
                0
            };

            for msg in conv.iter().skip(start) {
                messages.push(msg.clone());
            }
        }

        // Add current input
        messages.push(ConversationMessage::user(input.to_string()));

        Ok(messages)
    }

    /// Get state machine for lifecycle management
    pub fn state_machine(&self) -> &Arc<AgentStateMachine> {
        &self.state_machine
    }

    /// Initialize the agent and its state machine
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing LLMAgent '{}'", self.metadata.name);
        self.state_machine.initialize().await?;
        debug!("LLMAgent '{}' initialization completed", self.metadata.name);
        Ok(())
    }

    /// Start the agent execution
    pub async fn start(&self) -> Result<()> {
        info!("Starting LLMAgent '{}'", self.metadata.name);
        self.state_machine.start().await?;
        debug!("LLMAgent '{}' started successfully", self.metadata.name);
        Ok(())
    }

    /// Pause the agent execution
    pub async fn pause(&self) -> Result<()> {
        info!("Pausing LLMAgent '{}'", self.metadata.name);
        self.state_machine.pause().await?;
        debug!("LLMAgent '{}' paused", self.metadata.name);
        Ok(())
    }

    /// Resume the agent execution
    pub async fn resume(&self) -> Result<()> {
        info!("Resuming LLMAgent '{}'", self.metadata.name);
        self.state_machine.resume().await?;
        debug!("LLMAgent '{}' resumed", self.metadata.name);
        Ok(())
    }

    /// Stop the agent execution
    pub async fn stop(&self) -> Result<()> {
        info!("Stopping LLMAgent '{}'", self.metadata.name);
        self.state_machine.stop().await?;
        debug!("LLMAgent '{}' stopped", self.metadata.name);
        Ok(())
    }

    /// Terminate the agent
    pub async fn terminate(&self) -> Result<()> {
        info!("Terminating LLMAgent '{}'", self.metadata.name);
        self.state_machine.terminate().await?;
        debug!("LLMAgent '{}' terminated", self.metadata.name);
        Ok(())
    }

    /// Check if agent is healthy
    pub async fn is_healthy(&self) -> bool {
        self.state_machine.is_healthy().await
    }
}

#[async_trait]
impl BaseAgent for LLMAgent {
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

        // Auto-initialize if in Uninitialized state (like BasicAgent does)
        if current_state == crate::lifecycle::AgentState::Uninitialized {
            if let Err(e) = self.state_machine.initialize().await {
                return Err(LLMSpellError::Component {
                    message: format!(
                        "Failed to initialize LLMAgent '{}': {}",
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
                    "LLMAgent '{}' cannot execute in state {:?}",
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
                            "Failed to start LLMAgent '{}': {}",
                            self.metadata.name, e
                        ),
                        source: None,
                    });
                }
            } else {
                return Err(LLMSpellError::Component {
                    message: format!(
                        "LLMAgent '{}' is not ready for execution (current state: {:?})",
                        self.metadata.name, current_state
                    ),
                    source: None,
                });
            }
        }

        debug!(
            "LLMAgent '{}' executing with input: {}",
            self.metadata.name, input.text
        );

        // Build messages for the provider
        let messages = self.build_messages(&input.text)?;

        // Create provider input with conversation messages as JSON
        let messages_json =
            serde_json::to_string(&messages).map_err(|e| LLMSpellError::Configuration {
                message: format!("Failed to serialize messages: {}", e),
                source: None,
            })?;

        let mut provider_input = AgentInput::text(messages_json);

        // Add temperature and max_tokens to parameters
        if let Some(temp) = self.core_config.temperature {
            provider_input
                .parameters
                .insert("temperature".to_string(), serde_json::json!(temp));
        }

        if let Some(max_tokens) = self.core_config.max_tokens {
            provider_input
                .parameters
                .insert("max_tokens".to_string(), serde_json::json!(max_tokens));
        }

        // Call the provider
        let response = self.provider.complete(&provider_input).await?;

        // Update conversation history
        if let Ok(mut conv) = self.conversation.lock() {
            conv.push(ConversationMessage::user(input.text.clone()));
            conv.push(ConversationMessage::assistant(response.text.clone()));
        }

        debug!("LLMAgent '{}' completed execution", self.metadata.name);
        Ok(response)
    }

    async fn validate_input(&self, input: &AgentInput) -> Result<(), LLMSpellError> {
        if input.text.is_empty() {
            return Err(LLMSpellError::Validation {
                message: "Input text cannot be empty".to_string(),
                field: Some("text".to_string()),
            });
        }

        // Check token limits if we can estimate
        // This is a rough estimate - actual tokenization varies by model
        let estimated_tokens = input.text.len() / 4;
        if let Some(max_tokens) = self.core_config.max_tokens {
            if estimated_tokens > max_tokens {
                return Err(LLMSpellError::Validation {
                    message: format!(
                        "Input text too long. Estimated {} tokens, max {}",
                        estimated_tokens, max_tokens
                    ),
                    field: Some("text".to_string()),
                });
            }
        }

        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput, LLMSpellError> {
        error!(
            "LLMAgent '{}' encountered error: {}",
            self.metadata.name, error
        );

        // Transition to error state for serious errors
        match &error {
            LLMSpellError::Component { .. } | LLMSpellError::Provider { .. } => {
                if let Err(state_error) = self
                    .state_machine
                    .error(format!("LLM Agent error: {}", error))
                    .await
                {
                    warn!(
                        "Failed to transition LLMAgent '{}' to error state: {}",
                        self.metadata.name, state_error
                    );
                }
            }
            _ => {
                // Minor errors don't require state transition
                warn!("LLMAgent '{}' minor error: {}", self.metadata.name, error);
            }
        }

        // For LLM agents, we might want to use the LLM to generate error responses
        // For now, return a formatted error
        Ok(AgentOutput::text(format!(
            "I encountered an error while processing your request: {}",
            error
        )))
    }
}

#[async_trait]
impl Agent for LLMAgent {
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

    async fn add_message(&mut self, message: ConversationMessage) -> Result<(), LLMSpellError> {
        self.conversation
            .lock()
            .map(|mut conv| conv.push(message))
            .map_err(|_| LLMSpellError::Component {
                message: "Failed to lock conversation".to_string(),
                source: None,
            })
    }

    async fn clear_conversation(&mut self) -> Result<(), LLMSpellError> {
        self.conversation
            .lock()
            .map(|mut conv| conv.clear())
            .map_err(|_| LLMSpellError::Component {
                message: "Failed to lock conversation".to_string(),
                source: None,
            })
    }
}

#[cfg(test)]
mod tests {
    use crate::builder::AgentBuilder;

    #[tokio::test]
    async fn test_llm_agent_creation() {
        // This test requires a provider manager setup
        // We'll need to mock this in a real test
        let config = AgentBuilder::new("test-llm", "llm")
            .description("Test LLM agent")
            .with_model("openai", "gpt-4")
            .temperature(0.7)
            .max_tokens(1000)
            .build()
            .unwrap();

        // In a real test, we'd create a provider manager and test creation
        assert_eq!(config.name, "test-llm");
        assert_eq!(config.agent_type, "llm");
        assert!(config.model.is_some());
    }
}
