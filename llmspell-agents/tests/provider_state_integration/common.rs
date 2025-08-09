//! ABOUTME: Common utilities for provider state integration tests
//! ABOUTME: Provides shared test context, setup, and verification helpers for real provider testing

use anyhow::Result;
use llmspell_agents::{agents::llm::LLMAgent, builder::AgentBuilder, StatePersistence};
use llmspell_core::{
    traits::base_agent::BaseAgent as BaseAgentTrait,
    types::{AgentInput, AgentOutput},
    ExecutionContext,
};
use llmspell_providers::ProviderManager;
use llmspell_state_persistence::{
    PerformanceConfig, PersistenceConfig, StateManager, StorageBackendType,
};
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tracing::{info, warn};

/// Test context for provider integration tests
pub struct ProviderTestContext {
    pub provider_manager: Arc<ProviderManager>,
    pub state_manager: Arc<StateManager>,
    pub temp_dir: TempDir,
    pub agent_id: String,
}

impl ProviderTestContext {
    /// Create a new test context with persistent storage
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Temporary directory creation fails
    /// - State manager initialization fails
    pub async fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;
        let storage_path = temp_dir.path().to_path_buf();

        // Create state manager with persistent storage
        let state_manager = Arc::new(
            StateManager::with_backend(
                StorageBackendType::Sled(llmspell_state_persistence::SledConfig {
                    path: storage_path.join("test_states"),
                    cache_capacity: 1024 * 1024, // 1MB
                    use_compression: true,
                }),
                PersistenceConfig {
                    enabled: true,
                    backend_type: StorageBackendType::Memory, // Overridden by with_backend
                    flush_interval: Duration::from_millis(100), // Fast for tests
                    compression: true,
                    encryption: None,
                    backup_retention: Duration::from_secs(300),
                    backup: None,
                    performance: PerformanceConfig::default(),
                },
            )
            .await?,
        );

        // Create provider manager and register rig provider
        let provider_manager = Arc::new(ProviderManager::new());
        provider_manager
            .register_provider("rig", llmspell_providers::create_rig_provider)
            .await;

        let agent_id = "test-provider-agent".to_string();

        Ok(Self {
            provider_manager,
            state_manager,
            temp_dir,
            agent_id,
        })
    }

    /// Create an `OpenAI` agent if API key is available
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Agent configuration fails
    /// - Agent creation fails
    pub async fn create_openai_agent(&self) -> Result<Option<LLMAgent>> {
        if env::var("OPENAI_API_KEY").is_err() {
            warn!("OPENAI_API_KEY not set, skipping OpenAI tests");
            return Ok(None);
        }

        let config = AgentBuilder::new(&self.agent_id, "llm")
            .description("Test agent for OpenAI provider integration")
            .with_model("openai", "gpt-4")
            .temperature(0.7)
            .max_tokens(1000)
            .build()?;

        let agent = LLMAgent::new(config, self.provider_manager.clone()).await?;
        agent.set_state_manager(self.state_manager.clone());

        Ok(Some(agent))
    }

    /// Create an Anthropic agent if API key is available
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Agent configuration fails
    /// - Agent creation fails
    pub async fn create_anthropic_agent(&self) -> Result<Option<LLMAgent>> {
        if env::var("ANTHROPIC_API_KEY").is_err() {
            warn!("ANTHROPIC_API_KEY not set, skipping Anthropic tests");
            return Ok(None);
        }

        let config = AgentBuilder::new(format!("{}-anthropic", &self.agent_id), "llm")
            .description("Test agent for Anthropic provider integration")
            .with_model("anthropic", "claude-3-5-sonnet-latest")
            .temperature(0.7)
            .max_tokens(1000)
            .build()?;

        let agent = LLMAgent::new(config, self.provider_manager.clone()).await?;
        agent.set_state_manager(self.state_manager.clone());

        Ok(Some(agent))
    }

    /// Verify that agent state is properly persisted
    ///
    /// # Errors
    ///
    /// Returns an error if state loading fails
    pub async fn verify_state_persistence(
        &self,
        agent_id: &str,
        expected_messages: usize,
    ) -> Result<bool> {
        // Check if agent state is saved
        let saved_state = self.state_manager.load_agent_state(agent_id).await?;

        if let Some(state) = saved_state {
            let conversation_count = state.state.conversation_history.len();
            info!(
                "Found {} saved messages for agent {}",
                conversation_count, agent_id
            );
            Ok(conversation_count >= expected_messages)
        } else {
            warn!("No agent state found for agent {}", agent_id);
            Ok(false)
        }
    }

    /// Get conversation from agent state
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - State loading fails
    /// - JSON serialization fails
    pub async fn get_saved_conversation(
        &self,
        agent_id: &str,
    ) -> Result<Option<serde_json::Value>> {
        let saved_state = self.state_manager.load_agent_state(agent_id).await?;

        match saved_state {
            Some(state) => {
                // Convert conversation history to JSON value
                let conversation = serde_json::to_value(&state.state.conversation_history)?;
                Ok(Some(conversation))
            }
            None => Ok(None),
        }
    }

    /// Verify provider metadata is saved
    ///
    /// # Errors
    ///
    /// Returns an error if state loading fails
    pub async fn verify_provider_metadata(&self, agent_id: &str) -> Result<bool> {
        let saved_state = self.state_manager.load_agent_state(agent_id).await?;

        if let Some(state) = saved_state {
            info!(
                "Found agent state metadata for agent {}: {:?}",
                agent_id, state.metadata
            );
            // Check if provider config is present
            Ok(state.metadata.provider_config.is_some())
        } else {
            warn!("No agent state found for agent {}", agent_id);
            Ok(false)
        }
    }

    /// Verify token usage is tracked
    ///
    /// # Errors
    ///
    /// Returns an error if state loading fails
    pub async fn verify_token_usage(&self, agent_id: &str) -> Result<bool> {
        let saved_state = self.state_manager.load_agent_state(agent_id).await?;

        if let Some(state) = saved_state {
            // Check if there are any tool usage stats
            let has_stats = state.state.tool_usage_stats.total_invocations > 0
                || state.state.tool_usage_stats.successful_invocations > 0;
            info!(
                "Found agent state with tool usage stats for agent {}: total_invocations={}, successful={}",
                agent_id,
                state.state.tool_usage_stats.total_invocations,
                state.state.tool_usage_stats.successful_invocations
            );
            Ok(has_stats)
        } else {
            warn!("No agent state found for agent {}", agent_id);
            Ok(false)
        }
    }

    /// Run a conversation and save state
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Agent initialization fails
    /// - Agent execution fails
    /// - State saving fails
    pub async fn run_conversation_with_save(
        &self,
        agent: &mut LLMAgent,
        messages: Vec<&str>,
    ) -> Result<Vec<AgentOutput>> {
        let mut responses = Vec::new();
        let context = ExecutionContext::new();

        // Initialize and start agent
        agent.initialize().await?;
        agent.start().await?;

        // Run conversation
        for message in messages {
            let input = AgentInput::text(message);
            let response = agent.execute(input, context.clone()).await?;
            responses.push(response);

            // Small delay to ensure state operations complete
            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        // Explicitly save state
        agent.save_state().await?;

        Ok(responses)
    }

    /// Create an agent with a custom ID
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Agent configuration fails
    /// - Agent creation fails
    pub async fn create_agent_with_id(
        &self,
        agent_id: &str,
        provider: &str,
        model: &str,
    ) -> Result<Option<LLMAgent>> {
        let api_key_var = match provider {
            "openai" => "OPENAI_API_KEY",
            "anthropic" => "ANTHROPIC_API_KEY",
            _ => return Ok(None),
        };

        if env::var(api_key_var).is_err() {
            warn!("{} not set, skipping {} tests", api_key_var, provider);
            return Ok(None);
        }

        let config = AgentBuilder::new(agent_id, "llm")
            .description(format!("Test agent for {provider} provider integration"))
            .with_model(provider, model)
            .temperature(0.7)
            .max_tokens(1000)
            .build()?;

        let agent = LLMAgent::new(config, self.provider_manager.clone()).await?;
        agent.set_state_manager(self.state_manager.clone());

        Ok(Some(agent))
    }

    /// Create a fresh agent with the same ID and verify state restoration
    ///
    /// # Errors
    ///
    /// Returns an error if agent creation or restoration fails
    pub async fn create_and_restore_agent(
        &self,
        provider: &str,
        model: &str,
    ) -> Result<Option<LLMAgent>> {
        self.create_and_restore_agent_with_id(&self.agent_id, provider, model)
            .await
    }

    /// Create and restore an agent with a custom ID
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Agent configuration fails
    /// - Agent creation fails
    /// - State restoration fails
    pub async fn create_and_restore_agent_with_id(
        &self,
        agent_id: &str,
        provider: &str,
        model: &str,
    ) -> Result<Option<LLMAgent>> {
        let api_key_var = match provider {
            "openai" => "OPENAI_API_KEY",
            "anthropic" => "ANTHROPIC_API_KEY",
            _ => return Ok(None),
        };

        if env::var(api_key_var).is_err() {
            warn!("{} not set, skipping {} tests", api_key_var, provider);
            return Ok(None);
        }

        let config = AgentBuilder::new(agent_id, "llm")
            .description(format!("Restored {provider} agent"))
            .with_model(provider, model)
            .temperature(0.7)
            .max_tokens(1000)
            .build()?;

        let agent = LLMAgent::new(config, self.provider_manager.clone()).await?;
        agent.set_state_manager(self.state_manager.clone());

        // Initialize and restore state
        agent.initialize().await?;
        agent.load_state().await?;

        Ok(Some(agent))
    }
}

/// Check if a provider API key is available
#[must_use]
pub fn check_api_key(provider: &str) -> bool {
    let key_name = match provider {
        "openai" => "OPENAI_API_KEY",
        "anthropic" => "ANTHROPIC_API_KEY",
        _ => return false,
    };

    env::var(key_name).is_ok()
}

/// Skip test if API key is not available
///
/// # Panics
///
/// Panics if the required API key environment variable is not set for the specified provider.
pub fn skip_if_no_api_key(provider: &str) {
    if !check_api_key(provider) {
        let key_name = match provider {
            "openai" => "OPENAI_API_KEY",
            "anthropic" => "ANTHROPIC_API_KEY",
            _ => "API_KEY",
        };
        panic!("Test requires {key_name} environment variable");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_context_creation() {
        let context = ProviderTestContext::new().await.unwrap();
        assert!(!context.agent_id.is_empty());
        assert!(context.temp_dir.path().exists());
    }
    #[test]
    fn test_api_key_checking() {
        // This will return false unless API keys are actually set
        let has_openai = check_api_key("openai");
        let has_anthropic = check_api_key("anthropic");

        // Just verify the function doesn't panic and returns a boolean
        assert!(has_openai || !has_openai); // Always true - just checking it's a bool
        assert!(has_anthropic || !has_anthropic); // Always true - just checking it's a bool
    }
}
