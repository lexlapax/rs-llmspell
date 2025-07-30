//! ABOUTME: Session-specific replay adapter that bridges session operations to existing replay infrastructure
//! ABOUTME: Provides session context mapping and converts between session and hook replay formats

use crate::{Result, SessionError, SessionId};
use llmspell_events::EventBus;
use llmspell_hooks::persistence::SerializedHookExecution as HooksSerializedHookExecution;
use llmspell_hooks::replay::{
    BatchReplayRequest, BatchReplayResponse, ReplayConfig, ReplayManager, ReplayMode,
    ReplaySchedule, ReplayState,
};
use llmspell_state_persistence::manager::{HookReplayManager, SerializedHookExecution};
use llmspell_storage::StorageBackend;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant, SystemTime};
use tracing::{debug, info};
use uuid::Uuid;

use super::session_controls::{
    SessionBreakpoint, SessionReplayControlConfig, SessionReplayControls, SessionReplayProgress,
};

/// Session-specific replay configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionReplayConfig {
    /// Replay mode (exact, modified, simulate, debug)
    pub mode: ReplayMode,
    /// Target timestamp for partial replay
    pub target_timestamp: Option<SystemTime>,
    /// Whether to compare results with original execution
    pub compare_results: bool,
    /// Timeout for the entire session replay
    pub timeout: Duration,
    /// Whether to stop on first error
    pub stop_on_error: bool,
    /// Session-specific metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Default for SessionReplayConfig {
    fn default() -> Self {
        Self {
            mode: ReplayMode::Exact,
            target_timestamp: None,
            compare_results: true,
            timeout: Duration::from_secs(300), // 5 minutes for full session
            stop_on_error: true,
            metadata: HashMap::new(),
        }
    }
}

impl SessionReplayConfig {
    /// Convert to the underlying `ReplayConfig`
    pub fn into_replay_config(self) -> ReplayConfig {
        ReplayConfig {
            mode: self.mode,
            modifications: Vec::new(), // Session modifications will be handled separately
            compare_results: self.compare_results,
            timeout: Duration::from_secs(30), // Per-hook timeout
            stop_on_error: self.stop_on_error,
            tags: vec!["session".to_string()],
        }
    }
}

/// Session replay result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionReplayResult {
    /// Session ID that was replayed
    pub session_id: SessionId,
    /// Correlation ID used for the session
    pub correlation_id: Uuid,
    /// Start time of the replay
    pub start_time: SystemTime,
    /// Total duration of the replay
    pub total_duration: Duration,
    /// Number of hooks replayed
    pub hooks_replayed: usize,
    /// Number of successful hook replays
    pub successful_replays: usize,
    /// Number of failed hook replays
    pub failed_replays: usize,
    /// Underlying batch replay response
    pub batch_response: BatchReplayResponse,
    /// Session-specific metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl From<BatchReplayResponse> for SessionReplayResult {
    fn from(batch_response: BatchReplayResponse) -> Self {
        Self {
            session_id: SessionId::new(),   // Will be set by the adapter
            correlation_id: Uuid::new_v4(), // Will be set by the adapter
            start_time: SystemTime::now(),
            total_duration: batch_response.total_duration,
            hooks_replayed: batch_response.results.len(),
            successful_replays: batch_response.success_count,
            failed_replays: batch_response.failure_count,
            batch_response,
            metadata: HashMap::new(),
        }
    }
}

/// Active session replay tracking
#[derive(Debug, Clone)]
pub struct SessionReplayStatus {
    /// Session being replayed
    pub session_id: SessionId,
    /// Current replay state
    pub state: ReplayState,
    /// Start time of replay
    pub start_time: Instant,
    /// Number of hooks processed
    pub hooks_processed: usize,
    /// Total hooks to process
    pub total_hooks: usize,
    /// Current hook being replayed
    pub current_hook: Option<String>,
}

/// Session replay adapter that bridges session operations to existing replay infrastructure
pub struct SessionReplayAdapter {
    /// Core replay manager from llmspell-hooks
    replay_manager: Arc<ReplayManager>,
    /// Hook replay manager from llmspell-state-persistence
    hook_replay_manager: Arc<HookReplayManager>,
    /// Session storage backend
    storage_backend: Arc<dyn StorageBackend>,
    /// Event bus for publishing replay events
    #[allow(dead_code)]
    event_bus: Arc<EventBus>,
    /// Active session replays
    pub(crate) active_replays: Arc<RwLock<HashMap<SessionId, SessionReplayStatus>>>,
    /// Session replay controls
    controls: Arc<SessionReplayControls>,
}

impl SessionReplayAdapter {
    /// Create a new session replay adapter
    pub fn new(
        replay_manager: Arc<ReplayManager>,
        hook_replay_manager: Arc<HookReplayManager>,
        storage_backend: Arc<dyn StorageBackend>,
        event_bus: Arc<EventBus>,
    ) -> Self {
        let control_config = SessionReplayControlConfig::default();
        let controls = Arc::new(SessionReplayControls::new(control_config));

        Self {
            replay_manager,
            hook_replay_manager,
            storage_backend,
            event_bus,
            active_replays: Arc::new(RwLock::new(HashMap::new())),
            controls,
        }
    }

    /// Load session correlation ID from storage
    async fn load_session_correlation_id(&self, session_id: &SessionId) -> Result<Uuid> {
        // Try to load from session_metadata first (new format)
        let metadata_key = format!("session_metadata:{}", session_id);
        if let Ok(Some(metadata_bytes)) = self.storage_backend.get(&metadata_key).await {
            if let Ok(metadata) = serde_json::from_slice::<serde_json::Value>(&metadata_bytes) {
                if let Some(correlation_id_str) =
                    metadata.get("correlation_id").and_then(|v| v.as_str())
                {
                    return Uuid::parse_str(correlation_id_str).map_err(|e| {
                        SessionError::general(format!("Invalid correlation_id: {}", e))
                    });
                }
            }
        }

        // Fallback to try loading from the main session key (for backward compatibility)
        let session_key = format!("session:{}", session_id);
        let session_bytes = self
            .storage_backend
            .get(&session_key)
            .await
            .map_err(|e| SessionError::Storage(e.to_string()))?;

        // The session data is stored in bincode format, not JSON
        // So we can't extract correlation_id from it directly
        // Return an error indicating the session needs to be saved in the new format
        if session_bytes.is_some() {
            Err(SessionError::general(
                "Session exists but correlation_id not found. Session needs to be saved in new format.",
            ))
        } else {
            Err(SessionError::general(format!(
                "Session {} not found",
                session_id
            )))
        }
    }

    /// Convert state-persistence executions to hooks format
    fn convert_executions_to_hooks_format(
        executions: Vec<SerializedHookExecution>,
    ) -> Vec<HooksSerializedHookExecution> {
        executions
            .into_iter()
            .map(|exec| HooksSerializedHookExecution {
                hook_id: exec.hook_id,
                execution_id: exec.execution_id,
                correlation_id: exec.correlation_id,
                hook_context: exec.hook_context,
                result: exec.result,
                timestamp: exec.timestamp,
                duration: exec.duration,
                metadata: exec.metadata,
            })
            .collect()
    }

    /// Check if a session can be replayed
    pub async fn can_replay_session(&self, session_id: &SessionId) -> Result<bool> {
        match self.load_session_correlation_id(session_id).await {
            Ok(correlation_uuid) => {
                // Check if we have hook executions for this correlation_id
                let executions = self
                    .hook_replay_manager
                    .get_hook_executions_by_correlation(correlation_uuid)
                    .await
                    .map_err(|e| SessionError::replay(e.to_string()))?;

                Ok(!executions.is_empty())
            }
            Err(e) => {
                debug!("Cannot replay session {}: {}", session_id, e);
                Ok(false)
            }
        }
    }

    /// Replay a session using the existing replay infrastructure
    ///
    /// # Panics
    ///
    /// Panics if the active replays mutex is poisoned
    pub async fn replay_session(
        &self,
        session_id: &SessionId,
        config: SessionReplayConfig,
    ) -> Result<SessionReplayResult> {
        info!("Starting replay for session {}", session_id);
        let start_time = SystemTime::now();

        // Load session correlation ID
        let correlation_uuid = self.load_session_correlation_id(session_id).await?;

        // Get hook executions for this session (in state-persistence format)
        let mut state_executions = self
            .hook_replay_manager
            .get_hook_executions_by_correlation(correlation_uuid)
            .await
            .map_err(|e| SessionError::replay(format!("Failed to get hook executions: {}", e)))?;

        if state_executions.is_empty() {
            return Err(SessionError::replay("No hook executions found for session"));
        }

        // Sort executions by timestamp for correct replay order
        state_executions.sort_by_key(|e| e.timestamp);

        // Filter executions by target timestamp if specified
        if let Some(target) = config.target_timestamp {
            let before_count = state_executions.len();
            state_executions.retain(|e| e.timestamp <= target);
            info!(
                "Filtered from {} to {} executions up to target timestamp",
                before_count,
                state_executions.len()
            );
        }

        info!(
            "Found {} hook executions to replay for session {}",
            state_executions.len(),
            session_id
        );

        // Track active replay using controls
        self.controls
            .update_progress(session_id, 0, None)
            .unwrap_or_else(|e| {
                debug!("Failed to update initial progress: {}", e);
            });

        // Also track in legacy format for backward compatibility
        {
            let mut active = self.active_replays.write().unwrap();
            active.insert(
                *session_id,
                SessionReplayStatus {
                    session_id: *session_id,
                    state: ReplayState::Running,
                    start_time: Instant::now(),
                    hooks_processed: 0,
                    total_hooks: state_executions.len(),
                    current_hook: None,
                },
            );
        }

        // Convert state-persistence executions to hooks format
        let hook_executions = Self::convert_executions_to_hooks_format(state_executions);

        // Create batch replay request
        let batch_request = BatchReplayRequest {
            executions: hook_executions,
            config: config.clone().into_replay_config(),
            parallel: false, // Session replay should maintain order
            max_concurrent: 1,
        };

        // Execute batch replay using the actual replay infrastructure
        let batch_response = self
            .replay_manager
            .batch_replay(batch_request)
            .await
            .map_err(|e| SessionError::replay(format!("Batch replay failed: {}", e)))?;

        // Create session-specific result
        let result = SessionReplayResult {
            session_id: *session_id,
            correlation_id: correlation_uuid,
            start_time,
            total_duration: start_time.elapsed().unwrap_or_default(),
            hooks_replayed: batch_response.results.len(),
            successful_replays: batch_response.success_count,
            failed_replays: batch_response.failure_count,
            batch_response: batch_response.clone(),
            metadata: config.metadata,
        };

        info!(
            "Session replay completed: {}/{} hooks replayed successfully in {:?}",
            result.successful_replays, result.hooks_replayed, result.total_duration
        );

        // Update replay status
        {
            let mut active = self.active_replays.write().unwrap();
            if let Some(status) = active.get_mut(session_id) {
                status.state = if result.failed_replays > 0 {
                    ReplayState::Failed(format!("{} hooks failed", result.failed_replays))
                } else {
                    ReplayState::Completed
                };
                status.hooks_processed = result.hooks_replayed;
            }
        }

        Ok(result)
    }

    /// Get replay timeline for a session
    pub async fn get_session_timeline(
        &self,
        session_id: &SessionId,
    ) -> Result<Vec<SerializedHookExecution>> {
        // Load session correlation ID
        let correlation_uuid = self.load_session_correlation_id(session_id).await?;

        // Get and sort hook executions
        let mut executions = self
            .hook_replay_manager
            .get_hook_executions_by_correlation(correlation_uuid)
            .await
            .map_err(|e| SessionError::replay(e.to_string()))?;

        executions.sort_by_key(|e| e.timestamp);
        Ok(executions)
    }

    /// Get current replay status for a session
    ///
    /// # Panics
    ///
    /// Panics if the active replays mutex is poisoned
    pub fn get_replay_status(&self, session_id: &SessionId) -> Option<SessionReplayStatus> {
        self.active_replays.read().unwrap().get(session_id).cloned()
    }

    /// Get all active replay statuses
    ///
    /// # Panics
    ///
    /// Panics if the active replays mutex is poisoned
    pub fn get_all_active_replays(&self) -> Vec<SessionReplayStatus> {
        self.active_replays
            .read()
            .unwrap()
            .values()
            .cloned()
            .collect()
    }

    /// Stop/cancel session replay
    ///
    /// # Panics
    ///
    /// Panics if the active replays mutex is poisoned
    pub fn stop_replay(&self, session_id: &SessionId) -> Result<()> {
        let mut active = self.active_replays.write().unwrap();
        if let Some(status) = active.get_mut(session_id) {
            status.state = ReplayState::Cancelled;
            info!("Stopped replay for session {}", session_id);
            Ok(())
        } else {
            Err(SessionError::replay("No active replay found for session"))
        }
    }

    /// Clear completed replays
    ///
    /// # Panics
    ///
    /// Panics if the active replays mutex is poisoned
    pub fn clear_completed_replays(&self) {
        let mut active = self.active_replays.write().unwrap();
        active.retain(|_, status| {
            !matches!(
                status.state,
                ReplayState::Completed | ReplayState::Failed(_) | ReplayState::Cancelled
            )
        });
    }

    /// Update replay progress (called during replay execution)
    ///
    /// # Panics
    ///
    /// Panics if the active replays mutex is poisoned
    pub fn update_replay_progress(
        &self,
        session_id: &SessionId,
        hooks_processed: usize,
        current_hook: Option<String>,
    ) {
        let mut active = self.active_replays.write().unwrap();
        if let Some(status) = active.get_mut(session_id) {
            status.hooks_processed = hooks_processed;
            status.current_hook = current_hook;
        }
    }

    /// Query hook executions for a specific session
    pub async fn query_session_hooks(
        &self,
        session_id: &SessionId,
        filter: SessionHookFilter,
    ) -> Result<Vec<SerializedHookExecution>> {
        // Load session correlation ID
        let correlation_uuid = self.load_session_correlation_id(session_id).await?;

        // Get all hook executions for this correlation
        let mut executions = self
            .hook_replay_manager
            .get_hook_executions_by_correlation(correlation_uuid)
            .await
            .map_err(|e| SessionError::replay(e.to_string()))?;

        // Apply filters
        if let Some(start_time) = filter.start_time {
            executions.retain(|e| e.timestamp >= start_time);
        }

        if let Some(end_time) = filter.end_time {
            executions.retain(|e| e.timestamp <= end_time);
        }

        if let Some(hook_id) = filter.hook_id {
            executions.retain(|e| e.hook_id == hook_id);
        }

        if let Some(max_results) = filter.max_results {
            executions.truncate(max_results);
        }

        // Sort by timestamp
        executions.sort_by_key(|e| e.timestamp);

        Ok(executions)
    }

    /// Get session replay metadata
    pub async fn get_session_replay_metadata(
        &self,
        session_id: &SessionId,
    ) -> Result<SessionReplayMetadata> {
        // Load correlation ID
        let correlation_uuid = self.load_session_correlation_id(session_id).await?;

        // Get hook executions count
        let executions = self
            .hook_replay_manager
            .get_hook_executions_by_correlation(correlation_uuid)
            .await
            .map_err(|e| SessionError::replay(e.to_string()))?;

        let total_hooks = executions.len();
        let first_timestamp = executions.iter().map(|e| e.timestamp).min();
        let last_timestamp = executions.iter().map(|e| e.timestamp).max();
        let total_duration = match (first_timestamp, last_timestamp) {
            (Some(start), Some(end)) => end.duration_since(start).ok(),
            _ => None,
        };

        Ok(SessionReplayMetadata {
            session_id: *session_id,
            correlation_id: correlation_uuid,
            total_hooks,
            first_hook_timestamp: first_timestamp,
            last_hook_timestamp: last_timestamp,
            total_duration,
            can_replay: total_hooks > 0,
        })
    }

    /// List all sessions that can be replayed
    pub async fn list_replayable_sessions(&self) -> Result<Vec<SessionId>> {
        // List all session_metadata keys
        let prefix = "session_metadata:";
        let keys = self
            .storage_backend
            .list_keys(prefix)
            .await
            .map_err(|e| SessionError::Storage(e.to_string()))?;

        let mut replayable_sessions = Vec::new();

        for key in keys {
            // Extract session ID from key
            if let Some(session_id_str) = key.strip_prefix(prefix) {
                if let Ok(session_id) = SessionId::from_str(session_id_str) {
                    // Check if session has hook executions
                    if self.can_replay_session(&session_id).await.unwrap_or(false) {
                        replayable_sessions.push(session_id);
                    }
                }
            }
        }

        Ok(replayable_sessions)
    }

    /// Get the replay controls
    pub fn controls(&self) -> &Arc<SessionReplayControls> {
        &self.controls
    }

    /// Schedule a session replay
    pub async fn schedule_replay(
        &self,
        session_id: &SessionId,
        config: SessionReplayConfig,
        schedule: ReplaySchedule,
    ) -> Result<llmspell_hooks::replay::ScheduledReplay> {
        // Load correlation ID
        let correlation_uuid = self.load_session_correlation_id(session_id).await?;

        // Get hook executions
        let executions = self
            .hook_replay_manager
            .get_hook_executions_by_correlation(correlation_uuid)
            .await
            .map_err(|e| SessionError::replay(format!("Failed to get hook executions: {}", e)))?;

        if executions.is_empty() {
            return Err(SessionError::replay("No hook executions found for session"));
        }

        // Convert to batch replay request
        let hook_executions = Self::convert_executions_to_hooks_format(executions);
        let batch_request = BatchReplayRequest {
            executions: hook_executions,
            config: config.into_replay_config(),
            parallel: false,
            max_concurrent: 1,
        };

        // Schedule using controls
        self.controls
            .schedule_replay(*session_id, batch_request, schedule)
            .await
    }

    /// Pause session replay
    pub async fn pause_replay(&self, session_id: &SessionId) -> Result<()> {
        self.controls.pause_replay(session_id).await
    }

    /// Resume session replay
    pub async fn resume_replay(&self, session_id: &SessionId) -> Result<()> {
        self.controls.resume_replay(session_id).await
    }

    /// Set replay speed
    pub async fn set_replay_speed(&self, session_id: &SessionId, multiplier: f64) -> Result<()> {
        self.controls.set_replay_speed(session_id, multiplier).await
    }

    /// Add a breakpoint
    pub async fn add_breakpoint(&self, breakpoint: SessionBreakpoint) -> Result<()> {
        self.controls.add_breakpoint(breakpoint).await
    }

    /// Remove a breakpoint
    pub async fn remove_breakpoint(
        &self,
        session_id: &SessionId,
        breakpoint_id: Uuid,
    ) -> Result<()> {
        self.controls
            .remove_breakpoint(session_id, breakpoint_id)
            .await
    }

    /// Step to next hook (when paused)
    pub async fn step_next(&self, session_id: &SessionId) -> Result<()> {
        self.controls.step_next(session_id).await
    }

    /// Get session replay progress
    pub fn get_replay_progress(&self, session_id: &SessionId) -> Option<SessionReplayProgress> {
        self.controls.get_progress(session_id)
    }

    /// Get all active replay progresses
    pub fn get_active_replay_progresses(&self) -> Vec<SessionReplayProgress> {
        self.controls.get_active_replays()
    }

    /// Clear session controls
    pub fn clear_session_controls(&self, session_id: &SessionId) {
        self.controls.clear_session_controls(session_id);
    }
}

/// Filter for querying session hooks
#[derive(Debug, Clone, Default)]
pub struct SessionHookFilter {
    /// Start time filter
    pub start_time: Option<SystemTime>,
    /// End time filter
    pub end_time: Option<SystemTime>,
    /// Specific hook ID filter
    pub hook_id: Option<String>,
    /// Maximum number of results
    pub max_results: Option<usize>,
}

/// Session replay metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionReplayMetadata {
    /// Session ID
    pub session_id: SessionId,
    /// Correlation ID
    pub correlation_id: Uuid,
    /// Total number of hooks
    pub total_hooks: usize,
    /// First hook timestamp
    pub first_hook_timestamp: Option<SystemTime>,
    /// Last hook timestamp
    pub last_hook_timestamp: Option<SystemTime>,
    /// Total duration
    pub total_duration: Option<Duration>,
    /// Whether the session can be replayed
    pub can_replay: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_replay_config_default() {
        let config = SessionReplayConfig::default();
        assert_eq!(config.mode, ReplayMode::Exact);
        assert!(config.compare_results);
        assert!(config.stop_on_error);
        assert_eq!(config.timeout, Duration::from_secs(300));
    }

    #[test]
    fn test_session_replay_config_conversion() {
        let session_config = SessionReplayConfig {
            mode: ReplayMode::Debug,
            compare_results: false,
            stop_on_error: false,
            ..Default::default()
        };

        let replay_config = session_config.into_replay_config();
        assert_eq!(replay_config.mode, ReplayMode::Debug);
        assert!(!replay_config.compare_results);
        assert!(!replay_config.stop_on_error);
        assert!(replay_config.tags.contains(&"session".to_string()));
    }

    #[test]
    fn test_session_replay_result_from_batch_response() {
        let batch_response = BatchReplayResponse {
            results: vec![],
            total_duration: Duration::from_secs(10),
            success_count: 5,
            failure_count: 1,
            metadata: HashMap::new(),
        };

        let session_result = SessionReplayResult::from(batch_response);
        assert_eq!(session_result.total_duration, Duration::from_secs(10));
        assert_eq!(session_result.successful_replays, 5);
        assert_eq!(session_result.failed_replays, 1);
        assert_eq!(session_result.hooks_replayed, 0); // Empty results vec
    }
}
