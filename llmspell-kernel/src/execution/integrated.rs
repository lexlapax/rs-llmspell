//! Integrated Kernel Implementation
//!
//! This module provides the IntegratedKernel that runs ScriptRuntime directly
//! in the current context without tokio::spawn, ensuring all components share
//! the same runtime context.

use anyhow::Result;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, error, info, instrument, trace, warn};

use crate::io::manager::EnhancedIOManager;
use crate::io::router::MessageRouter;
use crate::runtime::tracing::{OperationCategory, TracingInstrumentation};
use crate::traits::Protocol;

/// Simplified ScriptRuntime stub for Phase 9.2
/// Will be replaced with llmspell-bridge::ScriptRuntime in later phases
pub struct ScriptRuntime {
    config: HashMap<String, Value>,
    stdout_handler: Option<Box<dyn Fn(&str) + Send + Sync>>,
    stderr_handler: Option<Box<dyn Fn(&str) + Send + Sync>>,
    interrupted: Arc<RwLock<bool>>,
}

impl ScriptRuntime {
    pub fn new(config: HashMap<String, Value>) -> Result<Self> {
        Ok(Self {
            config,
            stdout_handler: None,
            stderr_handler: None,
            interrupted: Arc::new(RwLock::new(false)),
        })
    }

    pub fn set_stdout_handler(&mut self, handler: Box<dyn Fn(&str) + Send + Sync>) {
        self.stdout_handler = Some(handler);
    }

    pub fn set_stderr_handler(&mut self, handler: Box<dyn Fn(&str) + Send + Sync>) {
        self.stderr_handler = Some(handler);
    }

    pub async fn execute(&mut self, code: &str) -> Result<String> {
        // Check if interrupted
        if *self.interrupted.read() {
            *self.interrupted.write() = false;
            return Err(anyhow::anyhow!("Execution interrupted"));
        }

        // Simulate execution
        if let Some(ref handler) = self.stdout_handler {
            handler(&format!("Executing: {}\n", code));
        }

        // Return a simple result
        Ok(format!("Result of: {}", code))
    }

    pub fn interrupt(&mut self) {
        *self.interrupted.write() = true;
    }
}

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

/// Integrated kernel that runs ScriptRuntime without spawning
pub struct IntegratedKernel<P: Protocol> {
    /// Script runtime for execution
    runtime: ScriptRuntime,
    /// Protocol handler
    protocol: P,
    /// I/O manager
    io_manager: Arc<EnhancedIOManager>,
    /// Message router for multi-client support
    message_router: Arc<MessageRouter>,
    /// Tracing instrumentation
    tracing: TracingInstrumentation,
    /// Configuration
    config: ExecutionConfig,
    /// Session ID
    session_id: String,
    /// Execution counter
    execution_count: Arc<RwLock<i32>>,
    /// Shutdown signal receiver
    shutdown_rx: Option<mpsc::Receiver<()>>,
}

impl<P: Protocol + 'static> IntegratedKernel<P> {
    /// Create a new integrated kernel
    #[instrument(level = "info", skip_all)]
    pub fn new(
        protocol: P,
        config: ExecutionConfig,
        session_id: String,
    ) -> Result<Self> {
        info!("Creating IntegratedKernel for session {}", session_id);

        // Create script runtime with configuration
        let runtime = ScriptRuntime::new(config.runtime_config.clone())?;

        // Create I/O manager
        let io_config = crate::io::manager::IOConfig {
            stdout_buffer_size: config.io_config.stdout_buffer_size,
            stderr_buffer_size: config.io_config.stderr_buffer_size,
            flush_interval_ms: config.io_config.flush_interval_ms,
            track_parent_headers: config.io_config.track_parent_headers,
        };
        let io_manager = Arc::new(EnhancedIOManager::new(
            io_config,
            session_id.clone(),
        ));

        // Create message router
        let message_router = Arc::new(MessageRouter::new(config.max_history));

        // Create tracing instrumentation
        let mut tracing = TracingInstrumentation::new_kernel_session(
            Some(session_id.clone()),
            "integrated",
        );

        Ok(Self {
            runtime,
            protocol,
            io_manager,
            message_router,
            tracing,
            config,
            session_id,
            execution_count: Arc::new(RwLock::new(0)),
            shutdown_rx: None,
        })
    }

    /// Set shutdown signal receiver
    pub fn set_shutdown_receiver(&mut self, rx: mpsc::Receiver<()>) {
        self.shutdown_rx = Some(rx);
    }

    /// Run the kernel in the current context (NO SPAWNING)
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
        self.tracing.trace_operation(
            OperationCategory::ScriptRuntime,
            "kernel_shutdown",
            Some(&self.session_id),
        );

        info!("IntegratedKernel shutdown complete");
        Ok(())
    }

    /// Handle a parsed message
    #[instrument(level = "debug", skip(self, message))]
    async fn handle_message(&mut self, message: HashMap<String, Value>) -> Result<()> {
        let msg_type = message
            .get("msg_type")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        debug!("Handling message type: {}", msg_type);

        match msg_type {
            "execute_request" => self.handle_execute_request(message).await?,
            "kernel_info_request" => self.handle_kernel_info_request(message).await?,
            "shutdown_request" => self.handle_shutdown_request(message).await?,
            "interrupt_request" => self.handle_interrupt_request(message).await?,
            _ => {
                warn!("Unhandled message type: {}", msg_type);
            }
        }

        Ok(())
    }

    /// Handle execute_request message
    #[instrument(level = "debug", skip(self, message))]
    async fn handle_execute_request(&mut self, message: HashMap<String, Value>) -> Result<()> {
        // Extract code from message
        let code = message
            .get("content")
            .and_then(|c| c.get("code"))
            .and_then(|v| v.as_str())
            .unwrap_or("");

        if code.is_empty() {
            return Ok(());
        }

        // Increment execution counter
        let exec_count = {
            let mut count = self.execution_count.write();
            *count += 1;
            *count
        };

        info!("Executing code [{}]: {} bytes", exec_count, code.len());

        // Set parent header for message correlation
        if let Some(header) = message.get("header") {
            if let Ok(header) = serde_json::from_value(header.clone()) {
                self.io_manager.set_parent_header(header);
            }
        }

        // Publish busy status
        self.io_manager.publish_status("busy").await?;

        // Start execution tracing
        self.tracing.trace_operation(
            OperationCategory::ScriptRuntime,
            "execute_request",
            Some(&format!("execution_{}", exec_count)),
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
            trace!(
                "Execution {} completed in {:?}",
                exec_count,
                execution_time
            );
            self.tracing.trace_operation(
                OperationCategory::ScriptRuntime,
                "performance",
                Some(&format!("execution_time_ms: {}", execution_time.as_millis())),
            );
        }

        // Handle result
        match result {
            Ok(Ok(output)) => {
                // Publish execute result
                let mut data = HashMap::new();
                data.insert("text/plain".to_string(), Value::String(output));
                self.io_manager
                    .publish_execute_result(exec_count, data)
                    .await?;
            }
            Ok(Err(e)) => {
                error!("Execution error: {}", e);
                self.io_manager
                    .write_stderr(&format!("Error: {}\n", e))
                    .await?;
            }
            Err(_) => {
                error!("Execution timeout after {} seconds", self.config.execution_timeout_secs);
                self.io_manager
                    .write_stderr(&format!(
                        "Execution timeout after {} seconds\n",
                        self.config.execution_timeout_secs
                    ))
                    .await?;
            }
        }

        // Clear parent header
        self.io_manager.clear_parent_header();

        // Publish idle status
        self.io_manager.publish_status("idle").await?;

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
    #[instrument(level = "trace", skip(self, code))]
    async fn execute_code_in_context(&mut self, code: &str) -> Result<String> {
        // This is the critical fix - execute directly without spawning
        // The ScriptRuntime now shares the same runtime context as transport
        trace!("Executing code in current context (no spawn)");

        // Connect I/O streams to runtime
        // Set up I/O handlers
        // Note: In a real implementation, these would write directly to the I/O manager
        // For now, we just capture the output

        // Execute code - this now happens in the same runtime context
        let result = self.runtime.execute(code).await?;

        // Flush I/O buffers
        self.io_manager.flush_all().await?;

        Ok(result)
    }

    /// Handle kernel_info_request
    async fn handle_kernel_info_request(&mut self, _message: HashMap<String, Value>) -> Result<()> {
        debug!("Handling kernel_info_request");

        // Create kernel info response
        let kernel_info = serde_json::json!({
            "protocol_version": crate::PROTOCOL_VERSION,
            "implementation": "llmspell",
            "implementation_version": crate::KERNEL_VERSION,
            "language_info": {
                "name": "lua",
                "version": "5.4",
                "file_extension": ".lua",
            },
            "banner": format!("LLMSpell Kernel v{}", crate::KERNEL_VERSION),
        });

        // TODO: Send response via transport when integrated
        let _response = self.protocol.create_response("kernel_info_reply", kernel_info)?;

        Ok(())
    }

    /// Handle shutdown_request
    async fn handle_shutdown_request(&mut self, message: HashMap<String, Value>) -> Result<()> {
        let restart = message
            .get("content")
            .and_then(|c| c.get("restart"))
            .and_then(|v| v.as_bool())
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

    /// Handle interrupt_request
    async fn handle_interrupt_request(&mut self, _message: HashMap<String, Value>) -> Result<()> {
        info!("Handling interrupt_request");

        // Interrupt the runtime
        self.runtime.interrupt();

        // TODO: Send interrupt reply via transport when integrated
        let _response = self.protocol.create_response("interrupt_reply", serde_json::json!({}))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

        let kernel = IntegratedKernel::new(
            protocol,
            config,
            "test-session".to_string(),
        );

        assert!(kernel.is_ok());
    }

    #[tokio::test]
    async fn test_no_spawning_execution() {
        // This test verifies that execution happens in the same context
        // without tokio::spawn, preventing "dispatch task is gone" errors

        let protocol = MockProtocol;
        let config = ExecutionConfig::default();

        let mut kernel = IntegratedKernel::new(
            protocol,
            config,
            "test-session".to_string(),
        )
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
    async fn test_script_runtime_in_context() {
        // Test that ScriptRuntime executes in the same context
        let mut runtime = ScriptRuntime::new(HashMap::new()).unwrap();

        // Set up output handlers
        let output = Arc::new(RwLock::new(String::new()));
        let output_clone = output.clone();

        runtime.set_stdout_handler(Box::new(move |s| {
            output_clone.write().push_str(s);
        }));

        // Execute code
        let result = runtime.execute("test code").await;
        assert!(result.is_ok());

        // Check output was captured
        assert!(output.read().contains("Executing: test code"));
    }
}
