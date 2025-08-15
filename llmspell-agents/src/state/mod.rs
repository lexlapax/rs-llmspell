//! State management module for agent isolation and sharing
//!
//! Provides secure multi-agent state isolation with controlled sharing patterns.
//! This module ensures that agents can maintain private state while also supporting
//! controlled data sharing between agents based on defined permissions and boundaries.
//!
//! # Overview
//!
//! The state management system provides:
//! - **Isolation**: Each agent has its own isolated state space
//! - **Controlled Sharing**: Explicit permissions for state sharing between agents
//! - **Persistence**: Optional state persistence to storage backends
//! - **Auditing**: Complete audit trail of state access and modifications
//!
//! # Examples
//!
//! ```ignore
//! use llmspell_agents::state::{StateIsolationManager, StatePermission};
//!
//! // Create a state manager
//! let manager = StateIsolationManager::new();
//!
//! // Set agent state with isolation
//! manager.set_state("agent-1", "key", value).await?;
//!
//! // Grant read permission to another agent
//! manager.grant_permission("agent-1", "agent-2", StatePermission::Read);
//! ```

pub mod isolation;
pub mod persistence;
pub mod sharing;

pub use isolation::{
    IsolatedStateAccessor, IsolatedStateAgent, IsolationBoundary, SharedScopeConfig,
    StateAccessAudit, StateAccessControl, StateIsolationManager, StateOperation, StatePermission,
    StateScope,
};

pub use sharing::{
    SharedStateAccessor, SharedStateAgent, SharedStateChannel, SharingPattern, StateMessage,
    StateSharingManager,
};

pub use persistence::{StateManagerHolder, StatePersistence, ToolStats};
