//! ABOUTME: Workflow executor trait and implementations for execution management
//! ABOUTME: Provides standardized workflow execution with monitoring and cancellation

use crate::types::{WorkflowInput, WorkflowOutput};
use async_trait::async_trait;
use llmspell_core::{
    traits::{base_agent::BaseAgent, workflow::Workflow},
    LLMSpellError, Result,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{oneshot, RwLock};
use tokio::task::JoinHandle;
use tracing::field::Empty;
use tracing::{debug, info, instrument, warn, Span};

/// Execution metrics for workflow runs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetrics {
    /// Total execution time
    pub duration: Duration,
    /// Number of steps executed
    pub steps_executed: usize,
    /// Number of steps that failed
    pub steps_failed: usize,
    /// Memory usage in bytes (if available)
    pub memory_usage: Option<usize>,
    /// CPU usage percentage (if available)
    pub cpu_usage: Option<f64>,
}

/// Execution context for workflows
#[derive(Debug)]
pub struct ExecutionContext {
    /// Cancellation token
    pub cancel_token: Option<oneshot::Receiver<()>>,
    /// Execution timeout
    pub timeout: Option<Duration>,
    /// Enable metrics collection
    pub collect_metrics: bool,
    /// Enable execution tracing
    pub enable_tracing: bool,
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self {
            cancel_token: None,
            timeout: None,
            collect_metrics: true,
            enable_tracing: false,
        }
    }
}

/// Hook for monitoring workflow execution
#[async_trait]
pub trait ExecutionHook: Send + Sync {
    /// Called before workflow execution starts
    async fn before_execution(&self, workflow_name: &str, input: &WorkflowInput) -> Result<()>;

    /// Called after workflow execution completes
    async fn after_execution(
        &self,
        workflow_name: &str,
        output: &WorkflowOutput,
        metrics: &ExecutionMetrics,
    ) -> Result<()>;

    /// Called when workflow execution fails
    async fn on_error(&self, workflow_name: &str, error: &LLMSpellError) -> Result<()>;
}

/// Trait for workflow execution management
#[async_trait]
pub trait WorkflowExecutor: Send + Sync {
    /// Execute a workflow with the given input
    async fn execute_workflow(
        &self,
        workflow: Arc<dyn Workflow + Send + Sync>,
        input: WorkflowInput,
    ) -> Result<WorkflowOutput>;

    /// Execute a workflow with execution context (timeout, cancellation, etc.)
    async fn execute_with_context(
        &self,
        workflow: Arc<dyn Workflow + Send + Sync>,
        input: WorkflowInput,
        context: ExecutionContext,
    ) -> Result<WorkflowOutput>;

    /// Execute a workflow asynchronously, returning a handle
    fn execute_async(
        &self,
        workflow: Arc<dyn Workflow + Send + Sync>,
        input: WorkflowInput,
    ) -> JoinHandle<Result<WorkflowOutput>>;

    /// Cancel a running workflow execution
    async fn cancel_execution(&self, execution_id: &str) -> Result<()>;

    /// Get execution metrics for a completed workflow
    async fn get_metrics(&self, execution_id: &str) -> Result<Option<ExecutionMetrics>>;

    /// Register an execution hook
    async fn register_hook(&self, hook: Arc<dyn ExecutionHook>) -> Result<()>;
}

/// Default implementation of WorkflowExecutor
pub struct DefaultWorkflowExecutor {
    /// Active executions
    active_executions: Arc<RwLock<std::collections::HashMap<String, oneshot::Sender<()>>>>,
    /// Execution metrics storage
    metrics_store: Arc<RwLock<std::collections::HashMap<String, ExecutionMetrics>>>,
    /// Registered hooks
    hooks: Arc<RwLock<Vec<Arc<dyn ExecutionHook>>>>,
}

impl DefaultWorkflowExecutor {
    /// Create a new default workflow executor
    pub fn new() -> Self {
        Self {
            active_executions: Arc::new(RwLock::new(std::collections::HashMap::new())),
            metrics_store: Arc::new(RwLock::new(std::collections::HashMap::new())),
            hooks: Arc::new(RwLock::new(Vec::new())),
        }
    }

    #[instrument(level = "trace", skip(self, input), fields(workflow_name = %workflow_name, hook_count = Empty))]
    async fn run_before_hooks(&self, workflow_name: &str, input: &WorkflowInput) -> Result<()> {
        let hooks = self.hooks.read().await;
        Span::current().record("hook_count", hooks.len());
        for hook in hooks.iter() {
            if let Err(e) = hook.before_execution(workflow_name, input).await {
                warn!("Before execution hook failed: {}", e);
            }
        }
        Ok(())
    }

    #[instrument(level = "trace", skip(self, output, metrics), fields(
        workflow_name = %workflow_name,
        duration_ms = metrics.duration.as_millis() as u64,
        steps_executed = metrics.steps_executed
    ))]
    async fn run_after_hooks(
        &self,
        workflow_name: &str,
        output: &WorkflowOutput,
        metrics: &ExecutionMetrics,
    ) -> Result<()> {
        let hooks = self.hooks.read().await;
        for hook in hooks.iter() {
            if let Err(e) = hook.after_execution(workflow_name, output, metrics).await {
                warn!("After execution hook failed: {}", e);
            }
        }
        Ok(())
    }

    #[instrument(level = "debug", skip(self, error), fields(
        workflow_name = %workflow_name,
        error_type = ?error
    ))]
    async fn run_error_hooks(&self, workflow_name: &str, error: &LLMSpellError) -> Result<()> {
        let hooks = self.hooks.read().await;
        for hook in hooks.iter() {
            if let Err(e) = hook.on_error(workflow_name, error).await {
                warn!("Error hook failed: {}", e);
            }
        }
        Ok(())
    }
}

impl Default for DefaultWorkflowExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl WorkflowExecutor for DefaultWorkflowExecutor {
    #[instrument(level = "info", skip(self, workflow, input), fields(
        workflow_name = Empty,
        execution_id = Empty,
        input_size = input.input.to_string().len()
    ))]
    async fn execute_workflow(
        &self,
        workflow: Arc<dyn Workflow + Send + Sync>,
        input: WorkflowInput,
    ) -> Result<WorkflowOutput> {
        let context = ExecutionContext::default();
        self.execute_with_context(workflow, input, context).await
    }

    #[instrument(level = "info", skip(self, workflow, input, context), fields(
        workflow_name = Empty,
        execution_id = Empty,
        with_timeout = context.timeout.is_some(),
        with_cancellation = context.cancel_token.is_some(),
        collect_metrics = context.collect_metrics
    ))]
    async fn execute_with_context(
        &self,
        workflow: Arc<dyn Workflow + Send + Sync>,
        input: WorkflowInput,
        context: ExecutionContext,
    ) -> Result<WorkflowOutput> {
        // Get workflow name from metadata (all workflows implement BaseAgent)
        let base_agent = workflow.clone() as Arc<dyn BaseAgent + Send + Sync>;
        let workflow_name = base_agent.metadata().name.clone();
        let execution_id = format!("exec_{}", uuid::Uuid::new_v4());
        let start_time = Instant::now();

        // Record tracing fields
        Span::current().record("workflow_name", workflow_name.as_str());
        Span::current().record("execution_id", execution_id.as_str());

        debug!(
            "Starting workflow execution: {} ({})",
            workflow_name, execution_id
        );

        // Run before hooks
        self.run_before_hooks(&workflow_name, &input).await?;

        // Set up cancellation if needed
        let (cancel_tx, _cancel_rx) = oneshot::channel();
        if context.cancel_token.is_some() {
            let mut executions = self.active_executions.write().await;
            executions.insert(execution_id.clone(), cancel_tx);
        }

        // Convert workflow to BaseAgent for execution
        let agent = workflow.clone() as Arc<dyn BaseAgent + Send + Sync>;

        // Convert WorkflowInput to AgentInput
        use crate::adapters::WorkflowInputAdapter;
        let agent_input = WorkflowInputAdapter::to_agent_input(input.clone());

        // Create execution context
        let exec_context = llmspell_core::execution_context::ExecutionContext::new();

        // Execute with timeout if specified
        let result = if let Some(timeout) = context.timeout {
            match tokio::time::timeout(timeout, agent.execute(agent_input, exec_context)).await {
                Ok(result) => result,
                Err(_) => {
                    #[allow(clippy::cast_possible_truncation)]
                    let timeout_ms = timeout.as_millis() as u64;
                    return Err(LLMSpellError::Timeout {
                        message: format!("Workflow execution timed out: {}", workflow_name),
                        duration_ms: Some(timeout_ms),
                    });
                }
            }
        } else {
            agent.execute(agent_input, exec_context).await
        };

        // Convert result to WorkflowOutput
        let output = match result {
            Ok(agent_output) => {
                use crate::adapters::WorkflowOutputAdapter;
                let duration = start_time.elapsed();
                WorkflowOutputAdapter::from_agent_output(agent_output, duration)
            }
            Err(e) => {
                // Run error hooks
                self.run_error_hooks(&workflow_name, &e).await?;
                return Err(e);
            }
        };

        // Collect metrics if enabled
        if context.collect_metrics {
            let metrics = ExecutionMetrics {
                duration: output.duration,
                steps_executed: output.steps_executed,
                steps_failed: output.steps_failed,
                memory_usage: None, // TODO: Implement memory tracking
                cpu_usage: None,    // TODO: Implement CPU tracking
            };

            let mut metrics_store = self.metrics_store.write().await;
            metrics_store.insert(execution_id.clone(), metrics.clone());

            // Run after hooks
            self.run_after_hooks(&workflow_name, &output, &metrics)
                .await?;
        }

        // Clean up active execution
        if context.cancel_token.is_some() {
            let mut executions = self.active_executions.write().await;
            executions.remove(&execution_id);
        }

        info!(
            "Workflow execution completed: {} ({})",
            workflow_name, execution_id
        );
        Ok(output)
    }

    #[instrument(level = "debug", skip(self, workflow, input))]
    fn execute_async(
        &self,
        workflow: Arc<dyn Workflow + Send + Sync>,
        input: WorkflowInput,
    ) -> JoinHandle<Result<WorkflowOutput>> {
        let executor = Arc::new(self.clone());
        tokio::spawn(async move { executor.execute_workflow(workflow, input).await })
    }

    #[instrument(level = "info", skip(self))]
    async fn cancel_execution(&self, execution_id: &str) -> Result<()> {
        let mut executions = self.active_executions.write().await;
        if let Some(cancel_tx) = executions.remove(execution_id) {
            let _ = cancel_tx.send(()); // Ignore error if receiver dropped
            info!("Cancelled workflow execution: {}", execution_id);
            Ok(())
        } else {
            Err(LLMSpellError::Resource {
                message: format!("No active execution found: {}", execution_id),
                resource_type: Some("workflow_execution".to_string()),
                source: None,
            })
        }
    }

    #[instrument(level = "debug", skip(self))]
    async fn get_metrics(&self, execution_id: &str) -> Result<Option<ExecutionMetrics>> {
        let metrics = self.metrics_store.read().await;
        Ok(metrics.get(execution_id).cloned())
    }

    #[instrument(level = "debug", skip_all)]
    async fn register_hook(&self, hook: Arc<dyn ExecutionHook>) -> Result<()> {
        let mut hooks = self.hooks.write().await;
        hooks.push(hook);
        Ok(())
    }
}

// Clone implementation for executor
impl Clone for DefaultWorkflowExecutor {
    fn clone(&self) -> Self {
        Self {
            active_executions: self.active_executions.clone(),
            metrics_store: self.metrics_store.clone(),
            hooks: self.hooks.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_context_default() {
        let context = ExecutionContext::default();
        assert!(context.cancel_token.is_none());
        assert!(context.timeout.is_none());
        assert!(context.collect_metrics);
        assert!(!context.enable_tracing);
    }

    #[test]
    fn test_execution_metrics_creation() {
        let metrics = ExecutionMetrics {
            duration: Duration::from_secs(5),
            steps_executed: 10,
            steps_failed: 1,
            memory_usage: Some(1024 * 1024),
            cpu_usage: Some(75.5),
        };

        assert_eq!(metrics.duration, Duration::from_secs(5));
        assert_eq!(metrics.steps_executed, 10);
        assert_eq!(metrics.steps_failed, 1);
        assert_eq!(metrics.memory_usage, Some(1024 * 1024));
        assert_eq!(metrics.cpu_usage, Some(75.5));
    }

    #[tokio::test]
    async fn test_default_executor_creation() {
        let executor = DefaultWorkflowExecutor::new();
        assert!(executor.active_executions.read().await.is_empty());
        assert!(executor.metrics_store.read().await.is_empty());
        assert!(executor.hooks.read().await.is_empty());
    }
}
