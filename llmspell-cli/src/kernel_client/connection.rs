//! Kernel connection management for CLI client
//!
//! Phase 9.8.10: Simplified architecture
//! - In-process kernel (default): No discovery needed
//! - External kernel (--connect): User provides connection details

use anyhow::Result;
use async_trait::async_trait;
use llmspell_bridge::{
    circuit_breaker::CircuitBreaker,
    hook_profiler::WorkloadClassifier,
    session_recorder::SessionRecorder,
};
use serde_json::Value;
use std::sync::{Arc, Mutex};
use std::time::Duration;


/// Kernel client entry point
pub struct KernelClient;

impl KernelClient {
    pub fn new() -> Self {
        Self
    }

    pub async fn execute(&mut self, _code: &str) -> Result<ExecuteResult> {
        // This is a placeholder - actual implementation requires external kernel
        Ok(ExecuteResult {
            success: true,
            output: Some(String::new()),
            error: None,
        })
    }
}

/// Result from kernel execution
#[derive(Debug, Clone)]
pub struct ExecuteResult {
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
}

/// Trait for kernel connections
#[async_trait]
pub trait KernelConnectionTrait: Send + Sync {
    /// Connect to kernel or start new one
    async fn connect_or_start(&mut self) -> Result<()>;

    /// Check if connected
    fn is_connected(&self) -> bool;

    /// Disconnect from kernel
    async fn disconnect(&mut self) -> Result<()>;

    /// Execute code
    async fn execute(&mut self, code: &str) -> Result<String>;

    /// Execute inline code
    async fn execute_inline(&mut self, code: &str) -> Result<String>;

    /// Start REPL session
    async fn repl(&mut self) -> Result<()>;

    /// Get kernel info
    async fn info(&mut self) -> Result<Value>;

    /// Send debug command
    async fn send_debug_command(&mut self, command: Value) -> Result<Value>;

    /// Classify workload
    fn classify_workload(&self, operation: &str) -> WorkloadClassifier;

    /// Get execution manager (for debug)
    fn execution_manager(&self) -> Option<&dyn std::any::Any>;
}

/// Circuit breaker for kernel operations
pub trait CliCircuitBreakerTrait: Send + Sync {
    /// Execute operation with circuit breaker protection
    fn execute<'a>(&'a self, operation: &'a str) -> Result<()>;

    /// Check if circuit is open
    fn is_open(&self) -> bool;

    /// Reset the circuit
    fn reset(&self);
}

/// Simple null circuit breaker that always allows operations
struct NullCircuitBreaker {
    config: llmspell_bridge::circuit_breaker::CircuitBreakerConfig,
}

impl NullCircuitBreaker {
    fn new() -> Self {
        Self {
            config: llmspell_bridge::circuit_breaker::CircuitBreakerConfig::default(),
        }
    }
}

impl CircuitBreaker for NullCircuitBreaker {
    fn allow_operation(&self, _context: &llmspell_bridge::circuit_breaker::OperationContext) -> bool {
        true // Always allow
    }

    fn record_operation(&mut self, _context: llmspell_bridge::circuit_breaker::OperationContext) {
        // No-op
    }

    fn trip(&mut self) {
        // No-op
    }

    fn reset(&mut self) {
        // No-op
    }

    fn state(&self) -> llmspell_bridge::circuit_breaker::CircuitState {
        llmspell_bridge::circuit_breaker::CircuitState::Closed
    }

    fn config(&self) -> &llmspell_bridge::circuit_breaker::CircuitBreakerConfig {
        &self.config
    }

    fn report(&self) -> llmspell_bridge::circuit_breaker::CircuitBreakerReport {
        llmspell_bridge::circuit_breaker::CircuitBreakerReport {
            state: llmspell_bridge::circuit_breaker::CircuitState::Closed,
            error_rate: 0.0,
            operations_count: 0,
            failures_count: 0,
            backoff_duration: None,
            next_attempt: None,
            state_transitions: 0,
        }
    }

    fn adapt_backoff(&mut self, _new_duration: Duration) {
        // No-op
    }
}

/// CLI circuit breaker implementation
pub struct CliCircuitBreaker {
    _breaker: Arc<Mutex<dyn CircuitBreaker>>,
}

impl CliCircuitBreaker {
    pub fn new() -> Self {
        let breaker: Arc<Mutex<dyn CircuitBreaker>> = Arc::new(Mutex::new(NullCircuitBreaker::new()));
        Self { _breaker: breaker }
    }
}

impl Default for CliCircuitBreaker {
    fn default() -> Self {
        Self::new()
    }
}

impl CliCircuitBreakerTrait for CliCircuitBreaker {
    fn execute<'a>(&'a self, _operation: &'a str) -> Result<()> {
        // Circuit breaker just allows all operations for now
        Ok(())
    }

    fn is_open(&self) -> bool {
        false // Circuit is never open in this simple implementation
    }

    fn reset(&self) {
        // No-op for simple implementation
    }
}

/// Kernel connection builder (simplified - mostly for tests)
pub struct KernelConnectionBuilder {
    circuit_breaker: Option<Box<dyn CliCircuitBreakerTrait>>,
    diagnostics: Option<llmspell_bridge::diagnostics_bridge::DiagnosticsBridge>,
    connection_timeout: Duration,
}

impl KernelConnectionBuilder {
    pub fn new() -> Self {
        Self {
            circuit_breaker: None,
            diagnostics: None,
            connection_timeout: Duration::from_secs(10),
        }
    }

    pub fn circuit_breaker(mut self, breaker: Box<dyn CliCircuitBreakerTrait>) -> Self {
        self.circuit_breaker = Some(breaker);
        self
    }

    pub fn diagnostics(
        mut self,
        diagnostics: llmspell_bridge::diagnostics_bridge::DiagnosticsBridge,
    ) -> Self {
        self.diagnostics = Some(diagnostics);
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.connection_timeout = timeout;
        self
    }

    pub async fn build(self) -> Result<Box<dyn KernelConnectionTrait>> {
        // This builder is mainly for tests - real code requires external kernel
        anyhow::bail!("KernelConnectionBuilder is deprecated - use external kernel")
    }
}

impl Default for KernelConnectionBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Monitored kernel connection wrapper for performance tracking
pub struct MonitoredKernelConnection<T: KernelConnectionTrait> {
    inner: T,
    _recorder: Option<Arc<dyn SessionRecorder>>,
}

impl<T: KernelConnectionTrait> MonitoredKernelConnection<T> {
    pub fn new(inner: T, recorder: Option<Arc<dyn SessionRecorder>>) -> Self {
        Self {
            inner,
            _recorder: recorder,
        }
    }

    pub fn inner(&self) -> &T {
        &self.inner
    }

    pub fn inner_mut(&mut self) -> &mut T {
        &mut self.inner
    }
}

#[async_trait]
impl<T: KernelConnectionTrait> KernelConnectionTrait for MonitoredKernelConnection<T> {
    async fn connect_or_start(&mut self) -> Result<()> {
        self.inner.connect_or_start().await
    }

    fn is_connected(&self) -> bool {
        self.inner.is_connected()
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.inner.disconnect().await
    }

    async fn execute(&mut self, code: &str) -> Result<String> {
        self.inner.execute(code).await
    }

    async fn execute_inline(&mut self, code: &str) -> Result<String> {
        self.inner.execute_inline(code).await
    }

    async fn repl(&mut self) -> Result<()> {
        self.inner.repl().await
    }

    async fn info(&mut self) -> Result<Value> {
        self.inner.info().await
    }

    async fn send_debug_command(&mut self, command: Value) -> Result<Value> {
        self.inner.send_debug_command(command).await
    }

    fn classify_workload(&self, operation: &str) -> WorkloadClassifier {
        self.inner.classify_workload(operation)
    }

    fn execution_manager(&self) -> Option<&dyn std::any::Any> {
        self.inner.execution_manager()
    }
}