// ABOUTME: Hook replay management module providing advanced replay capabilities
// ABOUTME: Includes scheduling, parameter modification, and result comparison features

pub mod comparator;
pub mod manager;
pub mod scheduler;

use crate::persistence::SerializedHookExecution;
use crate::result::HookResult;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, SystemTime};
use uuid::Uuid;

// Re-export main types
pub use comparator::{ComparisonResult, HookResultComparator};
pub use manager::{ReplayManager, ReplayRequest, ReplayResponse, ReplayState};
pub use scheduler::{ReplaySchedule, ReplayScheduler, ScheduledReplay};

/// Replay execution mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplayMode {
    /// Execute exactly as recorded
    Exact,
    /// Allow parameter modifications
    Modified,
    /// Simulate execution without running hooks
    Simulate,
    /// Execute with debugging enabled
    Debug,
}

/// Parameter modification for replay
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterModification {
    /// Path to the parameter (e.g., "context.data.input")
    pub path: String,
    /// New value for the parameter
    pub value: serde_json::Value,
    /// Whether to apply this modification
    pub enabled: bool,
}

/// Replay configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayConfig {
    /// Execution mode
    pub mode: ReplayMode,
    /// Parameter modifications to apply
    pub modifications: Vec<ParameterModification>,
    /// Whether to compare results with original
    pub compare_results: bool,
    /// Timeout for each hook execution
    pub timeout: Duration,
    /// Whether to stop on first error
    pub stop_on_error: bool,
    /// Tags for categorizing replays
    pub tags: Vec<String>,
}

impl Default for ReplayConfig {
    fn default() -> Self {
        Self {
            mode: ReplayMode::Exact,
            modifications: Vec::new(),
            compare_results: true,
            timeout: Duration::from_secs(30),
            stop_on_error: true,
            tags: Vec::new(),
        }
    }
}

/// Replay execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayResult {
    /// Unique replay ID
    pub replay_id: Uuid,
    /// Original execution ID
    pub original_execution_id: Uuid,
    /// Hook name that was replayed
    pub hook_name: String,
    /// Start time of replay
    pub start_time: SystemTime,
    /// Duration of replay
    pub duration: Duration,
    /// Result from hook execution
    pub hook_result: Result<HookResult, String>,
    /// Comparison with original if requested
    pub comparison: Option<ComparisonResult>,
    /// Applied modifications
    pub applied_modifications: Vec<ParameterModification>,
    /// Execution metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Batch replay request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchReplayRequest {
    /// Executions to replay
    pub executions: Vec<SerializedHookExecution>,
    /// Common configuration for all replays
    pub config: ReplayConfig,
    /// Whether to run in parallel
    pub parallel: bool,
    /// Maximum concurrent replays if parallel
    pub max_concurrent: usize,
}

/// Batch replay response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchReplayResponse {
    /// Individual replay results
    pub results: Vec<ReplayResult>,
    /// Total duration
    pub total_duration: Duration,
    /// Success count
    pub success_count: usize,
    /// Failure count
    pub failure_count: usize,
    /// Batch metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_replay_config_default() {
        let config = ReplayConfig::default();
        assert_eq!(config.mode, ReplayMode::Exact);
        assert!(config.modifications.is_empty());
        assert!(config.compare_results);
        assert_eq!(config.timeout, Duration::from_secs(30));
    }

    #[test]
    fn test_parameter_modification() {
        let modification = ParameterModification {
            path: "context.data.value".to_string(),
            value: serde_json::json!(42),
            enabled: true,
        };

        assert_eq!(modification.path, "context.data.value");
        assert_eq!(modification.value, serde_json::json!(42));
        assert!(modification.enabled);
    }
}
