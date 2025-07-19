//! ABOUTME: Error handling strategies for basic workflows
//! ABOUTME: Provides retry logic, error recovery, and failure management

use super::traits::{ErrorStrategy, StepResult};
use llmspell_core::{LLMSpellError, Result};
use std::time::Duration;
use tracing::{debug, error, warn};

/// Error handler for basic workflows
pub struct ErrorHandler {
    default_strategy: ErrorStrategy,
}

impl ErrorHandler {
    /// Create a new error handler with default strategy
    pub fn new(default_strategy: ErrorStrategy) -> Self {
        Self { default_strategy }
    }

    /// Handle a step failure and determine next action
    pub async fn handle_step_failure(
        &self,
        step_result: &StepResult,
        strategy: Option<&ErrorStrategy>,
    ) -> Result<ErrorAction> {
        let strategy = strategy.unwrap_or(&self.default_strategy);

        match strategy {
            ErrorStrategy::FailFast => {
                error!(
                    "Step '{}' failed with FailFast strategy: {}",
                    step_result.step_name,
                    step_result
                        .error
                        .as_ref()
                        .unwrap_or(&"Unknown error".to_string())
                );
                Ok(ErrorAction::StopWorkflow)
            }
            ErrorStrategy::Continue => {
                warn!(
                    "Step '{}' failed, continuing with next step: {}",
                    step_result.step_name,
                    step_result
                        .error
                        .as_ref()
                        .unwrap_or(&"Unknown error".to_string())
                );
                Ok(ErrorAction::ContinueToNext)
            }
            ErrorStrategy::Retry { max_attempts, .. } => {
                if step_result.retry_count < *max_attempts {
                    debug!(
                        "Step '{}' failed, will retry (attempt {}/{})",
                        step_result.step_name,
                        step_result.retry_count + 1,
                        max_attempts
                    );
                    Ok(ErrorAction::RetryStep)
                } else {
                    error!(
                        "Step '{}' failed after {} retry attempts",
                        step_result.step_name, max_attempts
                    );
                    Ok(ErrorAction::StopWorkflow)
                }
            }
        }
    }

    /// Analyze workflow-level errors and provide recovery suggestions
    pub async fn analyze_workflow_error(
        &self,
        error: &LLMSpellError,
        completed_steps: &[StepResult],
        remaining_steps: usize,
    ) -> Result<WorkflowErrorAnalysis> {
        let error_type = match error {
            LLMSpellError::Timeout { .. } => WorkflowErrorType::Timeout,
            LLMSpellError::Validation { .. } => WorkflowErrorType::Validation,
            LLMSpellError::Tool { .. } => WorkflowErrorType::ToolFailure,
            LLMSpellError::Component { .. } => WorkflowErrorType::AgentFailure,
            LLMSpellError::Workflow { .. } => WorkflowErrorType::WorkflowLogic,
            LLMSpellError::Resource { .. } => WorkflowErrorType::ResourceExhaustion,
            LLMSpellError::Network { .. } => WorkflowErrorType::NetworkFailure,
            _ => WorkflowErrorType::Unknown,
        };

        let progress = if completed_steps.is_empty() && remaining_steps == 0 {
            0.0
        } else {
            completed_steps.len() as f64 / (completed_steps.len() + remaining_steps) as f64
        };

        let successful_steps = completed_steps.iter().filter(|r| r.success).count();
        let failed_steps = completed_steps.len() - successful_steps;

        let recovery_suggestion = self.suggest_recovery_action(&error_type, progress, failed_steps);

        Ok(WorkflowErrorAnalysis {
            error_type,
            error_message: error.to_string(),
            progress_percentage: (progress * 100.0) as u32,
            successful_steps,
            failed_steps,
            remaining_steps,
            recovery_suggestion,
            is_recoverable: matches!(
                error_type,
                WorkflowErrorType::Timeout
                    | WorkflowErrorType::NetworkFailure
                    | WorkflowErrorType::ResourceExhaustion
            ),
        })
    }

    /// Calculate retry delay based on strategy and attempt count
    pub fn calculate_retry_delay(
        &self,
        strategy: &ErrorStrategy,
        attempt: u32,
        exponential_backoff: bool,
    ) -> Duration {
        match strategy {
            ErrorStrategy::Retry { backoff_ms, .. } => {
                if exponential_backoff {
                    Duration::from_millis(backoff_ms * 2_u64.pow(attempt))
                } else {
                    Duration::from_millis(*backoff_ms)
                }
            }
            _ => Duration::from_millis(0),
        }
    }

    /// Suggest recovery action based on error type and workflow state
    fn suggest_recovery_action(
        &self,
        error_type: &WorkflowErrorType,
        progress: f64,
        failed_steps: usize,
    ) -> RecoveryAction {
        match error_type {
            WorkflowErrorType::Timeout => {
                if progress > 0.8 {
                    RecoveryAction::RetryFromLastSuccessful
                } else {
                    RecoveryAction::IncreaseTimeouts
                }
            }
            WorkflowErrorType::NetworkFailure => RecoveryAction::RetryWithBackoff,
            WorkflowErrorType::ResourceExhaustion => RecoveryAction::ReduceConcurrency,
            WorkflowErrorType::Validation => RecoveryAction::FixInputData,
            WorkflowErrorType::ToolFailure | WorkflowErrorType::AgentFailure => {
                if failed_steps > 3 {
                    RecoveryAction::CheckConfiguration
                } else {
                    RecoveryAction::RetryWithBackoff
                }
            }
            WorkflowErrorType::WorkflowLogic => RecoveryAction::ReviewWorkflowDesign,
            WorkflowErrorType::Unknown => RecoveryAction::ManualIntervention,
        }
    }
}

/// Action to take after a step failure
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorAction {
    /// Stop the entire workflow
    StopWorkflow,
    /// Continue to the next step
    ContinueToNext,
    /// Retry the current step
    RetryStep,
}

/// Types of workflow errors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkflowErrorType {
    Timeout,
    Validation,
    ToolFailure,
    AgentFailure,
    WorkflowLogic,
    ResourceExhaustion,
    NetworkFailure,
    Unknown,
}

/// Suggested recovery actions
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryAction {
    RetryFromLastSuccessful,
    RetryWithBackoff,
    IncreaseTimeouts,
    ReduceConcurrency,
    FixInputData,
    CheckConfiguration,
    ReviewWorkflowDesign,
    ManualIntervention,
}

/// Analysis of a workflow error
#[derive(Debug, Clone)]
pub struct WorkflowErrorAnalysis {
    pub error_type: WorkflowErrorType,
    pub error_message: String,
    pub progress_percentage: u32,
    pub successful_steps: usize,
    pub failed_steps: usize,
    pub remaining_steps: usize,
    pub recovery_suggestion: RecoveryAction,
    pub is_recoverable: bool,
}

impl WorkflowErrorAnalysis {
    /// Generate a human-readable error report
    pub fn generate_report(&self) -> String {
        format!(
            "Workflow Error Analysis:\n\
            - Error Type: {:?}\n\
            - Message: {}\n\
            - Progress: {}% complete\n\
            - Steps: {} successful, {} failed, {} remaining\n\
            - Recoverable: {}\n\
            - Suggested Action: {:?}",
            self.error_type,
            self.error_message,
            self.progress_percentage,
            self.successful_steps,
            self.failed_steps,
            self.remaining_steps,
            self.is_recoverable,
            self.recovery_suggestion
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_core::ComponentId;
    use std::time::Duration;

    #[tokio::test]
    async fn test_fail_fast_strategy() {
        let handler = ErrorHandler::new(ErrorStrategy::FailFast);

        let failed_result = StepResult::failure(
            ComponentId::new(),
            "test_step".to_string(),
            "Test failure".to_string(),
            Duration::from_secs(1),
            0,
        );

        let action = handler
            .handle_step_failure(&failed_result, None)
            .await
            .unwrap();
        assert_eq!(action, ErrorAction::StopWorkflow);
    }

    #[tokio::test]
    async fn test_continue_strategy() {
        let handler = ErrorHandler::new(ErrorStrategy::Continue);

        let failed_result = StepResult::failure(
            ComponentId::new(),
            "test_step".to_string(),
            "Test failure".to_string(),
            Duration::from_secs(1),
            0,
        );

        let action = handler
            .handle_step_failure(&failed_result, None)
            .await
            .unwrap();
        assert_eq!(action, ErrorAction::ContinueToNext);
    }

    #[tokio::test]
    async fn test_retry_strategy() {
        let handler = ErrorHandler::new(ErrorStrategy::Retry {
            max_attempts: 3,
            backoff_ms: 1000,
        });

        // First failure - should retry
        let failed_result_retry = StepResult::failure(
            ComponentId::new(),
            "test_step".to_string(),
            "Test failure".to_string(),
            Duration::from_secs(1),
            1, // First retry attempt
        );

        let action = handler
            .handle_step_failure(&failed_result_retry, None)
            .await
            .unwrap();
        assert_eq!(action, ErrorAction::RetryStep);

        // Max retries reached - should stop
        let failed_result_max = StepResult::failure(
            ComponentId::new(),
            "test_step".to_string(),
            "Test failure".to_string(),
            Duration::from_secs(1),
            3, // Max attempts reached
        );

        let action = handler
            .handle_step_failure(&failed_result_max, None)
            .await
            .unwrap();
        assert_eq!(action, ErrorAction::StopWorkflow);
    }

    #[tokio::test]
    async fn test_retry_delay_calculation() {
        let handler = ErrorHandler::new(ErrorStrategy::FailFast);
        let strategy = ErrorStrategy::Retry {
            max_attempts: 3,
            backoff_ms: 1000,
        };

        // Test exponential backoff
        let delay1 = handler.calculate_retry_delay(&strategy, 0, true);
        let delay2 = handler.calculate_retry_delay(&strategy, 1, true);
        let delay3 = handler.calculate_retry_delay(&strategy, 2, true);

        assert_eq!(delay1, Duration::from_millis(1000)); // 1000 * 2^0
        assert_eq!(delay2, Duration::from_millis(2000)); // 1000 * 2^1
        assert_eq!(delay3, Duration::from_millis(4000)); // 1000 * 2^2

        // Test fixed delay
        let delay_fixed = handler.calculate_retry_delay(&strategy, 2, false);
        assert_eq!(delay_fixed, Duration::from_millis(1000));
    }

    #[tokio::test]
    async fn test_workflow_error_analysis() {
        let handler = ErrorHandler::new(ErrorStrategy::FailFast);

        let timeout_error = LLMSpellError::Timeout {
            message: "Workflow timed out".to_string(),
            duration_ms: Some(5000),
        };

        let completed_steps = vec![
            StepResult::success(
                ComponentId::new(),
                "step1".to_string(),
                "success".to_string(),
                Duration::from_secs(1),
            ),
            StepResult::failure(
                ComponentId::new(),
                "step2".to_string(),
                "failure".to_string(),
                Duration::from_secs(1),
                0,
            ),
        ];

        let analysis = handler
            .analyze_workflow_error(&timeout_error, &completed_steps, 2)
            .await
            .unwrap();

        assert_eq!(analysis.error_type, WorkflowErrorType::Timeout);
        assert_eq!(analysis.successful_steps, 1);
        assert_eq!(analysis.failed_steps, 1);
        assert_eq!(analysis.remaining_steps, 2);
        assert_eq!(analysis.progress_percentage, 50); // 2/(2+2) * 100
        assert!(analysis.is_recoverable);
    }
}
