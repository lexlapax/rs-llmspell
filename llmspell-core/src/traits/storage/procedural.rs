//! Procedural memory trait for state transition pattern learning
//!
//! Defines the core abstraction for learning and tracking repeated state transitions.
//! When transitions occur frequently (≥ threshold), they become learned patterns.
//!
//! # Pattern Learning Workflow
//!
//! 1. **Record transitions**: `record_transition()` tracks state changes
//! 2. **Automatic pattern detection**: When frequency ≥ 3, creates Pattern
//! 3. **Query patterns**: `get_learned_patterns()` retrieves high-frequency transitions
//!
//! # Implementation Status
//!
//! - **Phase 13.7.4**: State transition tracking implemented
//! - **Phase 13c.3**: Full pattern storage/retrieval (this migration)
//!
//! Migrated from llmspell-memory/src/traits/procedural.rs as part of Phase 13c.3.

use crate::types::storage::procedural::Pattern;
use anyhow::Result;
use async_trait::async_trait;

/// Procedural memory trait for learning state transition patterns
///
/// Tracks repeated state transitions to identify learned behaviors. When a specific
/// state transition occurs frequently enough (≥ minimum frequency threshold), it is
/// recorded as a learned Pattern.
///
/// # Pattern Detection
///
/// Patterns are created automatically when:
/// - A state transition `scope:key → value` occurs
/// - The same transition has occurred ≥ `min_frequency` times (default: 3)
/// - Timestamps track first and last occurrence
///
/// # Examples
///
/// ```no_run
/// # use llmspell_core::traits::storage::ProceduralMemory;
/// # async fn example(memory: impl ProceduralMemory) -> anyhow::Result<()> {
/// // Record user setting theme to dark mode
/// let freq = memory.record_transition(
///     "user:alice",
///     "theme",
///     Some("light"),  // Previous value
///     "dark"          // New value
/// ).await?;
///
/// // After 3+ transitions, query learned patterns
/// let patterns = memory.get_learned_patterns(3).await?;
/// for pattern in patterns {
///     println!("Learned: {}:{} → {} ({} times)",
///         pattern.scope, pattern.key, pattern.value, pattern.frequency);
/// }
/// # Ok(())
/// # }
/// ```
#[async_trait]
pub trait ProceduralMemory: Send + Sync {
    /// Record a state transition for pattern learning
    ///
    /// Tracks `scope:key` transitions from `from_value` to `to_value`.
    /// When frequency reaches threshold (≥3 by default), creates a learned pattern.
    ///
    /// # Arguments
    ///
    /// * `scope` - State scope (e.g., "global", "session:xyz", "user:alice")
    /// * `key` - State key (e.g., "config.theme", "ui.layout")
    /// * `from_value` - Previous value (None for initial set)
    /// * `to_value` - New value
    ///
    /// # Returns
    ///
    /// New frequency count for this transition (increments with each call)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // First time user sets dark mode
    /// let freq1 = memory.record_transition("user:alice", "theme", None, "dark").await?;
    /// assert_eq!(freq1, 1);
    ///
    /// // User toggles back to light, then dark again
    /// memory.record_transition("user:alice", "theme", Some("dark"), "light").await?;
    /// let freq3 = memory.record_transition("user:alice", "theme", Some("light"), "dark").await?;
    /// assert_eq!(freq3, 2);  // Second time transitioning to dark
    /// ```
    async fn record_transition(
        &self,
        scope: &str,
        key: &str,
        from_value: Option<&str>,
        to_value: &str,
    ) -> Result<u32>;

    /// Get frequency count for a specific state transition
    ///
    /// Returns how many times the `scope:key → value` transition has occurred.
    ///
    /// # Arguments
    ///
    /// * `scope` - State scope to query
    /// * `key` - State key to query
    /// * `value` - Target value to check frequency for
    ///
    /// # Returns
    ///
    /// Number of times this exact transition occurred (0 if never)
    ///
    /// # Examples
    ///
    /// ```ignore
    /// let freq = memory.get_pattern_frequency("user:alice", "theme", "dark").await?;
    /// if freq >= 3 {
    ///     println!("User prefers dark mode (occurred {} times)", freq);
    /// }
    /// ```
    async fn get_pattern_frequency(&self, scope: &str, key: &str, value: &str) -> Result<u32>;

    /// Get all learned patterns above minimum frequency threshold
    ///
    /// Returns patterns sorted by frequency (descending), enabling identification
    /// of the most common behaviors.
    ///
    /// # Arguments
    ///
    /// * `min_frequency` - Minimum occurrences to be considered a pattern (typically 3)
    ///
    /// # Returns
    ///
    /// Patterns sorted by frequency (highest first), all with `frequency ≥ min_frequency`
    ///
    /// # Examples
    ///
    /// ```ignore
    /// // Get all patterns that occurred at least 3 times
    /// let patterns = memory.get_learned_patterns(3).await?;
    ///
    /// // Analyze most common behaviors
    /// for (i, pattern) in patterns.iter().take(10).enumerate() {
    ///     println!("{}. {}:{} → {} ({} times)",
    ///         i + 1, pattern.scope, pattern.key, pattern.value, pattern.frequency);
    /// }
    /// ```
    async fn get_learned_patterns(&self, min_frequency: u32) -> Result<Vec<Pattern>>;

    /// Get a pattern by unique identifier
    ///
    /// # Status
    ///
    /// **Placeholder**: Will be fully implemented in Phase 13.3 with pattern IDs.
    /// Currently returns empty result.
    ///
    /// # Arguments
    ///
    /// * `id` - Pattern unique identifier
    ///
    /// # Returns
    ///
    /// Pattern if found (currently unimplemented)
    async fn get_pattern(&self, id: &str) -> Result<()>;

    /// Store a learned pattern explicitly
    ///
    /// # Status
    ///
    /// **Placeholder**: Will be fully implemented in Phase 13.3 with explicit pattern storage.
    /// Currently patterns are created automatically via `record_transition()`.
    ///
    /// # Arguments
    ///
    /// * `pattern_data` - Serialized pattern data
    ///
    /// # Returns
    ///
    /// Pattern ID if stored (currently unimplemented)
    async fn store_pattern(&self, pattern_data: &str) -> Result<String>;
}
