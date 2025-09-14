//! Debug Session Management
//!
//! This module provides session management for debug operations,
//! including interactive debugging, condition evaluation, and session persistence.
//!
//! Migrated from Phase-9 branch llmspell-debug crate (531 lines)

use super::coordinator::{DebugCoordinator, DebugEvent, DebugResponse};
use super::execution_bridge::{Variable, VariableScope};
use anyhow::Result;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::mpsc;
use tracing::{debug, info, instrument};

/// Debug session state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SessionState {
    /// Session initialized
    Initialized,
    /// Session running
    Running,
    /// Session paused
    Paused,
    /// Session terminated
    Terminated,
}

/// Debug session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugSessionConfig {
    /// Stop on entry
    pub stop_on_entry: bool,
    /// Stop on exceptions
    pub stop_on_exception: bool,
    /// Enable conditional breakpoints
    pub enable_conditions: bool,
    /// Enable watch expressions
    pub enable_watch: bool,
    /// Maximum stack depth
    pub max_stack_depth: usize,
    /// Timeout for operations (ms)
    pub operation_timeout_ms: u64,
}

impl Default for DebugSessionConfig {
    fn default() -> Self {
        Self {
            stop_on_entry: false,
            stop_on_exception: true,
            enable_conditions: true,
            enable_watch: true,
            max_stack_depth: 100,
            operation_timeout_ms: 5000,
        }
    }
}

/// Interactive debug session
pub struct DebugSession {
    /// Session ID
    id: String,
    /// Debug coordinator
    coordinator: Arc<DebugCoordinator>,
    /// Session state
    state: Arc<RwLock<SessionState>>,
    /// Configuration
    config: DebugSessionConfig,
    /// Watch expressions
    watch_expressions: Arc<RwLock<Vec<String>>>,
    /// Session metadata
    metadata: Arc<RwLock<SessionMetadata>>,
    /// Event receiver
    event_rx: Option<mpsc::Receiver<DebugEvent>>,
}

/// Session metadata
#[derive(Debug, Clone)]
struct SessionMetadata {
    /// Script path
    script_path: Option<String>,
    /// Start time
    started_at: Instant,
    /// Last activity time
    last_activity: Instant,
    /// Total breakpoints hit
    breakpoints_hit: usize,
    /// Total steps executed
    steps_executed: usize,
}

impl DebugSession {
    /// Create a new debug session
    pub fn new(id: String, config: DebugSessionConfig) -> Self {
        let coordinator = Arc::new(DebugCoordinator::new(id.clone()));
        let event_rx = coordinator.take_event_receiver();

        Self {
            id,
            coordinator,
            state: Arc::new(RwLock::new(SessionState::Initialized)),
            config,
            watch_expressions: Arc::new(RwLock::new(Vec::new())),
            metadata: Arc::new(RwLock::new(SessionMetadata {
                script_path: None,
                started_at: Instant::now(),
                last_activity: Instant::now(),
                breakpoints_hit: 0,
                steps_executed: 0,
            })),
            event_rx,
        }
    }

    /// Initialize session with script
    #[instrument(level = "info", skip(self))]
    pub async fn initialize(&mut self, script_path: String) -> Result<()> {
        info!("Initializing debug session for {}", script_path);
        
        self.metadata.write().script_path = Some(script_path.clone());
        self.coordinator.start_session(script_path)?;
        
        *self.state.write() = SessionState::Running;
        
        if self.config.stop_on_entry {
            self.pause().await?;
        }
        
        Ok(())
    }

    /// Run the session
    pub async fn run(&mut self) -> Result<()> {
        *self.state.write() = SessionState::Running;
        self.coordinator.continue_execution().await?;
        Ok(())
    }

    /// Pause the session
    pub async fn pause(&mut self) -> Result<()> {
        *self.state.write() = SessionState::Paused;
        self.coordinator.pause().await?;
        Ok(())
    }

    /// Step into
    pub async fn step_into(&mut self) -> Result<()> {
        self.metadata.write().steps_executed += 1;
        self.coordinator.step_into().await?;
        Ok(())
    }

    /// Step over
    pub async fn step_over(&mut self) -> Result<()> {
        self.metadata.write().steps_executed += 1;
        self.coordinator.step_over().await?;
        Ok(())
    }

    /// Step out
    pub async fn step_out(&mut self) -> Result<()> {
        self.metadata.write().steps_executed += 1;
        self.coordinator.step_out().await?;
        Ok(())
    }

    /// Continue execution
    pub async fn continue_execution(&mut self) -> Result<()> {
        *self.state.write() = SessionState::Running;
        self.coordinator.continue_execution().await?;
        Ok(())
    }

    /// Set a breakpoint
    pub async fn set_breakpoint(&self, source: String, line: u32) -> Result<DebugResponse> {
        self.coordinator.set_breakpoint(source, line).await
    }

    /// Remove a breakpoint
    pub async fn remove_breakpoint(&self, id: &str) -> Result<DebugResponse> {
        self.coordinator.remove_breakpoint(id).await
    }

    /// Add watch expression
    pub fn add_watch(&self, expression: String) {
        self.watch_expressions.write().push(expression);
    }

    /// Remove watch expression
    pub fn remove_watch(&self, index: usize) {
        let mut watches = self.watch_expressions.write();
        if index < watches.len() {
            watches.remove(index);
        }
    }

    /// Evaluate watch expressions
    pub async fn evaluate_watches(&self) -> Vec<Variable> {
        let watches = self.watch_expressions.read().clone();
        let mut results = Vec::new();
        
        for (i, expr) in watches.iter().enumerate() {
            results.push(Variable {
                name: format!("watch_{}", i),
                value: format!("<{}>", expr), // Simplified
                var_type: "watch".to_string(),
                has_children: false,
                reference: None,
            });
        }
        
        results
    }

    /// Get variables
    pub async fn get_variables(&self, scope: VariableScope) -> Result<DebugResponse> {
        self.coordinator.get_variables(scope, None).await
    }

    /// Get stack frames
    pub async fn get_stack_frames(&self) -> Result<DebugResponse> {
        self.coordinator.get_stack_frames().await
    }

    /// Handle debug event
    pub async fn handle_event(&mut self, event: DebugEvent) {
        match event {
            DebugEvent::Paused { reason, source, line } => {
                *self.state.write() = SessionState::Paused;
                info!("Session paused: {} at {}:{}", reason, source, line);
            }
            DebugEvent::Resumed => {
                *self.state.write() = SessionState::Running;
                debug!("Session resumed");
            }
            DebugEvent::BreakpointHit { .. } => {
                self.metadata.write().breakpoints_hit += 1;
            }
            DebugEvent::Terminated { reason } => {
                *self.state.write() = SessionState::Terminated;
                info!("Session terminated: {:?}", reason);
            }
            _ => {}
        }
        
        self.metadata.write().last_activity = Instant::now();
    }

    /// Process events
    pub async fn process_events(&mut self) -> Result<()> {
        // Collect events first to avoid borrow conflicts
        let mut events = Vec::new();
        if let Some(ref mut rx) = self.event_rx {
            while let Ok(event) = rx.try_recv() {
                events.push(event);
            }
        }

        // Process collected events
        for event in events {
            self.handle_event(event).await;
        }

        Ok(())
    }

    /// Terminate session
    pub async fn terminate(&mut self) -> Result<()> {
        *self.state.write() = SessionState::Terminated;
        self.coordinator.end_session(&self.id)?;
        Ok(())
    }

    /// Get session state
    pub fn state(&self) -> SessionState {
        *self.state.read()
    }

    /// Get session ID
    pub fn id(&self) -> &str {
        &self.id
    }
}

/// Debug session manager for managing multiple sessions
pub struct DebugSessionManager {
    /// Active sessions
    sessions: Arc<RwLock<HashMap<String, Arc<RwLock<DebugSession>>>>>,
    /// Default configuration
    default_config: DebugSessionConfig,
}

impl DebugSessionManager {
    /// Create a new session manager
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
            default_config: DebugSessionConfig::default(),
        }
    }

    /// Create a new session
    pub fn create_session(&self, config: Option<DebugSessionConfig>) -> String {
        let session_id = uuid::Uuid::new_v4().to_string();
        let config = config.unwrap_or_else(|| self.default_config.clone());
        let session = Arc::new(RwLock::new(DebugSession::new(session_id.clone(), config)));
        
        self.sessions.write().insert(session_id.clone(), session);
        
        info!("Created debug session: {}", session_id);
        session_id
    }

    /// Get a session
    pub fn get_session(&self, id: &str) -> Option<Arc<RwLock<DebugSession>>> {
        self.sessions.read().get(id).cloned()
    }

    /// Remove a session
    pub async fn remove_session(&self, id: &str) -> Result<()> {
        if let Some(session) = self.sessions.write().remove(id) {
            session.write().terminate().await?;
            info!("Removed debug session: {}", id);
        }
        Ok(())
    }

    /// List active sessions
    pub fn list_sessions(&self) -> Vec<String> {
        self.sessions.read().keys().cloned().collect()
    }

    /// Clean up terminated sessions
    pub async fn cleanup_terminated(&self) -> Result<()> {
        let sessions = self.sessions.read();
        let terminated: Vec<String> = sessions
            .iter()
            .filter(|(_, s)| s.read().state() == SessionState::Terminated)
            .map(|(id, _)| id.clone())
            .collect();
        drop(sessions);
        
        for id in terminated {
            self.remove_session(&id).await?;
        }
        
        Ok(())
    }
}

impl Default for DebugSessionManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_debug_session() {
        let config = DebugSessionConfig::default();
        let mut session = DebugSession::new("test-session".to_string(), config);
        
        assert_eq!(session.state(), SessionState::Initialized);
        
        session.initialize("test.lua".to_string()).await.unwrap();
        assert_eq!(session.state(), SessionState::Running);
        
        session.pause().await.unwrap();
        assert_eq!(session.state(), SessionState::Paused);
        
        session.terminate().await.unwrap();
        assert_eq!(session.state(), SessionState::Terminated);
    }

    #[tokio::test]
    async fn test_session_manager() {
        let manager = DebugSessionManager::new();
        
        let session_id = manager.create_session(None);
        assert!(!session_id.is_empty());
        
        assert!(manager.get_session(&session_id).is_some());
        
        let sessions = manager.list_sessions();
        assert_eq!(sessions.len(), 1);
        
        manager.remove_session(&session_id).await.unwrap();
        assert!(manager.get_session(&session_id).is_none());
    }
}
