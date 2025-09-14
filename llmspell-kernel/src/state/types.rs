//! State type definitions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant, SystemTime};

/// Execution state tracking
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExecutionState {
    /// Current execution counter
    pub execution_count: i32,
    /// Active execution ID
    pub current_execution_id: Option<String>,
    /// Execution status
    pub status: ExecutionStatus,
    /// Code being executed
    pub current_code: Option<String>,
    /// Execution start time
    #[serde(skip)]
    pub start_time: Option<Instant>,
    /// Total execution time
    pub total_execution_time: Duration,
    /// Variables in current execution context
    pub variables: HashMap<String, String>,
    /// Execution history (limited to last 100)
    pub history: Vec<ExecutionRecord>,
}

/// Execution status
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub enum ExecutionStatus {
    /// No active execution
    #[default]
    Idle,
    /// Currently executing code
    Running,
    /// Execution is paused
    Paused,
    /// Execution failed with error
    Error(String),
    /// Execution completed successfully
    Completed,
}

/// Record of a single execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    /// Unique execution ID
    pub id: String,
    /// Code that was executed
    pub code: String,
    /// Execution result if successful
    pub result: Option<String>,
    /// Error message if failed
    pub error: Option<String>,
    /// Time taken to execute
    pub duration: Duration,
    /// When the execution occurred
    pub timestamp: SystemTime,
}

impl ExecutionState {
    /// Increment execution counter
    pub fn increment_counter(&mut self) -> i32 {
        self.execution_count += 1;
        self.execution_count
    }

    /// Start a new execution
    pub fn start_execution(&mut self, id: String, code: String) {
        self.current_execution_id = Some(id);
        self.current_code = Some(code);
        self.status = ExecutionStatus::Running;
        self.start_time = Some(Instant::now());
    }

    /// Complete current execution
    ///
    /// # Panics
    ///
    /// This function will never panic as `unwrap` is only called after checking `is_some()`
    pub fn complete_execution(&mut self, result: Option<String>, error: Option<String>) {
        if let (Some(id), Some(code), Some(start)) = (
            self.current_execution_id.take(),
            self.current_code.take(),
            self.start_time.take(),
        ) {
            let duration = start.elapsed();
            self.total_execution_time += duration;

            let record = ExecutionRecord {
                id,
                code,
                result,
                error: error.clone(),
                duration,
                timestamp: SystemTime::now(),
            };

            self.history.push(record);

            // Keep only last 100 executions
            if self.history.len() > 100 {
                self.history.remove(0);
            }

            self.status = if let Some(err) = error {
                ExecutionStatus::Error(err)
            } else {
                ExecutionStatus::Completed
            };
        }
    }

    /// Pause execution
    pub fn pause(&mut self) {
        if self.status == ExecutionStatus::Running {
            self.status = ExecutionStatus::Paused;
        }
    }

    /// Resume execution
    pub fn resume(&mut self) {
        if self.status == ExecutionStatus::Paused {
            self.status = ExecutionStatus::Running;
        }
    }
}

/// Session state management
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SessionState {
    /// Session ID
    pub session_id: Option<String>,
    /// Session creation time
    pub created_at: Option<SystemTime>,
    /// Last activity time
    pub last_activity: Option<SystemTime>,
    /// Session metadata
    pub metadata: HashMap<String, String>,
    /// Active breakpoints
    pub breakpoints: Vec<BreakpointInfo>,
    /// Session artifacts (files, outputs, etc.)
    pub artifacts: Vec<SessionArtifact>,
    /// Session status
    pub status: SessionStatus,
    /// Resource usage
    pub resources: ResourceUsage,
}

/// Session status
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub enum SessionStatus {
    /// Session is active and ready
    #[default]
    Active,
    /// Session is temporarily paused
    Paused,
    /// Session has been terminated
    Terminated,
    /// Session has been archived
    Archived,
}

/// Breakpoint information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakpointInfo {
    /// Unique breakpoint ID
    pub id: String,
    /// Source file path
    pub source: String,
    /// Line number
    pub line: u32,
    /// Optional condition expression
    pub condition: Option<String>,
    /// Number of times hit
    pub hit_count: u32,
    /// Whether breakpoint is enabled
    pub enabled: bool,
}

/// Session artifact
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionArtifact {
    /// Unique artifact ID
    pub id: String,
    /// Artifact name
    pub name: String,
    /// Type of artifact
    pub artifact_type: String,
    /// Size in bytes
    pub size: usize,
    /// Creation timestamp
    pub created_at: SystemTime,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Resource usage tracking
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceUsage {
    /// Memory usage in bytes
    pub memory_bytes: u64,
    /// CPU time in milliseconds
    pub cpu_time_ms: u64,
    /// Number of API calls
    pub api_calls: u64,
    /// Number of tool invocations
    pub tool_calls: u64,
}

impl SessionState {
    /// Set session ID
    pub fn set_id(&mut self, id: &str) {
        self.session_id = Some(id.to_string());
        self.created_at = Some(SystemTime::now());
    }

    /// Update last activity
    pub fn touch(&mut self) {
        self.last_activity = Some(SystemTime::now());
    }

    /// Add breakpoint
    pub fn add_breakpoint(&mut self, bp: BreakpointInfo) {
        self.breakpoints.push(bp);
    }

    /// Remove breakpoint
    pub fn remove_breakpoint(&mut self, id: &str) {
        self.breakpoints.retain(|bp| bp.id != id);
    }

    /// Add artifact
    pub fn add_artifact(&mut self, artifact: SessionArtifact) {
        self.artifacts.push(artifact);
    }

    /// Pause session
    pub fn pause(&mut self) {
        self.status = SessionStatus::Paused;
    }

    /// Resume session
    pub fn resume(&mut self) {
        if self.status == SessionStatus::Paused {
            self.status = SessionStatus::Active;
        }
    }

    /// Terminate session
    pub fn terminate(&mut self) {
        self.status = SessionStatus::Terminated;
    }
}

/// Debug state coordination
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DebugState {
    /// Whether debugging is enabled
    pub enabled: bool,
    /// Current debug mode
    pub mode: DebugMode,
    /// Stack frames
    pub stack_frames: Vec<StackFrameInfo>,
    /// Variable scopes
    pub variables: HashMap<String, Vec<VariableInfo>>,
    /// Watch expressions
    pub watches: Vec<WatchExpression>,
    /// Debug session ID
    pub debug_session_id: Option<String>,
    /// Step counter
    pub step_count: u64,
}

/// Debug mode
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub enum DebugMode {
    /// No debugging active
    #[default]
    None,
    /// Step into function calls
    StepIn,
    /// Step over function calls
    StepOver,
    /// Step out of current function
    StepOut,
    /// Continue execution until next breakpoint
    Continue,
    /// Pause execution
    Pause,
}

/// Stack frame information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StackFrameInfo {
    /// Unique frame ID
    pub id: String,
    /// Function or method name
    pub name: String,
    /// Source file path
    pub source: String,
    /// Line number
    pub line: u32,
    /// Optional column number
    pub column: Option<u32>,
}

/// Variable information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableInfo {
    /// Variable name
    pub name: String,
    /// Variable value as string
    pub value: String,
    /// Variable type
    pub var_type: String,
    /// Variable scope
    pub scope: String,
}

/// Watch expression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchExpression {
    /// Unique watch ID
    pub id: String,
    /// Expression to evaluate
    pub expression: String,
    /// Evaluated value
    pub value: Option<String>,
    /// Evaluation error if any
    pub error: Option<String>,
}

impl DebugState {
    /// Enable debugging
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable debugging
    pub fn disable(&mut self) {
        self.enabled = false;
        self.mode = DebugMode::None;
    }

    /// Set debug mode
    pub fn set_mode(&mut self, mode: DebugMode) {
        self.mode = mode;
    }

    /// Add stack frame
    pub fn push_frame(&mut self, frame: StackFrameInfo) {
        self.stack_frames.push(frame);
    }

    /// Pop stack frame
    pub fn pop_frame(&mut self) -> Option<StackFrameInfo> {
        self.stack_frames.pop()
    }

    /// Clear all frames
    pub fn clear_frames(&mut self) {
        self.stack_frames.clear();
    }

    /// Add watch expression
    pub fn add_watch(&mut self, watch: WatchExpression) {
        self.watches.push(watch);
    }

    /// Update watch value
    pub fn update_watch(&mut self, id: &str, value: Option<String>, error: Option<String>) {
        if let Some(watch) = self.watches.iter_mut().find(|w| w.id == id) {
            watch.value = value;
            watch.error = error;
        }
    }

    /// Increment step counter
    pub fn step(&mut self) {
        self.step_count += 1;
    }
}
