//! Test helpers for CLI integration testing
//!
//! Provides helper functions for creating test instances with null implementations.

use crate::kernel_connection::{
    KernelConnectionBuilder, NullKernelConnection, NullKernelDiscovery,
};
use crate::repl_interface::CLIReplInterface;
use llmspell_bridge::{
    diagnostics_bridge::DiagnosticsBridge, null_circuit_breaker::NullCircuitBreaker,
    null_hook_profiler::NullHookProfiler, null_profiler::NullProfiler,
    null_session_recorder::NullSessionRecorder,
};
use llmspell_config::LLMSpellConfig;

/// Create a test kernel connection with null implementations
pub fn create_test_kernel() -> NullKernelConnection {
    NullKernelConnection::new()
}

/// Create a test REPL interface with null implementations
pub fn create_test_cli() -> CLIReplInterface {
    let kernel = Box::new(create_test_kernel());

    let diagnostics = DiagnosticsBridge::builder()
        .profiler(Box::new(NullProfiler::new()))
        .hook_profiler(Box::new(NullHookProfiler::new()))
        .circuit_breaker(Box::new(NullCircuitBreaker::new()))
        .session_recorder(Box::new(NullSessionRecorder::new()))
        .build();

    CLIReplInterface::builder()
        .kernel(kernel)
        .diagnostics(diagnostics)
        .build()
        .expect("Failed to create test CLI")
}

/// Create a test kernel connection builder with null implementations
pub fn create_test_kernel_builder() -> KernelConnectionBuilder {
    KernelConnectionBuilder::new()
        .discovery(Box::new(NullKernelDiscovery))
        .circuit_breaker(Box::new(NullCircuitBreaker::new()))
        .session_recorder(Box::new(NullSessionRecorder::new()))
}

/// Create a test configuration
pub fn create_test_config() -> LLMSpellConfig {
    LLMSpellConfig::default()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel_connection::KernelConnectionTrait;

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
        assert_eq!(result.as_str().unwrap(), "null execution");

        // Test disconnection
        kernel.disconnect().await.expect("Failed to disconnect");
        assert!(!kernel.is_connected());
    }

    #[test]
    fn test_create_test_cli() {
        let _cli = create_test_cli();
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
