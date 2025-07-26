// ABOUTME: Comprehensive replay management system for hook executions
// ABOUTME: Provides advanced replay capabilities with timeline reconstruction and debugging

use crate::persistence::{
    HookPersistenceManager, HookReplayEngine, ReplayOptions, SerializedHookExecution,
    StorageBackend,
};
use crate::result::HookResult;
use crate::traits::ReplayableHook;
use crate::types::HookPoint;
use anyhow::Result;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Replay session configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplaySessionConfig {
    /// Session name for identification
    pub name: String,
    /// Whether to capture intermediate states
    pub capture_states: bool,
    /// Whether to validate hook outputs
    pub validate_outputs: bool,
    /// Speed multiplier for replay (1.0 = real-time)
    pub speed_multiplier: f64,
    /// Break on errors during replay
    pub break_on_error: bool,
    /// Maximum memory for captured states
    pub max_memory_mb: usize,
}

impl Default for ReplaySessionConfig {
    fn default() -> Self {
        Self {
            name: format!("replay_{}", chrono::Utc::now().format("%Y%m%d_%H%M%S")),
            capture_states: true,
            validate_outputs: true,
            speed_multiplier: 1.0,
            break_on_error: true,
            max_memory_mb: 100,
        }
    }
}

/// Replay session state
#[derive(Debug)]
pub struct ReplaySession {
    pub config: ReplaySessionConfig,
    pub start_time: SystemTime,
    pub executions_replayed: u64,
    pub errors_encountered: Vec<ReplayError>,
    pub captured_states: VecDeque<CapturedState>,
    pub breakpoints: Vec<ReplayBreakpoint>,
}

/// Captured state during replay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapturedState {
    pub timestamp: SystemTime,
    pub execution_id: Uuid,
    pub hook_id: String,
    pub context_snapshot: serde_json::Value,
    pub result: HookResult,
    pub metadata: HashMap<String, String>,
}

/// Replay error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayError {
    pub timestamp: SystemTime,
    pub execution_id: Uuid,
    pub hook_id: String,
    pub error_message: String,
    pub error_type: ReplayErrorType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReplayErrorType {
    DeserializationError,
    ExecutionError,
    ValidationError,
    TimeoutError,
    ResourceError,
}

/// Breakpoint for debugging replay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayBreakpoint {
    pub condition: BreakpointCondition,
    pub action: BreakpointAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BreakpointCondition {
    HookId(String),
    HookPoint(HookPoint),
    ExecutionId(Uuid),
    ErrorOccurred,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BreakpointAction {
    Pause,
    Log(String),
    ModifyContext(serde_json::Value),
    Skip,
}

/// Replay timeline for visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayTimeline {
    pub entries: Vec<TimelineEntry>,
    pub total_duration: Duration,
    pub component_interactions: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimelineEntry {
    pub timestamp: SystemTime,
    pub relative_time: Duration,
    pub execution_id: Uuid,
    pub hook_id: String,
    pub hook_point: HookPoint,
    pub component_id: String,
    pub duration: Duration,
    pub result_type: String,
}

/// Advanced replay manager with debugging capabilities
pub struct ReplayManager {
    #[allow(dead_code)]
    persistence_manager: Arc<HookPersistenceManager>,
    storage_backend: Arc<dyn StorageBackend>,
    replay_engine: Arc<RwLock<HookReplayEngine>>,
    active_sessions: Arc<RwLock<HashMap<String, ReplaySession>>>,
    hook_registry: Arc<RwLock<HashMap<String, Arc<dyn ReplayableHook>>>>,
}

impl ReplayManager {
    /// Create a new replay manager
    pub fn new(
        persistence_manager: Arc<HookPersistenceManager>,
        storage_backend: Arc<dyn StorageBackend>,
    ) -> Self {
        Self {
            persistence_manager,
            storage_backend,
            replay_engine: Arc::new(RwLock::new(HookReplayEngine::new())),
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
            hook_registry: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a replayable hook
    pub fn register_hook(&self, hook_id: String, hook: Arc<dyn ReplayableHook>) {
        self.hook_registry.write().insert(hook_id, hook);
    }

    /// Start a new replay session
    pub async fn start_replay_session(&self, config: ReplaySessionConfig) -> Result<String> {
        let session_name = config.name.clone();
        let session = ReplaySession {
            config,
            start_time: SystemTime::now(),
            executions_replayed: 0,
            errors_encountered: Vec::new(),
            captured_states: VecDeque::new(),
            breakpoints: Vec::new(),
        };

        self.active_sessions
            .write()
            .insert(session_name.clone(), session);

        info!("Started replay session: {}", session_name);
        Ok(session_name)
    }

    /// Replay executions by correlation ID
    pub async fn replay_by_correlation(
        &self,
        session_name: &str,
        correlation_id: Uuid,
        options: ReplayOptions,
    ) -> Result<Vec<HookResult>> {
        // Load executions from storage
        let executions = self
            .storage_backend
            .load_executions_by_correlation(&correlation_id)
            .await?;

        if executions.is_empty() {
            warn!("No executions found for correlation ID: {}", correlation_id);
            return Ok(Vec::new());
        }

        info!(
            "Found {} executions for correlation ID: {}",
            executions.len(),
            correlation_id
        );

        // Replay executions
        self.replay_executions(session_name, executions, options)
            .await
    }

    /// Replay specific executions
    pub async fn replay_executions(
        &self,
        session_name: &str,
        executions: Vec<SerializedHookExecution>,
        options: ReplayOptions,
    ) -> Result<Vec<HookResult>> {
        let mut results = Vec::new();
        let mut session = self
            .active_sessions
            .write()
            .get_mut(session_name)
            .ok_or_else(|| anyhow::anyhow!("Session not found: {}", session_name))?
            .clone();

        for execution in executions {
            // Check breakpoints
            if self.should_break(&session, &execution) {
                self.handle_breakpoint(&mut session, &execution).await?;
            }

            // Get the hook
            let hook = self
                .hook_registry
                .read()
                .get(&execution.hook_id)
                .cloned()
                .ok_or_else(|| {
                    anyhow::anyhow!("Hook not registered for replay: {}", execution.hook_id)
                })?;

            // Replay the execution
            match self
                .replay_single_execution(&mut session, hook.as_ref(), &execution, &options)
                .await
            {
                Ok(result) => {
                    results.push(result);
                    session.executions_replayed += 1;
                }
                Err(e) => {
                    let error = ReplayError {
                        timestamp: SystemTime::now(),
                        execution_id: execution.execution_id,
                        hook_id: execution.hook_id.clone(),
                        error_message: e.to_string(),
                        error_type: ReplayErrorType::ExecutionError,
                    };
                    session.errors_encountered.push(error);

                    if session.config.break_on_error {
                        return Err(e);
                    }
                }
            }
        }

        // Update session
        self.active_sessions
            .write()
            .insert(session_name.to_string(), session);

        Ok(results)
    }

    /// Replay a single execution with session tracking
    async fn replay_single_execution(
        &self,
        session: &mut ReplaySession,
        hook: &dyn ReplayableHook,
        execution: &SerializedHookExecution,
        options: &ReplayOptions,
    ) -> Result<HookResult> {
        let start = std::time::Instant::now();

        // Apply speed multiplier
        if options.simulate_timing && session.config.speed_multiplier != 1.0 {
            let adjusted_duration = Duration::from_secs_f64(
                execution.duration.as_secs_f64() / session.config.speed_multiplier,
            );
            tokio::time::sleep(adjusted_duration).await;
        }

        // Replay using the engine
        // To avoid holding lock across await, we use a temporary engine
        // This is acceptable since engine state is mostly internal statistics
        let mut temp_engine = HookReplayEngine::new();
        let result = temp_engine
            .replay_execution(hook, execution, options)
            .await?;

        // Update the shared engine statistics
        {
            let _engine = self.replay_engine.write();
            let (_temp_total, _temp_success, _temp_failed, _temp_duration) =
                temp_engine.get_statistics();
            // Note: This is a simplified approach. In production, you'd want to merge statistics properly
            // For now, we skip merging to avoid complex state management while preserving the lock
        }

        // Capture state if configured
        if session.config.capture_states {
            let context = hook.deserialize_context(&execution.hook_context)?;
            let captured = CapturedState {
                timestamp: SystemTime::now(),
                execution_id: execution.execution_id,
                hook_id: execution.hook_id.clone(),
                context_snapshot: serde_json::to_value(&context)?,
                result: result.clone(),
                metadata: execution
                    .metadata
                    .iter()
                    .map(|(k, v)| (k.clone(), v.to_string()))
                    .collect(),
            };

            session.captured_states.push_back(captured);

            // Limit memory usage
            while session.captured_states.len() > 1000 {
                session.captured_states.pop_front();
            }
        }

        debug!(
            "Replayed hook {} in {:?}",
            execution.hook_id,
            start.elapsed()
        );

        Ok(result)
    }

    /// Build timeline from executions
    pub async fn build_timeline(&self, correlation_id: Uuid) -> Result<ReplayTimeline> {
        let executions = self
            .storage_backend
            .load_executions_by_correlation(&correlation_id)
            .await?;

        if executions.is_empty() {
            return Ok(ReplayTimeline {
                entries: Vec::new(),
                total_duration: Duration::from_secs(0),
                component_interactions: HashMap::new(),
            });
        }

        let mut entries = Vec::new();
        let mut component_interactions: HashMap<String, Vec<String>> = HashMap::new();
        let start_time = executions.first().unwrap().timestamp;
        let end_time = executions.last().unwrap().timestamp;

        for execution in executions {
            // Deserialize context to get component info
            let hook = self.hook_registry.read().get(&execution.hook_id).cloned();

            if let Some(hook) = hook {
                if let Ok(context) = hook.deserialize_context(&execution.hook_context) {
                    let component_id = format!(
                        "{:?}:{}",
                        context.component_id.component_type, context.component_id.name
                    );

                    let entry = TimelineEntry {
                        timestamp: execution.timestamp,
                        relative_time: execution
                            .timestamp
                            .duration_since(start_time)
                            .unwrap_or(Duration::from_secs(0)),
                        execution_id: execution.execution_id,
                        hook_id: execution.hook_id.clone(),
                        hook_point: context.point,
                        component_id: component_id.clone(),
                        duration: execution.duration,
                        result_type: execution
                            .result
                            .split("::")
                            .last()
                            .unwrap_or("Unknown")
                            .to_string(),
                    };

                    entries.push(entry);

                    // Track component interactions
                    component_interactions
                        .entry(component_id)
                        .or_default()
                        .push(execution.hook_id.clone());
                }
            }
        }

        let total_duration = end_time
            .duration_since(start_time)
            .unwrap_or(Duration::from_secs(0));

        Ok(ReplayTimeline {
            entries,
            total_duration,
            component_interactions,
        })
    }

    /// Check if should break at this execution
    fn should_break(&self, session: &ReplaySession, execution: &SerializedHookExecution) -> bool {
        for breakpoint in &session.breakpoints {
            match &breakpoint.condition {
                BreakpointCondition::HookId(id) => {
                    if &execution.hook_id == id {
                        return true;
                    }
                }
                BreakpointCondition::ExecutionId(id) => {
                    if &execution.execution_id == id {
                        return true;
                    }
                }
                _ => {}
            }
        }
        false
    }

    /// Handle breakpoint action
    async fn handle_breakpoint(
        &self,
        _session: &mut ReplaySession,
        execution: &SerializedHookExecution,
    ) -> Result<()> {
        info!(
            "Breakpoint hit at execution {} ({})",
            execution.execution_id, execution.hook_id
        );

        // In a real implementation, this would pause and wait for user input
        // For now, we just log
        Ok(())
    }

    /// Add breakpoint to session
    pub fn add_breakpoint(&self, session_name: &str, breakpoint: ReplayBreakpoint) -> Result<()> {
        let mut sessions = self.active_sessions.write();
        let session = sessions
            .get_mut(session_name)
            .ok_or_else(|| anyhow::anyhow!("Session not found: {}", session_name))?;

        session.breakpoints.push(breakpoint);
        Ok(())
    }

    /// Get replay session
    pub fn get_session(&self, session_name: &str) -> Option<ReplaySession> {
        self.active_sessions.read().get(session_name).cloned()
    }

    /// End replay session
    pub fn end_session(&self, session_name: &str) -> Result<ReplaySession> {
        self.active_sessions
            .write()
            .remove(session_name)
            .ok_or_else(|| anyhow::anyhow!("Session not found: {}", session_name))
    }
}

impl Clone for ReplaySession {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            start_time: self.start_time,
            executions_replayed: self.executions_replayed,
            errors_encountered: self.errors_encountered.clone(),
            captured_states: self.captured_states.clone(),
            breakpoints: self.breakpoints.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replay_session_config() {
        let config = ReplaySessionConfig::default();
        assert!(config.capture_states);
        assert!(config.validate_outputs);
        assert_eq!(config.speed_multiplier, 1.0);
        assert!(config.break_on_error);
    }

    #[test]
    fn test_breakpoint_conditions() {
        let bp1 = ReplayBreakpoint {
            condition: BreakpointCondition::HookId("test_hook".to_string()),
            action: BreakpointAction::Pause,
        };

        let _bp2 = ReplayBreakpoint {
            condition: BreakpointCondition::ErrorOccurred,
            action: BreakpointAction::Log("Error occurred".to_string()),
        };

        // Verify serialization works
        let serialized = serde_json::to_string(&bp1).unwrap();
        let deserialized: ReplayBreakpoint = serde_json::from_str(&serialized).unwrap();
        match deserialized.condition {
            BreakpointCondition::HookId(id) => assert_eq!(id, "test_hook"),
            _ => panic!("Wrong condition type"),
        }
    }
}
