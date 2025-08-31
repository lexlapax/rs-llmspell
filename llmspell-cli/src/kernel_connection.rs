//! Kernel connection management for CLI client
//!
//! Provides dependency-injected kernel connection with workload-aware performance.

use anyhow::Result;
use async_trait::async_trait;
use llmspell_bridge::execution_context::SharedExecutionContext;
use llmspell_bridge::{
    circuit_breaker::CircuitBreaker, debug_state_cache::DebugStateCache,
    diagnostics_bridge::DiagnosticsBridge, execution_bridge::ExecutionManager,
    hook_profiler::WorkloadClassifier, lua::debug_state_cache_impl::LuaDebugStateCache,
    session_recorder::SessionRecorder,
};
use llmspell_debug::session_manager::DebugSessionManager;
use llmspell_repl::{
    client::ConnectedClient,
    connection::ConnectionInfo,
    discovery::KernelDiscovery,
    protocol::{LDPRequest, LDPResponse},
};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::RwLock;

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
                .unwrap_or_else(|| Box::new(RealKernelDiscovery::new())),
            circuit_breaker: self.circuit_breaker,
            session_recorder: self.session_recorder,
            diagnostics: self.diagnostics,
            connection_info: None,
            client: None,
            execution_manager: None,
            debug_session_manager: None,
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
    debug_session_manager: Option<Arc<RwLock<DebugSessionManager>>>,
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
            frame_id: Some(session_id.clone()),
            context: Some("debug".to_string()),
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
