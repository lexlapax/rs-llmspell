//! Generic client for connecting to kernels
//!
//! This module provides a client implementation that mirrors the kernel architecture,
//! using the same Transport and Protocol traits for consistency.

use anyhow::{Context, Result};
use chrono::Utc;
use uuid::Uuid;

use crate::connection::ConnectionInfo;
use crate::jupyter::protocol::{JupyterMessage, MessageContent, MessageHeader, StreamType};
use crate::traits::transport::ChannelConfig;
use crate::traits::{Protocol, Transport, TransportConfig};

/// Generic client for connecting to kernels
pub struct GenericClient<T: Transport, P: Protocol> {
    /// Transport layer for communication
    transport: T,
    /// Protocol handler
    protocol: P,
    /// Connection information
    connection_info: ConnectionInfo,
    /// Client session ID
    session_id: String,
    /// Username for this client
    username: String,
    /// Execution counter
    execution_count: u32,
}

impl<T: Transport, P: Protocol> GenericClient<T, P> {
    /// Create a new client and connect to a kernel
    ///
    /// # Errors
    ///
    /// Returns an error if connection fails
    pub async fn connect(
        mut transport: T,
        protocol: P,
        connection_info: ConnectionInfo,
    ) -> Result<Self> {
        // Configure transport for client mode (connect instead of bind)
        let config = Self::create_client_config(&connection_info);
        transport
            .connect(&config)
            .await
            .context("Failed to connect to kernel")?;

        Ok(Self {
            transport,
            protocol,
            connection_info,
            session_id: Uuid::new_v4().to_string(),
            username: whoami::username(),
            execution_count: 0,
        })
    }

    /// Create transport configuration for client connection
    fn create_client_config(conn_info: &ConnectionInfo) -> TransportConfig {
        let mut config = TransportConfig {
            transport_type: conn_info.transport.clone(),
            base_address: conn_info.ip.clone(),
            channels: std::collections::HashMap::new(),
        };

        // Configure channels for client mode (opposite patterns from server)
        config.channels.insert(
            "shell".to_string(),
            ChannelConfig {
                pattern: "req".to_string(), // Client uses REQ for request-reply
                endpoint: conn_info.shell_port.to_string(),
            },
        );

        config.channels.insert(
            "iopub".to_string(),
            ChannelConfig {
                pattern: "sub".to_string(), // Client subscribes
                endpoint: conn_info.iopub_port.to_string(),
            },
        );

        config.channels.insert(
            "stdin".to_string(),
            ChannelConfig {
                pattern: "req".to_string(), // Use REQ for request-reply
                endpoint: conn_info.stdin_port.to_string(),
            },
        );

        config.channels.insert(
            "control".to_string(),
            ChannelConfig {
                pattern: "req".to_string(), // Use REQ for request-reply
                endpoint: conn_info.control_port.to_string(),
            },
        );

        config
    }

    /// Get the session ID
    #[must_use]
    pub fn session_id(&self) -> &str {
        &self.session_id
    }

    /// Get the connection info
    #[must_use]
    pub const fn connection_info(&self) -> &ConnectionInfo {
        &self.connection_info
    }
}

/// Jupyter-specific client implementation
impl GenericClient<crate::transport::ZmqTransport, crate::jupyter::JupyterProtocol> {
    /// Execute code on the kernel
    ///
    /// # Errors
    ///
    /// Returns an error if execution fails
    pub async fn execute(&mut self, code: &str) -> Result<MessageContent> {
        self.execute_with_args(code, vec![]).await
    }

    /// Process `IOPub` message for our execution
    fn process_iopub_message(msg: &JupyterMessage, msg_id: &str) {
        // Check if this message is for our execution
        let is_our_message = msg
            .parent_header
            .as_ref()
            .is_some_and(|parent| parent.msg_id == msg_id);

        if is_our_message {
            match &msg.content {
                MessageContent::Status { execution_state } => {
                    tracing::debug!("Execution state: {:?}", execution_state);
                }
                MessageContent::Stream { name, text } => match name {
                    StreamType::Stdout => print!("{text}"),
                    StreamType::Stderr => eprint!("{text}"),
                },
                MessageContent::ExecuteResult { data, .. } => {
                    if let Some(text_plain) = data.get("text/plain") {
                        if let Some(text) = text_plain.as_str() {
                            print!("{text}");
                        }
                    }
                }
                MessageContent::Error {
                    ename,
                    evalue,
                    traceback,
                } => {
                    eprintln!("{ename}: {evalue}");
                    for line in traceback {
                        eprintln!("{line}");
                    }
                }
                _ => {} // Ignore other IOPub messages
            }
        }
    }

    /// Create an execute request message
    fn create_execute_request(&self, code: &str, args: Vec<String>) -> JupyterMessage {
        let mut metadata = serde_json::Map::new();
        metadata.insert("__identities".to_string(), serde_json::json!([""]));

        JupyterMessage {
            header: MessageHeader {
                msg_id: Uuid::new_v4().to_string(),
                msg_type: "execute_request".to_string(),
                username: self.username.clone(),
                session: self.session_id.clone(),
                date: Utc::now(),
                version: "5.3".to_string(),
            },
            parent_header: None,
            metadata: serde_json::Value::Object(metadata),
            content: MessageContent::ExecuteRequest {
                code: code.to_string(),
                silent: false,
                store_history: Some(true),
                user_expressions: None,
                allow_stdin: Some(false),
                stop_on_error: Some(true),
                script_args: if args.is_empty() { None } else { Some(args) },
            },
        }
    }

    /// Process `IOPub` messages during execution
    async fn process_iopub_during_execution(&self, msg_id: &str) -> Result<()> {
        if let Some(iopub_bytes) = self.transport.recv("iopub").await? {
            match self.protocol.decode(iopub_bytes, "iopub") {
                Ok(iopub_msg) => {
                    Self::process_iopub_message(&iopub_msg, msg_id);
                }
                Err(e) => {
                    tracing::debug!("Failed to decode IOPub message: {}", e);
                }
            }
        }
        Ok(())
    }

    /// Check for execute reply on shell channel
    async fn check_for_execute_reply(&self, msg_id: &str) -> Result<Option<MessageContent>> {
        if let Some(reply_bytes) = self.transport.recv("shell").await? {
            let reply = self.protocol.decode(reply_bytes, "shell")?;
            tracing::debug!("Received shell message: {}", reply.header.msg_type);

            if reply.header.msg_type == "execute_reply" {
                if let Some(parent) = &reply.parent_header {
                    if parent.msg_id == msg_id {
                        tracing::debug!("Got our execute_reply!");
                        return Ok(Some(reply.content.clone()));
                    }
                }
            }
        }
        Ok(None)
    }

    /// Execute code on the kernel with script arguments
    ///
    /// # Errors
    ///
    /// Returns an error if execution fails
    pub async fn execute_with_args(
        &mut self,
        code: &str,
        args: Vec<String>,
    ) -> Result<MessageContent> {
        self.execution_count += 1;

        // Create and send execute request
        let msg = self.create_execute_request(code, args);
        let msg_id = msg.header.msg_id.clone();
        let request_bytes = self.protocol.encode(&msg, "shell")?;
        self.transport.send("shell", request_bytes).await?;

        // Wait for execution to complete
        let timeout_duration = tokio::time::Duration::from_secs(2);
        let start_time = tokio::time::Instant::now();

        loop {
            // Check for timeout
            if start_time.elapsed() > timeout_duration {
                tracing::warn!("Execution timed out waiting for completion");
                break;
            }

            // Process IOPub messages
            self.process_iopub_during_execution(&msg_id).await?;

            // Check for execute reply
            if let Some(reply) = self.check_for_execute_reply(&msg_id).await? {
                return Ok(reply);
            }

            // Small delay to avoid busy waiting
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        Err(anyhow::anyhow!("No execute reply received"))
    }

    /// Request kernel information
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails
    pub async fn kernel_info(&mut self) -> Result<MessageContent> {
        // Create kernel info request with empty identity
        let mut metadata = serde_json::Map::new();
        metadata.insert("__identities".to_string(), serde_json::json!([""]));

        let msg = JupyterMessage {
            header: MessageHeader {
                msg_id: Uuid::new_v4().to_string(),
                msg_type: "kernel_info_request".to_string(),
                username: self.username.clone(),
                session: self.session_id.clone(),
                date: Utc::now(),
                version: "5.3".to_string(),
            },
            parent_header: None,
            metadata: serde_json::Value::Object(metadata),
            content: MessageContent::KernelInfoRequest {},
        };

        // Send request on shell channel
        let request_bytes = self.protocol.encode(&msg, "shell")?;
        self.transport.send("shell", request_bytes).await?;

        // Wait for reply
        loop {
            if let Some(reply_bytes) = self.transport.recv("shell").await? {
                let reply = self.protocol.decode(reply_bytes, "shell")?;

                if reply.header.msg_type == "kernel_info_reply" {
                    return Ok(reply.content);
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
    }

    /// Send a debug request to the kernel
    ///
    /// # Errors
    ///
    /// Returns an error if the debug request fails
    pub async fn debug_request(&mut self, command: serde_json::Value) -> Result<MessageContent> {
        // Extract command details from the Value
        let command_str = command
            .get("command")
            .and_then(|c| c.as_str())
            .unwrap_or("unknown")
            .to_string();

        let arguments = command
            .get("arguments")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));

        let seq = command
            .get("seq")
            .and_then(serde_json::Value::as_u64)
            .and_then(|s| u32::try_from(s).ok())
            .unwrap_or(1);

        // Create debug request with empty identity
        let mut metadata = serde_json::Map::new();
        metadata.insert("__identities".to_string(), serde_json::json!([""]));

        let msg = JupyterMessage {
            header: MessageHeader {
                msg_id: Uuid::new_v4().to_string(),
                msg_type: "debug_request".to_string(),
                username: self.username.clone(),
                session: self.session_id.clone(),
                date: Utc::now(),
                version: "5.3".to_string(),
            },
            parent_header: None,
            metadata: serde_json::Value::Object(metadata),
            content: MessageContent::DebugRequest {
                command: command_str,
                arguments,
                seq,
            },
        };

        // Send request on control channel (debug uses control channel)
        let request_bytes = self.protocol.encode(&msg, "control")?;
        self.transport.send("control", request_bytes).await?;

        // Wait for reply
        loop {
            if let Some(reply_bytes) = self.transport.recv("control").await? {
                let reply = self.protocol.decode(reply_bytes, "control")?;

                if reply.header.msg_type == "debug_reply" {
                    return Ok(reply.content);
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
    }

    /// Shutdown the kernel
    ///
    /// # Errors
    ///
    /// Returns an error if shutdown fails
    pub async fn shutdown(&mut self, restart: bool) -> Result<()> {
        // Create shutdown request with empty identity
        let mut metadata = serde_json::Map::new();
        metadata.insert("__identities".to_string(), serde_json::json!([""]));

        let msg = JupyterMessage {
            header: MessageHeader {
                msg_id: Uuid::new_v4().to_string(),
                msg_type: "shutdown_request".to_string(),
                username: self.username.clone(),
                session: self.session_id.clone(),
                date: Utc::now(),
                version: "5.3".to_string(),
            },
            parent_header: None,
            metadata: serde_json::Value::Object(metadata),
            content: MessageContent::ShutdownRequest { restart },
        };

        // Send request on control channel
        let request_bytes = self.protocol.encode(&msg, "control")?;
        self.transport.send("control", request_bytes).await?;

        // Wait for reply
        loop {
            if let Some(reply_bytes) = self.transport.recv("control").await? {
                let reply = self.protocol.decode(reply_bytes, "control")?;

                if reply.header.msg_type == "shutdown_reply" {
                    return Ok(());
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
    }
}

// Type alias for Jupyter client
pub type JupyterClient =
    GenericClient<crate::transport::ZmqTransport, crate::jupyter::JupyterProtocol>;
