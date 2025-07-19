//! ABOUTME: Basic workflow types for input/output and state management
//! ABOUTME: Provides simplified types for memory-based workflow execution

use llmspell_core::ComponentId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Basic workflow input containing initial data and configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicWorkflowInput {
    /// Initial input data for the workflow
    pub input: serde_json::Value,
    /// Optional context variables
    pub context: HashMap<String, serde_json::Value>,
    /// Execution timeout for the entire workflow
    pub timeout: Option<Duration>,
}

impl BasicWorkflowInput {
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

/// Basic workflow output containing results and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicWorkflowOutput {
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

impl BasicWorkflowOutput {
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

/// Memory-based workflow state for basic workflows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicWorkflowState {
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

impl BasicWorkflowState {
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

impl Default for BasicWorkflowState {
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

/// Basic workflow configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicWorkflowConfig {
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
}

impl Default for BasicWorkflowConfig {
    fn default() -> Self {
        Self {
            max_execution_time: Some(Duration::from_secs(300)), // 5 minutes
            default_step_timeout: Duration::from_secs(30),
            max_retry_attempts: 3,
            retry_delay_ms: 1000, // 1 second base delay
            exponential_backoff: true,
            continue_on_error: false,
        }
    }
}

/// Context for step execution within a workflow
#[derive(Debug, Clone)]
pub struct StepExecutionContext {
    /// Reference to workflow state
    pub workflow_state: BasicWorkflowState,
    /// Step-specific timeout
    pub timeout: Option<Duration>,
    /// Current retry attempt (0 for first attempt)
    pub retry_attempt: u32,
    /// Whether this is the final retry attempt
    pub is_final_retry: bool,
}

impl StepExecutionContext {
    pub fn new(workflow_state: BasicWorkflowState, timeout: Option<Duration>) -> Self {
        Self {
            workflow_state,
            timeout,
            retry_attempt: 0,
            is_final_retry: false,
        }
    }

    pub fn with_retry(mut self, attempt: u32, max_attempts: u32) -> Self {
        self.retry_attempt = attempt;
        self.is_final_retry = attempt >= max_attempts;
        self
    }
}
