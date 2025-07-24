//! ABOUTME: Integration of llmspell-hooks HookExecutor with workflow infrastructure
//! ABOUTME: Provides WorkflowExecutor that orchestrates hook execution for workflows

use crate::hooks::StepContext;
use crate::types::WorkflowState;
use llmspell_core::{ComponentMetadata, ExecutionContext, Result};
use llmspell_hooks::{
    CircuitBreaker, ComponentId as HookComponentId, ComponentType, HookContext, HookExecutor,
    HookPoint, HookRegistry,
};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// Configuration for workflow lifecycle hook integration
#[derive(Debug, Clone)]
pub struct WorkflowLifecycleConfig {
    /// Enable hook execution (can be disabled for performance)
    pub enable_hooks: bool,
    /// Enable circuit breaker protection
    pub enable_circuit_breaker: bool,
    /// Maximum time allowed for hook execution
    pub max_hook_execution_time: Duration,
    /// Circuit breaker configuration
    pub circuit_breaker_failure_threshold: u32,
    pub circuit_breaker_recovery_time: Duration,
    /// Enable comprehensive audit logging
    pub enable_audit_logging: bool,
    /// Maximum security level allowed for workflow execution
    pub max_security_level: llmspell_core::traits::tool::SecurityLevel,
}

impl Default for WorkflowLifecycleConfig {
    fn default() -> Self {
        Self {
            enable_hooks: true,
            enable_circuit_breaker: true,
            max_hook_execution_time: Duration::from_millis(200), // 200ms max for hooks
            circuit_breaker_failure_threshold: 5,
            circuit_breaker_recovery_time: Duration::from_secs(30),
            enable_audit_logging: true,
            max_security_level: llmspell_core::traits::tool::SecurityLevel::Privileged,
        }
    }
}

/// Workflow-specific hook context with execution metadata
#[derive(Debug, Clone)]
pub struct WorkflowHookContext {
    /// Base hook context
    pub base_context: HookContext,
    /// Workflow metadata
    pub workflow_metadata: ComponentMetadata,
    /// Current workflow state
    pub workflow_state: WorkflowState,
    /// Workflow type (Sequential, Conditional, Loop, Parallel)
    pub workflow_type: String,
    /// Current step information (if applicable)
    pub step_context: Option<StepContext>,
    /// Shared workflow data
    pub shared_data: HashMap<String, JsonValue>,
    /// Workflow execution phase
    pub execution_phase: WorkflowExecutionPhase,
    /// Pattern-specific context (e.g., iteration count for loops)
    pub pattern_context: HashMap<String, JsonValue>,
}

/// Workflow execution phases for hook context
#[derive(Debug, Clone)]
pub enum WorkflowExecutionPhase {
    WorkflowStart,
    WorkflowComplete,
    StepBoundary,
    StateChange,
    SharedDataAccess,
    ConditionEvaluation,
    BranchSelection,
    LoopIterationStart,
    LoopIterationComplete,
    LoopTermination,
    ParallelFork,
    ParallelJoin,
    ParallelSynchronization,
    ErrorHandling,
}

impl WorkflowHookContext {
    /// Create a new workflow hook context
    pub fn new(
        component_id: HookComponentId,
        workflow_metadata: ComponentMetadata,
        workflow_state: WorkflowState,
        workflow_type: String,
        execution_phase: WorkflowExecutionPhase,
    ) -> Self {
        let hook_point = match &execution_phase {
            WorkflowExecutionPhase::WorkflowStart => {
                HookPoint::Custom("workflow_start".to_string())
            }
            WorkflowExecutionPhase::WorkflowComplete => {
                HookPoint::Custom("workflow_complete".to_string())
            }
            WorkflowExecutionPhase::StepBoundary => {
                HookPoint::Custom("workflow_step_boundary".to_string())
            }
            WorkflowExecutionPhase::StateChange => {
                HookPoint::Custom("workflow_state_change".to_string())
            }
            WorkflowExecutionPhase::SharedDataAccess => {
                HookPoint::Custom("workflow_shared_data".to_string())
            }
            WorkflowExecutionPhase::ErrorHandling => HookPoint::WorkflowError,
            WorkflowExecutionPhase::ConditionEvaluation => {
                HookPoint::Custom("condition_evaluation".to_string())
            }
            WorkflowExecutionPhase::BranchSelection => {
                HookPoint::Custom("branch_selection".to_string())
            }
            WorkflowExecutionPhase::LoopIterationStart => {
                HookPoint::Custom("loop_iteration_start".to_string())
            }
            WorkflowExecutionPhase::LoopIterationComplete => {
                HookPoint::Custom("loop_iteration_complete".to_string())
            }
            WorkflowExecutionPhase::LoopTermination => {
                HookPoint::Custom("loop_termination".to_string())
            }
            WorkflowExecutionPhase::ParallelFork => HookPoint::Custom("parallel_fork".to_string()),
            WorkflowExecutionPhase::ParallelJoin => HookPoint::Custom("parallel_join".to_string()),
            WorkflowExecutionPhase::ParallelSynchronization => {
                HookPoint::Custom("parallel_synchronization".to_string())
            }
        };
        let base_context = HookContext::new(hook_point, component_id);

        Self {
            base_context,
            workflow_metadata,
            workflow_state,
            workflow_type,
            step_context: None,
            shared_data: HashMap::new(),
            execution_phase,
            pattern_context: HashMap::new(),
        }
    }

    /// Set step context (for step boundary hooks)
    pub fn with_step_context(mut self, step_context: StepContext) -> Self {
        self.step_context = Some(step_context);
        self
    }

    /// Set shared data
    pub fn with_shared_data(mut self, shared_data: HashMap<String, JsonValue>) -> Self {
        self.shared_data = shared_data;
        self
    }

    /// Add pattern-specific context
    pub fn with_pattern_context(mut self, key: String, value: JsonValue) -> Self {
        self.pattern_context.insert(key, value);
        self
    }

    /// Get hook point for this execution phase
    pub fn get_hook_point(&self) -> HookPoint {
        self.base_context.point.clone()
    }
}

/// Enhanced workflow executor with hook integration
#[derive(Clone)]
pub struct WorkflowExecutor {
    /// Hook executor for running hooks
    hook_executor: Option<Arc<HookExecutor>>,
    /// Circuit breaker for performance protection
    circuit_breaker: Option<Arc<CircuitBreaker>>,
    /// Configuration
    config: WorkflowLifecycleConfig,
    /// Component ID for this workflow executor
    #[allow(dead_code)]
    component_id: HookComponentId,
}

impl WorkflowExecutor {
    /// Create a new workflow executor with hook integration
    pub fn new(
        config: WorkflowLifecycleConfig,
        hook_executor: Option<Arc<HookExecutor>>,
        hook_registry: Option<Arc<HookRegistry>>,
    ) -> Self {
        let component_id =
            HookComponentId::new(ComponentType::Workflow, "workflow_executor".to_string());

        let circuit_breaker = if config.enable_circuit_breaker {
            Some(Arc::new(CircuitBreaker::new("workflow_hooks".to_string())))
        } else {
            None
        };

        // If we have a hook registry but no executor, create one
        let hook_executor = match (hook_executor, hook_registry) {
            (Some(exec), _) => Some(exec),
            (None, Some(_registry)) => {
                let exec = Arc::new(HookExecutor::new());
                // Note: HookExecutor should ideally have a method to set registry
                // For now, we'll just use the executor as-is
                Some(exec)
            }
            (None, None) => None,
        };

        Self {
            hook_executor,
            circuit_breaker,
            config,
            component_id,
        }
    }

    /// Execute hooks for a workflow phase
    pub async fn execute_workflow_hooks(
        &self,
        workflow_context: WorkflowHookContext,
    ) -> Result<()> {
        if !self.config.enable_hooks {
            return Ok(());
        }

        let hook_point = workflow_context.get_hook_point();
        debug!(
            "Executing workflow hooks for phase: {:?}",
            workflow_context.execution_phase
        );

        // Check circuit breaker
        if let Some(circuit_breaker) = &self.circuit_breaker {
            if !circuit_breaker.can_execute() {
                warn!("Circuit breaker open for workflow hooks, skipping execution");
                return Ok(());
            }
        }

        let start_time = Instant::now();

        // Execute hooks if we have an executor
        if let Some(_hook_executor) = &self.hook_executor {
            // Convert workflow context to hook context
            let mut hook_context = workflow_context.base_context.clone();

            // Add workflow-specific metadata
            hook_context.metadata.insert(
                "workflow_type".to_string(),
                workflow_context.workflow_type.clone(),
            );
            hook_context.metadata.insert(
                "workflow_state".to_string(),
                format!("{:?}", workflow_context.workflow_state),
            );
            hook_context.metadata.insert(
                "execution_phase".to_string(),
                format!("{:?}", workflow_context.execution_phase),
            );

            // Add step context if present
            if let Some(step_ctx) = &workflow_context.step_context {
                hook_context
                    .metadata
                    .insert("step_name".to_string(), step_ctx.name.clone());
                hook_context
                    .metadata
                    .insert("step_index".to_string(), step_ctx.index.to_string());
                hook_context
                    .metadata
                    .insert("step_type".to_string(), step_ctx.step_type.clone());
            }

            // Add pattern context
            for (key, value) in &workflow_context.pattern_context {
                hook_context
                    .data
                    .insert(format!("pattern_{}", key), value.clone());
            }

            // For now, we don't execute hooks directly through HookExecutor
            // This is a placeholder that maintains the structure without actual hook execution
            // TODO: Integrate with HookRegistry when API is stabilized

            let _hook_point = hook_point; // Avoid unused warning
            let _hook_context = hook_context; // Avoid unused warning

            // Simulate successful hook execution
            let duration = start_time.elapsed();
            debug!(
                "Workflow hooks would execute for phase: {:?}",
                workflow_context.execution_phase
            );

            // Record success with circuit breaker
            if let Some(circuit_breaker) = &self.circuit_breaker {
                circuit_breaker.record_success(duration);
            }
        }

        // Audit logging if enabled
        if self.config.enable_audit_logging {
            self.log_audit_event(&workflow_context);
        }

        Ok(())
    }

    /// Execute hooks for step boundaries
    pub async fn execute_step_hooks(
        &self,
        workflow_metadata: ComponentMetadata,
        workflow_state: WorkflowState,
        workflow_type: String,
        step_context: StepContext,
        _is_pre_execution: bool,
    ) -> Result<()> {
        let phase = WorkflowExecutionPhase::StepBoundary;

        let component_id = HookComponentId::new(
            ComponentType::Workflow,
            format!("workflow_{}", workflow_metadata.name),
        );

        let mut hook_context = WorkflowHookContext::new(
            component_id,
            workflow_metadata,
            workflow_state,
            workflow_type,
            phase,
        );
        hook_context = hook_context.with_step_context(step_context);

        self.execute_workflow_hooks(hook_context).await
    }

    /// Execute hooks for state changes
    pub async fn execute_state_change_hooks(
        &self,
        workflow_metadata: ComponentMetadata,
        old_state: WorkflowState,
        new_state: WorkflowState,
        workflow_type: String,
    ) -> Result<()> {
        let component_id = HookComponentId::new(
            ComponentType::Workflow,
            format!("workflow_{}", workflow_metadata.name),
        );

        let mut hook_context = WorkflowHookContext::new(
            component_id,
            workflow_metadata,
            new_state,
            workflow_type,
            WorkflowExecutionPhase::StateChange,
        );

        // Add old state to pattern context
        hook_context = hook_context.with_pattern_context(
            "old_state".to_string(),
            JsonValue::String(format!("{:?}", old_state)),
        );

        self.execute_workflow_hooks(hook_context).await
    }

    /// Log audit event for workflow execution
    fn log_audit_event(&self, context: &WorkflowHookContext) {
        info!(
            workflow = %context.workflow_metadata.name,
            workflow_type = %context.workflow_type,
            state = ?context.workflow_state,
            phase = ?context.execution_phase,
            "Workflow hook audit event"
        );
    }
}

/// Trait for workflows to implement hook-aware execution
#[async_trait::async_trait]
pub trait HookableWorkflowExecution: Send + Sync {
    /// Execute the workflow with hooks
    async fn execute_with_hooks(
        &self,
        input: serde_json::Value,
        context: ExecutionContext,
        workflow_executor: &WorkflowExecutor,
    ) -> Result<serde_json::Value>;
}
