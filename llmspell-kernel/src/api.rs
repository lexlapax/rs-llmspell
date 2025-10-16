//! High-level kernel API for embedded and client modes
//!
//! This module provides the public API for working with kernels,
//! abstracting away the complexity of protocols and transports.

use crate::execution::integrated::IntegratedKernel;
use crate::protocols::jupyter::JupyterProtocol;
use crate::traits::protocol::Protocol;
use crate::traits::{ChannelConfig, Transport, TransportConfig};
use crate::transport::inprocess::InProcessTransport;
use anyhow::Result;
use async_trait::async_trait;
use llmspell_config::LLMSpellConfig;
use llmspell_core::traits::script_executor::{
    ScriptExecutionMetadata, ScriptExecutionOutput, ScriptExecutor,
};
use llmspell_core::{Agent, ComponentLookup, Tool, Workflow};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{debug, error, info, trace, warn};
use uuid::Uuid;

/// Configuration for starting a kernel service
pub struct KernelServiceConfig {
    /// Port to listen on
    pub port: u16,
    /// Execution configuration (includes daemon settings)
    pub exec_config: crate::execution::ExecutionConfig,
    /// Optional kernel ID
    pub kernel_id: Option<String>,
    /// Optional connection file path
    pub connection_file_path: Option<PathBuf>,
    /// Maximum number of clients (TODO: Implement limiting)
    pub max_clients: usize,
    /// Log rotation size limit in bytes
    pub log_rotate_size: Option<u64>,
    /// Number of log files to keep
    pub log_rotate_count: usize,
    /// Script executor implementation
    pub script_executor: Arc<dyn ScriptExecutor>,
}

/// Handle for an embedded kernel running in-process
pub struct KernelHandle {
    kernel: IntegratedKernel<JupyterProtocol>,
    kernel_id: String,
    transport: Arc<InProcessTransport>,
    protocol: JupyterProtocol,
}

impl KernelHandle {
    /// Run the kernel until shutdown
    ///
    /// # Errors
    ///
    /// Returns an error if the kernel fails to run
    pub async fn run(self) -> Result<()> {
        info!("Running embedded kernel {}", self.kernel_id);
        self.kernel.run().await
    }

    /// Execute code and return result
    ///
    /// # Errors
    ///
    /// Returns an error if the execution fails or communication with kernel fails
    pub async fn execute(&mut self, code: &str) -> Result<String> {
        debug!("Executing code in kernel {}", self.kernel_id);

        // Create execute_request message
        let content = serde_json::json!({
            "code": code,
            "silent": false,
            "store_history": true,
            "user_expressions": {},
            "allow_stdin": false,
        });

        let request = self.protocol.create_request("execute_request", content)?;

        // Send request through transport
        self.transport.send("shell", vec![request]).await?;

        // Wait for execute_reply
        loop {
            if let Some(reply_parts) = self.transport.recv("shell").await? {
                if let Some(first_part) = reply_parts.first() {
                    // Parse reply and extract result
                    let reply_msg = self.protocol.parse_message(first_part)?;
                    if let Some(content) = reply_msg.get("content") {
                        // Extract execution result
                        return Ok(format!("Result: {content:?}"));
                    }
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
    }

    /// Send a tool request to the kernel and return the response
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or communication with kernel fails
    pub async fn send_tool_request(
        &mut self,
        content: serde_json::Value,
    ) -> Result<serde_json::Value> {
        debug!("Sending tool request to kernel {}", self.kernel_id);

        // Create tool_request message
        let request = self.protocol.create_request("tool_request", content)?;

        debug!(
            "Sending tool_request on shell channel, message size: {}",
            request.len()
        );

        // Send request through transport
        self.transport.send("shell", vec![request]).await?;

        // Wait for tool_reply
        let start_time = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(900);

        loop {
            if start_time.elapsed() > timeout {
                return Err(anyhow::anyhow!("Timeout waiting for tool_reply"));
            }

            if let Some(reply_parts) = self.transport.recv("shell").await? {
                trace!(
                    "Client received {} parts on shell channel",
                    reply_parts.len()
                );

                // Handle multipart Jupyter wire protocol format
                let delimiter = b"<IDS|MSG>";
                let delimiter_idx = reply_parts
                    .iter()
                    .position(|part| part.as_slice() == delimiter);

                let reply_msg: HashMap<String, serde_json::Value> = if let Some(idx) = delimiter_idx
                {
                    // Parse multipart message (header at idx+2, content at idx+5)
                    if reply_parts.len() > idx + 5 {
                        let header =
                            serde_json::from_slice::<serde_json::Value>(&reply_parts[idx + 2])?;
                        let content =
                            serde_json::from_slice::<serde_json::Value>(&reply_parts[idx + 5])?;

                        let mut msg = HashMap::new();
                        msg.insert("header".to_string(), header);
                        msg.insert("content".to_string(), content);
                        msg
                    } else {
                        continue; // Incomplete message, wait for next
                    }
                } else if let Some(first_part) = reply_parts.first() {
                    // Try parsing as simple JSON message for backward compatibility
                    match self.protocol.parse_message(first_part) {
                        Ok(msg) => msg,
                        Err(_) => continue, // Not a valid message, wait for next
                    }
                } else {
                    continue; // No parts, wait for next
                };

                // Check if this is a tool_reply
                if let Some(header) = reply_msg.get("header") {
                    if let Some(msg_type) = header.get("msg_type") {
                        if msg_type == "tool_reply" {
                            // Extract and return the content's content field
                            if let Some(content_wrapper) = reply_msg.get("content") {
                                // The content contains the actual response nested in a "content" field
                                if let Some(actual_content) = content_wrapper.get("content") {
                                    return Ok(actual_content.clone());
                                }
                                return Ok(content_wrapper.clone());
                            }
                        }
                    }
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
    }

    /// Send a template request to the kernel and return the response
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or communication with kernel fails
    pub async fn send_template_request(
        &mut self,
        content: serde_json::Value,
    ) -> Result<serde_json::Value> {
        debug!("Sending template request to kernel {}", self.kernel_id);

        // Create template_request message
        let request = self.protocol.create_request("template_request", content)?;

        debug!(
            "Sending template_request on shell channel, message size: {}",
            request.len()
        );

        // Send request through transport
        self.transport.send("shell", vec![request]).await?;

        // Wait for template_reply
        let start_time = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(900);

        loop {
            if start_time.elapsed() > timeout {
                return Err(anyhow::anyhow!("Timeout waiting for template_reply"));
            }

            if let Some(reply_parts) = self.transport.recv("shell").await? {
                trace!(
                    "Client received {} parts on shell channel",
                    reply_parts.len()
                );

                // Handle multipart Jupyter wire protocol format
                let delimiter = b"<IDS|MSG>";
                let delimiter_idx = reply_parts
                    .iter()
                    .position(|part| part.as_slice() == delimiter);

                let reply_msg: HashMap<String, serde_json::Value> = if let Some(idx) = delimiter_idx
                {
                    // Parse multipart message (header at idx+2, content at idx+5)
                    if reply_parts.len() > idx + 5 {
                        let header =
                            serde_json::from_slice::<serde_json::Value>(&reply_parts[idx + 2])?;
                        let content =
                            serde_json::from_slice::<serde_json::Value>(&reply_parts[idx + 5])?;

                        let mut msg = HashMap::new();
                        msg.insert("header".to_string(), header);
                        msg.insert("content".to_string(), content);
                        msg
                    } else {
                        continue; // Incomplete message, wait for next
                    }
                } else if let Some(first_part) = reply_parts.first() {
                    // Try parsing as simple JSON message for backward compatibility
                    match self.protocol.parse_message(first_part) {
                        Ok(msg) => msg,
                        Err(_) => continue, // Not a valid message, wait for next
                    }
                } else {
                    continue; // No parts, wait for next
                };

                // Check if this is a template_reply
                if let Some(header) = reply_msg.get("header") {
                    if let Some(msg_type) = header.get("msg_type") {
                        if msg_type == "template_reply" {
                            // Extract and return the content's content field
                            if let Some(content_wrapper) = reply_msg.get("content") {
                                // The content contains the actual response nested in a "content" field
                                if let Some(actual_content) = content_wrapper.get("content") {
                                    return Ok(actual_content.clone());
                                }
                                return Ok(content_wrapper.clone());
                            }
                        }
                    }
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
    }

    /// Send a model management request to the kernel and return the response
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or communication with kernel fails
    pub async fn send_model_request(
        &mut self,
        content: serde_json::Value,
    ) -> Result<serde_json::Value> {
        debug!("Sending model request to kernel {}", self.kernel_id);

        // Create model_request message
        let request = self.protocol.create_request("model_request", content)?;

        debug!(
            "Sending model_request on shell channel, message size: {}",
            request.len()
        );

        // Send request through transport
        self.transport.send("shell", vec![request]).await?;

        // Wait for model_reply
        let start_time = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(900);

        loop {
            if start_time.elapsed() > timeout {
                return Err(anyhow::anyhow!("Timeout waiting for model_reply"));
            }

            if let Some(reply_parts) = self.transport.recv("shell").await? {
                trace!(
                    "Client received {} parts on shell channel",
                    reply_parts.len()
                );

                // Handle multipart Jupyter wire protocol format
                let delimiter = b"<IDS|MSG>";
                let delimiter_idx = reply_parts
                    .iter()
                    .position(|part| part.as_slice() == delimiter);

                let reply_msg: HashMap<String, serde_json::Value> = if let Some(idx) = delimiter_idx
                {
                    // Parse multipart message (header at idx+2, content at idx+5)
                    if reply_parts.len() > idx + 5 {
                        let header =
                            serde_json::from_slice::<serde_json::Value>(&reply_parts[idx + 2])?;
                        let content =
                            serde_json::from_slice::<serde_json::Value>(&reply_parts[idx + 5])?;

                        let mut msg = HashMap::new();
                        msg.insert("header".to_string(), header);
                        msg.insert("content".to_string(), content);
                        msg
                    } else {
                        continue; // Incomplete message, wait for next
                    }
                } else if let Some(first_part) = reply_parts.first() {
                    // Try parsing as simple JSON message for backward compatibility
                    match self.protocol.parse_message(first_part) {
                        Ok(msg) => msg,
                        Err(_) => continue, // Not a valid message, wait for next
                    }
                } else {
                    continue; // No parts, wait for next
                };

                // Check if this is a model_reply
                if let Some(header) = reply_msg.get("header") {
                    if let Some(msg_type) = header.get("msg_type") {
                        if msg_type == "model_reply" {
                            // Extract and return the content's content field
                            if let Some(content_wrapper) = reply_msg.get("content") {
                                // The content contains the actual response nested in a "content" field
                                if let Some(actual_content) = content_wrapper.get("content") {
                                    return Ok(actual_content.clone());
                                }
                                return Ok(content_wrapper.clone());
                            }
                        }
                    }
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
    }

    /// Get the kernel ID
    pub fn kernel_id(&self) -> &str {
        &self.kernel_id
    }

    /// Get the transport for client connections
    pub fn transport(&self) -> Arc<InProcessTransport> {
        self.transport.clone()
    }

    /// Convert handle into the underlying kernel
    pub fn into_kernel(self) -> IntegratedKernel<JupyterProtocol> {
        self.kernel
    }
}

/// Handle for a client connection to a kernel
pub struct ClientHandle {
    protocol: JupyterProtocol,
    connection_string: String,
    transport: Box<dyn Transport>,
}

impl ClientHandle {
    /// Execute code on the remote kernel
    ///
    /// # Errors
    ///
    /// Returns an error if the execution fails or communication with kernel fails
    pub async fn execute(&mut self, code: &str) -> Result<String> {
        debug!("Sending execute request to kernel");

        // Create execute_request message
        let content = serde_json::json!({
            "code": code,
            "silent": false,
            "store_history": true,
            "user_expressions": {},
            "allow_stdin": false,
        });

        let request = self.protocol.create_request("execute_request", content)?;

        // Send request through transport
        self.transport.send("shell", vec![request]).await?;

        // Wait for execute_reply
        loop {
            if let Some(reply_parts) = self.transport.recv("shell").await? {
                if let Some(first_part) = reply_parts.first() {
                    // Parse reply and extract result
                    let reply_msg = self.protocol.parse_message(first_part)?;
                    if let Some(content) = reply_msg.get("content") {
                        // Extract execution result
                        return Ok(format!("Result: {content:?}"));
                    }
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
    }

    /// Send a tool request to the remote kernel and return the response
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or communication with kernel fails
    pub async fn send_tool_request(
        &mut self,
        content: serde_json::Value,
    ) -> Result<serde_json::Value> {
        debug!("Sending tool request to remote kernel");

        // Create tool_request message
        let request = self.protocol.create_request("tool_request", content)?;

        // Send request through transport
        self.transport.send("shell", vec![request]).await?;

        // Wait for tool_reply
        let start_time = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(900);

        loop {
            if start_time.elapsed() > timeout {
                return Err(anyhow::anyhow!("Timeout waiting for tool_reply"));
            }

            if let Some(reply_parts) = self.transport.recv("shell").await? {
                trace!(
                    "Client received {} parts on shell channel",
                    reply_parts.len()
                );

                // Handle multipart Jupyter wire protocol format
                let delimiter = b"<IDS|MSG>";
                let delimiter_idx = reply_parts
                    .iter()
                    .position(|part| part.as_slice() == delimiter);

                let reply_msg: HashMap<String, serde_json::Value> = if let Some(idx) = delimiter_idx
                {
                    // Parse multipart message (header at idx+2, content at idx+5)
                    if reply_parts.len() > idx + 5 {
                        let header =
                            serde_json::from_slice::<serde_json::Value>(&reply_parts[idx + 2])?;
                        let content =
                            serde_json::from_slice::<serde_json::Value>(&reply_parts[idx + 5])?;

                        let mut msg = HashMap::new();
                        msg.insert("header".to_string(), header);
                        msg.insert("content".to_string(), content);
                        msg
                    } else {
                        continue; // Incomplete message, wait for next
                    }
                } else if let Some(first_part) = reply_parts.first() {
                    // Try parsing as simple JSON message for backward compatibility
                    match self.protocol.parse_message(first_part) {
                        Ok(msg) => msg,
                        Err(_) => continue, // Not a valid message, wait for next
                    }
                } else {
                    continue; // No parts, wait for next
                };

                // Check if this is a tool_reply
                if let Some(header) = reply_msg.get("header") {
                    if let Some(msg_type) = header.get("msg_type") {
                        if msg_type == "tool_reply" {
                            // Extract and return the content's content field
                            if let Some(content_wrapper) = reply_msg.get("content") {
                                // The content contains the actual response nested in a "content" field
                                if let Some(actual_content) = content_wrapper.get("content") {
                                    return Ok(actual_content.clone());
                                }
                                return Ok(content_wrapper.clone());
                            }
                        }
                    }
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
    }

    /// Send a template request to the remote kernel and return the response
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or communication with kernel fails
    pub async fn send_template_request(
        &mut self,
        content: serde_json::Value,
    ) -> Result<serde_json::Value> {
        debug!("Sending template request to remote kernel");

        // Create template_request message
        let request = self.protocol.create_request("template_request", content)?;

        // Send request through transport
        self.transport.send("shell", vec![request]).await?;

        // Wait for template_reply
        let start_time = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(900);

        loop {
            if start_time.elapsed() > timeout {
                return Err(anyhow::anyhow!("Timeout waiting for template_reply"));
            }

            if let Some(reply_parts) = self.transport.recv("shell").await? {
                trace!(
                    "Client received {} parts on shell channel",
                    reply_parts.len()
                );

                // Handle multipart Jupyter wire protocol format
                let delimiter = b"<IDS|MSG>";
                let delimiter_idx = reply_parts
                    .iter()
                    .position(|part| part.as_slice() == delimiter);

                let reply_msg: HashMap<String, serde_json::Value> = if let Some(idx) = delimiter_idx
                {
                    // Parse multipart message (header at idx+2, content at idx+5)
                    if reply_parts.len() > idx + 5 {
                        let header =
                            serde_json::from_slice::<serde_json::Value>(&reply_parts[idx + 2])?;
                        let content =
                            serde_json::from_slice::<serde_json::Value>(&reply_parts[idx + 5])?;

                        let mut msg = HashMap::new();
                        msg.insert("header".to_string(), header);
                        msg.insert("content".to_string(), content);
                        msg
                    } else {
                        continue; // Incomplete message, wait for next
                    }
                } else if let Some(first_part) = reply_parts.first() {
                    // Try parsing as simple JSON message for backward compatibility
                    match self.protocol.parse_message(first_part) {
                        Ok(msg) => msg,
                        Err(_) => continue, // Not a valid message, wait for next
                    }
                } else {
                    continue; // No parts, wait for next
                };

                // Check if this is a template_reply
                if let Some(header) = reply_msg.get("header") {
                    if let Some(msg_type) = header.get("msg_type") {
                        if msg_type == "template_reply" {
                            // Extract and return the content's content field
                            if let Some(content_wrapper) = reply_msg.get("content") {
                                // The content contains the actual response nested in a "content" field
                                if let Some(actual_content) = content_wrapper.get("content") {
                                    return Ok(actual_content.clone());
                                }
                                return Ok(content_wrapper.clone());
                            }
                        }
                    }
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
    }

    /// Send a model management request to the remote kernel and return the response
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails or communication with kernel fails
    pub async fn send_model_request(
        &mut self,
        content: serde_json::Value,
    ) -> Result<serde_json::Value> {
        debug!("Sending model request to remote kernel");

        // Create model_request message
        let request = self.protocol.create_request("model_request", content)?;

        // Send request through transport
        self.transport.send("shell", vec![request]).await?;

        // Wait for model_reply
        let start_time = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(900);

        loop {
            if start_time.elapsed() > timeout {
                return Err(anyhow::anyhow!("Timeout waiting for model_reply"));
            }

            if let Some(reply_parts) = self.transport.recv("shell").await? {
                trace!(
                    "Client received {} parts on shell channel",
                    reply_parts.len()
                );

                // Handle multipart Jupyter wire protocol format
                let delimiter = b"<IDS|MSG>";
                let delimiter_idx = reply_parts
                    .iter()
                    .position(|part| part.as_slice() == delimiter);

                let reply_msg: HashMap<String, serde_json::Value> = if let Some(idx) = delimiter_idx
                {
                    // Parse multipart message (header at idx+2, content at idx+5)
                    if reply_parts.len() > idx + 5 {
                        let header =
                            serde_json::from_slice::<serde_json::Value>(&reply_parts[idx + 2])?;
                        let content =
                            serde_json::from_slice::<serde_json::Value>(&reply_parts[idx + 5])?;

                        let mut msg = HashMap::new();
                        msg.insert("header".to_string(), header);
                        msg.insert("content".to_string(), content);
                        msg
                    } else {
                        continue; // Incomplete message, wait for next
                    }
                } else if let Some(first_part) = reply_parts.first() {
                    // Try parsing as simple JSON message for backward compatibility
                    match self.protocol.parse_message(first_part) {
                        Ok(msg) => msg,
                        Err(_) => continue, // Not a valid message, wait for next
                    }
                } else {
                    continue; // No parts, wait for next
                };

                // Check if this is a model_reply
                if let Some(header) = reply_msg.get("header") {
                    if let Some(msg_type) = header.get("msg_type") {
                        if msg_type == "model_reply" {
                            // Extract and return the content's content field
                            if let Some(content_wrapper) = reply_msg.get("content") {
                                // The content contains the actual response nested in a "content" field
                                if let Some(actual_content) = content_wrapper.get("content") {
                                    return Ok(actual_content.clone());
                                }
                                return Ok(content_wrapper.clone());
                            }
                        }
                    }
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
    }

    /// Run interactive REPL connected to the kernel
    ///
    /// # Errors
    ///
    /// Returns an error if the REPL fails to start or connect
    pub fn run_repl(&mut self) -> Result<()> {
        info!("Starting REPL connected to {}", self.connection_string);
        // This would start an interactive REPL session
        // using the transport to communicate with the kernel
        Ok(())
    }
}

/// Handle for a kernel running in service mode
pub struct ServiceHandle {
    kernel: IntegratedKernel<JupyterProtocol>,
    port: u16,
    connection_file: PathBuf,
}

impl ServiceHandle {
    /// Run the kernel service until shutdown
    ///
    /// # Errors
    ///
    /// Returns an error if the kernel service fails to run
    pub async fn run(self) -> Result<()> {
        info!("Running kernel service on port {}", self.port);
        info!("Connection file: {:?}", self.connection_file);

        // Debug: Check if kernel has transport before running
        debug!(
            "ServiceHandle::run() - kernel has transport: {}",
            self.kernel.has_transport()
        );

        self.kernel.run().await
    }

    /// Get the connection file path
    pub fn connection_file(&self) -> &Path {
        &self.connection_file
    }
}

/// Start an embedded kernel with a custom script executor
///
/// This is used when the caller wants to provide a specific script executor
/// implementation, such as a real `ScriptRuntime` from llmspell-bridge.
///
/// # Errors
///
/// Returns an error if the kernel fails to start or transport setup fails
pub async fn start_embedded_kernel_with_executor(
    config: LLMSpellConfig,
    script_executor: Arc<dyn ScriptExecutor>,
) -> Result<KernelHandle> {
    start_embedded_kernel_with_executor_and_provider(config, script_executor, None).await
}

/// Create and initialize a provider manager from config (Phase 11.FIX.1)
///
/// This creates a `ProviderManager`, registers all provider factories (ollama, candle, rig),
/// and initializes provider instances from the configuration.
///
/// # Errors
///
/// Returns an error if provider initialization fails
pub async fn create_provider_manager(
    config: &LLMSpellConfig,
) -> Result<Arc<llmspell_providers::ProviderManager>> {
    let pm = Arc::new(llmspell_providers::ProviderManager::new());

    // Register all provider factories
    pm.register_provider("ollama", llmspell_providers::create_ollama_provider)
        .await;
    pm.register_provider("candle", llmspell_providers::create_candle_provider)
        .await;
    pm.register_provider("rig", llmspell_providers::create_rig_provider)
        .await;
    debug!("Registered provider factories: ollama, candle, rig");

    // Initialize provider instances from configuration
    debug!(
        "Initializing providers from config (found {} provider configs)",
        config.providers.providers.len()
    );
    for (name, config_provider) in &config.providers.providers {
        if !config_provider.enabled {
            debug!("Skipping disabled provider: {}", name);
            continue;
        }

        let provider_config = llmspell_providers::ProviderConfig {
            name: name.clone(),
            provider_type: config_provider.provider_type.clone(),
            model: config_provider.default_model.clone().unwrap_or_default(),
            endpoint: config_provider.base_url.clone(),
            api_key: config_provider.api_key.clone().or_else(|| {
                config_provider
                    .api_key_env
                    .as_ref()
                    .and_then(|env| std::env::var(env).ok())
            }),
            timeout_secs: config_provider.timeout_seconds,
            max_retries: config_provider.max_retries,
            custom_config: config_provider.options.clone(),
        };

        match pm.init_provider(provider_config).await {
            Ok(()) => {
                info!(
                    "Initialized provider: {} (type: {})",
                    name, config_provider.provider_type
                );
            }
            Err(e) => {
                warn!("Failed to initialize provider {}: {}", name, e);
            }
        }
    }

    Ok(pm)
}

/// Start embedded kernel with script executor and optional provider manager (Phase 11.FIX.1)
///
/// If `provider_manager` is provided, it will be used. Otherwise, a new one will be created.
/// This enables sharing a single `ProviderManager` between kernel and script runtime.
///
/// # Errors
///
/// Returns an error if the kernel fails to start or transport setup fails
pub async fn start_embedded_kernel_with_executor_and_provider(
    config: LLMSpellConfig,
    script_executor: Arc<dyn ScriptExecutor>,
    provider_manager: Option<Arc<llmspell_providers::ProviderManager>>,
) -> Result<KernelHandle> {
    let kernel_id = format!("embedded-{}", Uuid::new_v4());
    let session_id = format!("session-{}", Uuid::new_v4());

    info!("Starting embedded kernel {}", kernel_id);
    debug!("start_embedded_kernel_with_executor called");

    // Create Jupyter protocol
    let protocol = JupyterProtocol::new(session_id.clone(), kernel_id.clone());

    // Create bidirectional in-process transport pair
    // Important: We must use create_pair() to ensure transports can communicate
    let (mut kernel_transport, mut client_transport) = InProcessTransport::create_pair();

    trace!(
        "Created transport pair - kernel: {:p}, client: {:p}",
        &raw const kernel_transport,
        &raw const client_transport
    );

    // Setup Jupyter 5-channel configuration
    let mut transport_config = TransportConfig {
        transport_type: "inprocess".to_string(),
        base_address: String::new(),
        channels: HashMap::new(),
        auth_key: None,
    };

    // Setup required Jupyter channels
    for channel in &["shell", "iopub", "stdin", "control", "heartbeat"] {
        transport_config.channels.insert(
            (*channel).to_string(),
            ChannelConfig {
                endpoint: String::new(),
                pattern: String::new(),
                options: HashMap::new(),
            },
        );
    }

    // Setup paired channels for bidirectional communication
    // This is crucial - we MUST set up the channels BEFORE passing the transports
    for channel_name in transport_config.channels.keys() {
        InProcessTransport::setup_paired_channel(
            &mut kernel_transport,
            &mut client_transport,
            channel_name,
        );
        trace!("Setup paired channel: {}", channel_name);
    }

    // Build execution config from LLMSpellConfig
    let exec_config = build_execution_config(&config);

    // Use provided provider manager or create new one (Phase 11.FIX.1)
    let provider_manager = if let Some(pm) = provider_manager {
        debug!("Using provided provider manager (shared with script runtime)");
        pm
    } else {
        debug!("Creating new provider manager for kernel");
        create_provider_manager(&config).await?
    };

    // Use the provided script executor (clone it for sharing between kernels)
    let script_executor_clone = script_executor.clone();
    let provider_manager_clone = provider_manager.clone();

    // Create integrated kernel with the provided executor
    let mut kernel = IntegratedKernel::new(
        protocol.clone(),
        exec_config.clone(),
        session_id.clone(),
        script_executor,
        Some(provider_manager),
    )
    .await?;

    // Set kernel transport for kernel message processing
    kernel.set_transport(Box::new(kernel_transport));

    // Spawn the kernel to run in background and process messages
    let kernel_id_clone = kernel_id.clone();
    tokio::spawn(async move {
        debug!("Starting embedded kernel {} event loop", kernel_id_clone);
        if let Err(e) = kernel.run().await {
            error!(
                "Embedded kernel {} event loop failed: {}",
                kernel_id_clone, e
            );
        } else {
            debug!("Embedded kernel {} event loop completed", kernel_id_clone);
        }
    });

    // For embedded mode, create a minimal kernel handle that only contains what's needed for message sending
    // The actual kernel is running in the background spawn
    let dummy_kernel = IntegratedKernel::new(
        protocol.clone(),
        exec_config.clone(),
        format!("dummy-{session_id}"),
        script_executor_clone,
        Some(provider_manager_clone),
    )
    .await?;

    let transport_arc = Arc::new(client_transport);

    let handle = KernelHandle {
        kernel: dummy_kernel, // This won't be used for embedded mode - only transport and protocol matter
        kernel_id: kernel_id.clone(),
        transport: transport_arc, // CLI uses client transport
        protocol,
    };
    debug!("Created KernelHandle with kernel_id: {}", kernel_id);
    Ok(handle)
}

// Shared stub tool implementation for testing
struct StubTool {
    meta: llmspell_core::types::ComponentMetadata,
    category: llmspell_core::traits::tool::ToolCategory,
    security: llmspell_core::traits::tool::SecurityLevel,
}

impl StubTool {
    fn new(
        name: &str,
        description: &str,
        category: llmspell_core::traits::tool::ToolCategory,
    ) -> Self {
        Self {
            meta: llmspell_core::types::ComponentMetadata {
                id: llmspell_core::types::ComponentId::from_name(name),
                name: name.to_string(),
                description: description.to_string(),
                version: llmspell_core::types::Version::new(1, 0, 0),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            category,
            security: llmspell_core::traits::tool::SecurityLevel::Safe,
        }
    }
}

#[async_trait]
impl llmspell_core::traits::base_agent::BaseAgent for StubTool {
    fn metadata(&self) -> &llmspell_core::types::ComponentMetadata {
        &self.meta
    }

    async fn execute_impl(
        &self,
        input: llmspell_core::types::AgentInput,
        _context: llmspell_core::ExecutionContext,
    ) -> llmspell_core::Result<llmspell_core::types::AgentOutput> {
        // Simple stub implementation that returns the tool name and input
        Ok(llmspell_core::types::AgentOutput::text(format!(
            "Executed {} with input: {}",
            self.meta.name, input.text
        )))
    }

    async fn validate_input(
        &self,
        _input: &llmspell_core::types::AgentInput,
    ) -> llmspell_core::Result<()> {
        // Stub always accepts any input
        Ok(())
    }

    async fn handle_error(
        &self,
        error: llmspell_core::error::LLMSpellError,
    ) -> llmspell_core::Result<llmspell_core::types::AgentOutput> {
        // Simple error handling
        Ok(llmspell_core::types::AgentOutput::text(format!(
            "Error in {}: {}",
            self.meta.name, error
        )))
    }
}

impl Tool for StubTool {
    fn category(&self) -> llmspell_core::traits::tool::ToolCategory {
        self.category.clone()
    }

    fn security_level(&self) -> llmspell_core::traits::tool::SecurityLevel {
        self.security.clone()
    }

    fn schema(&self) -> llmspell_core::traits::tool::ToolSchema {
        // Simple schema for stubs
        llmspell_core::traits::tool::ToolSchema {
            name: self.meta.name.clone(),
            description: self.meta.description.clone(),
            parameters: vec![],
            returns: Some(llmspell_core::traits::tool::ParameterType::String),
        }
    }
}

/// Start an embedded kernel that runs in-process
///
/// This is used when the CLI runs without --connect flag.
/// The kernel runs in the same process as the CLI.
///
/// # Errors
///
/// Returns an error if the kernel fails to start or transport setup fails
#[allow(clippy::too_many_lines)]
pub async fn start_embedded_kernel(config: LLMSpellConfig) -> Result<KernelHandle> {
    // Create a default executor with stub component registry for backward compatibility
    struct DefaultExecutor {
        component_registry: Arc<DefaultStubComponentRegistry>,
    }

    // Reuse stub component registry for embedded mode
    struct DefaultStubComponentRegistry;

    #[async_trait]
    impl ComponentLookup for DefaultStubComponentRegistry {
        async fn list_tools(&self) -> Vec<String> {
            // Return all placeholder tools for embedded testing
            vec![
                "calculator",
                "file-operations",
                "web-scraper",
                "json-processor",
                "text-analyzer",
                "data-converter",
                "image-processor",
                "api-tester",
                "base64-encoder",
                "citation-formatter",
                "sitemap-crawler",
                "graphql-query",
                "url-analyzer",
                "http-requester",
                "video-processor",
                "webpage-monitor",
                "service-checker",
                "environment-reader",
                "uuid-generator",
                "hash-calculator",
                "data-validator",
                "web-searcher",
                "webhook-caller",
                "process-executor",
                "datetime-handler",
                "file-search",
                "file-watcher",
                "audio-processor",
                "diff-calculator",
                "system-monitor",
                "graph-builder",
                "text-manipulator",
                "file-converter",
            ]
            .into_iter()
            .map(String::from)
            .collect()
        }

        #[allow(clippy::too_many_lines)]
        async fn get_tool(&self, name: &str) -> Option<Arc<dyn Tool>> {
            // Return actual stub tools for embedded testing
            use llmspell_core::traits::tool::ToolCategory;

            let tool = match name {
                "calculator" => StubTool::new(
                    "calculator",
                    "Perform mathematical calculations",
                    ToolCategory::Utility,
                ),
                "file-operations" => StubTool::new(
                    "file-operations",
                    "Read and write files",
                    ToolCategory::Filesystem,
                ),
                "web-scraper" => {
                    StubTool::new("web-scraper", "Scrape web pages", ToolCategory::Web)
                }
                "json-processor" => {
                    StubTool::new("json-processor", "Process JSON data", ToolCategory::Data)
                }
                "text-analyzer" => StubTool::new(
                    "text-analyzer",
                    "Analyze text content",
                    ToolCategory::Analysis,
                ),
                "data-converter" => StubTool::new(
                    "data-converter",
                    "Convert between data formats",
                    ToolCategory::Data,
                ),
                "image-processor" => {
                    StubTool::new("image-processor", "Process images", ToolCategory::Media)
                }
                "api-tester" => {
                    StubTool::new("api-tester", "Test API endpoints", ToolCategory::Api)
                }
                "base64-encoder" => StubTool::new(
                    "base64-encoder",
                    "Encode/decode base64",
                    ToolCategory::Utility,
                ),
                "citation-formatter" => StubTool::new(
                    "citation-formatter",
                    "Format citations",
                    ToolCategory::Utility,
                ),
                "sitemap-crawler" => StubTool::new(
                    "sitemap-crawler",
                    "Crawl website sitemaps",
                    ToolCategory::Web,
                ),
                "graphql-query" => StubTool::new(
                    "graphql-query",
                    "Execute GraphQL queries",
                    ToolCategory::Api,
                ),
                "url-analyzer" => {
                    StubTool::new("url-analyzer", "Analyze URL components", ToolCategory::Web)
                }
                "http-requester" => {
                    StubTool::new("http-requester", "Make HTTP requests", ToolCategory::Web)
                }
                "video-processor" => StubTool::new(
                    "video-processor",
                    "Process video files",
                    ToolCategory::Media,
                ),
                "webpage-monitor" => StubTool::new(
                    "webpage-monitor",
                    "Monitor webpage changes",
                    ToolCategory::Web,
                ),
                "service-checker" => StubTool::new(
                    "service-checker",
                    "Check service status",
                    ToolCategory::System,
                ),
                "environment-reader" => StubTool::new(
                    "environment-reader",
                    "Read environment variables",
                    ToolCategory::System,
                ),
                "uuid-generator" => {
                    StubTool::new("uuid-generator", "Generate UUIDs", ToolCategory::Utility)
                }
                "hash-calculator" => {
                    StubTool::new("hash-calculator", "Calculate hashes", ToolCategory::Utility)
                }
                "data-validator" => {
                    StubTool::new("data-validator", "Validate data", ToolCategory::Data)
                }
                "web-searcher" => {
                    StubTool::new("web-searcher", "Search the web", ToolCategory::Web)
                }
                "webhook-caller" => {
                    StubTool::new("webhook-caller", "Call webhooks", ToolCategory::Api)
                }
                "process-executor" => StubTool::new(
                    "process-executor",
                    "Execute processes",
                    ToolCategory::System,
                ),
                "datetime-handler" => StubTool::new(
                    "datetime-handler",
                    "Handle dates and times",
                    ToolCategory::Utility,
                ),
                "file-search" => {
                    StubTool::new("file-search", "Search files", ToolCategory::Filesystem)
                }
                "file-watcher" => StubTool::new(
                    "file-watcher",
                    "Watch file changes",
                    ToolCategory::Filesystem,
                ),
                "audio-processor" => StubTool::new(
                    "audio-processor",
                    "Process audio files",
                    ToolCategory::Media,
                ),
                "diff-calculator" => StubTool::new(
                    "diff-calculator",
                    "Calculate differences",
                    ToolCategory::Analysis,
                ),
                "system-monitor" => StubTool::new(
                    "system-monitor",
                    "Monitor system resources",
                    ToolCategory::System,
                ),
                "graph-builder" => {
                    StubTool::new("graph-builder", "Build graphs", ToolCategory::Data)
                }
                "text-manipulator" => {
                    StubTool::new("text-manipulator", "Manipulate text", ToolCategory::Utility)
                }
                "file-converter" => {
                    StubTool::new("file-converter", "Convert file formats", ToolCategory::Data)
                }
                _ => return None,
            };
            Some(Arc::new(tool))
        }

        async fn list_agents(&self) -> Vec<String> {
            vec![]
        }

        async fn get_agent(&self, _name: &str) -> Option<Arc<dyn Agent>> {
            None
        }

        async fn list_workflows(&self) -> Vec<String> {
            vec![]
        }

        async fn get_workflow(&self, _name: &str) -> Option<Arc<dyn Workflow>> {
            None
        }
    }

    #[async_trait]
    impl ScriptExecutor for DefaultExecutor {
        async fn execute_script(
            &self,
            _script: &str,
        ) -> Result<ScriptExecutionOutput, llmspell_core::error::LLMSpellError> {
            Ok(ScriptExecutionOutput {
                output: serde_json::json!(
                    "Default executor - use start_embedded_kernel_with_executor for real execution"
                ),
                console_output: vec![],
                metadata: ScriptExecutionMetadata {
                    duration: std::time::Duration::from_millis(0),
                    language: "lua".to_string(),
                    exit_code: Some(0),
                    warnings: vec![],
                },
            })
        }

        fn language(&self) -> &'static str {
            "lua"
        }

        fn component_registry(&self) -> Option<Arc<dyn ComponentLookup>> {
            Some(self.component_registry.clone())
        }
    }

    let script_executor = Arc::new(DefaultExecutor {
        component_registry: Arc::new(DefaultStubComponentRegistry),
    }) as Arc<dyn ScriptExecutor>;
    start_embedded_kernel_with_executor(config, script_executor).await
}

/// Connect to an existing kernel service as a client
///
/// This is used when the CLI runs with --connect flag.
/// The CLI acts as a Jupyter client connecting to a remote kernel.
///
/// # Errors
///
/// Returns an error if connection fails or the connection string is invalid
pub async fn connect_to_kernel(connection_string: &str) -> Result<ClientHandle> {
    info!("Connecting to kernel at: {}", connection_string);

    // Create client protocol
    let protocol = JupyterProtocol::new_client();

    // Parse connection string to determine transport type
    let mut transport: Box<dyn Transport>;
    let mut transport_config = TransportConfig {
        transport_type: String::new(),
        base_address: String::new(),
        channels: HashMap::new(),
        auth_key: None,
    };

    if connection_string.starts_with("tcp://") {
        // TCP connection: tcp://host:port
        let addr = connection_string.trim_start_matches("tcp://");
        let parts: Vec<&str> = addr.split(':').collect();

        if parts.len() != 2 {
            return Err(anyhow::anyhow!(
                "Invalid TCP connection string: {}",
                connection_string
            ));
        }

        transport_config.transport_type = "zeromq".to_string();
        transport_config.base_address = parts[0].to_string();

        let base_port: u16 = parts[1]
            .parse()
            .map_err(|_| anyhow::anyhow!("Invalid port: {}", parts[1]))?;

        // Setup 5 Jupyter channels with sequential ports
        transport_config.channels.insert(
            "shell".to_string(),
            ChannelConfig {
                endpoint: base_port.to_string(),
                pattern: "dealer".to_string(), // Client uses dealer
                options: HashMap::new(),
            },
        );
        transport_config.channels.insert(
            "iopub".to_string(),
            ChannelConfig {
                endpoint: (base_port + 1).to_string(),
                pattern: "sub".to_string(), // Client subscribes
                options: HashMap::new(),
            },
        );
        transport_config.channels.insert(
            "stdin".to_string(),
            ChannelConfig {
                endpoint: (base_port + 2).to_string(),
                pattern: "dealer".to_string(),
                options: HashMap::new(),
            },
        );
        transport_config.channels.insert(
            "control".to_string(),
            ChannelConfig {
                endpoint: (base_port + 3).to_string(),
                pattern: "dealer".to_string(),
                options: HashMap::new(),
            },
        );
        transport_config.channels.insert(
            "heartbeat".to_string(),
            ChannelConfig {
                endpoint: (base_port + 4).to_string(),
                pattern: "req".to_string(), // Client requests heartbeat
                options: HashMap::new(),
            },
        );

        transport = crate::traits::create_transport("zeromq")?;
    } else if std::path::Path::new(connection_string)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
    {
        // Connection file
        return Err(anyhow::anyhow!(
            "Connection file support not yet implemented. Use tcp:// format"
        ));
    } else {
        // Named kernel (e.g., "my-kernel")
        // This would look up the kernel in a registry
        return Err(anyhow::anyhow!(
            "Named kernel connections not yet implemented. Use tcp:// format"
        ));
    }

    // Connect the transport
    transport.connect(&transport_config).await?;

    Ok(ClientHandle {
        protocol,
        connection_string: connection_string.to_string(),
        transport,
    })
}

/// Start a kernel in service mode with full configuration
///
/// This is the enhanced version that accepts `ExecutionConfig` with daemon settings.
///
/// # Errors
///
/// Returns an error if the kernel service fails to start or bind to the port
pub async fn start_kernel_service_with_config(
    config: KernelServiceConfig,
) -> Result<ServiceHandle> {
    let kernel_id = config
        .kernel_id
        .unwrap_or_else(|| format!("service-{}", Uuid::new_v4()));
    let session_id = format!("session-{}", Uuid::new_v4());

    info!(
        "Starting kernel service {} on port {}",
        kernel_id, config.port
    );

    // Create Jupyter protocol
    let mut protocol = JupyterProtocol::new(session_id.clone(), kernel_id.clone());

    // Create ConnectionFileManager early to get the HMAC key
    let mut conn_manager =
        crate::connection::ConnectionFileManager::new(kernel_id.clone(), config.port);

    // Set the HMAC key on the protocol from the connection info
    protocol.set_hmac_key(&conn_manager.info().key);

    // Create and bind transport, updating connection manager with actual ports
    info!("About to setup kernel transport on port {}", config.port);
    let transport = setup_kernel_transport(config.port, &mut conn_manager).await?;
    info!("Transport setup complete");

    // Create integrated kernel with protocol that has the HMAC key
    let mut kernel = IntegratedKernel::new(
        protocol.clone(),
        config.exec_config.clone(),
        session_id,
        config.script_executor,
        None,
    )
    .await?;

    // Set the transport on the kernel
    kernel.set_transport(Box::new(transport));

    // Debug: Verify transport was set
    debug!("Transport set on kernel: {}", kernel.has_transport());

    // Note: Daemon mode is now handled by the CLI before creating tokio runtime
    // The daemonization happens BEFORE this async function is called
    // The PID file is already created by DaemonManager::daemonize()
    // We just need to handle log rotation if configured
    if config.exec_config.daemon_mode {
        if let Some(ref daemon_config) = config.exec_config.daemon_config {
            // Set up log rotation if configured
            if let Some(ref log_path) = daemon_config.stdout_path {
                if let Some(size_limit) = config.log_rotate_size {
                    let log_config = crate::daemon::LogRotationConfig {
                        max_size: size_limit,
                        max_files: config.log_rotate_count,
                        compress: true,
                        base_path: log_path.clone(),
                    };
                    let _log_rotator = crate::daemon::LogRotator::new(log_config);
                    info!(
                        "Log rotation configured: max size {} bytes, keeping {} files",
                        size_limit, config.log_rotate_count
                    );
                }
            }

            // PID file is already created by DaemonManager::daemonize() in main.rs
            // Do NOT create it again here as it causes "Another instance is already running" error
            if let Some(ref pid_path) = daemon_config.pid_file {
                debug!(
                    "PID file already created at {:?} by DaemonManager",
                    pid_path
                );
            }
        }
    }

    // Write the connection file
    let connection_file = if let Some(path) = config.connection_file_path {
        // Use specified path
        std::fs::write(&path, serde_json::to_string_pretty(conn_manager.info())?)?;
        path
    } else {
        // Use default path
        conn_manager.write()?
    };

    info!("Connection file written to: {:?}", connection_file);

    Ok(ServiceHandle {
        kernel,
        port: config.port,
        connection_file,
    })
}

/// Helper function to set up kernel transport with `ZeroMQ`
///
/// Creates a `ZeroMQ` transport, binds to the specified ports,
/// and updates the connection manager with actual bound ports.
async fn setup_kernel_transport(
    base_port: u16,
    conn_manager: &mut crate::connection::ConnectionFileManager,
) -> Result<crate::transport::zeromq::ZmqTransport> {
    info!(
        "setup_kernel_transport: Creating ZeroMQ transport for port {}",
        base_port
    );

    // Create ZeroMQ transport for the kernel service
    let mut transport = crate::transport::zeromq::ZmqTransport::new()?;
    info!("setup_kernel_transport: ZeroMQ transport created");

    // Build transport configuration for Jupyter 5 channels
    // Special handling for port 0 - let OS assign all ports independently
    let transport_config = TransportConfig {
        transport_type: "tcp".to_string(),
        base_address: "127.0.0.1".to_string(),
        channels: {
            let mut channels = HashMap::new();

            // When base_port is 0, use 0 for all channels to let OS assign each independently
            // Otherwise use sequential ports starting from base_port
            let (shell_port, iopub_port, stdin_port, control_port, hb_port) = if base_port == 0 {
                (
                    "0".to_string(),
                    "0".to_string(),
                    "0".to_string(),
                    "0".to_string(),
                    "0".to_string(),
                )
            } else {
                (
                    base_port.to_string(),
                    (base_port + 1).to_string(),
                    (base_port + 2).to_string(),
                    (base_port + 3).to_string(),
                    (base_port + 4).to_string(),
                )
            };

            channels.insert(
                "shell".to_string(),
                ChannelConfig {
                    pattern: "router".to_string(),
                    endpoint: shell_port,
                    options: HashMap::new(),
                },
            );
            channels.insert(
                "iopub".to_string(),
                ChannelConfig {
                    pattern: "pub".to_string(),
                    endpoint: iopub_port,
                    options: HashMap::new(),
                },
            );
            channels.insert(
                "stdin".to_string(),
                ChannelConfig {
                    pattern: "router".to_string(),
                    endpoint: stdin_port,
                    options: HashMap::new(),
                },
            );
            channels.insert(
                "control".to_string(),
                ChannelConfig {
                    pattern: "router".to_string(),
                    endpoint: control_port,
                    options: HashMap::new(),
                },
            );
            channels.insert(
                "heartbeat".to_string(),
                ChannelConfig {
                    pattern: "rep".to_string(),
                    endpoint: hb_port,
                    options: HashMap::new(),
                },
            );
            channels
        },
        auth_key: None,
    };

    info!(
        "setup_kernel_transport: About to bind transport with config for {} channels",
        transport_config.channels.len()
    );

    // Bind transport and get actual ports (important when port 0 is used)
    let bound_ports = transport.bind(&transport_config).await?;

    info!(
        "setup_kernel_transport: Binding complete, got bound_ports: {:?}",
        bound_ports
    );

    // Update connection manager with actual bound ports
    if let Some(ports) = bound_ports {
        conn_manager.update_ports(
            ports.shell,
            ports.iopub,
            ports.stdin,
            ports.control,
            ports.hb,
        );
        info!(
            "Kernel bound to actual ports - shell: {}, iopub: {}, stdin: {}, control: {}, hb: {}",
            ports.shell, ports.iopub, ports.stdin, ports.control, ports.hb
        );
    }

    Ok(transport)
}

/// Start a kernel in service mode that listens for connections (legacy)
///
/// This is used when starting a kernel as a service that other clients can connect to.
///
/// # Errors
///
/// Returns an error if the kernel service fails to start or bind to the port
#[allow(clippy::too_many_lines)]
pub async fn start_kernel_service(port: u16, config: LLMSpellConfig) -> Result<ServiceHandle> {
    // TODO: In subtask 9.4.6.4, this will be replaced with real ScriptRuntime from llmspell-bridge
    // For now, create a stub executor with minimal ComponentRegistry
    struct ServiceStubExecutor {
        component_registry: Arc<StubComponentRegistry>,
    }

    // Stub ComponentRegistry with actual tool implementations for testing
    struct StubComponentRegistry;

    #[async_trait]
    impl ComponentLookup for StubComponentRegistry {
        async fn list_tools(&self) -> Vec<String> {
            // Return a reasonable set of stub tools for testing
            vec![
                "calculator".to_string(),
                "file-operations".to_string(),
                "web-scraper".to_string(),
                "json-processor".to_string(),
                "text-analyzer".to_string(),
                "data-converter".to_string(),
                "system-monitor".to_string(),
                "environment-reader".to_string(),
            ]
        }

        async fn get_tool(&self, name: &str) -> Option<Arc<dyn Tool>> {
            // Return actual stub tools for testing
            use llmspell_core::traits::tool::ToolCategory;

            let tool = match name {
                "calculator" => StubTool::new(
                    "calculator",
                    "Perform mathematical calculations",
                    ToolCategory::Utility,
                ),
                "file-operations" => StubTool::new(
                    "file-operations",
                    "Read and write files",
                    ToolCategory::Filesystem,
                ),
                "web-scraper" => {
                    StubTool::new("web-scraper", "Scrape web pages", ToolCategory::Web)
                }
                "json-processor" => {
                    StubTool::new("json-processor", "Process JSON data", ToolCategory::Data)
                }
                "text-analyzer" => StubTool::new(
                    "text-analyzer",
                    "Analyze text content",
                    ToolCategory::Analysis,
                ),
                "data-converter" => StubTool::new(
                    "data-converter",
                    "Convert between data formats",
                    ToolCategory::Data,
                ),
                "system-monitor" => StubTool::new(
                    "system-monitor",
                    "Monitor system resources",
                    ToolCategory::System,
                ),
                "environment-reader" => StubTool::new(
                    "environment-reader",
                    "Read environment variables",
                    ToolCategory::System,
                ),
                _ => return None,
            };
            Some(Arc::new(tool))
        }

        async fn list_agents(&self) -> Vec<String> {
            vec![]
        }

        async fn get_agent(&self, _name: &str) -> Option<Arc<dyn Agent>> {
            None
        }

        async fn list_workflows(&self) -> Vec<String> {
            vec![]
        }

        async fn get_workflow(&self, _name: &str) -> Option<Arc<dyn Workflow>> {
            None
        }
    }

    #[async_trait]
    impl ScriptExecutor for ServiceStubExecutor {
        async fn execute_script(
            &self,
            _script: &str,
        ) -> Result<ScriptExecutionOutput, llmspell_core::error::LLMSpellError> {
            Ok(ScriptExecutionOutput {
                output: serde_json::json!("Service stub executor - will be replaced in 9.4.6.4"),
                console_output: vec![],
                metadata: ScriptExecutionMetadata {
                    duration: std::time::Duration::from_millis(0),
                    language: "stub".to_string(),
                    exit_code: Some(0),
                    warnings: vec![],
                },
            })
        }

        fn language(&self) -> &'static str {
            "stub"
        }

        fn component_registry(&self) -> Option<Arc<dyn ComponentLookup>> {
            Some(self.component_registry.clone())
        }
    }

    let kernel_id = format!("service-{}", Uuid::new_v4());
    let session_id = format!("session-{}", Uuid::new_v4());

    info!("Starting kernel service {} on port {}", kernel_id, port);

    // Create Jupyter protocol
    let protocol = JupyterProtocol::new(session_id.clone(), kernel_id.clone());

    // Build execution config
    let exec_config = build_execution_config(&config);

    let script_executor = Arc::new(ServiceStubExecutor {
        component_registry: Arc::new(StubComponentRegistry),
    }) as Arc<dyn ScriptExecutor>;

    // Create integrated kernel
    let kernel =
        IntegratedKernel::new(protocol, exec_config, session_id, script_executor, None).await?;
    // Note: Service kernels don't need transport set here as they use external connections

    // Write connection file for clients
    let connection_file = write_connection_file(port, &kernel_id)?;

    Ok(ServiceHandle {
        kernel,
        port,
        connection_file,
    })
}

/// Build `ExecutionConfig` from `LLMSpellConfig`
fn build_execution_config(config: &LLMSpellConfig) -> crate::execution::ExecutionConfig {
    crate::execution::ExecutionConfig {
        runtime_config: serde_json::to_value(config)
            .ok()
            .and_then(|v| v.as_object().cloned())
            .unwrap_or_default()
            .into_iter()
            .collect(),
        io_config: crate::execution::IOConfig::default(),
        max_history: 1000,
        execution_timeout_secs: 300,
        monitor_agents: true,
        track_performance: true,
        daemon_mode: false,
        daemon_config: None,
        health_thresholds: None,
    }
}

/// Write connection file for kernel service
fn write_connection_file(port: u16, kernel_id: &str) -> Result<PathBuf> {
    let connection_dir = dirs::runtime_dir()
        .or_else(dirs::cache_dir)
        .unwrap_or_else(|| PathBuf::from("/tmp"));

    let connection_file = connection_dir.join(format!("kernel-{kernel_id}.json"));

    let connection_info = serde_json::json!({
        "shell_port": port,
        "iopub_port": port + 1,
        "stdin_port": port + 2,
        "control_port": port + 3,
        "hb_port": port + 4,
        "ip": "127.0.0.1",
        "transport": "tcp",
        "signature_scheme": "hmac-sha256",
        "kernel_name": "llmspell",
    });

    std::fs::write(
        &connection_file,
        serde_json::to_string_pretty(&connection_info)?,
    )?;

    Ok(connection_file)
}
