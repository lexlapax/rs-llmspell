//! ABOUTME: Agent factory system for creating and configuring agents
//! ABOUTME: Provides flexible agent creation with builder pattern and dependency injection

use crate::lifecycle::StateMachineConfig;
use anyhow::Result;
use async_trait::async_trait;
use llmspell_core::traits::agent::Agent;
use llmspell_hooks::HookRegistry;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::info;

/// Configuration for creating agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    /// Unique name for the agent
    pub name: String,

    /// Description of the agent's purpose
    pub description: String,

    /// Agent type identifier (e.g., "llm", "tool-orchestrator", "workflow")
    pub agent_type: String,

    /// Model configuration (if applicable)
    pub model: Option<ModelConfig>,

    /// Tool IDs this agent can use
    pub allowed_tools: Vec<String>,

    /// Custom configuration parameters
    pub custom_config: serde_json::Map<String, serde_json::Value>,

    /// Resource limits
    pub resource_limits: ResourceLimits,
}

impl AgentConfig {
    /// Create a new builder for `AgentConfig`
    pub fn builder(name: impl Into<String>) -> AgentConfigBuilder {
        AgentConfigBuilder::new(name)
    }
}

/// Model configuration for LLM-based agents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Provider name (e.g., "openai", "anthropic")
    pub provider: String,

    /// Model identifier
    pub model_id: String,

    /// Temperature setting
    pub temperature: Option<f32>,

    /// Maximum tokens
    pub max_tokens: Option<u32>,

    /// Additional provider-specific settings
    pub settings: serde_json::Map<String, serde_json::Value>,
}

/// Resource limits for agent execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum execution time in seconds
    pub max_execution_time_secs: u64,

    /// Maximum memory usage in MB
    pub max_memory_mb: u64,

    /// Maximum number of tool calls
    pub max_tool_calls: u32,

    /// Maximum recursion depth
    pub max_recursion_depth: u8,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_execution_time_secs: 300, // 5 minutes
            max_memory_mb: 512,
            max_tool_calls: 100,
            max_recursion_depth: 10,
        }
    }
}

/// Builder for `AgentConfig`
#[derive(Debug, Clone)]
pub struct AgentConfigBuilder {
    name: String,
    description: String,
    agent_type: String,
    model: Option<ModelConfig>,
    allowed_tools: Vec<String>,
    custom_config: serde_json::Map<String, serde_json::Value>,
    resource_limits: ResourceLimits,
}

impl AgentConfigBuilder {
    /// Create a new builder with required name
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            agent_type: String::from("basic"),
            model: None,
            allowed_tools: Vec::new(),
            custom_config: serde_json::Map::new(),
            resource_limits: ResourceLimits::default(),
        }
    }

    /// Set the agent's description
    #[must_use]
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Set the agent type
    #[must_use]
    pub fn agent_type(mut self, agent_type: impl Into<String>) -> Self {
        self.agent_type = agent_type.into();
        self
    }

    /// Set the model configuration
    #[must_use]
    pub fn model(mut self, model: ModelConfig) -> Self {
        self.model = Some(model);
        self
    }

    /// Add an allowed tool
    #[must_use]
    pub fn allow_tool(mut self, tool_id: impl Into<String>) -> Self {
        self.allowed_tools.push(tool_id.into());
        self
    }

    /// Set allowed tools
    #[must_use]
    pub fn allowed_tools(mut self, tools: Vec<String>) -> Self {
        self.allowed_tools = tools;
        self
    }

    /// Add a custom configuration parameter
    #[must_use]
    pub fn custom_param(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.custom_config.insert(key.into(), value);
        self
    }

    /// Set custom configuration
    #[must_use]
    pub fn custom_config(mut self, config: serde_json::Map<String, serde_json::Value>) -> Self {
        self.custom_config = config;
        self
    }

    /// Set resource limits
    #[must_use]
    pub const fn resource_limits(mut self, limits: ResourceLimits) -> Self {
        self.resource_limits = limits;
        self
    }

    /// Set maximum execution time
    #[must_use]
    pub const fn max_execution_time_secs(mut self, secs: u64) -> Self {
        self.resource_limits.max_execution_time_secs = secs;
        self
    }

    /// Set maximum memory usage
    #[must_use]
    pub const fn max_memory_mb(mut self, mb: u64) -> Self {
        self.resource_limits.max_memory_mb = mb;
        self
    }

    /// Set maximum tool calls
    #[must_use]
    pub const fn max_tool_calls(mut self, calls: u32) -> Self {
        self.resource_limits.max_tool_calls = calls;
        self
    }

    /// Build the final `AgentConfig`
    #[must_use]
    pub fn build(self) -> AgentConfig {
        AgentConfig {
            name: self.name,
            description: self.description,
            agent_type: self.agent_type,
            model: self.model,
            allowed_tools: self.allowed_tools,
            custom_config: self.custom_config,
            resource_limits: self.resource_limits,
        }
    }
}

/// Factory for creating agent instances
#[async_trait]
pub trait AgentFactory: Send + Sync {
    /// Create an agent from configuration
    ///
    /// # Errors
    ///
    /// Returns an error if agent creation fails due to invalid configuration,
    /// resource constraints, dependency resolution failures, or system-level issues.
    async fn create_agent(&self, config: AgentConfig) -> Result<Arc<dyn Agent>>;

    /// Create an agent from a template
    ///
    /// # Errors
    ///
    /// Returns an error if the template is not found, template instantiation fails,
    /// or the resulting agent configuration is invalid.
    async fn create_from_template(&self, template_name: &str) -> Result<Arc<dyn Agent>>;

    /// List available agent templates
    fn list_templates(&self) -> Vec<&str>;

    /// Validate agent configuration
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration is invalid (e.g., missing required fields,
    /// invalid tool references, conflicting settings, or unsupported agent types).
    fn validate_config(&self, config: &AgentConfig) -> Result<()>;
}

/// Default implementation of `AgentFactory`
pub struct DefaultAgentFactory {
    /// Template registry
    templates: std::collections::HashMap<String, AgentConfig>,

    /// Creation hooks
    creation_hooks: Vec<Arc<dyn CreationHook>>,

    /// Provider manager for LLM agents
    provider_manager: Option<Arc<llmspell_providers::ProviderManager>>,

    /// Hook registry for state machine integration
    hook_registry: Option<Arc<HookRegistry>>,

    /// Default state machine configuration
    default_state_config: StateMachineConfig,
}

/// Hook that runs during agent creation
#[async_trait]
pub trait CreationHook: Send + Sync {
    /// Called before agent creation
    async fn before_create(&self, config: &AgentConfig) -> Result<()>;

    /// Called after agent creation
    async fn after_create(&self, agent: &Arc<dyn Agent>) -> Result<()>;
}

impl DefaultAgentFactory {
    /// Create a new agent factory with provider manager
    #[must_use]
    pub fn new(provider_manager: Arc<llmspell_providers::ProviderManager>) -> Self {
        let mut templates = std::collections::HashMap::new();

        // LLM agent is now the default template
        templates.insert(
            "llm".to_string(),
            AgentConfig {
                name: "llm-agent".to_string(),
                description: "LLM-powered agent for intelligent interactions".to_string(),
                agent_type: "llm".to_string(),
                model: Some(ModelConfig {
                    provider: String::new(),              // Will be set from model_id
                    model_id: "openai/gpt-4".to_string(), // Default model
                    temperature: Some(0.7),
                    max_tokens: Some(2000),
                    settings: serde_json::Map::new(),
                }),
                allowed_tools: vec![],
                custom_config: serde_json::Map::new(),
                resource_limits: ResourceLimits::default(),
            },
        );

        // Basic agent for testing only
        templates.insert(
            "basic".to_string(),
            AgentConfig {
                name: "basic-agent".to_string(),
                description: "Basic echo agent for testing".to_string(),
                agent_type: "basic".to_string(),
                model: None,
                allowed_tools: vec![],
                custom_config: serde_json::Map::new(),
                resource_limits: ResourceLimits::default(),
            },
        );

        templates.insert(
            "tool-orchestrator".to_string(),
            AgentConfig {
                name: "tool-orchestrator".to_string(),
                description: "LLM agent that orchestrates tool execution".to_string(),
                agent_type: "llm".to_string(), // Changed to LLM type
                model: Some(ModelConfig {
                    provider: String::new(),
                    model_id: "openai/gpt-4".to_string(),
                    temperature: Some(0.3), // Lower temperature for tool use
                    max_tokens: Some(2000),
                    settings: serde_json::Map::new(),
                }),
                allowed_tools: vec!["*".to_string()], // Access to all tools
                custom_config: serde_json::Map::new(),
                resource_limits: ResourceLimits::default(),
            },
        );

        Self {
            templates,
            creation_hooks: vec![],
            provider_manager: Some(provider_manager),
            hook_registry: None,
            default_state_config: StateMachineConfig::default(),
        }
    }

    /// Add a creation hook
    pub fn add_hook(&mut self, hook: Arc<dyn CreationHook>) {
        self.creation_hooks.push(hook);
    }

    /// Add a custom template
    pub fn add_template(&mut self, name: String, config: AgentConfig) {
        self.templates.insert(name, config);
    }

    /// Set hook registry for state machine integration
    #[must_use]
    pub fn with_hook_registry(mut self, hook_registry: Arc<HookRegistry>) -> Self {
        self.hook_registry = Some(hook_registry);
        self
    }

    /// Configure default state machine settings
    #[must_use]
    pub const fn with_state_config(mut self, state_config: StateMachineConfig) -> Self {
        self.default_state_config = state_config;
        self
    }

    /// Enable hooks and circuit breaker by default
    #[must_use]
    pub fn with_enhanced_lifecycle(mut self) -> Self {
        self.default_state_config = StateMachineConfig {
            feature_flags: crate::lifecycle::state_machine::StateMachineFeatureFlags {
                enable_logging: true,
                enable_hooks: true,
                enable_circuit_breaker: true,
                ..Default::default()
            },
            ..StateMachineConfig::default()
        };
        self
    }

    /// Run creation hooks before creating agent
    async fn run_before_hooks(&self, config: &AgentConfig) -> Result<()> {
        for hook in &self.creation_hooks {
            hook.before_create(config).await?;
        }
        Ok(())
    }

    /// Run creation hooks after creating agent
    async fn run_after_hooks(&self, agent: &Arc<dyn Agent>) -> Result<()> {
        for hook in &self.creation_hooks {
            hook.after_create(agent).await?;
        }
        Ok(())
    }

    /// Initialize agent lifecycle by calling initialize on the concrete type
    fn initialize_agent_lifecycle(agent_type: &str, agent_name: &str) {
        info!(
            "Agent '{}' of type '{}' created with lifecycle management",
            agent_name, agent_type
        );
        // Note: Individual agents are responsible for calling initialize() when they're ready
        // The factory creates them with state machines but doesn't auto-initialize
        // This allows for more controlled startup sequences
    }
}

// Removed Default impl - factory now requires provider manager

#[async_trait]
impl AgentFactory for DefaultAgentFactory {
    async fn create_agent(&self, config: AgentConfig) -> Result<Arc<dyn Agent>> {
        // Validate configuration
        self.validate_config(&config)?;

        // Run before hooks
        self.run_before_hooks(&config).await?;

        // Capture values we need after moving config
        let agent_type = config.agent_type.clone();
        let agent_name = config.name.clone();

        // Create agent based on type
        let agent: Arc<dyn Agent> = match config.agent_type.as_str() {
            "llm" => {
                // LLM agents require provider manager
                let provider_manager = self
                    .provider_manager
                    .as_ref()
                    .ok_or_else(|| anyhow::anyhow!("Provider manager required for LLM agents"))?;
                Arc::new(crate::agents::LLMAgent::new(config, provider_manager.clone()).await?)
            }
            "basic" => {
                // Basic agent for testing only
                Arc::new(crate::agents::BasicAgent::new(config)?)
            }
            _ => {
                return Err(anyhow::anyhow!("Unknown agent type: {}", config.agent_type));
            }
        };

        // Run after hooks
        self.run_after_hooks(&agent).await?;

        // Initialize agent lifecycle
        Self::initialize_agent_lifecycle(&agent_type, &agent_name);

        Ok(agent)
    }

    async fn create_from_template(&self, template_name: &str) -> Result<Arc<dyn Agent>> {
        let config = self
            .templates
            .get(template_name)
            .ok_or_else(|| anyhow::anyhow!("Template not found: {}", template_name))?
            .clone();

        self.create_agent(config).await
    }

    fn list_templates(&self) -> Vec<&str> {
        self.templates
            .keys()
            .map(std::string::String::as_str)
            .collect()
    }

    fn validate_config(&self, config: &AgentConfig) -> Result<()> {
        // Basic validation
        if config.name.is_empty() {
            anyhow::bail!("Agent name cannot be empty");
        }

        if config.resource_limits.max_execution_time_secs == 0 {
            anyhow::bail!("Max execution time must be greater than 0");
        }

        if config.resource_limits.max_memory_mb == 0 {
            anyhow::bail!("Max memory must be greater than 0");
        }

        // Validate model config if present
        if let Some(model) = &config.model {
            if model.provider.is_empty() {
                anyhow::bail!("Model provider cannot be empty");
            }
            if model.model_id.is_empty() {
                anyhow::bail!("Model ID cannot be empty");
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_providers::ProviderManager;
    #[test]
    fn test_default_resource_limits() {
        let limits = ResourceLimits::default();
        assert_eq!(limits.max_execution_time_secs, 300);
        assert_eq!(limits.max_memory_mb, 512);
        assert_eq!(limits.max_tool_calls, 100);
        assert_eq!(limits.max_recursion_depth, 10);
    }

    fn create_test_factory() -> DefaultAgentFactory {
        let provider_manager = Arc::new(ProviderManager::new());
        DefaultAgentFactory::new(provider_manager)
    }
    #[tokio::test]
    async fn test_factory_templates() {
        let factory = create_test_factory();
        let templates = factory.list_templates();
        assert!(templates.contains(&"basic"));
        assert!(templates.contains(&"llm"));
    }
    #[tokio::test]
    async fn test_config_validation() {
        let factory = create_test_factory();

        // Valid config
        let valid_config = AgentConfig {
            name: "test-agent".to_string(),
            description: "Test agent".to_string(),
            agent_type: "basic".to_string(),
            model: None,
            allowed_tools: vec![],
            custom_config: serde_json::Map::new(),
            resource_limits: ResourceLimits::default(),
        };
        assert!(factory.validate_config(&valid_config).is_ok());

        // Invalid config - empty name
        let invalid_config = AgentConfig {
            name: String::new(),
            ..valid_config.clone()
        };
        assert!(factory.validate_config(&invalid_config).is_err());

        // Invalid config - zero execution time
        let invalid_config = AgentConfig {
            resource_limits: ResourceLimits {
                max_execution_time_secs: 0,
                ..ResourceLimits::default()
            },
            ..valid_config.clone()
        };
        assert!(factory.validate_config(&invalid_config).is_err());

        // Invalid config - zero memory
        let invalid_config = AgentConfig {
            resource_limits: ResourceLimits {
                max_memory_mb: 0,
                ..ResourceLimits::default()
            },
            ..valid_config.clone()
        };
        assert!(factory.validate_config(&invalid_config).is_err());

        // Invalid model config - empty provider
        let invalid_config = AgentConfig {
            model: Some(ModelConfig {
                provider: String::new(),
                model_id: "gpt-4".to_string(),
                temperature: None,
                max_tokens: None,
                settings: serde_json::Map::new(),
            }),
            ..valid_config
        };
        assert!(factory.validate_config(&invalid_config).is_err());
    }
    #[tokio::test]
    async fn test_agent_creation() {
        let factory = create_test_factory();

        let config = AgentConfig {
            name: "test-basic".to_string(),
            description: "Test basic agent".to_string(),
            agent_type: "basic".to_string(),
            model: None,
            allowed_tools: vec![],
            custom_config: serde_json::Map::new(),
            resource_limits: ResourceLimits::default(),
        };

        let agent = factory.create_agent(config).await.unwrap();
        assert_eq!(agent.metadata().name, "test-basic");
    }
    #[tokio::test]
    async fn test_agent_creation_unknown_type() {
        let factory = create_test_factory();

        let config = AgentConfig {
            name: "test-unknown".to_string(),
            description: "Test unknown agent".to_string(),
            agent_type: "unknown-type".to_string(),
            model: None,
            allowed_tools: vec![],
            custom_config: serde_json::Map::new(),
            resource_limits: ResourceLimits::default(),
        };

        let result = factory.create_agent(config).await;
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert!(err.to_string().contains("Unknown agent type"));
    }
    #[tokio::test]
    async fn test_create_from_template() {
        let factory = create_test_factory();

        let agent = factory.create_from_template("basic").await.unwrap();
        assert_eq!(agent.metadata().name, "basic-agent");

        // Test non-existent template
        let result = factory.create_from_template("non-existent").await;
        assert!(result.is_err());
    }
    #[tokio::test]
    async fn test_hooks_execution() {
        use std::sync::atomic::{AtomicBool, Ordering};

        struct TestHook {
            before_called: Arc<AtomicBool>,
            after_called: Arc<AtomicBool>,
        }

        #[async_trait]
        impl CreationHook for TestHook {
            async fn before_create(&self, _config: &AgentConfig) -> Result<()> {
                self.before_called.store(true, Ordering::SeqCst);
                Ok(())
            }

            async fn after_create(&self, _agent: &Arc<dyn Agent>) -> Result<()> {
                self.after_called.store(true, Ordering::SeqCst);
                Ok(())
            }
        }

        let mut factory = create_test_factory();
        let before_called = Arc::new(AtomicBool::new(false));
        let after_called = Arc::new(AtomicBool::new(false));

        let hook = Arc::new(TestHook {
            before_called: before_called.clone(),
            after_called: after_called.clone(),
        });

        factory.add_hook(hook);

        let config = AgentConfig {
            name: "test-hooks".to_string(),
            description: "Test hooks".to_string(),
            agent_type: "basic".to_string(),
            model: None,
            allowed_tools: vec![],
            custom_config: serde_json::Map::new(),
            resource_limits: ResourceLimits::default(),
        };

        let _ = factory.create_agent(config).await.unwrap();

        assert!(before_called.load(Ordering::SeqCst));
        assert!(after_called.load(Ordering::SeqCst));
    }
    #[tokio::test]
    async fn test_add_custom_template() {
        let mut factory = create_test_factory();

        let custom_config = AgentConfig {
            name: "custom-agent".to_string(),
            description: "Custom agent template".to_string(),
            agent_type: "custom".to_string(),
            model: None,
            allowed_tools: vec!["tool1".to_string(), "tool2".to_string()],
            custom_config: serde_json::Map::new(),
            resource_limits: ResourceLimits::default(),
        };

        factory.add_template("custom".to_string(), custom_config);

        let templates = factory.list_templates();
        assert!(templates.contains(&"custom"));
    }
}
