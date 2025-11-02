//! ABOUTME: Session middleware implementation with configurable patterns
//! ABOUTME: Integrates with `HookRegistry` and `HookExecutor` for session operation processing

#![allow(clippy::missing_panics_doc)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::struct_excessive_bools)]

use anyhow::Result;
use async_trait::async_trait;
use futures::future::join_all;
use llmspell_hooks::{
    builtin::{CachingHook, LoggingHook, MetricsHook, RateLimitHook, SecurityHook},
    traits::Hook,
    types::HookMetadata,
    HookContext, HookExecutor, HookPoint, HookRegistry, HookResult,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Middleware configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiddlewareConfig {
    /// Enable logging middleware
    pub enable_logging: bool,
    /// Enable metrics collection
    pub enable_metrics: bool,
    /// Enable security checks
    pub enable_security: bool,
    /// Enable caching for read operations
    pub enable_caching: bool,
    /// Enable rate limiting
    pub enable_rate_limiting: bool,
    /// Custom middleware hooks
    pub custom_hooks: Vec<String>,
    /// Middleware execution pattern
    pub pattern: MiddlewarePattern,
}

impl Default for MiddlewareConfig {
    fn default() -> Self {
        Self {
            enable_logging: true,
            enable_metrics: true,
            enable_security: true,
            enable_caching: true,
            enable_rate_limiting: false,
            custom_hooks: Vec::new(),
            pattern: MiddlewarePattern::Sequential,
        }
    }
}

/// Middleware execution pattern
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum MiddlewarePattern {
    /// Execute middleware in sequence
    Sequential,
    /// Execute middleware in parallel
    Parallel,
    /// Use voting for consensus
    Voting {
        /// Percentage of hooks that must pass (0.0 to 1.0)
        threshold: f32,
    },
}

/// Type of middleware to create
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MiddlewareType {
    /// Middleware for session creation
    SessionCreate,
    /// Middleware for session read operations
    SessionRead,
    /// Middleware for session update operations
    SessionUpdate,
    /// Middleware for session deletion
    SessionDelete,
    /// General session operation
    SessionOperation,
}

/// Session middleware manager
pub struct SessionMiddleware {
    /// Middleware configuration
    config: MiddlewareConfig,
    /// Hook registry
    hook_registry: Arc<HookRegistry>,
    /// Hook executor
    hook_executor: Arc<HookExecutor>,
    /// Registered middleware chains
    middleware_chains: HashMap<MiddlewareType, Arc<dyn Hook>>,
}

impl SessionMiddleware {
    /// Create new session middleware
    pub fn new(
        config: MiddlewareConfig,
        hook_registry: Arc<HookRegistry>,
        hook_executor: Arc<HookExecutor>,
    ) -> Self {
        Self {
            config,
            hook_registry,
            hook_executor,
            middleware_chains: HashMap::new(),
        }
    }

    /// Initialize middleware chains
    ///
    /// # Errors
    /// Returns error if middleware chain creation fails
    pub fn initialize(&mut self) -> Result<()> {
        // Create middleware for different operation types
        self.middleware_chains.insert(
            MiddlewareType::SessionCreate,
            self.create_middleware_chain(MiddlewareType::SessionCreate),
        );

        self.middleware_chains.insert(
            MiddlewareType::SessionRead,
            self.create_middleware_chain(MiddlewareType::SessionRead),
        );

        self.middleware_chains.insert(
            MiddlewareType::SessionUpdate,
            self.create_middleware_chain(MiddlewareType::SessionUpdate),
        );

        self.middleware_chains.insert(
            MiddlewareType::SessionDelete,
            self.create_middleware_chain(MiddlewareType::SessionDelete),
        );

        self.middleware_chains.insert(
            MiddlewareType::SessionOperation,
            self.create_middleware_chain(MiddlewareType::SessionOperation),
        );

        // Register all middleware chains with the hook registry
        self.register_middleware()?;

        Ok(())
    }

    /// Create middleware chain for specific operation type
    fn create_middleware_chain(&self, middleware_type: MiddlewareType) -> Arc<dyn Hook> {
        let mut hooks: Vec<Arc<dyn Hook>> = Vec::new();

        // Add enabled middleware components
        if self.config.enable_logging {
            hooks.push(Arc::new(LoggingHook::new()));
        }

        if self.config.enable_metrics {
            hooks.push(Arc::new(MetricsHook::new()));
        }

        if self.config.enable_security {
            hooks.push(Arc::new(SecurityHook::new()));
        }

        // Add caching for read operations
        if self.config.enable_caching && middleware_type == MiddlewareType::SessionRead {
            hooks.push(Arc::new(CachingHook::new()));
        }

        // Add rate limiting if enabled
        if self.config.enable_rate_limiting {
            let rate_limiter = RateLimitHook::new()
                .with_rate_per_second(10.0) // 10 requests per second
                .with_burst(20);
            hooks.push(Arc::new(rate_limiter));
        }

        // Create composite hook based on pattern
        let name = format!("{middleware_type:?}Middleware");
        let middleware: Arc<dyn Hook> = match self.config.pattern {
            MiddlewarePattern::Sequential => Arc::new(
                SequentialMiddleware::new(&name)
                    .add_hooks(hooks)
                    .with_metadata(HookMetadata {
                        name: name.clone(),
                        version: "1.0.0".to_string(),
                        description: Some(format!(
                            "Middleware chain for {middleware_type:?} operations"
                        )),
                        priority: llmspell_hooks::Priority(50),
                        tags: vec!["middleware".to_string(), "session".to_string()],
                        language: llmspell_hooks::Language::Native,
                    }),
            ),
            MiddlewarePattern::Parallel => Arc::new(
                ParallelMiddleware::new(&name)
                    .add_hooks(hooks)
                    .with_metadata(HookMetadata {
                        name: name.clone(),
                        version: "1.0.0".to_string(),
                        description: Some(format!(
                            "Middleware chain for {middleware_type:?} operations"
                        )),
                        priority: llmspell_hooks::Priority(50),
                        tags: vec!["middleware".to_string(), "session".to_string()],
                        language: llmspell_hooks::Language::Native,
                    }),
            ),
            MiddlewarePattern::Voting { threshold } => Arc::new(
                VotingMiddleware::new(&name, f64::from(threshold))
                    .add_hooks(hooks)
                    .with_metadata(HookMetadata {
                        name: name.clone(),
                        version: "1.0.0".to_string(),
                        description: Some(format!(
                            "Middleware chain for {middleware_type:?} operations"
                        )),
                        priority: llmspell_hooks::Priority(50),
                        tags: vec!["middleware".to_string(), "session".to_string()],
                        language: llmspell_hooks::Language::Native,
                    }),
            ),
        };

        middleware
    }

    /// Register middleware with hook registry
    fn register_middleware(&self) -> Result<()> {
        // Map middleware types to hook points
        let mappings = vec![
            (MiddlewareType::SessionCreate, HookPoint::SessionStart),
            (MiddlewareType::SessionRead, HookPoint::SessionCheckpoint),
            (MiddlewareType::SessionUpdate, HookPoint::SessionSave),
            (MiddlewareType::SessionDelete, HookPoint::SessionEnd),
            (MiddlewareType::SessionOperation, HookPoint::SessionRestore),
        ];

        for (middleware_type, hook_point) in mappings {
            if let Some(middleware) = self.middleware_chains.get(&middleware_type) {
                self.hook_registry
                    .register_arc(hook_point, Arc::clone(middleware))
                    .map_err(|e| anyhow::anyhow!("Failed to register middleware: {e:?}"))?;
            }
        }

        Ok(())
    }

    /// Execute middleware for a specific operation
    ///
    /// # Errors
    /// Returns error if middleware execution fails or middleware chain is not found
    pub async fn execute_middleware(
        &self,
        middleware_type: MiddlewareType,
        context: &mut HookContext,
    ) -> Result<HookResult> {
        // Get the appropriate hook point for the middleware type
        let hook_point = match middleware_type {
            MiddlewareType::SessionCreate => HookPoint::SessionStart,
            MiddlewareType::SessionRead => HookPoint::SessionCheckpoint,
            MiddlewareType::SessionUpdate => HookPoint::SessionSave,
            MiddlewareType::SessionDelete => HookPoint::SessionEnd,
            MiddlewareType::SessionOperation => HookPoint::SessionRestore,
        };

        // Update context with the correct hook point
        context.point = hook_point.clone();

        // Get hooks for this point
        let hooks = self.hook_registry.get_hooks(&hook_point);

        // Execute hooks through the executor and get results
        let results = self
            .hook_executor
            .execute_hooks(&hooks, context)
            .await
            .map_err(|e| anyhow::anyhow!("Middleware execution failed: {e}"))?;

        // Return first non-Continue result or Continue if all passed
        for result in results {
            if !result.should_continue() {
                return Ok(result);
            }
        }

        Ok(HookResult::Continue)
    }

    /// Get middleware configuration
    pub fn config(&self) -> &MiddlewareConfig {
        &self.config
    }

    /// Update middleware configuration
    ///
    /// # Errors
    /// Returns error if middleware chain recreation fails
    pub fn update_config(&mut self, config: MiddlewareConfig) -> Result<()> {
        self.config = config;
        self.middleware_chains.clear();
        // Don't call initialize() as it registers hooks again
        // Instead, just recreate the middleware chains
        self.middleware_chains.insert(
            MiddlewareType::SessionCreate,
            self.create_middleware_chain(MiddlewareType::SessionCreate),
        );

        self.middleware_chains.insert(
            MiddlewareType::SessionRead,
            self.create_middleware_chain(MiddlewareType::SessionRead),
        );

        self.middleware_chains.insert(
            MiddlewareType::SessionUpdate,
            self.create_middleware_chain(MiddlewareType::SessionUpdate),
        );

        self.middleware_chains.insert(
            MiddlewareType::SessionDelete,
            self.create_middleware_chain(MiddlewareType::SessionDelete),
        );

        self.middleware_chains.insert(
            MiddlewareType::SessionOperation,
            self.create_middleware_chain(MiddlewareType::SessionOperation),
        );

        Ok(())
    }
}

/// Create session creation middleware
///
/// # Errors
/// Returns error if middleware initialization fails
pub fn create_session_middleware(
    config: MiddlewareConfig,
    registry: Arc<HookRegistry>,
    executor: Arc<HookExecutor>,
) -> Result<Arc<SessionMiddleware>> {
    let mut middleware = SessionMiddleware::new(config, registry, executor);
    middleware.initialize()?;
    Ok(Arc::new(middleware))
}

/// Create operation-specific middleware
///
/// # Errors
/// Returns error if middleware creation or initialization fails
pub fn create_operation_middleware(
    operation_type: MiddlewareType,
    registry: Arc<HookRegistry>,
    executor: Arc<HookExecutor>,
) -> Result<Arc<dyn Hook>> {
    let config = match operation_type {
        MiddlewareType::SessionRead => MiddlewareConfig {
            enable_caching: true,
            enable_rate_limiting: false,
            pattern: MiddlewarePattern::Parallel,
            ..Default::default()
        },
        MiddlewareType::SessionCreate | MiddlewareType::SessionDelete => MiddlewareConfig {
            enable_security: true,
            enable_logging: true,
            pattern: MiddlewarePattern::Sequential,
            ..Default::default()
        },
        _ => MiddlewareConfig::default(),
    };

    let middleware = SessionMiddleware::new(config, registry, executor);
    Ok(middleware.create_middleware_chain(operation_type))
}

/// Sequential middleware implementation
#[derive(Debug)]
pub struct SequentialMiddleware {
    #[allow(dead_code)]
    name: String,
    hooks: Vec<Arc<dyn Hook>>,
    metadata: HookMetadata,
}

impl SequentialMiddleware {
    /// Create new sequential middleware
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            hooks: Vec::new(),
            metadata: HookMetadata {
                name: name.to_string(),
                description: Some("Sequential middleware execution".to_string()),
                ..Default::default()
            },
        }
    }

    /// Add a hook
    #[must_use]
    pub fn add_hook(mut self, hook: Arc<dyn Hook>) -> Self {
        self.hooks.push(hook);
        self
    }

    /// Add multiple hooks
    #[must_use]
    pub fn add_hooks(mut self, hooks: Vec<Arc<dyn Hook>>) -> Self {
        self.hooks.extend(hooks);
        self
    }

    /// Set metadata
    #[must_use]
    pub fn with_metadata(mut self, metadata: HookMetadata) -> Self {
        self.metadata = metadata;
        self
    }
}

#[async_trait]
impl Hook for SequentialMiddleware {
    async fn execute(&self, context: &mut HookContext) -> Result<HookResult> {
        // Execute hooks in sequence, stop on first non-Continue
        for hook in &self.hooks {
            if hook.should_execute(context) {
                let result = hook.execute(context).await?;
                if !result.should_continue() {
                    return Ok(result);
                }
            }
        }
        Ok(HookResult::Continue)
    }

    fn metadata(&self) -> HookMetadata {
        self.metadata.clone()
    }

    fn should_execute(&self, context: &HookContext) -> bool {
        // Execute if any hook should execute
        self.hooks.iter().any(|h| h.should_execute(context))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Parallel middleware implementation
#[derive(Debug)]
pub struct ParallelMiddleware {
    #[allow(dead_code)]
    name: String,
    hooks: Vec<Arc<dyn Hook>>,
    metadata: HookMetadata,
}

impl ParallelMiddleware {
    /// Create new parallel middleware
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            hooks: Vec::new(),
            metadata: HookMetadata {
                name: name.to_string(),
                description: Some("Parallel middleware execution".to_string()),
                ..Default::default()
            },
        }
    }

    /// Add a hook
    #[must_use]
    pub fn add_hook(mut self, hook: Arc<dyn Hook>) -> Self {
        self.hooks.push(hook);
        self
    }

    /// Add multiple hooks
    #[must_use]
    pub fn add_hooks(mut self, hooks: Vec<Arc<dyn Hook>>) -> Self {
        self.hooks.extend(hooks);
        self
    }

    /// Set metadata
    #[must_use]
    pub fn with_metadata(mut self, metadata: HookMetadata) -> Self {
        self.metadata = metadata;
        self
    }
}

#[async_trait]
impl Hook for ParallelMiddleware {
    async fn execute(&self, context: &mut HookContext) -> Result<HookResult> {
        // Execute all hooks in parallel
        let futures: Vec<_> = self
            .hooks
            .iter()
            .filter(|h| h.should_execute(context))
            .map(|hook| {
                let mut ctx_clone = context.clone();
                let hook_clone = Arc::clone(hook);
                async move { hook_clone.execute(&mut ctx_clone).await }
            })
            .collect();

        let results = join_all(futures).await;

        // Check for errors first
        for result in &results {
            if let Err(e) = result {
                return Err(anyhow::anyhow!("Parallel execution failed: {e}"));
            }
        }

        // Return first non-Continue result
        for hook_result in results.into_iter().flatten() {
            if !hook_result.should_continue() {
                return Ok(hook_result);
            }
        }

        Ok(HookResult::Continue)
    }

    fn metadata(&self) -> HookMetadata {
        self.metadata.clone()
    }

    fn should_execute(&self, context: &HookContext) -> bool {
        self.hooks.iter().any(|h| h.should_execute(context))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Voting middleware implementation
#[derive(Debug)]
pub struct VotingMiddleware {
    #[allow(dead_code)]
    name: String,
    hooks: Vec<Arc<dyn Hook>>,
    threshold: f64,
    metadata: HookMetadata,
}

impl VotingMiddleware {
    /// Create new voting middleware
    pub fn new(name: &str, threshold: f64) -> Self {
        Self {
            name: name.to_string(),
            hooks: Vec::new(),
            threshold,
            metadata: HookMetadata {
                name: name.to_string(),
                description: Some(format!(
                    "Voting middleware with {}% threshold",
                    threshold * 100.0
                )),
                ..Default::default()
            },
        }
    }

    /// Add a hook
    #[must_use]
    pub fn add_hook(mut self, hook: Arc<dyn Hook>) -> Self {
        self.hooks.push(hook);
        self
    }

    /// Add multiple hooks
    #[must_use]
    pub fn add_hooks(mut self, hooks: Vec<Arc<dyn Hook>>) -> Self {
        self.hooks.extend(hooks);
        self
    }

    /// Set metadata
    #[must_use]
    pub fn with_metadata(mut self, metadata: HookMetadata) -> Self {
        self.metadata = metadata;
        self
    }
}

#[async_trait]
impl Hook for VotingMiddleware {
    async fn execute(&self, context: &mut HookContext) -> Result<HookResult> {
        if self.hooks.is_empty() {
            return Ok(HookResult::Continue);
        }

        // Execute all hooks and collect results
        let mut continue_count = 0u32;
        let mut total_count = 0u32;
        let mut non_continue_results = Vec::new();

        for hook in &self.hooks {
            if hook.should_execute(context) {
                total_count += 1;
                let result = hook.execute(context).await?;
                if result.should_continue() {
                    continue_count += 1;
                } else {
                    non_continue_results.push(result);
                }
            }
        }

        if total_count == 0 {
            return Ok(HookResult::Continue);
        }

        // Check if enough hooks voted to continue
        let continue_ratio = f64::from(continue_count) / f64::from(total_count);
        if continue_ratio >= self.threshold {
            Ok(HookResult::Continue)
        } else {
            // Return the first non-continue result
            Ok(non_continue_results
                .into_iter()
                .next()
                .unwrap_or(HookResult::Continue))
        }
    }

    fn metadata(&self) -> HookMetadata {
        self.metadata.clone()
    }

    fn should_execute(&self, context: &HookContext) -> bool {
        self.hooks.iter().any(|h| h.should_execute(context))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_hooks::types::{ComponentId, ComponentType};
    #[tokio::test]
    async fn test_middleware_creation() {
        let hook_registry = Arc::new(HookRegistry::new());
        let hook_executor = Arc::new(HookExecutor::new());

        let config = MiddlewareConfig::default();
        let mut middleware = SessionMiddleware::new(config, hook_registry, hook_executor);

        assert!(middleware.initialize().is_ok());
        assert_eq!(middleware.middleware_chains.len(), 5);
    }
    #[tokio::test]
    async fn test_middleware_execution() {
        let hook_registry = Arc::new(HookRegistry::new());
        let hook_executor = Arc::new(HookExecutor::new());

        let config = MiddlewareConfig {
            enable_logging: true,
            enable_metrics: true,
            enable_security: false,
            enable_caching: false,
            enable_rate_limiting: false,
            custom_hooks: vec![],
            pattern: MiddlewarePattern::Sequential,
        };

        let middleware =
            create_session_middleware(config, hook_registry.clone(), hook_executor.clone())
                .unwrap();

        let mut context = HookContext::new(
            HookPoint::SessionStart,
            ComponentId::new(ComponentType::Agent, "test-session".to_string()),
        );

        let result = middleware
            .execute_middleware(MiddlewareType::SessionCreate, &mut context)
            .await
            .unwrap();

        assert!(result.should_continue());
    }
    #[tokio::test]
    async fn test_parallel_middleware() {
        let hook_registry = Arc::new(HookRegistry::new());
        let hook_executor = Arc::new(HookExecutor::new());

        let config = MiddlewareConfig {
            pattern: MiddlewarePattern::Parallel,
            ..Default::default()
        };

        let mut middleware = SessionMiddleware::new(config, hook_registry, hook_executor);
        middleware.initialize().unwrap();

        // Verify parallel middleware was created
        assert!(middleware
            .middleware_chains
            .contains_key(&MiddlewareType::SessionRead));
    }
}
