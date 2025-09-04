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
use crate::security::SecurityManager;
use crate::transport::ZmqTransport;

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

        // Jupyter protocol will be handled by serve_jupyter() method

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

    /// Run the kernel with Jupyter protocol
    ///
    /// # Errors
    ///
    /// Returns an error if the Jupyter protocol server fails
    pub async fn run(mut self) -> Result<()> {
        tracing::info!(
            "Starting LLMSpell kernel {} with Jupyter protocol",
            self.kernel_id
        );

        // Start the Jupyter protocol server
        self.serve_jupyter().await
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
        // Channels stopped when UnifiedProtocolEngine shuts down

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

    /// Update execution state and return the new count
    async fn prepare_execution(&self) -> u32 {
        *self.execution_state.write().await = KernelState::Busy;
        let mut count = self.execution_count.lock().await;
        *count += 1;
        *count
    }

    /// Execute script with timeout handling
    async fn execute_with_timeout(
        &self,
        code: &str,
    ) -> std::result::Result<
        Result<llmspell_bridge::ScriptOutput, llmspell_core::LLMSpellError>,
        tokio::time::error::Elapsed,
    > {
        let timeout_duration =
            tokio::time::Duration::from_secs(self.resource_limits.max_execution_time);
        let runtime = self.runtime.clone();
        let code = code.to_string();

        tracing::debug!("About to execute script with timeout");
        tokio::time::timeout(timeout_duration, async move {
            tracing::debug!("Acquiring runtime lock for script execution");
            let runtime_guard = runtime.lock().await;
            tracing::debug!("Executing script");
            runtime_guard.execute_script(&code).await
        })
        .await
    }

    /// Finish execution and update state
    async fn finish_execution(&self, _silent: bool) {
        *self.execution_state.write().await = KernelState::Idle;
        // TODO: Broadcast idle status if not silent
    }

    /// Check if kernel can accept more clients
    pub async fn can_accept_client(&self) -> bool {
        let current_clients = self.client_manager.get_all_clients().await;
        current_clients.len() < self.config.max_clients
    }
}

// ================== JUPYTER PROTOCOL IMPLEMENTATION ==================

impl LLMSpellKernel {
    // ================== JUPYTER PROTOCOL IMPLEMENTATION ==================

    /// Serve Jupyter protocol using ZeroMQ transport (replaces old run() method)
    pub async fn serve_jupyter(&mut self) -> Result<()> {
        use crate::jupyter::connection::{
            ConnectionConfig, ConnectionInfo as JupyterConnectionInfo,
        };
        use crate::transport::ZmqTransport;

        tracing::info!(
            "Starting Jupyter protocol server for kernel {}",
            self.kernel_id
        );

        // Create Jupyter connection info
        let jupyter_connection = JupyterConnectionInfo::new(
            ConnectionConfig::new()
                .with_ip(self.config.ip.clone())
                .with_port_range(self.config.port_range_start)
                .with_kernel_name("llmspell".to_string()),
        )?;

        // Save connection file for Jupyter clients
        let connection_file = JupyterConnectionInfo::temp_connection_file(&self.kernel_id);
        jupyter_connection.save_to_file(&connection_file)?;
        tracing::info!("Jupyter connection file: {}", connection_file.display());

        // Bind ZeroMQ transport
        let transport = ZmqTransport::bind(&jupyter_connection)?;

        // Set kernel state to idle
        *self.execution_state.write().await = KernelState::Idle;

        // Main Jupyter protocol loop
        loop {
            // Handle shell messages first
            if let Err(e) = self.handle_shell_channel(&transport).await {
                tracing::error!("Shell channel error: {}", e);
            }

            // Handle control messages
            if let Err(e) = self.handle_control_channel(&transport).await {
                tracing::error!("Control channel error: {}", e);
                if e.to_string().contains("shutdown") {
                    break;
                }
            }

            // Handle stdin messages
            if let Err(e) = self.handle_stdin_channel(&transport).await {
                tracing::error!("Stdin channel error: {}", e);
            }

            // Handle heartbeat
            if let Err(e) = self.handle_heartbeat_channel(&transport).await {
                tracing::debug!("Heartbeat channel error: {}", e);
            }

            // Small delay to prevent busy loop
            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
        }

        tracing::info!("Jupyter protocol server shutting down");
        Ok(())
    }

    /// Handle messages from shell channel (execute, kernel_info, completion, inspection)
    async fn handle_shell_channel(&mut self, transport: &ZmqTransport) -> Result<()> {
        use crate::jupyter::protocol::MessageContent;

        if let Some(msg) = transport.recv_shell_msg()? {
            tracing::debug!("Received shell message: {}", msg.header.msg_type);

            let reply = match &msg.content {
                MessageContent::KernelInfoRequest {} => self.handle_kernel_info_request(&msg),
                MessageContent::ExecuteRequest { code, silent, .. } => {
                    self.handle_execute_request(&msg, code, *silent).await?
                }
                MessageContent::CompleteRequest { code, cursor_pos } => {
                    self.handle_complete_request(&msg, code, *cursor_pos)
                }
                MessageContent::InspectRequest {
                    code,
                    cursor_pos,
                    detail_level,
                } => self.handle_inspect_request(&msg, code, *cursor_pos, *detail_level),
                _ => {
                    tracing::warn!("Unhandled shell message type: {}", msg.header.msg_type);
                    return Ok(());
                }
            };

            transport.send_shell_reply(&reply)?;
        }
        Ok(())
    }

    /// Handle messages from control channel (shutdown, interrupt, daemon requests)
    async fn handle_control_channel(&mut self, transport: &ZmqTransport) -> Result<()> {
        use crate::jupyter::protocol::MessageContent;

        if let Some(msg) = transport.recv_control_msg()? {
            tracing::debug!("Received control message: {}", msg.header.msg_type);

            let reply = match &msg.content {
                MessageContent::ShutdownRequest { restart } => {
                    let reply = self.handle_shutdown_request(&msg, *restart).await?;
                    transport.send_control_reply(&reply)?;
                    return Err(anyhow::anyhow!("Kernel shutdown requested"));
                }
                MessageContent::InterruptRequest {} => self.handle_interrupt_request(&msg).await,
                MessageContent::DaemonRequest {
                    command,
                    kernel_id,
                    config,
                } => {
                    self.handle_daemon_request(&msg, command, kernel_id.as_deref(), config.as_ref())
                }
                _ => {
                    tracing::warn!("Unhandled control message type: {}", msg.header.msg_type);
                    return Ok(());
                }
            };

            transport.send_control_reply(&reply)?;
        }
        Ok(())
    }

    /// Handle messages from stdin channel (input requests)
    async fn handle_stdin_channel(&mut self, transport: &ZmqTransport) -> Result<()> {
        use crate::jupyter::protocol::MessageContent;

        if let Some(msg) = transport.recv_stdin_msg()? {
            tracing::debug!("Received stdin message: {}", msg.header.msg_type);

            match &msg.content {
                MessageContent::InputReply { value } => {
                    // Handle input from user
                    tracing::info!("Received input: {}", value);
                    return Ok(());
                }
                _ => {
                    tracing::warn!("Unhandled stdin message type: {}", msg.header.msg_type);
                    return Ok(());
                }
            };
        }
        Ok(())
    }

    /// Handle heartbeat channel (simple echo)
    async fn handle_heartbeat_channel(&self, transport: &ZmqTransport) -> Result<()> {
        transport.handle_heartbeat()?;
        Ok(())
    }

    // ================== JUPYTER MESSAGE HANDLERS ==================

    /// Handle kernel_info_request
    fn handle_kernel_info_request(
        &self,
        parent: &crate::jupyter::protocol::JupyterMessage,
    ) -> crate::jupyter::protocol::JupyterMessage {
        use crate::jupyter::protocol::{HelpLink, JupyterMessage, LanguageInfo, MessageContent};

        JupyterMessage::reply(
            parent,
            "kernel_info_reply",
            MessageContent::KernelInfoReply {
                status: "ok".to_string(),
                protocol_version: "5.3".to_string(),
                implementation: "llmspell".to_string(),
                implementation_version: env!("CARGO_PKG_VERSION").to_string(),
                language_info: LanguageInfo {
                    name: self.config.engine.clone(),
                    version: "1.0".to_string(),
                    mimetype: match self.config.engine.as_str() {
                        "lua" => "text/x-lua".to_string(),
                        "javascript" => "text/javascript".to_string(),
                        _ => "text/plain".to_string(),
                    },
                    file_extension: match self.config.engine.as_str() {
                        "lua" => ".lua".to_string(),
                        "javascript" => ".js".to_string(),
                        _ => ".txt".to_string(),
                    },
                    pygments_lexer: Some(self.config.engine.clone()),
                    codemirror_mode: Some(self.config.engine.clone()),
                    nbconvert_exporter: None,
                },
                banner: format!(
                    "LLMSpell {} - {} Script Engine",
                    env!("CARGO_PKG_VERSION"),
                    self.config.engine
                ),
                help_links: vec![HelpLink {
                    text: "LLMSpell Documentation".to_string(),
                    url: "https://github.com/llmspell/llmspell".to_string(),
                }],
            },
        )
    }

    /// Handle execute_request
    async fn handle_execute_request(
        &mut self,
        parent: &crate::jupyter::protocol::JupyterMessage,
        code: &String,
        silent: bool,
    ) -> Result<crate::jupyter::protocol::JupyterMessage> {
        use crate::jupyter::protocol::{ExecutionStatus, JupyterMessage, MessageContent};

        tracing::debug!("Executing code: {}", code);

        // Update execution count and state
        let execution_count = self.prepare_execution().await;

        // TODO: Publish execute_input to iopub if not silent

        // Execute code with timeout
        let result = self.execute_with_timeout(code).await;

        let reply = match result {
            Ok(Ok(script_output)) => {
                // TODO: Publish execution result to iopub if not silent

                JupyterMessage::reply(
                    parent,
                    "execute_reply",
                    MessageContent::ExecuteReply {
                        status: ExecutionStatus::Ok,
                        execution_count,
                        user_expressions: None,
                        payload: if let Ok(json_value) = serde_json::to_value(&script_output.output)
                        {
                            Some(vec![json_value])
                        } else {
                            None
                        },
                    },
                )
            }
            Ok(Err(_)) | Err(_) => {
                // TODO: Publish error to iopub if not silent

                JupyterMessage::reply(
                    parent,
                    "execute_reply",
                    MessageContent::ExecuteReply {
                        status: ExecutionStatus::Error,
                        execution_count,
                        user_expressions: None,
                        payload: None,
                    },
                )
            }
        };

        // Finish execution
        self.finish_execution(silent).await;

        Ok(reply)
    }

    /// Handle completion_request
    fn handle_complete_request(
        &self,
        parent: &crate::jupyter::protocol::JupyterMessage,
        _code: &str,
        _cursor_pos: u32,
    ) -> crate::jupyter::protocol::JupyterMessage {
        use crate::jupyter::protocol::{JupyterMessage, MessageContent};
        use std::collections::HashMap;

        // TODO: Implement actual completion logic based on script engine
        JupyterMessage::reply(
            parent,
            "complete_reply",
            MessageContent::CompleteReply {
                matches: vec![], // Empty for now
                cursor_start: 0,
                cursor_end: 0,
                metadata: HashMap::new(),
                status: "ok".to_string(),
            },
        )
    }

    /// Handle inspect_request
    fn handle_inspect_request(
        &self,
        parent: &crate::jupyter::protocol::JupyterMessage,
        _code: &str,
        _cursor_pos: u32,
        _detail_level: u32,
    ) -> crate::jupyter::protocol::JupyterMessage {
        use crate::jupyter::protocol::{JupyterMessage, MessageContent};
        use std::collections::HashMap;

        // TODO: Implement actual inspection logic based on script engine
        JupyterMessage::reply(
            parent,
            "inspect_reply",
            MessageContent::InspectReply {
                status: "ok".to_string(),
                found: false,
                data: HashMap::new(),
                metadata: HashMap::new(),
            },
        )
    }

    /// Handle shutdown_request
    async fn handle_shutdown_request(
        &mut self,
        parent: &crate::jupyter::protocol::JupyterMessage,
        restart: bool,
    ) -> Result<crate::jupyter::protocol::JupyterMessage> {
        use crate::jupyter::protocol::{JupyterMessage, MessageContent};

        tracing::info!("Shutdown requested, restart: {}", restart);

        // TODO: Implement proper shutdown logic

        Ok(JupyterMessage::reply(
            parent,
            "shutdown_reply",
            MessageContent::ShutdownReply {
                status: "ok".to_string(),
                restart,
            },
        ))
    }

    /// Handle interrupt_request  
    async fn handle_interrupt_request(
        &mut self,
        parent: &crate::jupyter::protocol::JupyterMessage,
    ) -> crate::jupyter::protocol::JupyterMessage {
        use crate::jupyter::protocol::{JupyterMessage, MessageContent};

        tracing::info!("Interrupt requested");

        // TODO: Implement actual interrupt logic (stop running execution)

        JupyterMessage::reply(
            parent,
            "interrupt_reply",
            MessageContent::InterruptReply {
                status: "ok".to_string(),
            },
        )
    }

    /// Handle daemon_request (custom LLMSpell extension)
    fn handle_daemon_request(
        &self,
        parent: &crate::jupyter::protocol::JupyterMessage,
        command: &crate::jupyter::protocol::DaemonCommand,
        _kernel_id: Option<&str>,
        _config: Option<&serde_json::Value>,
    ) -> crate::jupyter::protocol::JupyterMessage {
        use crate::jupyter::protocol::{DaemonCommand, JupyterMessage, KernelInfo, MessageContent};
        use chrono::Utc;

        match command {
            DaemonCommand::KernelStatus => {
                let kernel_info = KernelInfo {
                    kernel_id: self.kernel_id.clone(),
                    status: format!(
                        "{:?}",
                        *futures::executor::block_on(self.execution_state.read())
                    ),
                    engine: self.config.engine.clone(),
                    connections: 1, // TODO: Get actual connection count
                    uptime: 0,      // TODO: Calculate actual uptime
                    last_activity: Utc::now(),
                };

                JupyterMessage::reply(
                    parent,
                    "daemon_reply",
                    MessageContent::DaemonReply {
                        status: "ok".to_string(),
                        command: command.clone(),
                        result: serde_json::to_value(&kernel_info).ok(),
                        error: None,
                        kernels: Some(vec![kernel_info]),
                    },
                )
            }
            _ => {
                // TODO: Implement other daemon commands
                JupyterMessage::reply(
                    parent,
                    "daemon_reply",
                    MessageContent::DaemonReply {
                        status: "error".to_string(),
                        command: command.clone(),
                        result: None,
                        error: Some("Daemon command not yet implemented".to_string()),
                        kernels: None,
                    },
                )
            }
        }
    }
}

// Add Debug impl for kernel to satisfy MessageProcessor trait
impl std::fmt::Debug for LLMSpellKernel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LLMSpellKernel")
            .field("kernel_id", &self.kernel_id)
            .field("config", &self.config)
            .field("execution_state", &self.execution_state)
            .field("connection_info", &self.connection_info)
            .field("execution_count", &self.execution_count)
            .finish_non_exhaustive()
    }
}
