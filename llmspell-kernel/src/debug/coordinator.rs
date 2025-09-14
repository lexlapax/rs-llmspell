//! Debug Coordinator for managing debug sessions and operations
//!
//! This module provides the central coordination point for all debug operations,
//! managing breakpoints, execution control, and variable inspection across
//! different script engines and debug adapters.
//!
//! Migrated from Phase-9 branch (originally 878 lines)

use super::execution_bridge::{ExecutionManager, StepMode, Variable, VariableScope};
use anyhow::Result;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, instrument, trace, warn};

/// Debug event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DebugEvent {
    /// Execution paused at breakpoint
    Paused {
        /// Reason for pause
        reason: String,
        /// Source file
        source: String,
        /// Line number
        line: u32,
    },
    /// Execution resumed
    Resumed,
    /// Breakpoint hit
    BreakpointHit {
        /// Breakpoint ID
        id: String,
        /// Source file
        source: String,
        /// Line number
        line: u32,
    },
    /// Variable updated
    VariableUpdated {
        /// Variable name
        name: String,
        /// Variable value
        value: String,
    },
    /// Output from script
    Output {
        /// Output category
        category: String,
        /// Output text
        output: String,
    },
    /// Debug session terminated
    Terminated {
        /// Termination reason
        reason: Option<String>,
    },
}

/// Debug response types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DebugResponse {
    /// Success response
    Success,
    /// Error response
    Error(String),
    /// Variables response
    Variables(Vec<Variable>),
    /// Stack frames response
    StackFrames(Vec<super::execution_bridge::StackFrame>),
    /// Breakpoints response
    Breakpoints(Vec<super::execution_bridge::Breakpoint>),
}

/// Debug coordinator for managing debug operations
pub struct DebugCoordinator {
    /// Execution manager
    execution_manager: Arc<ExecutionManager>,
    /// Event sender
    event_tx: mpsc::Sender<DebugEvent>,
    /// Event receiver
    event_rx: Arc<RwLock<Option<mpsc::Receiver<DebugEvent>>>>,
    /// Active debug sessions
    sessions: Arc<RwLock<HashMap<String, DebugSessionInfo>>>,
    /// Session ID
    _session_id: String,
    /// Debug enabled flag
    debug_enabled: Arc<RwLock<bool>>,
}

/// Debug session information
#[derive(Debug, Clone)]
struct DebugSessionInfo {
    /// Session ID
    id: String,
    /// Script path
    _script_path: String,
    /// Active status
    active: bool,
    /// Start time
    _started_at: std::time::Instant,
}

impl DebugCoordinator {
    /// Create a new debug coordinator
    pub fn new(session_id: String) -> Self {
        let (event_tx, event_rx) = mpsc::channel(100);
        let execution_manager = Arc::new(ExecutionManager::new(session_id.clone()));

        Self {
            execution_manager,
            event_tx,
            event_rx: Arc::new(RwLock::new(Some(event_rx))),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            _session_id: session_id,
            debug_enabled: Arc::new(RwLock::new(true)),
        }
    }

    /// Enable or disable debugging
    pub fn set_debug_enabled(&self, enabled: bool) {
        *self.debug_enabled.write() = enabled;
        info!(
            "Debug mode {}",
            if enabled { "enabled" } else { "disabled" }
        );
    }

    /// Check if debugging is enabled
    pub fn is_debug_enabled(&self) -> bool {
        *self.debug_enabled.read()
    }

    /// Start a debug session
    ///
    /// # Errors
    /// Returns an error if session creation fails
    #[instrument(level = "info", skip(self))]
    pub fn start_session(&self, script_path: &str) -> Result<String> {
        let session_info = DebugSessionInfo {
            id: uuid::Uuid::new_v4().to_string(),
            _script_path: script_path.to_string(),
            active: true,
            _started_at: std::time::Instant::now(),
        };

        let session_id = session_info.id.clone();
        self.sessions
            .write()
            .insert(session_id.clone(), session_info);

        info!("Started debug session {} for {}", session_id, script_path);
        Ok(session_id)
    }

    /// End a debug session
    ///
    /// # Errors
    /// Returns an error if session termination fails
    pub fn end_session(&self, session_id: &str) -> Result<()> {
        if let Some(session) = self.sessions.write().get_mut(session_id) {
            session.active = false;
            debug!("Ended debug session {}", session_id);

            // Send termination event
            let _ = self.event_tx.try_send(DebugEvent::Terminated {
                reason: Some("Session ended".to_string()),
            });
        }
        Ok(())
    }

    /// Set a breakpoint
    ///
    /// # Errors
    /// Returns an error if breakpoint cannot be set
    pub fn set_breakpoint(&self, source: String, line: u32) -> Result<DebugResponse> {
        match self.execution_manager.set_breakpoint(source, line) {
            Ok(bp) => {
                debug!("Breakpoint set: {}:{}", bp.source, bp.line);
                Ok(DebugResponse::Breakpoints(vec![bp]))
            }
            Err(e) => {
                error!("Failed to set breakpoint: {}", e);
                Ok(DebugResponse::Error(e.to_string()))
            }
        }
    }

    /// Remove a breakpoint
    ///
    /// # Errors
    /// Returns an error if breakpoint cannot be removed
    pub fn remove_breakpoint(&self, id: &str) -> Result<DebugResponse> {
        match self.execution_manager.remove_breakpoint(id) {
            Ok(()) => {
                debug!("Breakpoint removed: {}", id);
                Ok(DebugResponse::Success)
            }
            Err(e) => {
                error!("Failed to remove breakpoint: {}", e);
                Ok(DebugResponse::Error(e.to_string()))
            }
        }
    }

    /// Continue execution
    ///
    /// # Errors
    /// Returns an error if execution cannot continue
    pub async fn continue_execution(&self) -> Result<DebugResponse> {
        self.execution_manager.resume(StepMode::Continue);

        let _ = self.event_tx.send(DebugEvent::Resumed).await;
        trace!("Continuing execution");

        Ok(DebugResponse::Success)
    }

    /// Step into
    ///
    /// # Errors
    /// Returns an error if step operation fails
    pub async fn step_into(&self) -> Result<DebugResponse> {
        self.execution_manager.resume(StepMode::StepIn);

        let _ = self.event_tx.send(DebugEvent::Resumed).await;
        trace!("Stepping into");

        Ok(DebugResponse::Success)
    }

    /// Step over
    ///
    /// # Errors
    /// Returns an error if step operation fails
    pub async fn step_over(&self) -> Result<DebugResponse> {
        self.execution_manager.resume(StepMode::StepOver);

        let _ = self.event_tx.send(DebugEvent::Resumed).await;
        trace!("Stepping over");

        Ok(DebugResponse::Success)
    }

    /// Step out
    ///
    /// # Errors
    /// Returns an error if step operation fails
    pub async fn step_out(&self) -> Result<DebugResponse> {
        self.execution_manager.resume(StepMode::StepOut);

        let _ = self.event_tx.send(DebugEvent::Resumed).await;
        trace!("Stepping out");

        Ok(DebugResponse::Success)
    }

    /// Pause execution
    ///
    /// # Errors
    /// Returns an error if pause operation fails
    pub async fn pause(&self) -> Result<DebugResponse> {
        self.execution_manager.pause();

        let _ = self
            .event_tx
            .send(DebugEvent::Paused {
                reason: "Manual pause".to_string(),
                source: "unknown".to_string(),
                line: 0,
            })
            .await;

        trace!("Paused execution");
        Ok(DebugResponse::Success)
    }

    /// Get variables
    ///
    /// # Errors
    /// Returns an error if variables cannot be retrieved
    pub fn get_variables(
        &self,
        scope: &VariableScope,
        frame_id: Option<&str>,
    ) -> Result<DebugResponse> {
        let variables = self.execution_manager.get_variables(scope, frame_id);
        Ok(DebugResponse::Variables(variables))
    }

    /// Get stack frames
    ///
    /// # Errors
    /// Returns an error if stack frames cannot be retrieved
    pub fn get_stack_frames(&self) -> Result<DebugResponse> {
        let frames = self.execution_manager.get_stack_frames();
        Ok(DebugResponse::StackFrames(frames))
    }

    /// Handle breakpoint hit
    pub async fn on_breakpoint_hit(&self, source: String, line: u32) {
        self.execution_manager.pause();

        let source_clone = source.clone();
        let _ = self
            .event_tx
            .send(DebugEvent::Paused {
                reason: "Breakpoint hit".to_string(),
                source,
                line,
            })
            .await;

        debug!("Breakpoint hit at {}:{}", source_clone, line);
    }

    /// Get execution manager
    pub fn execution_manager(&self) -> Arc<ExecutionManager> {
        self.execution_manager.clone()
    }

    /// Take event receiver
    pub fn take_event_receiver(&self) -> Option<mpsc::Receiver<DebugEvent>> {
        self.event_rx.write().take()
    }
}

/// Memory-aware debug coordinator for Phase 10 preparation
pub struct MemoryAwareDebugCoordinator {
    /// Base coordinator
    coordinator: DebugCoordinator,
    /// Memory bridge (prepared for Phase 10)
    memory_bridge: Option<Arc<dyn MemoryBridge>>,
}

/// Memory bridge trait for Phase 10 integration
pub trait MemoryBridge: Send + Sync {
    /// Track memory allocation
    fn track_allocation(&self, size: usize, location: &str);
    /// Track memory deallocation
    fn track_deallocation(&self, size: usize, location: &str);
    /// Get memory statistics
    fn get_stats(&self) -> MemoryStats;
}

/// Memory statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    /// Total allocated bytes
    allocated: usize,
    /// Total deallocated bytes
    deallocated: usize,
    /// Current usage
    current: usize,
    /// Peak usage
    peak: usize,
}

impl MemoryAwareDebugCoordinator {
    /// Create a new memory-aware debug coordinator
    pub fn new(session_id: String) -> Self {
        Self {
            coordinator: DebugCoordinator::new(session_id),
            memory_bridge: None,
        }
    }

    /// Set memory bridge (for Phase 10)
    pub fn set_memory_bridge(&mut self, bridge: Arc<dyn MemoryBridge>) {
        self.memory_bridge = Some(bridge);
        info!("Memory bridge connected to debug coordinator");
    }

    /// Get base coordinator
    pub fn coordinator(&self) -> &DebugCoordinator {
        &self.coordinator
    }

    /// Track debug operation with memory awareness
    pub fn track_operation(&self, operation: &str, size_hint: Option<usize>) {
        if let Some(ref bridge) = self.memory_bridge {
            if let Some(size) = size_hint {
                bridge.track_allocation(size, operation);
            }
        }
        trace!("Tracked operation: {} (size: {:?})", operation, size_hint);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_debug_coordinator() {
        let coordinator = DebugCoordinator::new("test-session".to_string());

        // Start session
        let session_id = coordinator.start_session("test.lua").unwrap();
        assert!(!session_id.is_empty());

        // Set breakpoint
        let response = coordinator
            .set_breakpoint("test.lua".to_string(), 10)
            .unwrap();
        match response {
            DebugResponse::Breakpoints(bps) => {
                assert_eq!(bps.len(), 1);
                assert_eq!(bps[0].line, 10);
            }
            _ => panic!("Expected breakpoints response"),
        }

        // Continue execution
        let response = coordinator.continue_execution().await.unwrap();
        assert!(matches!(response, DebugResponse::Success));

        // End session
        coordinator.end_session(&session_id).unwrap();
    }

    #[tokio::test]
    async fn test_memory_aware_coordinator() {
        let coordinator = MemoryAwareDebugCoordinator::new("test-session".to_string());

        // Track operation without memory bridge
        coordinator.track_operation("test_op", Some(1024));

        // Should work without memory bridge
        assert!(coordinator.memory_bridge.is_none());
    }
}
