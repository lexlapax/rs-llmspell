//! ABOUTME: Lifecycle middleware system for intercepting and augmenting agent lifecycle transitions
//! ABOUTME: Provides composable middleware chain for logging, metrics, security, and custom lifecycle behavior

use super::{
    events::{LifecycleEvent, LifecycleEventData, LifecycleEventSystem, LifecycleEventType},
    state_machine::StateContext,
};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

/// Middleware execution context
#[derive(Debug, Clone)]
pub struct MiddlewareContext {
    /// Request ID for tracing
    pub request_id: String,
    /// Agent ID being processed
    pub agent_id: String,
    /// Current lifecycle phase
    pub phase: LifecyclePhase,
    /// State transition context if applicable
    pub state_context: Option<StateContext>,
    /// Middleware-specific data
    pub data: HashMap<String, String>,
    /// Execution start time
    pub start_time: Instant,
}

impl MiddlewareContext {
    pub fn new(agent_id: String, phase: LifecyclePhase) -> Self {
        Self {
            request_id: Uuid::new_v4().to_string(),
            agent_id,
            phase,
            state_context: None,
            data: HashMap::new(),
            start_time: Instant::now(),
        }
    }

    pub fn with_state_context(mut self, context: StateContext) -> Self {
        self.state_context = Some(context);
        self
    }

    pub fn with_data(mut self, key: &str, value: &str) -> Self {
        self.data.insert(key.to_string(), value.to_string());
        self
    }

    pub fn get_data(&self, key: &str) -> Option<&String> {
        self.data.get(key)
    }

    pub fn set_data(&mut self, key: &str, value: &str) {
        self.data.insert(key.to_string(), value.to_string());
    }

    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
}

/// Lifecycle phases that middleware can intercept
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LifecyclePhase {
    /// Agent initialization
    Initialization,
    /// State transition
    StateTransition,
    /// Task execution
    TaskExecution,
    /// Resource allocation
    ResourceAllocation,
    /// Resource deallocation
    ResourceDeallocation,
    /// Health check
    HealthCheck,
    /// Shutdown
    Shutdown,
    /// Error handling
    ErrorHandling,
    /// Custom phase
    Custom(String),
}

impl LifecyclePhase {
    pub fn name(&self) -> String {
        match self {
            LifecyclePhase::Initialization => "initialization".to_string(),
            LifecyclePhase::StateTransition => "state_transition".to_string(),
            LifecyclePhase::TaskExecution => "task_execution".to_string(),
            LifecyclePhase::ResourceAllocation => "resource_allocation".to_string(),
            LifecyclePhase::ResourceDeallocation => "resource_deallocation".to_string(),
            LifecyclePhase::HealthCheck => "health_check".to_string(),
            LifecyclePhase::Shutdown => "shutdown".to_string(),
            LifecyclePhase::ErrorHandling => "error_handling".to_string(),
            LifecyclePhase::Custom(name) => name.clone(),
        }
    }
}

/// Lifecycle middleware trait
#[async_trait]
pub trait LifecycleMiddleware: Send + Sync {
    /// Called before the lifecycle phase executes
    async fn before(&self, context: &mut MiddlewareContext) -> Result<()>;

    /// Called after the lifecycle phase executes successfully
    async fn after(&self, context: &mut MiddlewareContext) -> Result<()>;

    /// Called if the lifecycle phase encounters an error
    async fn on_error(&self, context: &mut MiddlewareContext, error: &anyhow::Error) -> Result<()>;

    /// Get middleware name for identification
    fn name(&self) -> String;

    /// Get execution priority (lower numbers execute first)
    fn priority(&self) -> u8 {
        50
    }

    /// Check if middleware should run for given phase
    fn applies_to(&self, phase: LifecyclePhase) -> bool;

    /// Check if middleware is enabled
    fn is_enabled(&self) -> bool {
        true
    }
}

/// Middleware execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiddlewareExecutionResult {
    /// Middleware name
    pub middleware_name: String,
    /// Phase being processed
    pub phase: LifecyclePhase,
    /// Whether execution was successful
    pub success: bool,
    /// Execution duration
    pub duration: Duration,
    /// Error message if failed
    pub error: Option<String>,
    /// Additional metrics
    pub metrics: HashMap<String, f64>,
}

/// Middleware chain executor
pub struct LifecycleMiddlewareChain {
    /// Registered middleware in priority order
    middleware: Arc<RwLock<Vec<Arc<dyn LifecycleMiddleware>>>>,
    /// Event system for notifications
    event_system: Arc<LifecycleEventSystem>,
    /// Configuration
    config: MiddlewareConfig,
    /// Execution history
    execution_history: Arc<RwLock<Vec<MiddlewareExecutionResult>>>,
}

/// Middleware configuration
#[derive(Debug, Clone)]
pub struct MiddlewareConfig {
    /// Enable middleware execution
    pub enabled: bool,
    /// Maximum execution time per middleware
    pub max_execution_time: Duration,
    /// Enable detailed logging
    pub enable_logging: bool,
    /// Continue chain on middleware errors
    pub continue_on_error: bool,
    /// Maximum history size
    pub max_history_size: usize,
    /// Emit events for middleware execution
    pub emit_events: bool,
}

impl Default for MiddlewareConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_execution_time: Duration::from_secs(5),
            enable_logging: true,
            continue_on_error: false,
            max_history_size: 1000,
            emit_events: false, // Usually too verbose
        }
    }
}

impl LifecycleMiddlewareChain {
    /// Create new middleware chain
    pub fn new(event_system: Arc<LifecycleEventSystem>, config: MiddlewareConfig) -> Self {
        Self {
            middleware: Arc::new(RwLock::new(Vec::new())),
            event_system,
            config,
            execution_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Add middleware to the chain
    pub async fn add_middleware(&self, middleware: Arc<dyn LifecycleMiddleware>) {
        let mut chain = self.middleware.write().await;
        chain.push(middleware);

        // Sort by priority
        chain.sort_by_key(|m| m.priority());

        if self.config.enable_logging {
            debug!(
                "Added middleware '{}' with priority {}",
                chain.last().unwrap().name(),
                chain.last().unwrap().priority()
            );
        }
    }

    /// Execute middleware chain for a lifecycle phase
    pub async fn execute(&self, mut context: MiddlewareContext) -> Result<MiddlewareContext> {
        if !self.config.enabled {
            return Ok(context);
        }

        let start_time = Instant::now();
        let middleware_list = {
            let chain = self.middleware.read().await;
            chain
                .iter()
                .filter(|m| m.is_enabled() && m.applies_to(context.phase.clone()))
                .cloned()
                .collect::<Vec<_>>()
        };

        if self.config.enable_logging && !middleware_list.is_empty() {
            debug!(
                "Executing {} middleware for phase {:?} on agent {}",
                middleware_list.len(),
                context.phase,
                context.agent_id
            );
        }

        // Execute before hooks
        for middleware in &middleware_list {
            let execution_result = self
                .execute_middleware_before(middleware.as_ref(), &mut context)
                .await;
            self.record_execution(execution_result).await;

            if !self.config.continue_on_error && context.get_data("_error").is_some() {
                break;
            }
        }

        // If we're not in error state, execute after hooks in reverse order
        if context.get_data("_error").is_none() {
            for middleware in middleware_list.iter().rev() {
                let execution_result = self
                    .execute_middleware_after(middleware.as_ref(), &mut context)
                    .await;
                self.record_execution(execution_result).await;

                if !self.config.continue_on_error && context.get_data("_error").is_some() {
                    break;
                }
            }
        }

        // Emit event if configured
        if self.config.emit_events {
            let event = LifecycleEvent::new(
                LifecycleEventType::ExecutionCompleted,
                context.agent_id.clone(),
                LifecycleEventData::Generic {
                    message: format!("Middleware chain executed for phase {:?}", context.phase),
                    details: HashMap::from([
                        ("phase".to_string(), context.phase.name()),
                        (
                            "middleware_count".to_string(),
                            middleware_list.len().to_string(),
                        ),
                        (
                            "duration_ms".to_string(),
                            start_time.elapsed().as_millis().to_string(),
                        ),
                    ]),
                },
                "middleware_chain".to_string(),
            );

            if let Err(e) = self.event_system.emit(event).await {
                warn!("Failed to emit middleware execution event: {}", e);
            }
        }

        Ok(context)
    }

    /// Execute middleware error handlers
    pub async fn handle_error(
        &self,
        mut context: MiddlewareContext,
        error: &anyhow::Error,
    ) -> MiddlewareContext {
        if !self.config.enabled {
            return context;
        }

        let middleware_list = {
            let chain = self.middleware.read().await;
            chain
                .iter()
                .filter(|m| m.is_enabled() && m.applies_to(context.phase.clone()))
                .cloned()
                .collect::<Vec<_>>()
        };

        for middleware in &middleware_list {
            let execution_result = self
                .execute_middleware_error(middleware.as_ref(), &mut context, error)
                .await;
            self.record_execution(execution_result).await;
        }

        context
    }

    /// Execute single middleware before hook
    async fn execute_middleware_before(
        &self,
        middleware: &dyn LifecycleMiddleware,
        context: &mut MiddlewareContext,
    ) -> MiddlewareExecutionResult {
        let start_time = Instant::now();
        let middleware_name = middleware.name();

        match tokio::time::timeout(self.config.max_execution_time, middleware.before(context)).await
        {
            Ok(Ok(())) => MiddlewareExecutionResult {
                middleware_name,
                phase: context.phase.clone(),
                success: true,
                duration: start_time.elapsed(),
                error: None,
                metrics: HashMap::new(),
            },
            Ok(Err(e)) => {
                if self.config.enable_logging {
                    error!("Middleware '{}' before hook failed: {}", middleware_name, e);
                }
                context.set_data("_error", &e.to_string());
                MiddlewareExecutionResult {
                    middleware_name,
                    phase: context.phase.clone(),
                    success: false,
                    duration: start_time.elapsed(),
                    error: Some(e.to_string()),
                    metrics: HashMap::new(),
                }
            }
            Err(_) => {
                let error_msg = format!("Middleware '{}' before hook timed out", middleware_name);
                if self.config.enable_logging {
                    error!("{}", error_msg);
                }
                context.set_data("_error", &error_msg);
                MiddlewareExecutionResult {
                    middleware_name,
                    phase: context.phase.clone(),
                    success: false,
                    duration: start_time.elapsed(),
                    error: Some(error_msg),
                    metrics: HashMap::new(),
                }
            }
        }
    }

    /// Execute single middleware after hook
    async fn execute_middleware_after(
        &self,
        middleware: &dyn LifecycleMiddleware,
        context: &mut MiddlewareContext,
    ) -> MiddlewareExecutionResult {
        let start_time = Instant::now();
        let middleware_name = middleware.name();

        match tokio::time::timeout(self.config.max_execution_time, middleware.after(context)).await
        {
            Ok(Ok(())) => MiddlewareExecutionResult {
                middleware_name,
                phase: context.phase.clone(),
                success: true,
                duration: start_time.elapsed(),
                error: None,
                metrics: HashMap::new(),
            },
            Ok(Err(e)) => {
                if self.config.enable_logging {
                    error!("Middleware '{}' after hook failed: {}", middleware_name, e);
                }
                MiddlewareExecutionResult {
                    middleware_name,
                    phase: context.phase.clone(),
                    success: false,
                    duration: start_time.elapsed(),
                    error: Some(e.to_string()),
                    metrics: HashMap::new(),
                }
            }
            Err(_) => {
                let error_msg = format!("Middleware '{}' after hook timed out", middleware_name);
                if self.config.enable_logging {
                    error!("{}", error_msg);
                }
                MiddlewareExecutionResult {
                    middleware_name,
                    phase: context.phase.clone(),
                    success: false,
                    duration: start_time.elapsed(),
                    error: Some(error_msg),
                    metrics: HashMap::new(),
                }
            }
        }
    }

    /// Execute single middleware error hook
    async fn execute_middleware_error(
        &self,
        middleware: &dyn LifecycleMiddleware,
        context: &mut MiddlewareContext,
        error: &anyhow::Error,
    ) -> MiddlewareExecutionResult {
        let start_time = Instant::now();
        let middleware_name = middleware.name();

        match tokio::time::timeout(
            self.config.max_execution_time,
            middleware.on_error(context, error),
        )
        .await
        {
            Ok(Ok(())) => MiddlewareExecutionResult {
                middleware_name,
                phase: context.phase.clone(),
                success: true,
                duration: start_time.elapsed(),
                error: None,
                metrics: HashMap::new(),
            },
            Ok(Err(e)) => {
                if self.config.enable_logging {
                    error!("Middleware '{}' error hook failed: {}", middleware_name, e);
                }
                MiddlewareExecutionResult {
                    middleware_name,
                    phase: context.phase.clone(),
                    success: false,
                    duration: start_time.elapsed(),
                    error: Some(e.to_string()),
                    metrics: HashMap::new(),
                }
            }
            Err(_) => {
                let error_msg = format!("Middleware '{}' error hook timed out", middleware_name);
                if self.config.enable_logging {
                    error!("{}", error_msg);
                }
                MiddlewareExecutionResult {
                    middleware_name,
                    phase: context.phase.clone(),
                    success: false,
                    duration: start_time.elapsed(),
                    error: Some(error_msg),
                    metrics: HashMap::new(),
                }
            }
        }
    }

    /// Record middleware execution result
    async fn record_execution(&self, result: MiddlewareExecutionResult) {
        let mut history = self.execution_history.write().await;
        history.push(result);

        if history.len() > self.config.max_history_size {
            history.remove(0);
        }
    }

    /// Get execution history
    pub async fn get_execution_history(&self) -> Vec<MiddlewareExecutionResult> {
        let history = self.execution_history.read().await;
        history.clone()
    }

    /// Get middleware count
    pub async fn get_middleware_count(&self) -> usize {
        let chain = self.middleware.read().await;
        chain.len()
    }

    /// Clear middleware chain
    pub async fn clear(&self) {
        let mut chain = self.middleware.write().await;
        chain.clear();
    }
}

/// Built-in middleware implementations
/// Logging middleware
pub struct LoggingMiddleware {
    log_level: tracing::Level,
}

impl Default for LoggingMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl LoggingMiddleware {
    pub fn new() -> Self {
        Self {
            log_level: tracing::Level::DEBUG,
        }
    }

    pub fn with_level(mut self, level: tracing::Level) -> Self {
        self.log_level = level;
        self
    }
}

#[async_trait]
impl LifecycleMiddleware for LoggingMiddleware {
    async fn before(&self, context: &mut MiddlewareContext) -> Result<()> {
        match self.log_level {
            tracing::Level::DEBUG => debug!(
                "Agent {} starting phase {:?}",
                context.agent_id, context.phase
            ),
            tracing::Level::INFO => info!(
                "Agent {} starting phase {:?}",
                context.agent_id, context.phase
            ),
            _ => {}
        }
        Ok(())
    }

    async fn after(&self, context: &mut MiddlewareContext) -> Result<()> {
        match self.log_level {
            tracing::Level::DEBUG => debug!(
                "Agent {} completed phase {:?} in {:?}",
                context.agent_id,
                context.phase,
                context.elapsed()
            ),
            tracing::Level::INFO => info!(
                "Agent {} completed phase {:?} in {:?}",
                context.agent_id,
                context.phase,
                context.elapsed()
            ),
            _ => {}
        }
        Ok(())
    }

    async fn on_error(&self, context: &mut MiddlewareContext, error: &anyhow::Error) -> Result<()> {
        error!(
            "Agent {} failed in phase {:?} after {:?}: {}",
            context.agent_id,
            context.phase,
            context.elapsed(),
            error
        );
        Ok(())
    }

    fn name(&self) -> String {
        "logging".to_string()
    }

    fn priority(&self) -> u8 {
        100 // Low priority, run last
    }

    fn applies_to(&self, _phase: LifecyclePhase) -> bool {
        true // Log all phases
    }
}

/// Metrics collection middleware
pub struct MetricsMiddleware {
    metrics: Arc<RwLock<HashMap<String, f64>>>,
}

impl Default for MetricsMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsMiddleware {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get_metrics(&self) -> HashMap<String, f64> {
        let metrics = self.metrics.read().await;
        metrics.clone()
    }
}

#[async_trait]
impl LifecycleMiddleware for MetricsMiddleware {
    async fn before(&self, context: &mut MiddlewareContext) -> Result<()> {
        let key = format!("{}_{}_started", context.agent_id, context.phase.name());
        let mut metrics = self.metrics.write().await;
        *metrics.entry(key).or_insert(0.0) += 1.0;
        Ok(())
    }

    async fn after(&self, context: &mut MiddlewareContext) -> Result<()> {
        let duration_key = format!("{}_{}_duration_ms", context.agent_id, context.phase.name());
        let completed_key = format!("{}_{}_completed", context.agent_id, context.phase.name());

        let mut metrics = self.metrics.write().await;
        metrics.insert(duration_key, context.elapsed().as_millis() as f64);
        *metrics.entry(completed_key).or_insert(0.0) += 1.0;
        Ok(())
    }

    async fn on_error(
        &self,
        context: &mut MiddlewareContext,
        _error: &anyhow::Error,
    ) -> Result<()> {
        let key = format!("{}_{}_errors", context.agent_id, context.phase.name());
        let mut metrics = self.metrics.write().await;
        *metrics.entry(key).or_insert(0.0) += 1.0;
        Ok(())
    }

    fn name(&self) -> String {
        "metrics".to_string()
    }

    fn priority(&self) -> u8 {
        10 // High priority
    }

    fn applies_to(&self, _phase: LifecyclePhase) -> bool {
        true // Collect metrics for all phases
    }
}

/// Security validation middleware
pub struct SecurityMiddleware {
    trusted_agents: Vec<String>,
    security_policies: HashMap<LifecyclePhase, Vec<String>>,
}

impl Default for SecurityMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

impl SecurityMiddleware {
    pub fn new() -> Self {
        Self {
            trusted_agents: Vec::new(),
            security_policies: HashMap::new(),
        }
    }

    pub fn with_trusted_agents(mut self, agents: Vec<String>) -> Self {
        self.trusted_agents = agents;
        self
    }

    pub fn with_policy(mut self, phase: LifecyclePhase, policies: Vec<String>) -> Self {
        self.security_policies.insert(phase, policies);
        self
    }
}

#[async_trait]
impl LifecycleMiddleware for SecurityMiddleware {
    async fn before(&self, context: &mut MiddlewareContext) -> Result<()> {
        // Check if agent is trusted
        if !self.trusted_agents.contains(&context.agent_id) {
            // Apply security policies for untrusted agents
            if let Some(policies) = self.security_policies.get(&context.phase) {
                for policy in policies {
                    match policy.as_str() {
                        "require_auth" => {
                            if context.get_data("auth_token").is_none() {
                                return Err(anyhow!("Authentication required for untrusted agent"));
                            }
                        }
                        "limit_resources" => {
                            context.set_data("resource_limit", "restricted");
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    async fn after(&self, _context: &mut MiddlewareContext) -> Result<()> {
        Ok(())
    }

    async fn on_error(&self, context: &mut MiddlewareContext, error: &anyhow::Error) -> Result<()> {
        warn!(
            "Security middleware detected error in agent {} during phase {:?}: {}",
            context.agent_id, context.phase, error
        );
        Ok(())
    }

    fn name(&self) -> String {
        "security".to_string()
    }

    fn priority(&self) -> u8 {
        5 // Very high priority
    }

    fn applies_to(&self, phase: LifecyclePhase) -> bool {
        // Apply security to sensitive phases
        matches!(
            phase,
            LifecyclePhase::Initialization
                | LifecyclePhase::TaskExecution
                | LifecyclePhase::ResourceAllocation
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lifecycle::events::EventSystemConfig;

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_middleware_chain_basic() {
        let event_system = Arc::new(LifecycleEventSystem::new(EventSystemConfig::default()));
        let chain = LifecycleMiddlewareChain::new(event_system, MiddlewareConfig::default());

        chain
            .add_middleware(Arc::new(LoggingMiddleware::new()))
            .await;
        chain
            .add_middleware(Arc::new(MetricsMiddleware::new()))
            .await;

        let context =
            MiddlewareContext::new("test-agent".to_string(), LifecyclePhase::Initialization);

        let result = chain.execute(context).await.unwrap();
        assert_eq!(result.agent_id, "test-agent");
        assert_eq!(result.phase, LifecyclePhase::Initialization);
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_middleware_priority_ordering() {
        let event_system = Arc::new(LifecycleEventSystem::new(EventSystemConfig::default()));
        let chain = LifecycleMiddlewareChain::new(event_system, MiddlewareConfig::default());

        // Add middleware in reverse priority order
        chain
            .add_middleware(Arc::new(LoggingMiddleware::new()))
            .await; // Priority 100
        chain
            .add_middleware(Arc::new(MetricsMiddleware::new()))
            .await; // Priority 10
        chain
            .add_middleware(Arc::new(SecurityMiddleware::new()))
            .await; // Priority 5

        assert_eq!(chain.get_middleware_count().await, 3);
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_metrics_middleware() {
        let metrics = Arc::new(MetricsMiddleware::new());
        let mut context =
            MiddlewareContext::new("test-agent".to_string(), LifecyclePhase::Initialization);

        metrics.before(&mut context).await.unwrap();
        metrics.after(&mut context).await.unwrap();

        let collected_metrics = metrics.get_metrics().await;
        assert!(collected_metrics.contains_key("test-agent_initialization_started"));
        assert!(collected_metrics.contains_key("test-agent_initialization_completed"));
        assert!(collected_metrics.contains_key("test-agent_initialization_duration_ms"));
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_security_middleware() {
        let security = SecurityMiddleware::new()
            .with_trusted_agents(vec!["trusted-agent".to_string()])
            .with_policy(
                LifecyclePhase::Initialization,
                vec!["require_auth".to_string()],
            );

        // Trusted agent should pass
        let mut trusted_context =
            MiddlewareContext::new("trusted-agent".to_string(), LifecyclePhase::Initialization);
        assert!(security.before(&mut trusted_context).await.is_ok());

        // Untrusted agent without auth should fail
        let mut untrusted_context = MiddlewareContext::new(
            "untrusted-agent".to_string(),
            LifecyclePhase::Initialization,
        );
        assert!(security.before(&mut untrusted_context).await.is_err());

        // Untrusted agent with auth should pass
        let mut authed_context = MiddlewareContext::new(
            "untrusted-agent".to_string(),
            LifecyclePhase::Initialization,
        )
        .with_data("auth_token", "valid_token");
        assert!(security.before(&mut authed_context).await.is_ok());
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_middleware_error_handling() {
        struct FailingMiddleware;

        #[async_trait]
        impl LifecycleMiddleware for FailingMiddleware {
            async fn before(&self, _context: &mut MiddlewareContext) -> Result<()> {
                Err(anyhow!("Test error"))
            }

            async fn after(&self, _context: &mut MiddlewareContext) -> Result<()> {
                Ok(())
            }

            async fn on_error(
                &self,
                _context: &mut MiddlewareContext,
                _error: &anyhow::Error,
            ) -> Result<()> {
                Ok(())
            }

            fn name(&self) -> String {
                "failing".to_string()
            }

            fn applies_to(&self, _phase: LifecyclePhase) -> bool {
                true
            }
        }

        let event_system = Arc::new(LifecycleEventSystem::new(EventSystemConfig::default()));
        let config = MiddlewareConfig {
            continue_on_error: false,
            ..Default::default()
        };
        let chain = LifecycleMiddlewareChain::new(event_system, config);

        chain.add_middleware(Arc::new(FailingMiddleware)).await;

        let context =
            MiddlewareContext::new("test-agent".to_string(), LifecyclePhase::Initialization);

        let result = chain.execute(context).await.unwrap();
        // Should have error data
        assert!(result.get_data("_error").is_some());
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_lifecycle_phase_names() {
        assert_eq!(LifecyclePhase::Initialization.name(), "initialization");
        assert_eq!(LifecyclePhase::StateTransition.name(), "state_transition");
        assert_eq!(LifecyclePhase::Custom("test".to_string()).name(), "test");
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_middleware_context() {
        let mut context =
            MiddlewareContext::new("test-agent".to_string(), LifecyclePhase::Initialization)
                .with_data("test_key", "test_value");

        assert_eq!(context.agent_id, "test-agent");
        assert_eq!(context.phase, LifecyclePhase::Initialization);
        assert_eq!(
            context.get_data("test_key"),
            Some(&"test_value".to_string())
        );

        context.set_data("new_key", "new_value");
        assert_eq!(context.get_data("new_key"), Some(&"new_value".to_string()));

        assert!(context.elapsed() >= Duration::from_nanos(0));
    }
}
