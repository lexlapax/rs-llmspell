//! Hybrid Debug Runtime - Connects script execution to debug infrastructure
//!
//! This module provides the critical connection between `ScriptRuntime` (execution)
//! and debug infrastructure (control) that makes debugging actually functional.
//! It wraps `ScriptRuntime` with debug hooks that trigger `ExecutionManager` for
//! breakpoint/step control while sharing context with all debug components.

use crate::{ScriptOutput, ScriptRuntime};
use async_trait::async_trait;
use llmspell_config::LLMSpellConfig;
use llmspell_core::debug::{DebugCapability, DebugRequest, DebugResponse};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, trace};

/// Debug session information
#[derive(Debug, Clone)]
pub struct DebugSession {
    /// Unique session identifier
    pub session_id: String,
    /// Script being debugged
    pub script_content: String,
    /// Current debug state
    pub state: DebugSessionState,
    /// Session start time
    pub start_time: std::time::Instant,
}

/// Debug session state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugSessionState {
    /// Session initialized but not started
    Initialized,
    /// Debugging in progress
    Active,
    /// Paused at breakpoint or step
    Paused,
    /// Session completed
    Completed,
    /// Session failed with error
    Failed,
}

/// Processor error type
#[derive(Debug, thiserror::Error)]
pub enum ProcessorError {
    #[error("Processing failed: {0}")]
    ProcessingFailed(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Debug hook that gets called at key execution points
#[async_trait]
pub trait DebugHook: Send + Sync {
    /// Called before each line/statement execution
    async fn on_line(&self, line: u32, source: &str) -> DebugControl;

    /// Called when entering a function
    async fn on_function_enter(&self, name: &str, args: Vec<String>) -> DebugControl;

    /// Called when exiting a function
    async fn on_function_exit(&self, name: &str, result: Option<String>) -> DebugControl;

    /// Called when an exception occurs
    async fn on_exception(&self, error: &str, line: u32) -> DebugControl;

    /// Get as Any for downcasting
    fn as_any(&self) -> &dyn std::any::Any;
}

/// Control flow returned by debug hooks
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DebugControl {
    /// Continue normal execution
    Continue,
    /// Pause execution (hit breakpoint or step)
    Pause,
    /// Step to next line
    StepOver,
    /// Step into function call
    StepIn,
    /// Step out of current function
    StepOut,
    /// Terminate execution
    Terminate,
}

/// Hybrid runtime that combines `ScriptRuntime` with debug infrastructure
pub struct DebugRuntime {
    /// Underlying script runtime for execution (boxed to reduce future size)
    runtime: Box<ScriptRuntime>,

    /// Debug session information
    session: DebugSession,

    /// Registered debug capabilities (execution manager, variable inspector, etc.)
    capabilities: Arc<RwLock<HashMap<String, Arc<dyn DebugCapability>>>>,

    /// Debug hooks for intercepting execution
    _hooks: Arc<RwLock<Vec<Arc<dyn DebugHook>>>>,

    /// Current execution state
    state: Arc<RwLock<ExecutionState>>,
}

/// Execution state tracked by debug runtime
#[derive(Debug, Clone, Default)]
pub struct ExecutionState {
    /// Current line being executed
    current_line: u32,
    /// Current function (if any)
    current_function: Option<String>,
    /// Call stack depth
    _call_depth: usize,
    /// Whether we're currently stepping
    stepping: bool,
    /// Step mode (over/in/out)
    step_mode: Option<StepMode>,
}

#[derive(Debug, Clone, PartialEq)]
enum StepMode {
    Over,
    In,
    Out,
}

impl DebugRuntime {
    /// Create a new debug runtime with the given configuration
    ///
    /// # Errors
    ///
    /// Returns an error if the runtime creation fails
    pub fn new(
        config: LLMSpellConfig,
        session: DebugSession,
        capabilities: Arc<RwLock<HashMap<String, Arc<dyn DebugCapability>>>>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self, ProcessorError>> + Send>>
    {
        Box::pin(async move {
            // Create the underlying script runtime (boxed to reduce future size)
            let runtime = Box::new(
                ScriptRuntime::new_with_engine_name(&config.default_engine, config.clone())
                    .await
                    .map_err(|e| {
                        ProcessorError::Internal(format!("Failed to create runtime: {e}"))
                    })?,
            );

            Ok(Self {
                runtime,
                session,
                capabilities,
                _hooks: Arc::new(RwLock::new(Vec::new())),
                state: Arc::new(RwLock::new(ExecutionState {
                    current_line: 0,
                    current_function: None,
                    _call_depth: 0,
                    stepping: false,
                    step_mode: None,
                })),
            })
        })
    }

    /// Execute the script with debug hooks enabled
    ///
    /// # Errors
    ///
    /// Returns an error if script execution fails
    pub async fn execute(&mut self) -> Result<ScriptOutput, ProcessorError> {
        info!(
            "Starting debug execution for session: {}",
            self.session.session_id
        );

        // Update session state
        self.session.state = DebugSessionState::Active;

        // Note: Debug hooks are already installed in ScriptRuntime during creation
        // based on the debug configuration (interactive mode uses LuaDebugHookAdapter)

        // Execute the script with hooks active
        let result = self
            .runtime
            .execute_script(&self.session.script_content)
            .await
            .map_err(|e| {
                ProcessorError::ProcessingFailed(format!("Script execution failed: {e}"))
            })?;

        // Update session state
        self.session.state = DebugSessionState::Completed;

        Ok(result)
    }

    /// Process a debug command (set breakpoint, step, continue, etc.)
    ///
    /// # Errors
    ///
    /// Returns an error if the command processing fails or no capability is registered
    pub async fn process_debug_command(
        &mut self,
        command: DebugRequest,
    ) -> Result<DebugResponse, ProcessorError> {
        // Route to the appropriate capability
        let capabilities = self.capabilities.read().await;

        let capability_name = command.capability_name();

        if let Some(capability) = capabilities.get(&capability_name) {
            capability
                .process_debug_request(command)
                .await
                .map_err(|e| ProcessorError::ProcessingFailed(e.to_string()))
        } else {
            Err(ProcessorError::InvalidRequest(format!(
                "No capability registered for: {capability_name}"
            )))
        }
    }

    /// Get the current debug state
    pub async fn get_debug_state(&self) -> ExecutionState {
        self.state.read().await.clone()
    }

    /// Pause execution at current point
    pub async fn pause(&mut self) {
        let mut state = self.state.write().await;
        state.stepping = true;
        state.step_mode = Some(StepMode::Over);
        let line = state.current_line;
        drop(state);
        info!("Execution paused at line {}", line);
    }

    /// Resume execution
    pub async fn resume(&mut self) {
        let mut state = self.state.write().await;
        state.stepping = false;
        state.step_mode = None;
        let line = state.current_line;
        drop(state);
        info!("Execution resumed from line {}", line);
    }

    /// Step to next line
    pub async fn step_over(&mut self) {
        let mut state = self.state.write().await;
        state.stepping = true;
        state.step_mode = Some(StepMode::Over);
        let line = state.current_line;
        drop(state);
        trace!("Stepping over at line {}", line);
    }

    /// Step into function call
    pub async fn step_in(&mut self) {
        let mut state = self.state.write().await;
        state.stepping = true;
        state.step_mode = Some(StepMode::In);
        let line = state.current_line;
        drop(state);
        trace!("Stepping into at line {}", line);
    }

    /// Step out of current function
    pub async fn step_out(&mut self) {
        let mut state = self.state.write().await;
        state.stepping = true;
        state.step_mode = Some(StepMode::Out);
        let func = state.current_function.clone();
        drop(state);
        trace!("Stepping out from {:?}", func);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_debug_runtime_creation() {
        let config = LLMSpellConfig::default();
        let session = DebugSession {
            session_id: "test-session".to_string(),
            script_content: "print('test')".to_string(),
            state: DebugSessionState::Initialized,
            start_time: std::time::Instant::now(),
        };
        let capabilities = Arc::new(RwLock::new(HashMap::new()));

        let runtime = DebugRuntime::new(config, session, capabilities).await;
        assert!(runtime.is_ok());
    }
}
