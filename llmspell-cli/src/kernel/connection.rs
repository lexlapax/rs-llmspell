//! Kernel connection management for CLI client
//!
//! Phase 9.8: All execution goes through Jupyter kernel.
//! Old custom protocols have been removed.

use anyhow::Result;
use async_trait::async_trait;
use llmspell_bridge::{
    circuit_breaker::{CircuitBreaker, OperationContext},
    hook_profiler::WorkloadClassifier,
    session_recorder::SessionRecorder,
};
use llmspell_kernel::KernelDiscovery;
use llmspell_repl::ConnectionInfo;

// Import Jupyter ConnectionInfo with an alias to avoid conflicts
use llmspell_kernel::ConnectionInfo as JupyterConnectionInfo;
use serde_json::Value;
use std::collections::HashMap;
use std::process::Stdio;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::process::Command;
use tokio::sync::RwLock;
use tokio::time::sleep;

/// Connection format enum for Legacy vs Jupyter formats
#[derive(Debug, Clone)]
pub enum ConnectionFormat {
    /// Current format from llmspell-repl
    Legacy(ConnectionInfo),
    /// Future format from llmspell-kernel
    Jupyter(JupyterConnectionInfo),
}

impl ConnectionFormat {
    /// Extract the kernel ID regardless of format
    pub fn kernel_id(&self) -> &str {
        match self {
            ConnectionFormat::Legacy(info) => &info.kernel_id,
            ConnectionFormat::Jupyter(info) => &info.kernel_id,
        }
    }

    /// Extract the IP address regardless of format
    pub fn ip(&self) -> &str {
        match self {
            ConnectionFormat::Legacy(info) => &info.ip,
            ConnectionFormat::Jupyter(info) => &info.ip,
        }
    }

    /// Extract the shell port regardless of format  
    pub fn shell_port(&self) -> u16 {
        match self {
            ConnectionFormat::Legacy(info) => info.shell_port,
            ConnectionFormat::Jupyter(info) => info.shell_port,
        }
    }

    /// Convert to legacy format (for now)
    pub fn to_legacy(self) -> ConnectionInfo {
        match self {
            ConnectionFormat::Legacy(info) => info,
            ConnectionFormat::Jupyter(jupyter_info) => {
                // Convert Jupyter format to Legacy format
                ConnectionInfo::new(
                    jupyter_info.kernel_id,
                    jupyter_info.ip,
                    jupyter_info.shell_port,
                )
            }
        }
    }

    /// Detect format from file content
    pub async fn from_file(path: &std::path::Path) -> Result<Self> {
        let content = tokio::fs::read_to_string(path).await?;

        // Try to parse as Jupyter format first
        if let Ok(jupyter_info) = serde_json::from_str::<JupyterConnectionInfo>(&content) {
            return Ok(ConnectionFormat::Jupyter(jupyter_info));
        }

        // Fall back to legacy format
        if let Ok(legacy_info) = serde_json::from_str::<ConnectionInfo>(&content) {
            return Ok(ConnectionFormat::Legacy(legacy_info));
        }

        anyhow::bail!("Could not parse connection file as either Legacy or Jupyter format")
    }
}

/// Kernel client for Jupyter protocol communication
pub struct KernelClient;

impl KernelClient {
    /// Connect to kernel via Jupyter protocol
    pub async fn connect(_addr: &str) -> Result<Self> {
        // Phase 9.8: Jupyter protocol connection to be implemented
        anyhow::bail!("Kernel connection through Jupyter protocol not yet implemented")
    }

    /// Execute code through the kernel
    pub async fn execute(&self, _code: &str) -> Result<ExecuteResult> {
        anyhow::bail!("Execute through Jupyter protocol not yet implemented")
    }

    /// Health check for the kernel
    pub async fn health_check(&self) -> Result<bool> {
        Ok(false)
    }

    /// Shutdown the kernel connection
    pub fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}

/// Execution result from kernel
#[derive(Debug, Clone)]
pub struct ExecuteResult {
    pub success: bool,
    pub output: Option<String>,
    pub error: Option<String>,
}

/// Trait for kernel connection implementations
#[async_trait]
pub trait KernelConnectionTrait: Send + Sync {
    /// Execute code synchronously
    async fn execute(&mut self, code: &str) -> Result<String>;

    /// Execute code inline (for exec command)
    async fn execute_inline(&mut self, code: &str) -> Result<String>;

    /// Start an interactive REPL session
    async fn repl(&mut self) -> Result<()>;

    /// Get kernel info
    async fn info(&mut self) -> Result<Value>;

    /// Send debug command
    async fn send_debug_command(&mut self, command: Value) -> Result<Value>;

    /// Shutdown the kernel
    async fn shutdown(&mut self) -> Result<()>;
}

/// Discovery mechanism for finding kernels
pub trait CliKernelDiscoveryTrait: Send + Sync {
    /// Find a running kernel by ID
    fn find_kernel(&self, kernel_id: &str) -> Option<ConnectionInfo>;

    /// List all running kernels
    fn list_kernels(&self) -> Vec<ConnectionInfo>;

    /// Auto-start a kernel if needed
    fn auto_start_kernel(&self) -> Result<ConnectionInfo>;
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

/// Kernel discovery implementation
pub struct CliKernelDiscovery {
    discovery: Arc<RwLock<KernelDiscovery>>,
    runtime_path: Option<std::path::PathBuf>,
}

impl CliKernelDiscovery {
    pub fn new() -> Self {
        Self {
            discovery: Arc::new(RwLock::new(KernelDiscovery::new())),
            runtime_path: None,
        }
    }
}

impl CliKernelDiscoveryTrait for CliKernelDiscovery {
    fn find_kernel(&self, kernel_id: &str) -> Option<ConnectionInfo> {
        let discovery = futures::executor::block_on(self.discovery.read());
        discovery
            .list_kernels()
            .into_iter()
            .find(|k| k.kernel_id == kernel_id)
    }

    fn list_kernels(&self) -> Vec<ConnectionInfo> {
        let discovery = futures::executor::block_on(self.discovery.read());
        discovery.list_kernels()
    }

    fn auto_start_kernel(&self) -> Result<ConnectionInfo> {
        // Find kernel binary
        let kernel_binary = find_kernel_binary()?;

        // Start kernel process
        let kernel_id = uuid::Uuid::new_v4().to_string();
        let port = 9555; // Default port

        let mut cmd = Command::new(&kernel_binary);
        cmd.arg("--kernel-id")
            .arg(&kernel_id)
            .arg("--port")
            .arg(port.to_string())
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Spawn kernel
        let child = futures::executor::block_on(cmd.spawn())?;

        // Store process handle (in real implementation)
        let _ = child;

        // Return connection info
        Ok(ConnectionInfo::new(
            kernel_id,
            "127.0.0.1".to_string(),
            port,
        ))
    }
}

/// Find the kernel binary
fn find_kernel_binary() -> Result<std::path::PathBuf> {
    // Look for llmspell-kernel in PATH
    which::which("llmspell-kernel").or_else(|_| {
        // Check target directory
        let mut path = std::env::current_exe()?;
        path.pop();
        path.push("llmspell-kernel");
        if path.exists() {
            Ok(path)
        } else {
            anyhow::bail!("Could not find llmspell-kernel binary")
        }
    })
}

/// CLI circuit breaker implementation
pub struct CliCircuitBreaker {
    breaker: Arc<CircuitBreaker>,
}

impl CliCircuitBreaker {
    pub fn new() -> Self {
        Self {
            breaker: Arc::new(CircuitBreaker::new(
                3,                       // max_failures
                Duration::from_secs(60), // reset_timeout
                Duration::from_secs(30), // timeout
            )),
        }
    }
}

impl CliCircuitBreakerTrait for CliCircuitBreaker {
    fn execute<'a>(&'a self, operation: &'a str) -> Result<()> {
        let context = OperationContext {
            operation: operation.to_string(),
            workload: WorkloadClassifier::classify_workload(operation),
        };

        futures::executor::block_on(async { self.breaker.call(context, async { Ok(()) }).await })
    }

    fn is_open(&self) -> bool {
        self.breaker.is_open()
    }

    fn reset(&self) {
        self.breaker.reset()
    }
}

/// Builder for kernel connections
pub struct KernelConnectionBuilder {
    discovery: Option<Box<dyn CliKernelDiscoveryTrait>>,
    circuit_breaker: Option<Box<dyn CliCircuitBreakerTrait>>,
    connection_timeout: Duration,
}

impl KernelConnectionBuilder {
    pub fn new() -> Self {
        Self {
            discovery: None,
            circuit_breaker: None,
            connection_timeout: Duration::from_secs(10),
        }
    }

    pub fn discovery(mut self, discovery: Box<dyn CliKernelDiscoveryTrait>) -> Self {
        self.discovery = Some(discovery);
        self
    }

    pub fn circuit_breaker(mut self, breaker: Box<dyn CliCircuitBreakerTrait>) -> Self {
        self.circuit_breaker = Some(breaker);
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.connection_timeout = timeout;
        self
    }

    pub async fn build(self) -> Result<Box<dyn KernelConnectionTrait>> {
        // For now, return error as Jupyter implementation not ready
        anyhow::bail!("Kernel connection not yet implemented. Use llmspell-kernel directly.")
    }
}

/// Legacy protocol kernel connection (temporary during migration)
struct ProtocolKernelConnection {
    connection_info: ConnectionInfo,
}

#[async_trait]
impl KernelConnectionTrait for ProtocolKernelConnection {
    async fn execute(&mut self, _code: &str) -> Result<String> {
        anyhow::bail!("Protocol connection removed. Use Jupyter kernel.")
    }

    async fn execute_inline(&mut self, _code: &str) -> Result<String> {
        anyhow::bail!("Protocol connection removed. Use Jupyter kernel.")
    }

    async fn repl(&mut self) -> Result<()> {
        anyhow::bail!("Protocol connection removed. Use Jupyter kernel.")
    }

    async fn info(&mut self) -> Result<Value> {
        Ok(serde_json::json!({
            "kernel_id": self.connection_info.kernel_id,
            "status": "Protocol connection removed"
        }))
    }

    async fn send_debug_command(&mut self, _command: Value) -> Result<Value> {
        anyhow::bail!("Protocol connection removed. Use Jupyter kernel.")
    }

    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}

/// Performance monitoring kernel connection wrapper
pub struct MonitoredKernelConnection<T: KernelConnectionTrait> {
    inner: T,
    recorder: Arc<SessionRecorder>,
    start_time: Instant,
}

impl<T: KernelConnectionTrait> MonitoredKernelConnection<T> {
    pub fn new(inner: T, recorder: Arc<SessionRecorder>) -> Self {
        Self {
            inner,
            recorder,
            start_time: Instant::now(),
        }
    }

    fn record_event(&self, _event_type: &str, _data: Value) {
        // SessionEvent is an enum with specific variants
        // Monitoring removed - would need proper event variant
    }
}

#[async_trait]
impl<T: KernelConnectionTrait> KernelConnectionTrait for MonitoredKernelConnection<T> {
    async fn execute(&mut self, code: &str) -> Result<String> {
        self.record_event("execute_start", serde_json::json!({ "code": code }));
        let result = self.inner.execute(code).await;
        self.record_event(
            "execute_end",
            serde_json::json!({ "success": result.is_ok() }),
        );
        result
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

    async fn shutdown(&mut self) -> Result<()> {
        self.inner.shutdown().await
    }
}
