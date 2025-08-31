//! Profiler trait abstraction for diagnostics
//!
//! Provides trait-based abstraction for CPU and memory profiling,
//! enabling testability and supporting multiple profiler backends.

use std::error::Error;
use std::time::Instant;

/// Trait for CPU and memory profilers
///
/// This trait defines the interface for profiling implementations,
/// allowing for both real profilers (pprof) and test doubles.
pub trait Profiler: Send + Sync {
    /// Start CPU profiling
    ///
    /// # Arguments
    /// * `sample_rate` - Sampling rate in Hz (samples per second)
    ///
    /// # Errors
    /// Returns error if profiling cannot be started
    fn start(&mut self, sample_rate: i32) -> Result<(), Box<dyn Error>>;

    /// Stop CPU profiling and get report
    ///
    /// # Returns
    /// Profiling report data (e.g., pprof format)
    ///
    /// # Errors
    /// Returns error if profiling is not active
    fn stop(&mut self) -> Result<Vec<u8>, Box<dyn Error>>;

    /// Check if profiling is currently active
    fn is_active(&self) -> bool;

    /// Get profiling start time if active
    fn start_time(&self) -> Option<Instant>;
}

/// Real profiler implementation using pprof
pub struct PprofProfiler {
    guard: Option<Box<pprof::ProfilerGuard<'static>>>,
    start_time: Option<Instant>,
}

impl Default for PprofProfiler {
    fn default() -> Self {
        Self::new()
    }
}

impl PprofProfiler {
    /// Create a new pprof-based profiler
    #[must_use]
    pub const fn new() -> Self {
        Self {
            guard: None,
            start_time: None,
        }
    }
}

impl Profiler for PprofProfiler {
    fn start(&mut self, sample_rate: i32) -> Result<(), Box<dyn Error>> {
        if self.guard.is_some() {
            return Err("Profiling is already active".into());
        }

        let guard = Box::new(
            pprof::ProfilerGuard::new(sample_rate)
                .map_err(|e| format!("Failed to start profiler: {e}"))?,
        );

        self.guard = Some(guard);
        self.start_time = Some(Instant::now());
        Ok(())
    }

    fn stop(&mut self) -> Result<Vec<u8>, Box<dyn Error>> {
        use pprof::protos::Message;

        let guard = self.guard.take().ok_or("Profiling is not active")?;
        self.start_time = None;

        // Build profiling report
        let report = guard
            .report()
            .build()
            .map_err(|e| format!("Failed to build profile report: {e}"))?;

        // Convert to pprof format
        let mut buffer = Vec::new();
        let profile = report
            .pprof()
            .map_err(|e| format!("Failed to generate pprof: {e}"))?;

        // Write protobuf data to buffer
        profile
            .write_to_writer(&mut buffer)
            .map_err(|e| format!("Failed to write pprof data: {e}"))?;

        Ok(buffer)
    }

    fn is_active(&self) -> bool {
        self.guard.is_some()
    }

    fn start_time(&self) -> Option<Instant> {
        self.start_time
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test profiler for unit tests only
    pub struct TestProfiler {
        active: bool,
        start_time: Option<Instant>,
        sample_count: usize,
    }

    impl TestProfiler {
        pub fn new() -> Self {
            Self {
                active: false,
                start_time: None,
                sample_count: 0,
            }
        }

        pub fn add_sample(&mut self) {
            if self.active {
                self.sample_count += 1;
            }
        }
    }

    impl Profiler for TestProfiler {
        fn start(&mut self, _sample_rate: i32) -> Result<(), Box<dyn Error>> {
            if self.active {
                return Err("Profiling is already active".into());
            }
            self.active = true;
            self.start_time = Some(Instant::now());
            self.sample_count = 0;
            Ok(())
        }

        fn stop(&mut self) -> Result<Vec<u8>, Box<dyn Error>> {
            if !self.active {
                return Err("Profiling is not active".into());
            }
            self.active = false;
            self.start_time = None;
            let fake_data = format!("FAKE_PPROF_DATA_SAMPLES_{}", self.sample_count);
            Ok(fake_data.into_bytes())
        }

        fn is_active(&self) -> bool {
            self.active
        }

        fn start_time(&self) -> Option<Instant> {
            self.start_time
        }
    }

    #[test]
    fn test_profiler_lifecycle() {
        let mut profiler = TestProfiler::new();

        // Initially not active
        assert!(!profiler.is_active());
        assert!(profiler.start_time().is_none());

        // Start profiling
        assert!(profiler.start(100).is_ok());
        assert!(profiler.is_active());
        assert!(profiler.start_time().is_some());

        // Cannot start when already active
        assert!(profiler.start(100).is_err());

        // Add some samples
        profiler.add_sample();
        profiler.add_sample();
        assert_eq!(profiler.sample_count, 2);

        // Stop profiling
        let result = profiler.stop();
        assert!(result.is_ok());
        assert!(!profiler.is_active());

        let data = result.unwrap();
        let data_str = String::from_utf8(data).unwrap();
        assert!(data_str.contains("FAKE_PPROF_DATA_SAMPLES_2"));

        // Cannot stop when not active
        assert!(profiler.stop().is_err());
    }

    #[test]
    fn test_pprof_profiler_lifecycle() {
        // Test that PprofProfiler implements the trait correctly
        // Note: We don't actually start it in tests to avoid signal handler issues
        let profiler = PprofProfiler::new();
        assert!(!profiler.is_active());
        assert!(profiler.start_time().is_none());
    }
}
