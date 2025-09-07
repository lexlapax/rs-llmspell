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

        // Create execute request message with empty identity for REQ socket
        let mut metadata = serde_json::Map::new();
        // Add empty identity for client-initiated messages (REQ socket doesn't need routing)
        metadata.insert("__identities".to_string(), serde_json::json!([""]));

        let msg = JupyterMessage {
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
        };

        // Encode and send request on shell channel
        let request_bytes = self.protocol.encode(&msg, "shell")?;
        self.transport.send("shell", request_bytes).await?;

        let msg_id = msg.header.msg_id.clone();

        // Track execution state
        let mut reply_received = false;
        let mut idle_received = false;
        let mut execute_reply = None;

        // Listen to both shell and iopub channels until execution is complete
        while !reply_received || !idle_received {
            // Check IOPub for output messages
            if let Some(iopub_bytes) = self.transport.recv("iopub").await? {
                let iopub_msg = self.protocol.decode(iopub_bytes, "iopub")?;

                // Only process messages related to our execution
                if let Some(parent) = &iopub_msg.parent_header {
                    if parent.msg_id == msg_id {
                        match &iopub_msg.content {
                            MessageContent::Status { execution_state } => {
                                // Check for idle status to know execution is complete
                                if matches!(
                                    execution_state,
                                    crate::jupyter::protocol::ExecutionState::Idle
                                ) {
                                    idle_received = true;
                                }
                            }
                            MessageContent::Stream { name, text } => {
                                // Print stream output to stdout/stderr
                                match name {
                                    StreamType::Stdout => print!("{text}"),
                                    StreamType::Stderr => eprint!("{text}"),
                                }
                            }
                            MessageContent::ExecuteResult { data, .. } => {
                                // Handle execute results (e.g., returned values)
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
                                // Print errors to stderr
                                eprintln!("{ename}: {evalue}");
                                for line in traceback {
                                    eprintln!("{line}");
                                }
                            }
                            _ => {} // Ignore other IOPub messages
                        }
                    }
                }
            }

            // Check shell channel for execute reply
            if !reply_received {
                if let Some(reply_bytes) = self.transport.recv("shell").await? {
                    let reply = self.protocol.decode(reply_bytes, "shell")?;

                    // Check if this is our execute reply
                    if reply.header.msg_type == "execute_reply" {
                        if let Some(parent) = &reply.parent_header {
                            if parent.msg_id == msg_id {
                                execute_reply = Some(reply.content.clone());
                                reply_received = true;
                            }
                        }
                    }
                }
            }

            // Small delay to avoid busy waiting
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        // Return the execute reply content
        execute_reply.ok_or_else(|| anyhow::anyhow!("No execute reply received"))
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
