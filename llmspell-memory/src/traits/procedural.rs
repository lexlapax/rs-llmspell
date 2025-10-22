//! Procedural memory trait
//!
//! **Status**: Placeholder for Phase 13.3 (Procedural Memory & Pattern Learning).
//!
//! Procedural memory will store learned patterns and skills from repeated interactions.
//! This includes:
//! - State transition patterns (workflow sequences)
//! - Tool usage patterns (which tools work well together)
//! - Error recovery strategies
//! - Execution frequency tracking
//!
//! # Implementation Timeline
//!
//! - **Phase 13.3**: Basic pattern storage and retrieval
//! - **Phase 14**: Pattern learning from state transitions
//! - **Phase 15**: Skill optimization and transfer learning

use async_trait::async_trait;

use crate::error::Result;

/// Procedural memory trait (placeholder)
///
/// Will be fully implemented in Phase 13.3.
#[async_trait]
pub trait ProceduralMemory: Send + Sync {
    /// Placeholder: Get a pattern by ID
    async fn get_pattern(&self, id: &str) -> Result<()>;

    /// Placeholder: Store a learned pattern
    async fn store_pattern(&self, pattern_data: &str) -> Result<String>;
}
