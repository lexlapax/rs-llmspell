//! ABOUTME: Agent registration mechanism with validation and lifecycle management
//! ABOUTME: Handles agent registration, validation, and status transitions

use super::{AgentMetadata, AgentMetrics, AgentRegistry, AgentStatus};
use crate::factory::AgentConfig;
use anyhow::Result;
use llmspell_core::traits::agent::Agent;
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

/// Agent registration options
#[derive(Debug, Clone)]
pub struct RegistrationOptions {
    /// Custom agent ID (generated if not provided)
    pub agent_id: Option<String>,

    /// Initial categories
    pub categories: Vec<String>,

    /// Custom metadata
    pub custom_metadata: HashMap<String, serde_json::Value>,

    /// Auto-start agent after registration
    pub auto_start: bool,

    /// Enable heartbeat monitoring
    pub enable_heartbeat: bool,
}

impl Default for RegistrationOptions {
    fn default() -> Self {
        Self {
            agent_id: None,
            categories: Vec::new(),
            custom_metadata: HashMap::new(),
            auto_start: true,
            enable_heartbeat: true,
        }
    }
}

/// Agent registration builder
pub struct RegistrationBuilder {
    options: RegistrationOptions,
}

impl RegistrationBuilder {
    /// Create new registration builder
    #[must_use]
    pub fn new() -> Self {
        Self {
            options: RegistrationOptions::default(),
        }
    }

    /// Set custom agent ID
    #[must_use]
    pub fn with_id(mut self, id: String) -> Self {
        self.options.agent_id = Some(id);
        self
    }

    /// Add categories
    #[must_use]
    pub fn with_categories(mut self, categories: Vec<String>) -> Self {
        self.options.categories = categories;
        self
    }

    /// Add a single category
    #[must_use]
    pub fn add_category(mut self, category: String) -> Self {
        self.options.categories.push(category);
        self
    }

    /// Add custom metadata
    #[must_use]
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.options.custom_metadata.insert(key, value);
        self
    }

    /// Set auto-start behavior
    #[must_use]
    pub const fn auto_start(mut self, enabled: bool) -> Self {
        self.options.auto_start = enabled;
        self
    }

    /// Set heartbeat monitoring
    #[must_use]
    pub const fn enable_heartbeat(mut self, enabled: bool) -> Self {
        self.options.enable_heartbeat = enabled;
        self
    }

    /// Build registration options
    #[must_use]
    pub fn build(self) -> RegistrationOptions {
        self.options
    }
}

impl Default for RegistrationBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Agent registrar for managing registration process
pub struct AgentRegistrar<R: AgentRegistry> {
    registry: Arc<R>,
}

impl<R: AgentRegistry> AgentRegistrar<R> {
    /// Create new registrar
    pub const fn new(registry: Arc<R>) -> Self {
        Self { registry }
    }

    /// Register agent with default options
    ///
    /// # Errors
    ///
    /// Returns an error if agent registration fails
    pub async fn register_agent(
        &self,
        agent: Arc<dyn Agent>,
        config: &AgentConfig,
    ) -> Result<String> {
        let options = RegistrationOptions::default();
        self.register_agent_with_options(agent, config, options)
            .await
    }

    /// Register agent with custom options
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Agent validation fails
    /// - Metadata serialization fails
    /// - Registry registration fails
    /// - Heartbeat fails (if enabled)
    pub async fn register_agent_with_options(
        &self,
        agent: Arc<dyn Agent>,
        config: &AgentConfig,
        options: RegistrationOptions,
    ) -> Result<String> {
        // Generate or use provided ID
        let agent_id = options
            .agent_id
            .unwrap_or_else(|| format!("{}-{}", config.name, Uuid::new_v4()));

        // Validate agent
        self.validate_agent(&agent, config).await?;

        // Build custom metadata
        let mut custom_metadata = options.custom_metadata;
        custom_metadata.insert(
            "resource_limits".to_string(),
            serde_json::to_value(&config.resource_limits)?,
        );

        if !config.allowed_tools.is_empty() {
            custom_metadata.insert(
                "allowed_tools".to_string(),
                serde_json::to_value(&config.allowed_tools)?,
            );
        }

        // Create metadata
        let metadata = AgentMetadata {
            id: agent_id.clone(),
            name: config.name.clone(),
            agent_type: config.agent_type.clone(),
            description: config.description.clone(),
            categories: options.categories,
            custom_metadata,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            status: if options.auto_start {
                AgentStatus::Active
            } else {
                AgentStatus::Initializing
            },
            metrics: AgentMetrics::default(),
        };

        // Register with registry
        self.registry
            .register_agent(agent_id.clone(), agent, metadata)
            .await?;

        // Send initial heartbeat if enabled
        if options.enable_heartbeat {
            self.registry.heartbeat(&agent_id).await?;
        }

        Ok(agent_id)
    }

    /// Validate agent before registration
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Agent validation fails
    /// - Agent is not responsive
    /// - Configuration is invalid
    async fn validate_agent(&self, agent: &Arc<dyn Agent>, config: &AgentConfig) -> Result<()> {
        // Check agent is responsive
        let test_input = llmspell_core::types::AgentInput::text("__registry_validation__");

        // Try to validate input (should not fail for valid agent)
        agent
            .validate_input(&test_input)
            .await
            .map_err(|e| anyhow::anyhow!("Agent validation failed: {}", e))?;

        // Verify name matches
        if agent.metadata().name != config.name {
            anyhow::bail!(
                "Agent name mismatch: expected '{}', got '{}'",
                config.name,
                agent.metadata().name
            );
        }

        Ok(())
    }

    /// Unregister agent
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Agent not found
    /// - Status update fails
    /// - Removal from registry fails
    pub async fn unregister_agent(&self, id: &str) -> Result<()> {
        // Update status to stopped first
        self.registry
            .update_status(id, AgentStatus::Stopped)
            .await?;

        // Remove from registry
        self.registry.unregister_agent(id).await
    }

    /// Batch register multiple agents
    ///
    /// # Errors
    ///
    /// Returns an error if any agent registration fails
    pub async fn register_agents(
        &self,
        agents: Vec<(Arc<dyn Agent>, AgentConfig)>,
    ) -> Result<Vec<String>> {
        let mut ids = Vec::new();

        for (agent, config) in agents {
            let id = self.register_agent(agent, &config).await?;
            ids.push(id);
        }

        Ok(ids)
    }
}

/// Registration lifecycle hooks
#[async_trait::async_trait]
pub trait RegistrationHook: Send + Sync {
    /// Called before registration
    ///
    /// # Errors
    ///
    /// Returns an error if pre-registration validation fails
    async fn before_register(&self, config: &AgentConfig) -> Result<()>;

    /// Called after successful registration
    ///
    /// # Errors
    ///
    /// Returns an error if post-registration processing fails
    async fn after_register(&self, id: &str, metadata: &AgentMetadata) -> Result<()>;

    /// Called on registration failure
    async fn on_register_error(&self, config: &AgentConfig, error: &anyhow::Error) -> Result<()>;
}

/// Composite registration hook
pub struct CompositeRegistrationHook {
    hooks: Vec<Arc<dyn RegistrationHook>>,
}

impl CompositeRegistrationHook {
    /// Create new composite hook
    #[must_use]
    pub fn new() -> Self {
        Self { hooks: Vec::new() }
    }

    /// Add a hook
    pub fn add_hook(mut self, hook: Arc<dyn RegistrationHook>) -> Self {
        self.hooks.push(hook);
        self
    }
}

impl Default for CompositeRegistrationHook {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl RegistrationHook for CompositeRegistrationHook {
    async fn before_register(&self, config: &AgentConfig) -> Result<()> {
        for hook in &self.hooks {
            hook.before_register(config).await?;
        }
        Ok(())
    }

    async fn after_register(&self, id: &str, metadata: &AgentMetadata) -> Result<()> {
        for hook in &self.hooks {
            hook.after_register(id, metadata).await?;
        }
        Ok(())
    }

    async fn on_register_error(&self, config: &AgentConfig, error: &anyhow::Error) -> Result<()> {
        for hook in &self.hooks {
            hook.on_register_error(config, error).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_registration_builder() {
        let options = RegistrationBuilder::new()
            .with_id("test-agent".to_string())
            .add_category("test".to_string())
            .add_category("demo".to_string())
            .with_metadata("version".to_string(), serde_json::json!("1.0"))
            .auto_start(false)
            .enable_heartbeat(true)
            .build();

        assert_eq!(options.agent_id, Some("test-agent".to_string()));
        assert_eq!(options.categories.len(), 2);
        assert!(!options.auto_start);
        assert!(options.enable_heartbeat);
    }
}
