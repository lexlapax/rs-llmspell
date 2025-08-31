//! Profiling configuration with adaptive overhead thresholds
//!
//! Provides configurable profiling behavior based on workload characteristics
//! for future scalability and performance tuning.

use std::time::Duration;

/// Profiling configuration with adaptive thresholds
#[derive(Debug, Clone)]
pub struct ProfilingConfig {
    /// Sampling rate in Hz (samples per second)
    pub sample_rate_hz: u32,

    /// Adaptive overhead thresholds based on workload
    pub overhead_thresholds: OverheadThresholds,

    /// Enable automatic rate adjustment based on overhead
    pub adaptive_sampling: bool,

    /// Minimum sampling rate when adapting (Hz)
    pub min_sample_rate_hz: u32,

    /// Maximum sampling rate when adapting (Hz)
    pub max_sample_rate_hz: u32,

    /// Memory profiling settings
    pub memory_profiling: MemoryProfilingConfig,
}

/// Overhead thresholds that adapt to workload characteristics
#[derive(Debug, Clone)]
pub struct OverheadThresholds {
    /// Max overhead for micro workloads (<100ms) - higher tolerance
    pub micro_workload_percent: f64,

    /// Max overhead for light workloads (100ms-1s)
    pub light_workload_percent: f64,

    /// Max overhead for medium workloads (1s-10s)
    pub medium_workload_percent: f64,

    /// Max overhead for heavy workloads (>10s) - strict requirement
    pub heavy_workload_percent: f64,
}

/// Memory profiling specific configuration
#[derive(Debug, Clone)]
pub struct MemoryProfilingConfig {
    /// Enable memory profiling
    pub enabled: bool,

    /// Sample every Nth allocation
    pub sample_interval: usize,

    /// Minimum allocation size to track (bytes)
    pub min_allocation_size: usize,

    /// Warning threshold for potential memory leaks (bytes)
    pub leak_warning_threshold: usize,
}

impl ProfilingConfig {
    /// Create config optimized for production use
    #[must_use]
    pub const fn production() -> Self {
        Self {
            sample_rate_hz: 100,
            overhead_thresholds: OverheadThresholds::production(),
            adaptive_sampling: true,
            min_sample_rate_hz: 10,
            max_sample_rate_hz: 1000,
            memory_profiling: MemoryProfilingConfig::production(),
        }
    }

    /// Create config optimized for development/debugging
    #[must_use]
    pub const fn development() -> Self {
        Self {
            sample_rate_hz: 1000,
            overhead_thresholds: OverheadThresholds::development(),
            adaptive_sampling: false,
            min_sample_rate_hz: 100,
            max_sample_rate_hz: 10000,
            memory_profiling: MemoryProfilingConfig::development(),
        }
    }

    /// Create config optimized for benchmarking
    #[must_use]
    pub const fn benchmark() -> Self {
        Self {
            sample_rate_hz: 100,
            overhead_thresholds: OverheadThresholds::benchmark(),
            adaptive_sampling: false,
            min_sample_rate_hz: 10,
            max_sample_rate_hz: 100,
            memory_profiling: MemoryProfilingConfig::disabled(),
        }
    }

    /// Get appropriate threshold based on workload duration
    #[must_use]
    pub const fn get_overhead_threshold(&self, workload_duration: Duration) -> f64 {
        let millis = workload_duration.as_millis();
        match millis {
            0..=100 => self.overhead_thresholds.micro_workload_percent,
            101..=1000 => self.overhead_thresholds.light_workload_percent,
            1001..=10000 => self.overhead_thresholds.medium_workload_percent,
            _ => self.overhead_thresholds.heavy_workload_percent,
        }
    }

    /// Calculate adaptive sample rate based on observed overhead
    #[must_use]
    pub fn calculate_adaptive_rate(&self, current_overhead_percent: f64) -> u32 {
        if !self.adaptive_sampling {
            return self.sample_rate_hz;
        }

        // Reduce rate if overhead too high
        if current_overhead_percent > 10.0 {
            let reduction_factor = 10.0 / current_overhead_percent;
            let new_rate_f64 = f64::from(self.sample_rate_hz) * reduction_factor;
            // Safe cast: reduction_factor is always positive and less than 1.0
            // when overhead > 10%, so new_rate will be less than sample_rate_hz
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let new_rate = new_rate_f64.round() as u32;
            new_rate.max(self.min_sample_rate_hz)
        } else {
            self.sample_rate_hz
        }
    }
}

impl OverheadThresholds {
    /// Production thresholds - strict for performance
    #[must_use]
    pub const fn production() -> Self {
        Self {
            micro_workload_percent: 30.0, // Tolerate higher overhead for tiny workloads
            light_workload_percent: 15.0,
            medium_workload_percent: 10.0,
            heavy_workload_percent: 5.0, // Strict for long-running workloads
        }
    }

    /// Development thresholds - relaxed for debugging
    #[must_use]
    pub const fn development() -> Self {
        Self {
            micro_workload_percent: 50.0,
            light_workload_percent: 30.0,
            medium_workload_percent: 20.0,
            heavy_workload_percent: 10.0,
        }
    }

    /// Benchmark thresholds - for testing
    #[must_use]
    pub const fn benchmark() -> Self {
        Self {
            micro_workload_percent: 40.0,
            light_workload_percent: 20.0,
            medium_workload_percent: 10.0,
            heavy_workload_percent: 5.0,
        }
    }
}

impl MemoryProfilingConfig {
    /// Production memory profiling
    #[must_use]
    pub const fn production() -> Self {
        Self {
            enabled: true,
            sample_interval: 1000,     // Sample every 1000th allocation
            min_allocation_size: 1024, // Track allocations >= 1KB
            leak_warning_threshold: 500_000_000, // Warn at 500MB
        }
    }

    /// Development memory profiling
    #[must_use]
    pub const fn development() -> Self {
        Self {
            enabled: true,
            sample_interval: 100,                // More frequent sampling
            min_allocation_size: 256,            // Track smaller allocations
            leak_warning_threshold: 100_000_000, // Warn at 100MB
        }
    }

    /// Disabled memory profiling
    #[must_use]
    pub const fn disabled() -> Self {
        Self {
            enabled: false,
            sample_interval: usize::MAX,
            min_allocation_size: usize::MAX,
            leak_warning_threshold: usize::MAX,
        }
    }
}

impl Default for ProfilingConfig {
    fn default() -> Self {
        Self::production()
    }
}

/// Profiling metrics for adaptive behavior
#[derive(Debug, Clone)]
pub struct ProfilingMetrics {
    /// Observed overhead percentage
    pub overhead_percent: f64,

    /// Number of samples collected
    pub samples_collected: usize,

    /// Duration of profiled workload
    pub workload_duration: Duration,

    /// Current sampling rate
    pub current_sample_rate_hz: u32,

    /// Memory allocated during profiling
    pub memory_allocated_bytes: usize,
}

impl ProfilingMetrics {
    /// Check if overhead is acceptable for workload
    #[must_use]
    pub fn is_overhead_acceptable(&self, config: &ProfilingConfig) -> bool {
        let threshold = config.get_overhead_threshold(self.workload_duration);
        self.overhead_percent <= threshold
    }

    /// Get recommended sample rate based on metrics
    #[must_use]
    pub fn recommended_sample_rate(&self, config: &ProfilingConfig) -> u32 {
        config.calculate_adaptive_rate(self.overhead_percent)
    }
}
