//! ABOUTME: Tool execution state machine for tracking tool lifecycle states
//! ABOUTME: Provides state transitions and validation for tool execution phases

#![allow(clippy::significant_drop_tightening)]

use tracing::instrument;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::debug;

/// Tool execution states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ToolExecutionState {
    /// Tool not yet initialized
    Uninitialized,
    /// Tool is being initialized
    Initializing,
    /// Tool ready for execution
    Ready,
    /// Tool currently executing
    Executing,
    /// Tool execution completed successfully
    Completed,
    /// Tool execution failed
    Failed,
    /// Tool is being cleaned up
    CleaningUp,
    /// Tool fully cleaned up
    Terminated,
}

impl ToolExecutionState {
    /// Check if tool can start execution
    #[must_use]
    pub const fn can_execute(&self) -> bool {
        matches!(self, Self::Ready)
    }

    /// Check if tool is in a final state
    #[must_use]
    pub const fn is_terminal(&self) -> bool {
        matches!(self, Self::Terminated)
    }

    /// Check if tool is healthy
    #[must_use]
    pub const fn is_healthy(&self) -> bool {
        matches!(self, Self::Ready | Self::Executing | Self::Completed)
    }
}

/// Tool state machine for managing execution lifecycle
pub struct ToolStateMachine {
    /// Current state
    state: Arc<RwLock<ToolExecutionState>>,
    /// Tool name for logging
    tool_name: String,
    /// State transition history
    transition_history: Arc<RwLock<Vec<StateTransition>>>,
    /// Creation time
    created_at: Instant,
}

/// State transition record
#[derive(Debug, Clone)]
pub struct StateTransition {
    pub from: ToolExecutionState,
    pub to: ToolExecutionState,
    pub timestamp: Instant,
    pub duration_since_last: Duration,
}

impl ToolStateMachine {
    /// Create a new tool state machine
    #[must_use]
    pub fn new(tool_name: String) -> Self {
        Self {
            state: Arc::new(RwLock::new(ToolExecutionState::Uninitialized)),
            tool_name,
            transition_history: Arc::new(RwLock::new(Vec::new())),
            created_at: Instant::now(),
        }
    }

    /// Get current state
    #[instrument(skip(self))]
    pub async fn current_state(&self) -> ToolExecutionState {
        *self.state.read().await
    }

    /// Check if tool is in a specific state
    #[instrument(skip(self))]
    pub async fn is_state(&self, expected_state: ToolExecutionState) -> bool {
        *self.state.read().await == expected_state
    }

    /// Transition to a new state
    ///
    /// # Errors
    ///
    /// Returns an error if the requested state transition is invalid
    /// according to the state machine rules
    #[instrument(skip(self))]
    pub async fn transition_to(&self, new_state: ToolExecutionState) -> Result<()> {
        let mut state_guard = self.state.write().await;
        let current_state = *state_guard;

        // Validate transition
        if !Self::is_valid_transition(current_state, new_state) {
            return Err(anyhow::anyhow!(
                "Invalid state transition for tool '{}': {:?} -> {:?}",
                self.tool_name,
                current_state,
                new_state
            ));
        }

        // Record transition
        let now = Instant::now();
        let transition = StateTransition {
            from: current_state,
            to: new_state,
            timestamp: now,
            duration_since_last: now.duration_since(self.created_at),
        };

        {
            let mut history = self.transition_history.write().await;
            history.push(transition);
        }

        // Update state
        *state_guard = new_state;

        debug!(
            "Tool '{}' transitioned from {:?} to {:?}",
            self.tool_name, current_state, new_state
        );

        Ok(())
    }

    /// Initialize the tool
    ///
    /// # Errors
    ///
    /// Returns an error if the state transition is invalid
    #[instrument(skip(self))]
    pub async fn initialize(&self) -> Result<()> {
        self.transition_to(ToolExecutionState::Initializing).await?;
        // Simulate initialization time
        tokio::time::sleep(Duration::from_millis(1)).await;
        self.transition_to(ToolExecutionState::Ready).await
    }

    /// Start execution
    ///
    /// # Errors
    ///
    /// Returns an error if the state transition is invalid
    #[instrument(skip(self))]
    pub async fn start_execution(&self) -> Result<()> {
        self.transition_to(ToolExecutionState::Executing).await
    }

    /// Complete execution successfully
    ///
    /// # Errors
    ///
    /// Returns an error if the state transition is invalid
    #[instrument(skip(self))]
    pub async fn complete_execution(&self) -> Result<()> {
        self.transition_to(ToolExecutionState::Completed).await
    }

    /// Fail execution
    ///
    /// # Errors
    ///
    /// Returns an error if the state transition is invalid
    #[instrument(skip(self))]
    pub async fn fail_execution(&self) -> Result<()> {
        self.transition_to(ToolExecutionState::Failed).await
    }

    /// Start cleanup
    ///
    /// # Errors
    ///
    /// Returns an error if the state transition is invalid
    #[instrument(skip(self))]
    pub async fn start_cleanup(&self) -> Result<()> {
        self.transition_to(ToolExecutionState::CleaningUp).await
    }

    /// Terminate the tool
    ///
    /// # Errors
    ///
    /// Returns an error if the state transition to Terminated is invalid
    /// from the current state
    #[instrument(skip(self))]
    pub async fn terminate(&self) -> Result<()> {
        self.transition_to(ToolExecutionState::Terminated).await
    }

    /// Get state transition history
    #[instrument(skip(self))]
    pub async fn get_transition_history(&self) -> Vec<StateTransition> {
        self.transition_history.read().await.clone()
    }

    /// Get execution statistics
    #[instrument(skip(self))]
    pub async fn get_execution_stats(&self) -> ExecutionStats {
        let history = self.transition_history.read().await;
        let current_state = *self.state.read().await;

        let total_transitions = history.len();
        let execution_time = history.last().map_or_else(
            || Instant::now().duration_since(self.created_at),
            |last_transition| last_transition.duration_since_last,
        );

        // Count state durations
        let mut state_durations = std::collections::HashMap::new();
        let mut last_time = self.created_at;

        for transition in history.iter() {
            let duration = transition.timestamp.duration_since(last_time);
            *state_durations
                .entry(transition.from)
                .or_insert(Duration::ZERO) += duration;
            last_time = transition.timestamp;
        }

        // Add current state duration
        let current_duration = Instant::now().duration_since(last_time);
        *state_durations
            .entry(current_state)
            .or_insert(Duration::ZERO) += current_duration;

        ExecutionStats {
            current_state,
            total_transitions,
            execution_time,
            state_durations,
            is_healthy: current_state.is_healthy(),
            is_terminal: current_state.is_terminal(),
        }
    }

    /// Validate if a state transition is allowed
    const fn is_valid_transition(from: ToolExecutionState, to: ToolExecutionState) -> bool {
        use ToolExecutionState::{
            CleaningUp, Completed, Executing, Failed, Initializing, Ready, Terminated,
            Uninitialized,
        };

        match (from, to) {
            // All valid transitions
            (Uninitialized, Initializing | Terminated)
            | (Initializing, Ready | Failed | Terminated)
            | (Ready, Executing | CleaningUp | Terminated)
            | (Executing, Completed | Failed)
            | (Completed | Failed, CleaningUp | Terminated)
            | (CleaningUp, Terminated) => true,

            // Terminal states cannot transition
            _ => false,
        }
    }
}

/// Tool execution statistics
#[derive(Debug, Clone)]
pub struct ExecutionStats {
    pub current_state: ToolExecutionState,
    pub total_transitions: usize,
    pub execution_time: Duration,
    pub state_durations: std::collections::HashMap<ToolExecutionState, Duration>,
    pub is_healthy: bool,
    pub is_terminal: bool,
}

impl ExecutionStats {
    /// Get time spent in a specific state
    #[must_use]
    pub fn time_in_state(&self, state: ToolExecutionState) -> Duration {
        self.state_durations
            .get(&state)
            .copied()
            .unwrap_or(Duration::ZERO)
    }

    /// Get percentage of time spent in a specific state
    #[must_use]
    pub fn state_percentage(&self, state: ToolExecutionState) -> f64 {
        if self.execution_time.is_zero() {
            return 0.0;
        }

        let state_time = self.time_in_state(state);
        (state_time.as_secs_f64() / self.execution_time.as_secs_f64()) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_tool_state_machine_creation() {
        let state_machine = ToolStateMachine::new("test_tool".to_string());
        assert_eq!(
            state_machine.current_state().await,
            ToolExecutionState::Uninitialized
        );
        assert!(
            state_machine
                .is_state(ToolExecutionState::Uninitialized)
                .await
        );
    }
    #[tokio::test]
    async fn test_valid_state_transitions() {
        let state_machine = ToolStateMachine::new("test_tool".to_string());

        // Test normal execution flow
        assert!(state_machine.initialize().await.is_ok());
        assert_eq!(
            state_machine.current_state().await,
            ToolExecutionState::Ready
        );

        assert!(state_machine.start_execution().await.is_ok());
        assert_eq!(
            state_machine.current_state().await,
            ToolExecutionState::Executing
        );

        assert!(state_machine.complete_execution().await.is_ok());
        assert_eq!(
            state_machine.current_state().await,
            ToolExecutionState::Completed
        );

        assert!(state_machine.start_cleanup().await.is_ok());
        assert_eq!(
            state_machine.current_state().await,
            ToolExecutionState::CleaningUp
        );

        assert!(state_machine.terminate().await.is_ok());
        assert_eq!(
            state_machine.current_state().await,
            ToolExecutionState::Terminated
        );
    }
    #[tokio::test]
    async fn test_failure_flow() {
        let state_machine = ToolStateMachine::new("test_tool".to_string());

        assert!(state_machine.initialize().await.is_ok());
        assert!(state_machine.start_execution().await.is_ok());
        assert!(state_machine.fail_execution().await.is_ok());
        assert_eq!(
            state_machine.current_state().await,
            ToolExecutionState::Failed
        );

        assert!(state_machine.start_cleanup().await.is_ok());
        assert!(state_machine.terminate().await.is_ok());
    }
    #[tokio::test]
    async fn test_invalid_transitions() {
        let state_machine = ToolStateMachine::new("test_tool".to_string());

        // Try to execute without initializing
        let result = state_machine.start_execution().await;
        assert!(result.is_err());

        // Try to complete without executing
        assert!(state_machine.initialize().await.is_ok());
        let result = state_machine.complete_execution().await;
        assert!(result.is_err());
    }
    #[tokio::test]
    async fn test_execution_stats() {
        let state_machine = ToolStateMachine::new("test_tool".to_string());

        assert!(state_machine.initialize().await.is_ok());
        tokio::time::sleep(Duration::from_millis(10)).await;

        assert!(state_machine.start_execution().await.is_ok());
        tokio::time::sleep(Duration::from_millis(10)).await;

        assert!(state_machine.complete_execution().await.is_ok());

        let stats = state_machine.get_execution_stats().await;
        assert_eq!(stats.current_state, ToolExecutionState::Completed);
        assert!(stats.total_transitions >= 3);
        assert!(stats.execution_time > Duration::from_millis(20));
        assert!(stats.is_healthy);
        assert!(!stats.is_terminal); // Completed is not terminal until cleanup
    }
    #[tokio::test]
    async fn test_transition_history() {
        let state_machine = ToolStateMachine::new("test_tool".to_string());

        assert!(state_machine.initialize().await.is_ok());
        assert!(state_machine.start_execution().await.is_ok());
        assert!(state_machine.complete_execution().await.is_ok());

        let history = state_machine.get_transition_history().await;
        assert_eq!(history.len(), 4);

        assert_eq!(history[0].from, ToolExecutionState::Uninitialized);
        assert_eq!(history[0].to, ToolExecutionState::Initializing);

        assert_eq!(history[1].from, ToolExecutionState::Initializing);
        assert_eq!(history[1].to, ToolExecutionState::Ready);

        assert_eq!(history[2].from, ToolExecutionState::Ready);
        assert_eq!(history[2].to, ToolExecutionState::Executing);

        assert_eq!(history[3].from, ToolExecutionState::Executing);
        assert_eq!(history[3].to, ToolExecutionState::Completed);
    }
    #[tokio::test]
    async fn test_state_checks() {
        let state_machine = ToolStateMachine::new("test_tool".to_string());

        // Test initial state checks
        assert!(!state_machine.current_state().await.can_execute());
        assert!(!state_machine.current_state().await.is_terminal());
        assert!(!state_machine.current_state().await.is_healthy());

        // After initialization
        assert!(state_machine.initialize().await.is_ok());
        assert!(state_machine.current_state().await.can_execute());
        assert!(state_machine.current_state().await.is_healthy());

        // After termination
        assert!(state_machine.start_execution().await.is_ok());
        assert!(state_machine.complete_execution().await.is_ok());
        assert!(state_machine.start_cleanup().await.is_ok());
        assert!(state_machine.terminate().await.is_ok());
        assert!(state_machine.current_state().await.is_terminal());
        assert!(!state_machine.current_state().await.is_healthy());
    }
}
