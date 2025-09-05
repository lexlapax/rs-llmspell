//! In-process kernel implementation for embedded execution
//!
//! This module provides an in-process kernel that runs directly within the CLI process,
//! avoiding the overhead of ZeroMQ communication for local script execution.

use anyhow::Result;
use async_trait::async_trait;
use llmspell_config::LLMSpellConfig;
use llmspell_kernel::{
    kernel::{GenericKernel, KernelState},
    traits::{null::NullProtocol, null::NullTransport},
};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use super::{ExecuteResult, KernelConnectionTrait};
use llmspell_bridge::hook_profiler::WorkloadClassifier;

/// In-process kernel that embeds the kernel directly without network communication
pub struct InProcessKernel {
    /// The embedded kernel instance using null transport/protocol
    kernel: Arc<RwLock<GenericKernel<NullTransport, NullProtocol>>>,
    /// Connection state (always true for in-process)
    connected: bool,
    /// Configuration
    _config: Arc<LLMSpellConfig>,
}

impl InProcessKernel {
    /// Create a new in-process kernel
    pub async fn new(config: Arc<LLMSpellConfig>) -> Result<Self> {
        let kernel_id = Uuid::new_v4().to_string();

        // Create null transport and protocol for in-process execution
        let transport = NullTransport::new();
        let protocol = NullProtocol::new();

        // Create the embedded kernel
        let kernel =
            GenericKernel::new(kernel_id.clone(), config.clone(), transport, protocol).await?;

        // Set kernel to idle state
        {
            let mut state = kernel.execution_state.write().await;
            *state = KernelState::Idle;
        }

        Ok(Self {
            kernel: Arc::new(RwLock::new(kernel)),
            connected: true,
            _config: config,
        })
    }

    /// Execute code directly on the embedded kernel
    async fn execute_internal(&self, code: &str, _silent: bool) -> Result<ExecuteResult> {
        tracing::info!("InProcessKernel::execute_internal - Starting execution");
        tracing::info!("InProcessKernel - Code to execute: {}", code);

        // Get the kernel and access its runtime directly
        let kernel = self.kernel.read().await;
        tracing::info!("InProcessKernel - Got kernel reference, accessing ScriptRuntime directly");

        // Set kernel to busy state
        {
            let mut state = kernel.execution_state.write().await;
            *state = KernelState::Busy;
            tracing::info!("InProcessKernel - Set kernel state to BUSY");
        }

        // Execute code directly on the ScriptRuntime
        tracing::info!("InProcessKernel - Calling ScriptRuntime::execute_script (NO ZeroMQ!)");
        let result = {
            let runtime = kernel.runtime.lock().await;
            runtime.execute_script(code).await
        };

        // Set kernel back to idle state
        {
            let mut state = kernel.execution_state.write().await;
            *state = KernelState::Idle;
            tracing::info!("InProcessKernel - Set kernel state back to IDLE");
        }

        // Process the result
        match result {
            Ok(script_result) => {
                tracing::info!("InProcessKernel - Script executed successfully!");
                tracing::info!(
                    "InProcessKernel - Console output: {:?}",
                    script_result.console_output
                );
                tracing::info!("InProcessKernel - Return value: {:?}", script_result.output);

                // Extract output from the ScriptResult
                let output = if !script_result.console_output.is_empty() {
                    // Join console output lines
                    Some(script_result.console_output.join("\n"))
                } else if !script_result.output.is_null() {
                    // Use the main output value
                    Some(script_result.output.to_string())
                } else {
                    Some(String::new())
                };

                tracing::info!("InProcessKernel - Final output to return: {:?}", output);

                Ok(ExecuteResult {
                    success: true,
                    output,
                    error: None,
                })
            }
            Err(e) => {
                tracing::error!("InProcessKernel - Script execution failed: {}", e);
                Ok(ExecuteResult {
                    success: false,
                    output: None,
                    error: Some(e.to_string()),
                })
            }
        }
    }
}

#[async_trait]
impl KernelConnectionTrait for InProcessKernel {
    async fn connect_or_start(&mut self) -> Result<()> {
        // No-op for in-process kernel - always connected
        Ok(())
    }

    fn is_connected(&self) -> bool {
        // In-process kernel is always connected
        self.connected
    }

    async fn disconnect(&mut self) -> Result<()> {
        // Mark as disconnected
        self.connected = false;

        // Set kernel to stopping state
        {
            let kernel = self.kernel.read().await;
            let mut state = kernel.execution_state.write().await;
            *state = KernelState::Stopping;
        }

        // Note: We don't actually shutdown the kernel since it's embedded
        // The kernel will be dropped when this struct is dropped
        Ok(())
    }

    async fn execute(&mut self, code: &str) -> Result<String> {
        let result = self.execute_internal(code, false).await?;

        if result.success {
            Ok(result.output.unwrap_or_default())
        } else {
            Err(anyhow::anyhow!(
                "Execution failed: {}",
                result.error.unwrap_or_else(|| "Unknown error".to_string())
            ))
        }
    }

    async fn execute_inline(&mut self, code: &str) -> Result<String> {
        // For inline execution, we don't store history
        let result = self.execute_internal(code, true).await?;

        if result.success {
            Ok(result.output.unwrap_or_default())
        } else {
            Err(anyhow::anyhow!(
                "Execution failed: {}",
                result.error.unwrap_or_else(|| "Unknown error".to_string())
            ))
        }
    }

    async fn repl(&mut self) -> Result<()> {
        // REPL is handled by the CLI layer, not here
        anyhow::bail!("REPL should be handled by CLI layer, not kernel connection")
    }

    async fn info(&mut self) -> Result<Value> {
        let kernel = self.kernel.read().await;
        Ok(kernel.handle_kernel_info())
    }

    async fn send_debug_command(&mut self, command: Value) -> Result<Value> {
        // Direct call to kernel (no network overhead for in-process)
        let kernel = self.kernel.read().await;
        
        // Route debug command directly to GenericKernel's handler
        kernel.handle_debug_request(command).await
            .map_err(|e| anyhow::anyhow!("Debug command failed: {}", e))
    }

    fn classify_workload(&self, operation: &str) -> WorkloadClassifier {
        match operation {
            "execute_line" | "tab_complete" => WorkloadClassifier::Light,
            "execute_block" => WorkloadClassifier::Medium,
            "execute_file" => WorkloadClassifier::Heavy,
            _ => WorkloadClassifier::Light,
        }
    }

    fn execution_manager(&self) -> Option<&dyn std::any::Any> {
        // No execution manager for in-process kernel
        None
    }

}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test(flavor = "multi_thread")]
    async fn test_in_process_kernel_creation() {
        let config = Arc::new(LLMSpellConfig::default());
        let kernel = InProcessKernel::new(config).await;
        assert!(
            kernel.is_ok(),
            "Should create in-process kernel successfully"
        );

        let kernel = kernel.unwrap();
        assert!(
            kernel.is_connected(),
            "In-process kernel should be connected"
        );
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_in_process_execution() {
        let config = Arc::new(LLMSpellConfig::default());
        let mut kernel = InProcessKernel::new(config).await.unwrap();

        // Test simple execution
        let result = kernel.execute("print('hello')").await;
        assert!(result.is_ok(), "Should execute code successfully");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_in_process_disconnect() {
        let config = Arc::new(LLMSpellConfig::default());
        let mut kernel = InProcessKernel::new(config).await.unwrap();

        assert!(kernel.is_connected(), "Should be connected initially");

        let result = kernel.disconnect().await;
        assert!(result.is_ok(), "Should disconnect successfully");
        assert!(
            !kernel.is_connected(),
            "Should not be connected after disconnect"
        );
    }
}
