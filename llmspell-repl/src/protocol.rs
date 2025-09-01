//! Re-export protocol types from llmspell-protocol
//!
//! This module re-exports the protocol types for backward compatibility

pub use llmspell_protocol::{LDPRequest, LDPResponse, LRPRequest, LRPResponse};

// Re-export supporting types
pub use llmspell_protocol::types::{HelpLink, HistoryEntry, LanguageInfo};
