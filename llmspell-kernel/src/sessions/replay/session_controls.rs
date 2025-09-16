//! ABOUTME: Session-specific replay controls adapting existing replay infrastructure
//! ABOUTME: Provides scheduling, progress tracking, and execution control for session replays

use crate::sessions::{Result, SessionError, SessionId};
use llmspell_hooks::replay::{
    BatchReplayRequest, ReplayMode, ReplaySchedule, ReplayScheduler, ReplayState, ScheduledReplay,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
use tokio::sync::mpsc;
use tracing::debug;
use uuid::Uuid;

/// Session replay control configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionReplayControlConfig {
    /// Default replay mode for sessions
    pub default_mode: ReplayMode,
    /// Default timeout for session replays
    pub default_timeout: Duration,
    /// Whether to enable breakpoints
    pub enable_breakpoints: bool,
    /// Default speed multiplier (1.0 = normal speed)
    pub default_speed_multiplier: f64,
    /// Maximum concurrent session replays
    pub max_concurrent_replays: usize,
}

impl Default for SessionReplayControlConfig {
    fn default() -> Self {
        Self {
            default_mode: ReplayMode::Exact,
            default_timeout: Duration::from_secs(300), // 5 minutes for full session
            enable_breakpoints: false,
            default_speed_multiplier: 1.0,
            max_concurrent_replays: 10,
        }
    }
}

/// Session-specific breakpoint condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionBreakpointCondition {
    /// Break at specific hook execution
    HookExecution {
        /// ID of the hook to break on
        hook_id: String,
    },
    /// Break at specific timestamp
    Timestamp {
        /// Timestamp to break at
        timestamp: SystemTime,
    },
    /// Break on error
    OnError,
    /// Break on specific session state
    SessionState {
        /// Key in session state to check
        state_key: String,
        /// Expected value for the state key
        expected_value: serde_json::Value,
    },
    /// Break after N hooks executed
    HookCount {
        /// Number of hooks after which to break
        count: usize,
    },
}

/// Session replay breakpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionBreakpoint {
    /// Unique breakpoint ID
    pub id: Uuid,
    /// Session this breakpoint applies to
    pub session_id: SessionId,
    /// Breakpoint condition
    pub condition: SessionBreakpointCondition,
    /// Whether breakpoint is enabled
    pub enabled: bool,
    /// One-time breakpoint (auto-disable after hit)
    pub one_shot: bool,
    /// Callback data when breakpoint hits
    pub callback_data: Option<serde_json::Value>,
}

/// Extended replay state for sessions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SessionReplayState {
    /// Scheduled but not started
    Scheduled,
    /// Currently running
    Running,
    /// Paused by user
    Paused,
    /// Completed successfully
    Completed,
    /// Failed with error
    Failed(String),
    /// Cancelled by user
    Cancelled,
}

impl From<ReplayState> for SessionReplayState {
    fn from(state: ReplayState) -> Self {
        match state {
            ReplayState::Running => SessionReplayState::Running,
            ReplayState::Completed => SessionReplayState::Completed,
            ReplayState::Failed(msg) => SessionReplayState::Failed(msg),
            ReplayState::Cancelled => SessionReplayState::Cancelled,
        }
    }
}

impl SessionReplayState {
    /// Convert to base replay state if possible
    pub fn to_replay_state(&self) -> Option<ReplayState> {
        match self {
            SessionReplayState::Running => Some(ReplayState::Running),
            SessionReplayState::Completed => Some(ReplayState::Completed),
            SessionReplayState::Failed(msg) => Some(ReplayState::Failed(msg.clone())),
            SessionReplayState::Cancelled => Some(ReplayState::Cancelled),
            _ => None, // Scheduled and Paused don't map
        }
    }
}

/// Session replay progress information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionReplayProgress {
    /// Session being replayed
    pub session_id: SessionId,
    /// Current replay state
    pub state: SessionReplayState,
    /// Total hooks to replay
    pub total_hooks: usize,
    /// Hooks completed
    pub hooks_completed: usize,
    /// Current hook being replayed
    pub current_hook: Option<String>,
    /// Start time of replay
    pub start_time: SystemTime,
    /// Estimated time remaining
    pub estimated_time_remaining: Option<Duration>,
    /// Current speed multiplier
    pub speed_multiplier: f64,
    /// Active breakpoints
    pub active_breakpoints: Vec<Uuid>,
    /// Progress percentage (0-100)
    pub progress_percentage: f64,
}

/// Session replay speed control
#[derive(Debug, Clone)]
pub struct SessionReplaySpeed {
    /// Current speed multiplier
    multiplier: f64,
    /// Minimum speed (0.1x)
    min_speed: f64,
    /// Maximum speed (10x)
    max_speed: f64,
    /// Speed change step
    step: f64,
}

impl Default for SessionReplaySpeed {
    fn default() -> Self {
        Self {
            multiplier: 1.0,
            min_speed: 0.1,
            max_speed: 10.0,
            step: 0.5,
        }
    }
}

impl SessionReplaySpeed {
    /// Get current speed multiplier
    pub fn multiplier(&self) -> f64 {
        self.multiplier
    }

    /// Increase speed
    pub fn increase(&mut self) {
        self.multiplier = (self.multiplier + self.step).min(self.max_speed);
    }

    /// Decrease speed
    pub fn decrease(&mut self) {
        self.multiplier = (self.multiplier - self.step).max(self.min_speed);
    }

    /// Set specific speed
    pub fn set_speed(&mut self, speed: f64) {
        self.multiplier = speed.clamp(self.min_speed, self.max_speed);
    }

    /// Reset to normal speed
    pub fn reset(&mut self) {
        self.multiplier = 1.0;
    }

    /// Apply speed to duration
    pub fn apply_to_duration(&self, duration: Duration) -> Duration {
        if (self.multiplier - 1.0).abs() < f64::EPSILON {
            duration
        } else {
            Duration::from_secs_f64(duration.as_secs_f64() / self.multiplier)
        }
    }
}

/// Progress update callback
pub type ProgressCallback = Box<dyn Fn(&SessionReplayProgress) + Send + Sync>;

/// Breakpoint hit callback
pub type BreakpointCallback = Box<dyn Fn(&SessionBreakpoint, &SessionReplayProgress) + Send + Sync>;

/// Session replay controls
pub struct SessionReplayControls {
    /// Configuration
    config: SessionReplayControlConfig,
    /// Replay scheduler (from llmspell-hooks)
    scheduler: Arc<ReplayScheduler>,
    /// Active session replays
    active_replays: Arc<RwLock<HashMap<SessionId, SessionReplayProgress>>>,
    /// Session breakpoints
    breakpoints: Arc<RwLock<HashMap<SessionId, Vec<SessionBreakpoint>>>>,
    /// Speed controls per session
    speed_controls: Arc<RwLock<HashMap<SessionId, SessionReplaySpeed>>>,
    /// Progress callbacks
    progress_callbacks: Arc<RwLock<HashMap<SessionId, Vec<ProgressCallback>>>>,
    /// Breakpoint callbacks
    breakpoint_callbacks: Arc<RwLock<HashMap<SessionId, Vec<BreakpointCallback>>>>,
    /// Control command channel
    command_tx: mpsc::Sender<ControlCommand>,
    #[allow(dead_code)]
    command_rx: Arc<RwLock<mpsc::Receiver<ControlCommand>>>,
}

/// Control commands for session replay
#[derive(Debug)]
#[allow(dead_code)]
enum ControlCommand {
    /// Pause replay
    Pause { session_id: SessionId },
    /// Resume replay
    Resume { session_id: SessionId },
    /// Stop replay
    Stop { session_id: SessionId },
    /// Set speed
    SetSpeed {
        session_id: SessionId,
        multiplier: f64,
    },
    /// Add breakpoint
    AddBreakpoint { breakpoint: SessionBreakpoint },
    /// Remove breakpoint
    RemoveBreakpoint {
        session_id: SessionId,
        breakpoint_id: Uuid,
    },
    /// Step to next hook
    StepNext { session_id: SessionId },
}

impl SessionReplayControls {
    /// Create new session replay controls
    pub fn new(config: SessionReplayControlConfig) -> Self {
        let (command_tx, command_rx) = mpsc::channel(100);

        Self {
            config,
            scheduler: Arc::new(ReplayScheduler::new()),
            active_replays: Arc::new(RwLock::new(HashMap::new())),
            breakpoints: Arc::new(RwLock::new(HashMap::new())),
            speed_controls: Arc::new(RwLock::new(HashMap::new())),
            progress_callbacks: Arc::new(RwLock::new(HashMap::new())),
            breakpoint_callbacks: Arc::new(RwLock::new(HashMap::new())),
            command_tx,
            command_rx: Arc::new(RwLock::new(command_rx)),
        }
    }

    /// Schedule a session replay
    ///
    /// # Panics
    ///
    /// Panics if the speed controls or active replays mutex is poisoned
    pub async fn schedule_replay(
        &self,
        session_id: SessionId,
        replay_request: BatchReplayRequest,
        schedule: ReplaySchedule,
    ) -> Result<ScheduledReplay> {
        // Initialize speed control for session
        {
            let mut speed_controls = self.speed_controls.write().unwrap();
            speed_controls.entry(session_id).or_default();
        }

        // Initialize progress tracking
        {
            let mut active_replays = self.active_replays.write().unwrap();
            active_replays.insert(
                session_id,
                SessionReplayProgress {
                    session_id,
                    state: SessionReplayState::Scheduled,
                    total_hooks: replay_request.executions.len(),
                    hooks_completed: 0,
                    current_hook: None,
                    start_time: SystemTime::now(),
                    estimated_time_remaining: None,
                    speed_multiplier: self.config.default_speed_multiplier,
                    active_breakpoints: Vec::new(),
                    progress_percentage: 0.0,
                },
            );
        }

        // Convert to hooks replay request (simplified for now)
        let hooks_request = llmspell_hooks::replay::ReplayRequest {
            execution_id: replay_request
                .executions
                .first()
                .map(|e| e.execution_id)
                .unwrap_or_else(Uuid::new_v4),
            config: replay_request.config,
            correlation_id: replay_request.executions.first().map(|e| e.correlation_id),
        };

        // Schedule using existing scheduler
        let id = self
            .scheduler
            .schedule(hooks_request.clone(), schedule.clone())
            .await
            .map_err(|e| SessionError::replay(format!("Failed to schedule replay: {}", e)))?;

        // Create a ScheduledReplay response
        Ok(ScheduledReplay {
            id,
            request: hooks_request,
            schedule,
            next_execution: chrono::Utc::now(),
            execution_count: 0,
            active: true,
            created_at: chrono::Utc::now(),
            last_execution: None,
            last_result: None,
        })
    }

    /// Pause session replay
    pub async fn pause_replay(&self, session_id: &SessionId) -> Result<()> {
        self.command_tx
            .send(ControlCommand::Pause {
                session_id: *session_id,
            })
            .await
            .map_err(|e| SessionError::replay(format!("Failed to send pause command: {}", e)))?;

        self.update_replay_state(session_id, SessionReplayState::Paused)?;
        Ok(())
    }

    /// Resume session replay
    pub async fn resume_replay(&self, session_id: &SessionId) -> Result<()> {
        self.command_tx
            .send(ControlCommand::Resume {
                session_id: *session_id,
            })
            .await
            .map_err(|e| SessionError::replay(format!("Failed to send resume command: {}", e)))?;

        self.update_replay_state(session_id, SessionReplayState::Running)?;
        Ok(())
    }

    /// Stop session replay
    ///
    /// # Panics
    ///
    /// Panics if the active replays mutex is poisoned
    pub async fn stop_replay(&self, session_id: &SessionId) -> Result<()> {
        self.command_tx
            .send(ControlCommand::Stop {
                session_id: *session_id,
            })
            .await
            .map_err(|e| SessionError::replay(format!("Failed to send stop command: {}", e)))?;

        self.update_replay_state(session_id, SessionReplayState::Cancelled)?;

        // Clean up
        {
            let mut active_replays = self.active_replays.write().unwrap();
            active_replays.remove(session_id);
        }

        Ok(())
    }

    /// Set replay speed
    ///
    /// # Panics
    ///
    /// Panics if the speed controls mutex is poisoned
    pub async fn set_replay_speed(&self, session_id: &SessionId, multiplier: f64) -> Result<()> {
        // Update speed control
        {
            let mut speed_controls = self.speed_controls.write().unwrap();
            if let Some(speed_control) = speed_controls.get_mut(session_id) {
                speed_control.set_speed(multiplier);
            }
        }

        // Send command
        self.command_tx
            .send(ControlCommand::SetSpeed {
                session_id: *session_id,
                multiplier,
            })
            .await
            .map_err(|e| SessionError::replay(format!("Failed to send speed command: {}", e)))?;

        // Update progress
        self.update_progress_speed(session_id, multiplier)?;

        Ok(())
    }

    /// Add a breakpoint
    ///
    /// # Panics
    ///
    /// Panics if the breakpoints mutex is poisoned
    pub async fn add_breakpoint(&self, breakpoint: SessionBreakpoint) -> Result<()> {
        {
            let mut breakpoints = self.breakpoints.write().unwrap();
            breakpoints
                .entry(breakpoint.session_id)
                .or_default()
                .push(breakpoint.clone());
        }

        self.command_tx
            .send(ControlCommand::AddBreakpoint { breakpoint })
            .await
            .map_err(|e| SessionError::replay(format!("Failed to add breakpoint: {}", e)))?;

        Ok(())
    }

    /// Remove a breakpoint
    ///
    /// # Panics
    ///
    /// Panics if the breakpoints mutex is poisoned
    pub async fn remove_breakpoint(
        &self,
        session_id: &SessionId,
        breakpoint_id: Uuid,
    ) -> Result<()> {
        {
            let mut breakpoints = self.breakpoints.write().unwrap();
            if let Some(session_breakpoints) = breakpoints.get_mut(session_id) {
                session_breakpoints.retain(|bp| bp.id != breakpoint_id);
            }
        }

        self.command_tx
            .send(ControlCommand::RemoveBreakpoint {
                session_id: *session_id,
                breakpoint_id,
            })
            .await
            .map_err(|e| SessionError::replay(format!("Failed to remove breakpoint: {}", e)))?;

        Ok(())
    }

    /// Step to next hook (when paused)
    pub async fn step_next(&self, session_id: &SessionId) -> Result<()> {
        self.command_tx
            .send(ControlCommand::StepNext {
                session_id: *session_id,
            })
            .await
            .map_err(|e| SessionError::replay(format!("Failed to send step command: {}", e)))?;

        Ok(())
    }

    /// Get current progress
    ///
    /// # Panics
    ///
    /// Panics if the active replays mutex is poisoned
    pub fn get_progress(&self, session_id: &SessionId) -> Option<SessionReplayProgress> {
        self.active_replays.read().unwrap().get(session_id).cloned()
    }

    /// Get all active replays
    ///
    /// # Panics
    ///
    /// Panics if the active replays mutex is poisoned
    pub fn get_active_replays(&self) -> Vec<SessionReplayProgress> {
        self.active_replays
            .read()
            .unwrap()
            .values()
            .cloned()
            .collect()
    }

    /// Register progress callback
    ///
    /// # Panics
    ///
    /// Panics if the progress callbacks mutex is poisoned
    pub fn register_progress_callback(&self, session_id: SessionId, callback: ProgressCallback) {
        let mut callbacks = self.progress_callbacks.write().unwrap();
        callbacks.entry(session_id).or_default().push(callback);
    }

    /// Register breakpoint callback
    ///
    /// # Panics
    ///
    /// Panics if the breakpoint callbacks mutex is poisoned
    pub fn register_breakpoint_callback(
        &self,
        session_id: SessionId,
        callback: BreakpointCallback,
    ) {
        let mut callbacks = self.breakpoint_callbacks.write().unwrap();
        callbacks.entry(session_id).or_default().push(callback);
    }

    /// Update replay state
    fn update_replay_state(&self, session_id: &SessionId, state: SessionReplayState) -> Result<()> {
        let mut active_replays = self.active_replays.write().unwrap();
        if let Some(progress) = active_replays.get_mut(session_id) {
            progress.state = state;
            self.notify_progress_callbacks(progress);
            Ok(())
        } else {
            Err(SessionError::replay("No active replay found for session"))
        }
    }

    /// Update progress speed
    fn update_progress_speed(&self, session_id: &SessionId, multiplier: f64) -> Result<()> {
        let mut active_replays = self.active_replays.write().unwrap();
        if let Some(progress) = active_replays.get_mut(session_id) {
            progress.speed_multiplier = multiplier;
            self.notify_progress_callbacks(progress);
            Ok(())
        } else {
            Err(SessionError::replay("No active replay found for session"))
        }
    }

    /// Update replay progress
    ///
    /// # Panics
    ///
    /// Panics if the active replays or speed controls mutex is poisoned
    pub fn update_progress(
        &self,
        session_id: &SessionId,
        hooks_completed: usize,
        current_hook: Option<String>,
    ) -> Result<()> {
        let mut active_replays = self.active_replays.write().unwrap();
        if let Some(progress) = active_replays.get_mut(session_id) {
            progress.hooks_completed = hooks_completed;
            progress.current_hook = current_hook;
            progress.progress_percentage = if progress.total_hooks > 0 {
                #[allow(clippy::cast_precision_loss)]
                let percentage = (hooks_completed as f64 / progress.total_hooks as f64) * 100.0;
                percentage
            } else {
                0.0
            };

            // Estimate time remaining based on current speed
            if hooks_completed > 0 {
                let elapsed = progress.start_time.elapsed().unwrap_or_default();
                #[allow(clippy::cast_precision_loss)]
                let per_hook = elapsed.div_f64(hooks_completed as f64);
                let remaining_hooks = progress.total_hooks - hooks_completed;
                let speed_controls = self.speed_controls.read().unwrap();
                let speed = speed_controls
                    .get(session_id)
                    .map(|s| s.multiplier())
                    .unwrap_or(1.0);
                #[allow(clippy::cast_precision_loss)]
                let remaining = per_hook.mul_f64(remaining_hooks as f64);
                progress.estimated_time_remaining =
                    Some(Duration::from_secs_f64(remaining.as_secs_f64() / speed));
            }

            self.notify_progress_callbacks(progress);

            // Check breakpoints
            self.check_breakpoints(session_id, progress);

            Ok(())
        } else {
            Err(SessionError::replay("No active replay found for session"))
        }
    }

    /// Notify progress callbacks
    fn notify_progress_callbacks(&self, progress: &SessionReplayProgress) {
        let callbacks = self.progress_callbacks.read().unwrap();
        if let Some(session_callbacks) = callbacks.get(&progress.session_id) {
            for callback in session_callbacks {
                callback(progress);
            }
        }
    }

    /// Check and handle breakpoints
    fn check_breakpoints(&self, session_id: &SessionId, progress: &mut SessionReplayProgress) {
        let breakpoints = self.breakpoints.read().unwrap();
        if let Some(session_breakpoints) = breakpoints.get(session_id) {
            for breakpoint in session_breakpoints {
                if !breakpoint.enabled {
                    continue;
                }

                let should_break = match &breakpoint.condition {
                    SessionBreakpointCondition::HookExecution { hook_id } => {
                        progress.current_hook.as_ref() == Some(hook_id)
                    }
                    SessionBreakpointCondition::HookCount { count } => {
                        progress.hooks_completed >= *count
                    }
                    SessionBreakpointCondition::OnError => {
                        matches!(progress.state, SessionReplayState::Failed(_))
                    }
                    _ => false, // Other conditions need more context
                };

                if should_break {
                    progress.state = SessionReplayState::Paused;
                    progress.active_breakpoints.push(breakpoint.id);

                    // Notify breakpoint callbacks
                    let bp_callbacks = self.breakpoint_callbacks.read().unwrap();
                    if let Some(callbacks) = bp_callbacks.get(session_id) {
                        for callback in callbacks {
                            callback(breakpoint, progress);
                        }
                    }

                    if breakpoint.one_shot {
                        // Mark for removal (will be done after iteration)
                        debug!("One-shot breakpoint {} hit", breakpoint.id);
                    }
                }
            }
        }
    }

    /// Clear all controls for a session
    ///
    /// # Panics
    ///
    /// Panics if any of the control mutexes are poisoned
    pub fn clear_session_controls(&self, session_id: &SessionId) {
        {
            let mut active_replays = self.active_replays.write().unwrap();
            active_replays.remove(session_id);
        }
        {
            let mut breakpoints = self.breakpoints.write().unwrap();
            breakpoints.remove(session_id);
        }
        {
            let mut speed_controls = self.speed_controls.write().unwrap();
            speed_controls.remove(session_id);
        }
        {
            let mut callbacks = self.progress_callbacks.write().unwrap();
            callbacks.remove(session_id);
        }
        {
            let mut callbacks = self.breakpoint_callbacks.write().unwrap();
            callbacks.remove(session_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_speed_control() {
        let mut speed = SessionReplaySpeed::default();
        assert!((speed.multiplier() - 1.0).abs() < f64::EPSILON);

        speed.increase();
        assert!((speed.multiplier() - 1.5).abs() < f64::EPSILON);

        speed.decrease();
        assert!((speed.multiplier() - 1.0).abs() < f64::EPSILON);

        speed.set_speed(5.0);
        assert!((speed.multiplier() - 5.0).abs() < f64::EPSILON);

        speed.set_speed(20.0); // Should clamp to max
        assert!((speed.multiplier() - 10.0).abs() < f64::EPSILON);

        let duration = Duration::from_secs(10);
        let adjusted = speed.apply_to_duration(duration);
        assert_eq!(adjusted, Duration::from_secs(1)); // 10s / 10x = 1s
    }
    #[test]
    fn test_breakpoint_conditions() {
        let breakpoint = SessionBreakpoint {
            id: Uuid::new_v4(),
            session_id: SessionId::new(),
            condition: SessionBreakpointCondition::HookCount { count: 5 },
            enabled: true,
            one_shot: false,
            callback_data: None,
        };

        assert!(breakpoint.enabled);
        assert!(!breakpoint.one_shot);
    }
    #[test]
    fn test_replay_progress() {
        let progress = SessionReplayProgress {
            session_id: SessionId::new(),
            state: SessionReplayState::Running,
            total_hooks: 100,
            hooks_completed: 25,
            current_hook: Some("test_hook".to_string()),
            start_time: SystemTime::now(),
            estimated_time_remaining: Some(Duration::from_secs(75)),
            speed_multiplier: 1.0,
            active_breakpoints: Vec::new(),
            progress_percentage: 25.0,
        };

        assert!((progress.progress_percentage - 25.0).abs() < f64::EPSILON);
        assert_eq!(progress.hooks_completed, 25);
        assert_eq!(progress.total_hooks, 100);
    }
}
