//! Kernel connection management for CLI client
//!
//! Provides dependency-injected kernel connection with workload-aware performance.

use anyhow::Result;
use async_trait::async_trait;
use llmspell_bridge::execution_context::SharedExecutionContext;
use llmspell_bridge::{
    circuit_breaker::{CircuitBreaker, OperationContext},
    debug_state_cache::DebugStateCache,
    diagnostics_bridge::DiagnosticsBridge,
    execution_bridge::ExecutionManager,
    hook_profiler::WorkloadClassifier,
    lua::debug_state_cache_impl::LuaDebugStateCache,
    session_recorder::{SessionEvent, SessionRecorder},
};
use llmspell_debug::session_manager::DebugSessionManager;
use llmspell_engine::{LDPRequest, LDPResponse, LRPRequest, LRPResponse, ProtocolClient};
use llmspell_kernel::KernelDiscovery;
use llmspell_repl::{client::ConnectedClient, ConnectionInfo};
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

/// Temporary adapter while migrating from llmspell-engine to llmspell-kernel
pub struct KernelClient {
    /// Will eventually use Jupyter client
    /// For now, still uses ProtocolClient from engine
    inner: ProtocolClient,
}

impl KernelClient {
    /// Create a new KernelClient wrapping ProtocolClient
    pub async fn connect(addr: &str) -> Result<Self> {
        let inner = ProtocolClient::connect(addr).await?;
        Ok(Self { inner })
    }

    /// Execute code through the wrapped client
    pub async fn execute(&self, code: &str) -> Result<Value> {
        let request = LRPRequest::ExecuteRequest {
            code: code.to_string(),
            silent: false,
            store_history: true,
            user_expressions: None,
            allow_stdin: false,
            stop_on_error: true,
        };

        let response = self.inner.send_lrp_request(request).await?;

        match response {
            LRPResponse::ExecuteReply {
                status, payload, ..
            } => {
                if status == "ok" {
                    // Extract result from payload if present
                    if let Some(payload_vec) = payload {
                        if let Some(first) = payload_vec.first() {
                            Ok(first.clone())
                        } else {
                            Ok(Value::Null)
                        }
                    } else {
                        Ok(Value::Null)
                    }
                } else {
                    anyhow::bail!("Execution failed with status: {}", status)
                }
            }
            _ => anyhow::bail!("Unexpected response type"),
        }
    }

    /// Send debug command through the wrapped client
    pub async fn send_debug_command(&self, command: LDPRequest) -> Result<LDPResponse> {
        self.inner
            .send_ldp_request(command)
            .await
            .map_err(|e| anyhow::anyhow!("Debug command failed: {}", e))
    }

    /// Shutdown the wrapped client
    pub async fn shutdown(self) {
        self.inner.shutdown().await;
    }

    /// Check if the connection is healthy
    pub async fn health_check(&self) -> Result<bool> {
        // Try to execute a simple command to check if kernel is responsive
        match self.execute("return 'health_ok'").await {
            Ok(result) => {
                // Check if we got the expected response
                if let Some(s) = result.as_str() {
                    Ok(s == "health_ok")
                } else {
                    Ok(true) // Kernel responded, even if not with expected value
                }
            }
            Err(_) => Ok(false),
        }
    }
}

/// Trait for kernel connection operations
#[async_trait]
pub trait KernelConnectionTrait: Send + Sync {
    /// Connect to a kernel or start a new one
    async fn connect_or_start(&mut self) -> Result<()>;

    /// Execute code on the kernel
    async fn execute(&mut self, code: &str) -> Result<Value>;

    /// Send a debug command
    async fn send_debug_command(&mut self, command: LDPRequest) -> Result<LDPResponse>;

    /// Get execution manager
    fn execution_manager(&self) -> Option<Arc<ExecutionManager>>;

    /// Check if connected
    fn is_connected(&self) -> bool;

    /// Disconnect from kernel
    async fn disconnect(&mut self) -> Result<()>;

    /// Get current workload classification
    fn classify_workload(&self, operation: &str) -> WorkloadClassifier;

    /// Execute script with debug support
    async fn execute_script_debug(
        &mut self,
        script: &str,
        args: Vec<String>,
        session_id: String,
    ) -> Result<DebugExecutionHandle>;

    /// Check if debug mode is supported
    fn supports_debug(&self) -> bool;

    /// Perform a health check on the kernel
    async fn health_check(&mut self) -> Result<bool>;
}

/// Handle for debug execution
pub struct DebugExecutionHandle {
    pub session_id: String,
    pub shared_context: Arc<RwLock<SharedExecutionContext>>,
    pub debug_session: Arc<RwLock<DebugSessionManager>>,
}

/// Builder for kernel connection with dependency injection
#[derive(Default)]
pub struct KernelConnectionBuilder {
    discovery: Option<Box<dyn KernelDiscoveryTrait>>,
    circuit_breaker: Option<Box<dyn CircuitBreaker>>,
    session_recorder: Option<Box<dyn SessionRecorder>>,
    diagnostics: Option<DiagnosticsBridge>,
}

impl KernelConnectionBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the kernel discovery implementation
    pub fn discovery(mut self, discovery: Box<dyn KernelDiscoveryTrait>) -> Self {
        self.discovery = Some(discovery);
        self
    }

    /// Set the circuit breaker implementation
    pub fn circuit_breaker(mut self, circuit_breaker: Box<dyn CircuitBreaker>) -> Self {
        self.circuit_breaker = Some(circuit_breaker);
        self
    }

    /// Set the session recorder implementation
    pub fn session_recorder(mut self, session_recorder: Box<dyn SessionRecorder>) -> Self {
        self.session_recorder = Some(session_recorder);
        self
    }

    /// Set the diagnostics bridge
    pub fn diagnostics(mut self, diagnostics: DiagnosticsBridge) -> Self {
        self.diagnostics = Some(diagnostics);
        self
    }

    /// Build the kernel connection
    pub fn build(self) -> KernelConnection {
        KernelConnection {
            discovery: self
                .discovery
                .unwrap_or_else(|| Box::new(CliKernelDiscovery::new())),
            circuit_breaker: self.circuit_breaker,
            session_recorder: self.session_recorder,
            diagnostics: self.diagnostics,
            connection_info: None,
            client: None,
            protocol_client: None,
            execution_manager: None,
            debug_session_manager: None,
            connected: false,
            kernel_process: None,
        }
    }
}

/// Real kernel connection implementation
pub struct KernelConnection {
    discovery: Box<dyn KernelDiscoveryTrait>,
    #[allow(dead_code)]
    circuit_breaker: Option<Box<dyn CircuitBreaker>>,
    #[allow(dead_code)]
    session_recorder: Option<Box<dyn SessionRecorder>>,
    diagnostics: Option<DiagnosticsBridge>,
    connection_info: Option<ConnectionInfo>,
    client: Option<ConnectedClient>,
    protocol_client: Option<ProtocolClient>,
    execution_manager: Option<Arc<ExecutionManager>>,
    debug_session_manager: Option<Arc<RwLock<DebugSessionManager>>>,
    connected: bool,
    kernel_process: Option<tokio::process::Child>,
}

impl KernelConnection {
    /// Start a new kernel process and connect to it
    async fn start_new_kernel(&mut self) -> Result<()> {
        let kernel_id = uuid::Uuid::new_v4().to_string();
        tracing::debug!("[9.8.2] start_new_kernel - kernel_id: {}", kernel_id);
        let port = 9555; // TODO: Find available port
        let info = ConnectionInfo::new(kernel_id.clone(), "127.0.0.1".to_string(), port);

        // Write connection file first
        tracing::debug!("[9.8.2] start_new_kernel - writing connection file");
        info.write_connection_file().await?;

        // Spawn kernel process
        tracing::debug!(
            "[9.8.2] start_new_kernel - spawning kernel process on port {}",
            port
        );
        let mut kernel_process = Self::spawn_kernel(port).await?;

        // Give kernel time to fully start up
        tracing::debug!("[9.8.2] start_new_kernel - waiting 500ms for kernel to start");
        sleep(Duration::from_millis(500)).await;

        // Wait for kernel to be ready and connect
        // For legacy compatibility, connect to shell_port + 10 where TCP server runs
        let tcp_port = info.shell_port + 10;
        let addr = format!("{}:{}", info.ip, tcp_port);
        tracing::debug!(
            "[9.8.2] start_new_kernel - waiting for kernel at {} (legacy TCP port)",
            addr
        );

        match Self::wait_for_kernel_ready(&addr, 50).await {
            Ok(protocol_client) => {
                // Successfully started and connected
                self.connection_info = Some(info);
                self.client = Some(ConnectedClient::new("cli-user".to_string()));
                self.protocol_client = Some(protocol_client);
                self.kernel_process = Some(kernel_process);
                self.connected = true;
                tracing::info!("Started new kernel and connected via TCP");
                Ok(())
            }
            Err(e) => {
                // Failed to connect, kill the kernel process
                tracing::error!("Failed to connect to spawned kernel: {}", e);
                kernel_process.kill().await.ok();
                info.remove_connection_file().await.ok();
                Err(e)
            }
        }
    }

    /// Find the kernel binary path
    fn find_kernel_binary() -> Result<std::path::PathBuf> {
        // Look for "llmspell-kernel" instead of old name
        which::which("llmspell-kernel")
            .or_else(|_| {
                // Check target directory
                if let Ok(current_exe) = std::env::current_exe() {
                    let mut path = current_exe;
                    path.pop(); // Remove current binary name
                    path.push("llmspell-kernel");
                    if path.exists() {
                        return Ok(path);
                    }
                }

                // Try common development locations
                let possible_paths = [
                    "target/debug/llmspell-kernel",
                    "target/release/llmspell-kernel",
                    "../target/debug/llmspell-kernel", 
                    "../target/release/llmspell-kernel",
                ];

                for path_str in &possible_paths {
                    let path = std::path::Path::new(path_str);
                    if path.exists() && path.is_file() {
                        return Ok(path.to_path_buf());
                    }
                }

                // For tests, try to find it relative to the manifest directory
                if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
                    let test_path = std::path::Path::new(&manifest_dir)
                        .join("target")
                        .join("debug")
                        .join("llmspell-kernel");
                    if test_path.exists() {
                        return Ok(test_path);
                    }
                }

                anyhow::bail!(
                    "Could not find llmspell-kernel binary. Please ensure it is built and in your PATH."
                )
            })
    }

    /// Spawn a kernel process
    async fn spawn_kernel(port: u16) -> Result<tokio::process::Child> {
        let kernel_path = Self::find_kernel_binary()?;

        tracing::debug!(
            "[9.8.2] spawn_kernel - kernel binary path: {}",
            kernel_path.display()
        );
        tracing::info!("Starting kernel from: {}", kernel_path.display());

        let child = Command::new(&kernel_path)
            .arg("--port")
            .arg(port.to_string())
            .arg("--engine")
            .arg("lua") // Default to Lua engine
            .arg("--legacy-tcp") // Enable backward compatibility with LRP/TCP protocol
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::null())
            .spawn()
            .map_err(|e| anyhow::anyhow!("Failed to spawn kernel process: {}", e))?;

        tracing::info!("Kernel process spawned with PID: {:?}", child.id());
        Ok(child)
    }

    /// Wait for kernel to be ready by attempting to connect
    async fn wait_for_kernel_ready(addr: &str, max_retries: u32) -> Result<ProtocolClient> {
        let retry_delay = Duration::from_millis(100);

        for attempt in 1..=max_retries {
            tracing::debug!(
                "Attempting to connect to kernel (attempt {}/{})",
                attempt,
                max_retries
            );

            match ProtocolClient::connect(addr).await {
                Ok(client) => {
                    tracing::info!("Successfully connected to kernel");
                    return Ok(client);
                }
                Err(e) => {
                    if attempt < max_retries {
                        tracing::debug!(
                            "Connection attempt {} failed: {}, retrying...",
                            attempt,
                            e
                        );
                        sleep(retry_delay).await;
                    } else {
                        return Err(anyhow::anyhow!(
                            "Failed to connect to kernel after {} attempts: {}",
                            max_retries,
                            e
                        ));
                    }
                }
            }
        }

        unreachable!()
    }
}

#[async_trait]
impl KernelConnectionTrait for KernelConnection {
    async fn connect_or_start(&mut self) -> Result<()> {
        tracing::debug!("[9.8.2] connect_or_start - starting");
        // Try to discover existing kernel
        if let Some(kernel) = self.discovery.discover_first().await? {
            tracing::debug!(
                "[9.8.2] connect_or_start - found existing kernel: {}",
                kernel.kernel_id
            );
            // Try to connect to existing kernel via legacy TCP port
            let tcp_port = kernel.shell_port + 10;
            let addr = format!("{}:{}", kernel.ip, tcp_port);

            match ProtocolClient::connect(&addr).await {
                Ok(protocol_client) => {
                    tracing::debug!(
                        "[9.8.2] connect_or_start - successfully connected to existing kernel"
                    );
                    // Successfully connected to existing kernel
                    self.connection_info = Some(kernel);
                    self.client = Some(ConnectedClient::new("cli-user".to_string()));
                    self.protocol_client = Some(protocol_client);
                    self.connected = true;
                    tracing::info!("Connected to existing kernel via TCP");
                }
                Err(e) => {
                    tracing::warn!("Found kernel connection file but couldn't connect: {}", e);
                    tracing::info!("Will start a new kernel instead");

                    // Remove stale connection file
                    kernel.remove_connection_file().await.ok();

                    // Fall through to start new kernel
                    self.start_new_kernel().await?;
                }
            }
        } else {
            tracing::debug!(
                "[9.8.2] connect_or_start - no existing kernel found, starting new one"
            );
            // No existing kernel found, start new one
            self.start_new_kernel().await?;
        }

        // Initialize execution manager
        if let Some(_diagnostics) = &self.diagnostics {
            // Create execution manager with diagnostics
            // This would be properly initialized with the actual kernel runtime
            // For now, we just mark it as available
            // self.execution_manager = Some(Arc::new(ExecutionManager::new()));
        }

        Ok(())
    }

    async fn execute(&mut self, code: &str) -> Result<Value> {
        tracing::debug!("[9.8.2] execute - starting with code: {}", code);
        if !self.connected {
            anyhow::bail!("Not connected to kernel");
        }

        let protocol_client = self
            .protocol_client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Protocol client not initialized"))?;

        tracing::debug!("[9.8.2] execute - sending execute request via TCP");
        // Send execute request via TCP
        let request = LRPRequest::ExecuteRequest {
            code: code.to_string(),
            silent: false,
            store_history: true,
            user_expressions: None,
            allow_stdin: false,
            stop_on_error: true,
        };

        tracing::debug!("[9.8.2] execute - about to send_lrp_request");
        let response = protocol_client
            .send_lrp_request(request)
            .await
            .map_err(|e| {
                tracing::error!("[9.8.2] execute - send_lrp_request failed: {}", e);
                anyhow::anyhow!("Failed to execute code: {}", e)
            })?;
        tracing::debug!("[9.8.2] execute - received response from kernel");

        match response {
            LRPResponse::ExecuteReply {
                status, payload, ..
            } => {
                if status == "ok" {
                    // Extract result from payload if present
                    if let Some(payload_vec) = payload {
                        if let Some(first) = payload_vec.first() {
                            Ok(first.clone())
                        } else {
                            Ok(Value::Null)
                        }
                    } else {
                        Ok(Value::Null)
                    }
                } else {
                    anyhow::bail!("Execution failed with status: {}", status)
                }
            }
            _ => anyhow::bail!("Unexpected response type"),
        }
    }

    async fn send_debug_command(&mut self, command: LDPRequest) -> Result<LDPResponse> {
        if !self.connected {
            anyhow::bail!("Not connected to kernel");
        }

        let protocol_client = self
            .protocol_client
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Protocol client not initialized"))?;

        // Send debug command via TCP
        let response = protocol_client
            .send_ldp_request(command)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to send debug command: {}", e))?;
        Ok(response)
    }

    fn execution_manager(&self) -> Option<Arc<ExecutionManager>> {
        self.execution_manager.clone()
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    async fn disconnect(&mut self) -> Result<()> {
        // Shutdown TCP client first
        if let Some(client) = self.protocol_client.take() {
            client.shutdown().await;
        }

        // If we spawned a kernel process, shut it down gracefully
        if let Some(mut kernel_process) = self.kernel_process.take() {
            tracing::info!("Shutting down kernel process");

            // Try sending interrupt signal first (graceful shutdown)
            if let Some(pid) = kernel_process.id() {
                // Send SIGTERM for graceful shutdown (on Unix-like systems)
                #[cfg(unix)]
                {
                    use nix::sys::signal::{self, Signal};
                    use nix::unistd::Pid;

                    if let Err(e) = signal::kill(Pid::from_raw(pid as i32), Signal::SIGTERM) {
                        tracing::warn!("Failed to send SIGTERM to kernel: {}", e);
                    }
                }

                // On Windows, we can only kill the process
                #[cfg(windows)]
                {
                    kernel_process.kill().await.ok();
                }
            }

            // Give the process time to shut down gracefully
            let shutdown_timeout = Duration::from_secs(5);
            let shutdown_start = Instant::now();

            loop {
                // Check if process has exited
                match kernel_process.try_wait() {
                    Ok(Some(_status)) => {
                        tracing::info!("Kernel process exited gracefully");
                        break;
                    }
                    Ok(None) => {
                        // Process still running
                        if shutdown_start.elapsed() > shutdown_timeout {
                            // Timeout reached, force kill
                            tracing::warn!("Kernel process did not exit gracefully, force killing");
                            kernel_process.kill().await.ok();
                            kernel_process.wait().await.ok();
                            break;
                        }
                        // Wait a bit before checking again
                        sleep(Duration::from_millis(100)).await;
                    }
                    Err(e) => {
                        tracing::error!("Error checking kernel process status: {}", e);
                        kernel_process.kill().await.ok();
                        break;
                    }
                }
            }
        }

        // Remove connection file
        if let Some(info) = &self.connection_info {
            info.remove_connection_file().await?;
        }

        self.connected = false;
        Ok(())
    }

    fn classify_workload(&self, operation: &str) -> WorkloadClassifier {
        match operation {
            "tab_complete" | "syntax_check" => WorkloadClassifier::Micro,
            "execute_line" | "get_locals" => WorkloadClassifier::Light,
            "execute_block" | "debug_step" | "debug_execute" => WorkloadClassifier::Medium,
            "execute_file" | "profile" => WorkloadClassifier::Heavy,
            _ => WorkloadClassifier::Light,
        }
    }

    async fn execute_script_debug(
        &mut self,
        script: &str,
        args: Vec<String>,
        session_id: String,
    ) -> Result<DebugExecutionHandle> {
        if !self.connected {
            self.connect_or_start().await?;
        }

        // Initialize debug session manager if not already done
        if self.debug_session_manager.is_none() {
            // Create or get execution manager
            if self.execution_manager.is_none() {
                // Create a default execution manager for debug support
                let debug_cache: Arc<dyn DebugStateCache> = Arc::new(LuaDebugStateCache::new());
                self.execution_manager = Some(Arc::new(ExecutionManager::new(debug_cache)));
            }

            if let Some(exec_mgr) = &self.execution_manager {
                self.debug_session_manager = Some(Arc::new(RwLock::new(DebugSessionManager::new(
                    exec_mgr.clone(),
                ))));
            }
        }

        // Create shared execution context
        let shared_context = Arc::new(RwLock::new(SharedExecutionContext::new()));
        {
            let mut ctx = shared_context.write().await;
            ctx.correlation_id =
                Some(uuid::Uuid::parse_str(&session_id).unwrap_or_else(|_| uuid::Uuid::new_v4()));
            // Store args in variables for now (script_args field doesn't exist yet)
            for (i, arg) in args.iter().enumerate() {
                ctx.variables
                    .insert(format!("arg{}", i), serde_json::Value::String(arg.clone()));
            }
        }

        // Register debug session
        if let Some(debug_mgr) = &self.debug_session_manager {
            let mgr = debug_mgr.read().await;
            mgr.create_session("cli-user".to_string()).await?;
        }

        // Send debug execute request to kernel
        let request = LDPRequest::EvaluateRequest {
            expression: script.to_string(),
            frame_id: Some(0), // Use 0 as default frame
            context: Some("debug".to_string()),
            format: None,
        };

        // Send the debug request
        self.send_debug_command(request).await?;

        Ok(DebugExecutionHandle {
            session_id,
            shared_context,
            debug_session: self
                .debug_session_manager
                .as_ref()
                .expect("Debug session manager should be initialized")
                .clone(),
        })
    }

    fn supports_debug(&self) -> bool {
        self.execution_manager.is_some() || self.diagnostics.is_some()
    }

    async fn health_check(&mut self) -> Result<bool> {
        if !self.connected {
            return Ok(false);
        }

        // Try to execute a simple command to check if kernel is responsive
        match self.execute("return 'health_ok'").await {
            Ok(result) => {
                // Check if we got the expected response
                if let Some(s) = result.as_str() {
                    Ok(s == "health_ok")
                } else {
                    Ok(true) // Kernel responded, even if not with expected value
                }
            }
            Err(e) => {
                tracing::warn!("Health check failed: {}", e);

                // If health check failed, mark as disconnected
                self.connected = false;

                // Try to reconnect
                if let Err(reconnect_err) = self.connect_or_start().await {
                    tracing::error!(
                        "Failed to reconnect after health check failure: {}",
                        reconnect_err
                    );
                    Ok(false)
                } else {
                    // Successfully reconnected
                    Ok(true)
                }
            }
        }
    }
}

/// Trait for kernel discovery operations
#[async_trait]
pub trait KernelDiscoveryTrait: Send + Sync {
    /// Discover first available kernel
    async fn discover_first(&self) -> Result<Option<ConnectionInfo>>;

    /// Discover all kernels
    async fn discover_all(&self) -> Result<Vec<ConnectionInfo>>;
}

/// CLI-specific kernel discovery with dependency injection and enhanced features
pub struct CliKernelDiscovery {
    discovery: KernelDiscovery,
    connection_cache: Arc<RwLock<HashMap<String, ConnectionInfo>>>,
    circuit_breaker: Option<Box<dyn CircuitBreaker>>,
    session_recorder: Option<Box<dyn SessionRecorder>>,
    cleanup_on_exit: bool,
    max_retry_attempts: usize,
}

/// Builder for CLI kernel discovery
pub struct CliKernelDiscoveryBuilder {
    discovery: Option<KernelDiscovery>,
    circuit_breaker: Option<Box<dyn CircuitBreaker>>,
    session_recorder: Option<Box<dyn SessionRecorder>>,
    cleanup_on_exit: bool,
    max_retry_attempts: usize,
}

impl Default for CliKernelDiscoveryBuilder {
    fn default() -> Self {
        Self {
            discovery: None,
            circuit_breaker: None,
            session_recorder: None,
            cleanup_on_exit: true,
            max_retry_attempts: 3,
        }
    }
}

impl CliKernelDiscoveryBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn circuit_breaker(mut self, circuit_breaker: Box<dyn CircuitBreaker>) -> Self {
        self.circuit_breaker = Some(circuit_breaker);
        self
    }

    pub fn session_recorder(mut self, session_recorder: Box<dyn SessionRecorder>) -> Self {
        self.session_recorder = Some(session_recorder);
        self
    }

    pub fn cleanup_on_exit(mut self, cleanup: bool) -> Self {
        self.cleanup_on_exit = cleanup;
        self
    }

    pub fn max_retry_attempts(mut self, attempts: usize) -> Self {
        self.max_retry_attempts = attempts;
        self
    }

    pub fn build(self) -> CliKernelDiscovery {
        CliKernelDiscovery {
            discovery: self.discovery.unwrap_or_default(),
            connection_cache: Arc::new(RwLock::new(HashMap::new())),
            circuit_breaker: self.circuit_breaker,
            session_recorder: self.session_recorder,
            cleanup_on_exit: self.cleanup_on_exit,
            max_retry_attempts: self.max_retry_attempts,
        }
    }
}

impl Default for CliKernelDiscovery {
    fn default() -> Self {
        CliKernelDiscoveryBuilder::new().build()
    }
}

impl CliKernelDiscovery {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn builder() -> CliKernelDiscoveryBuilder {
        CliKernelDiscoveryBuilder::new()
    }

    /// Get multiple search paths for kernel discovery
    pub fn get_search_paths() -> Vec<std::path::PathBuf> {
        let mut paths = vec![];

        // Default user directory (~/.llmspell/kernels)
        if let Some(home) = dirs::home_dir() {
            paths.push(home.join(".llmspell").join("kernels"));
        }

        // System-wide directory (/tmp on Unix, %TEMP% on Windows)
        paths.push(std::env::temp_dir().join("llmspell-kernels"));

        // Current working directory (for development)
        if let Ok(cwd) = std::env::current_dir() {
            paths.push(cwd.join(".llmspell-kernels"));
        }

        // Check LLMSPELL_KERNEL_DIR environment variable
        if let Ok(kernel_dir) = std::env::var("LLMSPELL_KERNEL_DIR") {
            paths.push(std::path::PathBuf::from(kernel_dir));
        }

        // For tests, check the test-specific directory
        if cfg!(test) {
            paths.push(std::path::PathBuf::from("/tmp/llmspell-test-kernels"));
        }

        paths
    }

    /// Discover kernels from multiple locations
    pub async fn discover_from_multiple_locations(
        search_paths: &[std::path::PathBuf],
    ) -> Result<Vec<ConnectionInfo>> {
        let mut all_kernels = vec![];

        for path in search_paths {
            if !path.exists() {
                continue;
            }

            // Create a temporary KernelDiscovery for this path
            let discovery = KernelDiscovery::with_dir(path.clone());

            match discovery.discover_kernels().await {
                Ok(kernels) => {
                    tracing::debug!("Found {} kernels in {}", kernels.len(), path.display());
                    all_kernels.extend(kernels);
                }
                Err(e) => {
                    tracing::debug!("Failed to discover kernels in {}: {}", path.display(), e);
                }
            }
        }

        Ok(all_kernels)
    }

    /// Record a discovery attempt with session recorder
    fn record_discovery_attempt(&mut self, kernel_id: Option<&str>, success: bool) {
        if let Some(recorder) = &mut self.session_recorder {
            let event = if success {
                SessionEvent::ToolInvocation {
                    tool_name: format!("kernel_discovery_success:{}", kernel_id.unwrap_or("new")),
                    arguments: serde_json::json!({"kernel_id": kernel_id}),
                    context: SharedExecutionContext::new(),
                }
            } else {
                SessionEvent::ToolInvocation {
                    tool_name: format!(
                        "kernel_discovery_failed:{}",
                        kernel_id.unwrap_or("unknown")
                    ),
                    arguments: serde_json::json!({"kernel_id": kernel_id}),
                    context: SharedExecutionContext::new(),
                }
            };
            let _ = recorder.record_event(event);
        }
    }

    /// Get adaptive retry interval based on workload classification
    fn get_retry_interval(&self, attempt: usize) -> Duration {
        let base_interval = match WorkloadClassifier::Medium {
            WorkloadClassifier::Micro => Duration::from_millis(50),
            WorkloadClassifier::Light => Duration::from_millis(100),
            WorkloadClassifier::Medium => Duration::from_millis(200),
            WorkloadClassifier::Heavy => Duration::from_millis(500),
        };

        // Exponential backoff
        base_interval * (2_u32.pow(attempt as u32))
    }

    /// Try to connect with circuit breaker and retry logic
    async fn try_connect_with_retry(&mut self, info: &ConnectionInfo) -> Result<bool> {
        for attempt in 0..self.max_retry_attempts {
            let start_time = Instant::now();

            // Create operation context for circuit breaker
            let context = OperationContext {
                operation_name: "kernel_discovery".to_string(),
                workload: WorkloadClassifier::Medium,
                duration: Duration::default(), // Will be filled after operation
                success: false,                // Will be updated based on result
            };

            // Check circuit breaker
            if let Some(breaker) = &self.circuit_breaker {
                if !breaker.allow_operation(&context) {
                    tracing::debug!("Circuit breaker is open, skipping connection attempt");
                    return Ok(false);
                }
            }

            // Try to connect
            let connection_result = KernelDiscovery::is_kernel_alive(info).await;
            let duration = start_time.elapsed();

            match connection_result {
                Ok(true) => {
                    // Success - record it
                    if let Some(breaker) = &mut self.circuit_breaker {
                        let success_context = OperationContext {
                            operation_name: "kernel_discovery".to_string(),
                            workload: WorkloadClassifier::Medium,
                            duration,
                            success: true,
                        };
                        breaker.record_operation(success_context);
                    }
                    self.record_discovery_attempt(Some(&info.kernel_id), true);
                    return Ok(true);
                }
                Ok(false) if attempt < self.max_retry_attempts - 1 => {
                    // Kernel not responding, retry with adaptive interval
                    let interval = self.get_retry_interval(attempt);
                    tracing::debug!(
                        "Kernel {} not responding, retrying in {:?}",
                        info.kernel_id,
                        interval
                    );
                    sleep(interval).await;
                }
                Ok(false) => {
                    // Final attempt failed
                    if let Some(breaker) = &mut self.circuit_breaker {
                        let failure_context = OperationContext {
                            operation_name: "kernel_discovery".to_string(),
                            workload: WorkloadClassifier::Medium,
                            duration,
                            success: false,
                        };
                        breaker.record_operation(failure_context);
                    }
                }
                Err(e) => {
                    // Connection error
                    if let Some(breaker) = &mut self.circuit_breaker {
                        let failure_context = OperationContext {
                            operation_name: "kernel_discovery".to_string(),
                            workload: WorkloadClassifier::Medium,
                            duration,
                            success: false,
                        };
                        breaker.record_operation(failure_context);
                    }
                    tracing::warn!("Error connecting to kernel {}: {}", info.kernel_id, e);
                    if attempt < self.max_retry_attempts - 1 {
                        let interval = self.get_retry_interval(attempt);
                        sleep(interval).await;
                    }
                }
            }
        }

        self.record_discovery_attempt(Some(&info.kernel_id), false);
        Ok(false)
    }

    /// Clean up connection files on exit
    pub async fn cleanup(&self) -> Result<()> {
        if !self.cleanup_on_exit {
            return Ok(());
        }

        tracing::info!("Cleaning up kernel connections...");

        // Clean up cached connections
        let cache = self.connection_cache.read().await;
        for (kernel_id, info) in cache.iter() {
            tracing::debug!("Removing connection file for kernel {}", kernel_id);
            if let Err(e) = info.remove_connection_file().await {
                tracing::warn!("Failed to remove connection file for {}: {}", kernel_id, e);
            }
        }

        // Clean up stale connections
        if let Ok(removed) = self.discovery.cleanup_stale_connections().await {
            if !removed.is_empty() {
                tracing::info!("Cleaned up {} stale connections", removed.len());
            }
        }

        Ok(())
    }

    /// Enhanced discover_first that uses cache and circuit breaker (requires mutable access)
    pub async fn discover_first_alive(&mut self) -> Result<Option<ConnectionInfo>> {
        // Check cache first and clone the info to avoid borrow conflicts
        let cached_info = {
            let cache = self.connection_cache.read().await;
            cache.iter().next().map(|(_, info)| info.clone())
        };

        if let Some(info) = cached_info {
            if self.try_connect_with_retry(&info).await? {
                return Ok(Some(info));
            }
        }

        // Try to discover kernels from multiple locations
        let search_paths = CliKernelDiscovery::get_search_paths();
        let kernels = CliKernelDiscovery::discover_from_multiple_locations(&search_paths).await?;
        for info in kernels {
            if self.try_connect_with_retry(&info).await? {
                // Cache the connection
                {
                    let mut cache = self.connection_cache.write().await;
                    cache.insert(info.kernel_id.clone(), info.clone());
                }
                return Ok(Some(info));
            }
        }

        Ok(None)
    }

    /// Enhanced discover_all that uses cache and circuit breaker (requires mutable access)
    pub async fn discover_all_alive(&mut self) -> Result<Vec<ConnectionInfo>> {
        // Try to discover kernels from multiple locations
        let search_paths = CliKernelDiscovery::get_search_paths();
        let kernels = CliKernelDiscovery::discover_from_multiple_locations(&search_paths).await?;
        let mut alive_kernels = Vec::new();

        for info in kernels {
            if self.try_connect_with_retry(&info).await? {
                alive_kernels.push(info.clone());
                // Cache the connection
                {
                    let mut cache = self.connection_cache.write().await;
                    cache.insert(info.kernel_id.clone(), info);
                }
            }
        }

        Ok(alive_kernels)
    }
}

#[async_trait]
impl KernelDiscoveryTrait for CliKernelDiscovery {
    async fn discover_first(&self) -> Result<Option<ConnectionInfo>> {
        // Check cache first
        {
            let cache = self.connection_cache.read().await;
            if let Some((_, info)) = cache.iter().next() {
                // We need to check if it's still alive, but the method signature doesn't allow mut self
                // So we'll just return the cached info for now
                return Ok(Some(info.clone()));
            }
        }

        // Try to discover kernels from multiple locations
        let search_paths = CliKernelDiscovery::get_search_paths();
        let kernels = CliKernelDiscovery::discover_from_multiple_locations(&search_paths).await?;

        for info in kernels {
            // Check if kernel is alive - note: this is a limitation since we can't modify self
            // In real usage, the caller should use discover_first_alive instead
            if KernelDiscovery::is_kernel_alive(&info)
                .await
                .unwrap_or(false)
            {
                return Ok(Some(info));
            }
        }

        Ok(None)
    }

    async fn discover_all(&self) -> Result<Vec<ConnectionInfo>> {
        // Try to discover kernels from multiple locations
        let search_paths = CliKernelDiscovery::get_search_paths();
        let kernels = CliKernelDiscovery::discover_from_multiple_locations(&search_paths).await?;
        let mut alive_kernels = Vec::new();

        for info in kernels {
            if KernelDiscovery::is_kernel_alive(&info)
                .await
                .unwrap_or(false)
            {
                alive_kernels.push(info);
            }
        }

        Ok(alive_kernels)
    }
}

/// Null kernel connection for testing
pub struct NullKernelConnection {
    connected: bool,
    workload_classifier: WorkloadClassifier,
}

impl NullKernelConnection {
    pub fn new() -> Self {
        Self {
            connected: false,
            workload_classifier: WorkloadClassifier::Light,
        }
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

    async fn execute(&mut self, _code: &str) -> Result<Value> {
        Ok(Value::String("null execution".to_string()))
    }

    async fn send_debug_command(&mut self, _command: LDPRequest) -> Result<LDPResponse> {
        Ok(LDPResponse::ContinueResponse {
            all_threads_continued: Some(false),
        })
    }

    fn execution_manager(&self) -> Option<Arc<ExecutionManager>> {
        None
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.connected = false;
        Ok(())
    }

    fn classify_workload(&self, _operation: &str) -> WorkloadClassifier {
        self.workload_classifier
    }

    async fn execute_script_debug(
        &mut self,
        _script: &str,
        _args: Vec<String>,
        session_id: String,
    ) -> Result<DebugExecutionHandle> {
        // Null implementation for testing
        self.connected = true;
        let shared_context = Arc::new(RwLock::new(SharedExecutionContext::new()));
        {
            let mut ctx = shared_context.write().await;
            ctx.correlation_id =
                Some(uuid::Uuid::parse_str(&session_id).unwrap_or_else(|_| uuid::Uuid::new_v4()));
        }

        // Create a minimal execution manager for null implementation
        let debug_cache: Arc<dyn DebugStateCache> = Arc::new(LuaDebugStateCache::new());
        let exec_mgr = Arc::new(ExecutionManager::new(debug_cache));

        Ok(DebugExecutionHandle {
            session_id,
            shared_context,
            debug_session: Arc::new(RwLock::new(DebugSessionManager::new(exec_mgr))),
        })
    }

    fn supports_debug(&self) -> bool {
        false // Null implementation doesn't support real debugging
    }

    async fn health_check(&mut self) -> Result<bool> {
        Ok(false) // Null connection always fails health check
    }
}

/// Null kernel discovery for testing
pub struct NullKernelDiscovery;

#[async_trait]
impl KernelDiscoveryTrait for NullKernelDiscovery {
    async fn discover_first(&self) -> Result<Option<ConnectionInfo>> {
        Ok(None)
    }

    async fn discover_all(&self) -> Result<Vec<ConnectionInfo>> {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_bridge::{
        null_circuit_breaker::NullCircuitBreaker, null_session_recorder::NullSessionRecorder,
    };
    use llmspell_repl::ConnectionInfo;
    use std::time::Duration;

    #[tokio::test]
    async fn test_real_kernel_discovery_builder() {
        let discovery = CliKernelDiscovery::builder()
            .circuit_breaker(Box::new(NullCircuitBreaker::new()))
            .session_recorder(Box::new(NullSessionRecorder::new()))
            .cleanup_on_exit(false)
            .max_retry_attempts(2)
            .build();

        // Discovery should work even if no kernels exist
        let result = discovery.discover_first().await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_real_kernel_discovery_new() {
        let discovery = CliKernelDiscovery::new();

        // Default values should work
        let result = discovery.discover_all().await.unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_retry_interval_calculation() {
        let discovery = CliKernelDiscovery::new();

        // Test exponential backoff
        let interval0 = discovery.get_retry_interval(0);
        let interval1 = discovery.get_retry_interval(1);
        let interval2 = discovery.get_retry_interval(2);

        assert!(interval1 > interval0);
        assert!(interval2 > interval1);

        // Check that base interval is based on WorkloadClassifier::Medium
        assert_eq!(interval0, Duration::from_millis(200));
        assert_eq!(interval1, Duration::from_millis(400));
        assert_eq!(interval2, Duration::from_millis(800));
    }

    #[tokio::test]
    async fn test_connection_caching() {
        let mut discovery = CliKernelDiscovery::builder().cleanup_on_exit(false).build();

        // Initially cache should be empty
        {
            let cache = discovery.connection_cache.read().await;
            assert!(cache.is_empty());
        }

        // After discovering (even if no kernels exist), cache operations should work
        let _ = discovery.discover_first_alive().await.unwrap();
        // Cache will only be populated if actual kernels are found
    }

    #[tokio::test]
    async fn test_session_recording() {
        let recorder = Box::new(NullSessionRecorder::new());
        let mut discovery = CliKernelDiscovery::builder()
            .session_recorder(recorder)
            .cleanup_on_exit(false)
            .build();

        // Should record discovery attempts
        discovery.record_discovery_attempt(Some("test-kernel"), true);
        discovery.record_discovery_attempt(None, false);
    }

    #[tokio::test]
    async fn test_cleanup_functionality() {
        let discovery = CliKernelDiscovery::builder().cleanup_on_exit(true).build();

        // Should not error even with no connections
        assert!(discovery.cleanup().await.is_ok());

        let discovery_no_cleanup = CliKernelDiscovery::builder().cleanup_on_exit(false).build();

        // Should be a no-op when cleanup_on_exit is false
        assert!(discovery_no_cleanup.cleanup().await.is_ok());
    }

    #[tokio::test]
    async fn test_circuit_breaker_integration() {
        let mut discovery = CliKernelDiscovery::builder()
            .circuit_breaker(Box::new(NullCircuitBreaker::new()))
            .cleanup_on_exit(false)
            .max_retry_attempts(1)
            .build();

        // Create a fake connection info for testing
        let connection_info = ConnectionInfo::new(
            "test-kernel".to_string(),
            "127.0.0.1".to_string(),
            9999, // Use a port that won't be available
        );

        // Should handle connection failures gracefully with circuit breaker
        let result = discovery.try_connect_with_retry(&connection_info).await;
        assert!(result.is_ok());
        assert!(!result.unwrap()); // Should return false for unavailable kernel
    }

    #[test]
    fn test_builder_pattern() {
        let builder = CliKernelDiscoveryBuilder::new()
            .cleanup_on_exit(false)
            .max_retry_attempts(5);

        let discovery = builder.build();

        // Verify builder settings were applied
        assert!(!discovery.cleanup_on_exit);
        assert_eq!(discovery.max_retry_attempts, 5);
    }

    #[tokio::test]
    async fn test_kernel_discovery_trait_implementation() {
        let discovery = CliKernelDiscovery::new();

        // Test KernelDiscoveryTrait methods
        let first = discovery.discover_first().await.unwrap();
        assert!(first.is_none());

        let all = discovery.discover_all().await.unwrap();
        assert!(all.is_empty());
    }

    #[tokio::test]
    async fn test_enhanced_discovery_methods() {
        let mut discovery = CliKernelDiscovery::builder().cleanup_on_exit(false).build();

        // Test enhanced methods that require mutable access
        let first_alive = discovery.discover_first_alive().await.unwrap();
        assert!(first_alive.is_none());

        let all_alive = discovery.discover_all_alive().await.unwrap();
        assert!(all_alive.is_empty());
    }
}
