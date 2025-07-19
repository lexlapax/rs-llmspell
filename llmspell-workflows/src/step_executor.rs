//! ABOUTME: Step execution engine for basic workflows
//! ABOUTME: Handles individual step execution with timeout, retry, and error handling

use super::traits::{ErrorStrategy, StepResult, StepType, WorkflowStep};
use super::types::{StepExecutionContext, WorkflowConfig};
use llmspell_core::{ComponentId, LLMSpellError, Result};
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tracing::{debug, error, warn};

/// Basic step executor for workflow steps
pub struct StepExecutor {
    config: WorkflowConfig,
}

impl StepExecutor {
    /// Create a new step executor with configuration
    pub fn new(config: WorkflowConfig) -> Self {
        Self { config }
    }

    /// Execute a single step with retry logic
    pub async fn execute_step(
        &self,
        step: &WorkflowStep,
        context: StepExecutionContext,
    ) -> Result<StepResult> {
        let start_time = Instant::now();
        let step_timeout = step.timeout.unwrap_or(self.config.default_step_timeout);

        debug!(
            "Executing step '{}' (id: {:?}) with timeout: {:?}",
            step.name, step.id, step_timeout
        );

        // Execute with timeout
        let result = timeout(step_timeout, self.execute_step_internal(step, &context)).await;

        let duration = start_time.elapsed();

        match result {
            Ok(Ok(output)) => {
                debug!(
                    "Step '{}' completed successfully in {:?}",
                    step.name, duration
                );
                Ok(StepResult::success(
                    step.id,
                    step.name.clone(),
                    output,
                    duration,
                ))
            }
            Ok(Err(err)) => {
                warn!(
                    "Step '{}' failed: {} (duration: {:?})",
                    step.name, err, duration
                );
                Ok(StepResult::failure(
                    step.id,
                    step.name.clone(),
                    err.to_string(),
                    duration,
                    context.retry_attempt,
                ))
            }
            Err(_) => {
                error!("Step '{}' timed out after {:?}", step.name, step_timeout);
                Ok(StepResult::failure(
                    step.id,
                    step.name.clone(),
                    format!("Step timed out after {:?}", step_timeout),
                    duration,
                    context.retry_attempt,
                ))
            }
        }
    }

    /// Execute a step with retry logic
    pub async fn execute_step_with_retry(
        &self,
        step: &WorkflowStep,
        mut context: StepExecutionContext,
        error_strategy: &ErrorStrategy,
    ) -> Result<StepResult> {
        let max_attempts = match error_strategy {
            ErrorStrategy::Retry { max_attempts, .. } => *max_attempts,
            _ => 1, // No retry for other strategies
        };

        let mut last_result = None;

        for attempt in 0..max_attempts {
            context = context.with_retry(attempt, max_attempts);

            debug!(
                "Attempting step '{}' (attempt {}/{})",
                step.name,
                attempt + 1,
                max_attempts
            );

            let result = self.execute_step(step, context.clone()).await?;

            if result.success {
                return Ok(result);
            }

            last_result = Some(result);

            // Don't wait after the last attempt
            if attempt < max_attempts - 1 {
                if let ErrorStrategy::Retry { backoff_ms, .. } = error_strategy {
                    let delay = if self.config.exponential_backoff {
                        Duration::from_millis(backoff_ms * 2_u64.pow(attempt))
                    } else {
                        Duration::from_millis(*backoff_ms)
                    };

                    debug!("Step '{}' failed, retrying in {:?}", step.name, delay);

                    tokio::time::sleep(delay).await;
                }
            }
        }

        // Return the last failure result with updated retry count
        let mut final_result = last_result.unwrap();
        final_result.retry_count = max_attempts;
        Ok(final_result)
    }

    /// Internal step execution logic
    async fn execute_step_internal(
        &self,
        step: &WorkflowStep,
        context: &StepExecutionContext,
    ) -> Result<String> {
        match &step.step_type {
            StepType::Tool {
                tool_name,
                parameters,
            } => self.execute_tool_step(tool_name, parameters, context).await,
            StepType::Agent { agent_id, input } => {
                self.execute_agent_step(*agent_id, input, context).await
            }
            StepType::Custom {
                function_name,
                parameters,
            } => {
                self.execute_custom_step(function_name, parameters, context)
                    .await
            }
        }
    }

    /// Execute a tool step
    async fn execute_tool_step(
        &self,
        tool_name: &str,
        parameters: &serde_json::Value,
        _context: &StepExecutionContext,
    ) -> Result<String> {
        debug!("Executing tool step: {}", tool_name);

        // For now, return a mock result - this will be integrated with actual tools later
        // TODO: Integrate with llmspell-tools registry

        // Validate tool exists and parameters
        if tool_name.is_empty() {
            return Err(LLMSpellError::Workflow {
                message: "Tool name cannot be empty".to_string(),
                step: Some("tool_execution".to_string()),
                source: None,
            });
        }

        // Mock execution based on tool name
        let output = match tool_name {
            "calculator" => {
                let expression = parameters
                    .get("expression")
                    .and_then(|v| v.as_str())
                    .unwrap_or("0");
                format!("Calculator result for '{}': 42", expression)
            }
            "file_operations" => {
                let operation = parameters
                    .get("operation")
                    .and_then(|v| v.as_str())
                    .unwrap_or("read");
                format!("File operation '{}' completed", operation)
            }
            "json_processor" => {
                let default_input = serde_json::json!({});
                let input = parameters.get("input").unwrap_or(&default_input);
                format!("JSON processed: {}", input)
            }
            _ => {
                format!(
                    "Tool '{}' executed with parameters: {}",
                    tool_name, parameters
                )
            }
        };

        // Simulate some processing time
        tokio::time::sleep(Duration::from_millis(10)).await;

        Ok(output)
    }

    /// Execute an agent step
    async fn execute_agent_step(
        &self,
        agent_id: ComponentId,
        input: &str,
        _context: &StepExecutionContext,
    ) -> Result<String> {
        debug!("Executing agent step: {:?}", agent_id);

        // For now, return a mock result - this will be integrated with actual agents later
        // TODO: Integrate with llmspell-agents registry

        if input.is_empty() {
            return Err(LLMSpellError::Workflow {
                message: "Agent input cannot be empty".to_string(),
                step: Some("agent_execution".to_string()),
                source: None,
            });
        }

        // Mock agent execution
        let output = format!("Agent {:?} processed: {}", agent_id, input);

        // Simulate some processing time
        tokio::time::sleep(Duration::from_millis(20)).await;

        Ok(output)
    }

    /// Execute a custom function step
    async fn execute_custom_step(
        &self,
        function_name: &str,
        parameters: &serde_json::Value,
        _context: &StepExecutionContext,
    ) -> Result<String> {
        debug!("Executing custom step: {}", function_name);

        // For now, return a mock result - this will be extended with custom function support
        if function_name.is_empty() {
            return Err(LLMSpellError::Workflow {
                message: "Custom function name cannot be empty".to_string(),
                step: Some("custom_execution".to_string()),
                source: None,
            });
        }

        // Mock custom function execution
        let output = match function_name {
            "data_transform" => {
                format!("Data transformed with parameters: {}", parameters)
            }
            "validation" => "Validation completed with result: true".to_string(),
            "aggregation" => {
                format!(
                    "Aggregation completed: {}",
                    parameters.get("type").unwrap_or(&serde_json::json!("sum"))
                )
            }
            _ => {
                format!(
                    "Custom function '{}' executed with parameters: {}",
                    function_name, parameters
                )
            }
        };

        // Simulate some processing time
        tokio::time::sleep(Duration::from_millis(15)).await;

        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::WorkflowState;

    #[tokio::test]
    async fn test_step_executor_tool_execution() {
        let config = WorkflowConfig::default();
        let executor = StepExecutor::new(config);

        let step = WorkflowStep::new(
            "calculator_test".to_string(),
            StepType::Tool {
                tool_name: "calculator".to_string(),
                parameters: serde_json::json!({"expression": "2 + 2"}),
            },
        );

        let context = StepExecutionContext::new(WorkflowState::new(), None);
        let result = executor.execute_step(&step, context).await.unwrap();

        assert!(result.success);
        assert!(result.output.contains("Calculator result"));
        assert_eq!(result.retry_count, 0);
    }

    #[tokio::test]
    async fn test_step_executor_agent_execution() {
        let config = WorkflowConfig::default();
        let executor = StepExecutor::new(config);

        let agent_id = ComponentId::new();
        let step = WorkflowStep::new(
            "agent_test".to_string(),
            StepType::Agent {
                agent_id,
                input: "Process this data".to_string(),
            },
        );

        let context = StepExecutionContext::new(WorkflowState::new(), None);
        let result = executor.execute_step(&step, context).await.unwrap();

        assert!(result.success);
        assert!(result.output.contains("Agent"));
        assert!(result.output.contains("processed"));
    }

    #[tokio::test]
    async fn test_step_executor_custom_execution() {
        let config = WorkflowConfig::default();
        let executor = StepExecutor::new(config);

        let step = WorkflowStep::new(
            "custom_test".to_string(),
            StepType::Custom {
                function_name: "data_transform".to_string(),
                parameters: serde_json::json!({"type": "normalize"}),
            },
        );

        let context = StepExecutionContext::new(WorkflowState::new(), None);
        let result = executor.execute_step(&step, context).await.unwrap();

        assert!(result.success);
        assert!(result.output.contains("Data transformed"));
    }

    #[tokio::test]
    async fn test_step_executor_with_retry() {
        let mut config = WorkflowConfig::default();
        config.exponential_backoff = false; // Use fixed delay for faster test
        let executor = StepExecutor::new(config);

        // Create a step that will fail (empty tool name)
        let step = WorkflowStep::new(
            "failing_test".to_string(),
            StepType::Tool {
                tool_name: "".to_string(), // This will cause failure
                parameters: serde_json::json!({}),
            },
        );

        let context = StepExecutionContext::new(WorkflowState::new(), None);
        let error_strategy = ErrorStrategy::Retry {
            max_attempts: 3,
            backoff_ms: 10, // Short delay for test
        };

        let result = executor
            .execute_step_with_retry(&step, context, &error_strategy)
            .await
            .unwrap();

        assert!(!result.success);
        assert_eq!(result.retry_count, 3);
        assert!(result.error.is_some());
    }

    #[tokio::test]
    async fn test_step_executor_timeout() {
        let config = WorkflowConfig::default();
        let executor = StepExecutor::new(config);

        let step = WorkflowStep::new(
            "timeout_test".to_string(),
            StepType::Custom {
                function_name: "slow_function".to_string(),
                parameters: serde_json::json!({}),
            },
        )
        .with_timeout(Duration::from_millis(1)); // Very short timeout

        let context = StepExecutionContext::new(WorkflowState::new(), None);
        let result = executor.execute_step(&step, context).await.unwrap();

        assert!(!result.success);
        assert!(result.error.is_some());
        assert!(result.error.unwrap().contains("timed out"));
    }
}
