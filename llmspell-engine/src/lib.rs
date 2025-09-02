//! `LLMSpell` Engine - Unified Protocol Engine
//!
//! Central communication engine that unifies all protocol handling, transport management,
//! and message routing for the `LLMSpell` system.

//!
//! # Architecture
//!
//! The engine provides:
//! - Unified protocol handling (LRP, LDP, future MCP/LSP/DAP/A2A)
//! - Transport abstraction (TCP, future WebSocket/gRPC)
//! - Message routing and correlation
//! - Service mesh sidecar pattern support
//! - Zero-cost channel views
//!
//! # Structure
//!
//! - `transport` - Transport trait and implementations (foundational)
//! - `protocol/` - Protocol-specific implementations as submodules
//!   - `lrp` - `LLMSpell` REPL Protocol
//!   - `ldp` - `LLMSpell` Debug Protocol
//!   - `codec` - Message framing
//!   - `message` - Protocol messages
//! - `client` - Client implementation
//! - `server` - Server implementation (to be refactored into engine)
//!
//! # Usage
//!
//! ## Client Example
//! ```no_run
//! use llmspell_engine::ProtocolClient;
//! use llmspell_engine::protocol::lrp::LRPRequest;
//!
//! async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = ProtocolClient::connect("127.0.0.1:9555").await?;
//!     
//!     let request = LRPRequest::KernelInfoRequest;
//!     let response = client.send_lrp_request(request).await?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! ## Engine Example
//! ```no_run
//! use llmspell_engine::{UnifiedProtocolEngine, MessageProcessor, ProtocolType, ChannelType};
//! use std::sync::Arc;
//!
//! struct MyProcessor;
//!
//! #[async_trait::async_trait]
//! impl MessageProcessor for MyProcessor {
//!     async fn process_message(
//!         &self,
//!         protocol: ProtocolType,
//!         channel: ChannelType,
//!         message: Vec<u8>,
//!     ) -> Result<Vec<u8>, Box<dyn std::error::Error + Send + Sync>> {
//!         // Process message and return response
//!         Ok(message)
//!     }
//!     
//!     async fn handle_connection(
//!         &self,
//!         protocol: ProtocolType,
//!         stream: tokio::net::TcpStream,
//!     ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
//!         Ok(())
//!     }
//! }
//!
//! async fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     let transport = Box::new(llmspell_engine::transport::mock::MockTransport::new());
//!     let processor = Arc::new(MyProcessor);
//!     let engine = UnifiedProtocolEngine::with_processor(transport, processor);
//!     
//!     // Use engine.serve() to handle TCP connections
//!     
//!     Ok(())
//! }
//! ```

pub mod adapters;
pub mod channels;
pub mod client;
pub mod debug_bridge;
pub mod engine;
pub mod processor;
pub mod protocol;
pub mod server;
pub mod sidecar;
pub mod transport;

// Re-export core types at crate root for convenience
pub use adapters::{LDPAdapter, LRPAdapter};
pub use channels::{
    ChannelMessage, ChannelPorts, ChannelSet, ControlView, HeartbeatView, IOPubMessage, IOPubView,
    MessageAdapter, ShellView, StdinView,
};
pub use client::{ClientError, ProtocolClient};
pub use debug_bridge::{
    DebugBridge, DebugConfig, DebugMode, DebugPerformanceMonitor, DebugSession, DebugSessionState,
    LocalDebugConfig, PerformanceConfig, ProtocolDebugConfig,
};
pub use engine::{
    Capability, ChannelType, ChannelView, EngineError, MessageContent, MessageRouter,
    ProtocolAdapter, ProtocolEngine, ProtocolType, UnifiedProtocolEngine, UniversalMessage,
};
pub use processor::{MessageProcessor, NullMessageProcessor, ProcessorError};
pub use protocol::{
    HelpLink, HistoryEntry, LDPRequest, LDPResponse, LRPCodec, LRPRequest, LRPResponse,
    LanguageInfo, MessageHandler, MessageType, ProtocolMessage, Source,
};
pub use server::{ServerConfig, ServerError};
pub use transport::{Transport, TransportError};

// Re-export types module for temporary backward compatibility
// This will be removed once all imports are updated
pub mod types {
    pub use crate::protocol::ldp::*;
    pub use crate::protocol::lrp::*;
}

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
        // Test would require actual TCP setup
    }
}
