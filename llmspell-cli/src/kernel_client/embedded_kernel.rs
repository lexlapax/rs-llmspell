//! Embedded kernel that spawns GenericKernel in background thread
//!
//! This provides the benefits of the Jupyter protocol while running in the same process.
//! The kernel runs in a background thread and communicates via localhost ZeroMQ.

use anyhow::Result;
use async_trait::async_trait;
use llmspell_bridge::hook_profiler::WorkloadClassifier;
use llmspell_config::LLMSpellConfig;
use llmspell_kernel::{
    connection::ConnectionInfo, JupyterClient, JupyterKernel, JupyterProtocol, ZmqTransport,
};
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use uuid::Uuid;

use super::KernelConnectionTrait;

/// Find an available port for the kernel to bind to
async fn find_available_port() -> Result<u16> {
    // Try common ports first
    for port in [9555, 9556, 9557, 9558, 9559] {
        if is_port_available(port).await {
            return Ok(port);
        }
    }

    // Fall back to random port
    for _ in 0..10 {
        let port = 9500 + (rand::random::<u16>() % 500);
        if is_port_available(port).await {
            return Ok(port);
        }
    }

    anyhow::bail!("Could not find available port for embedded kernel")
}

async fn is_port_available(port: u16) -> bool {
    tokio::net::TcpListener::bind(("127.0.0.1", port))
        .await
        .is_ok()
}

/// Embedded kernel that runs in the same process
pub struct EmbeddedKernel {
    /// Handle to the kernel thread
    kernel_thread: Option<JoinHandle<Result<()>>>,
    /// The kernel ID
    kernel_id: String,
    /// Connection info for the kernel
    connection_info: ConnectionInfo,
    /// The client for communicating with the kernel
    client: Option<JupyterClient>,
    /// Whether the kernel is running
    running: bool,
    /// Shutdown sender
    shutdown_tx: Option<oneshot::Sender<()>>,
}

impl EmbeddedKernel {
    /// Create a new embedded kernel
    pub async fn new(config: Arc<LLMSpellConfig>) -> Result<Self> {
        let kernel_id = Uuid::new_v4().to_string();
        let port = find_available_port().await?;

        // Create connection info
        let connection_info = ConnectionInfo::new(kernel_id.clone(), "127.0.0.1".to_string(), port);

        // Create shutdown channel
        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        // Clone values for the thread
        let thread_kernel_id = kernel_id.clone();
        let thread_config = config.clone();
        let thread_conn_info = connection_info.clone();

        // Spawn kernel in background thread
        let kernel_thread = tokio::spawn(async move {
            tracing::info!(
                "Starting embedded kernel {} on port {}",
                thread_kernel_id,
                port
            );

            // Create transport and protocol
            let transport = ZmqTransport::new()?;
            let protocol = JupyterProtocol::new(thread_conn_info.clone());

            // Create and run kernel
            let mut kernel =
                JupyterKernel::new(thread_kernel_id.clone(), thread_config, transport, protocol)
                    .await?;

            // Run kernel with shutdown signal
            tokio::select! {
                result = kernel.serve() => {
                    tracing::info!("Embedded kernel {} stopped", thread_kernel_id);
                    result
                }
                _ = shutdown_rx => {
                    tracing::info!("Embedded kernel {} shutting down", thread_kernel_id);
                    Ok(())
                }
            }
        });

        // Wait briefly for kernel to start
        tokio::time::sleep(Duration::from_millis(200)).await;

        // Create client and connect to the kernel
        let transport = ZmqTransport::new()?;
        let protocol = JupyterProtocol::new(connection_info.clone());
        let client = JupyterClient::connect(transport, protocol, connection_info.clone()).await?;

        Ok(Self {
            kernel_thread: Some(kernel_thread),
            kernel_id,
            connection_info,
            client: Some(client),
            running: true,
            shutdown_tx: Some(shutdown_tx),
        })
    }
}

#[async_trait]
impl KernelConnectionTrait for EmbeddedKernel {
    async fn connect_or_start(&mut self) -> Result<()> {
        // Already started in new()
        if !self.running {
            anyhow::bail!("Embedded kernel is not running");
        }
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.running
    }

    async fn disconnect(&mut self) -> Result<()> {
        // Shutdown the client first
        if let Some(mut client) = self.client.take() {
            // Try to send shutdown request (ignore errors if kernel already stopped)
            let _ = client.shutdown(false).await;
        }

        // Send shutdown signal to kernel thread
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let _ = shutdown_tx.send(());
        }

        // Wait for kernel thread to finish
        if let Some(handle) = self.kernel_thread.take() {
            match tokio::time::timeout(Duration::from_secs(5), handle).await {
                Ok(Ok(Ok(()))) => {
                    tracing::info!("Embedded kernel shut down cleanly");
                }
                Ok(Ok(Err(e))) => {
                    tracing::warn!("Embedded kernel shutdown error: {}", e);
                }
                Ok(Err(e)) => {
                    tracing::error!("Embedded kernel panicked: {:?}", e);
                }
                Err(_) => {
                    tracing::warn!("Embedded kernel shutdown timed out");
                }
            }
        }

        self.running = false;
        Ok(())
    }

    async fn execute(&mut self, code: &str) -> Result<String> {
        self.execute_with_args(code, vec![]).await
    }

    async fn execute_with_args(&mut self, code: &str, args: Vec<String>) -> Result<String> {
        // Use the client to execute code via ZeroMQ
        let client = self
            .client
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Client not initialized"))?;

        let result = if args.is_empty() {
            client.execute(code).await?
        } else {
            client.execute_with_args(code, args).await?
        };

        // The kernel already printed to stdout via ScriptRuntime
        // Return empty string to avoid double printing
        // Check if there was an error in the execution
        if let llmspell_kernel::jupyter::protocol::MessageContent::ExecuteReply {
            status,
            ename,
            evalue,
            ..
        } = result
        {
            use llmspell_kernel::jupyter::protocol::ExecutionStatus;
            match status {
                ExecutionStatus::Ok => Ok(String::new()),
                ExecutionStatus::Error | ExecutionStatus::Aborted => {
                    let error_msg = format!(
                        "{}: {}",
                        ename.unwrap_or_else(|| "Error".to_string()),
                        evalue.unwrap_or_else(|| "Unknown error".to_string())
                    );
                    Err(anyhow::anyhow!(error_msg))
                }
            }
        } else {
            Ok(String::new())
        }
    }

    async fn execute_inline(&mut self, code: &str) -> Result<String> {
        // Same as execute but silent
        self.execute(code).await
    }

    async fn repl(&mut self) -> Result<()> {
        // REPL is handled by CLI layer
        anyhow::bail!("REPL should be handled by CLI layer, not kernel connection")
    }

    async fn info(&mut self) -> Result<Value> {
        Ok(serde_json::json!({
            "kernel_id": self.kernel_id,
            "status": if self.running { "running" } else { "stopped" },
            "connection": {
                "transport": "tcp",
                "ip": self.connection_info.ip,
                "shell_port": self.connection_info.shell_port,
                "iopub_port": self.connection_info.iopub_port,
                "stdin_port": self.connection_info.stdin_port,
                "control_port": self.connection_info.control_port,
                "hb_port": self.connection_info.hb_port,
            }
        }))
    }

    async fn send_debug_command(&mut self, command: Value) -> Result<Value> {
        // Route debug command through the Jupyter client
        let client = self
            .client
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Kernel client not connected"))?;

        // Send debug request and get reply
        let reply = client.debug_request(command).await?;

        // Convert MessageContent to Value
        // The debug_reply content should already be the DAP response
        match reply {
            llmspell_kernel::jupyter::protocol::MessageContent::DebugReply {
                body,
                success,
                command: _,
                request_seq: _,
                seq: _,
                message,
            } => {
                if !success {
                    if let Some(msg) = message {
                        anyhow::bail!("Debug request failed: {}", msg);
                    } else {
                        anyhow::bail!("Debug request failed");
                    }
                }
                Ok(body.unwrap_or_else(|| serde_json::json!({})))
            }
            _ => {
                anyhow::bail!("Unexpected reply type from debug request")
            }
        }
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
        // No direct access to execution manager in embedded kernel
        None
    }
}

impl Drop for EmbeddedKernel {
    fn drop(&mut self) {
        // Ensure kernel is shut down
        if self.running {
            if let Some(shutdown_tx) = self.shutdown_tx.take() {
                let _ = shutdown_tx.send(());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_embedded_kernel_creation() {
        let config = Arc::new(LLMSpellConfig::default());
        let kernel = EmbeddedKernel::new(config).await;
        assert!(kernel.is_ok(), "Should create embedded kernel successfully");

        let mut kernel = kernel.unwrap();
        assert!(kernel.is_connected(), "Should be connected after creation");

        // Clean shutdown
        let result = kernel.disconnect().await;
        assert!(result.is_ok(), "Should disconnect cleanly");
    }

    #[tokio::test]
    async fn test_embedded_kernel_execution() {
        let config = Arc::new(LLMSpellConfig {
            default_engine: "lua".to_string(),
            ..LLMSpellConfig::default()
        });

        let mut kernel = EmbeddedKernel::new(config).await.unwrap();

        // Test simple execution
        let result = kernel.execute("return 42").await;
        assert!(result.is_ok(), "Should execute code successfully");

        // Clean shutdown
        kernel.disconnect().await.unwrap();
    }
}
