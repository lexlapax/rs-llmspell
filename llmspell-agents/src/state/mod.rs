// ABOUTME: State management module for agent isolation and sharing
// ABOUTME: Provides secure multi-agent state isolation with controlled sharing patterns

pub mod isolation;
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
