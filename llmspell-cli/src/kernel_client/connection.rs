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
use chrono;
use hmac::{Hmac, Mac};
use llmspell_kernel::jupyter::connection::ConnectionInfo as JupyterConnectionInfo;
use serde_json::Value;
use sha2::Sha256;
use std::process::Stdio;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::process::Command;
use tokio::sync::RwLock;
use uuid;
use zmq::{Context as ZmqContext, Socket, SocketType};

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
            ConnectionFormat::Jupyter(info) => &info.kernel_name,
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
                    jupyter_info.kernel_name,
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
            breaker: Arc::new(Mutex::new(
                llmspell_bridge::circuit_breaker::ExponentialBackoffBreaker::default(),
            )),
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
        // Return basic implementation with Jupyter client
        let connection = BasicKernelConnection::new(
            self.discovery
                .unwrap_or_else(|| Box::new(CliKernelDiscovery::new())),
            self.circuit_breaker
                .unwrap_or_else(|| Box::new(CliCircuitBreaker::new())),
            self.diagnostics,
        )?;
        Ok(Box::new(connection))
    }
}

/// Basic kernel connection implementation (Phase 9.8.10 with Jupyter client)
pub struct BasicKernelConnection {
    discovery: Box<dyn CliKernelDiscoveryTrait>,
    circuit_breaker: Box<dyn CliCircuitBreakerTrait>,
    diagnostics: Option<llmspell_bridge::diagnostics_bridge::DiagnosticsBridge>,
    jupyter_client: JupyterKernelClient,
}

impl BasicKernelConnection {
    pub fn new(
        discovery: Box<dyn CliKernelDiscoveryTrait>,
        circuit_breaker: Box<dyn CliCircuitBreakerTrait>,
        diagnostics: Option<llmspell_bridge::diagnostics_bridge::DiagnosticsBridge>,
    ) -> Result<Self> {
        Ok(Self {
            discovery,
            circuit_breaker,
            diagnostics,
            jupyter_client: JupyterKernelClient::new()?,
        })
    }
}

#[async_trait]
impl KernelConnectionTrait for BasicKernelConnection {
    async fn connect_or_start(&mut self) -> Result<()> {
        self.jupyter_client.connect_or_start().await
    }

    fn is_connected(&self) -> bool {
        self.jupyter_client.is_connected()
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.jupyter_client.disconnect().await
    }

    async fn execute(&mut self, code: &str) -> Result<String> {
        self.jupyter_client.execute(code).await
    }

    async fn execute_inline(&mut self, code: &str) -> Result<String> {
        self.jupyter_client.execute_inline(code).await
    }

    async fn repl(&mut self) -> Result<()> {
        self.jupyter_client.repl().await
    }

    async fn info(&mut self) -> Result<Value> {
        let jupyter_info = self.jupyter_client.info().await?;
        Ok(serde_json::json!({
            "status": "BasicKernelConnection with JupyterKernelClient",
            "jupyter_info": jupyter_info,
            "implementation": "BasicKernelConnection"
        }))
    }

    async fn send_debug_command(&mut self, command: Value) -> Result<Value> {
        self.jupyter_client.send_debug_command(command).await
    }

    fn classify_workload(&self, operation: &str) -> WorkloadClassifier {
        classify_operation_workload(operation)
    }

    fn execution_manager(&self) -> Option<&dyn std::any::Any> {
        None
    }

    async fn shutdown(&mut self) -> Result<()> {
        self.jupyter_client.shutdown().await
    }
}

/// Jupyter kernel client using ZeroMQ transport
pub struct JupyterKernelClient {
    context: Arc<Mutex<ZmqContext>>,
    shell_socket: Arc<Mutex<Option<Socket>>>,
    iopub_socket: Arc<Mutex<Option<Socket>>>,
    connection_info: Option<JupyterConnectionInfo>,
    hmac_key: Option<Vec<u8>>,
    connected: bool,
    session_id: String,
    execution_count: u32,
}

impl JupyterKernelClient {
    pub fn new() -> Result<Self> {
        let context = ZmqContext::new();
        Ok(Self {
            context: Arc::new(Mutex::new(context)),
            shell_socket: Arc::new(Mutex::new(None)),
            iopub_socket: Arc::new(Mutex::new(None)),
            connection_info: None,
            hmac_key: None,
            connected: false,
            session_id: uuid::Uuid::new_v4().to_string(),
            execution_count: 0,
        })
    }

    pub fn connect_to_kernel(&mut self, connection_info: JupyterConnectionInfo) -> Result<()> {
        // Parse HMAC key
        let hmac_key = hex::decode(&connection_info.key)?;

        // Create shell socket (REQ to connect to kernel's ROUTER)
        let shell_socket = {
            let context = self.context.lock().unwrap();
            let socket = context.socket(SocketType::REQ)?;
            let addr = format!(
                "tcp://{}:{}",
                connection_info.ip, connection_info.shell_port
            );
            socket.connect(&addr)?;
            socket.set_linger(1000)?; // 1 second linger
            socket.set_rcvtimeo(5000)?; // 5 second timeout
            socket
        };

        // Create iopub socket (SUB to connect to kernel's PUB)
        let iopub_socket = {
            let context = self.context.lock().unwrap();
            let socket = context.socket(SocketType::SUB)?;
            let addr = format!(
                "tcp://{}:{}",
                connection_info.ip, connection_info.iopub_port
            );
            socket.connect(&addr)?;
            socket.set_subscribe(b"")?; // Subscribe to all messages
            socket.set_rcvtimeo(1000)?; // 1 second timeout for non-blocking
            socket
        };

        *self.shell_socket.lock().unwrap() = Some(shell_socket);
        *self.iopub_socket.lock().unwrap() = Some(iopub_socket);
        self.connection_info = Some(connection_info);
        self.hmac_key = Some(hmac_key);
        self.connected = true;

        Ok(())
    }

    pub fn create_message(&self, msg_type: &str, content: Value) -> Result<Vec<Vec<u8>>> {
        let msg_id = uuid::Uuid::new_v4().to_string();
        let timestamp = chrono::Utc::now().to_rfc3339();

        // Create header
        let header = serde_json::json!({
            "msg_id": msg_id,
            "msg_type": msg_type,
            "username": "client",
            "session": self.session_id,
            "date": timestamp,
            "version": "5.3"
        });

        let parent_header = serde_json::json!({});
        let metadata = serde_json::json!({});

        // Serialize components
        let header_str = serde_json::to_string(&header)?;
        let parent_header_str = serde_json::to_string(&parent_header)?;
        let metadata_str = serde_json::to_string(&metadata)?;
        let content_str = serde_json::to_string(&content)?;

        // Create signature
        let signature = if let Some(key) = &self.hmac_key {
            let mut mac = Hmac::<Sha256>::new_from_slice(key)?;
            mac.update(header_str.as_bytes());
            mac.update(parent_header_str.as_bytes());
            mac.update(metadata_str.as_bytes());
            mac.update(content_str.as_bytes());
            hex::encode(mac.finalize().into_bytes())
        } else {
            String::new()
        };

        // Build multipart message: [identity], signature, header, parent_header, metadata, content
        let parts = vec![
            b"client".to_vec(), // identity
            signature.into_bytes(),
            header_str.into_bytes(),
            parent_header_str.into_bytes(),
            metadata_str.into_bytes(),
            content_str.into_bytes(),
        ];

        Ok(parts)
    }

    pub fn send_execute_request(&mut self, code: &str) -> Result<()> {
        self.execution_count += 1;

        let content = serde_json::json!({
            "code": code,
            "silent": false,
            "store_history": true,
            "user_expressions": {},
            "allow_stdin": false,
            "stop_on_error": true
        });

        let parts = self.create_message("execute_request", content)?;

        let shell_socket = self.shell_socket.lock().unwrap();
        if let Some(socket) = shell_socket.as_ref() {
            socket.send_multipart(parts, 0)?;
        }

        Ok(())
    }

    pub fn receive_execute_reply(&self) -> Result<String> {
        let shell_socket = self.shell_socket.lock().unwrap();
        if let Some(socket) = shell_socket.as_ref() {
            match socket.recv_multipart(0) {
                Ok(parts) => {
                    if parts.len() >= 6 {
                        // Parse the reply content (last part)
                        let content_json: Value = serde_json::from_slice(&parts[5])?;

                        // Extract result from execute_reply
                        if let Some(status) = content_json.get("status") {
                            if status == "ok" {
                                Ok("Execution completed successfully".to_string())
                            } else {
                                let error_name = content_json
                                    .get("ename")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("Unknown");
                                let error_value = content_json
                                    .get("evalue")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or("Unknown error");
                                Ok(format!("Error: {}: {}", error_name, error_value))
                            }
                        } else {
                            Ok("No status in reply".to_string())
                        }
                    } else {
                        anyhow::bail!(
                            "Invalid message format: expected at least 6 parts, got {}",
                            parts.len()
                        )
                    }
                }
                Err(zmq::Error::EAGAIN) => anyhow::bail!("Timeout waiting for execute reply"),
                Err(e) => Err(e.into()),
            }
        } else {
            anyhow::bail!("Shell socket not connected")
        }
    }
}

#[async_trait]
impl KernelConnectionTrait for JupyterKernelClient {
    async fn connect_or_start(&mut self) -> Result<()> {
        // Try to find existing kernel first
        let discovery = CliKernelDiscovery::new();
        let discovery_guard = discovery.discovery.read().await;
        let kernels = discovery_guard.discover_kernels().await;

        if let Ok(kernels) = kernels {
            if let Some(kernel_info) = kernels.first() {
                // Convert to JupyterConnectionInfo format
                let jupyter_info = JupyterConnectionInfo {
                    shell_port: kernel_info.shell_port,
                    iopub_port: kernel_info.iopub_port,
                    stdin_port: kernel_info.stdin_port,
                    control_port: kernel_info.control_port,
                    hb_port: kernel_info.hb_port,
                    ip: kernel_info.ip.clone(),
                    key: kernel_info.key.clone(),
                    transport: kernel_info.transport.clone(),
                    signature_scheme: "hmac-sha256".to_string(),
                    kernel_name: "llmspell".to_string(),
                };

                self.connect_to_kernel(jupyter_info)?;
                return Ok(());
            }
        }

        anyhow::bail!("No running kernel found. Please start llmspell-kernel first.")
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    async fn disconnect(&mut self) -> Result<()> {
        *self.shell_socket.lock().unwrap() = None;
        *self.iopub_socket.lock().unwrap() = None;
        self.connected = false;
        Ok(())
    }

    async fn execute(&mut self, code: &str) -> Result<String> {
        if !self.connected {
            anyhow::bail!("Not connected to kernel");
        }

        self.send_execute_request(code)?;
        self.receive_execute_reply()
    }

    async fn execute_inline(&mut self, code: &str) -> Result<String> {
        self.execute(code).await
    }

    async fn repl(&mut self) -> Result<()> {
        anyhow::bail!("REPL mode not implemented for JupyterKernelClient")
    }

    async fn info(&mut self) -> Result<Value> {
        Ok(serde_json::json!({
            "status": "connected",
            "connected": self.connected,
            "implementation": "JupyterKernelClient",
            "session_id": self.session_id,
            "execution_count": self.execution_count
        }))
    }

    async fn send_debug_command(&mut self, _command: Value) -> Result<Value> {
        anyhow::bail!("Debug commands not yet implemented for JupyterKernelClient")
    }

    fn classify_workload(&self, operation: &str) -> WorkloadClassifier {
        classify_operation_workload(operation)
    }

    fn execution_manager(&self) -> Option<&dyn std::any::Any> {
        None
    }

    async fn shutdown(&mut self) -> Result<()> {
        self.disconnect().await
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
