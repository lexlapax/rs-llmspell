//! # Kernel Hook System
//!
//! Advanced hook system integration for the `LLMSpell` kernel with sophisticated
//! hook patterns and kernel-specific execution context.
//!
//! ## Features
//!
//! - **Kernel-Specific Hooks**: `PreExecute`, `PostExecute`, `PreDebug`, `StateChange`
//! - **Advanced Patterns**: `CompositeHook`, `ForkHook`, `RetryHook`, `ConditionalHook`
//! - **Dynamic Debug Flow**: Modify debug execution flow through hooks
//! - **Performance Monitoring**: <5% overhead with circuit breaker protection
//! - **Event Integration**: Seamless integration with kernel event correlation
//!
//! ## Architecture
//!
//! The kernel hook system extends the llmspell-hooks infrastructure with
//! kernel-specific contexts and execution flow integration.

pub mod conditional;
pub mod kernel_hooks;
pub mod performance;

// Re-export core hook infrastructure from llmspell-hooks
pub use llmspell_hooks::{
    CachingHook, CircuitBreaker, ComponentId, ComponentType, ForkBuilder, Hook, HookContext,
    HookExecutor, HookPoint, HookRegistry, HookResult, LoggingHook, MetricsHook,
    PerformanceMonitor, Priority, RetryBuilder, RetryHook,
};

// Note: CompositeHook patterns available in llmspell-hooks but not re-exported here
// Users can import directly from llmspell_hooks if needed

// Re-export kernel-specific types
pub use conditional::{Condition, ConditionBuilder, ConditionalHook};
pub use kernel_hooks::{
    DebugContext, ExecutionContext, KernelHook, KernelHookManager, PostExecuteHook, PreDebugHook,
    PreExecuteHook, StateChangeHook, StateContext,
};
pub use performance::{HookPerformanceMetrics, KernelPerformanceMonitor};

use crate::events::KernelEvent;
use anyhow::Result;
use std::time::Duration;

/// Kernel hook point definitions extending base `HookPoint`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KernelHookPoint {
    // Execution flow hooks
    /// Hook point triggered before kernel startup
    PreKernelStartup,
    /// Hook point triggered after kernel startup
    PostKernelStartup,
    /// Hook point triggered before code execution
    PreCodeExecution,
    /// Hook point triggered after code execution
    PostCodeExecution,
    /// Hook point triggered before script loading
    PreScriptLoad,
    /// Hook point triggered after script loading
    PostScriptLoad,

    // Debug flow hooks
    /// Hook point triggered before debug session starts
    PreDebugSession,
    /// Hook point triggered after debug session ends
    PostDebugSession,
    /// Hook point triggered before breakpoint handling
    PreBreakpoint,
    /// Hook point triggered after breakpoint handling
    PostBreakpoint,
    /// Hook point triggered before step execution
    PreStepExecution,
    /// Hook point triggered after step execution
    PostStepExecution,

    // State management hooks
    /// Hook point triggered before state changes
    PreStateChange,
    /// Hook point triggered after state changes
    PostStateChange,
    /// Hook point triggered before state persistence
    PreStatePersist,
    /// Hook point triggered after state persistence
    PostStatePersist,

    // Message handling hooks
    /// Hook point triggered before message handling
    PreMessageHandle,
    /// Hook point triggered after message handling
    PostMessageHandle,
    /// Hook point triggered before `IOPub` broadcast
    PreIOPubBroadcast,
    /// Hook point triggered after `IOPub` broadcast
    PostIOPubBroadcast,

    // Error handling hooks
    /// Hook point triggered on execution errors
    ExecutionError,
    /// Hook point triggered on debug errors
    DebugError,
    /// Hook point triggered on state errors
    StateError,
    /// Hook point triggered on system errors
    SystemError,
}

/// Convert kernel hook points to base hook points for compatibility
impl From<KernelHookPoint> for HookPoint {
    fn from(kernel_point: KernelHookPoint) -> Self {
        match kernel_point {
            KernelHookPoint::PreKernelStartup
            | KernelHookPoint::PostKernelStartup
            | KernelHookPoint::PreScriptLoad
            | KernelHookPoint::PostScriptLoad
            | KernelHookPoint::PreStateChange
            | KernelHookPoint::PostStateChange
            | KernelHookPoint::PreStatePersist
            | KernelHookPoint::PostStatePersist => HookPoint::SystemStartup,
            KernelHookPoint::PreCodeExecution
            | KernelHookPoint::PostCodeExecution
            | KernelHookPoint::PreMessageHandle
            | KernelHookPoint::PostMessageHandle => HookPoint::BeforeAgentExecution,
            KernelHookPoint::PreDebugSession
            | KernelHookPoint::PostDebugSession
            | KernelHookPoint::PreBreakpoint
            | KernelHookPoint::PostBreakpoint
            | KernelHookPoint::PreStepExecution
            | KernelHookPoint::PostStepExecution => HookPoint::BeforeToolExecution,
            KernelHookPoint::PreIOPubBroadcast | KernelHookPoint::PostIOPubBroadcast => {
                HookPoint::AfterAgentExecution
            }
            KernelHookPoint::ExecutionError
            | KernelHookPoint::StateError
            | KernelHookPoint::SystemError => HookPoint::AgentError,
            KernelHookPoint::DebugError => HookPoint::ToolError,
        }
    }
}

/// Hook manager specifically for kernel operations
pub struct KernelHookSystem {
    registry: HookRegistry,
    executor: HookExecutor,
    performance_monitor: KernelPerformanceMonitor,
    debug_flow_enabled: bool,
}

impl KernelHookSystem {
    /// Create a new kernel hook system
    pub fn new() -> Self {
        Self {
            registry: HookRegistry::new(),
            executor: HookExecutor::new(),
            performance_monitor: KernelPerformanceMonitor::new(),
            debug_flow_enabled: true,
        }
    }

    /// Register a kernel-specific hook
    ///
    /// # Errors
    ///
    /// Returns an error if the hook registration fails.
    pub fn register_kernel_hook(&mut self, hook: KernelHook) -> Result<()> {
        let hook_point = hook.hook_point();
        self.registry.register(hook_point, hook)?;
        Ok(())
    }

    /// Execute hooks for a specific kernel hook point
    ///
    /// # Errors
    ///
    /// Returns an error if hook execution fails.
    pub async fn execute_hooks(
        &self,
        point: KernelHookPoint,
        context: &mut HookContext,
    ) -> Result<HookResult> {
        let start_time = std::time::Instant::now();

        // Get hooks for this point from registry
        let hooks = self.registry.get_hooks(&point.into());

        // Execute hooks with performance monitoring using executor
        let results = self.executor.execute_hooks(&hooks, context).await?;

        // Aggregate results - return first non-Continue result or Continue
        let result = results
            .into_iter()
            .find(|r| !matches!(r, HookResult::Continue))
            .unwrap_or(HookResult::Continue);

        let duration = start_time.elapsed();
        self.performance_monitor
            .record_hook_execution(point, duration);

        // Check performance threshold
        if duration > Duration::from_millis(50) {
            tracing::warn!(
                "Hook execution for {:?} took {:?} (>50ms threshold)",
                point,
                duration
            );
        }

        Ok(result)
    }

    /// Enable or disable dynamic debug flow modification
    pub fn set_debug_flow_enabled(&mut self, enabled: bool) {
        self.debug_flow_enabled = enabled;
    }

    /// Check if debug flow modification is enabled
    pub fn is_debug_flow_enabled(&self) -> bool {
        self.debug_flow_enabled
    }

    /// Get performance metrics
    pub fn performance_metrics(&self) -> HookPerformanceMetrics {
        self.performance_monitor.metrics()
    }

    /// Trigger kernel event through hook system
    ///
    /// # Errors
    ///
    /// Returns an error if event processing or hook execution fails.
    pub async fn trigger_kernel_event(&self, event: KernelEvent) -> Result<()> {
        // Convert kernel event to hook context and execute relevant hooks
        let point = match &event {
            KernelEvent::ExecuteRequest { .. } => KernelHookPoint::PreCodeExecution,
            KernelEvent::ExecuteReply { .. } => KernelHookPoint::PostCodeExecution,
            KernelEvent::KernelStartup { .. } => KernelHookPoint::PreKernelStartup,
            KernelEvent::KernelShutdown { .. } => KernelHookPoint::PostKernelStartup,
            KernelEvent::StatusChange { .. } => KernelHookPoint::PreStateChange,
            _ => return Ok(()), // Not all events trigger hooks
        };

        let component_id = ComponentId::new(ComponentType::System, "kernel".to_string());
        let mut context = HookContext::new(point.into(), component_id);

        // Add event data to context
        context
            .data
            .insert("kernel_event".to_string(), serde_json::to_value(&event)?);

        let _result = self.execute_hooks(point, &mut context).await?;

        Ok(())
    }
}

impl Default for KernelHookSystem {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hooks::{ComponentId, ComponentType};

    #[test]
    fn test_kernel_hook_point_conversion() {
        let kernel_point = KernelHookPoint::PreCodeExecution;
        let base_point: HookPoint = kernel_point.into();
        assert_eq!(base_point, HookPoint::BeforeAgentExecution);
    }

    #[tokio::test]
    async fn test_kernel_hook_system_creation() {
        let hook_system = KernelHookSystem::new();
        assert!(hook_system.is_debug_flow_enabled());

        let metrics = hook_system.performance_metrics();
        assert_eq!(metrics.total_executions, 0);
    }

    #[tokio::test]
    async fn test_hook_execution_with_performance_monitoring() {
        let hook_system = KernelHookSystem::new();
        let component_id = ComponentId::new(ComponentType::System, "test".to_string());
        let mut context = HookContext::new(HookPoint::SystemStartup, component_id);

        let result = hook_system
            .execute_hooks(KernelHookPoint::PreKernelStartup, &mut context)
            .await;
        assert!(result.is_ok());

        let metrics = hook_system.performance_metrics();
        assert_eq!(metrics.total_executions, 1);
    }
}
