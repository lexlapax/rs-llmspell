//! ABOUTME: Agent state machine implementation for comprehensive lifecycle management
//! ABOUTME: Provides deterministic state transitions for agent initialization, execution, pausing, and termination

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::{Mutex, RwLock};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, warn};

// Hook system imports (optional - only used if hooks are enabled)
use llmspell_hooks::circuit_breaker::{BreakerConfig, BreakerState};
use llmspell_hooks::executor::HookExecutorConfig;
use llmspell_hooks::{
    CircuitBreaker, ComponentId, ComponentType, HookContext, HookExecutor, HookPoint, HookRegistry,
    HookResult,
};

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
    #[must_use]
    pub const fn can_execute(&self) -> bool {
        matches!(self, Self::Ready | Self::Running)
    }

    /// Check if state allows pausing
    #[must_use]
    pub const fn can_pause(&self) -> bool {
        matches!(self, Self::Running)
    }

    /// Check if state allows termination
    #[must_use]
    pub const fn can_terminate(&self) -> bool {
        !matches!(self, Self::Terminated | Self::Terminating)
    }

    /// Check if state indicates healthy operation
    #[must_use]
    pub const fn is_healthy(&self) -> bool {
        matches!(self, Self::Ready | Self::Running | Self::Paused)
    }

    /// Check if state indicates error condition
    #[must_use]
    pub const fn is_error(&self) -> bool {
        matches!(self, Self::Error | Self::Recovering)
    }
}

/// Maps agent states to appropriate hook points
fn state_to_hook_point(state: AgentState, is_entering: bool) -> HookPoint {
    match (state, is_entering) {
        (AgentState::Uninitialized, true) => HookPoint::SystemStartup,
        (AgentState::Initializing, true) => HookPoint::BeforeAgentInit,
        (AgentState::Ready, true) => HookPoint::AfterAgentInit,
        (AgentState::Running, true) => HookPoint::BeforeAgentExecution,
        (AgentState::Running, false) => HookPoint::AfterAgentExecution,
        (AgentState::Paused, _) => HookPoint::Custom("agent_paused".to_string()),
        (AgentState::Terminating, true) => HookPoint::Custom("before_agent_terminate".to_string()),
        (AgentState::Terminated, true) => HookPoint::SystemShutdown,
        (AgentState::Error, true) => HookPoint::AgentError,
        (AgentState::Recovering, _) => HookPoint::Custom("agent_recovering".to_string()),
        _ => HookPoint::Custom(format!(
            "agent_state_{:?}_{}",
            state,
            if is_entering { "enter" } else { "exit" }
        )),
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
    /// Enable hook execution during state transitions
    pub enable_hooks: bool,
    /// Hook executor configuration (if hooks enabled)
    pub hook_executor_config: Option<HookExecutorConfig>,
    /// Enable circuit breaker protection for state transitions
    pub enable_circuit_breaker: bool,
    /// Circuit breaker configuration
    pub circuit_breaker_config: BreakerConfig,
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
            enable_hooks: false, // Disabled by default for backward compatibility
            hook_executor_config: None,
            enable_circuit_breaker: true, // Enabled by default for protection
            circuit_breaker_config: BreakerConfig::default(),
        }
    }
}

impl StateMachineConfig {
    /// Create config with hooks enabled
    #[must_use]
    pub fn with_hooks(_hook_registry: Arc<HookRegistry>) -> Self {
        Self {
            enable_hooks: true,
            hook_executor_config: Some(HookExecutorConfig::default()),
            ..Default::default()
        }
    }

    /// Create config with custom hook executor configuration
    #[must_use]
    pub fn with_hook_config(hook_config: HookExecutorConfig) -> Self {
        Self {
            enable_hooks: true,
            hook_executor_config: Some(hook_config),
            ..Default::default()
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
    #[must_use]
    pub fn new(agent_id: String, current: AgentState, target: AgentState) -> Self {
        Self {
            agent_id,
            current_state: current,
            target_state: target,
            metadata: HashMap::new(),
            transition_start: SystemTime::now(),
        }
    }

    #[must_use]
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }

    #[must_use]
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
    #[must_use]
    pub const fn new(state: AgentState) -> Self {
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
        use AgentState::{
            Error, Initializing, Paused, Ready, Recovering, Running, Terminated, Terminating,
            Uninitialized,
        };

        match (self.state, target) {
            // Valid transitions
            (Uninitialized, Initializing | Error)
            | (Initializing, Ready | Error | Terminating)
            | (Ready, Running | Paused | Terminating | Error)
            | (Running, Ready | Paused | Terminating | Error)
            | (Paused, Ready | Running | Terminating | Error)
            | (Error, Recovering | Terminating | Terminated)
            | (Recovering, Ready | Error | Terminating)
            | (Terminating, Terminated | Error) => true,

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
    // Hook system integration (optional)
    hook_executor: Option<Arc<HookExecutor>>,
    hook_registry: Option<Arc<HookRegistry>>,
    active_cancellation_tokens: Arc<Mutex<HashMap<String, CancellationToken>>>,
    // Circuit breaker for state transition protection
    transition_circuit_breaker: Option<Arc<CircuitBreaker>>,
}

impl AgentStateMachine {
    /// Create new state machine for agent
    #[must_use]
    pub fn new(agent_id: String, config: StateMachineConfig) -> Self {
        let transition_circuit_breaker = if config.enable_circuit_breaker {
            Some(Arc::new(CircuitBreaker::with_config(
                format!("{agent_id}-state-transitions"),
                config.circuit_breaker_config.clone(),
            )))
        } else {
            None
        };

        let mut machine = Self {
            agent_id,
            current_state: Arc::new(RwLock::new(AgentState::Uninitialized)),
            config,
            handlers: HashMap::new(),
            transition_history: Arc::new(Mutex::new(Vec::new())),
            recovery_attempts: Arc::new(Mutex::new(0)),
            last_error: Arc::new(Mutex::new(None)),
            hook_executor: None,
            hook_registry: None,
            active_cancellation_tokens: Arc::new(Mutex::new(HashMap::new())),
            transition_circuit_breaker,
        };

        // Install default handlers
        machine.install_default_handlers();
        machine
    }

    /// Create new state machine with hook support
    #[must_use]
    pub fn with_hooks(
        agent_id: String,
        config: StateMachineConfig,
        hook_registry: Arc<HookRegistry>,
    ) -> Self {
        let hook_executor = if config.enable_hooks {
            let executor_config = config.hook_executor_config.clone().unwrap_or_default();
            Some(Arc::new(HookExecutor::with_config(executor_config)))
        } else {
            None
        };

        let transition_circuit_breaker = if config.enable_circuit_breaker {
            Some(Arc::new(CircuitBreaker::with_config(
                format!("{agent_id}-state-transitions"),
                config.circuit_breaker_config.clone(),
            )))
        } else {
            None
        };

        let mut machine = Self {
            agent_id,
            current_state: Arc::new(RwLock::new(AgentState::Uninitialized)),
            config,
            handlers: HashMap::new(),
            transition_history: Arc::new(Mutex::new(Vec::new())),
            recovery_attempts: Arc::new(Mutex::new(0)),
            last_error: Arc::new(Mutex::new(None)),
            hook_executor,
            hook_registry: Some(hook_registry),
            active_cancellation_tokens: Arc::new(Mutex::new(HashMap::new())),
            transition_circuit_breaker,
        };

        // Install default handlers
        machine.install_default_handlers();
        machine
    }

    /// Create state machine with default configuration
    #[must_use]
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

    /// Execute hooks for a state transition phase
    async fn execute_transition_hooks(
        &self,
        state: AgentState,
        is_entering: bool,
        context: &StateContext,
    ) -> Result<()> {
        // Only execute hooks if enabled and components are available
        if !self.config.enable_hooks {
            return Ok(());
        }

        let (hook_executor, hook_registry) = match (&self.hook_executor, &self.hook_registry) {
            (Some(executor), Some(registry)) => (executor, registry),
            _ => return Ok(()), // No hooks configured
        };

        let hook_point = state_to_hook_point(state, is_entering);
        let component_id = ComponentId::new(ComponentType::Agent, self.agent_id.clone());

        // Build hook context with full state information
        let mut hook_context = HookContext::new(hook_point.clone(), component_id);

        // Add metadata about the transition
        hook_context.insert_metadata("agent_id".to_string(), self.agent_id.clone());
        hook_context.insert_metadata(
            "from_state".to_string(),
            format!("{:?}", context.current_state),
        );
        hook_context.insert_metadata(
            "to_state".to_string(),
            format!("{:?}", context.target_state),
        );
        hook_context.insert_metadata(
            "transition_phase".to_string(),
            if is_entering {
                "enter".to_string()
            } else {
                "exit".to_string()
            },
        );

        // Add any additional metadata from the context
        for (key, value) in &context.metadata {
            hook_context.insert_metadata(key.clone(), value.clone());
        }

        // Get hooks for this point
        let hooks = hook_registry.get_hooks(&hook_point);

        if hooks.is_empty() {
            return Ok(());
        }

        // Execute hooks
        let results = hook_executor.execute_hooks(&hooks, &mut hook_context).await;

        match results {
            Ok(hook_results) => {
                // Check results for any that should block the transition
                for result in hook_results {
                    match result {
                        HookResult::Continue | HookResult::Skipped(_) => {}
                        HookResult::Retry {
                            delay: _,
                            max_attempts: _,
                        } => {
                            // For state transitions, we don't support retry
                            warn!("Hook requested retry for state transition - ignoring");
                        }
                        HookResult::Fork {
                            parallel_operations: _,
                        } => {
                            // For state transitions, we don't support forking
                            warn!("Hook attempted to fork state transition - ignoring");
                        }
                        HookResult::Modified(_) => {
                            // For state transitions, modifications are logged but don't affect transition
                            debug!("Hook modified state transition context - continuing");
                        }
                        HookResult::Cancel(reason) => {
                            // Cancel blocks the transition
                            return Err(anyhow!("State transition cancelled by hook: {}", reason));
                        }
                        HookResult::Redirect(target) => {
                            warn!("Hook attempted redirect during state transition to '{}' - ignoring", target);
                        }
                        HookResult::Replace(_) => {
                            warn!("Hook attempted to replace state transition result - ignoring");
                        }
                        HookResult::Cache { key: _, ttl: _ } => {
                            // Caching doesn't affect state transitions
                            debug!("Hook requested caching for state transition - noted");
                        }
                    }
                }

                debug!(
                    "Hooks executed successfully for {:?} ({}) on agent {}",
                    state,
                    if is_entering { "enter" } else { "exit" },
                    self.agent_id
                );
                Ok(())
            }
            Err(e) => {
                error!(
                    "Hook execution failed for {:?} ({}) on agent {}: {}",
                    state,
                    if is_entering { "enter" } else { "exit" },
                    self.agent_id,
                    e
                );
                // Log but continue - hooks should not block transitions on execution errors
                // This ensures backward compatibility and prevents hook failures from breaking agents
                warn!("Hook failed but continuing transition: {}", e);
                Ok(())
            }
        }
    }

    /// Cancel an ongoing state transition
    pub async fn cancel_transition(&self, from_state: AgentState, to_state: AgentState) -> bool {
        let transition_id = format!("{}-{:?}-{:?}", self.agent_id, from_state, to_state);

        let tokens = self.active_cancellation_tokens.lock().await;
        if let Some(token) = tokens.get(&transition_id) {
            token.cancel();
            true
        } else {
            false
        }
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

        // Check circuit breaker before attempting transition
        if let Some(ref circuit_breaker) = self.transition_circuit_breaker {
            match circuit_breaker.state() {
                BreakerState::Open => {
                    return Err(anyhow!(
                        "Circuit breaker is open - state transitions are blocked for agent {}",
                        self.agent_id
                    ));
                }
                BreakerState::HalfOpen => {
                    info!(
                        "Circuit breaker is half-open - attempting state transition for agent {}",
                        self.agent_id
                    );
                }
                BreakerState::Closed => {
                    // Normal operation
                }
            }
        }

        if current == target_state {
            return Ok(()); // No transition needed
        }

        // Create transition ID for cancellation tracking
        let transition_id = format!("{}-{:?}-{:?}", self.agent_id, current, target_state);
        let cancellation_token = CancellationToken::new();

        // Store cancellation token
        {
            let mut tokens = self.active_cancellation_tokens.lock().await;
            tokens.insert(transition_id.clone(), cancellation_token.clone());
        }

        // Execute transition with circuit breaker protection
        let transition_result = if let Some(ref circuit_breaker) = self.transition_circuit_breaker {
            // Check if circuit breaker allows execution
            if !circuit_breaker.can_execute() {
                return Err(anyhow!(
                    "Circuit breaker is open - state transitions are blocked for agent {}",
                    self.agent_id
                ));
            }

            // Execute the transition and record the result
            let start_time_cb = std::time::Instant::now();
            let result = self
                .execute_state_transition(current, target_state, reason.clone(), start_time)
                .await;
            let duration = start_time_cb.elapsed();

            match &result {
                Ok(()) => circuit_breaker.record_success(duration),
                Err(e) => circuit_breaker.record_failure(e),
            }

            result
        } else {
            // Direct execution without circuit breaker
            self.execute_state_transition(current, target_state, reason.clone(), start_time)
                .await
        };

        // Clean up cancellation token
        {
            let mut tokens = self.active_cancellation_tokens.lock().await;
            tokens.remove(&transition_id);
        }

        transition_result
    }

    /// Internal method to execute the actual state transition
    async fn execute_state_transition(
        &self,
        current: AgentState,
        target_state: AgentState,
        reason: Option<String>,
        start_time: SystemTime,
    ) -> Result<()> {
        // Create transition ID for cancellation tracking
        let transition_id = format!("{}-{:?}-{:?}", self.agent_id, current, target_state);
        let cancellation_token = CancellationToken::new();

        // Store cancellation token
        {
            let mut tokens = self.active_cancellation_tokens.lock().await;
            tokens.insert(transition_id.clone(), cancellation_token.clone());
        }

        // Set up timeout for the entire transition
        let timeout_result = tokio::time::timeout(self.config.max_transition_time, async {
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

            let mut context = StateContext::new(self.agent_id.clone(), current, target_state);
            if let Some(ref r) = reason {
                context = context.with_metadata("transition_reason", r);
            }

            if self.config.enable_logging {
                info!(
                    "Agent {} transitioning from {:?} to {:?}{}",
                    self.agent_id,
                    current,
                    target_state,
                    reason
                        .as_ref()
                        .map(|r| format!(" ({r})"))
                        .unwrap_or_default()
                );
            }

            // Execute exit hooks for current state
            if let Err(e) = self
                .execute_transition_hooks(current, false, &context)
                .await
            {
                error!(
                    "Exit hooks failed for state {:?} on agent {}: {}",
                    current, self.agent_id, e
                );
                return Err(e);
            }

            // Check for cancellation
            if cancellation_token.is_cancelled() {
                return Err(anyhow!("State transition cancelled"));
            }

            // Exit current state (existing handler logic)
            if let Some(handler) = self.handlers.get(&current) {
                if let Err(e) = handler.exit(&context).await {
                    error!(
                        "Failed to exit state {:?} for agent {}: {}",
                        current, self.agent_id, e
                    );
                    return Err(e);
                }
            }

            // Check for cancellation again
            if cancellation_token.is_cancelled() {
                return Err(anyhow!("State transition cancelled"));
            }

            // Update current state
            {
                let mut state = self.current_state.write().await;
                *state = target_state;
            }

            // Execute enter hooks for new state
            if let Err(e) = self
                .execute_transition_hooks(target_state, true, &context)
                .await
            {
                error!(
                    "Enter hooks failed for state {:?} on agent {}: {}",
                    target_state, self.agent_id, e
                );

                // Rollback state change
                {
                    let mut state = self.current_state.write().await;
                    *state = current;
                }
                return Err(e);
            }

            // Enter new state (existing handler logic)
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
        })
        .await;

        // Clean up cancellation token
        {
            let mut tokens = self.active_cancellation_tokens.lock().await;
            tokens.remove(&transition_id);
        }

        // Handle timeout
        match timeout_result {
            Ok(result) => result,
            Err(_) => Err(anyhow!(
                "State transition timed out after {:?}",
                self.config.max_transition_time
            )),
        }
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
            Some(format!("Error occurred: {error_message}")),
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
            Some(format!("Recovery attempt {attempt_count}")),
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

    /// Get hook executor metrics (if hooks are enabled)
    pub async fn get_hook_metrics(
        &self,
        hook_name: &str,
    ) -> Option<llmspell_hooks::PerformanceMetrics> {
        self.hook_executor.as_ref()?.get_metrics(hook_name)
    }

    /// Check if hooks are enabled
    #[must_use]
    pub const fn has_hooks(&self) -> bool {
        self.config.enable_hooks && self.hook_executor.is_some() && self.hook_registry.is_some()
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
