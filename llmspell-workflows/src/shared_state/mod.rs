//! ABOUTME: State management infrastructure for workflows (preparing for Phase 5)
//! ABOUTME: Provides thread-safe shared state access between workflow steps

pub mod builder;
pub mod shared;
pub mod types;

pub use builder::StateBuilder;
pub use shared::{WorkflowStateAccessor, WorkflowStateManager};
pub use types::{StateAccess, StateEntry, StateScope};
