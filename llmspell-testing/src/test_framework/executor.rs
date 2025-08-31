//! Test executor trait and execution context

use super::telemetry::TelemetryCollector;
use super::workload::WorkloadClass;
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;

/// Result type for test execution
pub trait TestResult: Send + Sync {
    /// Check if the test passed
    fn is_success(&self) -> bool;

    /// Get a summary of the result
    fn summary(&self) -> String;

    /// Get detailed metrics if available
    fn metrics(&self) -> Option<serde_json::Value> {
        None
    }
}

/// Execution mode determines workload and timeout behavior
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionMode {
    /// Running as a test (cargo test) - use Small workload
    Test,
    /// Running as a benchmark (cargo bench) - use Large workload
    Bench,
    /// Running as a stress test - use Stress workload
    Stress,
    /// Running in CI environment - use Medium workload
    CI,
}

impl ExecutionMode {
    /// Detect execution mode from environment
    pub fn from_env() -> Self {
        if std::env::var("CARGO_BENCH").is_ok() {
            ExecutionMode::Bench
        } else if std::env::var("CI").is_ok() {
            ExecutionMode::CI
        } else if std::env::var("STRESS_TEST").is_ok() {
            ExecutionMode::Stress
        } else {
            ExecutionMode::Test
        }
    }
}

/// Context provided to test executors
#[derive(Clone)]
pub struct ExecutionContext<C> {
    /// Test-specific configuration
    pub config: C,
    /// Execution mode (test/bench/stress/ci)
    pub mode: ExecutionMode,
    /// Telemetry collector for metrics
    pub telemetry: Arc<TelemetryCollector>,
    /// Optional timeout for the execution
    pub timeout: Option<Duration>,
}

impl<C: Clone> ExecutionContext<C> {
    /// Create a new execution context
    pub fn new(config: C, mode: ExecutionMode) -> Self {
        let workload = WorkloadClass::from_mode(mode);
        Self {
            config,
            mode,
            telemetry: Arc::new(TelemetryCollector::new()),
            timeout: Some(workload.timeout()),
        }
    }

    /// Create a test context with default configuration
    pub fn test_default(config: C) -> Self {
        Self::new(config, ExecutionMode::Test)
    }

    /// Create a benchmark context with default configuration
    pub fn bench_default(config: C) -> Self {
        Self::new(config, ExecutionMode::Bench)
    }

    /// Set a custom timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Remove timeout
    pub fn without_timeout(mut self) -> Self {
        self.timeout = None;
        self
    }
}

/// Trait for unified test execution
#[async_trait]
pub trait TestExecutor: Send + Sync {
    /// Configuration type for this executor
    type Config: Clone + Send + Sync;
    /// Result type for this executor
    type Result: TestResult;

    /// Execute test with automatic workload adaptation
    async fn execute(&self, context: ExecutionContext<Self::Config>) -> Self::Result;

    /// Get default configuration for this executor
    fn default_config(&self) -> Self::Config;

    /// Adapt workload based on execution mode
    fn adapt_workload(&self, mode: ExecutionMode) -> WorkloadClass {
        WorkloadClass::from_mode(mode)
    }

    /// Get a name for this executor
    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }
}
