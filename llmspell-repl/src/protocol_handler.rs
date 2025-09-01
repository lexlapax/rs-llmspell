//! Protocol message handler for kernel TCP server
//!
//! Implements the `MessageHandler` trait to process incoming LRP/LDP messages

use async_trait::async_trait;
use llmspell_engine::{MessageHandler, MessageType, ProtocolMessage};
use std::sync::Arc;
use tracing::{debug, warn};

use crate::kernel::LLMSpellKernel;
use crate::protocol::{LDPRequest, LDPResponse, LRPRequest, LRPResponse};

/// Protocol handler that routes messages to kernel methods
pub struct KernelProtocolHandler {
    kernel: Arc<LLMSpellKernel>,
}

impl KernelProtocolHandler {
    /// Create a new protocol handler for the kernel
    #[must_use]
    pub const fn new(kernel: Arc<LLMSpellKernel>) -> Self {
        Self { kernel }
    }
}

#[async_trait]
impl MessageHandler for KernelProtocolHandler {
    async fn handle(&self, msg: ProtocolMessage) -> Option<ProtocolMessage> {
        debug!("Handling message: {:?}", msg.msg_id);

        if msg.msg_type == MessageType::Request {
            self.handle_request(msg).await
        } else {
            warn!("Unexpected message type: {:?}", msg.msg_type);
            None
        }
    }
}

impl KernelProtocolHandler {
    async fn handle_request(&self, msg: ProtocolMessage) -> Option<ProtocolMessage> {
        // Try to parse as LRP request first
        if let Ok(lrp_request) = serde_json::from_value::<LRPRequest>(msg.content.clone()) {
            let response = self.handle_lrp_request(lrp_request).await;
            return Some(ProtocolMessage::response(msg.msg_id, response));
        }

        // Try to parse as LDP request
        if let Ok(ldp_request) = serde_json::from_value::<LDPRequest>(msg.content.clone()) {
            let response = Self::handle_ldp_request(&ldp_request);
            return Some(ProtocolMessage::response(msg.msg_id, response));
        }

        warn!("Unknown request type");
        None
    }

    async fn handle_lrp_request(&self, request: LRPRequest) -> LRPResponse {
        match request {
            LRPRequest::KernelInfoRequest => self.build_kernel_info(),
            LRPRequest::ExecuteRequest { code, silent, .. } => {
                self.execute_code(code, silent).await
            }
            LRPRequest::CompleteRequest {
                code: _,
                cursor_pos,
            } => {
                // TODO: Implement completion
                LRPResponse::CompleteReply {
                    matches: vec![],
                    cursor_start: cursor_pos,
                    cursor_end: cursor_pos,
                    metadata: Some(serde_json::Value::Null),
                    status: "ok".to_string(),
                }
            }
            LRPRequest::InspectRequest { .. } => {
                // TODO: Implement inspection
                LRPResponse::InspectReply {
                    status: "ok".to_string(),
                    found: false,
                    data: Some(serde_json::Value::Null),
                    metadata: Some(serde_json::Value::Null),
                }
            }
            LRPRequest::IsCompleteRequest { code } => LRPResponse::IsCompleteReply {
                status: if code.trim().is_empty() {
                    "incomplete"
                } else {
                    "complete"
                }
                .to_string(),
                indent: String::new(),
            },
            LRPRequest::ShutdownRequest { restart } => LRPResponse::ShutdownReply { restart },
            LRPRequest::InterruptRequest => LRPResponse::InterruptReply,
            LRPRequest::HistoryRequest { .. } => LRPResponse::HistoryReply { history: vec![] },
            LRPRequest::CommInfoRequest { .. } => LRPResponse::CommInfoReply {
                comms: serde_json::Value::Object(serde_json::Map::new()),
            },
            LRPRequest::ConnectRequest => LRPResponse::ConnectReply {
                shell_port: self.kernel.connection_info.shell_port,
                iopub_port: self.kernel.connection_info.iopub_port,
                stdin_port: self.kernel.connection_info.stdin_port,
                control_port: self.kernel.connection_info.control_port,
                hb_port: self.kernel.connection_info.hb_port,
            },
        }
    }

    fn handle_ldp_request(request: &LDPRequest) -> LDPResponse {
        match request {
            LDPRequest::InitializeRequest { .. } => LDPResponse::InitializeResponse {
                capabilities: serde_json::json!({
                    "supportsConfigurationDoneRequest": true,
                    "supportsFunctionBreakpoints": false,
                    "supportsConditionalBreakpoints": true,
                    "supportsEvaluateForHovers": true,
                    "supportsStepBack": false,
                    "supportsSetVariable": true,
                    "supportsRestartFrame": false,
                    "supportsStepInTargetsRequest": false,
                    "supportsModulesRequest": false,
                    "supportsTerminateThreadsRequest": false,
                    "supportsDelayedStackTraceLoading": false,
                }),
            },
            _ => {
                // TODO: Implement other debug commands
                // For now, return a generic response since ErrorResponse doesn't exist
                LDPResponse::InitializeResponse {
                    capabilities: serde_json::json!({
                        "error": "Debug command not yet implemented"
                    }),
                }
            }
        }
    }

    fn build_kernel_info(&self) -> LRPResponse {
        LRPResponse::KernelInfoReply {
            protocol_version: "5.3".to_string(),
            implementation: "llmspell".to_string(),
            implementation_version: env!("CARGO_PKG_VERSION").to_string(),
            language_info: crate::protocol::LanguageInfo {
                name: self.kernel.config.engine.clone(),
                version: "1.0.0".to_string(),
                mimetype: "text/plain".to_string(),
                file_extension: ".txt".to_string(),
                pygments_lexer: None,
                codemirror_mode: None,
                nbconvert_exporter: None,
            },
            banner: format!("LLMSpell Kernel - {}", self.kernel.config.engine),
            debugger: false,
            help_links: vec![],
        }
    }

    async fn execute_code(&self, code: String, silent: bool) -> LRPResponse {
        match self.kernel.execute_code("protocol", code, silent).await {
            Ok(response) => response,
            Err(e) => {
                warn!("Execution error: {}", e);
                LRPResponse::ExecuteReply {
                    status: "error".to_string(),
                    execution_count: 0,
                    user_expressions: None,
                    payload: None,
                }
            }
        }
    }
}
