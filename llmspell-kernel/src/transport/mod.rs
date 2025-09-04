//! Transport layer for Jupyter protocol
//!
//! Handles ZeroMQ socket management and message serialization for the 5 Jupyter channels:
//! - Shell: Execute requests and replies
//! - IOPub: Output publishing and status updates
//! - Stdin: Input requests from kernel to frontend 
//! - Control: Shutdown, interrupt, and daemon management
//! - Heartbeat: Keepalive mechanism

pub mod zeromq;
pub mod heartbeat;

pub use zeromq::ZmqTransport;