//! # Unix Daemon Support
//!
//! This module provides Unix daemon functionality for the kernel, including:
//! - Double-fork daemonization
//! - PID file management
//! - I/O redirection to log files
//! - Signal handling for graceful shutdown
//!
//! ## Architecture
//!
//! The daemon module uses the double-fork technique to properly detach from the
//! controlling terminal and create a true background process. This is the standard
//! Unix approach for creating daemons.

pub mod logging;
pub mod manager;
pub mod pid;
pub mod shutdown;
pub mod signals;

// Re-export key types
pub use manager::{DaemonConfig, DaemonManager};
pub use pid::PidFile;
pub use shutdown::{OperationGuard, ShutdownConfig, ShutdownCoordinator, ShutdownPhase};
pub use signals::{KernelMessage, SignalBridge, SignalHandler};
