//! ABOUTME: llmspell-security implementation crate
//! ABOUTME: Security sandbox for safe tool execution with file, network, and resource controls

pub mod access_control;
pub mod audit;
pub mod sandbox;

// Re-export main types
pub use access_control::{
    AccessDecision, OperationContext, SecurityFilter, SecurityPolicy, VectorAccessPolicy,
    VectorSecurityManager,
};
pub use audit::{AuditEntry, AuditEvent, AuditLogger};
pub use sandbox::{
    FileSandbox, IntegratedSandbox, NetworkSandbox, ResourceMonitor, SandboxContext,
    SandboxViolation,
};
