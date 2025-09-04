//! Core kernel service implementation
//!
//! The `GenericKernel` provides a protocol-agnostic kernel that works
//! with any Transport and Protocol implementation via traits.
//! This enables clean separation of concerns and easy extensibility.

use anyhow::Result;
use llmspell_bridge::ScriptRuntime;
use llmspell_config::LLMSpellConfig;
use std::sync::Arc;
use tokio::sync::{oneshot, Mutex, RwLock};
use uuid::Uuid;

use crate::client::ClientManager;
use crate::security::SecurityManager;
use crate::traits::{KernelMessage, Protocol, Transport};

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
    /// Script engine to use (lua, javascript)
    pub engine: String,
    /// `LLMSpell` runtime configuration
    pub runtime_config: LLMSpellConfig,
    /// Enable debug mode
    pub debug_enabled: bool,
    /// Maximum number of clients
    pub max_clients: usize,
    /// Enable authentication
    pub auth_enabled: bool,
}

/// Resource limits per client
#[derive(Debug, Clone)]
pub struct ClientResourceLimits {
    /// Maximum execution time in milliseconds
    pub max_execution_time_ms: u64,
    /// Maximum memory usage in bytes
    pub max_memory_bytes: usize,
    /// Maximum concurrent executions
    pub max_concurrent_executions: usize,
}

impl Default for ClientResourceLimits {
    fn default() -> Self {
        Self {
            max_execution_time_ms: 30000,        // 30 seconds
            max_memory_bytes: 100 * 1024 * 1024, // 100MB
            max_concurrent_executions: 5,
        }
    }
}

/// Generic kernel that works with any Transport and Protocol
pub struct GenericKernel<T: Transport, P: Protocol> {
    /// Unique kernel identifier
    pub kernel_id: String,

    /// Transport layer
    transport: T,

    /// Protocol handler
    protocol: P,

    /// Script runtime from llmspell-bridge
    pub runtime: Arc<Mutex<ScriptRuntime>>,

    /// Client manager
    pub client_manager: Arc<ClientManager>,

    /// Current execution state
    pub execution_state: Arc<RwLock<KernelState>>,

    /// Kernel configuration
    pub config: KernelConfig,

    /// Security manager
    pub security_manager: Arc<SecurityManager>,

    /// Resource limits per client
    pub resource_limits: ClientResourceLimits,

    /// Execution counter for tracking
    pub execution_count: Arc<Mutex<u32>>,

    /// Shutdown signal sender
    shutdown_tx: Option<oneshot::Sender<()>>,
}

impl<T: Transport, P: Protocol> GenericKernel<T, P> {
    /// Create a new kernel with given transport and protocol
    ///
    /// # Errors
    ///
    /// Returns an error if kernel initialization fails.
    ///
    /// # Panics
    ///
    /// Panics if `kernel_id` is None after generation (should never happen).
    pub async fn new(mut config: KernelConfig, mut transport: T, protocol: P) -> Result<Self> {
        // Generate kernel ID if not provided
        if config.kernel_id.is_none() {
            config.kernel_id = Some(Uuid::new_v4().to_string());
        }
        let kernel_id = config.kernel_id.clone().unwrap();

        tracing::info!(
            "Starting kernel {} with {} protocol and engine {}",
            kernel_id,
            protocol.name(),
            config.engine
        );

        // Create script runtime from llmspell-bridge
        let runtime =
            ScriptRuntime::new_with_engine_name(&config.engine, config.runtime_config.clone())
                .await
                .map_err(|e| anyhow::anyhow!("Failed to create script runtime: {}", e))?;

        // Get transport configuration from protocol
        let transport_config = protocol.transport_config();

        // Bind transport to addresses
        transport.bind(&transport_config).await?;

        tracing::info!(
            "Kernel {} bound to {} channels",
            kernel_id,
            transport.channels().len()
        );

        // Create security manager with kernel key
        let kernel_key = Uuid::new_v4().to_string();
        let security_manager = Arc::new(SecurityManager::new(kernel_key, config.auth_enabled));

        // Create client manager
        let client_manager = Arc::new(ClientManager::new(config.max_clients));

        Ok(Self {
            kernel_id,
            transport,
            protocol,
            runtime: Arc::new(Mutex::new(runtime)),
            client_manager,
            execution_state: Arc::new(RwLock::new(KernelState::Starting)),
            config,
            security_manager,
            resource_limits: ClientResourceLimits::default(),
            execution_count: Arc::new(Mutex::new(0)),
            shutdown_tx: None,
        })
    }

    /// Run the kernel - protocol-agnostic main loop
    ///
    /// # Errors
    ///
    /// Returns an error if the kernel fails to serve messages or encounters a fatal error.
    pub async fn serve(&mut self) -> Result<()> {
        tracing::info!(
            "Kernel {} serving with {} protocol (version {})",
            self.kernel_id,
            self.protocol.name(),
            self.protocol.version()
        );

        // Set state to idle
        *self.execution_state.write().await = KernelState::Idle;

        // Main protocol-agnostic loop
        loop {
            // Check all channels for messages
            for channel in self.transport.channels() {
                if let Some(parts) = self.transport.recv(&channel).await? {
                    // Decode message using protocol
                    let message = self.protocol.decode(parts, &channel)?;

                    // Process message
                    let should_reply = self.protocol.requires_reply(&message);
                    let reply_content = self.process_message(message.clone()).await?;

                    if should_reply {
                        // Create reply using protocol
                        let reply = self.protocol.create_reply(&message, reply_content)?;

                        // Encode reply
                        let reply_parts = self.protocol.encode(&reply, &channel)?;

                        // Send reply
                        let reply_channel = self.protocol.reply_channel(&reply);
                        self.transport.send(reply_channel, reply_parts).await?;
                    }

                    // Check for shutdown
                    if message.msg_type() == "shutdown_request" {
                        tracing::info!("Shutdown requested");
                        return Ok(());
                    }
                }
            }

            // Handle heartbeat if needed
            self.transport.heartbeat().await?;

            // Small yield to prevent busy loop
            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
        }
    }

    /// Process a message - protocol-agnostic
    async fn process_message<M: KernelMessage>(&self, message: M) -> Result<serde_json::Value> {
        let msg_type = message.msg_type();
        let content = message.content();

        tracing::debug!("Processing message type: {}", msg_type);

        // Handle common message types in a protocol-agnostic way
        match msg_type {
            "kernel_info_request" => Ok(self.handle_kernel_info()),
            "execute_request" => self.handle_execute(content).await,
            "shutdown_request" => self.handle_shutdown(content).await,
            "interrupt_request" => Ok(Self::handle_interrupt()),
            _ => {
                tracing::warn!("Unknown message type: {}", msg_type);
                Ok(serde_json::json!({
                    "status": "error",
                    "message": format!("Unknown message type: {}", msg_type)
                }))
            }
        }
    }

    fn handle_kernel_info(&self) -> serde_json::Value {
        serde_json::json!({
            "status": "ok",
            "protocol_version": "5.3",
            "implementation": "llmspell",
            "implementation_version": env!("CARGO_PKG_VERSION"),
            "language_info": {
                "name": self.config.engine.clone(),
                "version": "1.0.0",
                "file_extension": if self.config.engine == "lua" { ".lua" } else { ".js" }
            },
            "banner": format!("LLMSpell Kernel v{} - {}", env!("CARGO_PKG_VERSION"), self.config.engine),
            "help_links": []
        })
    }

    async fn handle_execute(&self, content: serde_json::Value) -> Result<serde_json::Value> {
        let code = content["code"].as_str().unwrap_or("");
        let silent = content["silent"].as_bool().unwrap_or(false);

        if !silent {
            *self.execution_state.write().await = KernelState::Busy;
        }

        let execution_count = {
            let mut count = self.execution_count.lock().await;
            *count += 1;
            *count
        };

        // Execute code using ScriptRuntime
        // Execute and collect output from the stream
        let mut output = String::new();
        let result = {
            let runtime = self.runtime.lock().await;
            runtime.execute_script_streaming(code).await
        };
        match result {
            Ok(mut script_stream) => {
                use futures::StreamExt;
                while let Some(chunk) = script_stream.stream.next().await {
                    match chunk {
                        Ok(agent_chunk) => {
                            // Extract text content from the chunk
                            output.push_str(&agent_chunk.content.to_string());
                        }
                        Err(e) => {
                            *self.execution_state.write().await = KernelState::Idle;
                            return Ok(serde_json::json!({
                                "status": "error",
                                "execution_count": execution_count,
                                "ename": "ExecutionError",
                                "evalue": e.to_string(),
                                "traceback": vec![e.to_string()]
                            }));
                        }
                    }
                }
            }
            Err(e) => {
                *self.execution_state.write().await = KernelState::Idle;
                return Ok(serde_json::json!({
                    "status": "error",
                    "execution_count": execution_count,
                    "ename": "ExecutionError",
                    "evalue": e.to_string(),
                    "traceback": vec![e.to_string()]
                }));
            }
        }

        *self.execution_state.write().await = KernelState::Idle;
        Ok(serde_json::json!({
            "status": "ok",
            "execution_count": execution_count,
            "user_expressions": {},
            "payload": [],
            "output": output
        }))
    }

    async fn handle_shutdown(&self, content: serde_json::Value) -> Result<serde_json::Value> {
        let restart = content["restart"].as_bool().unwrap_or(false);

        *self.execution_state.write().await = KernelState::Stopping;

        Ok(serde_json::json!({
            "status": "ok",
            "restart": restart
        }))
    }

    fn handle_interrupt() -> serde_json::Value {
        // Interrupt execution if possible
        tracing::info!("Interrupt requested");

        serde_json::json!({
            "status": "ok"
        })
    }

    /// Shutdown the kernel gracefully
    ///
    /// # Errors
    ///
    /// Returns an error if the shutdown process encounters issues.
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

        // Send shutdown signal if receiver exists
        if let Some(tx) = self.shutdown_tx {
            let _ = tx.send(());
        }

        tracing::info!("Kernel {} shutdown complete", self.kernel_id);
        Ok(())
    }
}

// === FACTORY METHOD FOR SIMPLIFIED KERNEL CREATION ===

use crate::{connection::ConnectionInfo, jupyter::JupyterProtocol, transport::ZmqTransport};

impl GenericKernel<ZmqTransport, JupyterProtocol> {
    /// Create kernel with Jupyter protocol and ZMQ transport defaults
    ///
    /// This factory method handles all the wiring internally:
    /// - Creates connection info from kernel config
    /// - Writes Jupyter connection file
    /// - Creates ZMQ transport and Jupyter protocol
    /// - Wires everything together into `GenericKernel`
    ///
    /// # Errors
    ///
    /// Returns an error if kernel creation or initialization fails.
    ///
    /// # Panics
    ///
    /// Panics if `kernel_id` is None after generation (should never happen).
    pub async fn from_config(mut config: KernelConfig) -> Result<Self> {
        // Generate kernel ID if not provided
        if config.kernel_id.is_none() {
            config.kernel_id = Some(Uuid::new_v4().to_string());
        }
        let kernel_id = config.kernel_id.clone().unwrap();

        // Create default connection info (IP and port will be handled by caller)
        let connection_info = ConnectionInfo::new(
            kernel_id.clone(),
            "127.0.0.1".to_string(), // Default IP
            9555,                    // Default starting port
        );

        // Write connection file for Jupyter clients
        connection_info.write_connection_file().await?;
        tracing::info!("Connection file written for kernel {}", kernel_id);

        // Create transport and protocol with defaults
        let transport = ZmqTransport::new()?;
        let protocol = JupyterProtocol::new(connection_info.clone());

        // Create kernel using the generic constructor
        Box::pin(Self::new(config, transport, protocol)).await
    }

    /// Create kernel with custom connection info
    ///
    /// This allows overriding IP and port settings while still
    /// getting the convenience of automatic wiring
    ///
    /// # Errors
    ///
    /// Returns an error if kernel creation fails.
    pub async fn from_config_with_connection(
        config: KernelConfig,
        connection_info: ConnectionInfo,
    ) -> Result<Self> {
        // Write connection file for Jupyter clients
        connection_info.write_connection_file().await?;
        tracing::info!(
            "Connection file written for kernel {}",
            connection_info.kernel_id
        );

        // Create transport and protocol
        let transport = ZmqTransport::new()?;
        let protocol = JupyterProtocol::new(connection_info.clone());

        // Create kernel using the generic constructor
        Box::pin(Self::new(config, transport, protocol)).await
    }
}
