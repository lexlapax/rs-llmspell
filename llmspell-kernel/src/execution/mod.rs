//! Execution Engine Integration
//!
//! This module provides the integrated kernel execution architecture that runs
//! `ScriptRuntime` in the same context as transport, eliminating runtime isolation
//! issues that cause "dispatch task is gone" errors.

pub mod integrated;

pub use integrated::{ExecutionConfig, IOConfig, IntegratedKernel, IntegratedKernelParams};
