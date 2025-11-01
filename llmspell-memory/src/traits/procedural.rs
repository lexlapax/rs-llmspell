//! Procedural memory trait
//!
//! **Status**: Partial implementation for Phase 13.7.4 (State-Memory Synchronization).
//!
//! Procedural memory stores learned patterns and skills from repeated interactions.
//! This includes:
//! - State transition patterns (workflow sequences) - Phase 13.7.4 ✅
//! - Tool usage patterns (which tools work well together) - Phase 13.3
//! - Error recovery strategies - Phase 13.3
//! - Execution frequency tracking - Phase 13.3
//!
//! # Implementation Timeline
//!
//! - **Phase 13.7.4**: State transition pattern tracking (`record_transition`, `get_pattern_frequency`)
//! - **Phase 13.3**: Full pattern storage and retrieval
//! - **Phase 14**: Pattern learning from state transitions
//! - **Phase 15**: Skill optimization and transfer learning

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::Result;

/// Learned pattern from repeated state transitions
///
/// Represents a state transition that occurs frequently enough to be
/// considered a learned behavior (≥3 occurrences).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Pattern {
    /// State scope (e.g., "global", "session:xyz")
    pub scope: String,
    /// State key (e.g., "config.theme")
    pub key: String,
    /// Transition value (e.g., "dark")
    pub value: String,
    /// Number of times this transition occurred
    pub frequency: u32,
    /// First occurrence timestamp (milliseconds since epoch)
    pub first_seen: u64,
    /// Last occurrence timestamp (milliseconds since epoch)
    pub last_seen: u64,
}

/// Procedural memory trait
///
/// Phase 13.7.4: Partially implemented with state transition pattern tracking.
/// Full pattern storage/retrieval deferred to Phase 13.3.
#[async_trait]
pub trait ProceduralMemory: Send + Sync {
    // === Phase 13.7.4: State Transition Pattern Tracking ===

    /// Record a state transition for pattern learning
    ///
    /// Tracks `scope:key` transitions from `from_value` to `to_value`.
    /// When frequency reaches threshold (≥3), creates a learned pattern.
    ///
    /// # Arguments
    ///
    /// * `scope` - State scope (e.g., "global", "session:xyz")
    /// * `key` - State key (e.g., "config.theme")
    /// * `from_value` - Previous value (None for initial set)
    /// * `to_value` - New value
    ///
    /// # Returns
    ///
    /// New frequency count for this transition
    async fn record_transition(
        &self,
        scope: &str,
        key: &str,
        from_value: Option<&str>,
        to_value: &str,
    ) -> Result<u32>;

    /// Get frequency count for a specific state transition
    ///
    /// # Returns
    ///
    /// Number of times `scope:key → value` transition occurred
    async fn get_pattern_frequency(&self, scope: &str, key: &str, value: &str) -> Result<u32>;

    /// Get all learned patterns above minimum frequency threshold
    ///
    /// # Arguments
    ///
    /// * `min_frequency` - Minimum occurrences to be considered a pattern (typically 3)
    ///
    /// # Returns
    ///
    /// Patterns sorted by frequency (descending)
    async fn get_learned_patterns(&self, min_frequency: u32) -> Result<Vec<Pattern>>;

    // === Phase 13.3: Legacy Placeholder Methods (to be replaced) ===

    /// Placeholder: Get a pattern by ID
    ///
    /// **Status**: Deprecated, will be replaced in Phase 13.3
    async fn get_pattern(&self, id: &str) -> Result<()>;

    /// Placeholder: Store a learned pattern
    ///
    /// **Status**: Deprecated, will be replaced in Phase 13.3
    async fn store_pattern(&self, pattern_data: &str) -> Result<String>;
}
