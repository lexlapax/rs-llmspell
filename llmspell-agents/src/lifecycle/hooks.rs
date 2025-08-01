//! ABOUTME: Agent lifecycle hooks for customizing agent creation and destruction
//! ABOUTME: Provides hooks that run at various stages of agent lifecycle

use crate::factory::{AgentConfig, CreationHook};
use anyhow::Result;
use async_trait::async_trait;
use llmspell_core::traits::agent::Agent;
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Validation hook that validates agent configuration
pub struct ValidationHook {
    /// Minimum allowed execution time
    min_execution_time_secs: u64,

    /// Maximum allowed execution time
    max_execution_time_secs: u64,

    /// Allowed agent types
    allowed_types: Vec<String>,
}

impl ValidationHook {
    pub fn new() -> Self {
        Self {
            min_execution_time_secs: 1,
            max_execution_time_secs: 86400, // 24 hours
            allowed_types: vec![
                "basic".to_string(),
                "tool-orchestrator".to_string(),
                "llm".to_string(),
                "workflow".to_string(),
                "research".to_string(),
                "code-assistant".to_string(),
                "data-processor".to_string(),
                "monitor".to_string(),
                "security".to_string(),
            ],
        }
    }

    pub fn with_execution_limits(mut self, min: u64, max: u64) -> Self {
        self.min_execution_time_secs = min;
        self.max_execution_time_secs = max;
        self
    }

    pub fn with_allowed_types(mut self, types: Vec<String>) -> Self {
        self.allowed_types = types;
        self
    }
}

impl Default for ValidationHook {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CreationHook for ValidationHook {
    async fn before_create(&self, config: &AgentConfig) -> Result<()> {
        debug!("Validating agent configuration for: {}", config.name);

        // Validate execution time limits
        let exec_time = config.resource_limits.max_execution_time_secs;
        if exec_time < self.min_execution_time_secs {
            anyhow::bail!(
                "Execution time {} is below minimum {}",
                exec_time,
                self.min_execution_time_secs
            );
        }
        if exec_time > self.max_execution_time_secs {
            anyhow::bail!(
                "Execution time {} exceeds maximum {}",
                exec_time,
                self.max_execution_time_secs
            );
        }

        // Validate agent type
        if !self.allowed_types.contains(&config.agent_type) {
            anyhow::bail!(
                "Agent type '{}' is not allowed. Allowed types: {:?}",
                config.agent_type,
                self.allowed_types
            );
        }

        // Validate tool permissions
        if config.allowed_tools.contains(&"*".to_string()) && config.agent_type == "basic" {
            warn!("Basic agent '{}' has access to all tools", config.name);
        }

        Ok(())
    }

    async fn after_create(&self, agent: &Arc<dyn Agent>) -> Result<()> {
        debug!("Agent '{}' created successfully", agent.metadata().name);
        Ok(())
    }
}

/// Logging hook that logs agent lifecycle events
pub struct LoggingHook {
    log_level: tracing::Level,
}

impl LoggingHook {
    pub fn new() -> Self {
        Self {
            log_level: tracing::Level::INFO,
        }
    }

    pub fn with_level(mut self, level: tracing::Level) -> Self {
        self.log_level = level;
        self
    }
}

impl Default for LoggingHook {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CreationHook for LoggingHook {
    async fn before_create(&self, config: &AgentConfig) -> Result<()> {
        match self.log_level {
            tracing::Level::DEBUG => debug!(
                "Creating agent '{}' of type '{}'",
                config.name, config.agent_type
            ),
            tracing::Level::INFO => info!(
                "Creating agent '{}' of type '{}'",
                config.name, config.agent_type
            ),
            _ => {}
        }
        Ok(())
    }

    async fn after_create(&self, agent: &Arc<dyn Agent>) -> Result<()> {
        let metadata = agent.metadata();
        match self.log_level {
            tracing::Level::DEBUG => debug!("Agent '{}' created successfully", metadata.name),
            tracing::Level::INFO => info!("Agent '{}' created successfully", metadata.name),
            _ => {}
        }
        Ok(())
    }
}

/// Metrics hook that tracks agent creation metrics
pub struct MetricsHook {
    creation_counter: Arc<std::sync::atomic::AtomicUsize>,
    start_times: Arc<tokio::sync::Mutex<std::collections::HashMap<String, std::time::Instant>>>,
}

impl MetricsHook {
    pub fn new() -> Self {
        Self {
            creation_counter: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            start_times: Arc::new(tokio::sync::Mutex::new(std::collections::HashMap::new())),
        }
    }

    pub fn get_creation_count(&self) -> usize {
        self.creation_counter
            .load(std::sync::atomic::Ordering::Relaxed)
    }
}

impl Default for MetricsHook {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CreationHook for MetricsHook {
    async fn before_create(&self, config: &AgentConfig) -> Result<()> {
        let mut times = self.start_times.lock().await;
        times.insert(config.name.clone(), std::time::Instant::now());
        Ok(())
    }

    async fn after_create(&self, agent: &Arc<dyn Agent>) -> Result<()> {
        let name = agent.metadata().name.clone();
        let mut times = self.start_times.lock().await;

        if let Some(start_time) = times.remove(&name) {
            let duration = start_time.elapsed();
            debug!("Agent '{}' created in {:?}", name, duration);
        }

        self.creation_counter
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }
}

/// Security hook that enforces security policies
pub struct SecurityHook {
    require_resource_limits: bool,
    max_tool_access: Option<usize>,
    forbidden_tools: Vec<String>,
}

impl SecurityHook {
    pub fn new() -> Self {
        Self {
            require_resource_limits: true,
            max_tool_access: None,
            forbidden_tools: vec![],
        }
    }

    pub fn with_max_tool_access(mut self, max: usize) -> Self {
        self.max_tool_access = Some(max);
        self
    }

    pub fn with_forbidden_tools(mut self, tools: Vec<String>) -> Self {
        self.forbidden_tools = tools;
        self
    }
}

impl Default for SecurityHook {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CreationHook for SecurityHook {
    async fn before_create(&self, config: &AgentConfig) -> Result<()> {
        // Check resource limits
        if self.require_resource_limits {
            if config.resource_limits.max_memory_mb == 0 {
                anyhow::bail!("Agent must have memory limits set");
            }
            if config.resource_limits.max_execution_time_secs == 0 {
                anyhow::bail!("Agent must have execution time limits set");
            }
        }

        // Check tool access
        if let Some(max_tools) = self.max_tool_access {
            if config.allowed_tools.contains(&"*".to_string()) {
                anyhow::bail!("Agent cannot have unrestricted tool access");
            }
            if config.allowed_tools.len() > max_tools {
                anyhow::bail!(
                    "Agent has access to {} tools, maximum allowed is {}",
                    config.allowed_tools.len(),
                    max_tools
                );
            }
        }

        // Check forbidden tools
        for tool in &config.allowed_tools {
            if self.forbidden_tools.contains(tool) {
                anyhow::bail!("Agent cannot access forbidden tool: {}", tool);
            }
        }

        Ok(())
    }

    async fn after_create(&self, _agent: &Arc<dyn Agent>) -> Result<()> {
        Ok(())
    }
}

/// Composite hook that runs multiple hooks
pub struct CompositeHook {
    hooks: Vec<Arc<dyn CreationHook>>,
}

impl CompositeHook {
    pub fn new() -> Self {
        Self { hooks: vec![] }
    }

    pub fn add_hook(mut self, hook: Arc<dyn CreationHook>) -> Self {
        self.hooks.push(hook);
        self
    }
}

impl Default for CompositeHook {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl CreationHook for CompositeHook {
    async fn before_create(&self, config: &AgentConfig) -> Result<()> {
        for hook in &self.hooks {
            hook.before_create(config).await?;
        }
        Ok(())
    }

    async fn after_create(&self, agent: &Arc<dyn Agent>) -> Result<()> {
        for hook in &self.hooks {
            hook.after_create(agent).await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::builder::AgentBuilder;

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_validation_hook() {
        let hook = ValidationHook::new()
            .with_execution_limits(10, 3600)
            .with_allowed_types(vec!["basic".to_string(), "test".to_string()]);

        // Valid config
        let valid_config = AgentBuilder::basic("test")
            .max_execution_time_secs(60)
            .build()
            .unwrap();
        assert!(hook.before_create(&valid_config).await.is_ok());

        // Invalid - execution time too low
        let invalid_config = AgentBuilder::basic("test")
            .max_execution_time_secs(5)
            .build()
            .unwrap();
        assert!(hook.before_create(&invalid_config).await.is_err());

        // Invalid - unknown agent type
        let invalid_config = AgentBuilder::new("test", "unknown")
            .max_execution_time_secs(60)
            .build()
            .unwrap();
        assert!(hook.before_create(&invalid_config).await.is_err());
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_security_hook() {
        let hook = SecurityHook::new()
            .with_max_tool_access(5)
            .with_forbidden_tools(vec!["dangerous_tool".to_string()]);

        // Valid config
        let valid_config = AgentBuilder::basic("test")
            .allow_tools(vec!["safe_tool".to_string()])
            .build()
            .unwrap();
        assert!(hook.before_create(&valid_config).await.is_ok());

        // Invalid - unrestricted tool access
        let invalid_config = AgentBuilder::basic("test")
            .allow_all_tools()
            .build()
            .unwrap();
        assert!(hook.before_create(&invalid_config).await.is_err());

        // Invalid - forbidden tool
        let invalid_config = AgentBuilder::basic("test")
            .allow_tool("dangerous_tool")
            .build()
            .unwrap();
        assert!(hook.before_create(&invalid_config).await.is_err());
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_metrics_hook() {
        let hook = MetricsHook::new();
        assert_eq!(hook.get_creation_count(), 0);

        let config = AgentBuilder::basic("test").build().unwrap();
        hook.before_create(&config).await.unwrap();

        // Simulate agent creation
        // In real usage, after_create would be called with actual agent
        // hook.after_create(&agent).await.unwrap();
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_logging_hook() {
        // Just test that it doesn't panic
        let hook = LoggingHook::new().with_level(tracing::Level::DEBUG);
        let config = AgentBuilder::basic("test").build().unwrap();

        assert!(hook.before_create(&config).await.is_ok());

        let agent: Arc<dyn Agent> = Arc::new(crate::agents::BasicAgent::new(config).unwrap());
        assert!(hook.after_create(&agent).await.is_ok());
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_composite_hook() {
        use std::sync::atomic::{AtomicUsize, Ordering};

        struct CountingHook {
            count: Arc<AtomicUsize>,
        }

        #[async_trait]
        impl CreationHook for CountingHook {
            async fn before_create(&self, _config: &AgentConfig) -> Result<()> {
                self.count.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }

            async fn after_create(&self, _agent: &Arc<dyn Agent>) -> Result<()> {
                self.count.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        }

        let count1 = Arc::new(AtomicUsize::new(0));
        let count2 = Arc::new(AtomicUsize::new(0));

        let composite = CompositeHook::new()
            .add_hook(Arc::new(CountingHook {
                count: count1.clone(),
            }))
            .add_hook(Arc::new(CountingHook {
                count: count2.clone(),
            }));

        let config = AgentBuilder::basic("test").build().unwrap();
        composite.before_create(&config).await.unwrap();

        let agent: Arc<dyn Agent> = Arc::new(crate::agents::BasicAgent::new(config).unwrap());
        composite.after_create(&agent).await.unwrap();

        assert_eq!(count1.load(Ordering::SeqCst), 2);
        assert_eq!(count2.load(Ordering::SeqCst), 2);
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_composite_hook_error_propagation() {
        struct FailingHook;

        #[async_trait]
        impl CreationHook for FailingHook {
            async fn before_create(&self, _config: &AgentConfig) -> Result<()> {
                anyhow::bail!("Test error")
            }

            async fn after_create(&self, _agent: &Arc<dyn Agent>) -> Result<()> {
                Ok(())
            }
        }

        let composite = CompositeHook::new().add_hook(Arc::new(FailingHook));

        let config = AgentBuilder::basic("test").build().unwrap();
        let result = composite.before_create(&config).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Test error"));
    }
}
