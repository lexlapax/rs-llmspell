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
use llmspell_engine::{DebugSession, DebugSessionState, ProcessorError};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, trace};

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
    hooks: Arc<RwLock<Vec<Arc<dyn DebugHook>>>>,

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
    call_depth: usize,
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
                hooks: Arc::new(RwLock::new(Vec::new())),
                state: Arc::new(RwLock::new(ExecutionState {
                    current_line: 0,
                    current_function: None,
                    call_depth: 0,
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

        // Install debug hooks into the runtime
        self.install_debug_hooks().await?;

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

    /// Install debug hooks into the runtime
    async fn install_debug_hooks(&mut self) -> Result<(), ProcessorError> {
        // Create the main debug hook that routes to ExecutionManager
        let exec_hook = Arc::new(ExecutionManagerHook::new(
            self.capabilities.clone(),
            self.state.clone(),
        ));

        let mut hooks = self.hooks.write().await;
        hooks.push(exec_hook.clone());
        let hook_count = hooks.len();
        drop(hooks);

        // Install the hook into the script runtime's engine
        self.runtime
            .install_debug_hooks(exec_hook)
            .map_err(|e| ProcessorError::Internal(format!("Failed to install debug hooks: {e}")))?;

        debug!("Installed {} debug hooks", hook_count);
        Ok(())
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

/// Debug hook implementation that routes to `ExecutionManager`
pub struct ExecutionManagerHook {
    _capabilities: Arc<RwLock<HashMap<String, Arc<dyn DebugCapability>>>>,
    state: Arc<RwLock<ExecutionState>>,
}

impl ExecutionManagerHook {
    /// Create a new `ExecutionManagerHook`
    #[must_use]
    pub fn new(
        capabilities: Arc<RwLock<HashMap<String, Arc<dyn DebugCapability>>>>,
        state: Arc<RwLock<ExecutionState>>,
    ) -> Self {
        Self {
            _capabilities: capabilities,
            state,
        }
    }
}

#[async_trait]
impl DebugHook for ExecutionManagerHook {
    async fn on_line(&self, line: u32, _source: &str) -> DebugControl {
        // Update current line
        {
            let mut state = self.state.write().await;
            state.current_line = line;
        }

        // Check if we should pause (breakpoint or stepping)
        // TODO: For now, we'll need to track breakpoints in the ExecutionManager
        // and check them here. This requires enhancing the ExecutionManager
        // to expose a method for checking breakpoints at a given line.
        // For initial implementation, we'll just check stepping state.

        // Check if we're stepping
        let state = self.state.read().await;
        let stepping = state.stepping;
        let step_mode = state.step_mode.clone();
        drop(state);

        if stepping {
            match step_mode {
                Some(StepMode::Over) => {
                    trace!("Step over at line {}", line);
                    return DebugControl::Pause;
                }
                Some(StepMode::In) => {
                    trace!("Step in at line {}", line);
                    return DebugControl::StepIn;
                }
                _ => {}
            }
        }

        DebugControl::Continue
    }

    async fn on_function_enter(&self, name: &str, _args: Vec<String>) -> DebugControl {
        let mut state = self.state.write().await;
        state.current_function = Some(name.to_string());
        state.call_depth += 1;

        let depth = state.call_depth;
        let stepping = state.stepping;
        let step_mode = state.step_mode.clone();
        drop(state);

        trace!("Entering function: {} (depth: {})", name, depth);

        // Check if we're stepping in
        if stepping && step_mode == Some(StepMode::In) {
            return DebugControl::Pause;
        }

        DebugControl::Continue
    }

    async fn on_function_exit(&self, name: &str, _result: Option<String>) -> DebugControl {
        let mut state = self.state.write().await;
        state.call_depth = state.call_depth.saturating_sub(1);

        if state.call_depth == 0 {
            state.current_function = None;
        }

        let depth = state.call_depth;
        let stepping = state.stepping;
        let step_mode = state.step_mode.clone();

        // Check if we're stepping out
        if stepping && step_mode == Some(StepMode::Out) {
            state.stepping = false; // Stop stepping after exiting
            drop(state);
            trace!("Exiting function: {} (depth: {})", name, depth);
            return DebugControl::Pause;
        }
        drop(state);

        trace!("Exiting function: {} (depth: {})", name, depth);
        DebugControl::Continue
    }

    async fn on_exception(&self, error: &str, line: u32) -> DebugControl {
        info!("Exception at line {}: {}", line, error);

        // Always pause on exceptions for debugging
        DebugControl::Pause
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

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_debug_control_flow() {
        let hook = ExecutionManagerHook::new(
            Arc::new(RwLock::new(HashMap::new())),
            Arc::new(RwLock::new(ExecutionState {
                current_line: 0,
                current_function: None,
                call_depth: 0,
                stepping: false,
                step_mode: None,
            })),
        );

        // Test normal execution continues
        let control = hook.on_line(1, "print('test')").await;
        assert_eq!(control, DebugControl::Continue);

        // Test function entry/exit
        let control = hook.on_function_enter("test_func", vec![]).await;
        assert_eq!(control, DebugControl::Continue);

        let control = hook.on_function_exit("test_func", None).await;
        assert_eq!(control, DebugControl::Continue);
    }
}
