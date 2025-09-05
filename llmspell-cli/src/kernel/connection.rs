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
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::process::Command;
use tokio::sync::RwLock;

/// Classify operation workload based on operation name
fn classify_operation_workload(operation: &str) -> WorkloadClassifier {
    match operation {
        "execute_line" | "tab_complete" => WorkloadClassifier::Light,
        "execute_block" => WorkloadClassifier::Medium,
        "execute_file" => WorkloadClassifier::Heavy,
        _ => WorkloadClassifier::Light,
    }
}

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
    /// Connect to existing kernel or start a new one
    async fn connect_or_start(&mut self) -> Result<()>;

    /// Check if currently connected to a kernel
    fn is_connected(&self) -> bool;

    /// Disconnect from the kernel
    async fn disconnect(&mut self) -> Result<()>;

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

    /// Classify workload for performance monitoring
    fn classify_workload(&self, operation: &str) -> WorkloadClassifier;

    /// Get execution manager (optional)
    fn execution_manager(&self) -> Option<&dyn std::any::Any>;

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
        let kernels = futures::executor::block_on(discovery.discover_kernels()).unwrap_or_default();
        kernels.into_iter().find(|k| k.kernel_id == kernel_id)
    }

    fn list_kernels(&self) -> Vec<ConnectionInfo> {
        let discovery = futures::executor::block_on(self.discovery.read());
        futures::executor::block_on(discovery.discover_kernels()).unwrap_or_default()
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
        let child = cmd.spawn()?;

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
    breaker: Arc<Mutex<dyn CircuitBreaker>>,
}

impl CliCircuitBreaker {
    pub fn new() -> Self {
        Self {
            breaker: Arc::new(Mutex::new(llmspell_bridge::circuit_breaker::ExponentialBackoffBreaker::default())),
        }
    }
}

impl CliCircuitBreakerTrait for CliCircuitBreaker {
    fn execute<'a>(&'a self, operation: &'a str) -> Result<()> {
        let context = OperationContext {
            operation_name: operation.to_string(),
            workload: classify_operation_workload(operation),
            duration: Duration::from_secs(0),
            success: true,
        };

        let breaker = self.breaker.lock().unwrap();
        if breaker.allow_operation(&context) {
            Ok(())
        } else {
            anyhow::bail!("Circuit breaker is open for operation: {}", operation)
        }
    }

    fn is_open(&self) -> bool {
        let breaker = self.breaker.lock().unwrap();
        breaker.is_open()
    }

    fn reset(&self) {
        let mut breaker = self.breaker.lock().unwrap();
        breaker.reset()
    }
}

/// Builder for kernel connections
pub struct KernelConnectionBuilder {
    discovery: Option<Box<dyn CliKernelDiscoveryTrait>>,
    circuit_breaker: Option<Box<dyn CliCircuitBreakerTrait>>,
    diagnostics: Option<llmspell_bridge::diagnostics_bridge::DiagnosticsBridge>,
    connection_timeout: Duration,
}

impl KernelConnectionBuilder {
    pub fn new() -> Self {
        Self {
            discovery: None,
            circuit_breaker: None,
            diagnostics: None,
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

    pub fn diagnostics(mut self, diagnostics: llmspell_bridge::diagnostics_bridge::DiagnosticsBridge) -> Self {
        self.diagnostics = Some(diagnostics);
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.connection_timeout = timeout;
        self
    }

    pub async fn build(self) -> Result<Box<dyn KernelConnectionTrait>> {
        // Return basic stub implementation that compiles
        let connection = BasicKernelConnection::new(
            self.discovery.unwrap_or_else(|| Box::new(CliKernelDiscovery::new())),
            self.circuit_breaker.unwrap_or_else(|| Box::new(CliCircuitBreaker::new())),
            self.diagnostics,
        );
        Ok(Box::new(connection))
    }
}

/// Basic kernel connection implementation (Phase 9.8.10 stub)
pub struct BasicKernelConnection {
    discovery: Box<dyn CliKernelDiscoveryTrait>,
    circuit_breaker: Box<dyn CliCircuitBreakerTrait>,
    diagnostics: Option<llmspell_bridge::diagnostics_bridge::DiagnosticsBridge>,
    connected: bool,
}

impl BasicKernelConnection {
    pub fn new(
        discovery: Box<dyn CliKernelDiscoveryTrait>,
        circuit_breaker: Box<dyn CliCircuitBreakerTrait>,
        diagnostics: Option<llmspell_bridge::diagnostics_bridge::DiagnosticsBridge>,
    ) -> Self {
        Self {
            discovery,
            circuit_breaker,
            diagnostics,
            connected: false,
        }
    }
}

#[async_trait]
impl KernelConnectionTrait for BasicKernelConnection {
    async fn connect_or_start(&mut self) -> Result<()> {
        // Phase 9.8.10: Stub implementation - will be replaced with actual Jupyter client
        self.connected = true;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.connected = false;
        Ok(())
    }

    async fn execute(&mut self, _code: &str) -> Result<String> {
        anyhow::bail!("Phase 9.8.10: In-process kernel execution not yet implemented")
    }

    async fn execute_inline(&mut self, _code: &str) -> Result<String> {
        anyhow::bail!("Phase 9.8.10: In-process kernel execution not yet implemented")
    }

    async fn repl(&mut self) -> Result<()> {
        anyhow::bail!("Phase 9.8.10: In-process kernel REPL not yet implemented")
    }

    async fn info(&mut self) -> Result<Value> {
        Ok(serde_json::json!({
            "status": "Phase 9.8.10 stub",
            "connected": self.connected,
            "implementation": "BasicKernelConnection"
        }))
    }

    async fn send_debug_command(&mut self, _command: Value) -> Result<Value> {
        anyhow::bail!("Phase 9.8.10: Debug commands not yet implemented")
    }

    fn classify_workload(&self, operation: &str) -> WorkloadClassifier {
        classify_operation_workload(operation)
    }

    fn execution_manager(&self) -> Option<&dyn std::any::Any> {
        None
    }

    async fn shutdown(&mut self) -> Result<()> {
        self.connected = false;
        Ok(())
    }
}

/// Legacy protocol kernel connection (temporary during migration)
struct ProtocolKernelConnection {
    connection_info: ConnectionInfo,
    connected: bool,
}

impl ProtocolKernelConnection {
    fn new(connection_info: ConnectionInfo) -> Self {
        Self {
            connection_info,
            connected: false,
        }
    }
}

#[async_trait]
impl KernelConnectionTrait for ProtocolKernelConnection {
    async fn connect_or_start(&mut self) -> Result<()> {
        anyhow::bail!("Protocol connection removed. Use Jupyter kernel.")
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.connected = false;
        Ok(())
    }

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

    fn classify_workload(&self, operation: &str) -> WorkloadClassifier {
        classify_operation_workload(operation)
    }

    fn execution_manager(&self) -> Option<&dyn std::any::Any> {
        None
    }

    async fn shutdown(&mut self) -> Result<()> {
        self.connected = false;
        Ok(())
    }
}

/// Performance monitoring kernel connection wrapper
pub struct MonitoredKernelConnection<T: KernelConnectionTrait> {
    inner: T,
    recorder: Arc<dyn SessionRecorder>,
    start_time: Instant,
}

impl<T: KernelConnectionTrait> MonitoredKernelConnection<T> {
    pub fn new(inner: T, recorder: Arc<dyn SessionRecorder>) -> Self {
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
    async fn connect_or_start(&mut self) -> Result<()> {
        self.record_event("connect_start", serde_json::json!({}));
        let result = self.inner.connect_or_start().await;
        self.record_event(
            "connect_end",
            serde_json::json!({ "success": result.is_ok() }),
        );
        result
    }

    fn is_connected(&self) -> bool {
        self.inner.is_connected()
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.record_event("disconnect_start", serde_json::json!({}));
        let result = self.inner.disconnect().await;
        self.record_event(
            "disconnect_end",
            serde_json::json!({ "success": result.is_ok() }),
        );
        result
    }

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

    fn classify_workload(&self, operation: &str) -> WorkloadClassifier {
        self.inner.classify_workload(operation)
    }

    fn execution_manager(&self) -> Option<&dyn std::any::Any> {
        self.inner.execution_manager()
    }

    async fn shutdown(&mut self) -> Result<()> {
        self.inner.shutdown().await
    }
}
