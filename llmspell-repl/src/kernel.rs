//! Core kernel service implementation
//!
//! The `LLMSpellKernel` wraps the `ScriptRuntime` from llmspell-bridge and provides
//! multi-client debugging and REPL capabilities.

use anyhow::Result;
use llmspell_bridge::ScriptRuntime;
use llmspell_config::LLMSpellConfig;
use std::sync::Arc;
use tokio::sync::{oneshot, Mutex, RwLock};
use uuid::Uuid;

use crate::client::{ClientManager, ConnectedClient};
use crate::connection::ConnectionInfo;
use crate::protocol::LRPResponse;
use crate::security::SecurityManager;

/// Kernel execution state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KernelState {
    /// Kernel is idle and ready for commands
    Idle,
    /// Kernel is executing code
    Busy,
    /// Kernel is starting up
    Starting,
    /// Kernel is shutting down
    Stopping,
}

/// Configuration for kernel startup
#[derive(Debug, Clone)]
pub struct KernelConfig {
    /// Unique kernel identifier
    pub kernel_id: Option<String>,
    /// IP address to bind to
    pub ip: String,
    /// Port range start for allocating channels
    pub port_range_start: u16,
    /// Enable debug mode
    pub debug_enabled: bool,
    /// Maximum number of clients
    pub max_clients: usize,
    /// Script engine to use (lua, javascript)
    pub engine: String,
    /// `LLMSpell` runtime configuration
    pub runtime_config: LLMSpellConfig,
    /// Enable authentication
    pub auth_enabled: bool,
}

impl Default for KernelConfig {
    fn default() -> Self {
        Self {
            kernel_id: None,
            ip: "127.0.0.1".to_string(),
            port_range_start: 9555,
            debug_enabled: false,
            max_clients: 10,
            engine: "lua".to_string(),
            runtime_config: LLMSpellConfig::default(),
            auth_enabled: false,
        }
    }
}

/// Resource limits per client
#[derive(Debug, Clone)]
pub struct ClientResourceLimits {
    /// Maximum execution time in seconds
    pub max_execution_time: u64,
    /// Maximum memory usage in bytes
    pub max_memory_bytes: usize,
    /// Maximum concurrent executions
    pub max_concurrent_executions: usize,
    /// Rate limit (requests per minute)
    pub rate_limit_per_minute: u32,
}

impl Default for ClientResourceLimits {
    fn default() -> Self {
        Self {
            max_execution_time: 60,              // 1 minute
            max_memory_bytes: 100 * 1024 * 1024, // 100MB
            max_concurrent_executions: 5,
            rate_limit_per_minute: 60,
        }
    }
}

/// Main kernel service that manages script execution and debugging
pub struct LLMSpellKernel {
    /// Unique kernel identifier
    pub kernel_id: String,

    /// Script runtime from llmspell-bridge
    pub runtime: Arc<Mutex<ScriptRuntime>>,

    /// Client manager
    pub client_manager: Arc<ClientManager>,

    /// Protocol server (implements `ProtocolEngine`)
    pub protocol_server: Option<Arc<llmspell_engine::ProtocolServer>>,

    /// Current execution state
    pub execution_state: Arc<RwLock<KernelState>>,

    /// Kernel configuration
    pub config: KernelConfig,

    /// Connection information
    pub connection_info: ConnectionInfo,

    /// Security manager
    pub security_manager: Arc<SecurityManager>,

    /// Resource limits per client
    pub resource_limits: ClientResourceLimits,

    /// Execution counter for tracking
    pub execution_count: Arc<Mutex<u32>>,

    /// Shutdown signal sender
    shutdown_tx: Option<oneshot::Sender<()>>,
    // Debug components will be added in Phase 9.2
    // pub debugger: Arc<Debugger>,
    // pub profiler: Arc<PerformanceProfiler>,
    // pub tracer: Arc<DistributedTracer>,
}

impl LLMSpellKernel {
    /// Start a new kernel with the given configuration
    ///
    /// # Errors
    ///
    /// Returns an error if kernel initialization fails
    ///
    /// # Panics
    ///
    /// Panics if `kernel_id` is None after generation (should never happen)
    pub async fn start(mut config: KernelConfig) -> Result<Self> {
        // Generate kernel ID if not provided
        if config.kernel_id.is_none() {
            config.kernel_id = Some(Uuid::new_v4().to_string());
        }
        let kernel_id = config.kernel_id.clone().unwrap();

        tracing::info!(
            "Starting LLMSpell kernel {} with engine {}",
            kernel_id,
            config.engine
        );

        // Create script runtime from llmspell-bridge
        let runtime =
            ScriptRuntime::new_with_engine_name(&config.engine, config.runtime_config.clone())
                .await
                .map_err(|e| anyhow::anyhow!("Failed to create script runtime: {}", e))?;

        // Protocol server will be initialized in run() method
        // We store it as Option to set it up after kernel creation

        // Create connection info
        let connection_info = ConnectionInfo::new(
            kernel_id.clone(),
            config.ip.clone(),
            config.port_range_start,
        );

        // Write connection file
        connection_info.write_connection_file().await?;

        // Create security manager
        let security_manager = Arc::new(SecurityManager::new(
            connection_info.key.clone(),
            config.auth_enabled,
        ));

        // Create client manager
        let client_manager = Arc::new(ClientManager::new(config.max_clients));

        // Create shutdown channel
        let (shutdown_tx, _shutdown_rx) = oneshot::channel();

        // Create kernel instance
        let kernel = Self {
            kernel_id: kernel_id.clone(),
            runtime: Arc::new(Mutex::new(runtime)),
            client_manager,
            protocol_server: None,
            execution_state: Arc::new(RwLock::new(KernelState::Starting)),
            config: config.clone(),
            connection_info,
            security_manager,
            resource_limits: ClientResourceLimits::default(),
            execution_count: Arc::new(Mutex::new(0)),
            shutdown_tx: Some(shutdown_tx),
        };

        // Set state to idle
        *kernel.execution_state.write().await = KernelState::Idle;

        tracing::info!("Kernel {} started successfully", kernel_id);
        Ok(kernel)
    }

    /// Start a new kernel with sidecar support for service mesh pattern
    ///
    /// # Errors
    ///
    /// Returns an error if kernel or sidecar initialization fails
    pub async fn start_with_sidecar(config: KernelConfig) -> Result<Self> {
        use llmspell_engine::sidecar::{
            LocalServiceDiscovery, NullMetricsCollector, Sidecar, SidecarConfig,
        };
        use llmspell_engine::UnifiedProtocolEngine;
        use llmspell_utils::circuit_breaker::{CircuitBreaker, CircuitBreakerConfig};
        use std::sync::Arc;

        // Create the kernel first
        let kernel = Self::start(config.clone()).await?;

        // Create protocol engine for sidecar
        // Note: This is a placeholder - in real implementation, we'd use the actual transport
        // from the ProtocolServer when it's created in run()
        let transport = Box::new(
            llmspell_engine::transport::tcp::TcpTransport::connect(&format!(
                "{}:{}",
                config.ip, config.port_range_start
            ))
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create transport: {}", e))?,
        );

        let engine = Arc::new(UnifiedProtocolEngine::new(transport));

        // Create sidecar components
        let circuit_breaker = CircuitBreaker::new(CircuitBreakerConfig::default());
        let discovery = Arc::new(LocalServiceDiscovery::new());
        let metrics = Arc::new(NullMetricsCollector); // Use null for now, can be replaced
        let sidecar_config = SidecarConfig::default();

        // Create sidecar
        let _sidecar = Sidecar::new(engine, circuit_breaker, discovery, metrics, sidecar_config);

        // In a full implementation, we would:
        // 1. Store the sidecar in the kernel struct
        // 2. Intercept all messages through the sidecar
        // 3. Register the kernel service with discovery

        tracing::info!("Kernel {} started with sidecar support", kernel.kernel_id);

        Ok(kernel)
    }

    /// Run the kernel event loop
    ///
    /// # Errors
    ///
    /// Returns an error if the event loop fails
    #[allow(clippy::cognitive_complexity)]
    pub async fn run(mut self) -> Result<()> {
        tracing::info!("Kernel {} entering main event loop", self.kernel_id);

        // Start channel listeners
        // Channels are now handled by ProtocolServer

        // Create a new shutdown channel (the old one is dropped with self)
        let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();
        // Drop the old shutdown sender if it exists
        drop(self.shutdown_tx.take());

        // Now we can move self into Arc
        let kernel_arc = Arc::new(self);
        let handler = Arc::new(crate::protocol_handler::KernelProtocolHandler::new(
            kernel_arc.clone(),
        ));

        let server_config = llmspell_engine::ServerConfig {
            ip: kernel_arc.config.ip.clone(),
            shell_port: kernel_arc.connection_info.shell_port,
            iopub_port: kernel_arc.connection_info.iopub_port,
            stdin_port: kernel_arc.connection_info.stdin_port,
            control_port: kernel_arc.connection_info.control_port,
            heartbeat_port: kernel_arc.connection_info.hb_port,
            max_clients: kernel_arc.config.max_clients,
        };

        // Create the protocol server
        let mut protocol_server = llmspell_engine::ProtocolServer::new(server_config, handler);

        // Create a ctrl-c handler
        let ctrl_c = tokio::signal::ctrl_c();

        // Spawn protocol server task
        let server_handle = tokio::spawn(async move {
            if let Err(e) = protocol_server.start().await {
                tracing::error!("Protocol server error: {}", e);
            }
        });

        // Keep shutdown_tx alive in a separate task
        let _shutdown_guard = tokio::spawn(async move {
            // This task will hold shutdown_tx until it's dropped
            let _tx = shutdown_tx;
            // Sleep forever (or until the task is cancelled)
            tokio::time::sleep(std::time::Duration::from_secs(u64::MAX)).await;
        });

        // Main event loop
        tokio::select! {
            // Wait for shutdown signal
            _ = shutdown_rx => {
                tracing::info!("Kernel {} received shutdown signal", kernel_arc.kernel_id);
            }
            // Wait for Ctrl+C
            _ = ctrl_c => {
                tracing::info!("Kernel {} received Ctrl+C signal", kernel_arc.kernel_id);
            }
            // Wait for server task to complete (error case)
            _ = server_handle => {
                tracing::warn!("Protocol server task ended unexpectedly");
            }
        }

        // Shutdown the kernel - need to extract from Arc
        if let Ok(kernel) = Arc::try_unwrap(kernel_arc) {
            kernel.shutdown().await?;
        } else {
            tracing::warn!("Could not cleanly shutdown kernel - Arc still has references");
        }
        Ok(())
    }

    /// Shutdown the kernel gracefully
    ///
    /// # Errors
    ///
    /// Returns an error if shutdown fails
    #[allow(clippy::cognitive_complexity)]
    pub async fn shutdown(self) -> Result<()> {
        tracing::info!("Shutting down kernel {}", self.kernel_id);

        // Set state to stopping
        *self.execution_state.write().await = KernelState::Stopping;

        // Disconnect all clients
        let all_clients = self.client_manager.get_all_clients().await;
        for client in all_clients {
            tracing::info!("Disconnecting client {}", client.client_id);
            self.client_manager.remove_client(&client.client_id).await;
        }

        // Stop channels
        // Channels stopped when ProtocolServer shuts down

        // Remove connection file
        self.connection_info.remove_connection_file().await?;

        // Send shutdown signal if receiver exists
        if let Some(tx) = self.shutdown_tx {
            let _ = tx.send(());
        }

        tracing::info!("Kernel {} shutdown complete", self.kernel_id);
        Ok(())
    }

    /// Add a new client connection
    ///
    /// # Errors
    ///
    /// Returns an error if adding the client fails
    pub async fn add_client(&self, client: ConnectedClient) -> Result<()> {
        // Check authentication if enabled
        if self.config.auth_enabled {
            // Client should have been authenticated before adding
            // This is handled by the connection handler
        }

        self.client_manager.add_client(client.clone()).await?;

        // Broadcast client connection to IOPub
        // TODO: Use ChannelSet
        // self.channels.iopub.publish(IOPubMessage::Status {
        //     execution_state: "idle".to_string(),
        // })?;

        tracing::info!(
            "Client {} connected to kernel {}",
            client.client_id,
            self.kernel_id
        );
        Ok(())
    }

    /// Remove a client connection
    ///
    /// # Errors
    ///
    /// Returns an error if removing the client fails
    pub async fn remove_client(&self, client_id: &str) -> Result<()> {
        if let Some(client) = self.client_manager.remove_client(client_id).await {
            tracing::info!(
                "Client {} disconnected from kernel {}",
                client.client_id,
                self.kernel_id
            );
        }
        Ok(())
    }

    /// Execute code for a client with resource limits
    ///
    /// # Errors
    ///
    /// Returns an error if code execution fails
    pub async fn execute_code(
        &self,
        _client_id: &str,
        code: String,
        silent: bool,
    ) -> Result<LRPResponse> {
        // Update execution state
        *self.execution_state.write().await = KernelState::Busy;

        // Increment execution counter
        let execution_count = {
            let mut count = self.execution_count.lock().await;
            *count += 1;
            *count
        };

        // Broadcast busy status
        if !silent {
            // TODO: Use ChannelSet
            // self.channels.iopub.publish(IOPubMessage::Status {
            //     execution_state: "busy".to_string(),
            // })?;
        }

        // Execute code with timeout
        let timeout_duration =
            tokio::time::Duration::from_secs(self.resource_limits.max_execution_time);

        let runtime = self.runtime.lock().await;
        let result = tokio::time::timeout(timeout_duration, runtime.execute_script(&code)).await;
        drop(runtime);

        // Handle result
        let response = match result {
            Ok(Ok(_output)) => {
                // Broadcast output if not silent
                if !silent {
                    // TODO: Use ChannelSet
                    // self.channels.iopub.publish(IOPubMessage::ExecuteResult {
                    //     execution_count,
                    //     data: serde_json::json!({
                    //         "text/plain": format!("{:?}", output.output)
                    //     }),
                    // })?;
                }

                LRPResponse::ExecuteReply {
                    status: "ok".to_string(),
                    execution_count,
                    user_expressions: None,
                    payload: None,
                }
            }
            Ok(Err(_e)) => {
                // Execution error
                if !silent {
                    // TODO: Use ChannelSet
                    // self.channels.iopub.publish(IOPubMessage::Error {
                    //     ename: "ExecutionError".to_string(),
                    //     evalue: e.to_string(),
                    //     traceback: vec![e.to_string()],
                    // })?;
                }

                LRPResponse::ExecuteReply {
                    status: "error".to_string(),
                    execution_count,
                    user_expressions: None,
                    payload: None,
                }
            }
            Err(_) => {
                // Timeout
                if !silent {
                    // TODO: Use ChannelSet
                    // self.channels.iopub.publish(IOPubMessage::Error {
                    //     ename: "TimeoutError".to_string(),
                    //     evalue: format!(
                    //         "Execution exceeded {} seconds",
                    //         self.resource_limits.max_execution_time
                    //     ),
                    //     traceback: vec![],
                    // })?;
                }

                LRPResponse::ExecuteReply {
                    status: "error".to_string(),
                    execution_count,
                    user_expressions: None,
                    payload: None,
                }
            }
        };

        // Update execution state back to idle
        *self.execution_state.write().await = KernelState::Idle;

        // Broadcast idle status
        if !silent {
            // TODO: Use ChannelSet
            // self.channels.iopub.publish(IOPubMessage::Status {
            //     execution_state: "idle".to_string(),
            // })?;
        }

        Ok(response)
    }

    /// Get kernel information
    #[must_use]
    pub fn get_kernel_info(&self) -> LRPResponse {
        use crate::protocol::{HelpLink, LanguageInfo};

        LRPResponse::KernelInfoReply {
            protocol_version: "1.0".to_string(),
            implementation: "llmspell".to_string(),
            implementation_version: env!("CARGO_PKG_VERSION").to_string(),
            language_info: LanguageInfo {
                name: self.config.engine.clone(),
                version: "1.0".to_string(),
                mimetype: match self.config.engine.as_str() {
                    "lua" => "text/x-lua",
                    "javascript" => "text/javascript",
                    _ => "text/plain",
                }
                .to_string(),
                file_extension: match self.config.engine.as_str() {
                    "lua" => ".lua",
                    "javascript" => ".js",
                    _ => ".txt",
                }
                .to_string(),
                pygments_lexer: Some(self.config.engine.clone()),
                codemirror_mode: Some(self.config.engine.clone()),
                nbconvert_exporter: None,
            },
            banner: format!(
                "LLMSpell Kernel v{} - {}",
                env!("CARGO_PKG_VERSION"),
                self.config.engine
            ),
            debugger: self.config.debug_enabled,
            help_links: vec![HelpLink {
                text: "LLMSpell Documentation".to_string(),
                url: "https://github.com/lexlapax/rs-llmspell".to_string(),
            }],
        }
    }

    /// Check if kernel can accept more clients
    pub async fn can_accept_client(&self) -> bool {
        let current_clients = self.client_manager.get_all_clients().await;
        current_clients.len() < self.config.max_clients
    }
}
