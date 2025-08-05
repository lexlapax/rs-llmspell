//! ABOUTME: Factory registry for managing multiple agent factories
//! ABOUTME: Allows registration and discovery of different factory implementations

use crate::factory::{AgentConfig, AgentFactory};
use anyhow::Result;
use async_trait::async_trait;
use llmspell_core::traits::agent::Agent;
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;

/// Registry for agent factories
pub struct FactoryRegistry {
    factories: Arc<RwLock<HashMap<String, Arc<dyn AgentFactory>>>>,
    default_factory: Arc<RwLock<Option<String>>>,
}

impl FactoryRegistry {
    /// Create a new factory registry
    #[must_use]
    pub fn new() -> Self {
        Self {
            factories: Arc::new(RwLock::new(HashMap::new())),
            default_factory: Arc::new(RwLock::new(None)),
        }
    }

    /// Register a factory with a given name
    ///
    /// # Errors
    ///
    /// Returns an error if a factory with the same name is already registered
    pub async fn register_factory(
        &self,
        name: String,
        factory: Arc<dyn AgentFactory>,
    ) -> Result<()> {
        let mut factories = self.factories.write().await;
        if factories.contains_key(&name) {
            anyhow::bail!("Factory '{}' is already registered", name);
        }
        factories.insert(name.clone(), factory);

        // Set as default if it's the first factory
        let mut default = self.default_factory.write().await;
        if default.is_none() {
            *default = Some(name);
        }

        Ok(())
    }

    /// Get a factory by name
    pub async fn get_factory(&self, name: &str) -> Option<Arc<dyn AgentFactory>> {
        let factories = self.factories.read().await;
        factories.get(name).cloned()
    }

    /// Get the default factory
    pub async fn get_default_factory(&self) -> Option<Arc<dyn AgentFactory>> {
        let default = self.default_factory.read().await;
        if let Some(name) = default.as_ref() {
            self.get_factory(name).await
        } else {
            None
        }
    }

    /// Set the default factory
    ///
    /// # Errors
    ///
    /// Returns an error if the factory is not registered
    pub async fn set_default_factory(&self, name: String) -> Result<()> {
        let factories = self.factories.read().await;
        if !factories.contains_key(&name) {
            anyhow::bail!("Factory '{}' is not registered", name);
        }

        let mut default = self.default_factory.write().await;
        *default = Some(name);
        Ok(())
    }

    /// List all registered factory names
    pub async fn list_factories(&self) -> Vec<String> {
        let factories = self.factories.read().await;
        factories.keys().cloned().collect()
    }

    /// Create an agent using a specific factory
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Factory is not found
    /// - Agent creation fails
    pub async fn create_agent_with(
        &self,
        factory_name: &str,
        config: AgentConfig,
    ) -> Result<Arc<dyn Agent>> {
        let factory = self
            .get_factory(factory_name)
            .await
            .ok_or_else(|| anyhow::anyhow!("Factory '{}' not found", factory_name))?;

        factory.create_agent(config).await
    }

    /// Create an agent using the default factory
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No default factory is set
    /// - Agent creation fails
    pub async fn create_agent(&self, config: AgentConfig) -> Result<Arc<dyn Agent>> {
        let factory = self
            .get_default_factory()
            .await
            .ok_or_else(|| anyhow::anyhow!("No default factory set"))?;

        factory.create_agent(config).await
    }

    /// Create an agent from a template using a specific factory
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Factory is not found
    /// - Template creation fails
    pub async fn create_from_template_with(
        &self,
        factory_name: &str,
        template_name: &str,
    ) -> Result<Arc<dyn Agent>> {
        let factory = self
            .get_factory(factory_name)
            .await
            .ok_or_else(|| anyhow::anyhow!("Factory '{}' not found", factory_name))?;

        factory.create_from_template(template_name).await
    }

    /// Create an agent from a template using the default factory
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No default factory is set
    /// - Template creation fails
    pub async fn create_from_template(&self, template_name: &str) -> Result<Arc<dyn Agent>> {
        let factory = self
            .get_default_factory()
            .await
            .ok_or_else(|| anyhow::anyhow!("No default factory set"))?;

        factory.create_from_template(template_name).await
    }
}

impl Default for FactoryRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Global factory registry instance
static GLOBAL_REGISTRY: std::sync::LazyLock<FactoryRegistry> =
    std::sync::LazyLock::new(FactoryRegistry::new);

/// Get the global factory registry
#[must_use]
pub fn global_registry() -> &'static FactoryRegistry {
    &GLOBAL_REGISTRY
}

/// Type alias for configuration customizer functions
type ConfigCustomizer = Box<dyn Fn(&mut AgentConfig) + Send + Sync>;

/// Specialized factory for creating agents with custom behavior
pub struct CustomAgentFactory {
    base_factory: Arc<dyn AgentFactory>,
    customizers: Vec<ConfigCustomizer>,
}

impl CustomAgentFactory {
    /// Create a new custom factory wrapping a base factory
    pub fn new(base_factory: Arc<dyn AgentFactory>) -> Self {
        Self {
            base_factory,
            customizers: vec![],
        }
    }

    /// Add a customizer function
    pub fn with_customizer<F>(mut self, f: F) -> Self
    where
        F: Fn(&mut AgentConfig) + Send + Sync + 'static,
    {
        self.customizers.push(Box::new(f));
        self
    }
}

#[async_trait]
impl AgentFactory for CustomAgentFactory {
    async fn create_agent(&self, mut config: AgentConfig) -> Result<Arc<dyn Agent>> {
        // Apply all customizers
        for customizer in &self.customizers {
            customizer(&mut config);
        }

        // Delegate to base factory
        self.base_factory.create_agent(config).await
    }

    async fn create_from_template(&self, template_name: &str) -> Result<Arc<dyn Agent>> {
        // For now, just delegate
        // TODO: Apply customizers to template config when we have agent implementations
        self.base_factory.create_from_template(template_name).await
    }

    fn list_templates(&self) -> Vec<&str> {
        self.base_factory.list_templates()
    }

    fn validate_config(&self, config: &AgentConfig) -> Result<()> {
        self.base_factory.validate_config(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factory::DefaultAgentFactory;
    use crate::ResourceLimits;
    use llmspell_providers::ProviderManager;

    #[tokio::test]
    async fn test_factory_registry() {
        let registry = FactoryRegistry::new();
        let provider_manager = Arc::new(ProviderManager::new());
        let factory1 = Arc::new(DefaultAgentFactory::new(provider_manager.clone()));
        let factory2 = Arc::new(DefaultAgentFactory::new(provider_manager.clone()));

        // Register factories
        registry
            .register_factory("factory1".to_string(), factory1)
            .await
            .unwrap();
        registry
            .register_factory("factory2".to_string(), factory2)
            .await
            .unwrap();

        // Check registration
        assert_eq!(registry.list_factories().await.len(), 2);
        assert!(registry.get_factory("factory1").await.is_some());
        assert!(registry.get_factory("factory2").await.is_some());
        assert!(registry.get_factory("nonexistent").await.is_none());

        // Check default factory
        let default = registry.get_default_factory().await;
        assert!(default.is_some());

        // Change default
        registry
            .set_default_factory("factory2".to_string())
            .await
            .unwrap();

        // Duplicate registration should fail
        let factory3 = Arc::new(DefaultAgentFactory::new(provider_manager.clone()));
        let result = registry
            .register_factory("factory1".to_string(), factory3)
            .await;
        assert!(result.is_err());
    }
    #[test]
    fn test_global_registry() {
        let registry = global_registry();
        // Should be able to access global registry
        let _ = registry;
    }
    #[test]
    fn test_custom_factory() {
        let provider_manager = Arc::new(ProviderManager::new());
        let base = Arc::new(DefaultAgentFactory::new(provider_manager));
        let custom = CustomAgentFactory::new(base)
            .with_customizer(|config| {
                config.resource_limits.max_execution_time_secs = 1000;
            })
            .with_customizer(|config| {
                config.allowed_tools.push("custom_tool".to_string());
            });

        // Verify templates are inherited
        let templates = custom.list_templates();
        assert!(!templates.is_empty());
    }
    #[tokio::test]
    async fn test_custom_factory_customization() {
        let provider_manager = Arc::new(ProviderManager::new());
        let base = Arc::new(DefaultAgentFactory::new(provider_manager));
        let custom = Arc::new(CustomAgentFactory::new(base).with_customizer(|config| {
            config.resource_limits.max_execution_time_secs = 1000;
            config.description = format!("{} (customized)", config.description);
        }));

        let agent = custom.create_from_template("basic").await.unwrap();

        // Can't directly verify customization without exposing config,
        // but we can verify agent was created
        assert_eq!(agent.metadata().name, "basic-agent");
    }
    #[tokio::test]
    async fn test_registry_default_factory() {
        let registry = FactoryRegistry::new();

        // No default factory initially
        let config = AgentConfig {
            name: "test".to_string(),
            description: "test".to_string(),
            agent_type: "basic".to_string(),
            model: None,
            allowed_tools: vec![],
            custom_config: serde_json::Map::new(),
            resource_limits: ResourceLimits::default(),
        };

        let result = registry.create_agent(config.clone()).await;
        assert!(result.is_err());

        // Register and set default
        let provider_manager = Arc::new(ProviderManager::new());
        let factory = Arc::new(DefaultAgentFactory::new(provider_manager));
        registry
            .register_factory("default".to_string(), factory)
            .await
            .unwrap();
        registry
            .set_default_factory("default".to_string())
            .await
            .unwrap();

        // Now should work
        let agent = registry.create_agent(config).await.unwrap();
        assert_eq!(agent.metadata().name, "test");
    }
    #[tokio::test]
    async fn test_registry_specific_factory() {
        let registry = FactoryRegistry::new();

        let provider_manager = Arc::new(ProviderManager::new());
        let factory1 = Arc::new(DefaultAgentFactory::new(provider_manager.clone()));
        let factory2 = Arc::new(DefaultAgentFactory::new(provider_manager.clone()));

        registry
            .register_factory("factory1".to_string(), factory1)
            .await
            .unwrap();
        registry
            .register_factory("factory2".to_string(), factory2)
            .await
            .unwrap();

        let config = AgentConfig {
            name: "test".to_string(),
            description: "test".to_string(),
            agent_type: "basic".to_string(),
            model: None,
            allowed_tools: vec![],
            custom_config: serde_json::Map::new(),
            resource_limits: ResourceLimits::default(),
        };

        // Create with specific factory
        let agent = registry
            .create_agent_with("factory2", config)
            .await
            .unwrap();
        assert_eq!(agent.metadata().name, "test");

        // Try with non-existent factory
        let config2 = AgentConfig {
            name: "test2".to_string(),
            description: "test2".to_string(),
            agent_type: "basic".to_string(),
            model: None,
            allowed_tools: vec![],
            custom_config: serde_json::Map::new(),
            resource_limits: ResourceLimits::default(),
        };

        let result = registry.create_agent_with("non-existent", config2).await;
        assert!(result.is_err());
    }
}
