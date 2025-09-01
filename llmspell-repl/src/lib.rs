//! `LLMSpell` REPL - Kernel-as-service with debugging infrastructure
//!
//! This crate provides a kernel service following Jupyter's proven multi-client architecture,
//! enabling interactive REPL sessions, debugging capabilities, and multi-client support
//! for the `LLMSpell` scripting platform.

pub mod client; // Client connection handling
pub mod connection; // Connection management
pub mod discovery; // Connection file discovery
pub mod kernel; // Core kernel service
pub mod protocol; // LRP/LDP protocol definitions
                  // protocol_handler removed - functionality moved to MessageProcessor in kernel
pub mod security; // Authentication and authorization

// Re-export main types for convenience
pub use connection::ConnectionInfo;
pub use kernel::LLMSpellKernel;
pub use protocol::{LDPRequest, LDPResponse, LRPRequest, LRPResponse};
