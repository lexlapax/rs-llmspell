//! ABOUTME: Agent factory system for creating and configuring agents
//! ABOUTME: Provides flexible agent creation with builder pattern and dependency injection

use anyhow::Result;
use async_trait::async_trait;
use llmspell_core::traits::agent::Agent;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

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

/// Factory for creating agent instances
#[async_trait]
pub trait AgentFactory: Send + Sync {
    /// Create an agent from configuration
    async fn create_agent(&self, config: AgentConfig) -> Result<Arc<dyn Agent>>;

    /// Create an agent from a template
    async fn create_from_template(&self, template_name: &str) -> Result<Arc<dyn Agent>>;

    /// List available agent templates
    fn list_templates(&self) -> Vec<&str>;

    /// Validate agent configuration
    fn validate_config(&self, config: &AgentConfig) -> Result<()>;
}

/// Default implementation of AgentFactory
pub struct DefaultAgentFactory {
    /// Template registry
    templates: std::collections::HashMap<String, AgentConfig>,

    /// Creation hooks
    creation_hooks: Vec<Arc<dyn CreationHook>>,
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
    /// Create a new agent factory
    pub fn new() -> Self {
        let mut templates = std::collections::HashMap::new();

        // Add default templates
        templates.insert(
            "basic".to_string(),
            AgentConfig {
                name: "basic-agent".to_string(),
                description: "Basic agent with no special capabilities".to_string(),
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
                description: "Agent that orchestrates tool execution".to_string(),
                agent_type: "tool-orchestrator".to_string(),
                model: None,
                allowed_tools: vec!["*".to_string()], // Access to all tools
                custom_config: serde_json::Map::new(),
                resource_limits: ResourceLimits::default(),
            },
        );

        Self {
            templates,
            creation_hooks: vec![],
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
}

impl Default for DefaultAgentFactory {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AgentFactory for DefaultAgentFactory {
    async fn create_agent(&self, config: AgentConfig) -> Result<Arc<dyn Agent>> {
        // Validate configuration
        self.validate_config(&config)?;

        // Run before hooks
        self.run_before_hooks(&config).await?;

        // Create agent based on type
        let agent: Arc<dyn Agent> = match config.agent_type.as_str() {
            "basic" => Arc::new(crate::agents::BasicAgent::new(config)?),
            "tool-orchestrator" => {
                // ToolOrchestratorAgent will be implemented in a future task
                anyhow::bail!("ToolOrchestratorAgent not yet implemented")
            }
            _ => anyhow::bail!("Unknown agent type: {}", config.agent_type),
        };

        // Run after hooks
        self.run_after_hooks(&agent).await?;

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
        self.templates.keys().map(|s| s.as_str()).collect()
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

    #[test]
    fn test_default_resource_limits() {
        let limits = ResourceLimits::default();
        assert_eq!(limits.max_execution_time_secs, 300);
        assert_eq!(limits.max_memory_mb, 512);
        assert_eq!(limits.max_tool_calls, 100);
        assert_eq!(limits.max_recursion_depth, 10);
    }

    #[test]
    fn test_factory_templates() {
        let factory = DefaultAgentFactory::new();
        let templates = factory.list_templates();
        assert!(templates.contains(&"basic"));
        assert!(templates.contains(&"tool-orchestrator"));
    }

    #[test]
    fn test_config_validation() {
        let factory = DefaultAgentFactory::new();

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
            name: "".to_string(),
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
                provider: "".to_string(),
                model_id: "gpt-4".to_string(),
                temperature: None,
                max_tokens: None,
                settings: serde_json::Map::new(),
            }),
            ..valid_config.clone()
        };
        assert!(factory.validate_config(&invalid_config).is_err());
    }

    #[tokio::test]
    async fn test_agent_creation() {
        let factory = DefaultAgentFactory::new();

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
        let factory = DefaultAgentFactory::new();

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
        let factory = DefaultAgentFactory::new();

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

        let mut factory = DefaultAgentFactory::new();
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

    #[test]
    fn test_add_custom_template() {
        let mut factory = DefaultAgentFactory::new();

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
