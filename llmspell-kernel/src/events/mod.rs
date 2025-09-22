//! Kernel Event Correlation System
//!
//! This module implements comprehensive event correlation for kernel operations,
//! integrating execution events, debug events, session events, and providing
//! `IOPub` broadcasting for multi-client support with distributed tracing.

pub mod correlation;

pub use correlation::{EventBroadcaster, KernelEvent, KernelEventCorrelator};
