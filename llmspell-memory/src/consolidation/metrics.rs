//! Consolidation metrics and monitoring
//!
//! Comprehensive metrics for consolidation performance, prompt effectiveness,
//! cost tracking, and consolidation lag. Integrates with llmspell-core observability.
//!
//! # Metrics Categories
//!
//! - **Core Metrics**: `entries_processed`, `decisions_by_type`, `latency_p95`
//! - **Prompt Performance**: DMR, parse success rate per `PromptVersion`
//! - **Cost Tracking**: tokens used, LLM cost by model
//! - **Consolidation Lag**: P50/P95/P99 time from episodic add → processed
//!
//! # Example
//!
//! ```rust,ignore
//! use llmspell_memory::consolidation::ConsolidationMetrics;
//!
//! let metrics = ConsolidationMetrics::new();
//! metrics.record_consolidation(result, duration).await;
//! let snapshot = metrics.snapshot().await;
//! println!("DMR: {:.2}%", snapshot.dmr.unwrap_or(0.0) * 100.0);
//! ```

use crate::consolidation::DecisionPayload;
use crate::types::ConsolidationResult;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Decision type for metrics tracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DecisionType {
    /// Add new entity
    Add,
    /// Update existing entity
    Update,
    /// Delete entity (tombstone)
    Delete,
    /// No operation
    Noop,
}

impl From<&DecisionPayload> for DecisionType {
    fn from(decision: &DecisionPayload) -> Self {
        match decision {
            DecisionPayload::Add { .. } => Self::Add,
            DecisionPayload::Update { .. } => Self::Update,
            DecisionPayload::Delete { .. } => Self::Delete,
            DecisionPayload::Noop => Self::Noop,
        }
    }
}

/// Decision distribution metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DecisionDistribution {
    /// Number of ADD decisions
    pub add_count: u64,
    /// Number of UPDATE decisions
    pub update_count: u64,
    /// Number of DELETE decisions
    pub delete_count: u64,
    /// Number of NOOP decisions
    pub noop_count: u64,
}

impl DecisionDistribution {
    /// Total decisions
    #[must_use]
    pub const fn total(&self) -> u64 {
        self.add_count + self.update_count + self.delete_count + self.noop_count
    }

    /// ADD percentage
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn add_percentage(&self) -> f64 {
        let total = self.total();
        if total == 0 {
            0.0
        } else {
            (self.add_count as f64 / total as f64) * 100.0
        }
    }

    /// UPDATE percentage
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn update_percentage(&self) -> f64 {
        let total = self.total();
        if total == 0 {
            0.0
        } else {
            (self.update_count as f64 / total as f64) * 100.0
        }
    }

    /// DELETE percentage
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn delete_percentage(&self) -> f64 {
        let total = self.total();
        if total == 0 {
            0.0
        } else {
            (self.delete_count as f64 / total as f64) * 100.0
        }
    }

    /// NOOP percentage
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn noop_percentage(&self) -> f64 {
        let total = self.total();
        if total == 0 {
            0.0
        } else {
            (self.noop_count as f64 / total as f64) * 100.0
        }
    }
}

/// Latency statistics with percentiles
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LatencyStats {
    /// Number of observations
    pub count: u64,
    /// Sum of all latencies (for average)
    pub sum_ms: f64,
    /// Minimum latency
    pub min_ms: f64,
    /// Maximum latency
    pub max_ms: f64,
    /// P50 (median)
    pub p50_ms: f64,
    /// P95
    pub p95_ms: f64,
    /// P99
    pub p99_ms: f64,
}

impl LatencyStats {
    /// Average latency
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn avg_ms(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.sum_ms / self.count as f64
        }
    }
}

/// Core consolidation metrics
///
/// Tracks entries processed, decision distribution, and latency statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CoreMetrics {
    /// Total entries processed
    pub entries_processed: u64,
    /// Decision distribution (ADD/UPDATE/DELETE/NOOP)
    pub decision_distribution: DecisionDistribution,
    /// Latency statistics (P50/P95/P99)
    pub latency: LatencyStats,
    /// Number of consolidation runs
    pub consolidations: u64,
    /// Number of parse failures (fell back to natural language)
    pub parse_failures: u64,
    /// Number of validation errors
    pub validation_errors: u64,
}

/// Consolidation metrics collector
///
/// Thread-safe metrics aggregation for consolidation performance.
pub struct ConsolidationMetrics {
    core: Arc<RwLock<CoreMetrics>>,
    latencies: Arc<RwLock<Vec<f64>>>, // For percentile calculation
}

impl Default for ConsolidationMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl ConsolidationMetrics {
    /// Create new metrics collector
    #[must_use]
    pub fn new() -> Self {
        Self {
            core: Arc::new(RwLock::new(CoreMetrics::default())),
            latencies: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Record a consolidation result
    ///
    /// Updates entries processed, decision distribution, and latency stats.
    ///
    /// # Panics
    ///
    /// Panics if latency values contain NaN (not expected in normal operation).
    pub async fn record_consolidation(
        &self,
        result: &ConsolidationResult,
        decisions: &[DecisionPayload],
        duration: Duration,
    ) {
        let duration_ms = duration.as_secs_f64() * 1000.0;

        // Update core metrics and push latency
        let mut sorted = {
            let mut core = self.core.write().await;

            // Update core metrics
            core.entries_processed += result.entries_processed as u64;
            core.consolidations += 1;

            // Update decision distribution
            for decision in decisions {
                match DecisionType::from(decision) {
                    DecisionType::Add => core.decision_distribution.add_count += 1,
                    DecisionType::Update => core.decision_distribution.update_count += 1,
                    DecisionType::Delete => core.decision_distribution.delete_count += 1,
                    DecisionType::Noop => core.decision_distribution.noop_count += 1,
                }
            }

            // Track latency and clone
            let cloned = {
                let mut latencies = self.latencies.write().await;
                latencies.push(duration_ms);
                latencies.clone()
            }; // Latencies lock dropped here

            cloned
        }; // Core lock dropped here

        // Sort for percentile calculation (outside locks)
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        // Update latency stats (outside lock to minimize hold time)
        #[allow(clippy::cast_precision_loss)]
        if !sorted.is_empty() {
            let mut core = self.core.write().await;

            core.latency.count = sorted.len() as u64;
            core.latency.sum_ms = sorted.iter().sum();
            core.latency.min_ms = sorted[0];
            core.latency.max_ms = sorted[sorted.len() - 1];

            // Calculate percentiles
            core.latency.p50_ms = percentile(&sorted, 50.0);
            core.latency.p95_ms = percentile(&sorted, 95.0);
            core.latency.p99_ms = percentile(&sorted, 99.0);
        }
    }

    /// Record a parse failure
    pub async fn record_parse_failure(&self) {
        self.core.write().await.parse_failures += 1;
    }

    /// Record a validation error
    pub async fn record_validation_error(&self) {
        self.core.write().await.validation_errors += 1;
    }

    /// Get current metrics snapshot
    pub async fn snapshot(&self) -> CoreMetrics {
        self.core.read().await.clone()
    }

    /// Reset all metrics
    pub async fn reset(&self) {
        *self.core.write().await = CoreMetrics::default();
        self.latencies.write().await.clear();
    }
}

/// Calculate percentile from sorted values using linear interpolation
#[allow(clippy::cast_precision_loss)]
#[allow(clippy::cast_possible_truncation)]
#[allow(clippy::cast_sign_loss)]
fn percentile(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }

    if sorted.len() == 1 {
        return sorted[0];
    }

    let rank = (p / 100.0) * (sorted.len() - 1) as f64;
    let lower_idx = rank.floor() as usize;
    let upper_idx = rank.ceil() as usize;

    if lower_idx == upper_idx {
        sorted[lower_idx]
    } else {
        // Linear interpolation
        let lower = sorted[lower_idx];
        let upper = sorted[upper_idx];
        let fraction = rank - lower_idx as f64;
        lower + fraction * (upper - lower)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decision_distribution() {
        let mut dist = DecisionDistribution::default();
        dist.add_count = 40;
        dist.update_count = 30;
        dist.delete_count = 10;
        dist.noop_count = 20;

        assert_eq!(dist.total(), 100);
        assert!((dist.add_percentage() - 40.0).abs() < 0.01);
        assert!((dist.update_percentage() - 30.0).abs() < 0.01);
        assert!((dist.delete_percentage() - 10.0).abs() < 0.01);
        assert!((dist.noop_percentage() - 20.0).abs() < 0.01);
    }

    #[test]
    fn test_decision_type_from_payload() {
        use std::collections::HashMap;

        let add = DecisionPayload::Add {
            entity_id: "test".to_string(),
        };
        assert_eq!(DecisionType::from(&add), DecisionType::Add);

        let update = DecisionPayload::Update {
            entity_id: "test".to_string(),
            changes: HashMap::new(),
        };
        assert_eq!(DecisionType::from(&update), DecisionType::Update);

        let delete = DecisionPayload::Delete {
            entity_id: "test".to_string(),
        };
        assert_eq!(DecisionType::from(&delete), DecisionType::Delete);

        let noop = DecisionPayload::Noop;
        assert_eq!(DecisionType::from(&noop), DecisionType::Noop);
    }

    #[test]
    fn test_latency_stats() {
        let stats = LatencyStats {
            count: 5,
            sum_ms: 500.0,
            min_ms: 50.0,
            max_ms: 150.0,
            p50_ms: 100.0,
            p95_ms: 145.0,
            p99_ms: 149.0,
        };

        assert!((stats.avg_ms() - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_percentile_calculation() {
        let values = vec![10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0, 100.0];

        // P50 with 10 values: rank = 0.5 * 9 = 4.5, interpolate between index 4 (50) and 5 (60)
        assert!((percentile(&values, 50.0) - 55.0).abs() < 0.01);

        // P95 with 10 values: rank = 0.95 * 9 = 8.55, interpolate between index 8 (90) and 9 (100)
        assert!((percentile(&values, 95.0) - 95.5).abs() < 0.01);

        // P99 with 10 values: rank = 0.99 * 9 = 8.91, interpolate between index 8 (90) and 9 (100)
        assert!((percentile(&values, 99.0) - 99.1).abs() < 0.01);

        // Edge cases
        assert_eq!(percentile(&values, 0.0), 10.0);
        assert_eq!(percentile(&values, 100.0), 100.0);
    }

    #[tokio::test]
    async fn test_metrics_recording() {
        use std::collections::HashMap;

        let metrics = ConsolidationMetrics::new();

        let result = ConsolidationResult {
            entries_processed: 10,
            entities_added: 5,
            entities_updated: 3,
            entities_deleted: 1,
            entries_skipped: 1,
            duration_ms: 100,
        };

        let decisions = vec![
            DecisionPayload::Add {
                entity_id: "e1".to_string(),
            },
            DecisionPayload::Update {
                entity_id: "e2".to_string(),
                changes: HashMap::new(),
            },
            DecisionPayload::Delete {
                entity_id: "e3".to_string(),
            },
            DecisionPayload::Noop,
        ];

        metrics
            .record_consolidation(&result, &decisions, Duration::from_millis(100))
            .await;

        let snapshot = metrics.snapshot().await;
        assert_eq!(snapshot.entries_processed, 10);
        assert_eq!(snapshot.consolidations, 1);
        assert_eq!(snapshot.decision_distribution.add_count, 1);
        assert_eq!(snapshot.decision_distribution.update_count, 1);
        assert_eq!(snapshot.decision_distribution.delete_count, 1);
        assert_eq!(snapshot.decision_distribution.noop_count, 1);
        assert!(snapshot.latency.avg_ms() > 0.0);
    }

    #[tokio::test]
    async fn test_parse_failure_tracking() {
        let metrics = ConsolidationMetrics::new();

        metrics.record_parse_failure().await;
        metrics.record_parse_failure().await;

        let snapshot = metrics.snapshot().await;
        assert_eq!(snapshot.parse_failures, 2);
    }

    #[tokio::test]
    async fn test_validation_error_tracking() {
        let metrics = ConsolidationMetrics::new();

        metrics.record_validation_error().await;

        let snapshot = metrics.snapshot().await;
        assert_eq!(snapshot.validation_errors, 1);
    }

    #[tokio::test]
    async fn test_metrics_reset() {
        use std::collections::HashMap;

        let metrics = ConsolidationMetrics::new();

        let result = ConsolidationResult {
            entries_processed: 5,
            entities_added: 2,
            entities_updated: 1,
            entities_deleted: 0,
            entries_skipped: 2,
            duration_ms: 50,
        };

        metrics
            .record_consolidation(&result, &[], Duration::from_millis(50))
            .await;

        metrics.reset().await;

        let snapshot = metrics.snapshot().await;
        assert_eq!(snapshot.entries_processed, 0);
        assert_eq!(snapshot.consolidations, 0);
    }
}
