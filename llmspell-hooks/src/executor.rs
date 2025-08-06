// ABOUTME: HookExecutor implementation with automatic performance protection via CircuitBreaker
// ABOUTME: Manages hook execution with monitoring, error handling, and performance guarantees

use crate::circuit_breaker::{BreakerConfig, CircuitBreaker, CircuitBreakerManager};
use crate::context::HookContext;
use crate::performance::{PerformanceConfig, PerformanceMetrics, PerformanceMonitor};
use crate::persistence::HookPersistenceManager;
use crate::result::HookResult;
use crate::traits::Hook;
use anyhow::Result;
use llmspell_events::{EventCorrelationTracker, EventLink, EventRelationship, UniversalEvent};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{info, warn};

/// Hook execution configuration
#[derive(Debug, Clone)]
pub struct HookExecutorConfig {
    /// Enable circuit breaker protection
    pub enable_circuit_breaker: bool,
    /// Enable performance monitoring
    pub enable_performance_monitoring: bool,
    /// Enable hook persistence
    pub enable_persistence: bool,
    /// Maximum execution time before circuit breaker triggers
    pub max_execution_time: Duration,
    /// Performance overhead target (e.g., 5%)
    pub performance_overhead_target: f64,
    /// Circuit breaker configuration
    pub breaker_config: BreakerConfig,
    /// Performance monitoring configuration
    pub performance_config: PerformanceConfig,
}

impl Default for HookExecutorConfig {
    fn default() -> Self {
        Self {
            enable_circuit_breaker: true,
            enable_performance_monitoring: true,
            enable_persistence: false, // Opt-in for backward compatibility
            max_execution_time: Duration::from_millis(100),
            performance_overhead_target: 0.05, // 5%
            breaker_config: BreakerConfig::default(),
            performance_config: PerformanceConfig::default(),
        }
    }
}

/// Hook executor with performance protection
pub struct HookExecutor {
    config: HookExecutorConfig,
    circuit_breakers: Arc<CircuitBreakerManager>,
    performance_monitor: Arc<PerformanceMonitor>,
    hook_configs: Arc<RwLock<HashMap<String, HookExecutionConfig>>>,
    persistence_manager: Option<Arc<HookPersistenceManager>>,
    correlation_tracker: Option<Arc<EventCorrelationTracker>>,
}

/// Per-hook execution configuration
#[derive(Debug, Clone)]
pub struct HookExecutionConfig {
    /// Custom timeout for this hook
    pub timeout: Option<Duration>,
    /// Whether circuit breaker is enabled for this hook
    pub use_circuit_breaker: bool,
    /// Custom circuit breaker config
    pub breaker_config: Option<BreakerConfig>,
    /// Whether to persist this hook's executions
    pub persist_executions: bool,
    /// Retention priority for persistence
    pub retention_priority: i32,
}

impl Default for HookExecutionConfig {
    fn default() -> Self {
        Self {
            timeout: None,
            use_circuit_breaker: true,
            breaker_config: None,
            persist_executions: true,
            retention_priority: 0,
        }
    }
}

impl HookExecutor {
    /// Create a new hook executor
    pub fn new() -> Self {
        Self::with_config(HookExecutorConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(config: HookExecutorConfig) -> Self {
        let circuit_breakers = Arc::new(CircuitBreakerManager::with_config(
            config.breaker_config.clone(),
        ));

        let performance_monitor = Arc::new(PerformanceMonitor::with_config(
            config.performance_config.clone(),
        ));

        Self {
            config,
            circuit_breakers,
            performance_monitor,
            hook_configs: Arc::new(RwLock::new(HashMap::new())),
            persistence_manager: None, // Set later via set_persistence_manager
            correlation_tracker: None, // Set later via set_correlation_tracker
        }
    }

    /// Execute a hook with protection
    pub async fn execute_hook(
        &self,
        hook: &dyn Hook,
        context: &mut HookContext,
    ) -> Result<HookResult> {
        // OPTIMIZATION: Cache metadata to avoid repeated calls
        let metadata = hook.metadata();
        let hook_name = &metadata.name; // Use reference instead of clone

        // Check if hook should execute
        if !hook.should_execute(context) {
            return Ok(HookResult::Skipped("Hook conditions not met".to_string()));
        }

        // OPTIMIZATION: Get config and circuit breaker in single operation
        let (hook_config, breaker_opt) = {
            let configs = self.hook_configs.read();
            let config = configs.get(hook_name).cloned().unwrap_or_default();
            let breaker = if self.config.enable_circuit_breaker && config.use_circuit_breaker {
                Some(self.get_circuit_breaker(hook_name, &config))
            } else {
                None
            };
            (config, breaker)
        };

        // Check circuit breaker if enabled
        if let Some(ref breaker) = breaker_opt {
            if !breaker.can_execute() {
                warn!("Circuit breaker open for hook: {}", hook_name);
                return Ok(HookResult::Skipped(format!(
                    "Circuit breaker open for hook: {}",
                    hook_name
                )));
            }
        }

        // OPTIMIZATION: Combine timer start and instant measurement
        let (timer, start) = if self.config.enable_performance_monitoring {
            let start = Instant::now();
            let timer = Some(self.performance_monitor.start_execution(hook_name));
            (timer, start)
        } else {
            (None, Instant::now())
        };

        // Create hook execution event for correlation tracking
        let execution_event = if let Some(ref tracker) = self.correlation_tracker {
            let event_data = serde_json::json!({
                "hook_name": hook_name,
                "hook_language": metadata.language,
                "component_id": context.component_id.name,
                "component_type": context.component_id.component_type,
                "hook_point": context.point,
                "operation": "hook_execution_start"
            });

            let event = UniversalEvent::new(
                "hook.execution.start",
                event_data,
                llmspell_events::Language::Rust,
            )
            .with_correlation_id(context.correlation_id)
            .with_source("hook_executor")
            .with_tag("hook_execution");

            tracker.track_event(event.clone());
            Some(event)
        } else {
            None
        };

        // Execute the hook
        let result = hook.execute(context).await;

        let duration = start.elapsed();

        // Create hook completion event and link to start event
        if let Some(ref tracker) = self.correlation_tracker {
            let success = result.is_ok();
            let event_data = serde_json::json!({
                "hook_name": hook_name,
                "success": success,
                "duration_ms": duration.as_millis(),
                "operation": "hook_execution_complete"
            });

            let completion_event = UniversalEvent::new(
                "hook.execution.complete",
                event_data,
                llmspell_events::Language::Rust,
            )
            .with_correlation_id(context.correlation_id)
            .with_source("hook_executor")
            .with_tag("hook_execution");

            tracker.track_event(completion_event.clone());

            // Create link between start and completion events
            if let Some(start_event) = execution_event {
                let link = EventLink::new(
                    start_event.id,
                    completion_event.id,
                    EventRelationship::PartOf,
                )
                .with_strength(1.0)
                .with_metadata("relationship", "execution_pair")
                .with_metadata("hook_name", hook_name.to_string());

                tracker.add_link(link);
            }
        }

        // Complete performance tracking
        if let Some(timer) = timer {
            timer.complete();
        }

        // OPTIMIZATION: Update circuit breaker using cached reference
        if let Some(breaker) = breaker_opt {
            match &result {
                Ok(_) => breaker.record_success(duration),
                Err(e) => breaker.record_failure(e),
            }
        }

        // OPTIMIZATION: Pre-compute timeout check
        let timeout = hook_config
            .timeout
            .unwrap_or(self.config.max_execution_time);
        if duration > timeout {
            warn!(
                "Hook {} execution took {:?}, exceeding timeout of {:?}",
                hook_name, duration, timeout
            );
        }

        // Persist hook execution if enabled
        if self.config.enable_persistence && hook_config.persist_executions {
            if let Some(ref _persistence_manager) = self.persistence_manager {
                // For now, we can't persist hooks that don't implement ReplayableHook
                // This would require all hooks to implement ReplayableHook or a different approach
                // TODO: Consider alternative persistence strategies
                /*
                if let Ok(ref hook_result) = result {
                        // Create hook metadata
                        let mut metadata = HookMetadata::new(
                            hook_name.clone(),
                            context.component_id.component_type.clone(),
                            context.component_id.name.clone(),
                        );

                        // Set metadata properties
                        metadata.retention_priority = hook_config.retention_priority;
                        metadata.modified_operation = matches!(
                            hook_result,
                            HookResult::Modified(_) | HookResult::Replace(_)
                        );

                        // Add tags based on result
                        match hook_result {
                            HookResult::Cancel(_) => metadata.add_tag("cancelled".to_string()),
                            HookResult::Replace(_) => metadata.add_tag("replaced".to_string()),
                            HookResult::Modified(_) => metadata.add_tag("modified".to_string()),
                            HookResult::Skipped(_) => metadata.add_tag("skipped".to_string()),
                            _ => {}
                        }

                        // Add performance tags if slow
                        if duration > timeout {
                            metadata.add_tag("slow_execution".to_string());
                        }

                        // Calculate context size
                        if let Ok(serialized) = replayable_hook.serialize_context(context) {
                            metadata.context_size = serialized.len();
                        }

                        // Persist the execution
                        if let Err(e) = persistence_manager
                            .persist_execution(
                                replayable_hook,
                                context,
                                hook_result,
                                duration,
                                metadata,
                            )
                            .await
                        {
                            debug!(
                                "Failed to persist hook execution for {}: {}",
                                hook_name, e
                            );
                        }
                    }
                }
                */
            }
        }

        result
    }

    /// Execute multiple hooks in sequence
    pub async fn execute_hooks(
        &self,
        hooks: &[Arc<dyn Hook>],
        context: &mut HookContext,
    ) -> Result<Vec<HookResult>> {
        let mut results = Vec::with_capacity(hooks.len());

        // Create batch execution event for correlation tracking
        if let Some(ref tracker) = self.correlation_tracker {
            let event_data = serde_json::json!({
                "hook_count": hooks.len(),
                "component_id": context.component_id.name,
                "component_type": context.component_id.component_type,
                "hook_point": context.point,
                "operation": "batch_hook_execution_start"
            });

            let batch_event = UniversalEvent::new(
                "hook.batch_execution.start",
                event_data,
                llmspell_events::Language::Rust,
            )
            .with_correlation_id(context.correlation_id)
            .with_source("hook_executor")
            .with_tag("batch_execution");

            tracker.track_event(batch_event);
        }

        for hook in hooks {
            let result = self.execute_hook(hook.as_ref(), context).await?;

            // Check if we should stop execution
            match &result {
                HookResult::Cancel(reason) => {
                    info!("Hook execution cancelled: {}", reason);
                    results.push(result);
                    break;
                }
                HookResult::Replace(_) => {
                    // Replace stops further execution
                    results.push(result);
                    break;
                }
                _ => {
                    results.push(result);
                }
            }
        }

        // Create batch completion event for correlation tracking
        if let Some(ref tracker) = self.correlation_tracker {
            let event_data = serde_json::json!({
                "hook_count": hooks.len(),
                "results_count": results.len(),
                "operation": "batch_hook_execution_complete"
            });

            let batch_completion_event = UniversalEvent::new(
                "hook.batch_execution.complete",
                event_data,
                llmspell_events::Language::Rust,
            )
            .with_correlation_id(context.correlation_id)
            .with_source("hook_executor")
            .with_tag("batch_execution");

            tracker.track_event(batch_completion_event);
        }

        Ok(results)
    }

    /// Configure a specific hook
    pub fn configure_hook(&self, hook_name: &str, config: HookExecutionConfig) {
        self.hook_configs
            .write()
            .insert(hook_name.to_string(), config);
    }

    /// Get circuit breaker for a hook
    fn get_circuit_breaker(
        &self,
        hook_name: &str,
        hook_config: &HookExecutionConfig,
    ) -> Arc<CircuitBreaker> {
        if let Some(custom_config) = &hook_config.breaker_config {
            self.circuit_breakers
                .create_custom(hook_name, custom_config.clone())
        } else {
            self.circuit_breakers.get_or_create(hook_name)
        }
    }

    /// Get performance metrics for a hook
    pub fn get_metrics(&self, hook_name: &str) -> Option<PerformanceMetrics> {
        self.performance_monitor.get_metrics(hook_name)
    }

    /// Get all performance metrics
    pub fn get_all_metrics(&self) -> HashMap<String, PerformanceMetrics> {
        self.performance_monitor.get_all_metrics()
    }

    /// Check if overhead is within target
    pub fn is_within_overhead_target(&self) -> bool {
        let metrics = self.get_all_metrics();
        if metrics.is_empty() {
            return true;
        }

        let total_hook_time: Duration = metrics.values().map(|m| m.total_duration).sum();

        let total_executions: u64 = metrics.values().map(|m| m.execution_count).sum();

        if total_executions == 0 {
            return true;
        }

        #[allow(clippy::cast_precision_loss)]
        let avg_hook_time = total_hook_time.as_secs_f64() / total_executions as f64;
        let overhead_ratio = avg_hook_time / self.config.max_execution_time.as_secs_f64();

        overhead_ratio <= self.config.performance_overhead_target
    }

    /// Reset circuit breaker for a hook
    pub fn reset_circuit_breaker(&self, hook_name: &str) {
        let breaker = self.circuit_breakers.get_or_create(hook_name);
        breaker.reset();
    }

    /// Reset all circuit breakers
    pub fn reset_all_circuit_breakers(&self) {
        self.circuit_breakers.reset_all();
    }

    /// Get circuit breaker stats
    pub fn get_circuit_breaker_stats(
        &self,
    ) -> Vec<(String, crate::circuit_breaker::CircuitBreakerStats)> {
        self.circuit_breakers.all_stats()
    }

    /// Generate performance report
    pub fn generate_performance_report(&self) -> crate::performance::PerformanceReport {
        self.performance_monitor
            .generate_report(self.config.max_execution_time)
    }

    /// Set the persistence manager for hook execution history
    pub fn set_persistence_manager(&mut self, manager: Arc<HookPersistenceManager>) {
        self.persistence_manager = Some(manager);
    }

    /// Check if persistence is enabled
    pub fn is_persistence_enabled(&self) -> bool {
        self.config.enable_persistence && self.persistence_manager.is_some()
    }

    /// Set the correlation tracker for hook execution correlation
    pub fn set_correlation_tracker(&mut self, tracker: Arc<EventCorrelationTracker>) {
        self.correlation_tracker = Some(tracker);
    }

    /// Check if correlation tracking is enabled
    pub fn is_correlation_tracking_enabled(&self) -> bool {
        self.correlation_tracker.is_some()
    }

    /// Get the correlation tracker
    pub fn correlation_tracker(&self) -> Option<Arc<EventCorrelationTracker>> {
        self.correlation_tracker.clone()
    }

    /// Execute hooks for artifact events
    pub async fn execute_artifact_hooks(
        &self,
        artifact_event: &llmspell_core::events::ArtifactEvent,
        hooks: &[Arc<dyn Hook>],
    ) -> Result<Vec<HookResult>> {
        use crate::artifact_hooks::{add_artifact_to_context, event_to_hook_point};

        // Convert artifact event to hook point
        let hook_point = event_to_hook_point(artifact_event);

        // Create context for artifact hooks
        let component_id = crate::types::ComponentId::new(
            crate::types::ComponentType::Custom("artifact_manager".to_string()),
            artifact_event.source.to_string(),
        );

        let mut context = HookContext::new(hook_point, component_id);
        if let Some(correlation_id) = artifact_event.metadata.correlation_id() {
            // Parse UUID from string
            if let Ok(uuid) = uuid::Uuid::parse_str(correlation_id) {
                context = context.with_correlation_id(uuid);
            }
        }

        // Add artifact-specific metadata
        match &artifact_event.event_type {
            llmspell_core::events::ArtifactEventType::Created(e) => {
                add_artifact_to_context(&mut context, &e.artifact.id, "created");
                context.insert_metadata("artifact_name".to_string(), e.artifact.name.clone());
                context.insert_metadata(
                    "artifact_type".to_string(),
                    e.artifact.artifact_type.clone(),
                );
            }
            llmspell_core::events::ArtifactEventType::Modified(e) => {
                add_artifact_to_context(&mut context, &e.artifact_id, "modified");
                context.insert_metadata(
                    "modifications".to_string(),
                    format!("{:?}", e.modifications),
                );
            }
            llmspell_core::events::ArtifactEventType::Deleted(e) => {
                add_artifact_to_context(&mut context, &e.artifact_id, "deleted");
                context.insert_metadata("permanent".to_string(), e.permanent.to_string());
            }
            llmspell_core::events::ArtifactEventType::Validated(e) => {
                add_artifact_to_context(&mut context, &e.artifact_id, "validated");
                context.insert_metadata(
                    "validation_type".to_string(),
                    format!("{:?}", e.validation_type),
                );
                context.insert_metadata("valid".to_string(), e.results.valid.to_string());
            }
            llmspell_core::events::ArtifactEventType::ValidationFailed(e) => {
                add_artifact_to_context(&mut context, &e.artifact_id, "validation_failed");
                context.insert_metadata(
                    "validation_type".to_string(),
                    format!("{:?}", e.validation_type),
                );
                context.insert_metadata("failures".to_string(), format!("{:?}", e.failures));
            }
            llmspell_core::events::ArtifactEventType::Derived(e) => {
                add_artifact_to_context(&mut context, &e.derived_artifact.id, "derived");
                context.insert_metadata("parent_artifact".to_string(), e.parent_id.to_string());
                context.insert_metadata(
                    "derivation_type".to_string(),
                    format!("{:?}", e.derivation_type),
                );
            }
            llmspell_core::events::ArtifactEventType::Accessed(e) => {
                add_artifact_to_context(&mut context, &e.artifact_id, "accessed");
                context.insert_metadata("access_type".to_string(), format!("{:?}", e.access_type));
            }
            _ => {}
        }

        // Execute all hooks for this artifact event
        self.execute_hooks(hooks, &mut context).await
    }
}

impl Default for HookExecutor {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for HookExecutor
pub struct HookExecutorBuilder {
    config: HookExecutorConfig,
}

impl HookExecutorBuilder {
    pub fn new() -> Self {
        Self {
            config: HookExecutorConfig::default(),
        }
    }

    pub fn with_circuit_breaker(mut self, enabled: bool) -> Self {
        self.config.enable_circuit_breaker = enabled;
        self
    }

    pub fn with_performance_monitoring(mut self, enabled: bool) -> Self {
        self.config.enable_performance_monitoring = enabled;
        self
    }

    pub fn with_max_execution_time(mut self, duration: Duration) -> Self {
        self.config.max_execution_time = duration;
        self
    }

    pub fn with_overhead_target(mut self, target: f64) -> Self {
        self.config.performance_overhead_target = target;
        self
    }

    pub fn with_breaker_config(mut self, config: BreakerConfig) -> Self {
        self.config.breaker_config = config;
        self
    }

    pub fn with_persistence(mut self, enabled: bool) -> Self {
        self.config.enable_persistence = enabled;
        self
    }

    pub fn build(self) -> HookExecutor {
        HookExecutor::with_config(self.config)
    }
}

impl Default for HookExecutorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::FnHook;
    use crate::types::HookPoint;
    use tokio;
    #[tokio::test]
    async fn test_hook_executor_basic() {
        let executor = HookExecutor::new();

        let hook = FnHook::new("test_hook", |ctx: &mut HookContext| {
            ctx.insert_metadata("executed".to_string(), "true".to_string());
            Ok(HookResult::Continue)
        });

        let component_id =
            crate::types::ComponentId::new(crate::types::ComponentType::System, "test".to_string());
        let mut context = HookContext::new(HookPoint::SystemStartup, component_id);

        let result = executor.execute_hook(&hook, &mut context).await.unwrap();
        assert!(matches!(result, HookResult::Continue));
        assert_eq!(context.get_metadata("executed"), Some("true"));

        // Check metrics
        let metrics = executor.get_metrics("test_hook").unwrap();
        assert_eq!(metrics.execution_count, 1);
    }
    #[tokio::test]
    async fn test_circuit_breaker_protection() {
        let config = HookExecutorConfig {
            breaker_config: BreakerConfig {
                failure_threshold: 2,
                ..Default::default()
            },
            ..Default::default()
        };

        let executor = HookExecutor::with_config(config);

        // Create a failing hook
        let failing_hook = FnHook::new("failing_hook", |_ctx: &mut HookContext| {
            Err(anyhow::anyhow!("Test error"))
        });

        let component_id =
            crate::types::ComponentId::new(crate::types::ComponentType::System, "test".to_string());
        let mut context = HookContext::new(HookPoint::SystemStartup, component_id.clone());

        // First failure
        let _ = executor.execute_hook(&failing_hook, &mut context).await;

        // Second failure - should open circuit
        let _ = executor.execute_hook(&failing_hook, &mut context).await;

        // Third attempt - should be skipped
        let mut context = HookContext::new(HookPoint::SystemStartup, component_id);
        let result = executor
            .execute_hook(&failing_hook, &mut context)
            .await
            .unwrap();

        match result {
            HookResult::Skipped(reason) => {
                assert!(reason.contains("Circuit breaker open"));
            }
            _ => panic!("Expected Skipped result"),
        }
    }
    #[tokio::test]
    async fn test_slow_hook_detection() {
        let config = HookExecutorConfig {
            max_execution_time: Duration::from_millis(50),
            breaker_config: BreakerConfig {
                slow_call_threshold: 2,
                slow_call_duration: Duration::from_millis(50),
                ..Default::default()
            },
            ..Default::default()
        };

        let executor = HookExecutor::with_config(config);

        // Create a slow hook
        let slow_hook = FnHook::new("slow_hook", |_ctx: &mut HookContext| {
            std::thread::sleep(Duration::from_millis(60));
            Ok(HookResult::Continue)
        });

        let component_id =
            crate::types::ComponentId::new(crate::types::ComponentType::System, "test".to_string());

        // Execute slow hook twice
        for _ in 0..2 {
            let mut context = HookContext::new(HookPoint::SystemStartup, component_id.clone());
            let _ = executor.execute_hook(&slow_hook, &mut context).await;
        }

        // Third execution should be skipped
        let mut context = HookContext::new(HookPoint::SystemStartup, component_id);
        let result = executor
            .execute_hook(&slow_hook, &mut context)
            .await
            .unwrap();

        match result {
            HookResult::Skipped(_) => {
                // Expected
            }
            _ => panic!("Expected Skipped result due to slow execution"),
        }
    }
    #[tokio::test]
    async fn test_hook_cancellation() {
        let executor = HookExecutor::new();

        let hook1: Arc<dyn Hook> = Arc::new(FnHook::new("hook1", |_ctx: &mut HookContext| {
            Ok(HookResult::Continue)
        }));

        let hook2: Arc<dyn Hook> = Arc::new(FnHook::new("hook2", |_ctx: &mut HookContext| {
            Ok(HookResult::Cancel("Test cancellation".to_string()))
        }));

        let hook3: Arc<dyn Hook> = Arc::new(FnHook::new("hook3", |_ctx: &mut HookContext| {
            Ok(HookResult::Continue)
        }));

        let hooks = vec![hook1, hook2, hook3];

        let component_id =
            crate::types::ComponentId::new(crate::types::ComponentType::System, "test".to_string());
        let mut context = HookContext::new(HookPoint::SystemStartup, component_id);

        let results = executor.execute_hooks(&hooks, &mut context).await.unwrap();

        // Should only execute first two hooks
        assert_eq!(results.len(), 2);
        assert!(matches!(results[0], HookResult::Continue));
        assert!(matches!(results[1], HookResult::Cancel(_)));
    }
    #[tokio::test]
    async fn test_custom_hook_configuration() {
        let executor = HookExecutor::new();

        // Configure a specific hook with custom timeout
        executor.configure_hook(
            "custom_hook",
            HookExecutionConfig {
                timeout: Some(Duration::from_millis(200)),
                use_circuit_breaker: true,
                breaker_config: None,
                persist_executions: true,
                retention_priority: 0,
            },
        );

        let hook = FnHook::new("custom_hook", |_ctx: &mut HookContext| {
            std::thread::sleep(Duration::from_millis(150));
            Ok(HookResult::Continue)
        });

        let component_id =
            crate::types::ComponentId::new(crate::types::ComponentType::System, "test".to_string());
        let mut context = HookContext::new(HookPoint::SystemStartup, component_id);

        // Should succeed with custom timeout
        let result = executor.execute_hook(&hook, &mut context).await.unwrap();
        assert!(matches!(result, HookResult::Continue));
    }
}
