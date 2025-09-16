//! ABOUTME: Session-specific debugging helpers that integrate with existing replay debugging features
//! ABOUTME: Provides state inspection, timeline navigation, and comparison tools for session replay

use crate::sessions::{Result, SessionError, SessionId};
use llmspell_hooks::persistence::{
    CapturedState, ReplayError, ReplayErrorType, ReplaySession, TimelineEntry,
};
use llmspell_hooks::replay::{ComparisonResult, HookResultComparator};
use llmspell_state_persistence::manager::SerializedHookExecution;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
use tracing::info;
use uuid::Uuid;

/// Session debugging helper that integrates existing replay debugging features
pub struct SessionDebugger {
    /// Hook result comparator from llmspell-hooks
    comparator: Arc<HookResultComparator>,
    /// Captured states during replay (from existing `ReplaySession`)
    captured_states: Arc<RwLock<HashMap<SessionId, VecDeque<CapturedState>>>>,
    /// Timeline entries for sessions
    timelines: Arc<RwLock<HashMap<SessionId, Vec<TimelineEntry>>>>,
    /// Session errors
    session_errors: Arc<RwLock<HashMap<SessionId, Vec<ReplayError>>>>,
}

impl SessionDebugger {
    /// Create a new session debugger
    pub fn new() -> Self {
        Self {
            comparator: Arc::new(HookResultComparator::new()),
            captured_states: Arc::new(RwLock::new(HashMap::new())),
            timelines: Arc::new(RwLock::new(HashMap::new())),
            session_errors: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Import captured states from a replay session
    ///
    /// # Panics
    ///
    /// Panics if the captured states or session errors mutex is poisoned
    pub fn import_replay_session(&self, session_id: SessionId, replay_session: &ReplaySession) {
        // Import captured states
        {
            let mut states = self.captured_states.write().unwrap();
            states.insert(session_id, replay_session.captured_states.clone());
        }

        // Import errors
        {
            let mut errors = self.session_errors.write().unwrap();
            errors.insert(session_id, replay_session.errors_encountered.clone());
        }

        info!(
            "Imported debug data for session {}: {} states, {} errors",
            session_id,
            replay_session.captured_states.len(),
            replay_session.errors_encountered.len()
        );
    }

    /// Inspect state at a specific point in time
    ///
    /// # Panics
    ///
    /// Panics if the captured states mutex is poisoned
    pub fn inspect_state_at(
        &self,
        session_id: &SessionId,
        timestamp: SystemTime,
    ) -> Result<Option<SessionState>> {
        let states = self.captured_states.read().unwrap();
        let session_states = states
            .get(session_id)
            .ok_or_else(|| SessionError::general("No debug data for session"))?;

        // Find the state closest to the requested timestamp
        let state = session_states
            .iter()
            .filter(|s| s.timestamp <= timestamp)
            .max_by_key(|s| s.timestamp);

        Ok(state.map(SessionState::from_captured))
    }

    /// Get all captured states for a session
    ///
    /// # Panics
    ///
    /// Panics if the captured states mutex is poisoned
    pub fn get_all_states(&self, session_id: &SessionId) -> Result<Vec<SessionState>> {
        let states = self.captured_states.read().unwrap();
        let session_states = states
            .get(session_id)
            .ok_or_else(|| SessionError::general("No debug data for session"))?;

        Ok(session_states
            .iter()
            .map(SessionState::from_captured)
            .collect())
    }

    /// Compare states at two different points in time
    pub fn compare_states(
        &self,
        session_id: &SessionId,
        timestamp1: SystemTime,
        timestamp2: SystemTime,
    ) -> Result<StateComparison> {
        let state1 = self
            .inspect_state_at(session_id, timestamp1)?
            .ok_or_else(|| SessionError::general("No state found at timestamp1"))?;
        let state2 = self
            .inspect_state_at(session_id, timestamp2)?
            .ok_or_else(|| SessionError::general("No state found at timestamp2"))?;

        Ok(StateComparison::compare(&state1, &state2))
    }

    /// Update timeline for a session
    ///
    /// # Panics
    ///
    /// Panics if the timelines mutex is poisoned
    pub fn update_timeline(&self, session_id: SessionId, executions: Vec<SerializedHookExecution>) {
        let mut timelines = self.timelines.write().unwrap();
        let timeline: Vec<TimelineEntry> = executions
            .into_iter()
            .map(|exec| TimelineEntry {
                timestamp: exec.timestamp,
                relative_time: Duration::from_secs(0), // Will be calculated from first timestamp
                execution_id: exec.execution_id,
                hook_id: exec.hook_id.clone(),
                hook_point: llmspell_hooks::HookPoint::BeforeAgentExecution, // Default hook point
                component_id: "session".to_string(),                         // Session component
                duration: exec.duration,
                result_type: exec.result.clone(),
            })
            .collect();

        timelines.insert(session_id, timeline);
    }

    /// Get timeline for a session
    ///
    /// # Panics
    ///
    /// Panics if the timelines mutex is poisoned
    pub fn get_timeline(&self, session_id: &SessionId) -> Option<Vec<TimelineEntry>> {
        self.timelines.read().unwrap().get(session_id).cloned()
    }

    /// Navigate to a specific point in the timeline
    pub fn navigate_to_timeline_point(
        &self,
        session_id: &SessionId,
        entry_index: usize,
    ) -> Result<SessionState> {
        let timeline = self
            .get_timeline(session_id)
            .ok_or_else(|| SessionError::general("No timeline for session"))?;

        let entry = timeline
            .get(entry_index)
            .ok_or_else(|| SessionError::general("Invalid timeline index"))?;

        self.inspect_state_at(session_id, entry.timestamp)?
            .ok_or_else(|| SessionError::general("No state at timeline point"))
    }

    /// Compare hook results between original and replay
    pub fn compare_hook_results(
        &self,
        original: &llmspell_hooks::result::HookResult,
        replayed: &llmspell_hooks::result::HookResult,
    ) -> ComparisonResult {
        self.comparator.compare(original, replayed)
    }

    /// Analyze errors for a session
    ///
    /// # Panics
    ///
    /// Panics if the session errors mutex is poisoned
    pub fn analyze_errors(&self, session_id: &SessionId) -> ErrorAnalysis {
        let errors = self.session_errors.read().unwrap();
        let session_errors = errors.get(session_id).cloned().unwrap_or_default();

        ErrorAnalysis::from_errors(&session_errors)
    }

    /// Add an error to the session
    ///
    /// # Panics
    ///
    /// Panics if the session errors mutex is poisoned
    pub fn add_error(&self, session_id: SessionId, error: ReplayError) {
        let mut errors = self.session_errors.write().unwrap();
        errors.entry(session_id).or_default().push(error);
    }

    /// Export debug data for a session
    pub fn export_debug_data(&self, session_id: &SessionId) -> Result<SessionDebugData> {
        let states = self.get_all_states(session_id)?;
        let timeline = self.get_timeline(session_id);
        let error_analysis = self.analyze_errors(session_id);

        Ok(SessionDebugData {
            session_id: *session_id,
            captured_states: states,
            timeline,
            error_analysis,
            export_time: SystemTime::now(),
        })
    }

    /// Clear debug data for a session
    ///
    /// # Panics
    ///
    /// Panics if any of the mutexes are poisoned
    pub fn clear_session_data(&self, session_id: &SessionId) {
        {
            let mut states = self.captured_states.write().unwrap();
            states.remove(session_id);
        }
        {
            let mut timelines = self.timelines.write().unwrap();
            timelines.remove(session_id);
        }
        {
            let mut errors = self.session_errors.write().unwrap();
            errors.remove(session_id);
        }
    }
}

/// Session state at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    /// When this state was captured
    pub timestamp: SystemTime,
    /// Hook execution ID
    pub execution_id: Uuid,
    /// Hook that was executed
    pub hook_id: String,
    /// Context snapshot
    pub context: serde_json::Value,
    /// Result of the hook execution
    pub result: String,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

impl SessionState {
    /// Create from a captured state
    pub fn from_captured(captured: &CapturedState) -> Self {
        Self {
            timestamp: captured.timestamp,
            execution_id: captured.execution_id,
            hook_id: captured.hook_id.clone(),
            context: captured.context_snapshot.clone(),
            result: format!("{:?}", captured.result),
            metadata: captured.metadata.clone(),
        }
    }
}

/// A specific difference found during comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Difference {
    /// Path to the difference
    pub path: String,
    /// Original value
    pub original: Option<serde_json::Value>,
    /// Replayed value
    pub replayed: Option<serde_json::Value>,
    /// Description of the difference
    pub description: String,
}

/// Comparison between two session states
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateComparison {
    /// First state timestamp
    pub timestamp1: SystemTime,
    /// Second state timestamp
    pub timestamp2: SystemTime,
    /// Context differences
    pub context_diffs: Vec<Difference>,
    /// Result differences
    pub result_diffs: Vec<Difference>,
    /// Metadata differences
    pub metadata_diffs: Vec<(String, Option<String>, Option<String>)>,
    /// Summary of changes
    pub summary: String,
}

impl StateComparison {
    /// Compare two session states
    pub fn compare(state1: &SessionState, state2: &SessionState) -> Self {
        let context_diffs = Self::compare_json(&state1.context, &state2.context);
        let result_diffs = if state1.result == state2.result {
            vec![]
        } else {
            vec![Difference {
                path: "result".to_string(),
                original: Some(serde_json::Value::String(state1.result.clone())),
                replayed: Some(serde_json::Value::String(state2.result.clone())),
                description: "Hook result changed".to_string(),
            }]
        };

        let metadata_diffs = Self::compare_metadata(&state1.metadata, &state2.metadata);

        let summary = format!(
            "Found {} context differences, {} result differences, {} metadata differences",
            context_diffs.len(),
            result_diffs.len(),
            metadata_diffs.len()
        );

        Self {
            timestamp1: state1.timestamp,
            timestamp2: state2.timestamp,
            context_diffs,
            result_diffs,
            metadata_diffs,
            summary,
        }
    }

    fn compare_json(val1: &serde_json::Value, val2: &serde_json::Value) -> Vec<Difference> {
        let mut diffs = Vec::new();
        Self::compare_json_recursive(val1, val2, "", &mut diffs);
        diffs
    }

    fn compare_json_recursive(
        val1: &serde_json::Value,
        val2: &serde_json::Value,
        path: &str,
        diffs: &mut Vec<Difference>,
    ) {
        match (val1, val2) {
            (serde_json::Value::Object(map1), serde_json::Value::Object(map2)) => {
                let all_keys: std::collections::HashSet<_> =
                    map1.keys().chain(map2.keys()).collect();
                for key in all_keys {
                    let new_path = if path.is_empty() {
                        key.to_string()
                    } else {
                        format!("{}.{}", path, key)
                    };
                    match (map1.get(key), map2.get(key)) {
                        (Some(v1), Some(v2)) => {
                            Self::compare_json_recursive(v1, v2, &new_path, diffs);
                        }
                        (Some(v1), None) => {
                            diffs.push(Difference {
                                path: new_path.clone(),
                                original: Some(v1.clone()),
                                replayed: None,
                                description: "Field removed".to_string(),
                            });
                        }
                        (None, Some(v2)) => {
                            diffs.push(Difference {
                                path: new_path.clone(),
                                original: None,
                                replayed: Some(v2.clone()),
                                description: "Field added".to_string(),
                            });
                        }
                        (None, None) => unreachable!(),
                    }
                }
            }
            _ => {
                if val1 != val2 {
                    diffs.push(Difference {
                        path: path.to_string(),
                        original: Some(val1.clone()),
                        replayed: Some(val2.clone()),
                        description: "Value changed".to_string(),
                    });
                }
            }
        }
    }

    fn compare_metadata(
        meta1: &HashMap<String, String>,
        meta2: &HashMap<String, String>,
    ) -> Vec<(String, Option<String>, Option<String>)> {
        let mut diffs = Vec::new();
        let all_keys: std::collections::HashSet<_> = meta1.keys().chain(meta2.keys()).collect();

        for key in all_keys {
            match (meta1.get(key), meta2.get(key)) {
                (Some(v1), Some(v2)) if v1 != v2 => {
                    diffs.push(((*key).to_string(), Some(v1.clone()), Some(v2.clone())));
                }
                (Some(v1), None) => {
                    diffs.push(((*key).to_string(), Some(v1.clone()), None));
                }
                (None, Some(v2)) => {
                    diffs.push(((*key).to_string(), None, Some(v2.clone())));
                }
                _ => {}
            }
        }

        diffs
    }
}

/// Error analysis for a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorAnalysis {
    /// Total number of errors
    pub total_errors: usize,
    /// Errors by type
    pub errors_by_type: HashMap<String, usize>,
    /// Errors by hook
    pub errors_by_hook: HashMap<String, usize>,
    /// Error timeline
    pub error_timeline: Vec<(SystemTime, String, String)>,
    /// Most common error
    pub most_common_error: Option<(String, usize)>,
    /// Error rate over time
    pub error_rate: Option<f64>,
}

impl ErrorAnalysis {
    /// Create analysis from replay errors
    ///
    /// # Panics
    ///
    /// Panics if `error_timeline` is empty when accessing `first()` or `last()`
    pub fn from_errors(errors: &[ReplayError]) -> Self {
        let total_errors = errors.len();
        let mut errors_by_type = HashMap::new();
        let mut errors_by_hook = HashMap::new();
        let mut error_timeline = Vec::new();

        for error in errors {
            // Count by type
            let type_name = match error.error_type {
                ReplayErrorType::DeserializationError => "DeserializationError",
                ReplayErrorType::ExecutionError => "ExecutionError",
                ReplayErrorType::ValidationError => "ValidationError",
                ReplayErrorType::TimeoutError => "TimeoutError",
                ReplayErrorType::ResourceError => "ResourceError",
            };
            *errors_by_type.entry(type_name.to_string()).or_insert(0) += 1;

            // Count by hook
            *errors_by_hook.entry(error.hook_id.clone()).or_insert(0) += 1;

            // Add to timeline
            error_timeline.push((
                error.timestamp,
                error.hook_id.clone(),
                error.error_message.clone(),
            ));
        }

        // Find most common error
        let most_common_error = errors_by_type
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(error_type, count)| (error_type.clone(), *count));

        // Calculate error rate if we have timeline data
        let error_rate = if error_timeline.len() >= 2 {
            let first = error_timeline.first().unwrap().0;
            let last = error_timeline.last().unwrap().0;
            if let Ok(duration) = last.duration_since(first) {
                let hours = duration.as_secs_f64() / 3600.0;
                if hours > 0.0 {
                    #[allow(clippy::cast_precision_loss)]
                    Some(total_errors as f64 / hours)
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        Self {
            total_errors,
            errors_by_type,
            errors_by_hook,
            error_timeline,
            most_common_error,
            error_rate,
        }
    }
}

/// Exported debug data for a session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionDebugData {
    /// Session ID
    pub session_id: SessionId,
    /// All captured states
    pub captured_states: Vec<SessionState>,
    /// Timeline entries
    pub timeline: Option<Vec<TimelineEntry>>,
    /// Error analysis
    pub error_analysis: ErrorAnalysis,
    /// When this data was exported
    pub export_time: SystemTime,
}

impl Default for SessionDebugger {
    fn default() -> Self {
        Self::new()
    }
}

// Include test module
#[cfg(test)]
#[path = "session_debug_tests.rs"]
mod tests;
