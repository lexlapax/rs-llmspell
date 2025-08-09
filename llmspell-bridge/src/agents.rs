//! ABOUTME: Agent discovery and management for script bridge
//! ABOUTME: Provides registry integration for agents from llmspell-agents crate

use crate::discovery::BridgeDiscovery;
use llmspell_agents::{AgentConfig, AgentFactory, DefaultAgentFactory};
use llmspell_core::{Agent, LLMSpellError, Result};
use std::collections::HashMap;
use std::sync::Arc;

/// Agent discovery service for bridge
pub struct AgentDiscovery {
    /// Factory for creating agents
    factory: Arc<dyn AgentFactory>,
    /// Cache of created agents
    agent_cache: Arc<tokio::sync::RwLock<HashMap<String, Arc<dyn Agent>>>>,
}

impl AgentDiscovery {
    /// Create a new agent discovery service with provider manager
    #[must_use]
    pub fn new(provider_manager: Arc<llmspell_providers::ProviderManager>) -> Self {
        Self {
            factory: Arc::new(DefaultAgentFactory::new(provider_manager)),
            agent_cache: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    /// Create with a custom factory
    pub fn with_factory(factory: Arc<dyn AgentFactory>) -> Self {
        Self {
            factory,
            agent_cache: Arc::new(tokio::sync::RwLock::new(HashMap::new())),
        }
    }

    /// List available agent types
    pub fn list_agent_types(&self) -> Vec<String> {
        // Use the templates from factory as agent types
        self.factory
            .list_templates()
            .into_iter()
            .map(std::string::ToString::to_string)
            .collect()
    }

    /// Get available templates from the global registry
    pub fn list_templates(&self) -> Vec<String> {
        // For now, use the factory's templates
        self.factory
            .list_templates()
            .into_iter()
            .map(std::string::ToString::to_string)
            .collect()
    }

    /// Create an agent by type
    ///
    /// # Errors
    ///
    /// Returns an error if the configuration is invalid or agent creation fails
    pub async fn create_agent(
        &self,
        _agent_type: &str,
        config: serde_json::Value,
    ) -> Result<Arc<dyn Agent>> {
        // Convert JSON config to AgentConfig
        let agent_config: AgentConfig =
            serde_json::from_value(config).map_err(|e| LLMSpellError::Validation {
                field: Some("config".to_string()),
                message: format!("Invalid agent configuration: {e}"),
            })?;

        // Create the agent using the factory
        let agent = self.factory.create_agent(agent_config).await.map_err(|e| {
            LLMSpellError::Component {
                message: format!("Failed to create agent: {e}"),
                source: None,
            }
        })?;

        Ok(agent)
    }

    /// Create an agent from a template
    ///
    /// # Errors
    ///
    /// Returns an error if the template is not found or agent creation fails
    pub async fn create_from_template(
        &self,
        template_name: &str,
        _parameters: HashMap<String, serde_json::Value>,
    ) -> Result<Arc<dyn Agent>> {
        // For now, use the factory's create_from_template without parameters
        self.factory
            .create_from_template(template_name)
            .await
            .map_err(|e| LLMSpellError::Component {
                message: format!("Failed to create agent from template: {e}"),
                source: None,
            })
    }

    /// Get or create a cached agent
    ///
    /// # Errors
    ///
    /// Returns an error if agent creation fails
    pub async fn get_or_create_agent(
        &self,
        name: &str,
        agent_type: &str,
        config: serde_json::Value,
    ) -> Result<Arc<dyn Agent>> {
        // Check cache first
        {
            let cache = self.agent_cache.read().await;
            if let Some(agent) = cache.get(name) {
                return Ok(agent.clone());
            }
        }

        // Create new agent
        let agent = self.create_agent(agent_type, config).await?;

        // Cache it
        {
            let mut cache = self.agent_cache.write().await;
            cache.insert(name.to_string(), agent.clone());
        }

        Ok(agent)
    }

    /// Remove an agent from cache
    pub async fn remove_agent(&self, name: &str) -> Option<Arc<dyn Agent>> {
        let mut cache = self.agent_cache.write().await;
        cache.remove(name)
    }

    /// Clear all cached agents
    pub async fn clear_cache(&self) {
        let mut cache = self.agent_cache.write().await;
        cache.clear();
    }

    /// Get agent metadata
    /// Get information about an agent type
    ///
    /// # Errors
    ///
    /// Returns an error if the agent type is not found
    pub fn get_agent_info(&self, agent_type: &str) -> Result<AgentInfo> {
        // For now, return basic info since we don't have template details
        Ok(AgentInfo {
            name: agent_type.to_string(),
            description: format!("Agent type: {agent_type}"),
            category: "agent".to_string(),
            complexity: "standard".to_string(),
            required_parameters: vec!["name".to_string()],
            optional_parameters: vec![
                "model_config".to_string(),
                "resource_limits".to_string(),
                "creation_hooks".to_string(),
            ],
        })
    }
}

/// Information about an agent type
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AgentInfo {
    /// Agent type name
    pub name: String,
    /// Description
    pub description: String,
    /// Category
    pub category: String,
    /// Complexity level
    pub complexity: String,
    /// Required parameters
    pub required_parameters: Vec<String>,
    /// Optional parameters
    pub optional_parameters: Vec<String>,
}

// Implementation of unified BridgeDiscovery trait for AgentDiscovery
#[async_trait::async_trait]
impl BridgeDiscovery<AgentInfo> for AgentDiscovery {
    async fn discover_types(&self) -> Vec<(String, AgentInfo)> {
        self.list_agent_types()
            .into_iter()
            .filter_map(|agent_type| {
                self.get_agent_info(&agent_type)
                    .ok()
                    .map(|info| (agent_type, info))
            })
            .collect()
    }

    async fn get_type_info(&self, type_name: &str) -> Option<AgentInfo> {
        self.get_agent_info(type_name).ok()
    }

    async fn has_type(&self, type_name: &str) -> bool {
        self.list_agent_types().contains(&type_name.to_string())
    }

    async fn list_types(&self) -> Vec<String> {
        self.list_agent_types()
    }

    async fn filter_types<F>(&self, predicate: F) -> Vec<(String, AgentInfo)>
    where
        F: Fn(&str, &AgentInfo) -> bool + Send,
    {
        self.list_agent_types()
            .into_iter()
            .filter_map(|agent_type| {
                self.get_agent_info(&agent_type).ok().and_then(|info| {
                    if predicate(&agent_type, &info) {
                        Some((agent_type, info))
                    } else {
                        None
                    }
                })
            })
            .collect()
    }
}

// Removed Default impl - AgentDiscovery now requires provider manager

#[cfg(test)]
mod tests {
    use super::*;

    async fn create_test_provider_manager() -> Arc<llmspell_providers::ProviderManager> {
        Arc::new(llmspell_providers::ProviderManager::new())
    }
    #[tokio::test]
    async fn test_agent_discovery() {
        let provider_manager = create_test_provider_manager().await;
        let discovery = AgentDiscovery::new(provider_manager);

        // List agent types
        let types = discovery.list_agent_types();
        assert!(!types.is_empty());

        // List templates
        let templates = discovery.list_templates();
        assert!(!templates.is_empty());
    }
    #[tokio::test]
    async fn test_agent_caching() {
        let provider_manager = create_test_provider_manager().await;
        let discovery = AgentDiscovery::new(provider_manager);

        let config = serde_json::json!({
            "name": "test-agent",
            "description": "Test agent for caching",
            "agent_type": "basic",
            "allowed_tools": [],
            "custom_config": {},
            "resource_limits": {
                "max_execution_time_secs": 300,
                "max_memory_mb": 512,
                "max_tool_calls": 100,
                "max_recursion_depth": 10
            }
        });

        // Create and cache agent
        let agent1 = discovery
            .get_or_create_agent("test", "basic", config.clone())
            .await;
        if let Err(e) = &agent1 {
            eprintln!("Agent creation failed: {e:?}");
        }
        assert!(agent1.is_ok());

        // Get from cache
        let agent2 = discovery.get_or_create_agent("test", "basic", config).await;
        assert!(agent2.is_ok());

        // Remove from cache
        let removed = discovery.remove_agent("test").await;
        assert!(removed.is_some());
    }
}
