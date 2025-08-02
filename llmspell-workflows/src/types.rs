//! ABOUTME: Workflow types for input/output and state management
//! ABOUTME: Provides types for memory-based workflow execution

use crate::shared_state::WorkflowStateAccessor;
use llmspell_core::ComponentId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Workflow input containing initial data and configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowInput {
    /// Initial input data for the workflow
    pub input: serde_json::Value,
    /// Optional context variables
    pub context: HashMap<String, serde_json::Value>,
    /// Execution timeout for the entire workflow
    pub timeout: Option<Duration>,
}

impl WorkflowInput {
    pub fn new(input: serde_json::Value) -> Self {
        Self {
            input,
            context: HashMap::new(),
            timeout: None,
        }
    }

    pub fn with_context(mut self, key: String, value: serde_json::Value) -> Self {
        self.context.insert(key, value);
        self
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }
}

/// Workflow output containing results and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowOutput {
    /// Final output data from the workflow
    pub output: serde_json::Value,
    /// Success status
    pub success: bool,
    /// Total execution duration
    pub duration: Duration,
    /// Number of steps executed
    pub steps_executed: usize,
    /// Number of failed steps
    pub steps_failed: usize,
    /// Final context state
    pub final_context: HashMap<String, serde_json::Value>,
    /// Error message if workflow failed
    pub error: Option<String>,
}

impl WorkflowOutput {
    pub fn success(
        output: serde_json::Value,
        duration: Duration,
        steps_executed: usize,
        final_context: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            output,
            success: true,
            duration,
            steps_executed,
            steps_failed: 0,
            final_context,
            error: None,
        }
    }

    pub fn failure(
        error: String,
        duration: Duration,
        steps_executed: usize,
        steps_failed: usize,
        final_context: HashMap<String, serde_json::Value>,
    ) -> Self {
        Self {
            output: serde_json::Value::Null,
            success: false,
            duration,
            steps_executed,
            steps_failed,
            final_context,
            error: Some(error),
        }
    }
}

/// Memory-based workflow state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowState {
    /// Workflow execution ID
    pub execution_id: ComponentId,
    /// Current step index
    pub current_step: usize,
    /// Shared data between steps
    pub shared_data: HashMap<String, serde_json::Value>,
    /// Step outputs for reference
    pub step_outputs: HashMap<ComponentId, serde_json::Value>,
    /// Execution start time (not serializable)
    #[serde(skip, default)]
    pub start_time: Option<Instant>,
    /// Last update time (not serializable)
    #[serde(skip, default = "Instant::now")]
    pub last_update: Instant,
}

impl WorkflowState {
    pub fn new() -> Self {
        Self {
            execution_id: ComponentId::new(),
            current_step: 0,
            shared_data: HashMap::new(),
            step_outputs: HashMap::new(),
            start_time: None,
            last_update: Instant::now(),
        }
    }

    pub fn start_execution(&mut self) {
        self.start_time = Some(Instant::now());
        self.last_update = Instant::now();
    }

    pub fn advance_step(&mut self) {
        self.current_step += 1;
        self.last_update = Instant::now();
    }

    pub fn set_step_output(&mut self, step_id: ComponentId, output: serde_json::Value) {
        self.step_outputs.insert(step_id, output);
        self.last_update = Instant::now();
    }

    pub fn get_step_output(&self, step_id: ComponentId) -> Option<&serde_json::Value> {
        self.step_outputs.get(&step_id)
    }

    pub fn set_shared_data(&mut self, key: String, value: serde_json::Value) {
        self.shared_data.insert(key, value);
        self.last_update = Instant::now();
    }

    pub fn get_shared_data(&self, key: &str) -> Option<&serde_json::Value> {
        self.shared_data.get(key)
    }

    pub fn execution_duration(&self) -> Option<Duration> {
        self.start_time.map(|start| start.elapsed())
    }

    pub fn reset(&mut self) {
        self.execution_id = ComponentId::new();
        self.current_step = 0;
        self.shared_data.clear();
        self.step_outputs.clear();
        self.start_time = None;
        self.last_update = Instant::now();
    }
}

impl Default for WorkflowState {
    fn default() -> Self {
        Self {
            execution_id: ComponentId::new(),
            current_step: 0,
            shared_data: HashMap::new(),
            step_outputs: HashMap::new(),
            start_time: None,
            last_update: Instant::now(),
        }
    }
}

/// Workflow configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowConfig {
    /// Maximum execution time for the entire workflow
    pub max_execution_time: Option<Duration>,
    /// Default timeout for individual steps
    pub default_step_timeout: Duration,
    /// Maximum retry attempts for failed steps
    pub max_retry_attempts: u32,
    /// Delay between retry attempts (base delay for exponential backoff)
    pub retry_delay_ms: u64,
    /// Whether to use exponential backoff for retries
    pub exponential_backoff: bool,
    /// Whether to continue execution after step failures
    pub continue_on_error: bool,
    /// Default error handling strategy
    pub default_error_strategy: crate::traits::ErrorStrategy,
}

impl WorkflowConfig {
    /// Create a new builder for WorkflowConfig
    pub fn builder() -> WorkflowConfigBuilder {
        WorkflowConfigBuilder::new()
    }

    /// Create a fast configuration preset (minimal retries, short timeouts)
    pub fn fast() -> Self {
        Self {
            max_execution_time: Some(Duration::from_secs(60)), // 1 minute
            default_step_timeout: Duration::from_secs(10),
            max_retry_attempts: 1,
            retry_delay_ms: 500,
            exponential_backoff: false,
            continue_on_error: false,
            default_error_strategy: crate::traits::ErrorStrategy::FailFast,
        }
    }

    /// Create a robust configuration preset (more retries, longer timeouts, continue on error)
    pub fn robust() -> Self {
        Self {
            max_execution_time: Some(Duration::from_secs(1800)), // 30 minutes
            default_step_timeout: Duration::from_secs(120),
            max_retry_attempts: 5,
            retry_delay_ms: 2000,
            exponential_backoff: true,
            continue_on_error: true,
            default_error_strategy: crate::traits::ErrorStrategy::Retry {
                max_attempts: 5,
                backoff_ms: 2000,
            },
        }
    }
}

impl Default for WorkflowConfig {
    fn default() -> Self {
        Self {
            max_execution_time: Some(Duration::from_secs(300)), // 5 minutes
            default_step_timeout: Duration::from_secs(30),
            max_retry_attempts: 3,
            retry_delay_ms: 1000, // 1 second base delay
            exponential_backoff: true,
            continue_on_error: false,
            default_error_strategy: crate::traits::ErrorStrategy::FailFast,
        }
    }
}

/// Builder for WorkflowConfig
#[derive(Debug, Clone)]
pub struct WorkflowConfigBuilder {
    config: WorkflowConfig,
}

impl WorkflowConfigBuilder {
    /// Create a new builder with default configuration
    pub fn new() -> Self {
        Self {
            config: WorkflowConfig::default(),
        }
    }

    /// Set maximum execution time for the entire workflow
    pub fn max_execution_time(mut self, duration: Option<Duration>) -> Self {
        self.config.max_execution_time = duration;
        self
    }

    /// Set default timeout for individual steps
    pub fn default_step_timeout(mut self, duration: Duration) -> Self {
        self.config.default_step_timeout = duration;
        self
    }

    /// Set maximum retry attempts for failed steps
    pub fn max_retry_attempts(mut self, attempts: u32) -> Self {
        self.config.max_retry_attempts = attempts;
        self
    }

    /// Set delay between retry attempts (base delay for exponential backoff)
    pub fn retry_delay_ms(mut self, delay_ms: u64) -> Self {
        self.config.retry_delay_ms = delay_ms;
        self
    }

    /// Set whether to use exponential backoff for retries
    pub fn exponential_backoff(mut self, enabled: bool) -> Self {
        self.config.exponential_backoff = enabled;
        self
    }

    /// Set whether to continue execution after step failures
    pub fn continue_on_error(mut self, enabled: bool) -> Self {
        self.config.continue_on_error = enabled;
        self
    }

    /// Set default error handling strategy
    pub fn default_error_strategy(mut self, strategy: crate::traits::ErrorStrategy) -> Self {
        self.config.default_error_strategy = strategy;
        self
    }

    /// Convenience method for setting retry strategy with common patterns
    pub fn retry_strategy(mut self, max_attempts: u32, delay_ms: u64, exponential: bool) -> Self {
        self.config.max_retry_attempts = max_attempts;
        self.config.retry_delay_ms = delay_ms;
        self.config.exponential_backoff = exponential;
        self
    }

    /// Convenience method alias for default_step_timeout
    pub fn default_timeout(self, duration: Duration) -> Self {
        self.default_step_timeout(duration)
    }

    /// Build the final WorkflowConfig
    pub fn build(self) -> WorkflowConfig {
        self.config
    }
}

impl Default for WorkflowConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Context for step execution within a workflow
#[derive(Debug, Clone)]
pub struct StepExecutionContext {
    /// Reference to workflow state
    pub workflow_state: WorkflowState,
    /// Shared state accessor for this workflow
    pub state_accessor: Option<WorkflowStateAccessor>,
    /// Step-specific timeout
    pub timeout: Option<Duration>,
    /// Current retry attempt (0 for first attempt)
    pub retry_attempt: u32,
    /// Whether this is the final retry attempt
    pub is_final_retry: bool,
}

impl StepExecutionContext {
    pub fn new(workflow_state: WorkflowState, timeout: Option<Duration>) -> Self {
        Self {
            workflow_state,
            state_accessor: None,
            timeout,
            retry_attempt: 0,
            is_final_retry: false,
        }
    }

    /// Add state accessor to the context
    pub fn with_state_accessor(mut self, accessor: WorkflowStateAccessor) -> Self {
        self.state_accessor = Some(accessor);
        self
    }

    pub fn with_retry(mut self, attempt: u32, max_attempts: u32) -> Self {
        self.retry_attempt = attempt;
        self.is_final_retry = attempt >= max_attempts;
        self
    }
}
