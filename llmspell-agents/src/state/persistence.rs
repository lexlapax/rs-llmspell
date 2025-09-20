// ABOUTME: State persistence extension trait and implementations for agents
// ABOUTME: Provides state save/load capabilities for any agent implementation

use anyhow::Result;
use async_trait::async_trait;
use llmspell_core::traits::agent::{Agent, ConversationMessage, MessageRole as CoreMessageRole};
use llmspell_kernel::state::{
    agent_state::{AgentMetadata as PersistentMetadata, MessageRole, PersistentAgentState},
    StateManager,
};
use std::sync::Arc;
use tracing::instrument;
use tracing::{debug, info};

/// Extension trait for agents to add state persistence capabilities
#[async_trait]
pub trait StatePersistence: Agent {
    /// Get the state manager for this agent
    fn state_manager(&self) -> Option<Arc<StateManager>>;

    /// Set the state manager for this agent
    fn set_state_manager(&self, state_manager: Arc<StateManager>);

    /// Save the agent's current state
    async fn save_state(&self) -> Result<()> {
        if let Some(state_manager) = self.state_manager() {
            let state = self.create_persistent_state().await?;
            state_manager
                .save_agent_state(&state)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to save agent state: {}", e))?;
            info!("Saved state for agent {}", self.metadata().id);
            Ok(())
        } else {
            Err(anyhow::anyhow!("No state manager configured"))
        }
    }

    /// Load the agent's state from storage
    #[instrument(skip(self))]
    async fn load_state(&self) -> Result<bool> {
        if let Some(state_manager) = self.state_manager() {
            let agent_id = self.metadata().id.to_string();
            match state_manager.load_agent_state(&agent_id).await {
                Ok(Some(state)) => {
                    self.restore_from_persistent_state(state).await?;
                    info!("Loaded state for agent {}", agent_id);
                    Ok(true)
                }
                Ok(None) => {
                    debug!("No saved state found for agent {}", agent_id);
                    Ok(false)
                }
                Err(e) => Err(anyhow::anyhow!("Failed to load agent state: {}", e)),
            }
        } else {
            Err(anyhow::anyhow!("No state manager configured"))
        }
    }

    /// Create a persistent state representation from current agent state
    #[instrument(skip(self))]
    async fn create_persistent_state(&self) -> Result<PersistentAgentState> {
        let mut state =
            PersistentAgentState::new(self.metadata().id.to_string(), self.metadata().name.clone());

        // Set metadata
        state.metadata = PersistentMetadata {
            name: self.metadata().name.clone(),
            description: Some(self.metadata().description.clone()),
            version: "1.0.0".to_string(),
            capabilities: vec![], // TODO: Extract from agent
            provider_config: None,
            tags: vec![],
        };

        // Convert conversation history
        let conversation = self.get_conversation().await?;
        for msg in conversation {
            let role = match msg.role {
                CoreMessageRole::User => MessageRole::User,
                CoreMessageRole::Assistant => MessageRole::Assistant,
                CoreMessageRole::System => MessageRole::System,
            };
            state.add_message(role, msg.content.clone());
        }

        // Add tool usage stats if available
        if let Some(tool_stats) = self.tool_usage_stats() {
            for (tool_name, stats) in tool_stats {
                state.record_tool_usage(&tool_name, stats.duration_ms, stats.success);
            }
        }

        Ok(state)
    }

    /// Restore agent state from persistent state
    #[instrument(skip(self))]
    async fn restore_from_persistent_state(&self, state: PersistentAgentState) -> Result<()> {
        // Clear current conversation
        self.clear_conversation().await?;

        // Restore conversation history
        for msg in &state.state.conversation_history {
            let role = match msg.role {
                MessageRole::User => CoreMessageRole::User,
                MessageRole::Assistant | MessageRole::Tool => CoreMessageRole::Assistant, // Map Tool to Assistant for now
                MessageRole::System => CoreMessageRole::System,
            };
            self.add_message(ConversationMessage::new(role, msg.content.clone()))
                .await?;
        }

        info!(
            "Restored {} messages for agent {}",
            state.state.conversation_history.len(),
            self.metadata().id
        );

        Ok(())
    }

    /// Get tool usage statistics (optional override)
    fn tool_usage_stats(&self) -> Option<std::collections::HashMap<String, ToolStats>> {
        None
    }
}

/// Tool usage statistics
#[derive(Debug, Clone)]
pub struct ToolStats {
    pub invocations: u32,
    pub duration_ms: u64,
    pub success: bool,
}

/// State manager holder trait for concrete implementations
pub trait StateManagerHolder {
    fn state_manager(&self) -> Option<Arc<StateManager>>;
    fn set_state_manager(&self, state_manager: Arc<StateManager>);
}

/// Macro to implement `PersistentAgent` trait for types that implement Agent + `StatePersistence`
#[macro_export]
macro_rules! impl_persistent_agent {
    ($agent_type:ty) => {
        #[async_trait::async_trait]
        impl llmspell_kernel::state::agent_state::PersistentAgent for $agent_type {
            fn agent_id(&self) -> &str {
                &self.agent_id_string
            }

            fn get_persistent_state(
                &self,
            ) -> llmspell_kernel::state::StateResult<llmspell_kernel::state::PersistentAgentState>
            {
                // Since we need async, we use block_on here
                let rt = tokio::runtime::Handle::current();
                rt.block_on(self.create_persistent_state()).map_err(|e| {
                    llmspell_kernel::state::StateError::SerializationError(e.to_string())
                })
            }

            fn apply_persistent_state(
                &self,
                state: llmspell_kernel::state::PersistentAgentState,
            ) -> llmspell_kernel::state::StateResult<()> {
                // Since we need async, we use block_on here
                let rt = tokio::runtime::Handle::current();
                rt.block_on(self.restore_from_persistent_state(state))
                    .map_err(|e| {
                        llmspell_kernel::state::StateError::SerializationError(e.to_string())
                    })
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_core::traits::agent::AgentConfig;
    use llmspell_core::traits::base_agent::BaseAgent;
    use llmspell_core::types::{AgentInput, AgentOutput};
    use llmspell_core::{ComponentMetadata, ExecutionContext};
    use std::sync::Mutex;

    // Mock agent for testing
    struct MockAgent {
        metadata: ComponentMetadata,
        agent_id_string: String,
        config: AgentConfig,
        conversation: Arc<Mutex<Vec<ConversationMessage>>>,
        state_manager: Arc<parking_lot::RwLock<Option<Arc<StateManager>>>>,
    }

    #[async_trait]
    impl BaseAgent for MockAgent {
        fn metadata(&self) -> &ComponentMetadata {
            &self.metadata
        }

        #[instrument(skip(self))]
        async fn execute_impl(
            &self,
            _input: AgentInput,
            _context: ExecutionContext,
        ) -> llmspell_core::Result<AgentOutput> {
            Ok(AgentOutput::text("Mock response"))
        }

        #[instrument(skip(self))]
        async fn validate_input(&self, _input: &AgentInput) -> llmspell_core::Result<()> {
            Ok(())
        }

        #[instrument(skip(self))]
        async fn handle_error(
            &self,
            error: llmspell_core::LLMSpellError,
        ) -> llmspell_core::Result<AgentOutput> {
            Ok(AgentOutput::text(format!("Error: {error}")))
        }
    }

    #[async_trait]
    impl Agent for MockAgent {
        fn config(&self) -> &AgentConfig {
            &self.config
        }

        #[instrument(skip(self))]
        async fn get_conversation(&self) -> llmspell_core::Result<Vec<ConversationMessage>> {
            self.conversation
                .lock()
                .map(|conv| conv.clone())
                .map_err(|_| llmspell_core::LLMSpellError::Component {
                    message: "Failed to lock conversation".to_string(),
                    source: None,
                })
        }

        #[instrument(skip(self))]
        async fn add_message(&self, message: ConversationMessage) -> llmspell_core::Result<()> {
            self.conversation
                .lock()
                .map(|mut conv| conv.push(message))
                .map_err(|_| llmspell_core::LLMSpellError::Component {
                    message: "Failed to lock conversation".to_string(),
                    source: None,
                })
        }

        #[instrument(skip(self))]
        async fn clear_conversation(&self) -> llmspell_core::Result<()> {
            self.conversation
                .lock()
                .map(|mut conv| conv.clear())
                .map_err(|_| llmspell_core::LLMSpellError::Component {
                    message: "Failed to lock conversation".to_string(),
                    source: None,
                })
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
    }

    // Implement PersistentAgent using the macro
    impl_persistent_agent!(MockAgent);
    #[tokio::test]
    async fn test_state_persistence_trait() {
        let metadata = ComponentMetadata::new("test-agent".to_string(), "Test agent".to_string());
        let agent_id_string = metadata.id.to_string();
        let agent = MockAgent {
            metadata,
            agent_id_string,
            config: AgentConfig::default(),
            conversation: Arc::new(Mutex::new(vec![])),
            state_manager: Arc::new(parking_lot::RwLock::new(None)),
        };

        // Add some conversation
        agent
            .add_message(ConversationMessage::user("Hello".to_string()))
            .await
            .unwrap();

        // Create persistent state
        let state = agent.create_persistent_state().await.unwrap();
        assert_eq!(state.state.conversation_history.len(), 1);
    }
}
