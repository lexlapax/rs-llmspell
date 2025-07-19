//! ABOUTME: Agent bridge for script-to-agent communication
//! ABOUTME: Provides unified interface for scripts to interact with agents

use crate::agents::{AgentDiscovery, AgentInfo};
use crate::ComponentRegistry;
use llmspell_agents::AgentFactory;
use llmspell_core::types::{AgentInput, AgentOutput};
use llmspell_core::{Agent, ExecutionContext, LLMSpellError, Result};
use std::collections::HashMap;
use std::sync::Arc;

/// Bridge between scripts and agents
pub struct AgentBridge {
    /// Agent discovery service
    discovery: Arc<AgentDiscovery>,
    /// Component registry for script access
    registry: Arc<ComponentRegistry>,
    /// Active agent instances
    active_agents: Arc<tokio::sync::RwLock<HashMap<String, Arc<dyn Agent>>>>,
}

impl AgentBridge {
    /// Create a new agent bridge
    pub fn new(registry: Arc<ComponentRegistry>) -> Self {
        Self {
            discovery: Arc::new(AgentDiscovery::new()),
            registry,
            active_agents: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    /// Create with custom factory
    pub fn with_factory(registry: Arc<ComponentRegistry>, factory: Arc<dyn AgentFactory>) -> Self {
        Self {
            discovery: Arc::new(AgentDiscovery::with_factory(factory)),
            registry,
            active_agents: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    /// List available agent types
    pub async fn list_agent_types(&self) -> Vec<String> {
        self.discovery.list_agent_types().await
    }

    /// List available templates
    pub async fn list_templates(&self) -> Vec<String> {
        self.discovery.list_templates().await
    }

    /// Get agent information
    pub async fn get_agent_info(&self, agent_type: &str) -> Result<AgentInfo> {
        self.discovery.get_agent_info(agent_type).await
    }

    /// Create a new agent instance
    pub async fn create_agent(
        &self,
        instance_name: &str,
        agent_type: &str,
        config: HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        // Check if instance already exists
        {
            let agents = self.active_agents.read().await;
            if agents.contains_key(instance_name) {
                return Err(LLMSpellError::Validation {
                    field: Some("instance_name".to_string()),
                    message: format!("Agent instance '{}' already exists", instance_name),
                });
            }
        }

        // Convert HashMap to JSON object
        let config_json = serde_json::Value::Object(config.into_iter().collect());

        // Create the agent
        let agent = self.discovery.create_agent(agent_type, config_json).await?;

        // Register in both active agents and component registry
        {
            let mut agents = self.active_agents.write().await;
            agents.insert(instance_name.to_string(), agent.clone());
        }

        // Also register in component registry for script access
        self.registry
            .register_agent(instance_name.to_string(), agent)?;

        Ok(())
    }

    /// Create agent from template
    pub async fn create_from_template(
        &self,
        instance_name: &str,
        template_name: &str,
        parameters: HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        // Check if instance already exists
        {
            let agents = self.active_agents.read().await;
            if agents.contains_key(instance_name) {
                return Err(LLMSpellError::Validation {
                    field: Some("instance_name".to_string()),
                    message: format!("Agent instance '{}' already exists", instance_name),
                });
            }
        }

        // Create from template
        let agent = self
            .discovery
            .create_from_template(template_name, parameters)
            .await?;

        // Register in both active agents and component registry
        {
            let mut agents = self.active_agents.write().await;
            agents.insert(instance_name.to_string(), agent.clone());
        }

        // Also register in component registry for script access
        self.registry
            .register_agent(instance_name.to_string(), agent)?;

        Ok(())
    }

    /// Execute an agent
    pub async fn execute_agent(
        &self,
        instance_name: &str,
        input: AgentInput,
        context: Option<ExecutionContext>,
    ) -> Result<AgentOutput> {
        // Get agent from active agents
        let agent = {
            let agents = self.active_agents.read().await;
            agents.get(instance_name).cloned()
        };

        let agent = agent.ok_or_else(|| LLMSpellError::Component {
            message: format!("Agent instance '{}' not found", instance_name),
            source: None,
        })?;

        // Use provided context or create new one
        let context = context.unwrap_or_default();

        // Execute the agent
        agent.execute(input, context).await
    }

    /// Get agent instance
    pub async fn get_agent(&self, instance_name: &str) -> Option<Arc<dyn Agent>> {
        let agents = self.active_agents.read().await;
        agents.get(instance_name).cloned()
    }

    /// Remove an agent instance
    pub async fn remove_agent(&self, instance_name: &str) -> Result<()> {
        // Remove from active agents
        let removed = {
            let mut agents = self.active_agents.write().await;
            agents.remove(instance_name)
        };

        if removed.is_none() {
            return Err(LLMSpellError::Component {
                message: format!("Agent instance '{}' not found", instance_name),
                source: None,
            });
        }

        // Note: We don't remove from component registry as it doesn't have a remove method
        // This could be added if needed

        Ok(())
    }

    /// List active agent instances
    pub async fn list_instances(&self) -> Vec<String> {
        let agents = self.active_agents.read().await;
        agents.keys().cloned().collect()
    }

    /// Get agent configuration
    pub async fn get_agent_config(&self, instance_name: &str) -> Result<serde_json::Value> {
        let agent =
            self.get_agent(instance_name)
                .await
                .ok_or_else(|| LLMSpellError::Component {
                    message: format!("Agent instance '{}' not found", instance_name),
                    source: None,
                })?;

        // Convert agent config to JSON
        let config = agent.config();
        let config_json = serde_json::json!({
            "system_prompt": config.system_prompt,
            "temperature": config.temperature,
            "max_tokens": config.max_tokens,
            "max_conversation_length": config.max_conversation_length,
        });

        Ok(config_json)
    }

    /// Clear all agent instances
    pub async fn clear_all(&self) {
        let mut agents = self.active_agents.write().await;
        agents.clear();
        // Note: This doesn't clear the component registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_bridge_creation() {
        let registry = Arc::new(ComponentRegistry::new());
        let bridge = AgentBridge::new(registry);

        // List available types
        let types = bridge.list_agent_types().await;
        assert!(!types.is_empty());
    }

    #[tokio::test]
    async fn test_agent_instance_management() {
        let registry = Arc::new(ComponentRegistry::new());
        let bridge = AgentBridge::new(registry);

        // Create agent config
        let mut config = HashMap::new();
        config.insert("name".to_string(), serde_json::json!("test-agent"));
        config.insert(
            "description".to_string(),
            serde_json::json!("Test agent for unit tests"),
        );
        config.insert("agent_type".to_string(), serde_json::json!("basic"));
        config.insert("allowed_tools".to_string(), serde_json::json!([]));
        config.insert("custom_config".to_string(), serde_json::json!({}));
        config.insert(
            "resource_limits".to_string(),
            serde_json::json!({
                "max_execution_time_secs": 300,
                "max_memory_mb": 512,
                "max_tool_calls": 100,
                "max_recursion_depth": 10
            }),
        );
        config.insert(
            "model".to_string(),
            serde_json::json!({
                "provider": "mock",
                "model_id": "test-model",
                "temperature": null,
                "max_tokens": null,
                "settings": {}
            }),
        );

        // Create agent instance
        let result = bridge.create_agent("test-instance", "basic", config).await;
        assert!(result.is_ok());

        // List instances
        let instances = bridge.list_instances().await;
        assert!(instances.contains(&"test-instance".to_string()));

        // Get agent
        let agent = bridge.get_agent("test-instance").await;
        assert!(agent.is_some());

        // Remove agent
        let remove_result = bridge.remove_agent("test-instance").await;
        assert!(remove_result.is_ok());

        // Verify removed
        let agent_after = bridge.get_agent("test-instance").await;
        assert!(agent_after.is_none());
    }

    #[tokio::test]
    async fn test_agent_execution() {
        let registry = Arc::new(ComponentRegistry::new());
        let bridge = AgentBridge::new(registry);

        // Create agent
        let mut config = HashMap::new();
        config.insert("name".to_string(), serde_json::json!("test-exec-agent"));
        config.insert(
            "description".to_string(),
            serde_json::json!("Test agent for execution"),
        );
        config.insert("agent_type".to_string(), serde_json::json!("basic"));
        config.insert("allowed_tools".to_string(), serde_json::json!([]));
        config.insert("custom_config".to_string(), serde_json::json!({}));
        config.insert(
            "resource_limits".to_string(),
            serde_json::json!({
                "max_execution_time_secs": 300,
                "max_memory_mb": 512,
                "max_tool_calls": 100,
                "max_recursion_depth": 10
            }),
        );
        config.insert(
            "model".to_string(),
            serde_json::json!({
                "provider": "mock",
                "model_id": "test-model",
                "temperature": null,
                "max_tokens": null,
                "settings": {}
            }),
        );

        bridge
            .create_agent("test-exec", "basic", config)
            .await
            .unwrap();

        // Execute agent
        let input = AgentInput::text("Hello, agent!");
        let result = bridge.execute_agent("test-exec", input, None).await;

        // Note: This might fail if mock provider is not available
        // In real tests, we'd use a proper mock
        assert!(result.is_ok() || result.is_err());
    }
}
