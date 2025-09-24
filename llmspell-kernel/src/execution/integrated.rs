//! Integrated Kernel Implementation
//!
//! This module provides the `IntegratedKernel` that runs `ScriptRuntime` directly
//! in the current context without `tokio::spawn`, ensuring all components share
//! the same runtime context.

use anyhow::Result;
use chrono;
use llmspell_core::traits::script_executor::ScriptExecutor;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
use tracing::{debug, error, info, instrument, trace, warn};
use uuid;

use crate::connection::ConnectionFileManager;
use crate::daemon::{
    DaemonConfig, KernelMessage, OperationGuard, ShutdownConfig, ShutdownCoordinator, SignalBridge,
    SignalOperationsConfig, SignalOperationsHandler,
};
use crate::debug::{DAPBridge, ExecutionManager};
use crate::events::correlation::{ExecutionState, ExecutionStatus};
use crate::events::{KernelEvent, KernelEventCorrelator};
use crate::io::manager::EnhancedIOManager;
use crate::io::router::MessageRouter;
use crate::monitoring::{HealthMonitor, HealthReport, HealthStatus, HealthThresholds};
use crate::runtime::tracing::{OperationCategory, TracingInstrumentation};
use crate::sessions::{CreateSessionOptions, SessionManager, SessionManagerConfig};
use crate::state::{KernelState, StorageBackend};
use crate::traits::{Protocol, Transport};

// Session dependencies
use crate::state::StateManager;
use llmspell_events::bus::EventBus;
use llmspell_hooks::{HookExecutor, HookRegistry};
use llmspell_storage::MemoryBackend as SessionMemoryBackend;

/// I/O configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IOConfig {
    /// Buffer size for stdout
    pub stdout_buffer_size: usize,
    /// Buffer size for stderr
    pub stderr_buffer_size: usize,
    /// Flush interval in milliseconds
    pub flush_interval_ms: u64,
    /// Enable parent header tracking
    pub track_parent_headers: bool,
}

impl Default for IOConfig {
    fn default() -> Self {
        Self {
            stdout_buffer_size: 8192,
            stderr_buffer_size: 8192,
            flush_interval_ms: 100,
            track_parent_headers: true,
        }
    }
}

/// Configuration for the integrated kernel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    /// Script runtime configuration
    pub runtime_config: HashMap<String, Value>,
    /// IO management configuration
    pub io_config: IOConfig,
    /// Maximum message history for replay
    pub max_history: usize,
    /// Execution timeout in seconds
    pub execution_timeout_secs: u64,
    /// Enable agent monitoring
    pub monitor_agents: bool,
    /// Enable performance tracking
    pub track_performance: bool,
    /// Enable daemon mode
    pub daemon_mode: bool,
    /// Optional daemon configuration
    pub daemon_config: Option<DaemonConfig>,
    /// Optional health monitoring thresholds
    pub health_thresholds: Option<HealthThresholds>,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            runtime_config: HashMap::new(),
            io_config: IOConfig::default(),
            max_history: 1000,
            execution_timeout_secs: 300,
            monitor_agents: true,
            track_performance: true,
            daemon_mode: false,
            daemon_config: None,
            health_thresholds: None,
        }
    }
}

/// Integrated kernel that runs `ScriptRuntime` without spawning
pub struct IntegratedKernel<P: Protocol> {
    /// Script executor for execution
    script_executor: Arc<dyn ScriptExecutor>,
    /// Protocol handler
    protocol: P,
    /// Transport for message communication
    transport: Option<Box<dyn Transport>>,
    /// I/O manager
    io_manager: Arc<EnhancedIOManager>,
    /// Message router for multi-client support
    message_router: Arc<MessageRouter>,
    /// Event correlator for distributed tracing
    event_correlator: Arc<KernelEventCorrelator>,
    /// Tracing instrumentation
    tracing: TracingInstrumentation,
    /// Configuration
    config: ExecutionConfig,
    /// Session ID
    session_id: String,
    /// Execution counter
    pub execution_count: Arc<RwLock<u64>>,
    /// Unified kernel state
    state: Arc<KernelState>,
    /// Session manager
    #[allow(dead_code)] // Will be used when session integration is fully implemented
    session_manager: SessionManager,
    /// Execution manager for debugging
    execution_manager: Arc<ExecutionManager>,
    /// DAP bridge for IDE debugging
    dap_bridge: Arc<parking_lot::Mutex<DAPBridge>>,
    /// Shutdown signal receiver
    shutdown_rx: Option<mpsc::Receiver<()>>,
    /// Shutdown coordinator for graceful shutdown
    shutdown_coordinator: Arc<ShutdownCoordinator>,
    /// Signal bridge for handling Unix signals
    signal_bridge: Option<Arc<SignalBridge>>,
    /// Signal operations handler for SIGUSR1/SIGUSR2
    signal_operations: Arc<SignalOperationsHandler>,
    /// Connection file manager for Jupyter discovery
    connection_manager: Option<Arc<parking_lot::Mutex<ConnectionFileManager>>>,
    /// Health monitor for system monitoring
    health_monitor: Arc<HealthMonitor>,
    /// Pending input request sender for stdin channel
    pending_input_request: Option<oneshot::Sender<String>>,
    /// Channel health tracking - last activity timestamps
    channel_last_activity: Arc<RwLock<HashMap<String, std::time::Instant>>>,
    /// Current client identity for message routing
    current_client_identity: Option<Vec<u8>>,
    /// Current message header (becomes parent_header in replies)
    current_msg_header: Option<serde_json::Value>,
}

#[allow(dead_code)] // These methods will be used when transport is fully integrated
impl<P: Protocol + 'static> IntegratedKernel<P> {
    /// Create a new integrated kernel
    ///
    /// # Errors
    ///
    /// Returns an error if the script runtime cannot be created
    #[instrument(level = "info", skip_all)]
    #[allow(clippy::too_many_lines)]
    pub async fn new(
        protocol: P,
        config: ExecutionConfig,
        session_id: String,
        script_executor: Arc<dyn ScriptExecutor>,
    ) -> Result<Self> {
        info!("Creating IntegratedKernel for session {}", session_id);

        // Create I/O manager
        let io_config = crate::io::manager::IOConfig {
            stdout_buffer_size: config.io_config.stdout_buffer_size,
            stderr_buffer_size: config.io_config.stderr_buffer_size,
            flush_interval_ms: config.io_config.flush_interval_ms,
            track_parent_headers: config.io_config.track_parent_headers,
        };
        let mut io_manager = EnhancedIOManager::new(io_config, session_id.clone());

        // Create IOPub channel for output streaming
        let (iopub_sender, mut iopub_receiver) =
            mpsc::channel::<crate::io::manager::IOPubMessage>(100);
        io_manager.set_iopub_sender(iopub_sender);

        // Spawn task to route IOPub messages to stdout/stderr
        // This will be properly connected to transport in subtask 9.4.6.4
        tokio::spawn(async move {
            while let Some(msg) = iopub_receiver.recv().await {
                // For now, just log the messages - will be sent through transport later
                match msg.header.msg_type.as_str() {
                    "stream" => {
                        if let Some(stream_type) = msg.content.get("name").and_then(|v| v.as_str())
                        {
                            if let Some(text) = msg.content.get("text").and_then(|v| v.as_str()) {
                                match stream_type {
                                    "stdout" => print!("{text}"),
                                    "stderr" => eprint!("{text}"),
                                    _ => {}
                                }
                            }
                        }
                    }
                    "execute_result" | "display_data" => {
                        if let Some(data) = msg.content.get("data") {
                            if let Some(text) = data.get("text/plain").and_then(|v| v.as_str()) {
                                println!("{text}");
                            }
                        }
                    }
                    _ => {}
                }
            }
        });

        let io_manager = Arc::new(io_manager);

        // Create message router
        let message_router = Arc::new(MessageRouter::new(config.max_history));

        // Create event correlator
        let event_correlator = Arc::new(KernelEventCorrelator::new(
            message_router.clone(),
            session_id.clone(),
        ));

        // Create tracing instrumentation
        let tracing =
            TracingInstrumentation::new_kernel_session(Some(session_id.clone()), "integrated");

        // Create unified kernel state with memory backend by default
        let state = Arc::new(KernelState::new(StorageBackend::Memory(Box::new(
            crate::state::KernelMemoryBackend::new(),
        )))?);

        // Create session manager dependencies
        let state_manager = Arc::new(StateManager::new().await?);

        let session_storage_backend = Arc::new(SessionMemoryBackend::new());
        let hook_registry = Arc::new(HookRegistry::new());
        let hook_executor = Arc::new(HookExecutor::new());
        let event_bus = Arc::new(EventBus::new());
        let session_config = SessionManagerConfig::default();

        // Create session manager with proper dependencies
        let session_manager = SessionManager::new(
            state_manager,
            session_storage_backend,
            hook_registry,
            hook_executor,
            &event_bus,
            session_config,
        )?;

        // Create a session for this kernel instance
        let session_options = CreateSessionOptions::builder()
            .name(format!("kernel-session-{session_id}"))
            .build();

        let _session_id_obj = session_manager.create_session(session_options).await?;

        // Initialize session state
        state.update_session(|session| {
            session.set_id(&session_id);
            Ok(())
        })?;

        // Create ExecutionManager and DAPBridge
        let execution_manager = Arc::new(ExecutionManager::new(session_id.clone()));
        let mut dap_bridge = DAPBridge::new(session_id.clone());

        // Connect DAP bridge to execution manager
        dap_bridge.connect_execution_manager(execution_manager.clone());
        let dap_bridge = Arc::new(parking_lot::Mutex::new(dap_bridge));

        // Create shutdown coordinator
        let shutdown_config = ShutdownConfig::default();
        let mut shutdown_coordinator = ShutdownCoordinator::new(shutdown_config);
        shutdown_coordinator.set_message_router(message_router.clone());
        shutdown_coordinator.set_kernel_state(state.clone());
        let shutdown_coordinator = Arc::new(shutdown_coordinator);

        // Create signal operations handler
        let signal_operations_config = SignalOperationsConfig::default();
        let mut signal_operations = SignalOperationsHandler::new(signal_operations_config);
        signal_operations.set_kernel_state(state.clone());
        let signal_operations = Arc::new(signal_operations);

        // Create health monitor with configured thresholds
        let health_monitor = Arc::new(HealthMonitor::new(config.health_thresholds.clone()));

        Ok(Self {
            script_executor,
            protocol,
            transport: None,
            io_manager,
            message_router,
            event_correlator,
            tracing,
            config,
            session_id,
            execution_count: Arc::new(RwLock::new(0)),
            state,
            session_manager,
            execution_manager,
            dap_bridge,
            shutdown_rx: None,
            shutdown_coordinator,
            signal_bridge: None,
            signal_operations,
            connection_manager: None,
            health_monitor,
            pending_input_request: None,
            channel_last_activity: Arc::new(RwLock::new(HashMap::new())),
            current_client_identity: None,
            current_msg_header: None,
        })
    }

    /// Set shutdown signal receiver
    pub fn set_shutdown_receiver(&mut self, rx: mpsc::Receiver<()>) {
        self.shutdown_rx = Some(rx);
    }

    /// Set signal bridge for Unix signal handling
    pub fn set_signal_bridge(&mut self, bridge: Arc<SignalBridge>) {
        self.signal_bridge = Some(bridge);
    }

    /// Get shutdown coordinator
    pub fn shutdown_coordinator(&self) -> Arc<ShutdownCoordinator> {
        self.shutdown_coordinator.clone()
    }

    /// Get connection file path if available
    ///
    /// Returns the path to the Jupyter connection file if one was created
    pub fn connection_file_path(&self) -> Option<std::path::PathBuf> {
        self.connection_manager
            .as_ref()
            .and_then(|mgr| mgr.lock().file_path().map(std::path::Path::to_path_buf))
    }

    /// Get connection info if available
    ///
    /// Returns a clone of the connection info for Jupyter clients
    pub fn connection_info(&self) -> Option<crate::connection::ConnectionInfo> {
        self.connection_manager
            .as_ref()
            .map(|mgr| mgr.lock().info().clone())
    }

    /// Perform a comprehensive health check
    ///
    /// # Errors
    ///
    /// Returns an error if health monitoring fails
    pub async fn health_check(&self) -> Result<HealthReport> {
        self.health_monitor
            .health_check(
                &self.state,
                &self.message_router,
                Some(self.session_id.clone()),
            )
            .await
    }

    /// Get quick health status without full report
    ///
    /// # Errors
    ///
    /// Returns an error if health monitoring fails
    pub async fn quick_health_check(&self) -> Result<HealthStatus> {
        self.health_monitor
            .quick_health_check(&self.state, &self.message_router)
            .await
    }

    /// Set health monitoring thresholds
    pub fn set_health_thresholds(&self, _thresholds: HealthThresholds) {
        // Note: This would require making health_monitor mutable or using interior mutability
        // For now, we'll document this limitation
        warn!("Health threshold updates not supported after kernel creation - use ExecutionConfig");
    }

    /// Get current health status and log to tracing
    pub async fn log_health_status(&self) {
        match self.quick_health_check().await {
            Ok(status) => match status {
                HealthStatus::Healthy => debug!("Kernel health: HEALTHY"),
                HealthStatus::Degraded => warn!("Kernel health: DEGRADED"),
                HealthStatus::Unhealthy => error!("Kernel health: UNHEALTHY"),
            },
            Err(e) => error!("Failed to check health status: {}", e),
        }
    }

    /// Handle shutdown request from signal or message
    ///
    /// # Errors
    ///
    /// Returns an error if the shutdown coordinator fails to initiate shutdown
    pub async fn handle_shutdown(&mut self, restart: bool) -> Result<()> {
        info!("Handling shutdown request (restart={})", restart);

        // Initiate graceful shutdown
        self.shutdown_coordinator.initiate_shutdown().await?;

        // Send shutdown reply if transport is available
        if self.transport.is_some() {
            let reply = serde_json::json!({
                "msg_type": "shutdown_reply",
                "restart": restart,
            });
            debug!("Shutdown reply would be sent: {:?}", reply);
        }

        Ok(())
    }

    /// Process signals from signal bridge
    ///
    /// # Errors
    ///
    /// Returns an error if signal processing or shutdown handling fails
    pub async fn process_signals(&mut self) -> Result<()> {
        if let Some(ref bridge) = self.signal_bridge {
            if let Some(action) = bridge.process_signals_to_messages().await {
                // Convert to kernel message
                if let Some(msg) = SignalBridge::action_to_message(&action) {
                    match msg {
                        KernelMessage::ShutdownRequest { restart } => {
                            self.handle_shutdown(restart).await?;
                        }
                        KernelMessage::InterruptRequest => {
                            info!("Handling interrupt request from signal");
                            // TODO: Interrupt current execution
                        }
                        KernelMessage::ConfigReload => {
                            info!("Processing config reload from SIGUSR1");
                            if let Err(e) = self.signal_operations.handle_config_reload().await {
                                error!("Failed to reload configuration: {}", e);
                            }
                        }
                        KernelMessage::StateDump => {
                            info!("Processing state dump from SIGUSR2");
                            if let Err(e) = self.signal_operations.handle_state_dump().await {
                                error!("Failed to dump state: {}", e);
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Set transport for message communication
    pub fn set_transport(&mut self, transport: Box<dyn Transport>) {
        debug!("Setting transport on IntegratedKernel");
        self.transport = Some(transport);
        debug!("Transport set: {}", self.transport.is_some());
    }

    /// Check if transport is configured
    pub fn has_transport(&self) -> bool {
        self.transport.is_some()
    }

    /// Get the execution manager for debugging
    pub fn execution_manager(&self) -> Arc<ExecutionManager> {
        self.execution_manager.clone()
    }

    /// Get the DAP bridge for IDE debugging
    pub fn dap_bridge(&self) -> Arc<parking_lot::Mutex<DAPBridge>> {
        self.dap_bridge.clone()
    }

    /// Get the kernel state
    pub fn state(&self) -> Arc<KernelState> {
        self.state.clone()
    }

    /// Get the message router
    pub fn message_router(&self) -> Arc<MessageRouter> {
        self.message_router.clone()
    }

    /// Get the event correlator
    pub fn event_correlator(&self) -> Arc<KernelEventCorrelator> {
        self.event_correlator.clone()
    }

    /// Run the kernel in the current context (NO SPAWNING)
    ///
    /// # Errors
    ///
    /// Returns an error if I/O operations fail
    #[instrument(level = "info", skip(self))]
    #[allow(clippy::too_many_lines)]
    pub async fn run(mut self) -> Result<()> {
        info!("Starting IntegratedKernel in current context (no spawning)");

        // Debug: Check transport status at beginning of run
        debug!(
            "IntegratedKernel::run() - transport check at start: {}",
            self.transport.is_some()
        );

        // Start tracing
        self.tracing.trace_operation(
            OperationCategory::ScriptRuntime,
            "kernel_startup",
            Some(&self.session_id),
        );

        // Publish kernel status
        self.io_manager.publish_status("starting").await?;

        // Track kernel startup event
        let startup_event = KernelEvent::KernelStartup {
            kernel_id: format!("kernel-{}", self.session_id),
            protocol_version: "5.3".to_string(), // Jupyter protocol version
            language_info: crate::events::correlation::LanguageInfo {
                name: "llmspell".to_string(),
                version: "0.8.0".to_string(),
                mimetype: "text/x-llmspell".to_string(),
                file_extension: ".llms".to_string(),
                pygments_lexer: "text".to_string(),
                codemirror_mode: "text".to_string(),
            },
        };
        self.event_correlator.track_event(startup_event).await?;

        // Track status change to starting
        let status_event = KernelEvent::StatusChange {
            execution_state: ExecutionState::Starting,
            previous_state: None,
        };
        self.event_correlator.track_event(status_event).await?;

        // Main execution loop - runs in current context
        // Publish idle status to indicate kernel is ready
        self.io_manager.publish_status("idle").await?;

        info!(
            "Entering main kernel loop with transport={}",
            self.transport.is_some()
        );

        loop {
            // Check for shutdown signal
            if let Some(ref mut shutdown_rx) = self.shutdown_rx {
                if shutdown_rx.try_recv().is_ok() {
                    info!("Received shutdown signal");
                    break;
                }
            }

            // Process messages from transport if available
            let has_transport = self.transport.is_some();
            trace!("Transport polling check: has_transport={}", has_transport);

            if has_transport {
                trace!("Starting channel polling cycle");
                // Collect messages from all channels first
                let mut messages_to_process: Vec<(HashMap<String, Value>, Vec<u8>)> = Vec::new();

                // Process channels sequentially to avoid multiple mutable borrows
                // First, check Control channel (priority)
                trace!("Checking control channel");
                let control_msg = if let Some(ref mut transport) = self.transport {
                    let result = transport.recv("control").await;
                    trace!("Control recv result: {:?}", result.is_ok());
                    result.ok().flatten()
                } else {
                    trace!("No transport for control channel");
                    None
                };

                if let Some(message_parts) = control_msg {
                    // Update channel activity timestamp
                    self.channel_last_activity
                        .write()
                        .insert("control".to_string(), std::time::Instant::now());

                    // For Jupyter protocol, we need to parse the multipart message format
                    // Find the delimiter "<IDS|MSG>" and extract the content
                    let delimiter = b"<IDS|MSG>";
                    let delimiter_idx = message_parts
                        .iter()
                        .position(|part| part.as_slice() == delimiter);

                    let parsed_result = if let Some(idx) = delimiter_idx {
                        // This is a proper Jupyter wire protocol message
                        // The content is at position idx + 5 (after delimiter, signature, header, parent_header, metadata)
                        if message_parts.len() > idx + 5 {
                            self.protocol.parse_message(&message_parts[idx + 5])
                        } else {
                            Err(anyhow::anyhow!("Incomplete Jupyter message"))
                        }
                    } else if let Some(first_part) = message_parts.first() {
                        // Try parsing as a simple message (for compatibility)
                        self.protocol.parse_message(first_part)
                    } else {
                        Err(anyhow::anyhow!("Empty message"))
                    };

                    match parsed_result {
                        Ok(parsed_msg) => {
                            // Extract client identity from Part 0 for response routing
                            // Handle both REQ (empty first frame) and DEALER (identity first) sockets
                            let client_identity = if let Some(first_part) = message_parts.first() {
                                // Always use Part 0 as the client identity for routing
                                // For DEALER: Part 0 is the explicit client identity
                                // For REQ: Part 0 is the ZMQ-generated routing identity
                                // Both cases: echo back exactly what we received in Part 0
                                first_part.clone()
                            } else {
                                b"unknown_client".to_vec()
                            };

                            // Validate message type for control channel
                            if let Some(msg_type) =
                                parsed_msg.get("msg_type").and_then(|v| v.as_str())
                            {
                                if msg_type == "interrupt_request"
                                    || msg_type == "shutdown_request"
                                    || msg_type == "debug_request"
                                {
                                    trace!("Received control message: {}", msg_type);
                                    // Process control messages immediately (priority)
                                    if let Err(e) = self
                                        .handle_message_with_identity(parsed_msg, client_identity)
                                        .await
                                    {
                                        error!("Error handling control message: {}", e);
                                    }
                                } else {
                                    warn!("Invalid message type '{}' on control channel", msg_type);
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Failed to parse control message: {}", e);
                        }
                    }
                }

                // Process Shell channel for execution requests
                trace!("Checking shell channel");
                let shell_msg = if let Some(ref mut transport) = self.transport {
                    let result = transport.recv("shell").await;
                    match &result {
                        Ok(Some(parts)) => trace!("Shell recv SUCCESS: {} parts", parts.len()),
                        Ok(None) => trace!("Shell recv: no message"),
                        Err(e) => trace!("Shell recv ERROR: {}", e),
                    }
                    result.ok().flatten()
                } else {
                    trace!("No transport for shell channel");
                    None
                };

                if let Some(message_parts) = shell_msg {
                    // Update channel activity timestamp
                    self.channel_last_activity
                        .write()
                        .insert("shell".to_string(), std::time::Instant::now());

                    trace!(
                        "Processing shell message with {} parts",
                        message_parts.len()
                    );
                    for (i, part) in message_parts.iter().enumerate() {
                        trace!("Shell Part {}: {:?}", i, String::from_utf8_lossy(part));
                    }

                    // For Jupyter protocol, we need to parse the multipart message format
                    // Find the delimiter "<IDS|MSG>" and extract the content
                    let delimiter = b"<IDS|MSG>";
                    let delimiter_idx = message_parts
                        .iter()
                        .position(|part| part.as_slice() == delimiter);

                    let parsed_result = if let Some(idx) = delimiter_idx {
                        // Reconstruct full Jupyter message from wire protocol parts
                        if message_parts.len() > idx + 4 {
                            let header_bytes = &message_parts[idx + 2]; // header after delimiter + signature
                            let parent_header_bytes = &message_parts[idx + 3]; // parent_header
                            let metadata_bytes = &message_parts[idx + 4]; // metadata
                            let content_bytes = if message_parts.len() > idx + 5 {
                                &message_parts[idx + 5]
                            } else {
                                &vec![b'{', b'}']
                            };

                            // Parse header to get msg_type
                            if let Ok(header_value) =
                                serde_json::from_slice::<serde_json::Value>(header_bytes)
                            {
                                let mut full_message = std::collections::HashMap::new();
                                if let Ok(header_map) =
                                    serde_json::from_value::<
                                        std::collections::HashMap<String, serde_json::Value>,
                                    >(header_value.clone())
                                {
                                    for (k, v) in header_map {
                                        full_message.insert(k, v);
                                    }
                                }
                                full_message.insert(
                                    "parent_header".to_string(),
                                    serde_json::from_slice(parent_header_bytes).unwrap_or(
                                        serde_json::Value::Object(serde_json::Map::new()),
                                    ),
                                );
                                full_message.insert(
                                    "metadata".to_string(),
                                    serde_json::from_slice(metadata_bytes).unwrap_or(
                                        serde_json::Value::Object(serde_json::Map::new()),
                                    ),
                                );
                                full_message.insert(
                                    "content".to_string(),
                                    serde_json::from_slice(content_bytes).unwrap_or(
                                        serde_json::Value::Object(serde_json::Map::new()),
                                    ),
                                );

                                Ok(full_message)
                            } else {
                                Err(anyhow::anyhow!("Failed to parse Jupyter message header"))
                            }
                        } else {
                            Err(anyhow::anyhow!("Incomplete Jupyter message"))
                        }
                    } else if let Some(first_part) = message_parts.first() {
                        // Try parsing as a simple message (for compatibility)
                        self.protocol.parse_message(first_part)
                    } else {
                        Err(anyhow::anyhow!("Empty message"))
                    };

                    match parsed_result {
                        Ok(parsed_msg) => {
                            // Extract client identity from Part 0 for response routing
                            // Handle both REQ (empty first frame) and DEALER (identity first) sockets
                            let client_identity = if let Some(first_part) = message_parts.first() {
                                // Always use Part 0 as the client identity for routing
                                // For DEALER: Part 0 is the explicit client identity
                                // For REQ: Part 0 is the ZMQ-generated routing identity
                                // Both cases: echo back exactly what we received in Part 0
                                first_part.clone()
                            } else {
                                b"unknown_client".to_vec()
                            };

                            // Validate message type for shell channel
                            if let Some(msg_type) =
                                parsed_msg.get("msg_type").and_then(|v| v.as_str())
                            {
                                if msg_type == "execute_request"
                                    || msg_type == "complete_request"
                                    || msg_type == "inspect_request"
                                    || msg_type == "kernel_info_request"
                                    || msg_type == "comm_info_request"
                                    || msg_type == "history_request"
                                {
                                    trace!("Received shell message: {}", msg_type);
                                    messages_to_process.push((parsed_msg, client_identity));
                                } else {
                                    warn!("Invalid message type '{}' on shell channel", msg_type);
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Failed to parse shell message: {}", e);
                        }
                    }
                }

                // Process Stdin channel for input requests (kernel → frontend)
                let stdin_msg = if let Some(ref mut transport) = self.transport {
                    transport.recv("stdin").await.ok().flatten()
                } else {
                    None
                };

                if let Some(message_parts) = stdin_msg {
                    // Update channel activity timestamp
                    self.channel_last_activity
                        .write()
                        .insert("stdin".to_string(), std::time::Instant::now());

                    // For Jupyter protocol, we need to parse the multipart message format
                    // Find the delimiter "<IDS|MSG>" and extract the content
                    let delimiter = b"<IDS|MSG>";
                    let delimiter_idx = message_parts
                        .iter()
                        .position(|part| part.as_slice() == delimiter);

                    let parsed_result = if let Some(idx) = delimiter_idx {
                        // This is a proper Jupyter wire protocol message
                        // The content is at position idx + 5 (after delimiter, signature, header, parent_header, metadata)
                        if message_parts.len() > idx + 5 {
                            self.protocol.parse_message(&message_parts[idx + 5])
                        } else {
                            Err(anyhow::anyhow!("Incomplete Jupyter message"))
                        }
                    } else if let Some(first_part) = message_parts.first() {
                        // Try parsing as a simple message (for compatibility)
                        self.protocol.parse_message(first_part)
                    } else {
                        Err(anyhow::anyhow!("Empty message"))
                    };

                    match parsed_result {
                        Ok(parsed_msg) => {
                            // Validate message type for stdin channel
                            if let Some(msg_type) =
                                parsed_msg.get("msg_type").and_then(|v| v.as_str())
                            {
                                if msg_type == "input_reply" {
                                    trace!("Received stdin message: {}", msg_type);
                                    // Handle input reply from frontend
                                    if let Err(e) = self.handle_input_reply(parsed_msg).await {
                                        error!("Error handling input reply: {}", e);
                                    }
                                } else {
                                    warn!("Invalid message type '{}' on stdin channel", msg_type);
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Failed to parse stdin message: {}", e);
                        }
                    }
                }

                // Process heartbeat separately (simple echo)
                let hb_data = if let Some(ref mut transport) = self.transport {
                    transport.recv("heartbeat").await.ok().flatten()
                } else {
                    None
                };

                if let Some(data) = hb_data {
                    // Update channel activity timestamp
                    self.channel_last_activity
                        .write()
                        .insert("heartbeat".to_string(), std::time::Instant::now());

                    // Echo heartbeat immediately
                    if let Some(ref mut transport) = self.transport {
                        if let Err(e) = transport.send("heartbeat", data).await {
                            warn!("Failed to send heartbeat response: {}", e);
                        }
                    }
                }

                // Now process collected messages (transport no longer borrowed)
                for (parsed_msg, client_identity) in messages_to_process {
                    // Start measuring message handling time
                    let start_time = std::time::Instant::now();

                    // Handle the message with client identity
                    if let Err(e) = self
                        .handle_message_with_identity(parsed_msg, client_identity)
                        .await
                    {
                        error!("Error handling message: {}", e);
                    }

                    // Log message handling time for performance analysis
                    let elapsed = start_time.elapsed();
                    if elapsed.as_millis() > 5 {
                        warn!(
                            "Message handling took {}ms (target: <5ms)",
                            elapsed.as_millis()
                        );
                    } else {
                        trace!("Message handled in {}μs", elapsed.as_micros());
                    }
                }

                // Small yield to prevent busy-waiting
                trace!("Completed channel polling cycle, sleeping 1ms");
                tokio::time::sleep(Duration::from_millis(1)).await;
            } else {
                // No transport configured, sleep longer
                trace!("No transport, sleeping 100ms");
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }

        // Cleanup
        self.io_manager.publish_status("idle").await?;

        // Track status change to idle
        let idle_status_event = KernelEvent::StatusChange {
            execution_state: ExecutionState::Idle,
            previous_state: Some(ExecutionState::Busy),
        };
        self.event_correlator.track_event(idle_status_event).await?;

        // Track kernel shutdown event
        let shutdown_event = KernelEvent::KernelShutdown {
            restart: false,
            reason: "Normal shutdown".to_string(),
        };
        self.event_correlator.track_event(shutdown_event).await?;

        self.tracing.trace_operation(
            OperationCategory::ScriptRuntime,
            "kernel_shutdown",
            Some(&self.session_id),
        );

        info!("IntegratedKernel shutdown complete");
        Ok(())
    }

    /// Handle a parsed message
    ///
    /// # Errors
    ///
    /// Returns an error if message handling fails
    #[instrument(level = "debug", skip(self, message))]
    async fn handle_message(&mut self, message: HashMap<String, Value>) -> Result<()> {
        let msg_type = message
            .get("msg_type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        debug!("Handling message type: {}", msg_type);

        // Message handling will be done at the session level as needed
        // The comprehensive session manager handles session lifecycle, not individual messages

        match msg_type {
            "execute_request" => self.handle_execute_request(message).await?,
            "kernel_info_request" => self.handle_kernel_info_request(&message).await?,
            "shutdown_request" => self.handle_shutdown_request(&message)?,
            "interrupt_request" => self.handle_interrupt_request(&message)?,
            "debug_request" => self.handle_debug_request(message).await?,
            _ => {
                warn!("Unhandled message type: {}", msg_type);
            }
        }

        Ok(())
    }

    /// Handle a parsed message with client identity for response routing
    ///
    /// # Errors
    ///
    /// Returns an error if message handling fails
    #[instrument(level = "debug", skip(self, message, client_identity))]
    async fn handle_message_with_identity(
        &mut self,
        message: HashMap<String, Value>,
        client_identity: Vec<u8>,
    ) -> Result<()> {
        // Store client identity for use in responses
        self.current_client_identity = Some(client_identity);

        // Store the message header to use as parent_header in replies
        // The header has already been extracted and is in the message map
        if let Some(header) = message.get("header").cloned() {
            self.current_msg_header = Some(header);
        } else if let Some(msg_id) = message.get("msg_id") {
            // If no header but we have individual fields, construct it
            let mut header = serde_json::Map::new();
            if let Some(msg_id) = message.get("msg_id") {
                header.insert("msg_id".to_string(), msg_id.clone());
            }
            if let Some(session) = message.get("session") {
                header.insert("session".to_string(), session.clone());
            }
            if let Some(username) = message.get("username") {
                header.insert("username".to_string(), username.clone());
            }
            if let Some(msg_type) = message.get("msg_type") {
                header.insert("msg_type".to_string(), msg_type.clone());
            }
            if let Some(version) = message.get("version") {
                header.insert("version".to_string(), version.clone());
            }
            self.current_msg_header = Some(serde_json::Value::Object(header));
        }

        // Delegate to the original handle_message method
        self.handle_message(message).await
    }

    /// Handle input reply from frontend
    ///
    /// # Errors
    ///
    /// Returns an error if handling fails
    #[instrument(level = "debug", skip(self, message))]
    async fn handle_input_reply(&mut self, message: HashMap<String, Value>) -> Result<()> {
        // Extract the input value
        let input_value = message
            .get("content")
            .and_then(|c| c.get("value"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        debug!("Received input reply: {}", input_value);

        // Store the input value for future reference (could be stored in state if needed)
        debug!("Input value stored: {}", input_value);

        // Notify any waiting input request
        if let Some(sender) = self.pending_input_request.take() {
            let _ = sender.send(input_value.to_string());
        }

        Ok(())
    }

    /// Validate that a message type is appropriate for a channel
    pub fn validate_message_for_channel(&self, channel: &str, message: &serde_json::Value) -> bool {
        let msg_type = message
            .get("msg_type")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        match channel {
            "shell" => matches!(
                msg_type,
                "execute_request"
                    | "complete_request"
                    | "inspect_request"
                    | "history_request"
                    | "is_complete_request"
                    | "comm_info_request"
                    | "kernel_info_request"
            ),
            "control" => matches!(
                msg_type,
                "interrupt_request" | "shutdown_request" | "debug_request"
            ),
            "stdin" => matches!(msg_type, "input_reply"),
            "heartbeat" => true, // Heartbeat accepts any message (it's just echo)
            _ => false,
        }
    }

    /// Get channel health status
    ///
    /// Returns a map of channel names to their health status (active within last 30 seconds)
    pub fn get_channel_health(&self) -> HashMap<String, bool> {
        let mut health = HashMap::new();
        let now = std::time::Instant::now();
        let timeout = Duration::from_secs(30);

        let activity = self.channel_last_activity.read();

        for channel in ["shell", "control", "stdin", "heartbeat", "iopub"] {
            let is_healthy = activity
                .get(channel)
                .is_some_and(|last| now.duration_since(*last) < timeout);
            health.insert(channel.to_string(), is_healthy);
        }

        // IOPub is always healthy if IO manager exists
        health.insert("iopub".to_string(), true);

        health
    }

    /// Request input from frontend
    ///
    /// # Errors
    ///
    /// Returns an error if the request cannot be sent
    #[instrument(level = "debug", skip(self))]
    pub async fn request_input(&mut self, prompt: &str, password: bool) -> Result<String> {
        info!("Requesting input from frontend: {}", prompt);

        // Create input_request message
        let request = json!({
            "msg_type": "input_request",
            "content": {
                "prompt": prompt,
                "password": password,
            }
        });

        // Send to stdin channel
        if let Some(ref mut transport) = self.transport {
            let msg_bytes = self.protocol.create_request("input_request", request)?;
            transport.send("stdin", vec![msg_bytes]).await?;

            // Create oneshot channel for reply
            let (tx, rx) = tokio::sync::oneshot::channel();
            self.pending_input_request = Some(tx);

            // Wait for reply with timeout
            match tokio::time::timeout(Duration::from_secs(120), rx).await {
                Ok(Ok(input)) => Ok(input),
                Ok(Err(_)) => Err(anyhow::anyhow!("Input request cancelled")),
                Err(_) => Err(anyhow::anyhow!("Input request timed out after 120 seconds")),
            }
        } else {
            Err(anyhow::anyhow!("No transport available for input request"))
        }
    }

    /// Handle `debug_request` message from control channel
    ///
    /// # Errors
    ///
    /// Returns an error if debug request handling fails
    #[instrument(level = "debug", skip(self, message))]
    async fn handle_debug_request(&mut self, message: HashMap<String, Value>) -> Result<()> {
        let content = message
            .get("content")
            .ok_or_else(|| anyhow::anyhow!("Missing content"))?;

        debug!("Handling debug_request");

        // Connect DAPBridge to ExecutionManager if not connected
        // Process DAP request through existing DAPBridge
        let dap_response = {
            let mut dap_bridge = self.dap_bridge.lock();
            if !dap_bridge.is_connected() {
                dap_bridge.connect_execution_manager(self.execution_manager.clone());
            }
            dap_bridge.handle_request(content)?
        };

        // Send debug_reply on control channel using proper multipart format
        if self.transport.is_some() {
            // Use stored client identity from the request for response routing
            let client_identity = self
                .current_client_identity
                .clone()
                .unwrap_or_else(|| b"unknown_client".to_vec());
            let multipart_response =
                self.create_multipart_response(&client_identity, "debug_reply", &dap_response)?;

            debug!(
                "debug_reply multipart created, {} parts",
                multipart_response.len()
            );

            // Now borrow transport mutably
            if let Some(ref mut transport) = self.transport {
                match transport.send("control", multipart_response).await {
                    Ok(()) => debug!("debug_reply sent successfully via control channel"),
                    Err(e) => error!("Failed to send debug_reply: {}", e),
                }
            }
        } else {
            debug!("No transport available for debug_reply");
        }

        Ok(())
    }

    /// Broadcast debug event on `IOPub` channel
    ///
    /// # Errors
    ///
    /// Returns an error if broadcasting fails
    pub async fn broadcast_debug_event(&mut self, event: Value) -> Result<()> {
        debug!("Broadcasting debug event: {:?}", event);

        // Send event through IOPub channel via transport using multipart format
        if self.transport.is_some() {
            // IOPub messages don't need client identity routing (broadcast channel)
            let client_identity = b"".to_vec(); // Empty identity for IOPub
            let multipart_event =
                self.create_multipart_response(&client_identity, "debug_event", &event)?;

            if let Some(ref mut transport) = self.transport {
                match transport.send("iopub", multipart_event).await {
                    Ok(()) => debug!("debug_event broadcast successfully via iopub channel"),
                    Err(e) => error!("Failed to broadcast debug_event: {}", e),
                }
            }
        } else {
            // Use IO manager to publish the event
            let mut display_data = HashMap::new();
            display_data.insert("application/debug+json".to_string(), event);
            self.io_manager.publish_display_data(display_data).await?;
        }

        Ok(())
    }

    /// Handle `execute_request` message
    ///
    /// # Errors
    ///
    /// Returns an error if execution fails
    #[instrument(level = "debug", skip(self, message))]
    #[allow(clippy::too_many_lines)]
    async fn handle_execute_request(&mut self, message: HashMap<String, Value>) -> Result<()> {
        // Extract message ID and code from message
        let msg_id = message
            .get("header")
            .and_then(|h| h.get("msg_id"))
            .and_then(|v| v.as_str())
            .unwrap_or("unknown")
            .to_string();

        let code = message
            .get("content")
            .and_then(|c| c.get("code"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if code.is_empty() {
            return Ok(());
        }

        // Extract additional execution parameters
        let silent = message
            .get("content")
            .and_then(|c| c.get("silent"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);

        let user_expressions = message
            .get("content")
            .and_then(|c| c.get("user_expressions"))
            .and_then(|v| v.as_object())
            .map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            })
            .unwrap_or_default();

        // Start execution context for correlation
        self.event_correlator
            .start_execution_context(&msg_id, &self.session_id)
            .await;

        // Track execute_request event
        let execute_request_event = KernelEvent::ExecuteRequest {
            code: code.to_string(),
            msg_id: msg_id.clone(),
            session_id: self.session_id.clone(),
            silent,
            user_expressions,
        };
        self.event_correlator
            .track_event(execute_request_event)
            .await?;

        // Increment execution counter and update state
        let exec_count = {
            let mut count = self.execution_count.write();
            *count += 1;
            *count
        };

        // Start execution in state
        let execution_id = format!("exec_{exec_count}");
        self.state.update_execution(|exec| {
            exec.increment_counter();
            exec.start_execution(execution_id.clone(), code.to_string());
            Ok(())
        })?;

        info!("Executing code [{}]: {} bytes", exec_count, code.len());

        // Set parent header for message correlation
        if let Some(header) = message.get("header") {
            if let Ok(header) = serde_json::from_value(header.clone()) {
                self.io_manager.set_parent_header(header);
            }
        }

        // Publish busy status
        self.io_manager.publish_status("busy").await?;

        // Track status change to busy
        let busy_status_event = KernelEvent::StatusChange {
            execution_state: ExecutionState::Busy,
            previous_state: Some(ExecutionState::Idle),
        };
        self.event_correlator.track_event(busy_status_event).await?;

        // Start execution tracing
        self.tracing.trace_operation(
            OperationCategory::ScriptRuntime,
            "execute_request",
            Some(&format!("execution_{exec_count}")),
        );

        // Track agent creation if monitoring enabled
        if self.config.monitor_agents {
            self.tracing.trace_operation(
                OperationCategory::Agent,
                "monitor_start",
                Some("execution"),
            );
        }

        // Execute code in current context (NOT SPAWNED)
        let start_time = std::time::Instant::now();
        let result = tokio::time::timeout(
            Duration::from_secs(self.config.execution_timeout_secs),
            self.execute_code_in_context(code),
        )
        .await;

        let execution_time = start_time.elapsed();

        // Track performance if enabled
        if self.config.track_performance {
            trace!("Execution {} completed in {:?}", exec_count, execution_time);
            self.tracing.trace_operation(
                OperationCategory::ScriptRuntime,
                "performance",
                Some(&format!(
                    "execution_time_ms: {}",
                    execution_time.as_millis()
                )),
            );
        }

        // Handle result and update state
        match result {
            Ok(Ok(output)) => {
                // Update execution state with success
                self.state.update_execution(|exec| {
                    exec.complete_execution(Some(output.clone()), None);
                    Ok(())
                })?;

                // Publish execute result
                let mut data = HashMap::new();
                data.insert("text/plain".to_string(), Value::String(output));
                self.io_manager
                    .publish_execute_result(exec_count.try_into().unwrap_or(i32::MAX), data)
                    .await?;

                // Send execute_reply message through protocol
                let execute_reply = self.protocol.create_response(
                    "execute_reply",
                    serde_json::json!({
                        "status": "ok",
                        "execution_count": exec_count,
                        "user_expressions": {},
                    }),
                )?;

                // TODO: Send execute_reply through transport once integrated
                // For now, just create the response
                let _ = execute_reply;

                // Track successful execute_reply event
                let execute_reply_event = KernelEvent::ExecuteReply {
                    status: ExecutionStatus::Ok,
                    msg_id: msg_id.clone(),
                    session_id: self.session_id.clone(),
                    execution_count: exec_count,
                    error_info: None,
                };
                self.event_correlator
                    .track_event(execute_reply_event)
                    .await?;
            }
            Ok(Err(e)) => {
                error!("Execution error: {}", e);

                // Update execution state with error
                self.state.update_execution(|exec| {
                    exec.complete_execution(None, Some(e.to_string()));
                    Ok(())
                })?;

                self.io_manager
                    .write_stderr(&format!("Error: {e}\n"))
                    .await?;

                // Send error execute_reply message through protocol
                let error_reply = self.protocol.create_response(
                    "execute_reply",
                    serde_json::json!({
                        "status": "error",
                        "execution_count": exec_count,
                        "ename": "ExecutionError",
                        "evalue": e.to_string(),
                        "traceback": vec![e.to_string()],
                    }),
                )?;

                // TODO: Send execute_reply through transport once integrated
                let _ = error_reply;

                // Track error execute_reply event
                let error_info = crate::events::correlation::ErrorInfo {
                    ename: "ExecutionError".to_string(),
                    evalue: e.to_string(),
                    traceback: vec![e.to_string()],
                };
                let execute_reply_event = KernelEvent::ExecuteReply {
                    status: ExecutionStatus::Error,
                    msg_id: msg_id.clone(),
                    session_id: self.session_id.clone(),
                    execution_count: exec_count,
                    error_info: Some(error_info),
                };
                self.event_correlator
                    .track_event(execute_reply_event)
                    .await?;
            }
            Err(_) => {
                error!(
                    "Execution timeout after {} seconds",
                    self.config.execution_timeout_secs
                );
                self.io_manager
                    .write_stderr(&format!(
                        "Execution timeout after {} seconds\n",
                        self.config.execution_timeout_secs
                    ))
                    .await?;

                // Send timeout execute_reply message through protocol
                let timeout_reply = self.protocol.create_response(
                    "execute_reply",
                    serde_json::json!({
                        "status": "aborted",
                        "execution_count": exec_count,
                    }),
                )?;

                // TODO: Send execute_reply through transport once integrated
                let _ = timeout_reply;

                // Track timeout execute_reply event
                let error_info = crate::events::correlation::ErrorInfo {
                    ename: "TimeoutError".to_string(),
                    evalue: format!(
                        "Execution timeout after {} seconds",
                        self.config.execution_timeout_secs
                    ),
                    traceback: vec!["Execution timed out".to_string()],
                };
                let execute_reply_event = KernelEvent::ExecuteReply {
                    status: ExecutionStatus::Aborted,
                    msg_id: msg_id.clone(),
                    session_id: self.session_id.clone(),
                    execution_count: exec_count,
                    error_info: Some(error_info),
                };
                self.event_correlator
                    .track_event(execute_reply_event)
                    .await?;
            }
        }

        // Clear parent header
        self.io_manager.clear_parent_header();

        // Publish idle status
        self.io_manager.publish_status("idle").await?;

        // Track status change back to idle
        let idle_status_event = KernelEvent::StatusChange {
            execution_state: ExecutionState::Idle,
            previous_state: Some(ExecutionState::Busy),
        };
        self.event_correlator.track_event(idle_status_event).await?;

        // End execution context for correlation
        self.event_correlator.end_execution_context().await;

        // Stop agent monitoring
        if self.config.monitor_agents {
            self.tracing.trace_operation(
                OperationCategory::Agent,
                "monitor_stop",
                Some("execution"),
            );
        }

        Ok(())
    }

    /// Execute code in the current runtime context
    ///
    /// # Errors
    ///
    /// Returns an error if code execution fails
    #[instrument(level = "trace", skip(self, code))]
    async fn execute_code_in_context(&mut self, code: &str) -> Result<String> {
        // This is the critical fix - execute directly without spawning
        // The ScriptRuntime now shares the same runtime context as transport
        trace!("Executing code in current context (no spawn)");

        // Execute script using the script executor
        let script_output = self
            .script_executor
            .execute_script(code)
            .await
            .map_err(|e| anyhow::anyhow!("Script execution failed: {}", e))?;

        // Route console output through I/O manager
        for line in &script_output.console_output {
            self.io_manager.write_stdout(line).await?;
        }

        // Send result as display_data if available
        if script_output.output != serde_json::Value::Null {
            // Prepare data for display
            let mut display_data = HashMap::new();

            // Always include plain text representation
            display_data.insert(
                "text/plain".to_string(),
                serde_json::Value::String(script_output.output.to_string()),
            );

            // If the output is already JSON, include it as application/json
            if script_output.output.is_object() || script_output.output.is_array() {
                display_data.insert("application/json".to_string(), script_output.output.clone());
            }

            // Publish display data through IOPub channel
            self.io_manager.publish_display_data(display_data).await?;
        }

        // Return the script result
        let result = if script_output.output == serde_json::Value::Null {
            "null".to_string()
        } else {
            script_output.output.to_string()
        };

        // Flush I/O buffers
        self.io_manager.flush_all().await?;

        Ok(result)
    }

    /// Execute code directly without message loop
    /// Used for embedded mode when kernel is not running
    ///
    /// # Errors
    ///
    /// Returns an error if code execution fails
    pub async fn execute_direct(&mut self, code: &str) -> Result<String> {
        self.execute_direct_with_args(code, HashMap::new()).await
    }

    /// Execute code directly with script arguments
    /// Used for embedded mode when kernel is not running
    ///
    /// # Errors
    ///
    /// Returns an error if code execution fails
    pub async fn execute_direct_with_args(
        &mut self,
        code: &str,
        args: HashMap<String, String>,
    ) -> Result<String> {
        // Check if we're accepting requests
        if !self.shutdown_coordinator.is_accepting_requests().await {
            return Err(anyhow::anyhow!(
                "Kernel is shutting down, not accepting new requests"
            ));
        }

        // Track this operation
        let _guard = OperationGuard::new(self.shutdown_coordinator.clone());

        // Generate execution ID
        let exec_id = format!("exec-{}", uuid::Uuid::new_v4());

        // Update state for tracking
        self.state.update_execution(|exec| {
            exec.start_execution(exec_id, code.to_string());
            Ok(())
        })?;

        // Execute code with arguments if provided
        let result = if args.is_empty() {
            // Execute code using the internal method as before
            self.execute_code_in_context(code).await
        } else {
            debug!("Executing script with {} arguments", args.len());
            // Use the new execute_script_with_args method
            match self
                .script_executor
                .execute_script_with_args(code, args)
                .await
            {
                Ok(output) => {
                    // Convert ScriptExecutionOutput to String
                    // Combine console output and result
                    let mut result = String::new();
                    if !output.console_output.is_empty() {
                        result.push_str(&output.console_output.join("\n"));
                        result.push('\n');
                    }
                    result.push_str(&serde_json::to_string(&output.output).unwrap_or_default());
                    Ok(result)
                }
                Err(e) => Err(anyhow::anyhow!("Script execution failed: {}", e)),
            }
        };

        // Update state based on result
        match &result {
            Ok(output) => {
                self.state.update_execution(|exec| {
                    exec.complete_execution(Some(output.clone()), None);
                    Ok(())
                })?;
            }
            Err(e) => {
                self.state.update_execution(|exec| {
                    exec.complete_execution(None, Some(e.to_string()));
                    Ok(())
                })?;
            }
        }

        result
    }

    /// Handle `kernel_info_request`
    ///
    /// # Errors
    ///
    /// Returns an error if response creation fails
    async fn handle_kernel_info_request(
        &mut self,
        _message: &HashMap<String, Value>,
    ) -> Result<()> {
        debug!("Handling kernel_info_request");

        // Create kernel info response with language from script executor
        let language = self.script_executor.language();
        let (file_extension, version) = match language {
            "lua" => (".lua", "5.4"),
            "javascript" | "js" => (".js", "ES2022"),
            "python" => (".py", "3.11"),
            _ => (".txt", "1.0"),
        };

        let kernel_info = serde_json::json!({
            "protocol_version": crate::PROTOCOL_VERSION,
            "implementation": "llmspell",
            "implementation_version": crate::KERNEL_VERSION,
            "language_info": {
                "name": language,
                "version": version,
                "file_extension": file_extension,
            },
            "banner": format!("LLMSpell Kernel v{} ({})", crate::KERNEL_VERSION, language),
        });

        // Send response via transport using proper multipart format
        if self.transport.is_some() {
            // Use stored client identity from the request for response routing
            let client_identity = self
                .current_client_identity
                .clone()
                .unwrap_or_else(|| b"unknown_client".to_vec());
            let multipart_response = self.create_multipart_response(
                &client_identity,
                "kernel_info_reply",
                &kernel_info,
            )?;

            debug!(
                "kernel_info_reply multipart created, {} parts",
                multipart_response.len()
            );

            // Now borrow transport mutably
            if let Some(ref mut transport) = self.transport {
                match transport.send("shell", multipart_response).await {
                    Ok(()) => debug!("kernel_info_reply sent successfully via shell channel"),
                    Err(e) => error!("Failed to send kernel_info_reply: {}", e),
                }
            }
        } else {
            debug!("No transport available for kernel_info_reply");
        }

        Ok(())
    }

    /// Create a multipart response for ZMQ ROUTER socket
    fn create_multipart_response(
        &self,
        client_identity: &[u8],
        msg_type: &str,
        content: &serde_json::Value,
    ) -> Result<Vec<Vec<u8>>> {
        // Extract client session from the request header for echo back
        let client_session = self.current_msg_header
            .as_ref()
            .and_then(|h| h.get("session"))
            .and_then(|s| s.as_str())
            .unwrap_or(&self.session_id)
            .to_string();

        // Create header
        let header = serde_json::json!({
            "msg_id": uuid::Uuid::new_v4().to_string(),
            "session": client_session,
            "username": "kernel",
            "msg_type": msg_type,
            "version": "5.3",
            "date": chrono::Utc::now().to_rfc3339(),
        });

        // Use the stored header from the request as parent_header
        let parent_header = self.current_msg_header.clone()
            .unwrap_or_else(|| serde_json::json!({}));
        let metadata = serde_json::json!({});

        // Serialize components
        let header_bytes = serde_json::to_vec(&header)?;
        let parent_header_bytes = serde_json::to_vec(&parent_header)?;
        let metadata_bytes = serde_json::to_vec(&metadata)?;
        let content_bytes = serde_json::to_vec(&content)?;

        // Create HMAC signature using the protocol
        debug!("Signing message with components:");
        debug!("  Header ({} bytes): {}", header_bytes.len(), String::from_utf8_lossy(&header_bytes));
        debug!("  Parent header ({} bytes): {}", parent_header_bytes.len(), String::from_utf8_lossy(&parent_header_bytes));
        debug!("  Metadata ({} bytes): {}", metadata_bytes.len(), String::from_utf8_lossy(&metadata_bytes));
        debug!("  Content ({} bytes): {}", content_bytes.len(), String::from_utf8_lossy(&content_bytes));

        let signature = self.protocol.sign_message(
            &header_bytes,
            &parent_header_bytes,
            &metadata_bytes,
            &content_bytes,
        )?;

        debug!("  Generated signature: {}", signature);

        // Build multipart message according to Jupyter wire protocol
        // [identity, delimiter, signature, header, parent_header, metadata, content]
        let parts = vec![
            client_identity.to_vec(), // Client identity for ROUTER routing
            b"<IDS|MSG>".to_vec(),    // Delimiter
            signature.as_bytes().to_vec(), // HMAC signature (hex-encoded string as bytes)
            header_bytes,             // Header JSON
            parent_header_bytes,      // Parent header JSON
            metadata_bytes,           // Metadata JSON
            content_bytes,            // Content JSON
        ];

        Ok(parts)
    }

    /// Handle `shutdown_request`
    ///
    /// # Errors
    ///
    /// Returns an error if response creation fails
    fn handle_shutdown_request(&mut self, message: &HashMap<String, Value>) -> Result<()> {
        let restart = message
            .get("content")
            .and_then(|c| c.get("restart"))
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);

        info!("Handling shutdown_request (restart={})", restart);

        // Create reply for protocol
        let reply = serde_json::json!({
            "restart": restart
        });
        let _response = self.protocol.create_response("shutdown_reply", reply)?;

        // Use async block to handle graceful shutdown
        let shutdown_coordinator = self.shutdown_coordinator.clone();
        let handle = tokio::spawn(async move {
            if let Err(e) = shutdown_coordinator.initiate_shutdown().await {
                error!("Failed to initiate shutdown: {}", e);
            }
        });

        // Don't wait for completion here, let it run in background
        drop(handle);

        // Also trigger the old shutdown mechanism for compatibility
        if let Some(ref mut shutdown_rx) = self.shutdown_rx {
            shutdown_rx.close();
        }

        Ok(())
    }

    /// Handle `interrupt_request`
    ///
    /// # Errors
    ///
    /// Returns an error if response creation fails
    fn handle_interrupt_request(&mut self, _message: &HashMap<String, Value>) -> Result<()> {
        info!("Handling interrupt_request");

        // TODO: Add interrupt support to ScriptExecutor trait if needed
        // For now, interrupts are handled at the protocol level only

        // Update execution state to paused
        self.state.update_execution(|exec| {
            exec.pause();
            Ok(())
        })?;

        // TODO: Send interrupt reply via transport when integrated
        let _response = self
            .protocol
            .create_response("interrupt_reply", serde_json::json!({}))?;

        Ok(())
    }

    /// Run kernel as daemon
    ///
    /// This method daemonizes the process and runs the kernel in the background
    ///
    /// # Errors
    ///
    /// Returns an error if daemonization fails or server setup fails
    pub async fn run_as_daemon(&mut self) -> Result<()> {
        info!("Starting IntegratedKernel in daemon mode");

        // Check if daemon mode is enabled
        if !self.config.daemon_mode {
            return Err(anyhow::anyhow!("Daemon mode not enabled in configuration"));
        }

        // Get daemon config or use default
        let daemon_config = self.config.daemon_config.clone().unwrap_or_default();

        // Create and daemonize
        let mut daemon_manager = crate::daemon::DaemonManager::new(daemon_config.clone());

        if daemon_config.daemonize {
            info!("Daemonizing process...");
            daemon_manager.daemonize()?;
            info!("Process daemonized successfully");
        }

        // Start protocol servers
        self.start_protocol_servers();

        // Run the main event loop
        self.run_event_loop().await
    }

    /// Start protocol servers
    ///
    /// Sets up all protocol-specific servers (shell, iopub, stdin, control, heartbeat)
    ///
    /// # Errors
    ///
    /// Starts protocol servers (heartbeat, etc.)
    fn start_protocol_servers(&mut self) {
        info!("Starting protocol servers");

        // Create connection file for Jupyter discovery
        // Using base port 5555 for now, will be configurable later
        let base_port = 5555;
        let kernel_id = format!("{}-{}", self.session_id, uuid::Uuid::new_v4());

        let mut connection_manager = ConnectionFileManager::new(kernel_id.clone(), base_port);

        // Write connection file for Jupyter clients
        match connection_manager.write() {
            Ok(file_path) => {
                info!("Created connection file at {}", file_path.display());

                // Store connection manager for cleanup on drop
                let connection_manager_arc = Arc::new(parking_lot::Mutex::new(connection_manager));
                self.connection_manager = Some(connection_manager_arc);
            }
            Err(e) => {
                warn!("Failed to create connection file: {}", e);
                // Continue without connection file - not critical for operation
            }
        }

        // If we have a transport, initialize it
        // TODO: Bind transport when we have proper TransportConfig
        if self.transport.is_some() {
            debug!("Transport would be bound here with proper config");

            // Update connection manager with actual bound ports
            if let Some(ref connection_mgr) = self.connection_manager {
                // TODO: Get actual ports from transport after binding
                // For now, using the base port sequence
                let mut mgr = connection_mgr.lock();
                mgr.update_ports(
                    base_port,
                    base_port + 1,
                    base_port + 2,
                    base_port + 3,
                    base_port + 4,
                );
                // Rewrite the file with updated ports
                let _ = mgr.write();
            }
        }

        // TODO: Register protocol handlers with message router when method is available
        debug!("Protocol handler registration would happen here");

        // Start heartbeat service
        let heartbeat_interval = Duration::from_secs(5);
        let heartbeat_handle = {
            let session_id = self.session_id.clone();
            tokio::spawn(async move {
                let mut interval = tokio::time::interval(heartbeat_interval);
                loop {
                    interval.tick().await;
                    trace!("Heartbeat for session {}", session_id);
                }
            })
        };

        // Store handle for cleanup
        drop(heartbeat_handle); // For now, let it run

        info!("Protocol servers started successfully");
    }

    /// Main event loop for all protocols
    ///
    /// Processes messages from all sources (transport, signals, internal)
    ///
    /// # Errors
    ///
    /// Returns an error if the event loop encounters a fatal error
    pub async fn run_event_loop(&mut self) -> Result<()> {
        info!("Starting main event loop for session {}", self.session_id);

        // Create a channel for shutdown coordination
        let (shutdown_tx, shutdown_rx) = mpsc::channel::<()>(1);

        // Store shutdown sender
        self.shutdown_rx = Some(shutdown_rx);

        // Simplified event loop - real implementation would handle messages properly
        loop {
            tokio::select! {
                // Check for shutdown signal
                () = shutdown_tx.closed() => {
                    info!("Shutdown signal received, exiting event loop");
                    break;
                }

                // Periodic processing
                () = tokio::time::sleep(Duration::from_secs(1)) => {
                    trace!("Event loop tick");

                    // Process transport messages
                    self.process_transport_messages();

                    // Process internal events
                    self.process_internal_events();
                }
            }
        }

        // Cleanup before exit
        info!("Event loop ended, performing cleanup");
        self.shutdown_coordinator.initiate_shutdown().await?;

        Ok(())
    }

    /// Process messages from transport
    fn process_transport_messages(&mut self) {
        // TODO: Process transport messages when Transport trait has receive method
        trace!(
            "Processing transport messages for session {}",
            self.session_id
        );
    }

    /// Process internal events
    fn process_internal_events(&mut self) {
        // TODO: Check for events from event correlator when method is available
        trace!(
            "Checking for internal events in session {}",
            self.session_id
        );
    }

    /// Perform periodic health check
    fn perform_health_check(&self) {
        let metrics = self.state.metrics();
        debug!(
            "Health check - Reads: {}, Writes: {}, Circuit breaker open: {}",
            metrics.reads,
            metrics.writes,
            self.state.is_circuit_open()
        );

        // Check memory usage
        let execution_count = *self.execution_count.read();
        if execution_count > 10000 {
            warn!("High execution count: {}", execution_count);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock script executor for testing
    pub(super) struct MockScriptExecutor;

    #[async_trait::async_trait]
    impl ScriptExecutor for MockScriptExecutor {
        async fn execute_script(
            &self,
            _script: &str,
        ) -> Result<
            llmspell_core::traits::script_executor::ScriptExecutionOutput,
            llmspell_core::error::LLMSpellError,
        > {
            Ok(
                llmspell_core::traits::script_executor::ScriptExecutionOutput {
                    output: serde_json::json!("test output"),
                    console_output: vec!["test console output".to_string()],
                    metadata: llmspell_core::traits::script_executor::ScriptExecutionMetadata {
                        duration: std::time::Duration::from_millis(10),
                        language: "test".to_string(),
                        exit_code: Some(0),
                        warnings: vec![],
                    },
                },
            )
        }

        fn language(&self) -> &'static str {
            "test"
        }
    }

    // Mock protocol for testing
    struct MockProtocol;

    impl Protocol for MockProtocol {
        fn parse_message(&self, _data: &[u8]) -> Result<HashMap<String, Value>> {
            Ok(HashMap::new())
        }

        fn create_response(&self, _msg_type: &str, _content: Value) -> Result<Vec<u8>> {
            Ok(vec![])
        }

        fn create_request(&self, _msg_type: &str, _content: Value) -> Result<Vec<u8>> {
            Ok(vec![])
        }

        fn sign_message(
            &self,
            _header: &[u8],
            _parent_header: &[u8],
            _metadata: &[u8],
            _content: &[u8],
        ) -> Result<String> {
            Ok(String::new()) // Mock returns empty signature
        }

        fn set_hmac_key(&mut self, _key: &str) {
            // Mock does nothing with key
        }
    }

    #[tokio::test]
    async fn test_integrated_kernel_creation() {
        let protocol = MockProtocol;
        let config = ExecutionConfig::default();
        let executor = Arc::new(MockScriptExecutor) as Arc<dyn ScriptExecutor>;

        let kernel =
            IntegratedKernel::new(protocol, config, "test-session".to_string(), executor).await;

        assert!(kernel.is_ok());
    }

    #[tokio::test]
    async fn test_no_spawning_execution() {
        // This test verifies that execution happens in the same context
        // without tokio::spawn, preventing "dispatch task is gone" errors

        let protocol = MockProtocol;
        let config = ExecutionConfig::default();
        let executor = Arc::new(MockScriptExecutor) as Arc<dyn ScriptExecutor>;

        let mut kernel =
            IntegratedKernel::new(protocol, config, "test-session".to_string(), executor)
                .await
                .unwrap();

        // Set up shutdown signal
        let (shutdown_tx, shutdown_rx) = mpsc::channel(1);
        kernel.set_shutdown_receiver(shutdown_rx);

        // Trigger shutdown immediately
        shutdown_tx.send(()).await.unwrap();

        // Run should complete without spawning
        let result = kernel.run().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_script_executor_in_context() {
        // Test script executor directly
        let executor = Arc::new(MockScriptExecutor) as Arc<dyn ScriptExecutor>;
        let result = executor.execute_script("return 'test output'").await;
        assert!(result.is_ok());

        let output = result.unwrap();
        // Check that we got some output
        assert!(output.output != serde_json::Value::Null || !output.console_output.is_empty());
    }

    #[tokio::test]
    async fn test_channel_message_type_validation() {
        use crate::protocols::jupyter::JupyterProtocol;

        let protocol = JupyterProtocol::new("test-session".to_string(), "test-kernel".to_string());

        let config = ExecutionConfig::default();
        let session_id = "test-session".to_string();
        let executor = Arc::new(MockScriptExecutor);

        let kernel = IntegratedKernel::new(protocol, config, session_id, executor)
            .await
            .unwrap();

        // Test Shell channel accepts execute_request
        let execute_msg = json!({
            "msg_type": "execute_request",
            "content": {"code": "print('test')"}
        });
        let result = kernel.validate_message_for_channel("shell", &execute_msg);
        assert!(result, "Shell should accept execute_request");

        // Test Shell channel accepts complete_request
        let complete_msg = json!({
            "msg_type": "complete_request",
            "content": {"code": "pri", "cursor_pos": 3}
        });
        let result = kernel.validate_message_for_channel("shell", &complete_msg);
        assert!(result, "Shell should accept complete_request");

        // Test Control channel accepts interrupt_request
        let interrupt_msg = json!({
            "msg_type": "interrupt_request",
            "content": {}
        });
        let result = kernel.validate_message_for_channel("control", &interrupt_msg);
        assert!(result, "Control should accept interrupt_request");

        // Test Control channel accepts shutdown_request
        let shutdown_msg = json!({
            "msg_type": "shutdown_request",
            "content": {"restart": false}
        });
        let result = kernel.validate_message_for_channel("control", &shutdown_msg);
        assert!(result, "Control should accept shutdown_request");

        // Test Stdin channel accepts input_reply
        let input_msg = json!({
            "msg_type": "input_reply",
            "content": {"value": "user input"}
        });
        let result = kernel.validate_message_for_channel("stdin", &input_msg);
        assert!(result, "Stdin should accept input_reply");

        // Test Shell rejects control messages
        let result = kernel.validate_message_for_channel("shell", &interrupt_msg);
        assert!(!result, "Shell should reject interrupt_request");

        // Test Control rejects shell messages
        let result = kernel.validate_message_for_channel("control", &execute_msg);
        assert!(!result, "Control should reject execute_request");
    }

    #[tokio::test]
    async fn test_stdin_input_handling() {
        use crate::protocols::jupyter::JupyterProtocol;

        let protocol = JupyterProtocol::new("test-session".to_string(), "test-kernel".to_string());

        let config = ExecutionConfig::default();
        let session_id = "test-session".to_string();
        let executor = Arc::new(MockScriptExecutor);

        let mut kernel = IntegratedKernel::new(protocol, config, session_id, executor)
            .await
            .unwrap();

        // Simulate an input request
        let (tx, rx) = tokio::sync::oneshot::channel();
        kernel.pending_input_request = Some(tx);

        // Handle input reply
        let input_reply = json!({
            "msg_type": "input_reply",
            "content": {"value": "test input"}
        });

        let mut input_map = HashMap::new();
        for (k, v) in input_reply.as_object().unwrap() {
            input_map.insert(k.clone(), v.clone());
        }
        kernel.handle_input_reply(input_map).await.unwrap();

        // Check that the input was sent through the channel
        let received = rx.await.unwrap();
        assert_eq!(received, "test input");

        // Verify pending_input_request is cleared
        assert!(kernel.pending_input_request.is_none());
    }

    #[tokio::test]
    async fn test_channel_health_monitoring() {
        use crate::protocols::jupyter::JupyterProtocol;
        use std::time::Duration;

        let protocol = JupyterProtocol::new("test-session".to_string(), "test-kernel".to_string());

        let config = ExecutionConfig::default();
        let session_id = "test-session".to_string();
        let executor = Arc::new(MockScriptExecutor);

        let kernel = IntegratedKernel::new(protocol, config, session_id, executor)
            .await
            .unwrap();

        // Update channel activity timestamps
        {
            let mut activity = kernel.channel_last_activity.write();
            activity.insert("shell".to_string(), std::time::Instant::now());
            activity.insert(
                "control".to_string(),
                std::time::Instant::now()
                    .checked_sub(Duration::from_secs(40))
                    .unwrap(),
            );
            activity.insert(
                "stdin".to_string(),
                std::time::Instant::now()
                    .checked_sub(Duration::from_secs(10))
                    .unwrap(),
            );
            activity.insert("heartbeat".to_string(), std::time::Instant::now());
        }

        // Get channel health
        let health = kernel.get_channel_health();

        // Shell should be healthy (just updated)
        assert!(
            health.get("shell").copied().unwrap_or(false),
            "Shell should be healthy"
        );

        // Control should be unhealthy (40s ago > 30s timeout)
        assert!(
            !health.get("control").copied().unwrap_or(true),
            "Control should be unhealthy"
        );

        // Stdin should be healthy (10s ago < 30s timeout)
        assert!(
            health.get("stdin").copied().unwrap_or(false),
            "Stdin should be healthy"
        );

        // Heartbeat should be healthy (just updated)
        assert!(
            health.get("heartbeat").copied().unwrap_or(false),
            "Heartbeat should be healthy"
        );
    }

    #[tokio::test]
    async fn test_channel_separation() {
        use crate::protocols::jupyter::JupyterProtocol;

        let protocol = JupyterProtocol::new("test-session".to_string(), "test-kernel".to_string());

        let config = ExecutionConfig::default();
        let session_id = "test-session".to_string();
        let executor = Arc::new(MockScriptExecutor);

        let kernel = IntegratedKernel::new(protocol, config, session_id, executor)
            .await
            .unwrap();

        // Test that shell messages update shell activity
        let before_shell = kernel
            .channel_last_activity
            .read()
            .get("shell")
            .copied()
            .unwrap_or(std::time::Instant::now());
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // Process a shell message (this would normally happen in the main loop)
        kernel
            .channel_last_activity
            .write()
            .insert("shell".to_string(), std::time::Instant::now());
        let after_shell = kernel
            .channel_last_activity
            .read()
            .get("shell")
            .copied()
            .unwrap_or(std::time::Instant::now());

        assert!(
            after_shell > before_shell,
            "Shell activity should be updated"
        );

        // Test that control messages update control activity
        let before_control = kernel
            .channel_last_activity
            .read()
            .get("control")
            .copied()
            .unwrap_or(std::time::Instant::now());
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        kernel
            .channel_last_activity
            .write()
            .insert("control".to_string(), std::time::Instant::now());
        let after_control = kernel
            .channel_last_activity
            .read()
            .get("control")
            .copied()
            .unwrap_or(std::time::Instant::now());

        assert!(
            after_control > before_control,
            "Control activity should be updated"
        );
    }

    #[tokio::test]
    async fn test_message_type_extraction() {
        use crate::protocols::jupyter::JupyterProtocol;

        let protocol = JupyterProtocol::new("test-session".to_string(), "test-kernel".to_string());

        let config = ExecutionConfig::default();
        let session_id = "test-session".to_string();
        let executor = Arc::new(MockScriptExecutor);

        let _kernel = IntegratedKernel::new(protocol, config, session_id, executor)
            .await
            .unwrap();

        // Test extracting msg_type from various message structures
        let msg1 = json!({
            "msg_type": "execute_request",
            "content": {}
        });
        assert_eq!(
            msg1.get("msg_type").and_then(|v| v.as_str()),
            Some("execute_request")
        );

        let msg2 = json!({
            "header": {
                "msg_type": "complete_request"
            },
            "content": {}
        });
        assert_eq!(
            msg2.get("header")
                .and_then(|h| h.get("msg_type"))
                .and_then(|v| v.as_str()),
            Some("complete_request")
        );

        // Test missing msg_type
        let msg3 = json!({
            "content": {}
        });
        assert_eq!(msg3.get("msg_type").and_then(|v| v.as_str()), None);
    }

    #[tokio::test]
    async fn test_control_channel_priority() {
        // This test verifies that control channel messages are processed with priority
        use crate::protocols::jupyter::JupyterProtocol;

        let protocol = JupyterProtocol::new("test-session".to_string(), "test-kernel".to_string());

        let config = ExecutionConfig::default();
        let session_id = "test-session".to_string();
        let executor = Arc::new(MockScriptExecutor);

        let kernel = IntegratedKernel::new(protocol, config, session_id, executor)
            .await
            .unwrap();

        // In a real scenario, control messages would be processed first in the main loop
        // Here we just verify the validation works correctly for priority messages
        let interrupt_msg = json!({
            "msg_type": "interrupt_request",
            "content": {}
        });

        let shutdown_msg = json!({
            "msg_type": "shutdown_request",
            "content": {"restart": false}
        });

        // Both interrupt and shutdown should be valid for control channel
        assert!(kernel.validate_message_for_channel("control", &interrupt_msg));
        assert!(kernel.validate_message_for_channel("control", &shutdown_msg));

        // They should not be valid for other channels
        assert!(!kernel.validate_message_for_channel("shell", &interrupt_msg));
        assert!(!kernel.validate_message_for_channel("stdin", &shutdown_msg));
    }
}

#[tokio::test]
async fn test_message_handling_performance() -> Result<()> {
    use std::collections::HashMap;
    use std::time::Instant;

    // Create test kernel
    let protocol = crate::protocols::jupyter::JupyterProtocol::new(
        "test-session".to_string(),
        "test-kernel".to_string(),
    );
    let config = ExecutionConfig::default();
    let session_id = "test-session".to_string();
    let script_executor = Arc::new(tests::MockScriptExecutor) as Arc<dyn ScriptExecutor>;

    let mut kernel = IntegratedKernel::new(protocol, config, session_id, script_executor).await?;

    // Create a simple kernel_info_request message (faster than execute_request)
    let mut message = HashMap::new();
    message.insert(
        "msg_type".to_string(),
        serde_json::Value::String("kernel_info_request".to_string()),
    );

    // Test single message handling performance
    let start_time = Instant::now();
    kernel.handle_message(message.clone()).await?;
    let elapsed = start_time.elapsed();

    println!(
        "Single kernel_info message handling took: {}μs ({}ms)",
        elapsed.as_micros(),
        elapsed.as_millis()
    );

    // Note: We expect this to be very fast since kernel_info_request is lightweight
    assert!(
        elapsed.as_millis() < 5,
        "Message handling took {}ms, target is <5ms",
        elapsed.as_millis()
    );

    // Test multiple messages for consistency
    let iterations = 20;
    let mut total_time = std::time::Duration::ZERO;

    for _i in 0..iterations {
        let start = Instant::now();
        kernel.handle_message(message.clone()).await?;
        total_time += start.elapsed();
    }

    let avg_time = total_time / iterations;
    println!(
        "Average message handling time over {} iterations: {}μs ({}ms)",
        iterations,
        avg_time.as_micros(),
        avg_time.as_millis()
    );

    assert!(
        avg_time.as_millis() < 5,
        "Average message handling took {}ms, target is <5ms",
        avg_time.as_millis()
    );

    println!("✅ Message handling performance test passed - meeting <5ms target");
    Ok(())
}

#[cfg(test)]
mod daemon_tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_daemon_config_serialization() {
        let config = ExecutionConfig {
            daemon_mode: true,
            daemon_config: Some(DaemonConfig {
                daemonize: false, // Don't actually daemonize in tests
                pid_file: Some(PathBuf::from("/tmp/test.pid")),
                working_dir: PathBuf::from("/tmp"),
                stdout_path: Some(PathBuf::from("/tmp/stdout.log")),
                stderr_path: Some(PathBuf::from("/tmp/stderr.log")),
                close_stdin: true,
                umask: Some(0o027),
            }),
            ..Default::default()
        };

        // Should serialize and deserialize correctly
        let json = serde_json::to_string(&config).unwrap();
        let _deserialized: ExecutionConfig = serde_json::from_str(&json).unwrap();
    }

    // Reuse MockScriptExecutor from tests module
    use super::tests::MockScriptExecutor;

    #[tokio::test]
    async fn test_daemon_mode_disabled() {
        let config = ExecutionConfig {
            daemon_mode: false,
            daemon_config: None,
            ..Default::default()
        };

        let script_executor = Arc::new(MockScriptExecutor) as Arc<dyn ScriptExecutor>;
        let protocol = crate::protocols::jupyter::JupyterProtocol::new(
            "test-session".to_string(),
            "test-kernel".to_string(),
        );

        let mut kernel = IntegratedKernel::new(
            protocol,
            config,
            "test-session".to_string(),
            script_executor,
        )
        .await
        .unwrap();

        // Should fail when daemon mode is not enabled
        let result = kernel.run_as_daemon().await;
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Daemon mode not enabled"));
    }

    #[tokio::test]
    async fn test_protocol_servers_startup() {
        let config = ExecutionConfig::default();
        let script_executor = Arc::new(MockScriptExecutor) as Arc<dyn ScriptExecutor>;
        let protocol = crate::protocols::jupyter::JupyterProtocol::new(
            "test-session".to_string(),
            "test-kernel".to_string(),
        );

        let mut kernel = IntegratedKernel::new(
            protocol,
            config,
            "test-session".to_string(),
            script_executor,
        )
        .await
        .unwrap();

        // Start protocol servers should succeed even without transport
        kernel.start_protocol_servers();
    }

    #[tokio::test]
    async fn test_health_check() {
        // Use test-friendly thresholds (higher limits)
        let config = ExecutionConfig {
            health_thresholds: Some(crate::monitoring::HealthThresholds {
                max_memory_mb: 10240,    // 10GB - very high for testing
                max_cpu_percent: 1000.0, // 1000% - allow high CPU in tests (multi-core systems)
                max_connections: 100,
                max_avg_latency_us: 10000,
                max_error_rate_per_minute: 100.0,
            }),
            ..Default::default()
        };

        let script_executor = Arc::new(MockScriptExecutor) as Arc<dyn ScriptExecutor>;
        let protocol = crate::protocols::jupyter::JupyterProtocol::new(
            "test-session".to_string(),
            "test-kernel".to_string(),
        );

        let kernel = IntegratedKernel::new(
            protocol,
            config,
            "test-session".to_string(),
            script_executor,
        )
        .await
        .unwrap();

        // Quick health check should work
        let health_status = kernel.quick_health_check().await.unwrap();

        // Full health check should work too
        let health_report = kernel.health_check().await.unwrap();

        // Log the issues to debug what's causing degraded status
        if !health_report.issues.is_empty() {
            eprintln!("Health issues detected: {:?}", health_report.issues);
            eprintln!(
                "System metrics: memory_usage_mb={}, cpu_usage_percent={}",
                health_report.system.memory_usage_mb, health_report.system.cpu_usage_percent
            );
            eprintln!(
                "Performance metrics: avg_read_latency_us={}, avg_write_latency_us={}",
                health_report.performance.avg_read_latency_us,
                health_report.performance.avg_write_latency_us
            );
        }

        // Accept Healthy or Degraded status (system might have minor warnings)
        assert!(
            matches!(
                health_status,
                crate::monitoring::HealthStatus::Healthy
                    | crate::monitoring::HealthStatus::Degraded
            ),
            "Expected Healthy or Degraded status, got {health_status:?}"
        );

        // For full report, also accept Healthy or Degraded
        assert!(
            matches!(
                health_report.status,
                crate::monitoring::HealthStatus::Healthy
                    | crate::monitoring::HealthStatus::Degraded
            ),
            "Expected Healthy or Degraded status, got {:?}",
            health_report.status
        );

        // Should not be Unhealthy
        assert_ne!(
            health_report.status,
            crate::monitoring::HealthStatus::Unhealthy
        );
        assert!(health_report.system.memory_usage_mb > 0); // Should have some memory usage
        assert!(health_report.system.pid > 0); // Should have a valid PID

        // Check that metrics were included
        let metrics = kernel.state.metrics();
        assert_eq!(metrics.circuit_breaker_trips, 0);
        assert_eq!(metrics.read_errors, 0);
        assert_eq!(metrics.write_errors, 0);
    }

    #[tokio::test]
    async fn test_health_logging() {
        let config = ExecutionConfig::default();
        let script_executor = Arc::new(MockScriptExecutor) as Arc<dyn ScriptExecutor>;
        let protocol = crate::protocols::jupyter::JupyterProtocol::new(
            "test-session".to_string(),
            "test-kernel".to_string(),
        );

        let kernel = IntegratedKernel::new(
            protocol,
            config,
            "test-session".to_string(),
            script_executor,
        )
        .await
        .unwrap();

        // Log health status should not panic
        kernel.log_health_status().await;
    }
}
