//! Protocol adapter implementations
//!
//! Provides concrete adapter implementations for different protocols.

use async_trait::async_trait;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use crate::engine::{
    Capability, ChannelType, EngineError, MessageContent, ProtocolAdapter, ProtocolType,
    UniversalMessage,
};
use crate::processor::MessageProcessor;
use crate::protocol::ldp::{LDPRequest, LDPResponse};
use crate::protocol::lrp::{LRPRequest, LRPResponse};

/// LRP (`LLMSpell` REPL Protocol) adapter
pub struct LRPAdapter {
    /// Optional message processor for handling requests
    processor: Option<Arc<dyn MessageProcessor>>,
}

impl Default for LRPAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl LRPAdapter {
    /// Create a new LRP adapter without a processor
    #[must_use]
    pub fn new() -> Self {
        Self { processor: None }
    }

    /// Create a new LRP adapter with a processor
    #[must_use]
    pub fn with_processor(processor: Arc<dyn MessageProcessor>) -> Self {
        Self {
            processor: Some(processor),
        }
    }
}

#[async_trait]
impl ProtocolAdapter for LRPAdapter {
    fn protocol_type(&self) -> ProtocolType {
        ProtocolType::LRP
    }

    fn adapt_inbound(&self, raw: &[u8]) -> Result<UniversalMessage, EngineError> {
        // Parse the raw bytes as JSON
        let json: Value = serde_json::from_slice(raw)
            .map_err(|e| EngineError::Conversion(format!("Failed to parse JSON: {e}")))?;

        // Try to parse as LRP request
        let request = serde_json::from_value::<LRPRequest>(json.clone())
            .map_err(|_| EngineError::Conversion("Failed to parse as LRP request".to_string()))?;

        // Determine channel based on request type
        let channel = match &request {
            LRPRequest::ShutdownRequest { .. } | LRPRequest::InterruptRequest => {
                ChannelType::Control
            }
            _ => ChannelType::Shell, // All other requests use Shell channel
        };

        // Convert to method and params for universal message
        let (method, params) = match &request {
            LRPRequest::KernelInfoRequest => ("kernel_info", Value::Null),
            LRPRequest::ExecuteRequest { code, silent, .. } => (
                "execute",
                serde_json::json!({ "code": code, "silent": silent }),
            ),
            LRPRequest::CompleteRequest { code, cursor_pos } => (
                "complete",
                serde_json::json!({ "code": code, "cursor_pos": cursor_pos }),
            ),
            LRPRequest::InspectRequest {
                code,
                cursor_pos,
                detail_level,
            } => (
                "inspect",
                serde_json::json!({
                    "code": code,
                    "cursor_pos": cursor_pos,
                    "detail_level": detail_level
                }),
            ),
            LRPRequest::IsCompleteRequest { code } => {
                ("is_complete", serde_json::json!({ "code": code }))
            }
            LRPRequest::ShutdownRequest { restart } => {
                ("shutdown", serde_json::json!({ "restart": restart }))
            }
            LRPRequest::InterruptRequest => ("interrupt", Value::Null),
            LRPRequest::HistoryRequest { .. } => ("history", json),
            LRPRequest::CommInfoRequest { .. } => ("comm_info", json),
            LRPRequest::ConnectRequest => ("connect", Value::Null),
        };

        // Store original request for potential processing
        let mut metadata = HashMap::new();
        metadata.insert(
            "lrp_request".to_string(),
            serde_json::to_value(&request).unwrap_or(Value::Null),
        );

        Ok(UniversalMessage {
            id: uuid::Uuid::new_v4().to_string(),
            protocol: ProtocolType::LRP,
            channel,
            content: MessageContent::Request {
                method: method.to_string(),
                params,
            },
            metadata,
        })
    }

    fn adapt_outbound(&self, msg: &UniversalMessage) -> Result<Vec<u8>, EngineError> {
        // Check if we have an LRP response in metadata (from processor)
        if let Some(lrp_response_value) = msg.metadata.get("lrp_response") {
            // Direct serialization of LRP response
            return serde_json::to_vec(lrp_response_value).map_err(|e| {
                EngineError::Conversion(format!("Failed to serialize LRP response: {e}"))
            });
        }

        // Otherwise, convert universal message to LRP format
        let json = match &msg.content {
            MessageContent::Response { result, error } => {
                // Try to extract LRP response from result
                result
                    .as_ref()
                    .map_or_else(|| error.clone().unwrap_or(Value::Null), Clone::clone)
            }
            MessageContent::Request { method, params } => {
                // Convert back to LRP request format
                match method.as_str() {
                    "kernel_info" => serde_json::to_value(LRPRequest::KernelInfoRequest),
                    "execute" => {
                        if let Some(code) = params.get("code").and_then(|v| v.as_str()) {
                            let silent = params
                                .get("silent")
                                .and_then(serde_json::Value::as_bool)
                                .unwrap_or(false);
                            serde_json::to_value(LRPRequest::ExecuteRequest {
                                code: code.to_string(),
                                silent,
                                store_history: true,
                                user_expressions: None,
                                allow_stdin: false,
                                stop_on_error: true,
                            })
                        } else {
                            return Err(EngineError::Conversion(
                                "Missing code parameter".to_string(),
                            ));
                        }
                    }
                    _ => Ok(Value::Null),
                }
                .map_err(|e| EngineError::Conversion(format!("Failed to serialize request: {e}")))?
            }
            MessageContent::Notification { event, data } => {
                serde_json::json!({
                    "event": event,
                    "data": data
                })
            }
            MessageContent::Raw { data } => {
                return Ok(data.clone());
            }
        };

        serde_json::to_vec(&json)
            .map_err(|e| EngineError::Conversion(format!("Failed to serialize to bytes: {e}")))
    }

    fn capabilities(&self) -> HashSet<Capability> {
        let mut caps = HashSet::new();
        caps.insert(Capability::RequestReply);
        caps.insert(Capability::PubSub);
        caps.insert(Capability::Control);
        caps.insert(Capability::Heartbeat);
        caps
    }
}

impl LRPAdapter {
    /// Process an LRP request using the processor if available
    pub async fn process_request(&self, request: LRPRequest) -> Option<LRPResponse> {
        if let Some(processor) = &self.processor {
            processor.process_lrp(request).await.ok()
        } else {
            None
        }
    }
}

/// LDP (`LLMSpell` Debug Protocol) adapter
pub struct LDPAdapter {
    /// Optional message processor for handling requests
    processor: Option<Arc<dyn MessageProcessor>>,
}

impl Default for LDPAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl LDPAdapter {
    /// Create a new LDP adapter without a processor
    #[must_use]
    pub fn new() -> Self {
        Self { processor: None }
    }

    /// Create a new LDP adapter with a processor
    #[must_use]
    pub fn with_processor(processor: Arc<dyn MessageProcessor>) -> Self {
        Self {
            processor: Some(processor),
        }
    }
}

#[async_trait]
impl ProtocolAdapter for LDPAdapter {
    fn protocol_type(&self) -> ProtocolType {
        ProtocolType::LDP
    }

    fn adapt_inbound(&self, raw: &[u8]) -> Result<UniversalMessage, EngineError> {
        // Parse the raw bytes as JSON
        let json: Value = serde_json::from_slice(raw)
            .map_err(|e| EngineError::Conversion(format!("Failed to parse JSON: {e}")))?;

        // Try to parse as LDP request
        let request = serde_json::from_value::<LDPRequest>(json.clone())
            .map_err(|_| EngineError::Conversion("Failed to parse as LDP request".to_string()))?;

        let (method, params) = match &request {
            LDPRequest::InitializeRequest { client_id, .. } => {
                ("initialize", serde_json::json!({ "client_id": client_id }))
            }
            LDPRequest::SetBreakpointsRequest { .. } => ("setBreakpoints", json),
            LDPRequest::ContinueRequest { thread_id, .. } => {
                ("continue", serde_json::json!({ "thread_id": thread_id }))
            }
            LDPRequest::NextRequest { thread_id, .. } => {
                ("next", serde_json::json!({ "thread_id": thread_id }))
            }
            LDPRequest::StepInRequest { thread_id, .. } => {
                ("stepIn", serde_json::json!({ "thread_id": thread_id }))
            }
            LDPRequest::StepOutRequest { thread_id, .. } => {
                ("stepOut", serde_json::json!({ "thread_id": thread_id }))
            }
            LDPRequest::PauseRequest { thread_id } => {
                ("pause", serde_json::json!({ "thread_id": thread_id }))
            }
            LDPRequest::StackTraceRequest { thread_id, .. } => {
                ("stackTrace", serde_json::json!({ "thread_id": thread_id }))
            }
            LDPRequest::EvaluateRequest { expression, .. } => {
                ("evaluate", serde_json::json!({ "expression": expression }))
            }
            _ => ("unknown", json),
        };

        // Store original request for potential processing
        let mut metadata = HashMap::new();
        metadata.insert(
            "ldp_request".to_string(),
            serde_json::to_value(&request).unwrap_or(Value::Null),
        );

        Ok(UniversalMessage {
            id: uuid::Uuid::new_v4().to_string(),
            protocol: ProtocolType::LDP,
            channel: ChannelType::Control,
            content: MessageContent::Request {
                method: method.to_string(),
                params,
            },
            metadata,
        })
    }

    fn adapt_outbound(&self, msg: &UniversalMessage) -> Result<Vec<u8>, EngineError> {
        // Check if we have an LDP response in metadata (from processor)
        if let Some(ldp_response_value) = msg.metadata.get("ldp_response") {
            // Direct serialization of LDP response
            return serde_json::to_vec(ldp_response_value).map_err(|e| {
                EngineError::Conversion(format!("Failed to serialize LDP response: {e}"))
            });
        }

        // Otherwise, convert universal message to LDP format
        let json = serde_json::to_value(&msg.content)
            .map_err(|e| EngineError::Conversion(format!("Failed to serialize: {e}")))?;

        serde_json::to_vec(&json)
            .map_err(|e| EngineError::Conversion(format!("Failed to serialize to bytes: {e}")))
    }

    fn capabilities(&self) -> HashSet<Capability> {
        let mut caps = HashSet::new();
        caps.insert(Capability::RequestReply);
        caps.insert(Capability::Control);
        caps.insert(Capability::Binary);
        caps
    }
}

impl LDPAdapter {
    /// Process an LDP request using the processor if available
    pub async fn process_request(&self, request: LDPRequest) -> Option<LDPResponse> {
        if let Some(processor) = &self.processor {
            processor.process_ldp(request).await.ok()
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lrp_adapter_capabilities() {
        let adapter = LRPAdapter::new();
        let caps = adapter.capabilities();

        assert!(caps.contains(&Capability::RequestReply));
        assert!(caps.contains(&Capability::PubSub));
        assert!(caps.contains(&Capability::Control));
        assert!(caps.contains(&Capability::Heartbeat));
    }

    #[test]
    fn test_ldp_adapter_capabilities() {
        let adapter = LDPAdapter::new();
        let caps = adapter.capabilities();

        assert!(caps.contains(&Capability::RequestReply));
        assert!(caps.contains(&Capability::Control));
        assert!(caps.contains(&Capability::Binary));
    }

    #[test]
    fn test_lrp_adapter_inbound() {
        let adapter = LRPAdapter::new();

        let request = LRPRequest::KernelInfoRequest;
        let json = serde_json::to_vec(&request).unwrap();

        let msg = adapter.adapt_inbound(&json).unwrap();

        assert_eq!(msg.protocol, ProtocolType::LRP);
        match msg.content {
            MessageContent::Request { method, .. } => {
                assert_eq!(method, "kernel_info");
            }
            _ => panic!("Expected Request content"),
        }
    }
}
