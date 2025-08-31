//! Hook profiler trait abstraction for performance monitoring of debug hooks
//!
//! Provides trait-based abstraction for monitoring hook execution performance,
//! enabling testability and supporting multiple profiler backends with
//! workload-aware adaptive thresholds.

use std::collections::HashMap;
use std::error::Error;
use std::time::{Duration, Instant};

/// Type of operation being profiled in hooks
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OperationType {
    /// Synchronous operation - stricter thresholds
    Synchronous,
    /// Asynchronous operation - relaxed thresholds  
    Asynchronous,
    /// Batch operation with specified item count
    Batch(usize),
}

/// Workload classification for adaptive profiling thresholds
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkloadClassifier {
    /// Micro operations - <1ms expected
    Micro,
    /// Light operations - <10ms expected
    Light,
    /// Medium operations - <100ms expected
    Medium,
    /// Heavy operations - <1s expected
    Heavy,
}

impl WorkloadClassifier {
    /// Get the base threshold for this workload type
    #[must_use]
    pub const fn base_threshold_ms(self) -> f64 {
        match self {
            Self::Micro => 0.1,   // 100 microseconds
            Self::Light => 1.0,   // 1 millisecond
            Self::Medium => 10.0, // 10 milliseconds
            Self::Heavy => 100.0, // 100 milliseconds
        }
    }
}

/// Configuration for hook profiling with adaptive thresholds
#[derive(Debug, Clone)]
pub struct HookProfilingConfig {
    /// Threshold for synchronous hooks (stricter)
    pub sync_hook_threshold_ms: f64,
    /// Threshold for asynchronous hooks (relaxed)
    pub async_hook_threshold_ms: f64,
    /// Threshold for batch operations based on size
    pub batch_operation_threshold_ms: f64,
    /// Enable adaptive sampling when overhead is high
    pub adaptive_sampling: bool,
    /// Workload classifier for adaptive thresholds
    pub workload_classifier: WorkloadClassifier,
    /// Maximum overhead percentage before reducing sampling
    pub max_overhead_percent: f64,
    /// Minimum sampling rate (never go below this)
    pub min_sampling_rate: f64,
}

impl Default for HookProfilingConfig {
    fn default() -> Self {
        Self {
            sync_hook_threshold_ms: 1.0,        // Strict for sync
            async_hook_threshold_ms: 5.0,       // Relaxed for async
            batch_operation_threshold_ms: 10.0, // Based on batch size
            adaptive_sampling: true,
            workload_classifier: WorkloadClassifier::Light,
            max_overhead_percent: 5.0, // 5% max overhead
            min_sampling_rate: 0.01,   // Never sample less than 1%
        }
    }
}

impl HookProfilingConfig {
    /// Create config optimized for micro operations
    #[must_use]
    pub fn micro() -> Self {
        Self {
            sync_hook_threshold_ms: 0.1,
            async_hook_threshold_ms: 0.5,
            batch_operation_threshold_ms: 1.0,
            workload_classifier: WorkloadClassifier::Micro,
            max_overhead_percent: 2.0, // Stricter overhead for micro
            ..Default::default()
        }
    }

    /// Create config optimized for heavy operations
    #[must_use]
    pub fn heavy() -> Self {
        Self {
            sync_hook_threshold_ms: 50.0,
            async_hook_threshold_ms: 200.0,
            batch_operation_threshold_ms: 500.0,
            workload_classifier: WorkloadClassifier::Heavy,
            max_overhead_percent: 10.0, // More relaxed for heavy
            ..Default::default()
        }
    }

    /// Get threshold for specific operation type
    #[must_use]
    pub fn threshold_for_operation(&self, op_type: OperationType) -> f64 {
        match op_type {
            OperationType::Synchronous => self.sync_hook_threshold_ms,
            OperationType::Asynchronous => self.async_hook_threshold_ms,
            OperationType::Batch(size) => {
                // Scale batch threshold by item count with log base scaling
                // Use log10(size + 1) to ensure Batch(1) < Batch(10) < Batch(100)
                #[allow(clippy::cast_precision_loss)] // Acceptable for batch size scaling
                {
                    self.batch_operation_threshold_ms * ((size.max(1) + 1) as f64).log10()
                }
            }
        }
    }
}

/// Profile report containing hook execution metrics
#[derive(Debug, Clone)]
pub struct ProfileReport {
    /// Hook execution samples
    pub hook_samples: HashMap<String, HookMetrics>,
    /// Total profiling duration
    pub duration: Duration,
    /// Overall overhead percentage
    pub overhead_percent: f64,
    /// Number of samples collected
    pub sample_count: usize,
    /// Current sampling rate
    pub sampling_rate: f64,
}

/// Metrics for a specific hook
#[derive(Debug, Clone)]
pub struct HookMetrics {
    /// Hook name
    pub name: String,
    /// Total execution count
    pub execution_count: u64,
    /// Total execution time
    pub total_duration: Duration,
    /// Average execution time
    pub avg_duration: Duration,
    /// Maximum execution time
    pub max_duration: Duration,
    /// Minimum execution time  
    pub min_duration: Duration,
    /// Operation type for this hook
    pub operation_type: OperationType,
    /// Number of threshold violations
    pub threshold_violations: u64,
}

impl HookMetrics {
    /// Create new hook metrics
    #[must_use]
    pub const fn new(name: String, operation_type: OperationType) -> Self {
        Self {
            name,
            execution_count: 0,
            total_duration: Duration::ZERO,
            avg_duration: Duration::ZERO,
            max_duration: Duration::ZERO,
            min_duration: Duration::MAX,
            operation_type,
            threshold_violations: 0,
        }
    }

    /// Record a new execution sample
    pub fn record_execution(&mut self, duration: Duration, threshold_ms: f64) {
        self.execution_count += 1;
        self.total_duration += duration;
        self.max_duration = self.max_duration.max(duration);
        self.min_duration = self.min_duration.min(duration);
        self.avg_duration = if self.execution_count > 0 {
            #[allow(clippy::cast_possible_truncation, clippy::cast_lossless)]
            // Safe for execution count
            {
                self.total_duration / (self.execution_count.min(u64::from(u32::MAX)) as u32).max(1)
            }
        } else {
            Duration::ZERO
        };

        // Check for threshold violation
        if duration.as_secs_f64() * 1000.0 > threshold_ms {
            self.threshold_violations += 1;
        }
    }
}

/// Trait for profiling hook execution performance
pub trait HookProfiler: Send + Sync {
    /// Start profiling with given configuration
    ///
    /// # Errors
    /// Returns error if profiling cannot be started or is already active
    fn start_profiling(&mut self, config: HookProfilingConfig) -> Result<(), Box<dyn Error>>;

    /// Stop profiling and get report
    ///
    /// # Errors
    /// Returns error if profiling is not active
    fn stop_profiling(&mut self) -> Result<ProfileReport, Box<dyn Error>>;

    /// Sample hook execution performance
    fn sample_hook_execution(
        &mut self,
        hook_name: &str,
        duration: Duration,
        op_type: OperationType,
    );

    /// Adapt sampling rate based on observed overhead
    fn adapt_sampling_rate(&mut self, observed_overhead: f64);

    /// Check if profiling is currently active
    fn is_active(&self) -> bool;

    /// Get current sampling rate
    fn sampling_rate(&self) -> f64;

    /// Get current configuration (if active)
    fn config(&self) -> Option<&HookProfilingConfig>;
}

/// Real hook profiler implementation
pub struct RealHookProfiler {
    /// Current configuration
    config: Option<HookProfilingConfig>,
    /// Hook metrics by name
    metrics: HashMap<String, HookMetrics>,
    /// Profiling start time
    start_time: Option<Instant>,
    /// Current sampling rate
    current_sampling_rate: f64,
    /// Total samples collected
    total_samples: usize,
    /// Overhead measurement samples
    overhead_samples: Vec<f64>,
}

impl RealHookProfiler {
    /// Create a new real hook profiler
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: None,
            metrics: HashMap::new(),
            start_time: None,
            current_sampling_rate: 1.0,
            total_samples: 0,
            overhead_samples: Vec::new(),
        }
    }

    /// Calculate current overhead percentage
    #[allow(clippy::cast_precision_loss)] // Acceptable for averaging calculation
    fn calculate_overhead(&self) -> f64 {
        if self.overhead_samples.is_empty() {
            return 0.0;
        }

        let sum: f64 = self.overhead_samples.iter().sum();
        sum / self.overhead_samples.len() as f64
    }
}

impl Default for RealHookProfiler {
    fn default() -> Self {
        Self::new()
    }
}

impl HookProfiler for RealHookProfiler {
    fn start_profiling(&mut self, config: HookProfilingConfig) -> Result<(), Box<dyn Error>> {
        if self.config.is_some() {
            return Err("Hook profiling is already active".into());
        }

        self.config = Some(config);
        self.start_time = Some(Instant::now());
        self.current_sampling_rate = 1.0;
        self.metrics.clear();
        self.total_samples = 0;
        self.overhead_samples.clear();

        Ok(())
    }

    fn stop_profiling(&mut self) -> Result<ProfileReport, Box<dyn Error>> {
        let _config = self.config.take().ok_or("Hook profiling is not active")?;
        let start_time = self.start_time.take().ok_or("No start time recorded")?;

        let duration = start_time.elapsed();
        let overhead_percent = self.calculate_overhead();

        let report = ProfileReport {
            hook_samples: self.metrics.clone(),
            duration,
            overhead_percent,
            sample_count: self.total_samples,
            sampling_rate: self.current_sampling_rate,
        };

        // Reset state
        self.metrics.clear();
        self.total_samples = 0;
        self.overhead_samples.clear();

        Ok(report)
    }

    fn sample_hook_execution(
        &mut self,
        hook_name: &str,
        duration: Duration,
        op_type: OperationType,
    ) {
        if let Some(ref config) = self.config {
            // Apply sampling rate
            if rand::random::<f64>() > self.current_sampling_rate {
                return;
            }

            let threshold = config.threshold_for_operation(op_type);

            let metrics = self
                .metrics
                .entry(hook_name.to_string())
                .or_insert_with(|| HookMetrics::new(hook_name.to_string(), op_type));

            metrics.record_execution(duration, threshold);
            self.total_samples += 1;

            // Record overhead sample (duration as percentage of expected threshold)
            let overhead = (duration.as_secs_f64() * 1000.0) / threshold * 100.0;
            self.overhead_samples.push(overhead);

            // Keep overhead samples window manageable
            if self.overhead_samples.len() > 1000 {
                self.overhead_samples.remove(0);
            }
        }
    }

    fn adapt_sampling_rate(&mut self, observed_overhead: f64) {
        if let Some(ref config) = self.config {
            if config.adaptive_sampling && observed_overhead > config.max_overhead_percent {
                // Reduce sampling rate proportionally to overhead
                let reduction_factor = config.max_overhead_percent / observed_overhead;
                self.current_sampling_rate =
                    (self.current_sampling_rate * reduction_factor).max(config.min_sampling_rate);
            }
        }
    }

    fn is_active(&self) -> bool {
        self.config.is_some()
    }

    fn sampling_rate(&self) -> f64 {
        self.current_sampling_rate
    }

    fn config(&self) -> Option<&HookProfilingConfig> {
        self.config.as_ref()
    }
}

// Add rand dependency if not already present - we'll use a simple approach
mod rand {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[allow(clippy::cast_precision_loss)] // Acceptable for random number generation
    pub fn random<T>() -> T
    where
        T: From<f64>,
    {
        let mut hasher = DefaultHasher::new();
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
            .hash(&mut hasher);
        let hash = hasher.finish();
        T::from((hash as f64) / (u64::MAX as f64))
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)] // Test constants are safe for exact comparison
mod tests {
    use super::*;

    #[test]
    fn test_workload_classifier_thresholds() {
        assert_eq!(WorkloadClassifier::Micro.base_threshold_ms(), 0.1);
        assert_eq!(WorkloadClassifier::Light.base_threshold_ms(), 1.0);
        assert_eq!(WorkloadClassifier::Medium.base_threshold_ms(), 10.0);
        assert_eq!(WorkloadClassifier::Heavy.base_threshold_ms(), 100.0);
    }

    #[test]
    fn test_hook_profiling_config_defaults() {
        let config = HookProfilingConfig::default();
        assert_eq!(config.sync_hook_threshold_ms, 1.0);
        assert_eq!(config.async_hook_threshold_ms, 5.0);
        assert!(config.adaptive_sampling);
    }

    #[test]
    fn test_hook_profiling_config_presets() {
        let micro_config = HookProfilingConfig::micro();
        assert_eq!(micro_config.sync_hook_threshold_ms, 0.1);
        assert_eq!(micro_config.max_overhead_percent, 2.0);

        let heavy_config = HookProfilingConfig::heavy();
        assert_eq!(heavy_config.sync_hook_threshold_ms, 50.0);
        assert_eq!(heavy_config.max_overhead_percent, 10.0);
    }

    #[test]
    fn test_threshold_for_operation() {
        let config = HookProfilingConfig::default();

        assert_eq!(
            config.threshold_for_operation(OperationType::Synchronous),
            1.0
        );
        assert_eq!(
            config.threshold_for_operation(OperationType::Asynchronous),
            5.0
        );

        // Batch threshold scales with log of size
        let batch_1 = config.threshold_for_operation(OperationType::Batch(1));
        let batch_10 = config.threshold_for_operation(OperationType::Batch(10));
        let batch_100 = config.threshold_for_operation(OperationType::Batch(100));

        assert!(batch_100 > batch_10);
        assert!(batch_10 > batch_1);
    }

    #[test]
    fn test_hook_metrics() {
        let mut metrics = HookMetrics::new("test_hook".to_string(), OperationType::Synchronous);

        assert_eq!(metrics.execution_count, 0);
        assert_eq!(metrics.threshold_violations, 0);

        // Record execution under threshold
        metrics.record_execution(Duration::from_millis(1), 2.0);
        assert_eq!(metrics.execution_count, 1);
        assert_eq!(metrics.threshold_violations, 0);

        // Record execution over threshold
        metrics.record_execution(Duration::from_millis(3), 2.0);
        assert_eq!(metrics.execution_count, 2);
        assert_eq!(metrics.threshold_violations, 1);
    }

    #[test]
    fn test_real_hook_profiler_lifecycle() {
        let mut profiler = RealHookProfiler::new();

        assert!(!profiler.is_active());
        assert_eq!(profiler.sampling_rate(), 1.0);

        // Start profiling
        let config = HookProfilingConfig::default();
        assert!(profiler.start_profiling(config).is_ok());
        assert!(profiler.is_active());

        // Cannot start when already active
        assert!(profiler
            .start_profiling(HookProfilingConfig::default())
            .is_err());

        // Sample some hook executions
        profiler.sample_hook_execution(
            "test_hook",
            Duration::from_millis(1),
            OperationType::Synchronous,
        );
        profiler.sample_hook_execution(
            "async_hook",
            Duration::from_millis(3),
            OperationType::Asynchronous,
        );

        // Stop profiling
        let report = profiler.stop_profiling();
        assert!(report.is_ok());
        assert!(!profiler.is_active());

        let report = report.unwrap();
        assert!(report.hook_samples.contains_key("test_hook"));
        assert!(report.hook_samples.contains_key("async_hook"));
    }

    #[test]
    fn test_adaptive_sampling() {
        let mut profiler = RealHookProfiler::new();
        let config = HookProfilingConfig {
            max_overhead_percent: 5.0,
            min_sampling_rate: 0.1,
            ..Default::default()
        };

        profiler.start_profiling(config).unwrap();

        // Initial sampling rate should be 1.0
        assert_eq!(profiler.sampling_rate(), 1.0);

        // Adapt to high overhead (10%)
        profiler.adapt_sampling_rate(10.0);

        // Should reduce sampling rate
        assert!(profiler.sampling_rate() < 1.0);
        assert!(profiler.sampling_rate() >= 0.1); // Should not go below minimum
    }
}
