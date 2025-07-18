//! ABOUTME: Context management module for hierarchical contexts and inheritance
//! ABOUTME: Provides hierarchy, inheritance, shared memory, event integration, and distributed sync

pub mod distributed;
pub mod event_integration;
pub mod hierarchy;
pub mod inheritance;
pub mod shared_memory;

// Re-export key types
pub use distributed::{ContextSync, DistributedContext};
pub use event_integration::{ContextEvent, ContextEventBus};
pub use hierarchy::{ContextNode, HierarchicalContext};
pub use inheritance::{FieldInheritance, InheritanceRules};
pub use shared_memory::{MemoryRegion, SharedMemoryManager};
