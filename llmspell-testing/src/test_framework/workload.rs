//! Workload classification and adaptation

use super::executor::ExecutionMode;
use std::time::Duration;

/// Workload classification for adaptive test execution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkloadClass {
    /// Micro workload: <100ms, ~100 items
    Micro,
    /// Small workload: <1s, ~1K items
    Small,
    /// Medium workload: <10s, ~10K items
    Medium,
    /// Large workload: <60s, ~100K items
    Large,
    /// Stress workload: Unlimited, ~1M+ items
    Stress,
}

impl WorkloadClass {
    /// Detect workload class from environment
    pub fn from_env() -> Self {
        if std::env::var("WORKLOAD").is_ok() {
            match std::env::var("WORKLOAD").unwrap().to_lowercase().as_str() {
                "micro" => WorkloadClass::Micro,
                "small" => WorkloadClass::Small,
                "medium" => WorkloadClass::Medium,
                "large" => WorkloadClass::Large,
                "stress" => WorkloadClass::Stress,
                _ => WorkloadClass::Small,
            }
        } else if std::env::var("CARGO_BENCH").is_ok() {
            WorkloadClass::Large
        } else if std::env::var("CI").is_ok() {
            WorkloadClass::Medium
        } else {
            WorkloadClass::Small
        }
    }

    /// Get workload class from execution mode
    pub fn from_mode(mode: ExecutionMode) -> Self {
        match mode {
            ExecutionMode::Test => WorkloadClass::Small,
            ExecutionMode::Bench => WorkloadClass::Large,
            ExecutionMode::Stress => WorkloadClass::Stress,
            ExecutionMode::CI => WorkloadClass::Medium,
        }
    }

    /// Get the number of events/items for this workload
    pub fn event_count(&self) -> usize {
        match self {
            Self::Micro => 100,
            Self::Small => 1_000,
            Self::Medium => 10_000,
            Self::Large => 100_000,
            Self::Stress => 1_000_000,
        }
    }

    /// Get the timeout for this workload
    pub fn timeout(&self) -> Duration {
        match self {
            Self::Micro => Duration::from_millis(100),
            Self::Small => Duration::from_secs(1),
            Self::Medium => Duration::from_secs(10),
            Self::Large => Duration::from_secs(60),
            Self::Stress => Duration::from_secs(300),
        }
    }

    /// Get the maximum overhead percentage for this workload
    pub fn max_overhead_percent(&self) -> f64 {
        match self {
            Self::Micro => 50.0, // Allow high overhead for tiny workloads
            Self::Small => 30.0,
            Self::Medium => 15.0,
            Self::Large => 10.0,
            Self::Stress => 5.0, // Strict for stress tests
        }
    }

    /// Get batch size for processing
    pub fn batch_size(&self) -> usize {
        match self {
            Self::Micro => 10,
            Self::Small => 100,
            Self::Medium => 1_000,
            Self::Large => 10_000,
            Self::Stress => 100_000,
        }
    }

    /// Check if this is a lightweight workload
    pub fn is_lightweight(&self) -> bool {
        matches!(self, Self::Micro | Self::Small)
    }

    /// Check if this is a heavyweight workload
    pub fn is_heavyweight(&self) -> bool {
        matches!(self, Self::Large | Self::Stress)
    }
}

/// Trait for types that can adapt their workload
pub trait WorkloadAdapter {
    /// Adapt the workload based on system capabilities
    fn adapt(&mut self, class: WorkloadClass);

    /// Get the current workload class
    fn current_workload(&self) -> WorkloadClass;
}

/// Simple workload configuration
#[derive(Debug, Clone)]
pub struct WorkloadConfig {
    pub class: WorkloadClass,
    pub event_count: usize,
    pub batch_size: usize,
    pub timeout: Duration,
}

impl WorkloadConfig {
    /// Create a new workload configuration
    pub fn new(class: WorkloadClass) -> Self {
        Self {
            class,
            event_count: class.event_count(),
            batch_size: class.batch_size(),
            timeout: class.timeout(),
        }
    }

    /// Create from environment
    pub fn from_env() -> Self {
        Self::new(WorkloadClass::from_env())
    }

    /// Scale the workload by a factor
    pub fn scale(mut self, factor: f64) -> Self {
        self.event_count = (self.event_count as f64 * factor) as usize;
        self.batch_size = (self.batch_size as f64 * factor) as usize;
        self
    }
}

impl WorkloadAdapter for WorkloadConfig {
    fn adapt(&mut self, class: WorkloadClass) {
        self.class = class;
        self.event_count = class.event_count();
        self.batch_size = class.batch_size();
        self.timeout = class.timeout();
    }

    fn current_workload(&self) -> WorkloadClass {
        self.class
    }
}
