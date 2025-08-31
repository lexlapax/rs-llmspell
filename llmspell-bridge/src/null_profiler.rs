//! Null profiler implementation for testing
//!
//! Provides a no-op profiler that implements the Profiler trait
//! without any actual profiling functionality. Safe for use in tests.

use crate::profiler::Profiler;
use std::error::Error;
use std::time::Instant;

/// Null profiler that does nothing (for testing)
pub struct NullProfiler {
    active: bool,
    start_time: Option<Instant>,
}

impl NullProfiler {
    /// Create a new null profiler
    #[must_use]
    pub const fn new() -> Self {
        Self {
            active: false,
            start_time: None,
        }
    }
}

impl Default for NullProfiler {
    fn default() -> Self {
        Self::new()
    }
}

impl Profiler for NullProfiler {
    fn start(&mut self, _sample_rate: i32) -> Result<(), Box<dyn Error>> {
        if self.active {
            return Err("Profiling is already active".into());
        }
        self.active = true;
        self.start_time = Some(Instant::now());
        Ok(())
    }

    fn stop(&mut self) -> Result<Vec<u8>, Box<dyn Error>> {
        if !self.active {
            return Err("Profiling is not active".into());
        }
        self.active = false;
        self.start_time = None;
        // Return minimal valid data
        Ok(vec![])
    }

    fn is_active(&self) -> bool {
        self.active
    }

    fn start_time(&self) -> Option<Instant> {
        self.start_time
    }
}
