//! Test helpers for CLI integration testing
//!
//! Provides helper functions for creating test instances with null implementations.

use crate::kernel_client::{KernelConnectionBuilder, KernelConnectionTrait};
use anyhow::Result;
use async_trait::async_trait;
use llmspell_bridge::{
    diagnostics_bridge::DiagnosticsBridge, hook_profiler::WorkloadClassifier,
    null_circuit_breaker::NullCircuitBreaker, null_hook_profiler::NullHookProfiler,
    null_profiler::NullProfiler, null_session_recorder::NullSessionRecorder,
};
use llmspell_config::LLMSpellConfig;
use serde_json::Value;

/// Null implementation of KernelConnectionTrait for testing
pub struct NullKernelConnection {
    connected: bool,
}

impl NullKernelConnection {
    pub fn new() -> Self {
        Self { connected: false }
    }
}

impl Default for NullKernelConnection {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl KernelConnectionTrait for NullKernelConnection {
    async fn connect_or_start(&mut self) -> Result<()> {
        self.connected = true;
        Ok(())
    }

    async fn execute(&mut self, _code: &str) -> Result<String> {
        Ok("null execution".to_string())
    }

    async fn execute_inline(&mut self, _code: &str) -> Result<String> {
        Ok("null inline execution".to_string())
    }

    async fn repl(&mut self) -> Result<()> {
        Ok(())
    }

    async fn info(&mut self) -> Result<Value> {
        Ok(serde_json::json!({
            "kernel": "null",
            "version": "0.0.0"
        }))
    }

    async fn send_debug_command(&mut self, _command: Value) -> Result<Value> {
        Ok(Value::Null)
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.connected = false;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn classify_workload(&self, _operation: &str) -> WorkloadClassifier {
        WorkloadClassifier::Light
    }

    fn execution_manager(&self) -> Option<&dyn std::any::Any> {
        None
    }
}

/// Create a test kernel connection with null implementations
pub fn create_test_kernel() -> NullKernelConnection {
    NullKernelConnection::new()
}

/// Create a test diagnostics bridge with null implementations
pub fn create_test_diagnostics() -> DiagnosticsBridge {
    DiagnosticsBridge::builder()
        .profiler(Box::new(NullProfiler::new()))
        .hook_profiler(Box::new(NullHookProfiler::new()))
        .circuit_breaker(Box::new(NullCircuitBreaker::new()))
        .session_recorder(Box::new(NullSessionRecorder::new()))
        .build()
}

/// Create a test kernel connection builder with null implementations
pub fn create_test_kernel_builder() -> KernelConnectionBuilder {
    KernelConnectionBuilder::new().diagnostics(create_test_diagnostics())
}

/// Create a test configuration
pub fn create_test_config() -> LLMSpellConfig {
    LLMSpellConfig::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_null_kernel_connection() {
        let mut kernel = create_test_kernel();

        // Test connection
        assert!(!kernel.is_connected());
        kernel.connect_or_start().await.expect("Failed to connect");
        assert!(kernel.is_connected());

        // Test execution
        let result = kernel
            .execute("test code")
            .await
            .expect("Failed to execute");
        assert_eq!(result, "null execution");

        // Test disconnection
        kernel.disconnect().await.expect("Failed to disconnect");
        assert!(!kernel.is_connected());
    }

    #[test]
    fn test_create_test_diagnostics() {
        let _diagnostics = create_test_diagnostics();
        // Just verify it can be created without panicking
    }

    #[test]
    fn test_workload_classification() {
        let kernel = create_test_kernel();
        use llmspell_bridge::hook_profiler::WorkloadClassifier;

        assert_eq!(
            kernel.classify_workload("tab_complete"),
            WorkloadClassifier::Light
        );
        assert_eq!(
            kernel.classify_workload("execute_line"),
            WorkloadClassifier::Light
        );
        assert_eq!(
            kernel.classify_workload("execute_block"),
            WorkloadClassifier::Light
        );
        assert_eq!(
            kernel.classify_workload("execute_file"),
            WorkloadClassifier::Light
        );
    }
}
