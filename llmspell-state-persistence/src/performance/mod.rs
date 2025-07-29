// ABOUTME: Performance optimization module for state persistence
// ABOUTME: Provides tiered performance architecture with fast paths for different data classes

pub mod async_hooks;
pub mod fast_path;
pub mod lockfree_agent;
pub mod state_class;
pub mod unified_serialization;

pub use async_hooks::{
    AsyncHookProcessor, HookBatcher, HookEvent, HookEventType, HookProcessorStatsSnapshot,
};
pub use fast_path::{FastPathConfig, FastPathManager};
pub use lockfree_agent::{FastAgentStateOps, LockFreeAgentStore};
pub use state_class::StateClass;
pub use unified_serialization::{StreamingSerializer, UnifiedSerializer, UnifiedSerializerBuilder};
