//! Core traits for the kernel
//!
//! This module contains the essential traits that define the kernel's behavior
//! and interfaces, allowing for flexible implementations while maintaining
//! clear contracts between components.

pub mod protocol;
pub mod transport;

pub use protocol::Protocol;
pub use transport::{ChannelConfig, Transport, TransportConfig, create_transport};