//! Protocol implementations for kernel communication
//!
//! This module provides the Jupyter protocol implementation
//! which is the ONLY protocol used by the kernel.

pub mod jupyter;
pub mod registry;

pub use jupyter::JupyterProtocol;
pub use registry::{ProtocolFactory, ProtocolRegistry};

/// Protocol configuration for creating protocol instances
#[derive(Debug, Clone)]
pub struct ProtocolConfig {
    /// Session identifier for this protocol instance
    pub session_id: String,
    /// Kernel identifier for tracking
    pub kernel_id: String,
    /// Optional port for network protocols
    pub port: Option<u16>,
    /// Optional connection file path for Jupyter protocols
    pub connection_file: Option<String>,
}
