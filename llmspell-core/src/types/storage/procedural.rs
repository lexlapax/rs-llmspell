//! Procedural memory types
//!
//! Domain types for state transition pattern learning including:
//! - `Pattern`: Learned state transition pattern with frequency tracking
//!
//! # Procedural Memory
//!
//! Tracks repeated state transitions to identify learned behaviors.
//! When a transition occurs frequently (≥3 times), it becomes a Pattern.
//!
//! Migrated from llmspell-memory/src/traits/procedural.rs as part of Phase 13c.3.

use serde::{Deserialize, Serialize};

/// Learned pattern from repeated state transitions
///
/// Represents a state transition that occurs frequently enough to be
/// considered a learned behavior (≥3 occurrences by default).
///
/// # Pattern Learning
///
/// Patterns are automatically created when:
/// 1. A state transition `scope:key → value` occurs
/// 2. The same transition has occurred ≥ `min_frequency` times (typically 3)
/// 3. The pattern is tracked with first/last occurrence timestamps
///
/// # Examples
///
/// ```
/// # use llmspell_core::types::storage::procedural::Pattern;
/// // Pattern for user preference: dark mode enabled 5 times
/// let pattern = Pattern {
///     scope: "user:alice".to_string(),
///     key: "theme".to_string(),
///     value: "dark".to_string(),
///     frequency: 5,
///     first_seen: 1704067200000,  // 2024-01-01 00:00:00 UTC
///     last_seen: 1704153600000,   // 2024-01-02 00:00:00 UTC
/// };
///
/// assert_eq!(pattern.scope, "user:alice");
/// assert_eq!(pattern.frequency, 5);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Pattern {
    /// State scope (e.g., "global", "session:xyz", "user:alice")
    pub scope: String,

    /// State key (e.g., "config.theme", "ui.layout")
    pub key: String,

    /// Transition value (e.g., "dark", "grid")
    pub value: String,

    /// Number of times this transition occurred
    pub frequency: u32,

    /// First occurrence timestamp (milliseconds since Unix epoch)
    pub first_seen: u64,

    /// Last occurrence timestamp (milliseconds since Unix epoch)
    pub last_seen: u64,
}
