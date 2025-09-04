//! ZeroMQ transport implementation for Jupyter protocol
//! 
//! Implements the 5-channel Jupyter messaging system:
//! - Shell: REQ-REP for execute requests
//! - IOPub: PUB for output publishing
//! - Stdin: REQ-REP for input requests  
//! - Control: REQ-REP for control/daemon messages
//! - Heartbeat: REP for keepalive

use anyhow::{Context, Result};
use serde_json::Value;
use zmq::{Context as ZmqContext, Socket, SocketType};

use crate::jupyter::connection::ConnectionInfo;
use crate::jupyter::protocol::JupyterMessage;

/// ZeroMQ transport managing all 5 Jupyter channels
pub struct ZmqTransport {
    context: ZmqContext,
    shell: Socket,      // REQ-REP for execute requests
    iopub: Socket,      // PUB for output publishing
    stdin: Socket,      // REQ-REP for input requests
    control: Socket,    // REQ-REP for control/daemon messages
    heartbeat: Socket,  // REP for heartbeat
    signing_key: String,
}

impl ZmqTransport {
    /// Create and bind ZeroMQ transport with connection info
    pub fn bind(connection_info: &ConnectionInfo) -> Result<Self> {
        let context = ZmqContext::new();
        
        // Create and bind shell socket (ROUTER pattern for Jupyter)
        let shell = context.socket(SocketType::ROUTER)
            .context("Failed to create shell socket")?;
        let shell_addr = format!("{}://{}:{}", 
            connection_info.transport, connection_info.ip, connection_info.shell_port);
        shell.bind(&shell_addr)
            .with_context(|| format!("Failed to bind shell socket to {}", shell_addr))?;

        // Create and bind iopub socket (PUB pattern)
        let iopub = context.socket(SocketType::PUB)
            .context("Failed to create iopub socket")?;
        let iopub_addr = format!("{}://{}:{}", 
            connection_info.transport, connection_info.ip, connection_info.iopub_port);
        iopub.bind(&iopub_addr)
            .with_context(|| format!("Failed to bind iopub socket to {}", iopub_addr))?;

        // Create and bind stdin socket (ROUTER pattern for Jupyter)
        let stdin = context.socket(SocketType::ROUTER)
            .context("Failed to create stdin socket")?;
        let stdin_addr = format!("{}://{}:{}", 
            connection_info.transport, connection_info.ip, connection_info.stdin_port);
        stdin.bind(&stdin_addr)
            .with_context(|| format!("Failed to bind stdin socket to {}", stdin_addr))?;

        // Create and bind control socket (ROUTER pattern for Jupyter)
        let control = context.socket(SocketType::ROUTER)
            .context("Failed to create control socket")?;
        let control_addr = format!("{}://{}:{}", 
            connection_info.transport, connection_info.ip, connection_info.control_port);
        control.bind(&control_addr)
            .with_context(|| format!("Failed to bind control socket to {}", control_addr))?;

        // Create and bind heartbeat socket (REP pattern)
        let heartbeat = context.socket(SocketType::REP)
            .context("Failed to create heartbeat socket")?;
        let heartbeat_addr = format!("{}://{}:{}", 
            connection_info.transport, connection_info.ip, connection_info.hb_port);
        heartbeat.bind(&heartbeat_addr)
            .with_context(|| format!("Failed to bind heartbeat socket to {}", heartbeat_addr))?;

        // Set socket options for non-blocking
        shell.set_rcvtimeo(100).context("Failed to set shell recv timeout")?;
        control.set_rcvtimeo(100).context("Failed to set control recv timeout")?;
        stdin.set_rcvtimeo(100).context("Failed to set stdin recv timeout")?;
        heartbeat.set_rcvtimeo(100).context("Failed to set heartbeat recv timeout")?;

        Ok(Self {
            context,
            shell,
            iopub,
            stdin,
            control,
            heartbeat,
            signing_key: connection_info.key.clone(),
        })
    }

    /// Receive message from shell channel (execute requests)
    pub fn recv_shell_msg(&self) -> Result<Option<JupyterMessage>> {
        self.recv_message(&self.shell, "shell")
    }

    /// Send reply to shell channel
    pub fn send_shell_reply(&self, msg: &JupyterMessage) -> Result<()> {
        self.send_message(&self.shell, msg, "shell")
    }

    /// Receive message from control channel (shutdown, interrupt, daemon requests)
    pub fn recv_control_msg(&self) -> Result<Option<JupyterMessage>> {
        self.recv_message(&self.control, "control")
    }

    /// Send reply to control channel
    pub fn send_control_reply(&self, msg: &JupyterMessage) -> Result<()> {
        self.send_message(&self.control, msg, "control")
    }

    /// Receive message from stdin channel (input requests)
    pub fn recv_stdin_msg(&self) -> Result<Option<JupyterMessage>> {
        self.recv_message(&self.stdin, "stdin")
    }

    /// Send reply to stdin channel
    pub fn send_stdin_reply(&self, msg: &JupyterMessage) -> Result<()> {
        self.send_message(&self.stdin, msg, "stdin")
    }

    /// Publish message to iopub channel (output, status updates)
    pub fn publish_iopub_msg(&self, msg: &JupyterMessage) -> Result<()> {
        self.send_message(&self.iopub, msg, "iopub")
    }

    /// Handle heartbeat (echo back immediately)
    pub fn handle_heartbeat(&self) -> Result<bool> {
        match self.heartbeat.recv_bytes(zmq::DONTWAIT) {
            Ok(data) => {
                // Echo the data back immediately
                self.heartbeat.send(&data, 0)
                    .context("Failed to send heartbeat response")?;
                Ok(true)
            }
            Err(zmq::Error::EAGAIN) => Ok(false), // No heartbeat received
            Err(e) => Err(e).context("Heartbeat receive error"),
        }
    }

    /// Internal: Receive and deserialize message from socket
    fn recv_message(&self, socket: &Socket, channel: &str) -> Result<Option<JupyterMessage>> {
        // Jupyter messages are multi-part: [identities, delimiter, hmac, header, parent_header, metadata, content]
        match socket.recv_multipart(zmq::DONTWAIT) {
            Ok(parts) => {
                tracing::debug!("Received {} parts on {} channel", parts.len(), channel);
                for (i, part) in parts.iter().enumerate() {
                    tracing::trace!("  Part {}: {} bytes, content: {:?}", 
                        i, 
                        part.len(), 
                        String::from_utf8_lossy(&part[..part.len().min(100)])
                    );
                }
                
                if parts.len() < 4 {
                    return Err(anyhow::anyhow!("Invalid message format on {} channel", channel));
                }

                // Find the delimiter (<IDS|MSG>)
                let mut delim_idx = None;
                for (i, part) in parts.iter().enumerate() {
                    if part == b"<IDS|MSG>" {
                        delim_idx = Some(i);
                        break;
                    }
                }

                let delim_idx = delim_idx.ok_or_else(|| 
                    anyhow::anyhow!("No delimiter found in message on {} channel", channel))?;

                if delim_idx + 5 >= parts.len() {
                    return Err(anyhow::anyhow!("Incomplete message on {} channel", channel));
                }

                // Store identities (everything before delimiter) for reply
                let identities: Vec<Vec<u8>> = parts[..delim_idx].iter().cloned().collect();

                // Extract message parts after delimiter
                let _hmac = &parts[delim_idx + 1];
                let header = &parts[delim_idx + 2]; 
                let parent_header = &parts[delim_idx + 3];
                let metadata = &parts[delim_idx + 4];
                let content = &parts[delim_idx + 5];

                // Deserialize JSON parts
                let header: crate::jupyter::protocol::MessageHeader = serde_json::from_slice(header)
                    .context("Failed to deserialize header")?;
                
                // Handle parent_header - Jupyter sends {} for empty parent
                let parent_header: Option<crate::jupyter::protocol::MessageHeader> = if parent_header.is_empty() || parent_header == b"{}" {
                    None
                } else {
                    Some(serde_json::from_slice(parent_header)
                        .context("Failed to deserialize parent_header")?)
                };
                let metadata: Value = serde_json::from_slice(metadata)
                    .context("Failed to deserialize metadata")?;
                
                // Deserialize content based on msg_type from header
                let content: crate::jupyter::protocol::MessageContent = 
                    Self::deserialize_content(&header.msg_type, content)
                        .context("Failed to deserialize content")?;

                let mut msg = JupyterMessage {
                    header,
                    parent_header,
                    metadata,
                    content,
                };
                
                // Store identities for reply (hack: use metadata field)
                if !identities.is_empty() {
                    msg.metadata["__identities"] = serde_json::json!(
                        identities.iter().map(|i| hex::encode(i)).collect::<Vec<_>>()
                    );
                }

                Ok(Some(msg))
            }
            Err(zmq::Error::EAGAIN) => Ok(None), // No message available
            Err(e) => Err(e).with_context(|| format!("Failed to receive message on {} channel", channel)),
        }
    }

    /// Internal: Serialize and send message to socket
    fn send_message(&self, socket: &Socket, msg: &JupyterMessage, channel: &str) -> Result<()> {
        // Extract identities from metadata if present
        let mut parts = Vec::new();
        
        // Add identities if they exist
        if let Some(identities) = msg.metadata.get("__identities") {
            if let Some(id_array) = identities.as_array() {
                for id in id_array {
                    if let Some(id_str) = id.as_str() {
                        if let Ok(id_bytes) = hex::decode(id_str) {
                            parts.push(id_bytes);
                        }
                    }
                }
            }
        }
        
        // If no identities, add empty one
        if parts.is_empty() {
            parts.push(vec![]);
        }
        
        // Add delimiter
        parts.push(b"<IDS|MSG>".to_vec());
        
        // Serialize message parts
        let header = serde_json::to_vec(&msg.header)
            .context("Failed to serialize header")?;
        let parent_header = match &msg.parent_header {
            Some(ph) => serde_json::to_vec(ph).context("Failed to serialize parent_header")?,
            None => vec![],
        };
        
        // Filter out __identities from metadata
        let mut clean_metadata = msg.metadata.clone();
        if let Some(obj) = clean_metadata.as_object_mut() {
            obj.remove("__identities");
        }
        let metadata = serde_json::to_vec(&clean_metadata)
            .context("Failed to serialize metadata")?;
        let content = serde_json::to_vec(&msg.content)
            .context("Failed to serialize content")?;

        // Calculate HMAC signature
        let hmac = self.calculate_hmac(&[&header, &parent_header, &metadata, &content]);

        // Add message parts
        parts.push(hmac);
        parts.push(header);
        parts.push(parent_header);
        parts.push(metadata);
        parts.push(content);

        socket.send_multipart(parts, 0)
            .with_context(|| format!("Failed to send message on {} channel", channel))?;

        Ok(())
    }

    /// Deserialize content based on message type
    fn deserialize_content(msg_type: &str, content_bytes: &[u8]) -> Result<crate::jupyter::protocol::MessageContent> {
        use crate::jupyter::protocol::MessageContent;
        
        // For most request messages, the content is empty or simple
        match msg_type {
            "kernel_info_request" => {
                // kernel_info_request has empty content
                Ok(MessageContent::KernelInfoRequest {})
            }
            "execute_request" => {
                // Parse execute request content
                let value: serde_json::Value = serde_json::from_slice(content_bytes)?;
                let code = value["code"].as_str().unwrap_or("").to_string();
                let silent = value["silent"].as_bool().unwrap_or(false);
                let store_history = value.get("store_history").and_then(|v| v.as_bool());
                let user_expressions = None; // TODO: Parse if needed
                let allow_stdin = value.get("allow_stdin").and_then(|v| v.as_bool());
                let stop_on_error = value.get("stop_on_error").and_then(|v| v.as_bool());
                
                Ok(MessageContent::ExecuteRequest {
                    code,
                    silent,
                    store_history,
                    user_expressions,
                    allow_stdin,
                    stop_on_error,
                })
            }
            "shutdown_request" => {
                let value: serde_json::Value = serde_json::from_slice(content_bytes)?;
                let restart = value["restart"].as_bool().unwrap_or(false);
                Ok(MessageContent::ShutdownRequest { restart })
            }
            "interrupt_request" => {
                Ok(MessageContent::InterruptRequest {})
            }
            // Add more message types as needed
            _ => {
                // For unknown types, try to deserialize generically
                // This might fail, but provides better error messages
                serde_json::from_slice(content_bytes)
                    .with_context(|| format!("Unknown message type: {}", msg_type))
            }
        }
    }

    /// Calculate HMAC signature for message authentication
    fn calculate_hmac(&self, parts: &[&[u8]]) -> Vec<u8> {
        use hmac::{Hmac, Mac};
        use sha2::Sha256;
        
        // Decode hex key
        let key_bytes = match hex::decode(&self.signing_key) {
            Ok(bytes) => bytes,
            Err(_) => {
                tracing::warn!("Failed to decode HMAC key, using empty signature");
                return vec![];
            }
        };
        
        // Create HMAC instance
        let mut mac = match Hmac::<Sha256>::new_from_slice(&key_bytes) {
            Ok(m) => m,
            Err(_) => {
                tracing::warn!("Failed to create HMAC, using empty signature");
                return vec![];
            }
        };
        
        // Update with all message parts
        for part in parts {
            mac.update(part);
        }
        
        // Get signature as hex string bytes
        let result = mac.finalize();
        hex::encode(result.into_bytes()).into_bytes()
    }
}

impl Drop for ZmqTransport {
    fn drop(&mut self) {
        // ZeroMQ sockets are automatically closed when dropped
        tracing::debug!("ZeroMQ transport dropped, sockets closed");
    }
}