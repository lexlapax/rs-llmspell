//! I/O Management and Message Routing
//!
//! This module provides multi-channel I/O routing for the kernel,
//! handling stdout/stderr capture and routing to the appropriate
//! Jupyter channels with parent header tracking for message correlation.

pub mod manager;
pub mod router;

pub use manager::{EnhancedIOManager, IOConfig};
pub use router::{MessageRouter, MessageDestination};