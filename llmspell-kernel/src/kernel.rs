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
use llmspell_sessions::{SessionManager, SessionManagerConfig};
use llmspell_state_persistence::{StateFactory, StateManager};
use std::sync::{Arc, Mutex as StdMutex};
use tokio::sync::{oneshot, Mutex, RwLock};
use uuid::Uuid;

use crate::callback_io::create_callback_io_context;
use crate::client_handler::ClientManager;
use crate::comm_handler::CommManager;
use crate::kernel_io::KernelSignalHandler;
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
    /// Shared session manager
    pub session_manager: Option<Arc<SessionManager>>,
    /// RAG pipeline
    pub rag_pipeline: Option<Arc<llmspell_rag::pipeline::RAGPipeline>>,

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

    /// Signal handler for interrupt propagation
    signal_handler: Arc<KernelSignalHandler>,

    /// Current request message for `IOPub` parent tracking
    current_request_message: Arc<RwLock<Option<P::Message>>>,
}

impl<T: Transport, P: Protocol> GenericKernel<T, P> {
    /// Create RAG pipeline based on configuration
    fn create_rag_pipeline(
        config: &Arc<LLMSpellConfig>,
        _state_manager: Option<&Arc<StateManager>>,
    ) -> Result<Option<Arc<llmspell_rag::pipeline::RAGPipeline>>> {
        use llmspell_rag::{
            embeddings::{CacheConfig, EmbeddingCache, EmbeddingFactory, EmbeddingProviderConfig},
            pipeline::{config::RAGConfig, RAGPipeline},
        };
        use llmspell_storage::{
            backends::vector::HNSWVectorStorage,
            vector_storage::{HNSWConfig, VectorStorage},
        };

        if !config.rag.enabled {
            return Ok(None);
        }

        // Create vector storage based on config
        // Currently only HNSW backend is implemented
        let dimensions = config.rag.vector_storage.dimensions;
        let hnsw_config = HNSWConfig::default();
        let storage: Arc<dyn VectorStorage> =
            Arc::new(HNSWVectorStorage::new(dimensions, hnsw_config)) as Arc<dyn VectorStorage>;

        // Create embedding factory
        let embedding_config = EmbeddingProviderConfig::default();
        let embedding_factory = Arc::new(EmbeddingFactory::new(embedding_config));

        // Create embedding cache
        let cache_config = CacheConfig::default();
        let embedding_cache = Arc::new(EmbeddingCache::new(cache_config));

        // Create RAG pipeline
        let rag_config = RAGConfig::default();
        let pipeline = RAGPipeline::new(rag_config, storage, embedding_factory, embedding_cache)
            .map_err(|e| anyhow::anyhow!("Failed to create RAG pipeline: {e}"))?;

        Ok(Some(Arc::new(pipeline)))
    }

    /// Create storage backend based on configuration
    fn create_storage_backend(
        backend_type: &str,
    ) -> Result<Arc<dyn llmspell_storage::StorageBackend>> {
        tracing::trace!("Creating storage backend: {}", backend_type);

        if backend_type == "sled" {
            tracing::trace!("Creating sled backend");
            let sled_backend = llmspell_storage::SledBackend::new()
                .map_err(|e| anyhow::anyhow!("Failed to create sled backend: {}", e))?;
            Ok(Arc::new(sled_backend))
        } else {
            tracing::trace!("Creating memory backend");
            Ok(Arc::new(llmspell_storage::MemoryBackend::new()))
        }
    }

    /// Create session manager dependencies
    fn create_session_dependencies() -> (
        Arc<llmspell_hooks::registry::HookRegistry>,
        Arc<llmspell_hooks::executor::HookExecutor>,
        Arc<llmspell_events::bus::EventBus>,
        SessionManagerConfig,
    ) {
        tracing::trace!("Creating hook registry and executor");
        let hook_registry = Arc::new(llmspell_hooks::registry::HookRegistry::new());
        let hook_executor = Arc::new(llmspell_hooks::executor::HookExecutor::new());
        tracing::trace!("Creating event bus");
        let event_bus = Arc::new(llmspell_events::bus::EventBus::new());
        let session_config = SessionManagerConfig::default();

        (hook_registry, hook_executor, event_bus, session_config)
    }

    /// Get or create state manager for session manager
    async fn get_or_create_state_manager(
        config: &Arc<LLMSpellConfig>,
        state_manager: Option<&Arc<StateManager>>,
    ) -> Result<Arc<StateManager>> {
        if let Some(state_mgr) = state_manager {
            tracing::trace!("Using existing state manager");
            Ok(state_mgr.clone())
        } else {
            // Create a temporary StateManager for SessionManager
            let temp_state = StateFactory::create_from_config(config).await?;
            match temp_state {
                Some(sm) => Ok(sm),
                None => {
                    // Create a default in-memory StateManager
                    Ok(Arc::new(StateManager::new().await.map_err(|e| {
                        anyhow::anyhow!("Failed to create StateManager: {e}")
                    })?))
                }
            }
        }
    }

    /// Create a session manager based on configuration
    /// NOTE: Currently unused - `ScriptRuntime` creates its own `SessionManager`
    #[allow(dead_code)]
    async fn create_session_manager(
        config: &Arc<LLMSpellConfig>,
        state_manager: Option<&Arc<StateManager>>,
    ) -> Result<Option<Arc<SessionManager>>> {
        tracing::trace!(
            "create_session_manager: sessions.enabled = {}",
            config.runtime.sessions.enabled
        );

        if !config.runtime.sessions.enabled {
            tracing::trace!("Sessions disabled, returning None");
            return Ok(None);
        }

        // Create storage backend
        let storage_backend =
            Self::create_storage_backend(&config.runtime.sessions.storage_backend)?;

        // Create dependencies
        let (hook_registry, hook_executor, event_bus, session_config) =
            Self::create_session_dependencies();

        // Get or create state manager
        let state_mgr = Self::get_or_create_state_manager(config, state_manager).await?;

        tracing::trace!(
            "Creating SessionManager instance, state_manager available: {}",
            state_manager.is_some()
        );

        // Create SessionManager
        let sm = SessionManager::new(
            state_mgr,
            storage_backend,
            hook_registry,
            hook_executor,
            &event_bus,
            session_config,
        )?;

        Ok(Some(Arc::new(sm)))
    }

    /// Initialize kernel components
    async fn initialize_kernel_components(
        config: &Arc<LLMSpellConfig>,
    ) -> Result<(
        Option<Arc<StateManager>>,
        ScriptRuntime,
        Arc<SessionMapper>,
        Arc<CommManager>,
    )> {
        Box::pin(Self::initialize_kernel_components_with_provider_manager(
            config, None,
        ))
        .await
    }

    async fn initialize_kernel_components_with_provider_manager(
        config: &Arc<LLMSpellConfig>,
        provider_manager: Option<Arc<llmspell_bridge::ProviderManager>>,
    ) -> Result<(
        Option<Arc<StateManager>>,
        ScriptRuntime,
        Arc<SessionMapper>,
        Arc<CommManager>,
    )> {
        // Create shared StateManager from config
        let state_manager = StateFactory::create_from_config(config)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create state manager: {}", e))?;

        // Create script runtime with optional provider manager
        let runtime =
            Self::create_runtime(config, state_manager.as_ref(), provider_manager.as_ref()).await?;

        // Create session mapper
        let session_mapper = Arc::new(if let Some(ref sm) = state_manager {
            SessionMapper::with_state_manager(Some(sm.clone())).await?
        } else {
            SessionMapper::with_state_manager(None).await?
        });

        // Create comm manager
        let comm_manager = Arc::new(CommManager::new(session_mapper.clone())?);

        Ok((state_manager, runtime, session_mapper, comm_manager))
    }

    /// Create script runtime with optional state manager and provider manager
    async fn create_runtime(
        config: &Arc<LLMSpellConfig>,
        state_manager: Option<&Arc<StateManager>>,
        provider_manager: Option<&Arc<llmspell_bridge::ProviderManager>>,
    ) -> Result<ScriptRuntime> {
        if let Some(sm) = state_manager {
            if let Some(pm) = provider_manager {
                // Both StateManager and ProviderManager provided
                ScriptRuntime::new_with_managers(
                    &config.default_engine,
                    (**config).clone(),
                    sm.clone(),
                    pm.clone(),
                )
                .await
                .map_err(|e| anyhow::anyhow!("Failed to create script runtime: {}", e))
            } else {
                // Only StateManager provided
                ScriptRuntime::new_with_engine_and_state_manager(
                    &config.default_engine,
                    (**config).clone(),
                    sm.clone(),
                )
                .await
                .map_err(|e| anyhow::anyhow!("Failed to create script runtime: {}", e))
            }
        } else {
            ScriptRuntime::new_with_engine_name(&config.default_engine, (**config).clone())
                .await
                .map_err(|e| anyhow::anyhow!("Failed to create script runtime: {}", e))
        }
    }

    /// Set up transport and security
    async fn setup_transport_and_security(
        transport: &mut T,
        protocol: &P,
        config: &Arc<LLMSpellConfig>,
    ) -> Result<Arc<SecurityManager>> {
        // Get transport configuration from protocol
        let transport_config = protocol.transport_config();

        // Bind transport to addresses
        transport.bind(&transport_config).await?;

        // Create security manager with kernel key
        let kernel_key = Uuid::new_v4().to_string();
        Ok(Arc::new(SecurityManager::new(
            kernel_key,
            config.runtime.kernel.auth_enabled,
        )))
    }

    /// Create a new kernel with given transport and protocol
    ///
    /// # Errors
    ///
    /// Returns an error if kernel initialization fails.
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

        // Initialize kernel components
        let (state_manager, runtime, session_mapper, comm_manager) =
            Box::pin(Self::initialize_kernel_components(&config)).await?;

        // Setup transport and security
        let security_manager =
            Self::setup_transport_and_security(&mut transport, &protocol, &config).await?;

        tracing::info!(
            "Kernel {} bound to {} channels",
            kernel_id,
            transport.channels().len()
        );

        // Create client manager
        let client_manager = Arc::new(ClientManager::new(config.runtime.kernel.max_clients));

        // Get managers from runtime
        let session_manager = runtime.get_session_manager();
        let runtime_state_manager = runtime.get_state_manager();

        // Use runtime's state manager if available, otherwise use kernel's
        let final_state_manager = runtime_state_manager.or(state_manager);

        // Create RAG pipeline
        let rag_pipeline = Self::create_rag_pipeline(&config, final_state_manager.as_ref())?;

        tracing::trace!("Kernel constructor complete, returning kernel instance");
        Ok(Self {
            kernel_id,
            transport,
            protocol,
            runtime: Arc::new(Mutex::new(runtime)),
            client_manager,
            execution_state: Arc::new(RwLock::new(KernelState::Starting)),
            config,
            state_manager: final_state_manager,
            session_manager,
            rag_pipeline,
            security_manager,
            resource_limits: ClientResourceLimits::default(),
            execution_count: Arc::new(Mutex::new(0)),
            session_mapper,
            comm_manager,
            shutdown_tx: None,
            signal_handler: Arc::new(KernelSignalHandler::new()),
            current_request_message: Arc::new(RwLock::new(None)),
        })
    }

    /// Publish a message to the `IOPub` channel
    ///
    /// # Errors
    ///
    /// Returns an error if message publishing fails
    pub async fn publish_iopub(&self, msg_type: &str, content: serde_json::Value) -> Result<()> {
        tracing::debug!("publish_iopub: Publishing {} message", msg_type);

        // Get the current request message for parent tracking
        let parent_msg = self.current_request_message.read().await.as_ref().cloned();

        tracing::debug!("publish_iopub: Creating broadcast message with parent tracking");

        // Use protocol's create_broadcast method (protocol-agnostic)
        let msg = self.protocol.create_broadcast(
            msg_type,
            content,
            parent_msg.as_ref(),
            &self.kernel_id,
        )?;

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
    ///
    /// # Errors
    ///
    /// Returns an error if publishing to the `IOPub` channel fails
    pub async fn publish_stream(&self, name: &str, text: &str) -> Result<()> {
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
        self.initialize_serving().await;
        self.run_message_loop().await
    }

    /// Initialize kernel for serving
    async fn initialize_serving(&self) {
        tracing::trace!("Kernel.serve() called for kernel {}", self.kernel_id);
        self.log_serving_info();
        tracing::trace!("Setting execution state to Idle");
        *self.execution_state.write().await = KernelState::Idle;
        tracing::trace!("Starting kernel message loop");
    }

    /// Main message processing loop
    async fn run_message_loop(&self) -> Result<()> {
        loop {
            if self.process_message_cycle().await? {
                return Ok(());
            }
        }
    }

    /// Process one cycle of the message loop
    async fn process_message_cycle(&self) -> Result<bool> {
        if self.process_available_messages().await? {
            tracing::info!("Shutdown requested");
            return Ok(true);
        }

        self.transport.heartbeat().await?;
        tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
        Ok(false)
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
        // Process control channel first for priority
        if self.check_control_channel().await? {
            return Ok(true);
        }

        // Process other channels
        self.check_other_channels().await
    }

    /// Check control channel for messages
    async fn check_control_channel(&self) -> Result<bool> {
        Ok(self
            .process_channel_message("control")
            .await?
            .unwrap_or(false))
    }

    /// Check non-control channels for messages
    async fn check_other_channels(&self) -> Result<bool> {
        for channel in self.transport.channels() {
            if Self::should_skip_channel(&channel) {
                continue;
            }

            if let Some(shutdown) = self.process_channel_message(&channel).await? {
                return Ok(shutdown);
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
        let Some(parts) = self.transport.recv(channel).await? else {
            return Ok(None);
        };

        let message = self.protocol.decode(parts, channel)?;
        self.store_request_header(&message).await?;
        let shutdown = self.handle_message_and_reply(message, channel).await?;
        Ok(Some(shutdown))
    }

    async fn store_request_header(&self, message: &P::Message) -> Result<()> {
        // Store the entire message for parent tracking
        *self.current_request_message.write().await = Some(message.clone());
        Ok(())
    }

    async fn handle_message_and_reply(&self, message: P::Message, channel: &str) -> Result<bool> {
        let msg_type = message.msg_type();
        let should_shutdown = self.check_shutdown_conditions(msg_type, channel);

        self.log_message_receipt(msg_type, channel);
        self.execute_message_flow(&message, channel).await?;
        self.handle_post_execution(msg_type, &message).await?;

        Ok(should_shutdown)
    }

    /// Execute the message flow (pre-execution, process, reply, post-execution)
    async fn execute_message_flow(&self, message: &P::Message, channel: &str) -> Result<()> {
        let flow = self.protocol.create_execution_flow(message);

        // Send pre-execution messages
        self.send_flow_messages(flow.pre_execution).await?;

        // Process and optionally reply
        let reply_content = self.process_message(message.clone()).await?;
        if self.protocol.requires_reply(message) {
            self.send_reply(message, reply_content, channel).await?;
        }

        // Send post-execution messages
        self.send_flow_messages(flow.post_execution).await?;
        Ok(())
    }

    /// Handle post-execution tasks based on message type
    async fn handle_post_execution(&self, msg_type: &str, message: &P::Message) -> Result<()> {
        if msg_type == "execute_request" {
            self.send_execute_idle_status(message).await?;
        }
        Ok(())
    }

    /// Check shutdown conditions based on message type and channel
    fn check_shutdown_conditions(&self, msg_type: &str, channel: &str) -> bool {
        if msg_type != "shutdown_request" {
            return false;
        }

        tracing::info!("Kernel {} shutdown requested", self.kernel_id);

        // In debug mode, only shutdown from control channel
        !self.config.debug.enabled || channel == "control"
    }

    /// Log message receipt
    fn log_message_receipt(&self, msg_type: &str, channel: &str) {
        tracing::info!(
            "Kernel {} received message: {} on channel: {}",
            self.kernel_id,
            msg_type,
            channel
        );
    }

    /// Send flow messages (pre or post execution)
    async fn send_flow_messages(&self, messages: Vec<(String, P::Message)>) -> Result<()> {
        for (channel, msg) in messages {
            let parts = self.protocol.encode(&msg, &channel)?;
            self.transport.send(&channel, parts).await?;
        }
        Ok(())
    }

    /// Send idle status after `execute_reply`
    async fn send_execute_idle_status(&self, message: &P::Message) -> Result<()> {
        if let Ok(idle_msg) = self
            .protocol
            .create_status_message(crate::traits::KernelStatus::Idle)
        {
            let mut idle_with_parent = idle_msg;
            if let Some(header) = message.header_for_parent() {
                idle_with_parent.set_parent_from_json(header);
            }
            let parts = self.protocol.encode(&idle_with_parent, "iopub")?;
            self.transport.send("iopub", parts).await?;
        }
        Ok(())
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

        // Handle control messages
        if let Some(result) = self
            .try_handle_control_message(msg_type, content.clone())
            .await?
        {
            return Ok(result);
        }

        // Handle comm messages
        if msg_type.starts_with("comm_") {
            return self.handle_comm_message(msg_type, content).await;
        }

        // Handle extension messages (state, session, rag)
        match msg_type {
            "debug_request" => self.handle_debug_request_message(content).await,
            "state_request" => self.handle_state_request(content).await,
            "session_request" => self.handle_session_request(content).await,
            "rag_request" => self.handle_rag_request(content).await,
            _ => {
                tracing::warn!("Unknown message type: {}", msg_type);
                Ok(serde_json::json!({
                    "status": "error",
                    "message": format!("Unknown message type: {}", msg_type)
                }))
            }
        }
    }

    /// Try to handle control messages
    async fn try_handle_control_message(
        &self,
        msg_type: &str,
        content: serde_json::Value,
    ) -> Result<Option<serde_json::Value>> {
        match msg_type {
            "kernel_info_request" => Ok(Some(self.handle_kernel_info())),
            "execute_request" => Ok(Some(self.handle_execute(content).await?)),
            "shutdown_request" => Ok(Some(self.handle_shutdown(content).await?)),
            "interrupt_request" => Ok(Some(self.handle_interrupt())),
            _ => Ok(None),
        }
    }

    /// Handle comm-related messages
    async fn handle_comm_message(
        &self,
        msg_type: &str,
        content: serde_json::Value,
    ) -> Result<serde_json::Value> {
        match msg_type {
            "comm_open" => self.handle_comm_open(content).await,
            "comm_msg" => self.handle_comm_msg(content).await,
            "comm_close" => self.handle_comm_close(content).await,
            "comm_info_request" => self.handle_comm_info_request(content).await,
            _ => Ok(serde_json::json!({
                "status": "error",
                "message": format!("Unknown comm message type: {}", msg_type)
            })),
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
        // Create callback-based IO context
        let output_buffer = Arc::new(StdMutex::new(String::new()));
        let output_buffer_clone = output_buffer.clone();

        let silent_flag = silent;
        let stdout_callback = move |text: &str| -> Result<(), llmspell_core::error::LLMSpellError> {
            // Collect output
            output_buffer_clone.lock().unwrap().push_str(text);

            // Also publish to IOPub if not silent
            if !silent_flag {
                // Note: We can't easily call async methods from here
                // For now, just collect the output
                // TODO: Implement proper async callback mechanism
            }
            Ok(())
        };

        let stderr_callback = |text: &str| -> Result<(), llmspell_core::error::LLMSpellError> {
            // For now, treat stderr same as stdout
            tracing::warn!("Script stderr: {}", text);
            Ok(())
        };

        let io_context = create_callback_io_context(
            stdout_callback,
            stderr_callback,
            self.signal_handler.clone(),
        );

        // Reset interrupt flag before execution
        self.signal_handler.reset();

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

            // Use execute_script_with_io to route output through our callbacks
            runtime.execute_script_with_io(code, io_context).await
        };

        match result {
            Ok(script_output) => {
                // Get the collected output
                let collected = output_buffer.lock().unwrap().clone();

                // Publish all output to IOPub if not silent
                if !silent && !collected.is_empty() {
                    let _ = self.publish_stream("stdout", &collected).await;
                }

                // Return the result value as string
                Ok(script_output.output.to_string())
            }
            Err(e) => {
                self.handle_execution_error(e.into(), silent, execution_count)
                    .await
            }
        }
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

        // Clear the current request message now that execution is complete
        *self.current_request_message.write().await = None;

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

    fn handle_interrupt(&self) -> serde_json::Value {
        // Set the interrupt flag in the signal handler
        tracing::info!("Interrupt requested, signaling execution to stop");
        self.signal_handler.interrupt();

        // Also set kernel state to indicate interruption
        // Note: We can't make this async, so we just set the flag
        // The running execution will check this flag and stop

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
    /// Handle `debug_request` message from Jupyter protocol
    ///
    /// This method is called when a `debug_request` message is received via `ZeroMQ`.
    /// It extracts the DAP command and routes it through the DAP bridge.
    ///
    /// # Errors
    ///
    /// Returns an error if debug is not enabled or if the debug command fails.
    async fn handle_debug_request_message(
        &self,
        content: serde_json::Value,
    ) -> Result<serde_json::Value> {
        // Extract the DAP command from the debug_request content
        let dap_command = serde_json::json!({
            "seq": content.get("seq").cloned().unwrap_or_else(|| serde_json::json!(1)),
            "type": "request",
            "command": content.get("command").cloned().unwrap_or_else(|| serde_json::json!("unknown")),
            "arguments": content.get("arguments").cloned().unwrap_or_else(|| serde_json::json!({})),
        });

        // Route through the existing debug handler which uses DAP bridge
        let result = self.handle_debug_request(dap_command).await?;

        // Wrap result in debug_reply format
        Ok(serde_json::json!({
            "body": result
        }))
    }

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

    /// Trigger an interrupt signal to the kernel
    pub fn interrupt(&self) {
        self.signal_handler.interrupt();
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

    /// Handle state management requests using the kernel's `StateManager`
    async fn handle_state_request(&self, content: serde_json::Value) -> Result<serde_json::Value> {
        use crate::jupyter::protocol::StateOperation;

        // Try to get StateManager from runtime first, then fall back to kernel's
        let state_manager = {
            let runtime = self.runtime.lock().await;
            runtime
                .get_state_manager()
                .or_else(|| self.state_manager.clone())
        }
        .ok_or_else(|| {
            anyhow::anyhow!("State management not available - no StateManager configured")
        })?;

        let operation: StateOperation = serde_json::from_value(
            content
                .get("operation")
                .ok_or_else(|| anyhow::anyhow!("Missing operation field"))?
                .clone(),
        )?;

        let scope = Self::parse_state_scope(&content);

        match operation {
            StateOperation::Show { key } => {
                self.handle_state_show(&state_manager, scope, key).await
            }
            StateOperation::Clear { key } => {
                self.handle_state_clear(&state_manager, scope, key).await
            }
            StateOperation::Export { format } => {
                self.handle_state_export(&state_manager, scope, format)
                    .await
            }
            StateOperation::Import { data, merge } => {
                self.handle_state_import(&state_manager, scope, data, merge)
                    .await
            }
        }
    }

    /// Parse state scope from request content
    fn parse_state_scope(content: &serde_json::Value) -> llmspell_state_persistence::StateScope {
        use llmspell_state_persistence::StateScope;
        content
            .get("scope")
            .and_then(|s| s.as_str())
            .map_or(StateScope::Global, |s| match s {
                "session" => StateScope::Session(String::new()),
                "user" => StateScope::User(String::new()),
                _ => StateScope::Global,
            })
    }

    /// Handle state show operation
    async fn handle_state_show(
        &self,
        state_manager: &Arc<StateManager>,
        scope: llmspell_state_persistence::StateScope,
        key: Option<String>,
    ) -> Result<serde_json::Value> {
        if let Some(key) = key {
            let value = state_manager
                .get(scope, &key)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to get state: {}", e))?;
            Ok(serde_json::json!({
                "status": "ok",
                "data": value
            }))
        } else {
            let keys = state_manager
                .list_keys(scope.clone())
                .await
                .map_err(|e| anyhow::anyhow!("Failed to list keys: {}", e))?;
            let mut data = serde_json::Map::new();
            for key in keys {
                if let Ok(Some(value)) = state_manager.get(scope.clone(), &key).await {
                    data.insert(key, value);
                }
            }
            Ok(serde_json::json!({
                "status": "ok",
                "data": serde_json::Value::Object(data)
            }))
        }
    }

    /// Handle state clear operation
    async fn handle_state_clear(
        &self,
        state_manager: &Arc<StateManager>,
        scope: llmspell_state_persistence::StateScope,
        key: Option<String>,
    ) -> Result<serde_json::Value> {
        if let Some(key) = key {
            state_manager
                .delete(scope, &key)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to clear state: {}", e))?;
        } else {
            state_manager
                .clear_scope(scope)
                .await
                .map_err(|e| anyhow::anyhow!("Failed to clear state: {}", e))?;
        }
        Ok(serde_json::json!({
            "status": "ok"
        }))
    }

    /// Handle state export operation
    async fn handle_state_export(
        &self,
        state_manager: &Arc<StateManager>,
        scope: llmspell_state_persistence::StateScope,
        format: Option<String>,
    ) -> Result<serde_json::Value> {
        let _format = format.as_deref().unwrap_or("json");
        let export_data = state_manager
            .get_all_in_scope(scope)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to export state: {}", e))?;

        let data_value = serde_json::to_value(export_data)
            .map_err(|e| anyhow::anyhow!("Failed to serialize export data: {}", e))?;

        // Format the export based on requested format
        // For now, just return JSON (YAML support can be added later)
        let formatted = data_value;

        Ok(serde_json::json!({
            "status": "ok",
            "data": formatted
        }))
    }

    /// Handle state import operation
    async fn handle_state_import(
        &self,
        state_manager: &Arc<StateManager>,
        scope: llmspell_state_persistence::StateScope,
        data: serde_json::Value,
        merge: bool,
    ) -> Result<serde_json::Value> {
        if !merge {
            state_manager
                .clear_scope(scope.clone())
                .await
                .map_err(|e| anyhow::anyhow!("Failed to clear before import: {}", e))?;
        }

        if let Some(obj) = data.as_object() {
            for (key, value) in obj {
                state_manager
                    .set(scope.clone(), key, value.clone())
                    .await
                    .map_err(|e| anyhow::anyhow!("Failed to import key {}: {}", key, e))?;
            }
        } else {
            return Err(anyhow::anyhow!("Import data must be an object"));
        }

        Ok(serde_json::json!({
            "status": "ok"
        }))
    }

    /// Handle session management requests using the kernel's `SessionManager`
    async fn handle_session_request(
        &self,
        content: serde_json::Value,
    ) -> Result<serde_json::Value> {
        use crate::jupyter::protocol::SessionOperation;

        // Try to get SessionManager from runtime first, then fall back to kernel's
        let session_manager = {
            let runtime = self.runtime.lock().await;
            runtime
                .get_session_manager()
                .or_else(|| self.session_manager.clone())
        }
        .ok_or_else(|| {
            anyhow::anyhow!("Session management not available - no SessionManager configured")
        })?;

        let operation: SessionOperation = serde_json::from_value(
            content
                .get("operation")
                .ok_or_else(|| anyhow::anyhow!("Missing operation field"))?
                .clone(),
        )?;

        match operation {
            SessionOperation::Create { name, description } => {
                self.handle_session_create(&session_manager, name, description)
                    .await
            }
            SessionOperation::List { query } => {
                self.handle_session_list(&session_manager, query).await
            }
            SessionOperation::Show { id } => self.handle_session_show(&session_manager, id).await,
            SessionOperation::Delete { id } => {
                self.handle_session_delete(&session_manager, id).await
            }
            SessionOperation::Export { id, format: _ } => {
                self.handle_session_export(&session_manager, id).await
            }
            SessionOperation::Replay { .. } => Ok(serde_json::json!({
                "status": "error",
                "error": "Session replay not yet implemented via kernel"
            })),
        }
    }

    /// Handle session create operation
    async fn handle_session_create(
        &self,
        session_manager: &Arc<SessionManager>,
        name: Option<String>,
        description: Option<String>,
    ) -> Result<serde_json::Value> {
        use llmspell_sessions::CreateSessionOptions;

        let options = CreateSessionOptions::builder()
            .name(name.unwrap_or_else(|| "unnamed".to_string()))
            .description(description.unwrap_or_else(|| "Created via kernel".to_string()))
            .build();

        match session_manager.create_session(options).await {
            Ok(session_id) => Ok(serde_json::json!({
                "status": "ok",
                "session_id": session_id.to_string()
            })),
            Err(e) => Ok(serde_json::json!({
                "status": "error",
                "error": format!("Failed to create session: {}", e)
            })),
        }
    }

    /// Handle session list operation
    async fn handle_session_list(
        &self,
        session_manager: &Arc<SessionManager>,
        query: Option<serde_json::Value>,
    ) -> Result<serde_json::Value> {
        use llmspell_sessions::SessionQuery;

        let query = query.map_or_else(SessionQuery::default, |q| {
            serde_json::from_value(q).unwrap_or_default()
        });

        match session_manager.list_sessions(query).await {
            Ok(sessions) => Ok(serde_json::json!({
                "status": "ok",
                "sessions": sessions
            })),
            Err(e) => Ok(serde_json::json!({
                "status": "error",
                "error": format!("Failed to list sessions: {}", e)
            })),
        }
    }

    /// Handle session show operation
    async fn handle_session_show(
        &self,
        session_manager: &Arc<SessionManager>,
        id: String,
    ) -> Result<serde_json::Value> {
        use llmspell_sessions::SessionId;
        use std::str::FromStr;

        let session_id =
            SessionId::from_str(&id).map_err(|e| anyhow::anyhow!("Invalid session ID: {}", e))?;

        match session_manager.get_session(&session_id).await {
            Ok(session) => {
                let metadata = session.metadata.read().await;
                Ok(serde_json::json!({
                    "status": "ok",
                    "session": *metadata
                }))
            }
            Err(e) => Ok(serde_json::json!({
                "status": "error",
                "error": format!("Failed to get session: {}", e)
            })),
        }
    }

    /// Handle session delete operation
    async fn handle_session_delete(
        &self,
        session_manager: &Arc<SessionManager>,
        id: String,
    ) -> Result<serde_json::Value> {
        use llmspell_sessions::SessionId;
        use std::str::FromStr;

        if id.is_empty() {
            self.delete_all_sessions(session_manager).await
        } else {
            let session_id = SessionId::from_str(&id)
                .map_err(|e| anyhow::anyhow!("Invalid session ID: {}", e))?;

            match session_manager.delete_session(&session_id).await {
                Ok(()) => Ok(serde_json::json!({
                    "status": "ok"
                })),
                Err(e) => Ok(serde_json::json!({
                    "status": "error",
                    "error": format!("Failed to delete session: {}", e)
                })),
            }
        }
    }

    /// Delete all sessions
    async fn delete_all_sessions(
        &self,
        session_manager: &Arc<SessionManager>,
    ) -> Result<serde_json::Value> {
        use llmspell_sessions::SessionQuery;

        let sessions = session_manager
            .list_sessions(SessionQuery::default())
            .await
            .map_err(|e| anyhow::anyhow!("Failed to list sessions: {}", e))?;

        let mut deleted = 0;
        for session in sessions {
            if session_manager.delete_session(&session.id).await.is_ok() {
                deleted += 1;
            }
        }

        Ok(serde_json::json!({
            "status": "ok",
            "deleted": deleted
        }))
    }

    /// Handle session export operation
    async fn handle_session_export(
        &self,
        session_manager: &Arc<SessionManager>,
        id: String,
    ) -> Result<serde_json::Value> {
        use llmspell_sessions::SessionId;
        use std::str::FromStr;

        let session_id =
            SessionId::from_str(&id).map_err(|e| anyhow::anyhow!("Invalid session ID: {}", e))?;

        match session_manager.get_session(&session_id).await {
            Ok(session) => {
                let metadata = session.metadata.read().await.clone();
                let timeline = session_manager
                    .get_session_timeline(&session_id)
                    .await
                    .unwrap_or_default();

                let export_data = serde_json::json!({
                    "metadata": metadata,
                    "timeline": timeline,
                    "config": session.config,
                });

                Ok(serde_json::json!({
                    "status": "ok",
                    "data": export_data
                }))
            }
            Err(e) => Ok(serde_json::json!({
                "status": "error",
                "error": format!("Failed to export session: {}", e)
            })),
        }
    }

    /// Handle RAG system requests using the kernel's `RAGPipeline`
    async fn handle_rag_request(&self, content: serde_json::Value) -> Result<serde_json::Value> {
        use crate::jupyter::protocol::RagOperation;

        tracing::debug!("handle_rag_request called with content: {:?}", content);

        // Check if RAG pipeline is available
        let Some(rag_pipeline) = self.check_rag_pipeline() else {
            return Ok(Self::create_rag_error(
                "RAG system not available - RAG is disabled in configuration",
            ));
        };

        // Parse the operation from the request
        let operation = Self::parse_rag_operation(&content)?;

        // Parse optional scope for multi-tenant isolation
        let scope = Self::parse_rag_scope(&content);

        // Handle the specific operation
        match operation {
            RagOperation::Ingest {
                path,
                content,
                metadata,
                chunk_size: _,
                recursive: _,
            } => {
                self.handle_rag_ingest(rag_pipeline, path, content, metadata, scope)
                    .await
            }
            RagOperation::Search {
                query,
                limit,
                threshold,
                metadata_filter: _,
            } => {
                self.handle_rag_search(rag_pipeline, query, limit, threshold, scope)
                    .await
            }
            RagOperation::Stats { detailed: _ } => self.handle_rag_stats(rag_pipeline, scope).await,
            RagOperation::Clear {
                scope: clear_scope,
                confirm: _,
            } => {
                self.handle_rag_clear(rag_pipeline, clear_scope, scope)
                    .await
            }
            RagOperation::Index { action: _ } => Ok(Self::create_rag_error(
                "Index operation not yet implemented via kernel",
            )),
        }
    }

    /// Check if RAG pipeline is available
    fn check_rag_pipeline(&self) -> Option<&llmspell_rag::pipeline::RAGPipeline> {
        self.rag_pipeline.as_ref().map(|pipeline| {
            tracing::debug!("RAG pipeline is available");
            pipeline.as_ref()
        })
    }

    /// Parse RAG operation from request content
    fn parse_rag_operation(
        content: &serde_json::Value,
    ) -> Result<crate::jupyter::protocol::RagOperation> {
        use crate::jupyter::protocol::RagOperation;

        serde_json::from_value::<RagOperation>(
            content
                .get("operation")
                .ok_or_else(|| anyhow::anyhow!("Missing operation field"))?
                .clone(),
        )
        .map_err(|e| anyhow::anyhow!("Failed to parse RAG operation: {}", e))
    }

    /// Parse optional scope from request
    fn parse_rag_scope(content: &serde_json::Value) -> Option<llmspell_state_traits::StateScope> {
        use llmspell_state_traits::StateScope;

        content
            .get("scope")
            .and_then(|s| s.as_str())
            .map(|s| StateScope::User(s.to_string()))
    }

    /// Create standard error response
    fn create_rag_error(error_msg: &str) -> serde_json::Value {
        serde_json::json!({
            "status": "error",
            "error": error_msg
        })
    }

    /// Handle document ingestion
    async fn handle_rag_ingest(
        &self,
        rag_pipeline: &llmspell_rag::pipeline::RAGPipeline,
        path: String,
        content: Option<String>,
        metadata: Option<serde_json::Value>,
        scope: Option<llmspell_state_traits::StateScope>,
    ) -> Result<serde_json::Value> {
        // Use path as document ID
        let document_id = path.clone();

        // Get document content
        let doc_content = content.unwrap_or_else(|| {
            // If no content provided, try to read from path
            std::fs::read_to_string(&path).unwrap_or_default()
        });

        // Ingest the document
        match rag_pipeline
            .ingest_document(document_id.clone(), doc_content, metadata, scope)
            .await
        {
            Ok(result) => {
                let reply = serde_json::json!({
                    "status": "ok",
                    "data": {
                        "document_id": document_id,
                        "chunks_stored": result.chunks_stored,
                        "storage_time_ms": 0 // Field doesn't exist, use placeholder
                    }
                });
                tracing::debug!("Ingest successful, returning: {:?}", reply);
                Ok(reply)
            }
            Err(e) => {
                tracing::error!("Ingest failed: {}", e);
                Ok(Self::create_rag_error(&format!(
                    "Failed to ingest document: {e}"
                )))
            }
        }
    }

    /// Handle document search
    async fn handle_rag_search(
        &self,
        rag_pipeline: &llmspell_rag::pipeline::RAGPipeline,
        query: String,
        limit: usize,
        threshold: Option<f32>,
        scope: Option<llmspell_state_traits::StateScope>,
    ) -> Result<serde_json::Value> {
        // Build query configuration
        let query_config = llmspell_rag::pipeline::config::QueryConfig {
            max_results: Some(limit),
            min_score: threshold,
            hybrid_weights: None,
            metadata_filters: std::collections::HashMap::new(),
            reranking: None,
        };

        // Execute search
        match rag_pipeline
            .search(query.clone(), scope, Some(query_config))
            .await
        {
            Ok(result) => {
                let results: Vec<serde_json::Value> = result
                    .results
                    .into_iter()
                    .map(|r| {
                        serde_json::json!({
                            "content": r.content,
                            "score": r.score,
                            "metadata": r.metadata
                        })
                    })
                    .collect();

                Ok(serde_json::json!({
                    "status": "ok",
                    "data": {
                        "query": query,
                        "results": results,
                        "search_time_ms": result.retrieval_time_ms
                    }
                }))
            }
            Err(e) => Ok(Self::create_rag_error(&format!("Search failed: {e}"))),
        }
    }

    /// Handle statistics request
    async fn handle_rag_stats(
        &self,
        rag_pipeline: &llmspell_rag::pipeline::RAGPipeline,
        scope: Option<llmspell_state_traits::StateScope>,
    ) -> Result<serde_json::Value> {
        tracing::debug!("Getting RAG stats for scope: {:?}", scope);

        match rag_pipeline.stats(scope).await {
            Ok(stats) => {
                let reply = serde_json::json!({
                    "status": "ok",
                    "data": {
                        "stats": {
                            "vectors_stored": stats.vectors_stored,
                            "memory_usage_bytes": stats.memory_usage_bytes,
                            "cache_hits": stats.cache_hits,
                            "cache_misses": stats.cache_misses,
                            "cache_hit_rate": stats.cache_hit_rate,
                            "estimated_cost_usd": stats.estimated_cost_usd
                        }
                    }
                });
                tracing::debug!("Stats retrieved successfully: {:?}", reply);
                Ok(reply)
            }
            Err(e) => {
                tracing::error!("Failed to get stats: {}", e);
                Ok(Self::create_rag_error(&format!("Failed to get stats: {e}")))
            }
        }
    }

    /// Handle clear operation
    async fn handle_rag_clear(
        &self,
        rag_pipeline: &llmspell_rag::pipeline::RAGPipeline,
        clear_scope: Option<String>,
        default_scope: Option<llmspell_state_traits::StateScope>,
    ) -> Result<serde_json::Value> {
        use llmspell_state_traits::StateScope;

        let scope_to_clear = clear_scope
            .map(StateScope::User)
            .or(default_scope)
            .unwrap_or(StateScope::Global);

        match rag_pipeline.clear_scope(&scope_to_clear).await {
            Ok(deleted_count) => Ok(serde_json::json!({
                "status": "ok",
                "data": {
                    "vectors_deleted": deleted_count
                }
            })),
            Err(e) => Ok(Self::create_rag_error(&format!(
                "Failed to clear scope: {e}"
            ))),
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

    /// Create kernel with provided `ProviderManager`
    ///
    /// This method allows injecting a `ProviderManager` that was created in the main thread,
    /// avoiding HTTP client context issues when the kernel runs in a `tokio::spawn` task.
    ///
    /// # Errors
    ///
    /// Returns an error if kernel creation fails.
    pub async fn new_with_provider_manager(
        kernel_id: String,
        config: Arc<LLMSpellConfig>,
        mut transport: ZmqTransport,
        protocol: JupyterProtocol,
        provider_manager: Option<Arc<llmspell_bridge::ProviderManager>>,
    ) -> Result<Self> {
        let kernel_id = if kernel_id.is_empty() {
            Uuid::new_v4().to_string()
        } else {
            kernel_id
        };

        tracing::info!(
            "Starting kernel {} with {} protocol and engine {} (with ProviderManager: {})",
            kernel_id,
            protocol.name(),
            config.default_engine,
            provider_manager.is_some()
        );

        // Initialize kernel components with ProviderManager
        let (state_manager, runtime, session_mapper, comm_manager) = Box::pin(
            Self::initialize_kernel_components_with_provider_manager(&config, provider_manager),
        )
        .await?;

        // Setup transport and security
        let security_manager =
            Self::setup_transport_and_security(&mut transport, &protocol, &config).await?;

        tracing::info!(
            "Kernel {} bound to {} channels",
            kernel_id,
            transport.channels().len()
        );

        // Create client manager
        let client_manager = Arc::new(ClientManager::new(config.runtime.kernel.max_clients));

        // Get managers from runtime
        let session_manager = runtime.get_session_manager();
        let runtime_state_manager = runtime.get_state_manager();

        // Use runtime's state manager if available, otherwise use kernel's
        let final_state_manager = runtime_state_manager.or(state_manager);

        // Create RAG pipeline
        let rag_pipeline = Self::create_rag_pipeline(&config, final_state_manager.as_ref())?;

        tracing::trace!("Kernel constructor complete, returning kernel instance");
        Ok(Self {
            kernel_id,
            transport,
            protocol,
            runtime: Arc::new(Mutex::new(runtime)),
            client_manager,
            execution_state: Arc::new(RwLock::new(KernelState::Starting)),
            config,
            state_manager: final_state_manager,
            session_manager,
            rag_pipeline,
            security_manager,
            resource_limits: ClientResourceLimits::default(),
            execution_count: Arc::new(Mutex::new(0)),
            session_mapper,
            comm_manager,
            shutdown_tx: None,
            signal_handler: Arc::new(KernelSignalHandler::new()),
            current_request_message: Arc::new(RwLock::new(None)),
        })
    }
}
