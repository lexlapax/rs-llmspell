//! ABOUTME: Memory-based state management for basic workflows
//! ABOUTME: Provides in-memory state storage and step coordination

use super::traits::{BasicStepResult, BasicWorkflowStatus};
use super::types::{BasicWorkflowConfig, BasicWorkflowState};
use llmspell_core::{ComponentId, LLMSpellError, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use tracing::{debug, warn};

/// Memory-based state manager for basic workflows
#[derive(Debug, Clone)]
pub struct BasicStateManager {
    state: Arc<RwLock<BasicWorkflowState>>,
    config: BasicWorkflowConfig,
    execution_history: Arc<RwLock<Vec<BasicStepResult>>>,
    workflow_status: Arc<RwLock<BasicWorkflowStatus>>,
}

impl BasicStateManager {
    /// Create a new state manager
    pub fn new(config: BasicWorkflowConfig) -> Self {
        Self {
            state: Arc::new(RwLock::new(BasicWorkflowState::new())),
            config,
            execution_history: Arc::new(RwLock::new(Vec::new())),
            workflow_status: Arc::new(RwLock::new(BasicWorkflowStatus::Pending)),
        }
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
            *status = BasicWorkflowStatus::Running;
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
            BasicWorkflowStatus::Completed
        } else {
            BasicWorkflowStatus::Failed
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

        *status = BasicWorkflowStatus::Cancelled;

        Ok(())
    }

    /// Get current workflow status
    pub async fn get_status(&self) -> Result<BasicWorkflowStatus> {
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
        let mut state = self.state.write().map_err(|e| LLMSpellError::Workflow {
            message: format!("Failed to acquire state lock: {}", e),
            step: None,
            source: None,
        })?;

        state.set_shared_data(key.clone(), value);
        debug!("Set shared data key: {}", key);

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
    pub async fn record_step_result(&self, result: BasicStepResult) -> Result<()> {
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
    pub async fn get_execution_history(&self) -> Result<Vec<BasicStepResult>> {
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
            total_duration / total_steps as u32
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
            *status = BasicWorkflowStatus::Pending;
        }

        Ok(())
    }

    /// Get a snapshot of the current state
    pub async fn get_state_snapshot(&self) -> Result<BasicWorkflowState> {
        let state = self.state.read().map_err(|e| LLMSpellError::Workflow {
            message: format!("Failed to acquire state lock: {}", e),
            step: None,
            source: None,
        })?;

        Ok(state.clone())
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

impl ExecutionStats {
    /// Calculate success rate as percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_steps == 0 {
            0.0
        } else {
            (self.successful_steps as f64 / self.total_steps as f64) * 100.0
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
        let config = BasicWorkflowConfig::default();
        let manager = BasicStateManager::new(config);

        // Initial state
        assert_eq!(
            manager.get_status().await.unwrap(),
            BasicWorkflowStatus::Pending
        );
        assert_eq!(manager.get_current_step().await.unwrap(), 0);

        // Start execution
        manager.start_execution().await.unwrap();
        assert_eq!(
            manager.get_status().await.unwrap(),
            BasicWorkflowStatus::Running
        );

        // Complete execution
        manager.complete_execution(true).await.unwrap();
        assert_eq!(
            manager.get_status().await.unwrap(),
            BasicWorkflowStatus::Completed
        );
    }

    #[tokio::test]
    async fn test_shared_data_management() {
        let config = BasicWorkflowConfig::default();
        let manager = BasicStateManager::new(config);

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
        let config = BasicWorkflowConfig::default();
        let manager = BasicStateManager::new(config);

        let step_id = ComponentId::new();
        let result = BasicStepResult::success(
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
        let config = BasicWorkflowConfig::default();
        let manager = BasicStateManager::new(config);

        // Record some results
        let step1 = BasicStepResult::success(
            ComponentId::new(),
            "step1".to_string(),
            "output1".to_string(),
            Duration::from_secs(1),
        );

        let step2 = BasicStepResult::failure(
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
        let config = BasicWorkflowConfig::default();
        let manager = BasicStateManager::new(config);

        // Set some state
        manager.start_execution().await.unwrap();
        manager.advance_step().await.unwrap();
        manager
            .set_shared_data("test".to_string(), serde_json::json!("value"))
            .await
            .unwrap();

        let result = BasicStepResult::success(
            ComponentId::new(),
            "test".to_string(),
            "output".to_string(),
            Duration::from_secs(1),
        );
        manager.record_step_result(result).await.unwrap();

        // Reset state
        manager.reset().await.unwrap();

        // Check everything is reset
        assert_eq!(
            manager.get_status().await.unwrap(),
            BasicWorkflowStatus::Pending
        );
        assert_eq!(manager.get_current_step().await.unwrap(), 0);
        assert!(manager.get_all_shared_data().await.unwrap().is_empty());
        assert!(manager.get_execution_history().await.unwrap().is_empty());
    }
}
