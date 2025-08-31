//! Null hook profiler implementation for testing
//!
//! Provides a no-op hook profiler that implements the `HookProfiler` trait
//! without any actual profiling functionality. Safe for use in tests.

use crate::hook_profiler::{HookProfiler, HookProfilingConfig, OperationType, ProfileReport};
use std::error::Error;
use std::time::{Duration, Instant};

/// Null hook profiler that does nothing (for testing)
pub struct NullHookProfiler {
    active: bool,
    start_time: Option<Instant>,
    config: Option<HookProfilingConfig>,
    sampling_rate: f64,
}

impl NullHookProfiler {
    /// Create a new null hook profiler
    #[must_use]
    pub const fn new() -> Self {
        Self {
            active: false,
            start_time: None,
            config: None,
            sampling_rate: 1.0,
        }
    }
}

impl Default for NullHookProfiler {
    fn default() -> Self {
        Self::new()
    }
}

impl HookProfiler for NullHookProfiler {
    fn start_profiling(&mut self, config: HookProfilingConfig) -> Result<(), Box<dyn Error>> {
        if self.active {
            return Err("Hook profiling is already active".into());
        }
        self.active = true;
        self.start_time = Some(Instant::now());
        self.config = Some(config);
        Ok(())
    }

    fn stop_profiling(&mut self) -> Result<ProfileReport, Box<dyn Error>> {
        if !self.active {
            return Err("Hook profiling is not active".into());
        }

        self.active = false;
        let start_time = self.start_time.take().unwrap();
        self.config = None;

        // Return minimal valid report
        Ok(ProfileReport {
            hook_samples: std::collections::HashMap::new(),
            duration: start_time.elapsed(),
            overhead_percent: 0.0,
            sample_count: 0,
            sampling_rate: self.sampling_rate,
        })
    }

    fn sample_hook_execution(
        &mut self,
        _hook_name: &str,
        _duration: Duration,
        _op_type: OperationType,
    ) {
        // No-op - do nothing
    }

    fn adapt_sampling_rate(&mut self, _observed_overhead: f64) {
        // No-op - do nothing
    }

    fn is_active(&self) -> bool {
        self.active
    }

    fn sampling_rate(&self) -> f64 {
        self.sampling_rate
    }

    fn config(&self) -> Option<&HookProfilingConfig> {
        self.config.as_ref()
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)] // Test constants are safe for exact comparison
mod tests {
    use super::*;

    #[test]
    fn test_null_hook_profiler_lifecycle() {
        let mut profiler = NullHookProfiler::new();

        // Initially not active
        assert!(!profiler.is_active());
        assert!(profiler.start_time.is_none());

        // Start profiling
        let config = HookProfilingConfig::default();
        assert!(profiler.start_profiling(config).is_ok());
        assert!(profiler.is_active());

        // Cannot start when already active
        assert!(profiler
            .start_profiling(HookProfilingConfig::default())
            .is_err());

        // Sample hook executions (no-op)
        profiler.sample_hook_execution(
            "test_hook",
            Duration::from_millis(1),
            OperationType::Synchronous,
        );
        profiler.adapt_sampling_rate(10.0);

        // Stop profiling
        let result = profiler.stop_profiling();
        assert!(result.is_ok());
        assert!(!profiler.is_active());

        let report = result.unwrap();
        assert!(report.hook_samples.is_empty());
        assert_eq!(report.sample_count, 0);

        // Cannot stop when not active
        assert!(profiler.stop_profiling().is_err());
    }

    #[test]
    fn test_null_hook_profiler_safe_for_tests() {
        // Verify it's safe to use in test scenarios
        let mut profiler = NullHookProfiler::new();
        profiler
            .start_profiling(HookProfilingConfig::default())
            .unwrap();

        // These should all be safe no-ops
        for i in 0..1000 {
            profiler.sample_hook_execution(
                &format!("hook_{i}"),
                Duration::from_millis(i % 10),
                OperationType::Synchronous,
            );
        }

        profiler.adapt_sampling_rate(50.0); // High overhead

        let report = profiler.stop_profiling().unwrap();
        assert_eq!(report.sample_count, 0); // No samples recorded
        assert_eq!(report.overhead_percent, 0.0); // No overhead
    }
}
