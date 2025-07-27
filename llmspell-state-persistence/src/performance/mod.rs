// ABOUTME: Performance optimization module for state persistence
// ABOUTME: Provides tiered performance architecture with fast paths for different data classes

pub mod fast_path;
pub mod lockfree_agent;
pub mod state_class;

pub use fast_path::{FastPathConfig, FastPathManager};
pub use lockfree_agent::{FastAgentStateOps, LockFreeAgentStore};
pub use state_class::StateClass;
