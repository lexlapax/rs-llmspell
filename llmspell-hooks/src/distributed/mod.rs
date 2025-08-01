// ABOUTME: Distributed hook system for future A2A protocol support (Phase 16-17 prep)
// ABOUTME: Provides DistributedHookContext for cross-network hook propagation and coordination

//! # Distributed Hook System
//!
//! This module provides the foundation for distributed hook execution across
//! multiple agents in a network. It's designed as preparation for Phase 16-17
//! A2A (Agent-to-Agent) protocol support.
//!
//! ## Features
//!
//! - **DistributedHookContext**: Extended context for network-aware hooks
//! - **Remote Agent Identification**: Track source and target agents
//! - **Propagation Control**: Flags to control cross-network propagation
//! - **Security**: Built-in security considerations for remote execution
//! - **Correlation**: Network-wide correlation tracking
//!
//! ## Example
//!
//! ```rust,no_run
//! use llmspell_hooks::distributed::{DistributedHookContext, PropagationFlags, RemoteAgentId};
//! use llmspell_hooks::{HookContext, HookPoint, ComponentId, ComponentType};
//! use uuid::Uuid;
//!
//! // Create a distributed context
//! let base_context = HookContext::new(
//!     HookPoint::BeforeAgentExecution,
//!     ComponentId::new(ComponentType::Agent, "local-agent".to_string())
//! );
//!
//! let distributed_context = DistributedHookContext::from_local(base_context)
//!     .with_source_agent(RemoteAgentId::new("node-1", "agent-1"))
//!     .with_propagation_flags(PropagationFlags::default().with_broadcast(true));
//! ```

mod context;

pub use context::{
    DistributedHookContext, DistributedHookContextBuilder, PropagationFlags, RemoteAgentId,
    SecurityContext,
};

// Version information for distributed protocol
pub const DISTRIBUTED_PROTOCOL_VERSION: &str = "0.1.0";

#[cfg(test)]
#[cfg_attr(test_category = "hook")]
mod tests {
    use super::*;

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_module_exports() {
        // Verify exports are accessible
        let _flags = PropagationFlags::default();
        let _agent_id = RemoteAgentId::new("test-node", "test-agent");
        assert_eq!(DISTRIBUTED_PROTOCOL_VERSION, "0.1.0");
    }
}
