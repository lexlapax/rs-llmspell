//! ABOUTME: Agent state machine implementation for comprehensive lifecycle management
//! ABOUTME: Provides deterministic state transitions for agent initialization, execution, pausing, and termination

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, error, info};

/// Agent lifecycle states
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentState {
    /// Fresh agent, no resources allocated
    Uninitialized,
    /// Resource allocation, tool loading in progress
    Initializing,
    /// Fully initialized, ready for execution
    Ready,
    /// Actively executing tasks
    Running,
    /// Temporarily suspended, state preserved
    Paused,
    /// Graceful shutdown in progress
    Terminating,
    /// Fully shut down, resources released
    Terminated,
    /// Error state, recovery needed
    Error,
    /// Attempting recovery from error
    Recovering,
}

impl AgentState {
    /// Check if state allows execution
    pub fn can_execute(&self) -> bool {
        matches!(self, AgentState::Ready | AgentState::Running)
    }

    /// Check if state allows pausing
    pub fn can_pause(&self) -> bool {
        matches!(self, AgentState::Running)
    }

    /// Check if state allows termination
    pub fn can_terminate(&self) -> bool {
        !matches!(self, AgentState::Terminated | AgentState::Terminating)
    }

    /// Check if state indicates healthy operation
    pub fn is_healthy(&self) -> bool {
        matches!(
            self,
            AgentState::Ready | AgentState::Running | AgentState::Paused
        )
    }

    /// Check if state indicates error condition
    pub fn is_error(&self) -> bool {
        matches!(self, AgentState::Error | AgentState::Recovering)
    }
}

/// State transition metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateTransition {
    pub from: AgentState,
    pub to: AgentState,
    pub timestamp: SystemTime,
    pub duration: Option<Duration>,
    pub reason: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// Agent state machine configuration
#[derive(Debug, Clone)]
pub struct StateMachineConfig {
    /// Maximum time allowed for state transitions
    pub max_transition_time: Duration,
    /// Enable automatic recovery from error states
    pub auto_recovery: bool,
    /// Maximum number of recovery attempts
    pub max_recovery_attempts: usize,
    /// Timeout for initialization process
    pub initialization_timeout: Duration,
    /// Timeout for graceful termination
    pub termination_timeout: Duration,
    /// Enable state transition logging
    pub enable_logging: bool,
    /// Enable state persistence
    pub enable_persistence: bool,
}

impl Default for StateMachineConfig {
    fn default() -> Self {
        Self {
            max_transition_time: Duration::from_millis(5000), // 5 seconds
            auto_recovery: true,
            max_recovery_attempts: 3,
            initialization_timeout: Duration::from_secs(30),
            termination_timeout: Duration::from_secs(10),
            enable_logging: true,
            enable_persistence: false,
        }
    }
}

/// State machine context for transitions
#[derive(Debug, Clone)]
pub struct StateContext {
    pub agent_id: String,
    pub current_state: AgentState,
    pub target_state: AgentState,
    pub metadata: HashMap<String, String>,
    pub transition_start: SystemTime,
}

impl StateContext {
    pub fn new(agent_id: String, current: AgentState, target: AgentState) -> Self {
        Self {
            agent_id,
            current_state: current,
            target_state: target,
            metadata: HashMap::new(),
            transition_start: SystemTime::now(),
        }
    }

    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }

    pub fn elapsed(&self) -> Duration {
        self.transition_start.elapsed().unwrap_or_default()
    }
}

/// Trait for handling state transitions
#[async_trait]
pub trait StateHandler: Send + Sync {
    /// Enter the state
    async fn enter(&self, context: &StateContext) -> Result<()>;

    /// Exit the state
    async fn exit(&self, context: &StateContext) -> Result<()>;

    /// Handle state-specific operations
    async fn handle(&self, context: &StateContext) -> Result<()>;

    /// Validate if transition to target state is allowed
    async fn can_transition_to(&self, target: AgentState) -> bool;

    /// Get state-specific metadata
    fn metadata(&self) -> HashMap<String, String> {
        HashMap::new()
    }
}

/// Default state handlers
pub struct DefaultStateHandler {
    state: AgentState,
}

impl DefaultStateHandler {
    pub fn new(state: AgentState) -> Self {
        Self { state }
    }
}

#[async_trait]
impl StateHandler for DefaultStateHandler {
    async fn enter(&self, context: &StateContext) -> Result<()> {
        debug!("Agent {} entering state {:?}", context.agent_id, self.state);
        Ok(())
    }

    async fn exit(&self, context: &StateContext) -> Result<()> {
        debug!("Agent {} exiting state {:?}", context.agent_id, self.state);
        Ok(())
    }

    async fn handle(&self, _context: &StateContext) -> Result<()> {
        // Default handler does nothing
        Ok(())
    }

    async fn can_transition_to(&self, target: AgentState) -> bool {
        use AgentState::*;

        match (self.state, target) {
            // From Uninitialized
            (Uninitialized, Initializing) => true,
            (Uninitialized, Error) => true,

            // From Initializing
            (Initializing, Ready) => true,
            (Initializing, Error) => true,
            (Initializing, Terminating) => true,

            // From Ready
            (Ready, Running) => true,
            (Ready, Paused) => true,
            (Ready, Terminating) => true,
            (Ready, Error) => true,

            // From Running
            (Running, Ready) => true,
            (Running, Paused) => true,
            (Running, Terminating) => true,
            (Running, Error) => true,

            // From Paused
            (Paused, Ready) => true,
            (Paused, Running) => true,
            (Paused, Terminating) => true,
            (Paused, Error) => true,

            // From Error
            (Error, Recovering) => true,
            (Error, Terminating) => true,
            (Error, Terminated) => true,

            // From Recovering
            (Recovering, Ready) => true,
            (Recovering, Error) => true,
            (Recovering, Terminating) => true,

            // From Terminating
            (Terminating, Terminated) => true,
            (Terminating, Error) => true,

            // From Terminated (final state)
            (Terminated, _) => false,

            // All others not allowed
            _ => false,
        }
    }
}

/// Agent state machine
pub struct AgentStateMachine {
    agent_id: String,
    current_state: Arc<RwLock<AgentState>>,
    config: StateMachineConfig,
    handlers: HashMap<AgentState, Arc<dyn StateHandler>>,
    transition_history: Arc<Mutex<Vec<StateTransition>>>,
    recovery_attempts: Arc<Mutex<usize>>,
    last_error: Arc<Mutex<Option<String>>>,
}

impl AgentStateMachine {
    /// Create new state machine for agent
    pub fn new(agent_id: String, config: StateMachineConfig) -> Self {
        let mut machine = Self {
            agent_id,
            current_state: Arc::new(RwLock::new(AgentState::Uninitialized)),
            config,
            handlers: HashMap::new(),
            transition_history: Arc::new(Mutex::new(Vec::new())),
            recovery_attempts: Arc::new(Mutex::new(0)),
            last_error: Arc::new(Mutex::new(None)),
        };

        // Install default handlers
        machine.install_default_handlers();
        machine
    }

    /// Create state machine with default configuration
    pub fn default(agent_id: String) -> Self {
        Self::new(agent_id, StateMachineConfig::default())
    }

    /// Install default state handlers
    fn install_default_handlers(&mut self) {
        for state in [
            AgentState::Uninitialized,
            AgentState::Initializing,
            AgentState::Ready,
            AgentState::Running,
            AgentState::Paused,
            AgentState::Terminating,
            AgentState::Terminated,
            AgentState::Error,
            AgentState::Recovering,
        ] {
            self.handlers
                .insert(state, Arc::new(DefaultStateHandler::new(state)));
        }
    }

    /// Get current state
    pub async fn current_state(&self) -> AgentState {
        *self.current_state.read().await
    }

    /// Check if agent is in specific state
    pub async fn is_state(&self, state: AgentState) -> bool {
        *self.current_state.read().await == state
    }

    /// Add custom state handler
    pub fn add_handler(&mut self, state: AgentState, handler: Arc<dyn StateHandler>) {
        self.handlers.insert(state, handler);
    }

    /// Transition to new state
    pub async fn transition_to(&self, target_state: AgentState) -> Result<()> {
        self.transition_to_with_reason(target_state, None).await
    }

    /// Transition to new state with reason
    pub async fn transition_to_with_reason(
        &self,
        target_state: AgentState,
        reason: Option<String>,
    ) -> Result<()> {
        let start_time = SystemTime::now();
        let current = self.current_state().await;

        if current == target_state {
            return Ok(()); // No transition needed
        }

        // Check if transition is allowed
        if let Some(handler) = self.handlers.get(&current) {
            if !handler.can_transition_to(target_state).await {
                return Err(anyhow!(
                    "Transition from {:?} to {:?} not allowed",
                    current,
                    target_state
                ));
            }
        }

        let context = StateContext::new(self.agent_id.clone(), current, target_state);

        if self.config.enable_logging {
            info!(
                "Agent {} transitioning from {:?} to {:?}",
                self.agent_id, current, target_state
            );
        }

        // Exit current state
        if let Some(handler) = self.handlers.get(&current) {
            if let Err(e) = handler.exit(&context).await {
                error!(
                    "Failed to exit state {:?} for agent {}: {}",
                    current, self.agent_id, e
                );
                return Err(e);
            }
        }

        // Update current state
        {
            let mut state = self.current_state.write().await;
            *state = target_state;
        }

        // Enter new state
        if let Some(handler) = self.handlers.get(&target_state) {
            if let Err(e) = handler.enter(&context).await {
                error!(
                    "Failed to enter state {:?} for agent {}: {}",
                    target_state, self.agent_id, e
                );

                // Rollback state change
                {
                    let mut state = self.current_state.write().await;
                    *state = current;
                }
                return Err(e);
            }
        }

        // Record transition
        let transition = StateTransition {
            from: current,
            to: target_state,
            timestamp: start_time,
            duration: start_time.elapsed().ok(),
            reason,
            metadata: context.metadata,
        };

        {
            let mut history = self.transition_history.lock().await;
            history.push(transition);
        }

        // Reset recovery attempts on successful transition to healthy state
        if target_state.is_healthy() {
            let mut attempts = self.recovery_attempts.lock().await;
            *attempts = 0;
        }

        if self.config.enable_logging {
            debug!(
                "Agent {} successfully transitioned to {:?} in {:?}",
                self.agent_id,
                target_state,
                start_time.elapsed().unwrap_or_default()
            );
        }

        Ok(())
    }

    /// Initialize agent (transition from Uninitialized to Ready)
    pub async fn initialize(&self) -> Result<()> {
        if !self.is_state(AgentState::Uninitialized).await {
            return Err(anyhow!(
                "Agent can only be initialized from Uninitialized state"
            ));
        }

        // Start initialization
        self.transition_to_with_reason(
            AgentState::Initializing,
            Some("Starting initialization".to_string()),
        )
        .await?;

        // Simulate initialization work
        // Removed sleep to fix slow tests - initialization is synchronous

        // Complete initialization
        self.transition_to_with_reason(
            AgentState::Ready,
            Some("Initialization completed".to_string()),
        )
        .await?;

        Ok(())
    }

    /// Start execution (transition to Running)
    pub async fn start(&self) -> Result<()> {
        let current = self.current_state().await;
        if !matches!(current, AgentState::Ready | AgentState::Paused) {
            return Err(anyhow!(
                "Agent can only start from Ready or Paused state, current: {:?}",
                current
            ));
        }

        self.transition_to_with_reason(AgentState::Running, Some("Starting execution".to_string()))
            .await
    }

    /// Pause execution
    pub async fn pause(&self) -> Result<()> {
        if !self.is_state(AgentState::Running).await {
            return Err(anyhow!("Agent can only be paused from Running state"));
        }

        self.transition_to_with_reason(AgentState::Paused, Some("Pausing execution".to_string()))
            .await
    }

    /// Resume execution
    pub async fn resume(&self) -> Result<()> {
        if !self.is_state(AgentState::Paused).await {
            return Err(anyhow!("Agent can only be resumed from Paused state"));
        }

        self.transition_to_with_reason(AgentState::Running, Some("Resuming execution".to_string()))
            .await
    }

    /// Stop execution (transition to Ready)
    pub async fn stop(&self) -> Result<()> {
        if !self.is_state(AgentState::Running).await {
            return Err(anyhow!("Agent can only be stopped from Running state"));
        }

        self.transition_to_with_reason(AgentState::Ready, Some("Stopping execution".to_string()))
            .await
    }

    /// Terminate agent (graceful shutdown)
    pub async fn terminate(&self) -> Result<()> {
        let current = self.current_state().await;
        if !current.can_terminate() {
            return Err(anyhow!(
                "Agent cannot be terminated from state {:?}",
                current
            ));
        }

        // Start termination
        self.transition_to_with_reason(
            AgentState::Terminating,
            Some("Starting graceful termination".to_string()),
        )
        .await?;

        // Simulate cleanup work
        // Removed sleep to fix slow tests - cleanup is synchronous

        // Complete termination
        self.transition_to_with_reason(
            AgentState::Terminated,
            Some("Termination completed".to_string()),
        )
        .await?;

        Ok(())
    }

    /// Trigger error state
    pub async fn error(&self, error_message: String) -> Result<()> {
        let current = self.current_state().await;
        if current == AgentState::Terminated {
            return Err(anyhow!("Cannot set error on terminated agent"));
        }

        // Store error message
        {
            let mut last_error = self.last_error.lock().await;
            *last_error = Some(error_message.clone());
        }

        self.transition_to_with_reason(
            AgentState::Error,
            Some(format!("Error occurred: {}", error_message)),
        )
        .await
    }

    /// Attempt recovery from error state
    pub async fn recover(&self) -> Result<()> {
        if !self.is_state(AgentState::Error).await {
            return Err(anyhow!("Agent can only recover from Error state"));
        }

        let mut attempts = self.recovery_attempts.lock().await;
        *attempts += 1;
        let attempt_count = *attempts;
        drop(attempts); // Release lock early to avoid potential deadlock

        if attempt_count > self.config.max_recovery_attempts {
            return Err(anyhow!(
                "Maximum recovery attempts ({}) exceeded",
                self.config.max_recovery_attempts
            ));
        }

        // Start recovery
        self.transition_to_with_reason(
            AgentState::Recovering,
            Some(format!("Recovery attempt {}", attempt_count)),
        )
        .await?;

        // Simulate recovery work
        // Removed sleep to fix slow tests - recovery is synchronous

        // Recovery successful - transition to Ready
        self.transition_to_with_reason(
            AgentState::Ready,
            Some("Recovery completed successfully".to_string()),
        )
        .await?;

        // Clear error message
        {
            let mut last_error = self.last_error.lock().await;
            *last_error = None;
        }

        Ok(())
    }

    /// Get transition history
    pub async fn get_transition_history(&self) -> Vec<StateTransition> {
        let history = self.transition_history.lock().await;
        history.clone()
    }

    /// Get last error message
    pub async fn get_last_error(&self) -> Option<String> {
        let last_error = self.last_error.lock().await;
        last_error.clone()
    }

    /// Get recovery attempt count
    pub async fn get_recovery_attempts(&self) -> usize {
        let attempts = self.recovery_attempts.lock().await;
        *attempts
    }

    /// Check if agent is healthy
    pub async fn is_healthy(&self) -> bool {
        self.current_state().await.is_healthy()
    }

    /// Get state machine metrics
    pub async fn get_metrics(&self) -> StateMachineMetrics {
        let current_state = self.current_state().await;
        let history = self.transition_history.lock().await;
        let recovery_attempts = self.recovery_attempts.lock().await;
        let last_error = self.last_error.lock().await;

        StateMachineMetrics {
            agent_id: self.agent_id.clone(),
            current_state,
            total_transitions: history.len(),
            recovery_attempts: *recovery_attempts,
            last_error: last_error.clone(),
            is_healthy: current_state.is_healthy(),
            uptime: history
                .first()
                .and_then(|t| t.timestamp.elapsed().ok())
                .unwrap_or_default(),
        }
    }
}

/// State machine metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateMachineMetrics {
    pub agent_id: String,
    pub current_state: AgentState,
    pub total_transitions: usize,
    pub recovery_attempts: usize,
    pub last_error: Option<String>,
    pub is_healthy: bool,
    pub uptime: Duration,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_state_machine_basic_transitions() {
        let machine = AgentStateMachine::default("test-agent".to_string());

        // Initial state should be Uninitialized
        assert_eq!(machine.current_state().await, AgentState::Uninitialized);

        // Initialize
        machine.initialize().await.unwrap();
        assert_eq!(machine.current_state().await, AgentState::Ready);

        // Start execution
        machine.start().await.unwrap();
        assert_eq!(machine.current_state().await, AgentState::Running);

        // Pause
        machine.pause().await.unwrap();
        assert_eq!(machine.current_state().await, AgentState::Paused);

        // Resume
        machine.resume().await.unwrap();
        assert_eq!(machine.current_state().await, AgentState::Running);

        // Stop
        machine.stop().await.unwrap();
        assert_eq!(machine.current_state().await, AgentState::Ready);

        // Terminate
        machine.terminate().await.unwrap();
        assert_eq!(machine.current_state().await, AgentState::Terminated);
    }

    #[tokio::test]
    async fn test_state_machine_error_handling() {
        let machine = AgentStateMachine::default("test-agent".to_string());

        machine.initialize().await.unwrap();
        machine.start().await.unwrap();

        // Trigger error
        machine
            .error("Test error occurred".to_string())
            .await
            .unwrap();
        assert_eq!(machine.current_state().await, AgentState::Error);

        // Check error message
        let error_msg = machine.get_last_error().await;
        assert_eq!(error_msg, Some("Test error occurred".to_string()));

        // Recover
        machine.recover().await.unwrap();
        assert_eq!(machine.current_state().await, AgentState::Ready);

        // Error message should be cleared
        let error_msg = machine.get_last_error().await;
        assert_eq!(error_msg, None);
    }

    #[tokio::test]
    async fn test_state_machine_invalid_transitions() {
        let machine = AgentStateMachine::default("test-agent".to_string());

        // Cannot start from Uninitialized
        let result = machine.start().await;
        assert!(result.is_err());

        // Cannot pause from Ready
        machine.initialize().await.unwrap();
        let result = machine.pause().await;
        assert!(result.is_err());

        // Cannot resume from Ready
        let result = machine.resume().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_state_machine_history() {
        let machine = AgentStateMachine::default("test-agent".to_string());

        machine.initialize().await.unwrap();
        machine.start().await.unwrap();
        machine.pause().await.unwrap();

        let history = machine.get_transition_history().await;
        assert_eq!(history.len(), 4); // Uninitialized -> Initializing -> Ready -> Running -> Paused

        // Check specific transitions
        assert_eq!(history[0].from, AgentState::Uninitialized);
        assert_eq!(history[0].to, AgentState::Initializing);
        assert_eq!(history[1].from, AgentState::Initializing);
        assert_eq!(history[1].to, AgentState::Ready);
        assert_eq!(history[2].from, AgentState::Ready);
        assert_eq!(history[2].to, AgentState::Running);
        assert_eq!(history[3].from, AgentState::Running);
        assert_eq!(history[3].to, AgentState::Paused);
    }

    #[tokio::test]
    async fn test_state_machine_metrics() {
        let machine = AgentStateMachine::default("test-agent".to_string());

        machine.initialize().await.unwrap();
        machine.start().await.unwrap();

        let metrics = machine.get_metrics().await;
        assert_eq!(metrics.agent_id, "test-agent");
        assert_eq!(metrics.current_state, AgentState::Running);
        assert_eq!(metrics.total_transitions, 3);
        assert_eq!(metrics.recovery_attempts, 0);
        assert!(metrics.is_healthy);
        assert!(metrics.uptime > Duration::from_millis(0));
    }

    #[tokio::test]
    async fn test_state_checks() {
        assert!(AgentState::Ready.can_execute());
        assert!(AgentState::Running.can_execute());
        assert!(!AgentState::Paused.can_execute());

        assert!(AgentState::Running.can_pause());
        assert!(!AgentState::Ready.can_pause());

        assert!(AgentState::Ready.can_terminate());
        assert!(!AgentState::Terminated.can_terminate());

        assert!(AgentState::Ready.is_healthy());
        assert!(AgentState::Running.is_healthy());
        assert!(!AgentState::Error.is_healthy());

        assert!(AgentState::Error.is_error());
        assert!(!AgentState::Ready.is_error());
    }
}
