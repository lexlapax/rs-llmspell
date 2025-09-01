//! Message processor trait for protocol handling
//!
//! Provides abstraction for processing protocol messages, enabling
//! clean separation between protocol handling and business logic.

use async_trait::async_trait;
use std::fmt::Debug;
use thiserror::Error;

use crate::protocol::ldp::{LDPRequest, LDPResponse};
use crate::protocol::lrp::{LRPRequest, LRPResponse};

/// Errors that can occur during message processing
#[derive(Error, Debug)]
pub enum ProcessorError {
    #[error("Processing failed: {0}")]
    ProcessingFailed(String),

    #[error("Not implemented: {0}")]
    NotImplemented(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

/// Message processor trait for handling protocol messages
///
/// This trait is implemented by services (like the kernel) to handle
/// protocol-specific messages. It enables dependency injection and
/// avoids circular dependencies between the engine and service crates.
#[async_trait]
pub trait MessageProcessor: Send + Sync + Debug {
    /// Process an LRP request and return a response
    async fn process_lrp(&self, request: LRPRequest) -> Result<LRPResponse, ProcessorError>;

    /// Process an LDP request and return a response
    async fn process_ldp(&self, request: LDPRequest) -> Result<LDPResponse, ProcessorError>;

    /// Get processor capabilities (optional)
    fn capabilities(&self) -> Vec<String> {
        vec!["lrp".to_string(), "ldp".to_string()]
    }
}

/// Null message processor for testing
#[derive(Debug, Clone)]
pub struct NullMessageProcessor;

#[async_trait]
impl MessageProcessor for NullMessageProcessor {
    async fn process_lrp(&self, request: LRPRequest) -> Result<LRPResponse, ProcessorError> {
        // Return minimal valid responses for testing
        match request {
            LRPRequest::KernelInfoRequest => Ok(LRPResponse::KernelInfoReply {
                protocol_version: "5.3".to_string(),
                implementation: "null".to_string(),
                implementation_version: "0.0.0".to_string(),
                language_info: crate::protocol::lrp::LanguageInfo {
                    name: "null".to_string(),
                    version: "0.0.0".to_string(),
                    mimetype: "text/plain".to_string(),
                    file_extension: ".txt".to_string(),
                    pygments_lexer: None,
                    codemirror_mode: None,
                    nbconvert_exporter: None,
                },
                banner: "Null Kernel".to_string(),
                debugger: false,
                help_links: vec![],
            }),
            LRPRequest::ExecuteRequest { .. } => Ok(LRPResponse::ExecuteReply {
                status: "ok".to_string(),
                execution_count: 0,
                user_expressions: None,
                payload: None,
            }),
            LRPRequest::ShutdownRequest { restart } => Ok(LRPResponse::ShutdownReply { restart }),
            LRPRequest::InterruptRequest => Ok(LRPResponse::InterruptReply),
            LRPRequest::ConnectRequest => Ok(LRPResponse::ConnectReply {
                shell_port: 0,
                iopub_port: 0,
                stdin_port: 0,
                control_port: 0,
                hb_port: 0,
            }),
            _ => Err(ProcessorError::NotImplemented(format!(
                "LRP request type not implemented in null processor"
            ))),
        }
    }

    async fn process_ldp(&self, request: LDPRequest) -> Result<LDPResponse, ProcessorError> {
        // Return minimal valid responses for testing
        match request {
            LDPRequest::InitializeRequest { .. } => Ok(LDPResponse::InitializeResponse {
                capabilities: serde_json::json!({
                    "supportsConfigurationDoneRequest": false,
                    "supportsFunctionBreakpoints": false,
                    "supportsConditionalBreakpoints": false,
                }),
            }),
            _ => Err(ProcessorError::NotImplemented(format!(
                "LDP request type not implemented in null processor"
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_null_processor_lrp() {
        let processor = NullMessageProcessor;

        let request = LRPRequest::KernelInfoRequest;
        let response = processor.process_lrp(request).await.unwrap();

        match response {
            LRPResponse::KernelInfoReply { implementation, .. } => {
                assert_eq!(implementation, "null");
            }
            _ => panic!("Expected KernelInfoReply"),
        }
    }

    #[tokio::test]
    async fn test_null_processor_ldp() {
        let processor = NullMessageProcessor;

        let request = LDPRequest::InitializeRequest {
            client_id: "test".to_string(),
            client_name: "test".to_string(),
            adapter_id: "test".to_string(),
            locale: None,
            lines_start_at_1: true,
            columns_start_at_1: true,
            path_format: None,
            supports_variable_type: false,
            supports_variable_paging: false,
            supports_run_in_terminal_request: false,
            supports_memory_references: false,
            supports_progress_reporting: false,
            supports_invalidated_event: false,
        };

        let response = processor.process_ldp(request).await.unwrap();

        match response {
            LDPResponse::InitializeResponse { .. } => {
                // Success
            }
            _ => panic!("Expected InitializeResponse"),
        }
    }
}
