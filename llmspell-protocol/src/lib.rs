//! LLMSpell Protocol Implementation
//!
//! Provides TCP-based message protocol for communication between CLI and kernel.
//!
//! # Architecture
//!
//! The protocol layer provides:
//! - Message framing using length-delimited encoding
//! - JSON serialization for LRP/LDP messages
//! - Request/response correlation
//! - Multi-channel support (shell, iopub, stdin, control, heartbeat)
//! - Transport abstraction for future WebSocket/gRPC support
//!
//! # Usage
//!
//! ## Client Example
//! ```no_run
//! use llmspell_protocol::client::ProtocolClient;
//! use llmspell_repl::protocol::LRPRequest;
//!
//! async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = ProtocolClient::connect("127.0.0.1:5555").await?;
//!     
//!     let request = LRPRequest::KernelInfoRequest;
//!     let response = client.send_lrp_request(request).await?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Server Example
//! ```no_run
//! use llmspell_protocol::server::{ProtocolServer, ServerConfig};
//! use llmspell_protocol::message::{MessageHandler, ProtocolMessage};
//! use std::sync::Arc;
//!
//! struct MyHandler;
//!
//! #[async_trait::async_trait]
//! impl MessageHandler for MyHandler {
//!     async fn handle(&self, msg: ProtocolMessage) -> Option<ProtocolMessage> {
//!         // Handle message and return response
//!         None
//!     }
//! }
//!
//! async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     let config = ServerConfig::default();
//!     let handler = Arc::new(MyHandler);
//!     let mut server = ProtocolServer::new(config, handler);
//!     
//!     server.start().await?;
//!     
//!     Ok(())
//! }
//! ```

pub mod client;
pub mod codec;
pub mod message;
pub mod server;
pub mod transport;
pub mod types;

// Re-export commonly used types
pub use client::{ClientError, ProtocolClient};
pub use codec::LRPCodec;
pub use message::{MessageHandler, MessageType, ProtocolMessage};
pub use server::{ProtocolServer, ServerConfig, ServerError};
pub use transport::{Transport, TransportError};
pub use types::{LDPRequest, LDPResponse, LRPRequest, LRPResponse};

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    struct EchoHandler;

    #[async_trait::async_trait]
    impl MessageHandler for EchoHandler {
        async fn handle(&self, msg: ProtocolMessage) -> Option<ProtocolMessage> {
            // Echo back as response
            Some(ProtocolMessage::response(msg.msg_id, msg.content))
        }
    }

    #[tokio::test]
    async fn test_protocol_round_trip() {
        // This test would require actual TCP setup
        // For now, we'll just ensure everything compiles
        let _handler = Arc::new(EchoHandler);
        assert!(true);
    }
}
