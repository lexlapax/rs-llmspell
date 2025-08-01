// ABOUTME: Hook execution inspector for debugging and analysis
// ABOUTME: Provides tools for examining hook execution history and patterns

use crate::persistence::{SerializedHookExecution, StorageBackend};
use crate::types::HookPoint;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use uuid::Uuid;

/// Inspection query for filtering executions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionQuery {
    /// Filter by correlation ID
    pub correlation_id: Option<Uuid>,
    /// Filter by hook IDs
    pub hook_ids: Option<Vec<String>>,
    /// Filter by hook points
    pub hook_points: Option<Vec<HookPoint>>,
    /// Filter by time range
    pub time_range: Option<TimeRange>,
    /// Filter by component ID pattern
    pub component_pattern: Option<String>,
    /// Filter by result type
    pub result_type: Option<ResultTypeFilter>,
    /// Maximum results to return
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: SystemTime,
    pub end: SystemTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResultTypeFilter {
    Continue,
    Modified,
    Cancelled,
    Replaced,
    Skipped,
    Any,
}

/// Execution analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionAnalysis {
    pub total_executions: usize,
    pub unique_hooks: HashSet<String>,
    pub execution_by_hook: HashMap<String, usize>,
    pub execution_by_point: HashMap<HookPoint, usize>,
    pub average_duration: Duration,
    pub slowest_hooks: Vec<(String, Duration)>,
    pub error_rate: f64,
    pub modification_rate: f64,
    pub cancellation_rate: f64,
    pub time_distribution: TimeDistribution,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeDistribution {
    pub hourly: HashMap<u32, usize>,
    pub daily: HashMap<u32, usize>,
    pub peak_hour: Option<u32>,
    pub peak_day: Option<u32>,
}

/// Pattern detection results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionPattern {
    pub pattern_type: PatternType,
    pub occurrences: usize,
    pub hook_sequence: Vec<String>,
    pub average_interval: Duration,
    pub components_involved: HashSet<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PatternType {
    Sequential,
    Parallel,
    Recursive,
    ErrorCascade,
    PerformanceBottleneck,
}

/// Hook execution inspector
pub struct HookInspector {
    storage_backend: Arc<dyn StorageBackend>,
}

impl HookInspector {
    /// Create a new inspector
    pub fn new(storage_backend: Arc<dyn StorageBackend>) -> Self {
        Self { storage_backend }
    }

    /// Query executions based on filters
    pub async fn query_executions(
        &self,
        query: InspectionQuery,
    ) -> Result<Vec<SerializedHookExecution>> {
        let mut results = Vec::new();

        // Start with correlation ID if provided
        if let Some(correlation_id) = query.correlation_id {
            let executions = self
                .storage_backend
                .load_executions_by_correlation(&correlation_id)
                .await?;
            results.extend(executions);
        } else {
            // For broader queries, we'd need to implement a search method in storage
            // For now, return empty
            return Ok(Vec::new());
        }

        // Apply filters
        results = self.apply_filters(results, &query);

        // Apply limit
        if let Some(limit) = query.limit {
            results.truncate(limit);
        }

        Ok(results)
    }

    /// Analyze executions
    pub async fn analyze_executions(
        &self,
        executions: &[SerializedHookExecution],
    ) -> Result<ExecutionAnalysis> {
        let total_executions = executions.len();
        let mut unique_hooks = HashSet::new();
        let mut execution_by_hook = HashMap::new();
        let execution_by_point = HashMap::new();
        let mut total_duration = Duration::from_secs(0);
        let mut durations_by_hook: HashMap<String, Vec<Duration>> = HashMap::new();
        let mut result_counts = HashMap::new();
        let mut hourly_dist = HashMap::new();
        let mut daily_dist = HashMap::new();

        for execution in executions {
            // Track unique hooks
            unique_hooks.insert(execution.hook_id.clone());

            // Count by hook
            *execution_by_hook
                .entry(execution.hook_id.clone())
                .or_insert(0) += 1;

            // Track durations
            total_duration += execution.duration;
            durations_by_hook
                .entry(execution.hook_id.clone())
                .or_default()
                .push(execution.duration);

            // Count result types
            let result_type = execution.result.split("::").last().unwrap_or("Unknown");
            *result_counts.entry(result_type.to_string()).or_insert(0) += 1;

            // Time distribution (would need proper date parsing in real implementation)
            if let Ok(duration_since_epoch) =
                execution.timestamp.duration_since(SystemTime::UNIX_EPOCH)
            {
                let hours = (duration_since_epoch.as_secs() / 3600) % 24;
                let days = duration_since_epoch.as_secs() / 86400;
                *hourly_dist.entry(hours as u32).or_insert(0) += 1;
                *daily_dist.entry(days as u32).or_insert(0) += 1;
            }
        }

        // Calculate average duration
        let average_duration = if total_executions > 0 {
            total_duration / total_executions as u32
        } else {
            Duration::from_secs(0)
        };

        // Find slowest hooks
        let mut slowest_hooks: Vec<(String, Duration)> = durations_by_hook
            .iter()
            .map(|(hook, durations)| {
                let avg = durations.iter().sum::<Duration>() / durations.len() as u32;
                (hook.clone(), avg)
            })
            .collect();
        slowest_hooks.sort_by(|a, b| b.1.cmp(&a.1));
        slowest_hooks.truncate(10);

        // Calculate rates
        let error_rate =
            *result_counts.get("Cancel").unwrap_or(&0) as f64 / total_executions as f64;
        let modification_rate =
            *result_counts.get("Modified").unwrap_or(&0) as f64 / total_executions as f64;
        let cancellation_rate =
            *result_counts.get("Cancel").unwrap_or(&0) as f64 / total_executions as f64;

        // Find peak times
        let peak_hour = hourly_dist
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(hour, _)| *hour);
        let peak_day = daily_dist
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(day, _)| *day);

        Ok(ExecutionAnalysis {
            total_executions,
            unique_hooks,
            execution_by_hook,
            execution_by_point,
            average_duration,
            slowest_hooks,
            error_rate,
            modification_rate,
            cancellation_rate,
            time_distribution: TimeDistribution {
                hourly: hourly_dist,
                daily: daily_dist,
                peak_hour,
                peak_day,
            },
        })
    }

    /// Detect execution patterns
    pub async fn detect_patterns(
        &self,
        executions: &[SerializedHookExecution],
    ) -> Result<Vec<ExecutionPattern>> {
        let mut patterns = Vec::new();

        // Detect sequential patterns
        if let Some(pattern) = self.detect_sequential_pattern(executions) {
            patterns.push(pattern);
        }

        // Detect error cascades
        if let Some(pattern) = self.detect_error_cascade(executions) {
            patterns.push(pattern);
        }

        // Detect performance bottlenecks
        if let Some(pattern) = self.detect_bottlenecks(executions) {
            patterns.push(pattern);
        }

        Ok(patterns)
    }

    /// Compare two sets of executions
    pub async fn compare_executions(
        &self,
        set_a: &[SerializedHookExecution],
        set_b: &[SerializedHookExecution],
    ) -> Result<ComparisonResult> {
        let analysis_a = self.analyze_executions(set_a).await?;
        let analysis_b = self.analyze_executions(set_b).await?;

        let duration_change = if analysis_a.average_duration > Duration::from_secs(0) {
            (analysis_b.average_duration.as_secs_f64() / analysis_a.average_duration.as_secs_f64()
                - 1.0)
                * 100.0
        } else {
            0.0
        };

        let error_rate_change = analysis_b.error_rate - analysis_a.error_rate;

        Ok(ComparisonResult {
            total_executions_change: set_b.len() as i64 - set_a.len() as i64,
            unique_hooks_change: analysis_b.unique_hooks.len() as i64
                - analysis_a.unique_hooks.len() as i64,
            average_duration_change_percent: duration_change,
            error_rate_change,
            new_hooks: analysis_b
                .unique_hooks
                .difference(&analysis_a.unique_hooks)
                .cloned()
                .collect(),
            removed_hooks: analysis_a
                .unique_hooks
                .difference(&analysis_b.unique_hooks)
                .cloned()
                .collect(),
        })
    }

    /// Apply filters to executions
    fn apply_filters(
        &self,
        mut executions: Vec<SerializedHookExecution>,
        query: &InspectionQuery,
    ) -> Vec<SerializedHookExecution> {
        // Filter by hook IDs
        if let Some(ref hook_ids) = query.hook_ids {
            executions.retain(|e| hook_ids.contains(&e.hook_id));
        }

        // Filter by time range
        if let Some(ref time_range) = query.time_range {
            executions.retain(|e| e.timestamp >= time_range.start && e.timestamp <= time_range.end);
        }

        // Filter by result type
        if let Some(ref result_type) = query.result_type {
            executions.retain(|e| match result_type {
                ResultTypeFilter::Continue => e.result.contains("Continue"),
                ResultTypeFilter::Modified => e.result.contains("Modified"),
                ResultTypeFilter::Cancelled => e.result.contains("Cancel"),
                ResultTypeFilter::Replaced => e.result.contains("Replace"),
                ResultTypeFilter::Skipped => e.result.contains("Skipped"),
                ResultTypeFilter::Any => true,
            });
        }

        executions
    }

    /// Detect sequential execution pattern
    fn detect_sequential_pattern(
        &self,
        executions: &[SerializedHookExecution],
    ) -> Option<ExecutionPattern> {
        if executions.len() < 3 {
            return None;
        }

        // Look for repeated sequences
        let mut sequences: HashMap<Vec<String>, usize> = HashMap::new();

        for window in executions.windows(3) {
            let sequence: Vec<String> = window.iter().map(|e| e.hook_id.clone()).collect();
            *sequences.entry(sequence).or_insert(0) += 1;
        }

        // Find most common sequence
        if let Some((sequence, count)) = sequences.iter().max_by_key(|(_, count)| *count) {
            if *count > 1 {
                return Some(ExecutionPattern {
                    pattern_type: PatternType::Sequential,
                    occurrences: *count,
                    hook_sequence: sequence.clone(),
                    average_interval: Duration::from_secs(0), // Would calculate actual interval
                    components_involved: HashSet::new(),
                });
            }
        }

        None
    }

    /// Detect error cascade pattern
    fn detect_error_cascade(
        &self,
        executions: &[SerializedHookExecution],
    ) -> Option<ExecutionPattern> {
        let mut error_sequences = Vec::new();
        let mut current_sequence = Vec::new();

        for execution in executions {
            if execution.result.contains("Cancel") || execution.result.contains("Error") {
                current_sequence.push(execution.hook_id.clone());
            } else if !current_sequence.is_empty() {
                if current_sequence.len() > 2 {
                    error_sequences.push(current_sequence.clone());
                }
                current_sequence.clear();
            }
        }

        if !error_sequences.is_empty() {
            let longest = error_sequences.iter().max_by_key(|s| s.len()).unwrap();
            return Some(ExecutionPattern {
                pattern_type: PatternType::ErrorCascade,
                occurrences: error_sequences.len(),
                hook_sequence: longest.clone(),
                average_interval: Duration::from_secs(0),
                components_involved: HashSet::new(),
            });
        }

        None
    }

    /// Detect performance bottlenecks
    fn detect_bottlenecks(
        &self,
        executions: &[SerializedHookExecution],
    ) -> Option<ExecutionPattern> {
        // Find hooks that consistently take longer than average
        let avg_duration =
            executions.iter().map(|e| e.duration).sum::<Duration>() / executions.len() as u32;
        let threshold = avg_duration * 3; // 3x average is considered a bottleneck

        let slow_hooks: Vec<String> = executions
            .iter()
            .filter(|e| e.duration > threshold)
            .map(|e| e.hook_id.clone())
            .collect();

        if !slow_hooks.is_empty() {
            return Some(ExecutionPattern {
                pattern_type: PatternType::PerformanceBottleneck,
                occurrences: slow_hooks.len(),
                hook_sequence: slow_hooks,
                average_interval: Duration::from_secs(0),
                components_involved: HashSet::new(),
            });
        }

        None
    }
}

/// Comparison result between two execution sets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonResult {
    pub total_executions_change: i64,
    pub unique_hooks_change: i64,
    pub average_duration_change_percent: f64,
    pub error_rate_change: f64,
    pub new_hooks: HashSet<String>,
    pub removed_hooks: HashSet<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_inspection_query() {
        let query = InspectionQuery {
            correlation_id: Some(Uuid::new_v4()),
            hook_ids: Some(vec!["test_hook".to_string()]),
            hook_points: None,
            time_range: Some(TimeRange {
                start: SystemTime::now() - Duration::from_secs(3600),
                end: SystemTime::now(),
            }),
            component_pattern: None,
            result_type: Some(ResultTypeFilter::Modified),
            limit: Some(100),
        };

        // Verify serialization
        let serialized = serde_json::to_string(&query).unwrap();
        let deserialized: InspectionQuery = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.limit, Some(100));
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_pattern_detection() {
        // Test pattern type serialization
        let pattern = ExecutionPattern {
            pattern_type: PatternType::Sequential,
            occurrences: 5,
            hook_sequence: vec!["hook1".to_string(), "hook2".to_string()],
            average_interval: Duration::from_secs(10),
            components_involved: HashSet::new(),
        };

        let serialized = serde_json::to_string(&pattern).unwrap();
        let deserialized: ExecutionPattern = serde_json::from_str(&serialized).unwrap();
        assert_eq!(deserialized.occurrences, 5);
    }
}
