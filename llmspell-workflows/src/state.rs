//! ABOUTME: Memory-based state management for workflows
//! ABOUTME: Provides in-memory state storage and step coordination

use super::hooks::{WorkflowExecutionPhase, WorkflowExecutor, WorkflowHookContext};
use super::traits::{StepResult, WorkflowStatus};
use super::types::{WorkflowConfig, WorkflowState};
use llmspell_core::{ComponentId, ComponentMetadata, LLMSpellError, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant, SystemTime};
use tracing::{debug, info, warn};

/// Memory-based state manager for workflows
#[derive(Clone)]
pub struct StateManager {
    state: Arc<RwLock<WorkflowState>>,
    config: WorkflowConfig,
    execution_history: Arc<RwLock<Vec<StepResult>>>,
    workflow_status: Arc<RwLock<WorkflowStatus>>,
    /// Optional workflow executor for hook integration
    workflow_executor: Option<Arc<WorkflowExecutor>>,
    /// Component ID for hooks (using hook's ComponentId type)
    component_id: llmspell_hooks::ComponentId,
    metadata: ComponentMetadata,
}

impl StateManager {
    /// Create a new state manager
    pub fn new(config: WorkflowConfig) -> Self {
        let component_id = llmspell_hooks::ComponentId::new(
            llmspell_hooks::ComponentType::Workflow,
            "state_manager".to_string(),
        );
        let metadata = ComponentMetadata::new(
            "state_manager".to_string(),
            "Workflow state manager".to_string(),
        );

        Self {
            state: Arc::new(RwLock::new(WorkflowState::new())),
            config,
            execution_history: Arc::new(RwLock::new(Vec::new())),
            workflow_status: Arc::new(RwLock::new(WorkflowStatus::Pending)),
            workflow_executor: None,
            component_id,
            metadata,
        }
    }

    /// Create with hook integration
    pub fn new_with_hooks(
        config: WorkflowConfig,
        workflow_executor: Arc<WorkflowExecutor>,
    ) -> Self {
        let component_id = llmspell_hooks::ComponentId::new(
            llmspell_hooks::ComponentType::Workflow,
            "state_manager".to_string(),
        );
        let metadata = ComponentMetadata::new(
            "state_manager".to_string(),
            "Workflow state manager with hooks".to_string(),
        );

        Self {
            state: Arc::new(RwLock::new(WorkflowState::new())),
            config,
            execution_history: Arc::new(RwLock::new(Vec::new())),
            workflow_status: Arc::new(RwLock::new(WorkflowStatus::Pending)),
            workflow_executor: Some(workflow_executor),
            component_id,
            metadata,
        }
    }

    /// Enable hook integration
    pub fn with_hooks(&mut self, workflow_executor: Arc<WorkflowExecutor>) {
        self.workflow_executor = Some(workflow_executor);
        self.metadata = ComponentMetadata::new(
            "state_manager".to_string(),
            "Workflow state manager with hooks".to_string(),
        );
    }

    /// Start workflow execution
    pub async fn start_execution(&self) -> Result<()> {
        debug!("Starting workflow execution");

        {
            let mut state = self.state.write().map_err(|e| LLMSpellError::Workflow {
                message: format!("Failed to acquire state lock: {}", e),
                step: None,
                source: None,
            })?;
            state.start_execution();
        }

        {
            let mut status = self
                .workflow_status
                .write()
                .map_err(|e| LLMSpellError::Workflow {
                    message: format!("Failed to acquire status lock: {}", e),
                    step: None,
                    source: None,
                })?;
            *status = WorkflowStatus::Running;
        }

        Ok(())
    }

    /// Complete workflow execution
    pub async fn complete_execution(&self, success: bool) -> Result<()> {
        debug!("Completing workflow execution with success: {}", success);

        let mut status = self
            .workflow_status
            .write()
            .map_err(|e| LLMSpellError::Workflow {
                message: format!("Failed to acquire status lock: {}", e),
                step: None,
                source: None,
            })?;

        *status = if success {
            WorkflowStatus::Completed
        } else {
            WorkflowStatus::Failed
        };

        Ok(())
    }

    /// Cancel workflow execution
    pub async fn cancel_execution(&self) -> Result<()> {
        debug!("Cancelling workflow execution");

        let mut status = self
            .workflow_status
            .write()
            .map_err(|e| LLMSpellError::Workflow {
                message: format!("Failed to acquire status lock: {}", e),
                step: None,
                source: None,
            })?;

        *status = WorkflowStatus::Cancelled;

        Ok(())
    }

    /// Get current workflow status
    pub async fn get_status(&self) -> Result<WorkflowStatus> {
        let status = self
            .workflow_status
            .read()
            .map_err(|e| LLMSpellError::Workflow {
                message: format!("Failed to acquire status lock: {}", e),
                step: None,
                source: None,
            })?;

        Ok(status.clone())
    }

    /// Advance to next step
    pub async fn advance_step(&self) -> Result<()> {
        let mut state = self.state.write().map_err(|e| LLMSpellError::Workflow {
            message: format!("Failed to acquire state lock: {}", e),
            step: None,
            source: None,
        })?;

        state.advance_step();
        debug!("Advanced to step {}", state.current_step);

        Ok(())
    }

    /// Get current step index
    pub async fn get_current_step(&self) -> Result<usize> {
        let state = self.state.read().map_err(|e| LLMSpellError::Workflow {
            message: format!("Failed to acquire state lock: {}", e),
            step: None,
            source: None,
        })?;

        Ok(state.current_step)
    }

    /// Set shared data
    pub async fn set_shared_data(&self, key: String, value: Value) -> Result<()> {
        // Get old value and update state in a single lock scope
        let old_value = {
            let mut state = self.state.write().map_err(|e| LLMSpellError::Workflow {
                message: format!("Failed to acquire state lock: {}", e),
                step: None,
                source: None,
            })?;

            let old = state.get_shared_data(&key).cloned();
            state.set_shared_data(key.clone(), value.clone());
            debug!("Set shared data key: {}", key);
            old
        }; // Lock is dropped here

        // Execute state change hooks if available
        if let Some(workflow_executor) = &self.workflow_executor {
            let workflow_state = self.get_state_snapshot().await?;
            let mut hook_ctx = WorkflowHookContext::new(
                self.component_id.clone(),
                self.metadata.clone(),
                workflow_state,
                "state_manager".to_string(),
                WorkflowExecutionPhase::StateChange,
            );
            hook_ctx = hook_ctx
                .with_pattern_context("key".to_string(), serde_json::Value::String(key.clone()));
            hook_ctx = hook_ctx.with_pattern_context(
                "old_value".to_string(),
                old_value.unwrap_or(serde_json::Value::Null),
            );
            hook_ctx = hook_ctx.with_pattern_context("new_value".to_string(), value);
            let _ = workflow_executor.execute_workflow_hooks(hook_ctx).await;
        }

        Ok(())
    }

    /// Get shared data
    pub async fn get_shared_data(&self, key: &str) -> Result<Option<Value>> {
        let state = self.state.read().map_err(|e| LLMSpellError::Workflow {
            message: format!("Failed to acquire state lock: {}", e),
            step: None,
            source: None,
        })?;

        Ok(state.get_shared_data(key).cloned())
    }

    /// Get all shared data
    pub async fn get_all_shared_data(&self) -> Result<HashMap<String, Value>> {
        let state = self.state.read().map_err(|e| LLMSpellError::Workflow {
            message: format!("Failed to acquire state lock: {}", e),
            step: None,
            source: None,
        })?;

        Ok(state.shared_data.clone())
    }

    /// Set step output
    pub async fn set_step_output(&self, step_id: ComponentId, output: Value) -> Result<()> {
        let mut state = self.state.write().map_err(|e| LLMSpellError::Workflow {
            message: format!("Failed to acquire state lock: {}", e),
            step: None,
            source: None,
        })?;

        state.set_step_output(step_id, output);
        debug!("Set step output for: {:?}", step_id);

        Ok(())
    }

    /// Get step output
    pub async fn get_step_output(&self, step_id: ComponentId) -> Result<Option<Value>> {
        let state = self.state.read().map_err(|e| LLMSpellError::Workflow {
            message: format!("Failed to acquire state lock: {}", e),
            step: None,
            source: None,
        })?;

        Ok(state.get_step_output(step_id).cloned())
    }

    /// Record step execution result
    pub async fn record_step_result(&self, result: StepResult) -> Result<()> {
        // Store step output if successful
        if result.success {
            let output_value = serde_json::json!(result.output);
            self.set_step_output(result.step_id, output_value).await?;
        }

        // Add to execution history
        {
            let mut history =
                self.execution_history
                    .write()
                    .map_err(|e| LLMSpellError::Workflow {
                        message: format!("Failed to acquire history lock: {}", e),
                        step: None,
                        source: None,
                    })?;

            history.push(result.clone());
        }

        debug!(
            "Recorded step result for '{}': success={}",
            result.step_name, result.success
        );

        Ok(())
    }

    /// Get execution history
    pub async fn get_execution_history(&self) -> Result<Vec<StepResult>> {
        let history = self
            .execution_history
            .read()
            .map_err(|e| LLMSpellError::Workflow {
                message: format!("Failed to acquire history lock: {}", e),
                step: None,
                source: None,
            })?;

        Ok(history.clone())
    }

    /// Get execution statistics
    pub async fn get_execution_stats(&self) -> Result<ExecutionStats> {
        let history = self
            .execution_history
            .read()
            .map_err(|e| LLMSpellError::Workflow {
                message: format!("Failed to acquire history lock: {}", e),
                step: None,
                source: None,
            })?;

        let state = self.state.read().map_err(|e| LLMSpellError::Workflow {
            message: format!("Failed to acquire state lock: {}", e),
            step: None,
            source: None,
        })?;

        let total_steps = history.len();
        let successful_steps = history.iter().filter(|r| r.success).count();
        let failed_steps = total_steps - successful_steps;

        let total_duration = history
            .iter()
            .map(|r| r.duration)
            .fold(Duration::ZERO, |acc, d| acc + d);

        let average_duration = if total_steps > 0 {
            #[allow(clippy::cast_possible_truncation)]
            let total_steps_u32 = total_steps as u32;
            total_duration / total_steps_u32
        } else {
            Duration::ZERO
        };

        let total_retries = history.iter().map(|r| r.retry_count).sum::<u32>();

        Ok(ExecutionStats {
            total_steps,
            successful_steps,
            failed_steps,
            total_duration,
            average_step_duration: average_duration,
            total_retries,
            execution_start_time: state.start_time,
            current_step: state.current_step,
        })
    }

    /// Check if workflow has exceeded maximum execution time
    pub async fn check_execution_timeout(&self) -> Result<bool> {
        if let Some(max_time) = self.config.max_execution_time {
            let state = self.state.read().map_err(|e| LLMSpellError::Workflow {
                message: format!("Failed to acquire state lock: {}", e),
                step: None,
                source: None,
            })?;

            if let Some(duration) = state.execution_duration() {
                if duration > max_time {
                    warn!(
                        "Workflow exceeded maximum execution time: {:?} > {:?}",
                        duration, max_time
                    );
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    /// Reset state to initial values
    pub async fn reset(&self) -> Result<()> {
        debug!("Resetting workflow state");

        {
            let mut state = self.state.write().map_err(|e| LLMSpellError::Workflow {
                message: format!("Failed to acquire state lock: {}", e),
                step: None,
                source: None,
            })?;
            state.reset();
        }

        {
            let mut history =
                self.execution_history
                    .write()
                    .map_err(|e| LLMSpellError::Workflow {
                        message: format!("Failed to acquire history lock: {}", e),
                        step: None,
                        source: None,
                    })?;
            history.clear();
        }

        {
            let mut status = self
                .workflow_status
                .write()
                .map_err(|e| LLMSpellError::Workflow {
                    message: format!("Failed to acquire status lock: {}", e),
                    step: None,
                    source: None,
                })?;
            *status = WorkflowStatus::Pending;
        }

        Ok(())
    }

    /// Get a snapshot of the current state
    pub async fn get_state_snapshot(&self) -> Result<WorkflowState> {
        let state = self.state.read().map_err(|e| LLMSpellError::Workflow {
            message: format!("Failed to acquire state lock: {}", e),
            step: None,
            source: None,
        })?;

        Ok(state.clone())
    }

    /// Track shared data access for hooks (call after get operations)
    pub async fn track_shared_data_access(
        &self,
        key: Option<&str>,
        access_type: &str,
    ) -> Result<()> {
        if let Some(workflow_executor) = &self.workflow_executor {
            let workflow_state = self.get_state_snapshot().await?;
            let mut hook_ctx = WorkflowHookContext::new(
                self.component_id.clone(),
                self.metadata.clone(),
                workflow_state,
                "state_manager".to_string(),
                WorkflowExecutionPhase::SharedDataAccess,
            );
            hook_ctx = hook_ctx.with_pattern_context(
                "access_type".to_string(),
                serde_json::Value::String(access_type.to_string()),
            );
            if let Some(k) = key {
                hook_ctx = hook_ctx.with_pattern_context(
                    "key".to_string(),
                    serde_json::Value::String(k.to_string()),
                );
            }
            let _ = workflow_executor.execute_workflow_hooks(hook_ctx).await;
        }
        Ok(())
    }
}

/// Execution statistics for workflow monitoring
#[derive(Debug, Clone)]
pub struct ExecutionStats {
    pub total_steps: usize,
    pub successful_steps: usize,
    pub failed_steps: usize,
    pub total_duration: Duration,
    pub average_step_duration: Duration,
    pub total_retries: u32,
    pub execution_start_time: Option<Instant>,
    pub current_step: usize,
}

impl std::fmt::Debug for StateManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StateManager")
            .field("config", &self.config)
            .field("workflow_status", &self.workflow_status)
            .field("has_hooks", &self.workflow_executor.is_some())
            .field("component_id", &self.component_id)
            .field("metadata", &self.metadata)
            .finish()
    }
}

impl ExecutionStats {
    /// Calculate success rate as percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_steps == 0 {
            0.0
        } else {
            #[allow(clippy::cast_precision_loss)]
            let successful_f64 = self.successful_steps as f64;
            #[allow(clippy::cast_precision_loss)]
            let total_f64 = self.total_steps as f64;
            (successful_f64 / total_f64) * 100.0
        }
    }

    /// Get current execution time if workflow is running
    pub fn current_execution_time(&self) -> Option<Duration> {
        self.execution_start_time.map(|start| start.elapsed())
    }

    /// Generate a formatted report
    pub fn generate_report(&self) -> String {
        format!(
            "Workflow Execution Statistics:\n\
            - Total Steps: {}\n\
            - Successful: {} ({:.1}%)\n\
            - Failed: {}\n\
            - Current Step: {}\n\
            - Total Duration: {:?}\n\
            - Average Step Duration: {:?}\n\
            - Total Retries: {}\n\
            - Execution Time: {:?}",
            self.total_steps,
            self.successful_steps,
            self.success_rate(),
            self.failed_steps,
            self.current_step,
            self.total_duration,
            self.average_step_duration,
            self.total_retries,
            self.current_execution_time().unwrap_or(Duration::ZERO)
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    #[tokio::test]
    async fn test_state_manager_lifecycle() {
        let config = WorkflowConfig::default();
        let manager = StateManager::new(config);

        // Initial state
        assert_eq!(manager.get_status().await.unwrap(), WorkflowStatus::Pending);
        assert_eq!(manager.get_current_step().await.unwrap(), 0);

        // Start execution
        manager.start_execution().await.unwrap();
        assert_eq!(manager.get_status().await.unwrap(), WorkflowStatus::Running);

        // Complete execution
        manager.complete_execution(true).await.unwrap();
        assert_eq!(
            manager.get_status().await.unwrap(),
            WorkflowStatus::Completed
        );
    }
    #[tokio::test]
    async fn test_shared_data_management() {
        let config = WorkflowConfig::default();
        let manager = StateManager::new(config);

        // Set shared data
        let test_value = serde_json::json!({"test": "value"});
        manager
            .set_shared_data("test_key".to_string(), test_value.clone())
            .await
            .unwrap();

        // Get shared data
        let retrieved = manager.get_shared_data("test_key").await.unwrap();
        assert_eq!(retrieved, Some(test_value));

        // Get non-existent key
        let missing = manager.get_shared_data("missing_key").await.unwrap();
        assert_eq!(missing, None);

        // Get all shared data
        let all_data = manager.get_all_shared_data().await.unwrap();
        assert_eq!(all_data.len(), 1);
        assert!(all_data.contains_key("test_key"));
    }
    #[tokio::test]
    async fn test_step_execution_tracking() {
        let config = WorkflowConfig::default();
        let manager = StateManager::new(config);

        let step_id = ComponentId::new();
        let result = StepResult::success(
            step_id,
            "test_step".to_string(),
            "test output".to_string(),
            Duration::from_secs(1),
        );

        // Record step result
        manager.record_step_result(result.clone()).await.unwrap();

        // Check step output was stored
        let output = manager.get_step_output(step_id).await.unwrap();
        assert_eq!(output, Some(serde_json::json!("test output")));

        // Check execution history
        let history = manager.get_execution_history().await.unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].step_id, step_id);
        assert!(history[0].success);
    }
    #[tokio::test]
    async fn test_execution_statistics() {
        let config = WorkflowConfig::default();
        let manager = StateManager::new(config);

        // Record some results
        let step1 = StepResult::success(
            ComponentId::new(),
            "step1".to_string(),
            "output1".to_string(),
            Duration::from_secs(1),
        );

        let step2 = StepResult::failure(
            ComponentId::new(),
            "step2".to_string(),
            "error".to_string(),
            Duration::from_secs(2),
            1,
        );

        manager.record_step_result(step1).await.unwrap();
        manager.record_step_result(step2).await.unwrap();

        // Get statistics
        let stats = manager.get_execution_stats().await.unwrap();
        assert_eq!(stats.total_steps, 2);
        assert_eq!(stats.successful_steps, 1);
        assert_eq!(stats.failed_steps, 1);
        assert_eq!(stats.success_rate(), 50.0);
        assert_eq!(stats.total_retries, 1);
        assert_eq!(stats.total_duration, Duration::from_secs(3));
    }
    #[tokio::test]
    async fn test_state_reset() {
        let config = WorkflowConfig::default();
        let manager = StateManager::new(config);

        // Set some state
        manager.start_execution().await.unwrap();
        manager.advance_step().await.unwrap();
        manager
            .set_shared_data("test".to_string(), serde_json::json!("value"))
            .await
            .unwrap();

        let result = StepResult::success(
            ComponentId::new(),
            "test".to_string(),
            "output".to_string(),
            Duration::from_secs(1),
        );
        manager.record_step_result(result).await.unwrap();

        // Reset state
        manager.reset().await.unwrap();

        // Check everything is reset
        assert_eq!(manager.get_status().await.unwrap(), WorkflowStatus::Pending);
        assert_eq!(manager.get_current_step().await.unwrap(), 0);
        assert!(manager.get_all_shared_data().await.unwrap().is_empty());
        assert!(manager.get_execution_history().await.unwrap().is_empty());
    }
}

// ==============================================================================
// PERSISTENT WORKFLOW STATE MANAGEMENT
// ==============================================================================

use async_trait::async_trait;
use llmspell_core::state::{StateManager as PersistentStateManager, StateScope};
use serde::{Deserialize, Serialize};

/// Persistent workflow state for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentWorkflowState {
    /// Workflow identifier
    pub workflow_id: String,
    /// Workflow configuration
    pub config: WorkflowConfig,
    /// Current workflow state
    pub workflow_state: WorkflowState,
    /// Current workflow status
    pub status: WorkflowStatus,
    /// Execution history
    pub execution_history: Vec<StepResult>,
    /// Workflow metadata
    pub metadata: ComponentMetadata,
    /// Execution statistics
    pub execution_stats: WorkflowExecutionStats,
    /// Checkpoint states for resumption
    pub checkpoints: HashMap<usize, WorkflowCheckpoint>,
    /// State timestamp
    pub last_updated: SystemTime,
    /// Custom workflow properties
    pub custom_properties: HashMap<String, Value>,
}

/// Workflow execution statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct WorkflowExecutionStats {
    /// Total number of workflow executions
    pub total_executions: u64,
    /// Number of successful executions
    pub successful_executions: u64,
    /// Number of failed executions
    pub failed_executions: u64,
    /// Number of cancelled executions
    pub cancelled_executions: u64,
    /// Total execution time across all runs
    pub total_execution_time_ms: u64,
    /// Average execution time per run
    pub average_execution_time_ms: f64,
    /// Last execution timestamp
    pub last_execution: Option<SystemTime>,
    /// Steps execution statistics
    pub step_stats: HashMap<String, StepStatistics>,
    /// Retry statistics
    pub retry_stats: RetryStatistics,
}

/// Step execution statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StepStatistics {
    /// Step name
    pub step_name: String,
    /// Total executions of this step
    pub executions: u64,
    /// Successful executions
    pub successes: u64,
    /// Failed executions
    pub failures: u64,
    /// Total execution time for this step
    pub total_time_ms: u64,
    /// Average execution time for this step
    pub average_time_ms: f64,
    /// Retry count for this step
    pub retries: u64,
}

/// Retry statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RetryStatistics {
    /// Total retry attempts across all steps
    pub total_retries: u64,
    /// Retries that eventually succeeded
    pub successful_retries: u64,
    /// Retries that ultimately failed
    pub failed_retries: u64,
    /// Average retries per failed step
    pub average_retries_per_failure: f64,
}

/// Workflow checkpoint for resumption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowCheckpoint {
    /// Step index when checkpoint was created
    pub step_index: usize,
    /// Workflow state at checkpoint
    pub state_snapshot: WorkflowState,
    /// Execution history up to checkpoint
    pub execution_history: Vec<StepResult>,
    /// Timestamp of checkpoint
    pub created_at: SystemTime,
    /// Checkpoint description
    pub description: String,
    /// Whether this is an automatic or manual checkpoint
    pub is_automatic: bool,
}

impl PersistentWorkflowState {
    /// Create new persistent workflow state
    pub fn new(workflow_id: String, config: WorkflowConfig, metadata: ComponentMetadata) -> Self {
        Self {
            workflow_id,
            config,
            workflow_state: WorkflowState::new(),
            status: WorkflowStatus::Pending,
            execution_history: Vec::new(),
            metadata,
            execution_stats: WorkflowExecutionStats::default(),
            checkpoints: HashMap::new(),
            last_updated: SystemTime::now(),
            custom_properties: HashMap::new(),
        }
    }
}

/// Extended state manager with persistence capabilities
pub struct PersistentWorkflowStateManager {
    /// In-memory state manager for current execution
    memory_state_manager: StateManager,
    /// Persistent state manager for long-term storage
    persistent_state_manager: Arc<dyn PersistentStateManager>,
    /// Current persistent state
    persistent_state: Option<PersistentWorkflowState>,
    /// Workflow identifier
    workflow_id: String,
    /// Whether to auto-save state changes
    #[allow(dead_code)]
    auto_save: bool,
}

impl PersistentWorkflowStateManager {
    /// Create new persistent workflow state manager
    pub fn new(
        config: WorkflowConfig,
        persistent_state_manager: Arc<dyn PersistentStateManager>,
        workflow_id: String,
    ) -> Self {
        Self {
            memory_state_manager: StateManager::new(config),
            persistent_state_manager,
            persistent_state: None,
            workflow_id,
            auto_save: true,
        }
    }

    /// Load existing workflow state from persistent storage
    pub async fn load_state(&mut self) -> Result<bool> {
        let state_scope = StateScope::Custom(format!("workflow_{}", self.workflow_id));

        match self
            .persistent_state_manager
            .get(state_scope, "state")
            .await
        {
            Ok(Some(state_value)) => {
                let persistent_state: PersistentWorkflowState =
                    serde_json::from_value(state_value)?;

                self.persistent_state = Some(persistent_state);
                info!("Loaded persistent state for workflow {}", self.workflow_id);
                Ok(true)
            }
            Ok(None) => {
                debug!("No saved state found for workflow {}", self.workflow_id);
                Ok(false)
            }
            Err(e) => Err(LLMSpellError::Internal {
                message: format!("Failed to load workflow state: {}", e),
                source: None,
            }),
        }
    }

    /// Save current state to persistent storage
    pub async fn save_state(&mut self) -> Result<()> {
        let metadata =
            ComponentMetadata::new(self.workflow_id.clone(), "Persistent workflow".to_string());

        let persistent_state = PersistentWorkflowState::new(
            self.workflow_id.clone(),
            self.memory_state_manager.config.clone(),
            metadata,
        );

        let state_scope = StateScope::Custom(format!("workflow_{}", self.workflow_id));

        self.persistent_state_manager
            .set(
                state_scope,
                "state",
                serde_json::to_value(&persistent_state)?,
            )
            .await
            .map_err(|e| LLMSpellError::Internal {
                message: format!("Failed to save workflow state: {}", e),
                source: None,
            })?;

        self.persistent_state = Some(persistent_state);
        info!("Saved persistent state for workflow {}", self.workflow_id);
        Ok(())
    }

    /// Get in-memory state manager
    pub fn memory_state_manager(&self) -> &StateManager {
        &self.memory_state_manager
    }
}

/// Extension trait for workflow components to add persistence capabilities
#[async_trait]
pub trait WorkflowStatePersistence {
    /// Get the persistent state manager
    fn persistent_state_manager(&self) -> Option<&PersistentWorkflowStateManager>;

    /// Set the persistent state manager
    fn set_persistent_state_manager(&mut self, manager: PersistentWorkflowStateManager);

    /// Load workflow state
    async fn load_workflow_state(&mut self) -> Result<bool>;
}
