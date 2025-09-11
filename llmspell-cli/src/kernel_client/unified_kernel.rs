//! Unified kernel client that handles both embedded and external connections
//!
//! This replaces the separate EmbeddedKernel implementation with a single unified approach.

use anyhow::{Context, Result};
use async_trait::async_trait;
use llmspell_bridge::hook_profiler::WorkloadClassifier;
use llmspell_config::LLMSpellConfig;
use llmspell_kernel::{
    ConnectionInfo, JupyterClient, JupyterKernel, JupyterProtocol, KernelDiscovery, ZmqTransport,
};
use serde_json::Value;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tokio::time::Duration;

use super::KernelConnectionTrait;

/// Unified kernel client that handles both embedded and external connections
pub struct UnifiedKernelClient {
    /// The underlying Jupyter client
    client: JupyterClient,
    /// Connection information
    connection_info: ConnectionInfo,
    /// For embedded kernels: handle to the kernel thread
    kernel_thread: Option<JoinHandle<Result<()>>>,
    /// For embedded kernels: shutdown channel
    shutdown_tx: Option<oneshot::Sender<()>>,
    /// Whether this is an embedded kernel
    is_embedded: bool,
}

impl UnifiedKernelClient {
    /// Connect to an external kernel using a connection string
    pub async fn connect_external(connection_string: String) -> Result<Self> {
        // Resolve the connection string to ConnectionInfo
        let connection_info = resolve_connection_string(&connection_string).await?;

        // Check if kernel is alive
        if !KernelDiscovery::is_kernel_alive(&connection_info).await? {
            anyhow::bail!(
                "Kernel {} is not responding to heartbeat",
                connection_info.kernel_id
            );
        }

        // Create client and connect
        let transport = ZmqTransport::new()?;
        let protocol = JupyterProtocol::new(connection_info.clone());
        let client = JupyterClient::connect(transport, protocol, connection_info.clone())
            .await
            .context("Failed to connect to external kernel")?;

        Ok(Self {
            client,
            connection_info,
            kernel_thread: None,
            shutdown_tx: None,
            is_embedded: false,
        })
    }

    /// Start an embedded kernel and connect to it
    pub async fn start_embedded(config: Arc<LLMSpellConfig>) -> Result<Self> {
        // Generate kernel ID
        let kernel_id = uuid::Uuid::new_v4().to_string();

        // Find an available port
        let port = find_available_port().await?;

        // Create connection info
        let mut connection_info =
            ConnectionInfo::new(kernel_id.clone(), "127.0.0.1".to_string(), port);
        connection_info.pid = Some(std::process::id());
        connection_info.started_at = Some(chrono::Utc::now());

        // Create shutdown channel
        let (shutdown_tx, shutdown_rx) = oneshot::channel();

        // Clone for the thread
        let thread_kernel_id = kernel_id.clone();
        let thread_config = config.clone();
        let thread_conn_info = connection_info.clone();

        // Spawn kernel in background
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

        // Wait for kernel to start
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Create client and connect
        let transport = ZmqTransport::new()?;
        let protocol = JupyterProtocol::new(connection_info.clone());
        let client = JupyterClient::connect(transport, protocol, connection_info.clone())
            .await
            .context("Failed to connect to embedded kernel")?;

        Ok(Self {
            client,
            connection_info,
            kernel_thread: Some(kernel_thread),
            shutdown_tx: Some(shutdown_tx),
            is_embedded: true,
        })
    }

    /// Execute code on the kernel
    pub async fn execute(&mut self, code: &str) -> Result<String> {
        let result = self.client.execute(code).await?;

        // Convert MessageContent to Result<String>
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

    /// Execute code with arguments
    pub async fn execute_with_args(&mut self, code: &str, args: Vec<String>) -> Result<String> {
        let result = if args.is_empty() {
            self.client.execute(code).await?
        } else {
            self.client.execute_with_args(code, args).await?
        };

        // Convert MessageContent to Result<String>
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

    /// Send a debug command
    pub async fn send_debug_command(&mut self, command: Value) -> Result<Value> {
        let reply = self.client.debug_request(command).await?;

        // Convert MessageContent to Value
        match reply {
            llmspell_kernel::jupyter::protocol::MessageContent::DebugReply {
                body,
                success,
                message,
                ..
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
            _ => anyhow::bail!("Unexpected reply type from debug request"),
        }
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        // For now, assume we're connected if we have a client
        // In the future, could add heartbeat check
        true
    }

    /// Disconnect from kernel
    pub async fn disconnect(&mut self) -> Result<()> {
        if self.is_embedded {
            // For embedded kernel, send shutdown and wait for thread

            // Try to send shutdown request (ignore errors)
            let _ = self.client.shutdown(false).await;

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
        }
        // For external kernel, we don't shut it down, just disconnect
        Ok(())
    }

    /// Get kernel info
    pub async fn info(&mut self) -> Result<Value> {
        Ok(serde_json::json!({
            "kernel_id": self.connection_info.kernel_id,
            "is_embedded": self.is_embedded,
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
}

impl Drop for UnifiedKernelClient {
    fn drop(&mut self) {
        // Clean shutdown for embedded kernels
        if self.is_embedded {
            if let Some(shutdown_tx) = self.shutdown_tx.take() {
                let _ = shutdown_tx.send(());
            }
        }
    }
}

/// Resolve a connection string to ConnectionInfo
///
/// Supports:
/// - Kernel ID: Looks up via KernelDiscovery
/// - host:port: Finds kernel with matching port
/// - /path/to/file.json: Reads connection file
pub async fn resolve_connection_string(connection: &str) -> Result<ConnectionInfo> {
    if connection.contains(':') && !connection.contains('/') {
        // Looks like host:port
        let parts: Vec<&str> = connection.split(':').collect();
        if parts.len() != 2 {
            anyhow::bail!("Invalid address format. Expected host:port");
        }

        let host = parts[0];
        let port: u16 = parts[1]
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid port number"))?;

        // Must discover the kernel to get the connection info with HMAC key
        let discovery = KernelDiscovery::new();
        let kernels = discovery.discover_kernels().await?;

        // Find kernel with matching port
        let matching_kernel = kernels.iter().find(|k| {
            k.shell_port == port && (k.ip == host || (host == "localhost" && k.ip == "127.0.0.1"))
        });

        match matching_kernel {
            Some(kernel_info) => {
                tracing::info!(
                    "Found kernel {} at {}:{}",
                    kernel_info.kernel_id,
                    host,
                    port
                );
                Ok(kernel_info.clone())
            }
            None => {
                anyhow::bail!(
                    "Could not find kernel at {}:{}. Kernels require HMAC authentication.\n\
                    Use 'llmspell kernel status' to list available kernels, then connect using:\n\
                    - Kernel ID: --connect <kernel-id>\n\
                    - Connection file: --connect /path/to/connection.json",
                    host,
                    port
                );
            }
        }
    } else if connection.contains('/') || connection.ends_with(".json") {
        // Looks like a file path
        let path = PathBuf::from(connection);
        ConnectionInfo::read_connection_file(&path).await
    } else {
        // Assume it's a kernel ID
        let discovery = KernelDiscovery::new();
        discovery
            .find_kernel(connection)
            .await?
            .ok_or_else(|| anyhow::anyhow!("Kernel {} not found", connection))
    }
}

/// Find an available port for the embedded kernel
async fn find_available_port() -> Result<u16> {
    // Try to bind to port 0 to get a random available port
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await?;
    let port = listener.local_addr()?.port();
    drop(listener); // Release the port

    // Add 5 to avoid conflicts with the 5 ZMQ ports
    Ok(port + 5)
}

// Implement KernelConnectionTrait for backward compatibility
#[async_trait]
impl KernelConnectionTrait for UnifiedKernelClient {
    async fn connect_or_start(&mut self) -> Result<()> {
        // Already connected in constructor
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.is_connected()
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.disconnect().await
    }

    async fn execute(&mut self, code: &str) -> Result<String> {
        self.execute(code).await
    }

    async fn execute_with_args(&mut self, code: &str, args: Vec<String>) -> Result<String> {
        self.execute_with_args(code, args).await
    }

    async fn execute_inline(&mut self, code: &str) -> Result<String> {
        // Same as execute
        self.execute(code).await
    }

    async fn repl(&mut self) -> Result<()> {
        // REPL is handled by CLI layer
        anyhow::bail!("REPL should be handled by CLI layer, not kernel connection")
    }

    async fn info(&mut self) -> Result<Value> {
        self.info().await
    }

    async fn send_debug_command(&mut self, command: Value) -> Result<Value> {
        self.send_debug_command(command).await
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
        // No direct access to execution manager in kernel client
        None
    }
}

// Note: Integration tests for UnifiedKernelClient are in tests/kernel_integration.rs
// These require actual kernel infrastructure and should not be unit tests
