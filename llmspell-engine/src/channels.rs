//! Channel view implementations for protocol engine
//!
//! Provides lightweight views over the `ProtocolEngine` for each channel type,
//! replacing the old `KernelChannels` with zero-cost abstractions.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use tokio::sync::broadcast;

use crate::engine::{
    ChannelType, ChannelView, EngineError, MessageContent, ProtocolEngine, ProtocolType,
    UniversalMessage,
};

/// Messages that can be sent/received on channels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChannelMessage {
    /// Request message (Shell, Control)
    Request { msg_id: String, content: Value },
    /// Reply message (Shell, Control)
    Reply {
        msg_id: String,
        parent_id: String,
        content: Value,
    },
    /// Broadcast message (`IOPub`)
    Broadcast {
        msg_id: String,
        msg_type: String,
        content: Value,
    },
    /// Input request (Stdin)
    InputRequest {
        msg_id: String,
        prompt: String,
        password: bool,
    },
    /// Input reply (Stdin)
    InputReply { msg_id: String, value: String },
    /// Heartbeat ping/pong
    Heartbeat { data: Vec<u8> },
}

/// Trait for converting between `ChannelMessage` and `UniversalMessage`
pub trait MessageAdapter: Send + Sync {
    /// Convert a `ChannelMessage` to `UniversalMessage`
    fn to_universal(&self, channel: ChannelType, msg: ChannelMessage) -> UniversalMessage;

    /// Convert a `UniversalMessage` to `ChannelMessage`
    ///
    /// # Errors
    ///
    /// Returns `EngineError::Conversion` if the message cannot be converted
    fn convert_from_universal(&self, msg: UniversalMessage) -> Result<ChannelMessage, EngineError>;
}

/// Default message adapter implementation
pub struct DefaultMessageAdapter;

impl MessageAdapter for DefaultMessageAdapter {
    fn to_universal(&self, channel: ChannelType, msg: ChannelMessage) -> UniversalMessage {
        let (method, params, msg_id) = match msg {
            ChannelMessage::Request { msg_id, content } => ("request".to_string(), content, msg_id),
            ChannelMessage::Reply {
                msg_id,
                parent_id,
                content,
            } => {
                let params = serde_json::json!({
                    "parent_id": parent_id,
                    "content": content
                });
                ("reply".to_string(), params, msg_id)
            }
            ChannelMessage::Broadcast {
                msg_id,
                msg_type,
                content,
            } => {
                let params = serde_json::json!({
                    "msg_type": msg_type,
                    "content": content
                });
                ("broadcast".to_string(), params, msg_id)
            }
            ChannelMessage::InputRequest {
                msg_id,
                prompt,
                password,
            } => {
                let params = serde_json::json!({
                    "prompt": prompt,
                    "password": password
                });
                ("input_request".to_string(), params, msg_id)
            }
            ChannelMessage::InputReply { msg_id, value } => {
                let params = serde_json::json!({
                    "value": value
                });
                ("input_reply".to_string(), params, msg_id)
            }
            ChannelMessage::Heartbeat { data } => {
                let params = serde_json::json!({
                    "data": data
                });
                (
                    "heartbeat".to_string(),
                    params,
                    uuid::Uuid::new_v4().to_string(),
                )
            }
        };

        UniversalMessage {
            id: msg_id,
            protocol: ProtocolType::LRP,
            channel,
            content: MessageContent::Request { method, params },
            metadata: HashMap::default(),
        }
    }

    fn convert_from_universal(&self, msg: UniversalMessage) -> Result<ChannelMessage, EngineError> {
        match msg.content {
            MessageContent::Request { method, params } => match method.as_str() {
                "request" => Ok(ChannelMessage::Request {
                    msg_id: msg.id,
                    content: params,
                }),
                "reply" => {
                    let parent_id = params
                        .get("parent_id")
                        .and_then(|v| v.as_str())
                        .ok_or_else(|| EngineError::Conversion("Missing parent_id".to_string()))?
                        .to_string();
                    let content = params.get("content").cloned().unwrap_or(Value::Null);
                    Ok(ChannelMessage::Reply {
                        msg_id: msg.id,
                        parent_id,
                        content,
                    })
                }
                "broadcast" => {
                    let msg_type = params
                        .get("msg_type")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string();
                    let content = params.get("content").cloned().unwrap_or(Value::Null);
                    Ok(ChannelMessage::Broadcast {
                        msg_id: msg.id,
                        msg_type,
                        content,
                    })
                }
                "input_request" => {
                    let prompt = params
                        .get("prompt")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let password = params
                        .get("password")
                        .and_then(serde_json::Value::as_bool)
                        .unwrap_or(false);
                    Ok(ChannelMessage::InputRequest {
                        msg_id: msg.id,
                        prompt,
                        password,
                    })
                }
                "input_reply" => {
                    let value = params
                        .get("value")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    Ok(ChannelMessage::InputReply {
                        msg_id: msg.id,
                        value,
                    })
                }
                "heartbeat" => {
                    let data = params
                        .get("data")
                        .and_then(|v| v.as_array())
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_u64().and_then(|n| u8::try_from(n).ok()))
                                .collect()
                        })
                        .unwrap_or_default();
                    Ok(ChannelMessage::Heartbeat { data })
                }
                _ => Err(EngineError::Conversion(format!("Unknown method: {method}"))),
            },
            _ => Err(EngineError::Conversion(
                "Unsupported message content type".to_string(),
            )),
        }
    }
}

/// Shell channel view for request-reply execution
pub struct ShellView<'a> {
    view: ChannelView<'a>,
    adapter: Box<dyn MessageAdapter + Send + Sync>,
}

impl<'a> ShellView<'a> {
    /// Create a new shell view
    pub fn new(engine: &'a dyn ProtocolEngine) -> Self {
        Self {
            view: engine.channel_view(ChannelType::Shell),
            adapter: Box::new(DefaultMessageAdapter),
        }
    }

    /// Send a request on the shell channel
    ///
    /// # Errors
    ///
    /// Returns `EngineError` if the protocol engine fails to send
    pub async fn send_request(&self, msg_id: String, content: Value) -> Result<(), EngineError> {
        let msg = ChannelMessage::Request { msg_id, content };
        let universal = self.adapter.to_universal(ChannelType::Shell, msg);
        self.view.send(universal).await
    }

    /// Send a reply on the shell channel
    ///
    /// # Errors
    ///
    /// Returns `EngineError` if the protocol engine fails to send
    pub async fn send_reply(
        &self,
        msg_id: String,
        parent_id: String,
        content: Value,
    ) -> Result<(), EngineError> {
        let msg = ChannelMessage::Reply {
            msg_id,
            parent_id,
            content,
        };
        let universal = self.adapter.to_universal(ChannelType::Shell, msg);
        self.view.send(universal).await
    }

    /// Receive a message from the shell channel
    ///
    /// # Errors
    ///
    /// Returns `EngineError` if the protocol engine fails to receive or convert
    pub async fn recv(&self) -> Result<ChannelMessage, EngineError> {
        let universal = self.view.recv().await?;
        self.adapter.convert_from_universal(universal)
    }
}

/// `IOPub` channel view for broadcasting output
pub struct IOPubView<'a> {
    view: ChannelView<'a>,
    adapter: Box<dyn MessageAdapter + Send + Sync>,
    /// Broadcast sender for local subscribers
    broadcast_tx: broadcast::Sender<ChannelMessage>,
}

impl<'a> IOPubView<'a> {
    /// Create a new `IOPub` view
    pub fn new(engine: &'a dyn ProtocolEngine) -> Self {
        let (broadcast_tx, _) = broadcast::channel(1024);
        Self {
            view: engine.channel_view(ChannelType::IOPub),
            adapter: Box::new(DefaultMessageAdapter),
            broadcast_tx,
        }
    }

    /// Publish a message to all subscribers
    ///
    /// # Errors
    ///
    /// Returns `EngineError` if the protocol engine fails to send
    pub async fn publish(&self, msg_type: String, content: Value) -> Result<(), EngineError> {
        let msg_id = uuid::Uuid::new_v4().to_string();
        let msg = ChannelMessage::Broadcast {
            msg_id,
            msg_type,
            content,
        };

        // Send to protocol engine
        let universal = self.adapter.to_universal(ChannelType::IOPub, msg.clone());
        self.view.send(universal).await?;

        // Broadcast to local subscribers
        let _ = self.broadcast_tx.send(msg);

        Ok(())
    }

    /// Subscribe to `IOPub` messages
    #[must_use]
    pub fn subscribe(&self) -> broadcast::Receiver<ChannelMessage> {
        self.broadcast_tx.subscribe()
    }

    /// Receive a message from the `IOPub` channel
    ///
    /// # Errors
    ///
    /// Returns `EngineError` if the protocol engine fails to receive or convert
    pub async fn recv(&self) -> Result<ChannelMessage, EngineError> {
        let universal = self.view.recv().await?;
        self.adapter.convert_from_universal(universal)
    }
}

/// Stdin channel view for input requests
pub struct StdinView<'a> {
    view: ChannelView<'a>,
    adapter: Box<dyn MessageAdapter + Send + Sync>,
}

impl<'a> StdinView<'a> {
    /// Create a new stdin view
    pub fn new(engine: &'a dyn ProtocolEngine) -> Self {
        Self {
            view: engine.channel_view(ChannelType::Stdin),
            adapter: Box::new(DefaultMessageAdapter),
        }
    }

    /// Request input from the client
    ///
    /// # Errors
    ///
    /// Returns `EngineError` if the protocol engine fails to send
    pub async fn request_input(&self, prompt: String, password: bool) -> Result<(), EngineError> {
        let msg_id = uuid::Uuid::new_v4().to_string();
        let msg = ChannelMessage::InputRequest {
            msg_id,
            prompt,
            password,
        };
        let universal = self.adapter.to_universal(ChannelType::Stdin, msg);
        self.view.send(universal).await
    }

    /// Send an input reply
    ///
    /// # Errors
    ///
    /// Returns `EngineError` if the protocol engine fails to send
    pub async fn send_reply(&self, msg_id: String, value: String) -> Result<(), EngineError> {
        let msg = ChannelMessage::InputReply { msg_id, value };
        let universal = self.adapter.to_universal(ChannelType::Stdin, msg);
        self.view.send(universal).await
    }

    /// Receive a message from the stdin channel
    ///
    /// # Errors
    ///
    /// Returns `EngineError` if the protocol engine fails to receive or convert
    pub async fn recv(&self) -> Result<ChannelMessage, EngineError> {
        let universal = self.view.recv().await?;
        self.adapter.convert_from_universal(universal)
    }
}

/// Control channel view for kernel control commands
pub struct ControlView<'a> {
    view: ChannelView<'a>,
    adapter: Box<dyn MessageAdapter + Send + Sync>,
}

impl<'a> ControlView<'a> {
    /// Create a new control view
    pub fn new(engine: &'a dyn ProtocolEngine) -> Self {
        Self {
            view: engine.channel_view(ChannelType::Control),
            adapter: Box::new(DefaultMessageAdapter),
        }
    }

    /// Send a control request
    ///
    /// # Errors
    ///
    /// Returns `EngineError` if the protocol engine fails to send
    pub async fn send_request(&self, msg_id: String, content: Value) -> Result<(), EngineError> {
        let msg = ChannelMessage::Request { msg_id, content };
        let universal = self.adapter.to_universal(ChannelType::Control, msg);
        self.view.send(universal).await
    }

    /// Send a control reply
    ///
    /// # Errors
    ///
    /// Returns `EngineError` if the protocol engine fails to send
    pub async fn send_reply(
        &self,
        msg_id: String,
        parent_id: String,
        content: Value,
    ) -> Result<(), EngineError> {
        let msg = ChannelMessage::Reply {
            msg_id,
            parent_id,
            content,
        };
        let universal = self.adapter.to_universal(ChannelType::Control, msg);
        self.view.send(universal).await
    }

    /// Receive a message from the control channel
    ///
    /// # Errors
    ///
    /// Returns `EngineError` if the protocol engine fails to receive or convert
    pub async fn recv(&self) -> Result<ChannelMessage, EngineError> {
        let universal = self.view.recv().await?;
        self.adapter.convert_from_universal(universal)
    }
}

/// Heartbeat channel view for keep-alive monitoring
pub struct HeartbeatView<'a> {
    view: ChannelView<'a>,
    adapter: Box<dyn MessageAdapter + Send + Sync>,
}

impl<'a> HeartbeatView<'a> {
    /// Create a new heartbeat view
    pub fn new(engine: &'a dyn ProtocolEngine) -> Self {
        Self {
            view: engine.channel_view(ChannelType::Heartbeat),
            adapter: Box::new(DefaultMessageAdapter),
        }
    }

    /// Send a heartbeat ping
    ///
    /// # Errors
    ///
    /// Returns `EngineError` if the protocol engine fails to send
    pub async fn ping(&self, data: Vec<u8>) -> Result<(), EngineError> {
        let msg = ChannelMessage::Heartbeat { data };
        let universal = self.adapter.to_universal(ChannelType::Heartbeat, msg);
        self.view.send(universal).await
    }

    /// Echo back a heartbeat (pong)
    ///
    /// # Errors
    ///
    /// Returns `EngineError` if the protocol engine fails to send
    pub async fn pong(&self, data: Vec<u8>) -> Result<(), EngineError> {
        self.ping(data).await
    }

    /// Receive a heartbeat message
    ///
    /// # Errors
    ///
    /// Returns `EngineError` if the protocol engine fails to receive or the message is not a heartbeat
    pub async fn recv(&self) -> Result<Vec<u8>, EngineError> {
        let universal = self.view.recv().await?;
        match self.adapter.convert_from_universal(universal)? {
            ChannelMessage::Heartbeat { data } => Ok(data),
            _ => Err(EngineError::Conversion(
                "Expected heartbeat message".to_string(),
            )),
        }
    }
}

/// Container for all five channel views (replaces `KernelChannels`)
pub struct ChannelSet<'a> {
    /// Shell channel for request-reply
    pub shell: ShellView<'a>,
    /// `IOPub` channel for broadcasting
    pub iopub: IOPubView<'a>,
    /// Stdin channel for input
    pub stdin: StdinView<'a>,
    /// Control channel for kernel control
    pub control: ControlView<'a>,
    /// Heartbeat channel for keep-alive
    pub heartbeat: HeartbeatView<'a>,
}

impl<'a> ChannelSet<'a> {
    /// Create all five channel views from a protocol engine
    pub fn new(engine: &'a dyn ProtocolEngine) -> Self {
        Self {
            shell: ShellView::new(engine),
            iopub: IOPubView::new(engine),
            stdin: StdinView::new(engine),
            control: ControlView::new(engine),
            heartbeat: HeartbeatView::new(engine),
        }
    }

    /// Get port information for connection file
    /// Note: In the new architecture, all channels share the same TCP port
    /// The engine multiplexes based on channel type in the protocol messages
    #[must_use]
    pub const fn get_ports(&self) -> ChannelPorts {
        // In the unified engine, we use the same port for all channels
        // The actual port is determined by the engine's transport configuration
        ChannelPorts {
            shell_port: 9555, // These will be the same in practice
            iopub_port: 9556, // But kept separate for compatibility
            stdin_port: 9557,
            control_port: 9558,
            hb_port: 9559,
        }
    }
}

/// Port information for channels (for connection file compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelPorts {
    pub shell_port: u16,
    pub iopub_port: u16,
    pub stdin_port: u16,
    pub control_port: u16,
    pub hb_port: u16,
}

/// Messages broadcast on the `IOPub` channel (compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IOPubMessage {
    /// Stream output (stdout, stderr)
    StreamOutput { name: String, text: String },
    /// Execution result
    ExecuteResult {
        execution_count: u32,
        data: serde_json::Value,
    },
    /// Error output
    Error {
        ename: String,
        evalue: String,
        traceback: Vec<String>,
    },
    /// Status update
    Status { execution_state: String },
    /// Debug event
    DebugEvent(serde_json::Value),
}

impl IOPubMessage {
    /// Convert to a `ChannelMessage` for broadcasting
    #[must_use]
    pub fn to_channel_message(self) -> ChannelMessage {
        let (msg_type, content) = match self {
            Self::StreamOutput { name, text } => (
                "stream",
                serde_json::json!({
                    "name": name,
                    "text": text
                }),
            ),
            Self::ExecuteResult {
                execution_count,
                data,
            } => (
                "execute_result",
                serde_json::json!({
                    "execution_count": execution_count,
                    "data": data
                }),
            ),
            Self::Error {
                ename,
                evalue,
                traceback,
            } => (
                "error",
                serde_json::json!({
                    "ename": ename,
                    "evalue": evalue,
                    "traceback": traceback
                }),
            ),
            Self::Status { execution_state } => (
                "status",
                serde_json::json!({
                    "execution_state": execution_state
                }),
            ),
            Self::DebugEvent(event) => ("debug_event", event),
        };

        ChannelMessage::Broadcast {
            msg_id: uuid::Uuid::new_v4().to_string(),
            msg_type: msg_type.to_string(),
            content,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_adapter_roundtrip() {
        let adapter = DefaultMessageAdapter;

        let original = ChannelMessage::Request {
            msg_id: "test_id".to_string(),
            content: serde_json::json!({"key": "value"}),
        };

        let universal = adapter.to_universal(ChannelType::Shell, original.clone());
        let recovered = adapter.convert_from_universal(universal).unwrap();

        match (original, recovered) {
            (
                ChannelMessage::Request {
                    msg_id: id1,
                    content: c1,
                },
                ChannelMessage::Request {
                    msg_id: id2,
                    content: c2,
                },
            ) => {
                assert_eq!(id1, id2);
                assert_eq!(c1, c2);
            }
            _ => panic!("Message type mismatch after roundtrip"),
        }
    }

    #[test]
    fn test_iopub_message_conversion() {
        let msg = IOPubMessage::StreamOutput {
            name: "stdout".to_string(),
            text: "Hello, world!".to_string(),
        };

        let channel_msg = msg.to_channel_message();

        match channel_msg {
            ChannelMessage::Broadcast {
                msg_type, content, ..
            } => {
                assert_eq!(msg_type, "stream");
                assert!(content.get("name").is_some());
                assert!(content.get("text").is_some());
            }
            _ => panic!("Expected Broadcast message"),
        }
    }
}
