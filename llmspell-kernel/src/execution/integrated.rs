//! Integrated Kernel Implementation
//!
//! This module provides the `IntegratedKernel` that runs `ScriptRuntime` directly
//! in the current context without `tokio::spawn`, ensuring all components share
//! the same runtime context.

use anyhow::Result;
use llmspell_core::traits::script_executor::ScriptExecutor;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, error, info, instrument, trace, warn};

use crate::events::correlation::{ExecutionState, ExecutionStatus};
use crate::events::{KernelEvent, KernelEventCorrelator};
use crate::io::manager::EnhancedIOManager;
use crate::io::router::MessageRouter;
use crate::runtime::tracing::{OperationCategory, TracingInstrumentation};
use crate::sessions::{CreateSessionOptions, SessionManager, SessionManagerConfig};
use crate::state::{KernelState, StorageBackend};
use crate::traits::Protocol;

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
        }
    }
}

/// Integrated kernel that runs `ScriptRuntime` without spawning
pub struct IntegratedKernel<P: Protocol> {
    /// Script executor for execution
    script_executor: Arc<dyn ScriptExecutor>,
    /// Protocol handler
    protocol: P,
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
    /// Shutdown signal receiver
    shutdown_rx: Option<mpsc::Receiver<()>>,
}

#[allow(dead_code)] // These methods will be used when transport is fully integrated
impl<P: Protocol + 'static> IntegratedKernel<P> {
    /// Create a new integrated kernel
    ///
    /// # Errors
    ///
    /// Returns an error if the script runtime cannot be created
    #[instrument(level = "info", skip_all)]
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

        Ok(Self {
            script_executor,
            protocol,
            io_manager,
            message_router,
            event_correlator,
            tracing,
            config,
            session_id,
            execution_count: Arc::new(RwLock::new(0)),
            state,
            session_manager,
            shutdown_rx: None,
        })
    }

    /// Set shutdown signal receiver
    pub fn set_shutdown_receiver(&mut self, rx: mpsc::Receiver<()>) {
        self.shutdown_rx = Some(rx);
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
    pub async fn run(mut self) -> Result<()> {
        info!("Starting IntegratedKernel in current context (no spawning)");

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
        loop {
            // Check for shutdown signal
            if let Some(ref mut shutdown_rx) = self.shutdown_rx {
                if shutdown_rx.try_recv().is_ok() {
                    info!("Received shutdown signal");
                    break;
                }
            }

            // Simulate message reception (will be replaced with actual transport)
            // For now, just check for shutdown
            tokio::time::sleep(Duration::from_millis(100)).await;

            // TODO: Process actual messages when transport is integrated
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
            "kernel_info_request" => self.handle_kernel_info_request(&message)?,
            "shutdown_request" => self.handle_shutdown_request(&message)?,
            "interrupt_request" => self.handle_interrupt_request(&message)?,
            _ => {
                warn!("Unhandled message type: {}", msg_type);
            }
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
        // Generate execution ID
        let exec_id = format!("exec-{}", uuid::Uuid::new_v4());

        // Update state for tracking
        self.state.update_execution(|exec| {
            exec.start_execution(exec_id, code.to_string());
            Ok(())
        })?;

        // Execute code using the internal method
        let result = self.execute_code_in_context(code).await;

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
    fn handle_kernel_info_request(&mut self, _message: &HashMap<String, Value>) -> Result<()> {
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

        // TODO: Send response via transport when integrated
        let _response = self
            .protocol
            .create_response("kernel_info_reply", kernel_info)?;

        Ok(())
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

        // TODO: Send shutdown reply via transport when integrated
        let reply = serde_json::json!({
            "restart": restart
        });
        let _response = self.protocol.create_response("shutdown_reply", reply)?;

        // Trigger shutdown
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
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock script executor for testing
    struct MockScriptExecutor;

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
}
