//! Kernel connection management for CLI client
//!
//! Provides dependency-injected kernel connection with workload-aware performance.

use anyhow::Result;
use async_trait::async_trait;
use llmspell_bridge::{
    circuit_breaker::CircuitBreaker,
    diagnostics_bridge::DiagnosticsBridge,
    execution_bridge::ExecutionManager,
    hook_profiler::WorkloadClassifier,
    session_recorder::SessionRecorder,
};
use llmspell_repl::{
    client::ConnectedClient,
    connection::ConnectionInfo,
    discovery::KernelDiscovery,
    protocol::{LDPRequest, LDPResponse},
};
use serde_json::Value;
use std::sync::Arc;

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
                .unwrap_or_else(|| Box::new(RealKernelDiscovery::new())),
            circuit_breaker: self.circuit_breaker,
            session_recorder: self.session_recorder,
            diagnostics: self.diagnostics,
            connection_info: None,
            client: None,
            execution_manager: None,
            connected: false,
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
    execution_manager: Option<Arc<ExecutionManager>>,
    connected: bool,
}

#[async_trait]
impl KernelConnectionTrait for KernelConnection {
    async fn connect_or_start(&mut self) -> Result<()> {
        // Try to discover existing kernel
        if let Some(kernel) = self.discovery.discover_first().await? {
            self.connection_info = Some(kernel);
            self.client = Some(ConnectedClient::new("cli-user".to_string()));
            self.connected = true;
            tracing::info!("Connected to existing kernel");
        } else {
            // Start new kernel
            let kernel_id = uuid::Uuid::new_v4().to_string();
            let info = ConnectionInfo::new(kernel_id, "127.0.0.1".to_string(), 5555);
            info.write_connection_file().await?;

            self.connection_info = Some(info);
            self.client = Some(ConnectedClient::new("cli-user".to_string()));
            self.connected = true;
            tracing::info!("Started new kernel");
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

    async fn execute(&mut self, _code: &str) -> Result<Value> {
        if !self.connected {
            anyhow::bail!("Not connected to kernel");
        }

        // Record execution if session recorder is available
        // TODO: Implement proper session recording when SessionEvent is updated

        // In a real implementation, this would send an execute request to the kernel
        // For now, return a placeholder
        Ok(Value::String("Execution placeholder".to_string()))
    }

    async fn send_debug_command(&mut self, _command: LDPRequest) -> Result<LDPResponse> {
        if !self.connected {
            anyhow::bail!("Not connected to kernel");
        }

        // In a real implementation, this would send the debug command to the kernel
        // For now, return a placeholder response
        Ok(LDPResponse::ContinueReply)
    }

    fn execution_manager(&self) -> Option<Arc<ExecutionManager>> {
        self.execution_manager.clone()
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    async fn disconnect(&mut self) -> Result<()> {
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
            "execute_block" | "debug_step" => WorkloadClassifier::Medium,
            "execute_file" | "profile" => WorkloadClassifier::Heavy,
            _ => WorkloadClassifier::Light,
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

/// Real kernel discovery implementation
#[derive(Default)]
pub struct RealKernelDiscovery {
    discovery: KernelDiscovery,
}

impl RealKernelDiscovery {
    pub fn new() -> Self {
        Self::default()
    }
}

#[async_trait]
impl KernelDiscoveryTrait for RealKernelDiscovery {
    async fn discover_first(&self) -> Result<Option<ConnectionInfo>> {
        let kernels = self.discovery.discover_kernels().await?;
        Ok(kernels.into_iter().next())
    }

    async fn discover_all(&self) -> Result<Vec<ConnectionInfo>> {
        self.discovery.discover_kernels().await
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
        Ok(LDPResponse::ContinueReply)
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
