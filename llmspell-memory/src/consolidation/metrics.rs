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

use crate::consolidation::{DecisionPayload, PromptVersion};
use crate::types::ConsolidationResult;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::{debug, info, trace};

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

/// Prompt version selection strategy for A/B testing
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VersionSelectionStrategy {
    /// Always use specified version (no A/B testing)
    Fixed(PromptVersion),
    /// Random selection per consolidation (50/50 V1/V2)
    RandomPerConsolidation,
    /// Random selection per session (sticky within session)
    RandomPerSession,
}

impl Default for VersionSelectionStrategy {
    fn default() -> Self {
        Self::Fixed(PromptVersion::V1)
    }
}

/// Per-version prompt performance metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PromptMetrics {
    /// Total consolidations with this version
    pub consolidations: u64,
    /// Successful parses (JSON parsed correctly)
    pub parse_successes: u64,
    /// Parse failures (fell back to natural language)
    pub parse_failures: u64,
    /// Decision distribution for this version
    pub decision_distribution: DecisionDistribution,
    /// Average DMR (if ground truth available)
    pub avg_dmr: Option<f64>,
}

impl PromptMetrics {
    /// Calculate parse success rate (0.0 to 1.0)
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn parse_success_rate(&self) -> f64 {
        let total = self.consolidations;
        if total == 0 {
            0.0
        } else {
            self.parse_successes as f64 / total as f64
        }
    }
}

/// Auto-promotion configuration for prompt versions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoPromotionConfig {
    /// Minimum sample size before considering promotion
    pub min_sample_size: u64,
    /// Required parse success rate improvement (e.g., 0.05 = 5% improvement)
    pub min_parse_improvement: f64,
    /// Enable automatic promotion (vs recommendation only)
    pub enabled: bool,
}

impl Default for AutoPromotionConfig {
    fn default() -> Self {
        Self {
            min_sample_size: 100,
            min_parse_improvement: 0.05,
            enabled: false, // Disabled by default, require manual confirmation
        }
    }
}

/// Model pricing (cost per token)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelPricing {
    /// Cost per input token (in USD)
    pub input_cost_per_token: f64,
    /// Cost per output token (in USD)
    pub output_cost_per_token: f64,
}

impl Default for ModelPricing {
    fn default() -> Self {
        Self {
            input_cost_per_token: 0.0,
            output_cost_per_token: 0.0,
        }
    }
}

/// Token usage for a single consolidation
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Tokens in prompt
    pub prompt_tokens: u64,
    /// Tokens in completion
    pub completion_tokens: u64,
    /// Total tokens (prompt + completion)
    pub total_tokens: u64,
}

impl TokenUsage {
    /// Calculate cost based on model pricing
    #[must_use]
    #[allow(clippy::cast_precision_loss)]
    pub fn calculate_cost(&self, pricing: &ModelPricing) -> f64 {
        (self.prompt_tokens as f64).mul_add(
            pricing.input_cost_per_token,
            self.completion_tokens as f64 * pricing.output_cost_per_token,
        )
    }
}

/// Per-model cost and token metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModelMetrics {
    /// Total consolidations with this model
    pub consolidations: u64,
    /// Total token usage
    pub token_usage: TokenUsage,
    /// Total cost (USD)
    pub total_cost: f64,
    /// Number of errors
    pub errors: u64,
}

/// Consolidation lag statistics (episodic add → processed)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LagStats {
    /// Number of measurements
    pub count: u64,
    /// P50 lag (milliseconds)
    pub p50_ms: f64,
    /// P95 lag (milliseconds)
    pub p95_ms: f64,
    /// P99 lag (milliseconds)
    pub p99_ms: f64,
}

/// Throughput metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ThroughputMetrics {
    /// Entries processed per second
    pub entries_per_sec: f64,
    /// Decisions made per second
    pub decisions_per_sec: f64,
    /// Window start timestamp
    pub window_start: Option<DateTime<Utc>>,
    /// Window end timestamp
    pub window_end: Option<DateTime<Utc>>,
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
    /// Per-version prompt metrics
    pub prompt_metrics: HashMap<PromptVersion, PromptMetrics>,
    /// Per-model cost and token metrics
    pub model_metrics: HashMap<String, ModelMetrics>,
    /// Consolidation lag statistics
    pub lag: LagStats,
    /// Throughput metrics
    pub throughput: ThroughputMetrics,
}

/// Consolidation metrics collector
///
/// Thread-safe metrics aggregation for consolidation performance.
pub struct ConsolidationMetrics {
    core: Arc<RwLock<CoreMetrics>>,
    latencies: Arc<RwLock<Vec<f64>>>, // For percentile calculation
    lags: Arc<RwLock<Vec<f64>>>,      // For lag percentile calculation
    version_strategy: Arc<RwLock<VersionSelectionStrategy>>,
    auto_promotion: Arc<RwLock<AutoPromotionConfig>>,
    #[allow(clippy::zero_sized_map_values)] // PromptVersion zero-sized until V2 added
    session_versions: Arc<RwLock<HashMap<String, PromptVersion>>>, // session_id -> version
    model_pricing: Arc<RwLock<HashMap<String, ModelPricing>>>, // model -> pricing
}

impl Default for ConsolidationMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl ConsolidationMetrics {
    /// Create new metrics collector
    #[must_use]
    #[allow(clippy::zero_sized_map_values)] // PromptVersion zero-sized until V2 added
    pub fn new() -> Self {
        info!("Creating new ConsolidationMetrics collector");
        Self {
            core: Arc::new(RwLock::new(CoreMetrics::default())),
            latencies: Arc::new(RwLock::new(Vec::new())),
            lags: Arc::new(RwLock::new(Vec::new())),
            version_strategy: Arc::new(RwLock::new(VersionSelectionStrategy::default())),
            auto_promotion: Arc::new(RwLock::new(AutoPromotionConfig::default())),
            session_versions: Arc::new(RwLock::new(HashMap::new())),
            model_pricing: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Set model pricing for cost calculation
    pub async fn set_model_pricing(&self, model: String, pricing: ModelPricing) {
        self.model_pricing.write().await.insert(model, pricing);
    }

    /// Get model pricing
    pub async fn get_model_pricing(&self, model: &str) -> Option<ModelPricing> {
        self.model_pricing.read().await.get(model).cloned()
    }

    /// Set version selection strategy
    pub async fn set_version_strategy(&self, strategy: VersionSelectionStrategy) {
        *self.version_strategy.write().await = strategy;
    }

    /// Set auto-promotion configuration
    pub async fn set_auto_promotion(&self, config: AutoPromotionConfig) {
        *self.auto_promotion.write().await = config;
    }

    /// Helper: Select version using Fixed strategy
    fn select_fixed_version(version: PromptVersion) -> PromptVersion {
        trace!("Fixed strategy: using version {:?}", version);
        version
    }

    /// Helper: Select version using `RandomPerConsolidation` strategy
    fn select_random_per_consolidation() -> PromptVersion {
        // Random 50/50 split (currently only V1 exists)
        // TODO: When V2 is added, implement: if rand() < 0.5 { V1 } else { V2 }
        trace!("RandomPerConsolidation strategy: using V1 (only version available)");
        PromptVersion::V1
    }

    /// Helper: Select version using `RandomPerSession` strategy
    async fn select_random_per_session(&self, session_id: &str) -> PromptVersion {
        // Get or create session-sticky version
        let version = *self
            .session_versions
            .write()
            .await
            .entry(session_id.to_string())
            .or_insert_with(|| {
                // Random selection on first use
                // TODO: When V2 is added, implement random selection
                trace!("RandomPerSession strategy: first use for session, using V1");
                PromptVersion::V1
            });
        trace!(
            "RandomPerSession strategy: session sticky version {:?}",
            version
        );
        version
    }

    /// Select prompt version for consolidation
    ///
    /// Uses configured strategy (Fixed/RandomPerConsolidation/RandomPerSession).
    pub async fn select_version(&self, session_id: &str) -> PromptVersion {
        let strategy = *self.version_strategy.read().await;
        debug!(
            "Selecting prompt version: strategy={:?}, session_id={}",
            strategy, session_id
        );

        let version = match strategy {
            VersionSelectionStrategy::Fixed(version) => Self::select_fixed_version(version),
            VersionSelectionStrategy::RandomPerConsolidation => {
                Self::select_random_per_consolidation()
            }
            VersionSelectionStrategy::RandomPerSession => {
                self.select_random_per_session(session_id).await
            }
        };

        debug!("Selected prompt version: {:?}", version);
        version
    }

    /// Record a consolidation result
    ///
    /// Updates entries processed, decision distribution, and latency stats.
    ///
    /// # Panics
    ///
    /// Panics if latency values contain NaN (not expected in normal operation).
    #[allow(clippy::significant_drop_tightening)] // Intentional lock scope for atomicity
    #[allow(clippy::too_many_arguments)] // Metrics collection requires many parameters
    pub async fn record_consolidation(
        &self,
        result: &ConsolidationResult,
        decisions: &[DecisionPayload],
        duration: Duration,
        version: PromptVersion,
        parse_success: bool,
        model: Option<&str>,
        token_usage: Option<TokenUsage>,
        episodic_timestamps: &[DateTime<Utc>],
    ) {
        info!(
            "Recording consolidation: entries_processed={}, decisions={}, version={:?}, parse_success={}, model={}",
            result.entries_processed,
            decisions.len(),
            version,
            parse_success,
            model.unwrap_or("none")
        );
        trace!(
            "Consolidation details: duration={:?}, episodic_timestamps={}",
            duration,
            episodic_timestamps.len()
        );

        let duration_ms = duration.as_secs_f64() * 1000.0;

        // Update core metrics and push latency
        let mut sorted = {
            let mut core = self.core.write().await;

            // Update core metrics
            core.entries_processed += result.entries_processed as u64;
            core.consolidations += 1;

            // Update global decision distribution
            for decision in decisions {
                match DecisionType::from(decision) {
                    DecisionType::Add => core.decision_distribution.add_count += 1,
                    DecisionType::Update => core.decision_distribution.update_count += 1,
                    DecisionType::Delete => core.decision_distribution.delete_count += 1,
                    DecisionType::Noop => core.decision_distribution.noop_count += 1,
                }
            }

            // Update per-version metrics
            let version_metrics = core.prompt_metrics.entry(version).or_default();
            version_metrics.consolidations += 1;

            if parse_success {
                version_metrics.parse_successes += 1;
            } else {
                version_metrics.parse_failures += 1;
            }

            // Update per-version decision distribution
            for decision in decisions {
                match DecisionType::from(decision) {
                    DecisionType::Add => version_metrics.decision_distribution.add_count += 1,
                    DecisionType::Update => version_metrics.decision_distribution.update_count += 1,
                    DecisionType::Delete => version_metrics.decision_distribution.delete_count += 1,
                    DecisionType::Noop => version_metrics.decision_distribution.noop_count += 1,
                }
            }

            // Update per-model metrics (token usage, cost)
            if let (Some(model_name), Some(usage)) = (model, token_usage.clone()) {
                let model_metrics = core
                    .model_metrics
                    .entry(model_name.to_string())
                    .or_default();
                model_metrics.consolidations += 1;
                model_metrics.token_usage.prompt_tokens += usage.prompt_tokens;
                model_metrics.token_usage.completion_tokens += usage.completion_tokens;
                model_metrics.token_usage.total_tokens += usage.total_tokens;

                // Calculate cost if pricing available
                if let Some(pricing) = self.model_pricing.read().await.get(model_name) {
                    let cost = usage.calculate_cost(pricing);
                    model_metrics.total_cost += cost;
                }
            }

            // Update throughput window
            let now = Utc::now();
            if core.throughput.window_start.is_none() {
                core.throughput.window_start = Some(now);
            }
            core.throughput.window_end = Some(now);

            // Track latency and clone
            let cloned = {
                let mut latencies = self.latencies.write().await;
                latencies.push(duration_ms);
                latencies.clone()
            }; // Latencies lock dropped here

            cloned
        }; // Core lock dropped here

        // Update lag metrics from episodic timestamps
        self.update_lag_metrics(episodic_timestamps).await;

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

    /// Update consolidation lag metrics from episodic timestamps
    async fn update_lag_metrics(&self, episodic_timestamps: &[DateTime<Utc>]) {
        if episodic_timestamps.is_empty() {
            return;
        }

        let now = Utc::now();
        let mut lag_values = self.lags.write().await;

        #[allow(clippy::cast_precision_loss)]
        // Milliseconds precision acceptable for lag metrics
        for timestamp in episodic_timestamps {
            let lag_ms = (now - *timestamp).num_milliseconds() as f64;
            lag_values.push(lag_ms);
        }

        // Update lag stats
        if !lag_values.is_empty() {
            let mut sorted_lags = lag_values.clone();
            drop(lag_values); // Release lock before sorting and stats update
            sorted_lags.sort_by(|a, b| a.partial_cmp(b).unwrap());

            let mut core = self.core.write().await;
            #[allow(clippy::cast_precision_loss)]
            {
                core.lag.count = sorted_lags.len() as u64;
                core.lag.p50_ms = percentile(&sorted_lags, 50.0);
                core.lag.p95_ms = percentile(&sorted_lags, 95.0);
                core.lag.p99_ms = percentile(&sorted_lags, 99.0);
            }
        }
    }

    /// Get current metrics snapshot
    pub async fn snapshot(&self) -> CoreMetrics {
        self.core.read().await.clone()
    }

    /// Reset all metrics
    pub async fn reset(&self) {
        info!("Resetting all consolidation metrics");
        *self.core.write().await = CoreMetrics::default();
        self.latencies.write().await.clear();
        self.lags.write().await.clear();
        self.session_versions.write().await.clear();
        debug!("All metrics cleared");
    }

    /// Calculate current throughput metrics
    ///
    /// Returns entries/sec and decisions/sec based on accumulated data and time window.
    /// Also updates core.throughput with calculated values.
    pub async fn calculate_throughput(&self) -> ThroughputMetrics {
        debug!("Calculating throughput metrics");
        let mut core = self.core.write().await;

        #[allow(clippy::cast_precision_loss)] // Milliseconds precision acceptable for throughput
        let window_duration_secs = if let (Some(start), Some(end)) =
            (core.throughput.window_start, core.throughput.window_end)
        {
            (end - start).num_milliseconds() as f64 / 1000.0
        } else {
            0.0
        };

        if window_duration_secs > 0.0 {
            #[allow(clippy::cast_precision_loss)]
            {
                core.throughput.entries_per_sec =
                    core.entries_processed as f64 / window_duration_secs;
                core.throughput.decisions_per_sec =
                    core.decision_distribution.total() as f64 / window_duration_secs;
            }
            debug!(
                "Throughput calculated: entries/sec={:.2}, decisions/sec={:.2}, window={:.2}s",
                core.throughput.entries_per_sec,
                core.throughput.decisions_per_sec,
                window_duration_secs
            );
        } else {
            debug!("Throughput: no time window available");
        }

        core.throughput.clone()
    }

    /// Record model error
    #[allow(clippy::significant_drop_tightening)] // Short operation, lock scope acceptable
    pub async fn record_model_error(&self, model: &str) {
        let mut core = self.core.write().await;
        let model_metrics = core.model_metrics.entry(model.to_string()).or_default();
        model_metrics.errors += 1;
    }

    /// Get model metrics
    pub async fn get_model_metrics(&self, model: &str) -> Option<ModelMetrics> {
        self.core.read().await.model_metrics.get(model).cloned()
    }

    /// Helper: Check if version has sufficient sample size
    fn has_sufficient_samples(
        metrics: &PromptMetrics,
        config: &AutoPromotionConfig,
        version: PromptVersion,
    ) -> bool {
        if metrics.consolidations >= config.min_sample_size {
            true
        } else {
            trace!(
                "Version {:?}: insufficient samples ({} < {})",
                version,
                metrics.consolidations,
                config.min_sample_size
            );
            false
        }
    }

    /// Helper: Calculate parse rate improvement from baseline to candidate
    fn calculate_parse_improvement(baseline: &PromptMetrics, candidate: &PromptMetrics) -> f64 {
        let baseline_rate = baseline.parse_success_rate();
        let candidate_rate = candidate.parse_success_rate();
        candidate_rate - baseline_rate
    }

    /// Helper: Check if improvement meets promotion threshold
    fn meets_promotion_threshold(
        improvement: f64,
        config: &AutoPromotionConfig,
        version: PromptVersion,
    ) -> bool {
        if improvement >= config.min_parse_improvement {
            info!(
                "Auto-promotion recommended: version {:?} shows {:.2}% improvement (threshold={:.2}%)",
                version,
                improvement * 100.0,
                config.min_parse_improvement * 100.0
            );
            true
        } else {
            false
        }
    }

    /// Helper: Evaluate candidate version against baseline
    fn evaluate_candidate(
        version: PromptVersion,
        metrics: &PromptMetrics,
        baseline: &PromptMetrics,
        config: &AutoPromotionConfig,
    ) -> Option<PromptVersion> {
        if version == PromptVersion::V1 {
            return None; // Skip baseline itself
        }

        trace!("Evaluating version {:?} for auto-promotion", version);

        if !Self::has_sufficient_samples(metrics, config, version) {
            return None;
        }

        let improvement = Self::calculate_parse_improvement(baseline, metrics);
        let baseline_rate = baseline.parse_success_rate();
        let candidate_rate = metrics.parse_success_rate();

        debug!(
            "Version {:?}: baseline_rate={:.2}%, candidate_rate={:.2}%, improvement={:.2}%",
            version,
            baseline_rate * 100.0,
            candidate_rate * 100.0,
            improvement * 100.0
        );

        if Self::meets_promotion_threshold(improvement, config, version) {
            Some(version)
        } else {
            None
        }
    }

    /// Check if auto-promotion should occur
    ///
    /// Compares performance of available versions and returns promotion recommendation.
    /// Returns `Some(version)` if promotion criteria met, `None` otherwise.
    #[allow(clippy::significant_drop_tightening)] // Read locks held for full comparison
    pub async fn check_auto_promotion(&self) -> Option<PromptVersion> {
        debug!("Checking auto-promotion criteria");
        let core = self.core.read().await;
        let config = self.auto_promotion.read().await;

        // Need at least 2 versions to compare
        if core.prompt_metrics.len() < 2 {
            debug!(
                "Auto-promotion: not enough versions ({} < 2)",
                core.prompt_metrics.len()
            );
            return None;
        }

        // Find baseline version (currently V1)
        let baseline = core.prompt_metrics.get(&PromptVersion::V1)?;

        // Check each candidate version
        for (version, metrics) in &core.prompt_metrics {
            if let Some(promoted) = Self::evaluate_candidate(*version, metrics, baseline, &config) {
                return Some(promoted);
            }
        }

        debug!("Auto-promotion: no version meets criteria");
        None
    }

    /// Get prompt metrics for specific version
    pub async fn get_prompt_metrics(&self, version: PromptVersion) -> Option<PromptMetrics> {
        self.core.read().await.prompt_metrics.get(&version).cloned()
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
        let dist = DecisionDistribution {
            add_count: 40,
            update_count: 30,
            delete_count: 10,
            noop_count: 20,
        };

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
        assert!((percentile(&values, 0.0) - 10.0).abs() < 0.01);
        assert!((percentile(&values, 100.0) - 100.0).abs() < 0.01);
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
            entries_failed: 0,
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
            .record_consolidation(
                &result,
                &decisions,
                Duration::from_millis(100),
                PromptVersion::V1,
                true,
                None,
                None,
                &[],
            )
            .await;

        let snapshot = metrics.snapshot().await;
        assert_eq!(snapshot.entries_processed, 10);
        assert_eq!(snapshot.consolidations, 1);
        assert_eq!(snapshot.decision_distribution.add_count, 1);
        assert_eq!(snapshot.decision_distribution.update_count, 1);
        assert_eq!(snapshot.decision_distribution.delete_count, 1);
        assert_eq!(snapshot.decision_distribution.noop_count, 1);
        assert!(snapshot.latency.avg_ms() > 0.0);

        // Check per-version metrics
        let v1_metrics = snapshot.prompt_metrics.get(&PromptVersion::V1).unwrap();
        assert_eq!(v1_metrics.consolidations, 1);
        assert_eq!(v1_metrics.parse_successes, 1);
        assert_eq!(v1_metrics.parse_failures, 0);
        assert!((v1_metrics.parse_success_rate() - 1.0).abs() < 0.01);
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
        let metrics = ConsolidationMetrics::new();

        let result = ConsolidationResult {
            entries_processed: 5,
            entities_added: 2,
            entities_updated: 1,
            entities_deleted: 0,
            entries_skipped: 2,
            entries_failed: 0,
            duration_ms: 50,
        };

        metrics
            .record_consolidation(
                &result,
                &[],
                Duration::from_millis(50),
                PromptVersion::V1,
                true,
                None,
                None,
                &[],
            )
            .await;

        metrics.reset().await;

        let snapshot = metrics.snapshot().await;
        assert_eq!(snapshot.entries_processed, 0);
        assert_eq!(snapshot.consolidations, 0);
        assert!(snapshot.prompt_metrics.is_empty());
    }

    #[tokio::test]
    async fn test_prompt_metrics_per_version() {
        let metrics = ConsolidationMetrics::new();

        let result = ConsolidationResult {
            entries_processed: 5,
            entities_added: 2,
            entities_updated: 1,
            entities_deleted: 0,
            entries_skipped: 2,
            entries_failed: 0,
            duration_ms: 50,
        };

        // Record successful parse
        metrics
            .record_consolidation(
                &result,
                &[],
                Duration::from_millis(50),
                PromptVersion::V1,
                true,
                None,
                None,
                &[],
            )
            .await;

        // Record failed parse
        metrics
            .record_consolidation(
                &result,
                &[],
                Duration::from_millis(60),
                PromptVersion::V1,
                false,
                None,
                None,
                &[],
            )
            .await;

        let v1_metrics = metrics.get_prompt_metrics(PromptVersion::V1).await.unwrap();
        assert_eq!(v1_metrics.consolidations, 2);
        assert_eq!(v1_metrics.parse_successes, 1);
        assert_eq!(v1_metrics.parse_failures, 1);
        assert!((v1_metrics.parse_success_rate() - 0.5).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_version_selection_fixed() {
        let metrics = ConsolidationMetrics::new();
        metrics
            .set_version_strategy(VersionSelectionStrategy::Fixed(PromptVersion::V1))
            .await;

        let version = metrics.select_version("session-1").await;
        assert_eq!(version, PromptVersion::V1);
    }

    #[tokio::test]
    async fn test_version_selection_random_per_consolidation() {
        let metrics = ConsolidationMetrics::new();
        metrics
            .set_version_strategy(VersionSelectionStrategy::RandomPerConsolidation)
            .await;

        // Currently only V1 exists, so should always return V1
        let version = metrics.select_version("session-1").await;
        assert_eq!(version, PromptVersion::V1);
    }

    #[tokio::test]
    async fn test_version_selection_random_per_session() {
        let metrics = ConsolidationMetrics::new();
        metrics
            .set_version_strategy(VersionSelectionStrategy::RandomPerSession)
            .await;

        // Same session should get same version
        let version1 = metrics.select_version("session-1").await;
        let version2 = metrics.select_version("session-1").await;
        assert_eq!(version1, version2);

        // Currently only V1 exists
        assert_eq!(version1, PromptVersion::V1);
    }

    #[tokio::test]
    async fn test_auto_promotion_not_enough_samples() {
        let metrics = ConsolidationMetrics::new();

        let config = AutoPromotionConfig {
            min_sample_size: 100,
            min_parse_improvement: 0.05,
            enabled: true,
        };
        metrics.set_auto_promotion(config).await;

        // No promotion with only one version
        let promotion = metrics.check_auto_promotion().await;
        assert!(promotion.is_none());
    }

    #[tokio::test]
    async fn test_token_usage_and_cost() {
        let metrics = ConsolidationMetrics::new();

        // Set pricing
        let pricing = ModelPricing {
            input_cost_per_token: 0.000_001,  // $1 per million tokens
            output_cost_per_token: 0.000_002, // $2 per million tokens
        };
        metrics
            .set_model_pricing("ollama/llama3.2:3b".to_string(), pricing)
            .await;

        let result = ConsolidationResult {
            entries_processed: 5,
            entities_added: 2,
            entities_updated: 1,
            entities_deleted: 0,
            entries_skipped: 2,
            entries_failed: 0,
            duration_ms: 50,
        };

        let usage = TokenUsage {
            prompt_tokens: 1000,
            completion_tokens: 500,
            total_tokens: 1500,
        };

        metrics
            .record_consolidation(
                &result,
                &[],
                Duration::from_millis(50),
                PromptVersion::V1,
                true,
                Some("ollama/llama3.2:3b"),
                Some(usage.clone()),
                &[],
            )
            .await;

        let model_metrics = metrics
            .get_model_metrics("ollama/llama3.2:3b")
            .await
            .unwrap();
        assert_eq!(model_metrics.consolidations, 1);
        assert_eq!(model_metrics.token_usage.prompt_tokens, 1000);
        assert_eq!(model_metrics.token_usage.completion_tokens, 500);
        assert_eq!(model_metrics.token_usage.total_tokens, 1500);

        // Cost: (1000 * 0.000001) + (500 * 0.000002) = 0.001 + 0.001 = 0.002
        assert!((model_metrics.total_cost - 0.002).abs() < 0.0001);
    }

    #[tokio::test]
    async fn test_consolidation_lag() {
        use chrono::Duration as ChronoDuration;

        let metrics = ConsolidationMetrics::new();

        let result = ConsolidationResult {
            entries_processed: 3,
            entities_added: 1,
            entities_updated: 0,
            entities_deleted: 0,
            entries_skipped: 2,
            entries_failed: 0,
            duration_ms: 50,
        };

        // Simulate episodic entries added 100ms, 200ms, 300ms ago
        let now = Utc::now();
        let timestamps = vec![
            now - ChronoDuration::milliseconds(100),
            now - ChronoDuration::milliseconds(200),
            now - ChronoDuration::milliseconds(300),
        ];

        metrics
            .record_consolidation(
                &result,
                &[],
                Duration::from_millis(50),
                PromptVersion::V1,
                true,
                None,
                None,
                &timestamps,
            )
            .await;

        let snapshot = metrics.snapshot().await;
        assert_eq!(snapshot.lag.count, 3);
        // P50 should be around 200ms
        assert!(snapshot.lag.p50_ms > 150.0 && snapshot.lag.p50_ms < 250.0);
        // P95 should be around 300ms
        assert!(snapshot.lag.p95_ms > 250.0 && snapshot.lag.p95_ms < 350.0);
    }

    #[tokio::test]
    async fn test_throughput_calculation() {
        let metrics = ConsolidationMetrics::new();

        let result = ConsolidationResult {
            entries_processed: 100,
            entities_added: 50,
            entities_updated: 30,
            entities_deleted: 10,
            entries_skipped: 10,
            entries_failed: 0,
            duration_ms: 1000,
        };

        let decisions = vec![
            DecisionPayload::Add {
                entity_id: "e1".to_string(),
            },
            DecisionPayload::Add {
                entity_id: "e2".to_string(),
            },
            DecisionPayload::Update {
                entity_id: "e3".to_string(),
                changes: HashMap::new(),
            },
        ];

        // Record first consolidation
        metrics
            .record_consolidation(
                &result,
                &decisions,
                Duration::from_millis(1000),
                PromptVersion::V1,
                true,
                None,
                None,
                &[],
            )
            .await;

        // Wait to create time window
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Record second consolidation (extends window)
        metrics
            .record_consolidation(
                &result,
                &decisions,
                Duration::from_millis(1000),
                PromptVersion::V1,
                true,
                None,
                None,
                &[],
            )
            .await;

        let throughput = metrics.calculate_throughput().await;
        // Should process 200 entries over ~100ms window = ~2000 entries/sec
        assert!(throughput.entries_per_sec > 0.0);
        assert!(throughput.decisions_per_sec > 0.0);
        assert!(throughput.window_start.is_some());
        assert!(throughput.window_end.is_some());
    }

    #[tokio::test]
    async fn test_model_error_tracking() {
        let metrics = ConsolidationMetrics::new();

        metrics.record_model_error("ollama/llama3.2:3b").await;
        metrics.record_model_error("ollama/llama3.2:3b").await;

        let model_metrics = metrics
            .get_model_metrics("ollama/llama3.2:3b")
            .await
            .unwrap();
        assert_eq!(model_metrics.errors, 2);
    }
}
