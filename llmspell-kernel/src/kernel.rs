//! Core kernel service implementation
//!
//! The `GenericKernel` provides a protocol-agnostic kernel that works
//! with any Transport and Protocol implementation via traits.
//! This enables clean separation of concerns and easy extensibility.

use anyhow::Result;
use llmspell_bridge::{
    execution_bridge::{Breakpoint, DebugCommand, ExecutionManager},
    ScriptRuntime,
};
use llmspell_config::LLMSpellConfig;
use llmspell_state_persistence::{StateFactory, StateManager};
use std::sync::Arc;
use tokio::sync::{oneshot, Mutex, RwLock};
use uuid::Uuid;

use crate::client_handler::ClientManager;
use crate::comm_handler::CommManager;
use crate::security::SecurityManager;
use crate::session_persistence::SessionMapper;
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

    /// Shared configuration
    pub config: Arc<LLMSpellConfig>,

    /// Shared state manager
    pub state_manager: Option<Arc<StateManager>>,

    /// Security manager
    pub security_manager: Arc<SecurityManager>,

    /// Resource limits per client
    pub resource_limits: ClientResourceLimits,

    /// Execution counter for tracking
    pub execution_count: Arc<Mutex<u32>>,

    /// Session persistence mapper
    pub session_mapper: Arc<SessionMapper>,

    /// Comm channel manager for session management
    pub comm_manager: Arc<CommManager>,

    /// Shutdown signal sender
    shutdown_tx: Option<oneshot::Sender<()>>,

    /// Current request header for `IOPub` parent tracking
    current_request_header: Arc<RwLock<Option<serde_json::Value>>>,
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
    pub async fn new(
        kernel_id: String,
        config: Arc<LLMSpellConfig>,
        mut transport: T,
        protocol: P,
    ) -> Result<Self> {
        let kernel_id = if kernel_id.is_empty() {
            Uuid::new_v4().to_string()
        } else {
            kernel_id
        };

        tracing::info!(
            "Starting kernel {} with {} protocol and engine {}",
            kernel_id,
            protocol.name(),
            config.default_engine
        );

        // Create shared StateManager from config
        let state_manager = StateFactory::create_from_config(&config)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create state manager: {}", e))?;

        // Create script runtime from llmspell-bridge with shared state manager
        let runtime = if let Some(ref sm) = state_manager {
            ScriptRuntime::new_with_engine_and_state_manager(
                &config.default_engine,
                (*config).clone(),
                sm.clone(),
            )
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create script runtime: {}", e))?
        } else {
            // Fallback to creating runtime without external state manager
            ScriptRuntime::new_with_engine_name(&config.default_engine, (*config).clone())
                .await
                .map_err(|e| anyhow::anyhow!("Failed to create script runtime: {}", e))?
        };

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
        let security_manager = Arc::new(SecurityManager::new(
            kernel_key,
            config.runtime.kernel.auth_enabled,
        ));

        // Create client manager
        let client_manager = Arc::new(ClientManager::new(config.runtime.kernel.max_clients));

        // Create session mapper with shared state manager
        let session_mapper = Arc::new(if let Some(ref sm) = state_manager {
            SessionMapper::with_state_manager(Some(sm.clone())).await?
        } else {
            SessionMapper::with_state_manager(None).await?
        });

        // Create comm manager for session communication
        let comm_manager = Arc::new(CommManager::new(session_mapper.clone())?);

        Ok(Self {
            kernel_id,
            transport,
            protocol,
            runtime: Arc::new(Mutex::new(runtime)),
            client_manager,
            execution_state: Arc::new(RwLock::new(KernelState::Starting)),
            config,
            state_manager,
            security_manager,
            resource_limits: ClientResourceLimits::default(),
            execution_count: Arc::new(Mutex::new(0)),
            session_mapper,
            comm_manager,
            shutdown_tx: None,
            current_request_header: Arc::new(RwLock::new(None)),
        })
    }

    /// Publish a message to the `IOPub` channel
    ///
    /// # Errors
    ///
    /// Returns an error if message publishing fails
    async fn publish_iopub(&self, msg_type: &str, content: serde_json::Value) -> Result<()> {
        tracing::debug!("publish_iopub: Publishing {} message", msg_type);

        // Both conditions return None, so simplify
        let parent_msg = None;

        tracing::debug!("publish_iopub: Creating broadcast message with parent tracking");

        // Use protocol's create_broadcast method (protocol-agnostic)
        let msg = self
            .protocol
            .create_broadcast(msg_type, content, parent_msg, &self.kernel_id)?;

        // Encode the message for sending
        let parts = self.protocol.encode(&msg, "iopub")?;

        // Send via transport on IOPub channel
        self.transport.send("iopub", parts).await?;
        tracing::debug!(
            "publish_iopub: Successfully published {} message to IOPub",
            msg_type
        );
        Ok(())
    }

    /// Publish execution status to `IOPub`
    async fn publish_status(&self, status: &str) -> Result<()> {
        self.publish_iopub(
            "status",
            serde_json::json!({
                "execution_state": status
            }),
        )
        .await
    }

    /// Publish stream output to `IOPub`
    async fn publish_stream(&self, name: &str, text: &str) -> Result<()> {
        self.publish_iopub(
            "stream",
            serde_json::json!({
                "name": name,
                "text": text
            }),
        )
        .await
    }

    /// Publish execute input echo to `IOPub`
    async fn publish_execute_input(&self, code: &str, execution_count: u32) -> Result<()> {
        self.publish_iopub(
            "execute_input",
            serde_json::json!({
                "code": code,
                "execution_count": execution_count
            }),
        )
        .await
    }

    /// Publish execute result to `IOPub`
    async fn publish_execute_result(
        &self,
        execution_count: u32,
        data: serde_json::Value,
    ) -> Result<()> {
        self.publish_iopub(
            "execute_result",
            serde_json::json!({
                "execution_count": execution_count,
                "data": data,
                "metadata": {}
            }),
        )
        .await
    }

    /// Publish error to `IOPub`
    async fn publish_error(&self, ename: &str, evalue: &str, traceback: Vec<String>) -> Result<()> {
        self.publish_iopub(
            "error",
            serde_json::json!({
                "ename": ename,
                "evalue": evalue,
                "traceback": traceback
            }),
        )
        .await
    }

    /// Run the kernel - protocol-agnostic main loop
    ///
    /// # Errors
    ///
    /// Returns an error if the kernel fails to serve messages or encounters a fatal error.
    pub async fn serve(&mut self) -> Result<()> {
        self.log_serving_info();
        *self.execution_state.write().await = KernelState::Idle;

        loop {
            if self.process_available_messages().await? {
                tracing::info!("Shutdown requested");
                return Ok(());
            }

            self.transport.heartbeat().await?;
            tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
        }
    }

    fn log_serving_info(&self) {
        tracing::info!(
            "Kernel {} serving with {} protocol (version {})",
            self.kernel_id,
            self.protocol.name(),
            self.protocol.version()
        );
    }

    /// Process messages from all channels
    /// Returns true if shutdown was requested
    async fn process_available_messages(&self) -> Result<bool> {
        // Process control channel first for priority (shutdown/interrupt)
        if let Some(shutdown_requested) = self.process_channel_message("control").await? {
            return Ok(shutdown_requested);
        }

        // Then process other channels
        for channel in self.transport.channels() {
            if Self::should_skip_channel(&channel) {
                continue;
            }

            if let Some(shutdown_requested) = self.process_channel_message(&channel).await? {
                return Ok(shutdown_requested);
            }
        }
        Ok(false)
    }

    fn should_skip_channel(channel: &str) -> bool {
        // Skip iopub (output only), heartbeat (handled separately), and control (already processed)
        channel == "iopub" || channel == "heartbeat" || channel == "control"
    }

    /// Process a single channel's message
    /// Returns Some(true) if shutdown was requested
    async fn process_channel_message(&self, channel: &str) -> Result<Option<bool>> {
        if let Some(parts) = self.transport.recv(channel).await? {
            let message = self.protocol.decode(parts, channel)?;
            self.store_request_header(&message).await?;

            let shutdown_requested = self.handle_message_and_reply(message, channel).await?;
            return Ok(Some(shutdown_requested));
        }
        Ok(None)
    }

    async fn store_request_header(&self, message: &P::Message) -> Result<()> {
        if let Some(header) = message.header_for_parent() {
            *self.current_request_header.write().await = Some(header);
        }
        Ok(())
    }

    async fn handle_message_and_reply(&self, message: P::Message, channel: &str) -> Result<bool> {
        let is_shutdown = message.msg_type() == "shutdown_request";
        let should_reply = self.protocol.requires_reply(&message);

        // Get execution flow from protocol
        let flow = self.protocol.create_execution_flow(&message);

        // Send pre-execution messages
        for (channel, msg) in flow.pre_execution {
            let parts = self.protocol.encode(&msg, &channel)?;
            self.transport.send(&channel, parts).await?;
        }

        // Process the message (potentially with output capture)
        let reply_content = if flow.capture_output {
            // For now, process normally - OutputCapture integration will come in 9.8.13.3.5
            self.process_message(message.clone()).await?
        } else {
            self.process_message(message.clone()).await?
        };

        // Send the reply if needed
        if should_reply {
            self.send_reply(&message, reply_content, channel).await?;
        }

        // Send post-execution messages (including status:idle)
        for (channel, msg) in flow.post_execution {
            let parts = self.protocol.encode(&msg, &channel)?;
            self.transport.send(&channel, parts).await?;
        }

        // Send status:idle after execute_reply for execute_request
        if message.msg_type() == "execute_request" {
            if let Ok(idle_msg) = self
                .protocol
                .create_status_message(crate::traits::KernelStatus::Idle)
            {
                // Set parent header for the status message
                let mut idle_with_parent = idle_msg;
                if let Some(header) = message.header_for_parent() {
                    idle_with_parent.set_parent_from_json(header);
                }
                let parts = self.protocol.encode(&idle_with_parent, "iopub")?;
                self.transport.send("iopub", parts).await?;
            }
        }

        Ok(is_shutdown)
    }

    async fn send_reply(
        &self,
        message: &P::Message,
        reply_content: serde_json::Value,
        channel: &str,
    ) -> Result<()> {
        let reply = self.protocol.create_reply(message, reply_content)?;
        let reply_parts = self.protocol.encode(&reply, channel)?;
        let reply_channel = self.protocol.reply_channel(&reply);
        self.transport.send(reply_channel, reply_parts).await
    }

    /// Process a message - protocol-agnostic
    async fn process_message(&self, message: P::Message) -> Result<serde_json::Value> {
        let msg_type = message.msg_type();
        let content = message.content();

        tracing::debug!("Processing message type: {}", msg_type);

        // Handle common message types in a protocol-agnostic way
        match msg_type {
            "kernel_info_request" => Ok(self.handle_kernel_info()),
            "execute_request" => self.handle_execute(content).await,
            "shutdown_request" => self.handle_shutdown(content).await,
            "interrupt_request" => Ok(Self::handle_interrupt()),
            "comm_open" => self.handle_comm_open(content).await,
            "comm_msg" => self.handle_comm_msg(content).await,
            "comm_close" => self.handle_comm_close(content).await,
            "comm_info_request" => self.handle_comm_info_request(content).await,
            _ => {
                tracing::warn!("Unknown message type: {}", msg_type);
                Ok(serde_json::json!({
                    "status": "error",
                    "message": format!("Unknown message type: {}", msg_type)
                }))
            }
        }
    }

    pub fn handle_kernel_info(&self) -> serde_json::Value {
        // Include session metadata extensions for llmspell
        let session_metadata = serde_json::json!({
            "persistence_enabled": true,
            "session_mapper": "llmspell-sessions",
            "state_backend": "llmspell-state-persistence",
            "comm_targets": [
                crate::comm_handler::SESSION_COMM_TARGET,
                crate::comm_handler::STATE_COMM_TARGET,
            ],
            "max_clients": self.config.runtime.kernel.max_clients,
            "kernel_id": self.kernel_id.clone(),
        });

        serde_json::json!({
            "status": "ok",
            "protocol_version": "5.3",
            "implementation": "llmspell",
            "implementation_version": env!("CARGO_PKG_VERSION"),
            "language_info": {
                "name": self.config.default_engine.clone(),
                "version": "1.0.0",
                "mimetype": match self.config.default_engine.as_str() {
                    "lua" => "text/x-lua",
                    "javascript" | "js" => "text/javascript",
                    "python" | "py" => "text/x-python",
                    _ => "text/plain"
                },
                "file_extension": match self.config.default_engine.as_str() {
                    "lua" => ".lua",
                    "javascript" | "js" => ".js",
                    "python" | "py" => ".py",
                    _ => ".txt"
                }
            },
            "banner": format!("LLMSpell Kernel v{} - {}", env!("CARGO_PKG_VERSION"), self.config.default_engine),
            "help_links": [],
            // LLMSpell extensions
            "llmspell_session_metadata": session_metadata
        })
    }

    async fn handle_execute(&self, content: serde_json::Value) -> Result<serde_json::Value> {
        let (code, silent, script_args, execution_count) = self.setup_execution(&content).await?;

        if !silent {
            self.publish_execution_start(code, execution_count).await;
        }

        let output = self
            .execute_code_streaming(code, silent, script_args, execution_count)
            .await?;

        self.finalize_execution(silent, execution_count, output)
            .await
    }

    /// Setup execution parameters and session state
    async fn setup_execution<'a>(
        &self,
        content: &'a serde_json::Value,
    ) -> Result<(&'a str, bool, Option<Vec<String>>, u32)> {
        let code = content["code"].as_str().unwrap_or("");
        let silent = content["silent"].as_bool().unwrap_or(false);
        let script_args = content["script_args"].as_array().map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        });

        // Get session ID from the message metadata (if available)
        let session_id = content["metadata"]["session_id"]
            .as_str()
            .unwrap_or(&self.kernel_id);

        // Get or create session for this execution
        let llmspell_session_id = self
            .session_mapper
            .get_or_create_session(session_id, &self.kernel_id)
            .await?;

        let execution_count = {
            let mut count = self.execution_count.lock().await;
            *count += 1;
            *count
        };

        // Store execution count in session state
        self.session_mapper
            .store_execution_count(&llmspell_session_id, execution_count)
            .await?;

        Ok((code, silent, script_args, execution_count))
    }

    /// Publish execution start notifications
    async fn publish_execution_start(&self, code: &str, execution_count: u32) {
        // Set state to busy
        *self.execution_state.write().await = KernelState::Busy;

        // Publish status update to IOPub
        let _ = self.publish_status("busy").await;

        // Publish execute_input to IOPub
        let _ = self.publish_execute_input(code, execution_count).await;
    }

    /// Execute code with streaming output
    async fn execute_code_streaming(
        &self,
        code: &str,
        silent: bool,
        script_args: Option<Vec<String>>,
        execution_count: u32,
    ) -> Result<String> {
        let mut output = String::new();
        let result = {
            let mut runtime = self.runtime.lock().await;
            // Set script arguments if provided
            if let Some(args) = script_args {
                // Convert Vec<String> to HashMap<String, String> for runtime
                let mut args_map = std::collections::HashMap::new();
                // Set script name as arg[0]
                args_map.insert("0".to_string(), "script".to_string());
                // Add positional arguments starting from 1
                for (i, arg) in args.iter().enumerate() {
                    args_map.insert((i + 1).to_string(), arg.clone());
                }

                if let Err(e) = runtime.set_script_args(args_map).await {
                    tracing::warn!("Failed to set script arguments: {}", e);
                }
            }
            runtime.execute_script_streaming(code).await
        };

        match result {
            Ok(mut script_stream) => {
                use futures::StreamExt;
                while let Some(chunk) = script_stream.stream.next().await {
                    match chunk {
                        Ok(agent_chunk) => {
                            // Extract text content from the chunk
                            let chunk_text = agent_chunk.content.to_string();
                            output.push_str(&chunk_text);

                            // Stream output to IOPub in real-time if not silent
                            if !silent && !chunk_text.is_empty() {
                                let _ = self.publish_stream("stdout", &chunk_text).await;
                            }
                        }
                        Err(e) => {
                            return self
                                .handle_execution_error(e.into(), silent, execution_count)
                                .await;
                        }
                    }
                }
            }
            Err(e) => {
                return self
                    .handle_execution_error(e.into(), silent, execution_count)
                    .await;
            }
        }

        Ok(output)
    }

    /// Handle execution errors consistently
    async fn handle_execution_error(
        &self,
        e: anyhow::Error,
        silent: bool,
        execution_count: u32,
    ) -> Result<String> {
        // Publish error to IOPub if not silent
        if !silent {
            let _ = self
                .publish_error("ExecutionError", &e.to_string(), vec![e.to_string()])
                .await;
            let _ = self.publish_status("idle").await;
        }

        *self.execution_state.write().await = KernelState::Idle;
        Err(anyhow::anyhow!(serde_json::json!({
            "status": "error",
            "execution_count": execution_count,
            "ename": "ExecutionError",
            "evalue": e.to_string(),
            "traceback": vec![e.to_string()]
        })))
    }

    /// Finalize execution and return response
    async fn finalize_execution(
        &self,
        silent: bool,
        execution_count: u32,
        output: String,
    ) -> Result<serde_json::Value> {
        // Publish execute_result to IOPub if we have output and not silent
        if !silent && !output.is_empty() {
            let _ = self
                .publish_execute_result(
                    execution_count,
                    serde_json::json!({
                        "text/plain": output.clone()
                    }),
                )
                .await;
        }

        // Set state back to idle and publish status
        *self.execution_state.write().await = KernelState::Idle;
        if !silent {
            let _ = self.publish_status("idle").await;
        }

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

        // Save sessions and mark clean shutdown
        self.save_sessions_on_shutdown().await;

        Ok(serde_json::json!({
            "status": "ok",
            "restart": restart
        }))
    }

    async fn save_sessions_on_shutdown(&self) {
        if self.state_manager.is_none() {
            return;
        }

        self.save_all_sessions_safely().await;
        self.mark_clean_shutdown_safely().await;
    }

    async fn save_all_sessions_safely(&self) {
        if let Err(e) = self.session_mapper.save_all_sessions().await {
            tracing::error!("Failed to save sessions on shutdown request: {}", e);
        } else {
            tracing::info!("Saved sessions on shutdown request");
        }
    }

    async fn mark_clean_shutdown_safely(&self) {
        if let Err(e) = self.session_mapper.mark_clean_shutdown().await {
            tracing::error!("Failed to mark clean shutdown: {}", e);
        }
    }

    fn handle_interrupt() -> serde_json::Value {
        // Interrupt execution if possible
        tracing::info!("Interrupt requested");

        serde_json::json!({
            "status": "ok"
        })
    }

    async fn handle_comm_open(&self, content: serde_json::Value) -> Result<serde_json::Value> {
        tracing::debug!("handle_comm_open received content: {:?}", content);
        let comm_id = content["comm_id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing comm_id"))?;
        let target_name = content["target_name"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing target_name"))?;

        // Get session from message metadata (would normally come from header)
        let session_id = content["session"].as_str().unwrap_or("default-session");

        self.comm_manager
            .open_comm(
                comm_id.to_string(),
                target_name.to_string(),
                session_id,
                &self.kernel_id,
            )
            .await?;

        tracing::debug!("Opened comm {} with target {}", comm_id, target_name);

        // Send comm_open reply on IOPub to acknowledge the comm channel
        // This lets the client know we accepted the comm
        let _ = self
            .publish_iopub(
                "comm_open",
                serde_json::json!({
                    "comm_id": comm_id,
                    "target_name": target_name,
                    "data": {
                        "status": "ready",
                        "session_id": session_id,
                        "kernel_id": self.kernel_id,
                        "capabilities": ["session_artifacts", "state_access"]
                    }
                }),
            )
            .await;

        // For llmspell.session target, send initial session info
        if target_name == crate::comm_handler::SESSION_COMM_TARGET {
            // Get the current session info
            if let Some(session_state) = self.session_mapper.get_session(&self.kernel_id).await {
                let _ = self
                    .publish_iopub(
                        "comm_msg",
                        serde_json::json!({
                            "comm_id": comm_id,
                            "data": {
                                "type": "session_info",
                                "session_id": session_state.session_id.to_string(),
                                "jupyter_id": session_state.jupyter_id,
                                "kernel_id": session_state.kernel_id,
                                "execution_count": session_state.execution_count,
                                "created_at": session_state.created_at.to_string()
                            }
                        }),
                    )
                    .await;
            }
        }

        Ok(serde_json::json!({
            "status": "ok"
        }))
    }

    async fn handle_comm_msg(&self, content: serde_json::Value) -> Result<serde_json::Value> {
        let comm_id = content["comm_id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing comm_id"))?;
        let data = content["data"].clone();

        // Handle comm message and get response
        let response = self.comm_manager.handle_comm_msg(comm_id, data).await?;

        // Send response back via IOPub comm_msg
        let _ = self
            .publish_iopub(
                "comm_msg",
                serde_json::json!({
                    "comm_id": comm_id,
                    "data": response
                }),
            )
            .await;

        Ok(serde_json::to_value(response)?)
    }

    async fn handle_comm_close(&self, content: serde_json::Value) -> Result<serde_json::Value> {
        let comm_id = content["comm_id"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Missing comm_id"))?;

        self.comm_manager.close_comm(comm_id).await?;

        tracing::debug!("Closed comm {}", comm_id);

        Ok(serde_json::json!({
            "status": "ok"
        }))
    }

    async fn handle_comm_info_request(
        &self,
        content: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let target_name = content["target_name"].as_str();

        let comms = self.comm_manager.get_comm_info(target_name).await;

        Ok(serde_json::json!({
            "status": "ok",
            "comms": comms
        }))
    }

    /// Handle debug requests via existing `ExecutionManager` API
    ///
    /// Routes debug commands to the `ScriptRuntime`'s `ExecutionManager`, providing
    /// a unified interface for debugging functionality regardless of kernel type.
    ///
    /// # Errors
    ///
    /// Returns an error if debug is not enabled or if the debug command fails.
    #[allow(clippy::significant_drop_tightening)] // Runtime lock is needed for entire match
    pub async fn handle_debug_request(
        &self,
        content: serde_json::Value,
    ) -> Result<serde_json::Value> {
        // Access ExecutionManager through ScriptRuntime
        let runtime = self.runtime.lock().await;
        let exec_mgr = runtime
            .get_execution_manager()
            .ok_or_else(|| anyhow::anyhow!("Debug not enabled - use --debug flag"))?;

        // Use DAP bridge for standard DAP protocol support
        let dap_bridge = crate::dap_bridge::DAPBridge::new(exec_mgr.clone());
        
        // If this looks like a DAP request, use the bridge
        if content.get("type").and_then(|t| t.as_str()) == Some("request") {
            return dap_bridge.handle_request(content).await;
        }
        
        // Otherwise fall back to legacy command handling for backward compatibility
        let command = content["command"].as_str().unwrap_or("");
        let args = &content["arguments"];

        match command {
            "setBreakpoints" => self.handle_set_breakpoints(exec_mgr.as_ref(), args).await,
            "continue" => self.handle_debug_continue(exec_mgr.as_ref()).await,
            "stepIn" => self.handle_debug_step_in(exec_mgr.as_ref()).await,
            "stepOver" => self.handle_debug_step_over(exec_mgr.as_ref()).await,
            "stepOut" => self.handle_debug_step_out(exec_mgr.as_ref()).await,
            "getVariables" => self.handle_get_variables(exec_mgr.as_ref(), args).await,
            "getStack" => self.handle_get_stack(exec_mgr.as_ref()).await,
            _ => Err(anyhow::anyhow!("Unknown debug command: {}", command)),
        }
    }

    async fn handle_set_breakpoints(
        &self,
        exec_mgr: &ExecutionManager,
        args: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        let source = args["source"]["name"].as_str().unwrap_or("repl");
        let mut breakpoint_ids = Vec::new();

        if let Some(lines) = args["lines"].as_array() {
            for line in lines {
                if let Some(line_num) = line.as_u64() {
                    let bp =
                        Breakpoint::new(source.to_string(), u32::try_from(line_num).unwrap_or(0));
                    let id = exec_mgr.add_breakpoint(bp).await;
                    breakpoint_ids.push(id);
                }
            }
        }
        Ok(serde_json::json!({
            "success": true,
            "breakpoints": breakpoint_ids
        }))
    }

    async fn handle_debug_continue(
        &self,
        exec_mgr: &ExecutionManager,
    ) -> Result<serde_json::Value> {
        exec_mgr.send_command(DebugCommand::Continue).await;
        Ok(serde_json::json!({"success": true}))
    }

    async fn handle_debug_step_in(&self, exec_mgr: &ExecutionManager) -> Result<serde_json::Value> {
        exec_mgr.send_command(DebugCommand::StepInto).await;
        Ok(serde_json::json!({"success": true}))
    }

    async fn handle_debug_step_over(
        &self,
        exec_mgr: &ExecutionManager,
    ) -> Result<serde_json::Value> {
        exec_mgr.send_command(DebugCommand::StepOver).await;
        Ok(serde_json::json!({"success": true}))
    }

    async fn handle_debug_step_out(
        &self,
        exec_mgr: &ExecutionManager,
    ) -> Result<serde_json::Value> {
        exec_mgr.send_command(DebugCommand::StepOut).await;
        Ok(serde_json::json!({"success": true}))
    }

    async fn handle_get_variables(
        &self,
        exec_mgr: &ExecutionManager,
        args: &serde_json::Value,
    ) -> Result<serde_json::Value> {
        let frame_id = args["frameId"].as_str();
        let variables = exec_mgr.get_variables(frame_id).await;
        Ok(serde_json::json!({
            "success": true,
            "variables": variables
        }))
    }

    async fn handle_get_stack(&self, exec_mgr: &ExecutionManager) -> Result<serde_json::Value> {
        let stack = exec_mgr.get_stack_trace().await;
        Ok(serde_json::json!({
            "success": true,
            "stackFrames": stack
        }))
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

        // Save sessions and mark clean shutdown
        self.save_sessions_on_shutdown().await;

        // Disconnect all clients
        self.disconnect_all_clients().await;

        // Send shutdown signal if receiver exists
        if let Some(tx) = self.shutdown_tx {
            let _ = tx.send(());
        }

        tracing::info!("Kernel {} shutdown complete", self.kernel_id);
        Ok(())
    }

    async fn disconnect_all_clients(&self) {
        let all_clients = self.client_manager.get_all_clients().await;
        for client in all_clients {
            tracing::info!("Disconnecting client {}", client.client_id);
            self.client_manager.remove_client(&client.client_id).await;
        }
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
    pub async fn from_config(
        kernel_id: Option<String>,
        config: Arc<LLMSpellConfig>,
    ) -> Result<Self> {
        // Generate kernel ID if not provided
        let kernel_id = kernel_id.unwrap_or_else(|| Uuid::new_v4().to_string());

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
        Box::pin(Self::new(kernel_id, config, transport, protocol)).await
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
        kernel_id: String,
        config: Arc<LLMSpellConfig>,
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
        Box::pin(Self::new(kernel_id, config, transport, protocol)).await
    }
}
