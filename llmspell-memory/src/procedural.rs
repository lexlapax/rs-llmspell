//! Procedural memory implementations (learned patterns and skills)
//!
//! Phase 13.7.4: Pattern tracking via state transitions
//! Future: Full pattern learning (Phase 13.3+)

use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::RwLock;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::Result;
use crate::traits::{Pattern, ProceduralMemory};

/// In-memory pattern tracker for state transition learning
///
/// Tracks state transition frequencies and creates learned patterns
/// when transitions occur ≥3 times (threshold).
pub struct InMemoryPatternTracker {
    /// Transition frequency counter: (scope, key, value) → (frequency, `first_seen`, `last_seen`)
    patterns: RwLock<HashMap<String, (u32, u64, u64)>>,
}

impl InMemoryPatternTracker {
    /// Create new in-memory pattern tracker
    #[must_use]
    pub fn new() -> Self {
        Self {
            patterns: RwLock::new(HashMap::new()),
        }
    }

    /// Generate pattern key from scope, key, value
    /// Uses '|' as delimiter to support scopes with colons (e.g., "session:test-session")
    fn pattern_key(scope: &str, key: &str, value: &str) -> String {
        format!("{scope}|{key}|{value}")
    }

    /// Get current timestamp in milliseconds since epoch
    #[allow(clippy::cast_possible_truncation)]
    fn now_millis() -> u64 {
        // Cast is acceptable: u128 → u64 milliseconds won't overflow until year 584,942,417 AD
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("System time before UNIX epoch")
            .as_millis() as u64
    }
}

impl Default for InMemoryPatternTracker {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ProceduralMemory for InMemoryPatternTracker {
    async fn record_transition(
        &self,
        scope: &str,
        key: &str,
        _from_value: Option<&str>,
        to_value: &str,
    ) -> Result<u32> {
        let pattern_key = Self::pattern_key(scope, key, to_value);
        let now = Self::now_millis();

        let mut patterns = self.patterns.write().expect("RwLock poisoned");
        let entry = patterns.entry(pattern_key).or_insert((0, now, now));

        entry.0 += 1; // Increment frequency
        entry.2 = now; // Update last_seen
        let freq = entry.0;

        drop(patterns); // Explicitly release lock

        Ok(freq)
    }

    async fn get_pattern_frequency(&self, scope: &str, key: &str, value: &str) -> Result<u32> {
        let pattern_key = Self::pattern_key(scope, key, value);
        let freq = {
            let patterns = self.patterns.read().expect("RwLock poisoned");
            patterns.get(&pattern_key).map_or(0, |(freq, _, _)| *freq)
        };
        Ok(freq)
    }

    async fn get_learned_patterns(&self, min_frequency: u32) -> Result<Vec<Pattern>> {
        let mut patterns: Vec<Pattern> = {
            let patterns_guard = self.patterns.read().expect("RwLock poisoned");
            patterns_guard
                .iter()
                .filter(|(_, (freq, _, _))| *freq >= min_frequency)
                .map(|(key, (freq, first, last))| {
                    // Parse "scope|key|value" back into components
                    let parts: Vec<&str> = key.splitn(3, '|').collect();
                    Pattern {
                        scope: parts[0].to_string(),
                        key: parts[1].to_string(),
                        value: parts[2].to_string(),
                        frequency: *freq,
                        first_seen: *first,
                        last_seen: *last,
                    }
                })
                .collect()
        };

        // Sort by frequency descending
        patterns.sort_by(|a, b| b.frequency.cmp(&a.frequency));

        Ok(patterns)
    }

    // Legacy placeholder methods (Phase 13.3)
    async fn get_pattern(&self, _id: &str) -> Result<()> {
        Ok(())
    }

    async fn store_pattern(&self, _pattern_data: &str) -> Result<String> {
        Ok(String::new())
    }
}

/// No-op procedural memory stub
///
/// Placeholder for contexts that don't need pattern tracking.
pub struct NoopProceduralMemory;

#[async_trait]
impl ProceduralMemory for NoopProceduralMemory {
    async fn record_transition(
        &self,
        _scope: &str,
        _key: &str,
        _from_value: Option<&str>,
        _to_value: &str,
    ) -> Result<u32> {
        Ok(0)
    }

    async fn get_pattern_frequency(&self, _scope: &str, _key: &str, _value: &str) -> Result<u32> {
        Ok(0)
    }

    async fn get_learned_patterns(&self, _min_frequency: u32) -> Result<Vec<Pattern>> {
        Ok(Vec::new())
    }

    async fn get_pattern(&self, _id: &str) -> Result<()> {
        Ok(())
    }

    async fn store_pattern(&self, _pattern_data: &str) -> Result<String> {
        Ok(String::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pattern_tracking() {
        let tracker = InMemoryPatternTracker::new();

        // Record same transition 3 times
        let freq1 = tracker
            .record_transition("global", "theme", None, "dark")
            .await
            .unwrap();
        assert_eq!(freq1, 1);

        let freq2 = tracker
            .record_transition("global", "theme", Some("light"), "dark")
            .await
            .unwrap();
        assert_eq!(freq2, 2);

        let freq3 = tracker
            .record_transition("global", "theme", Some("light"), "dark")
            .await
            .unwrap();
        assert_eq!(freq3, 3);

        // Verify frequency
        let freq = tracker
            .get_pattern_frequency("global", "theme", "dark")
            .await
            .unwrap();
        assert_eq!(freq, 3);
    }

    #[tokio::test]
    async fn test_learned_patterns_threshold() {
        let tracker = InMemoryPatternTracker::new();

        // Record transitions with different frequencies
        for _ in 0..5 {
            tracker
                .record_transition("global", "theme", None, "dark")
                .await
                .unwrap();
        }
        for _ in 0..2 {
            tracker
                .record_transition("global", "lang", None, "rust")
                .await
                .unwrap();
        }
        tracker
            .record_transition("session:x", "state", None, "active")
            .await
            .unwrap();

        // Get patterns with threshold ≥3
        let patterns = tracker.get_learned_patterns(3).await.unwrap();
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].key, "theme");
        assert_eq!(patterns[0].value, "dark");
        assert_eq!(patterns[0].frequency, 5);
    }

    #[tokio::test]
    async fn test_pattern_sorting() {
        let tracker = InMemoryPatternTracker::new();

        for _ in 0..3 {
            tracker
                .record_transition("global", "a", None, "1")
                .await
                .unwrap();
        }
        for _ in 0..7 {
            tracker
                .record_transition("global", "b", None, "2")
                .await
                .unwrap();
        }
        for _ in 0..5 {
            tracker
                .record_transition("global", "c", None, "3")
                .await
                .unwrap();
        }

        let patterns = tracker.get_learned_patterns(3).await.unwrap();
        assert_eq!(patterns.len(), 3);
        assert_eq!(patterns[0].frequency, 7); // b
        assert_eq!(patterns[1].frequency, 5); // c
        assert_eq!(patterns[2].frequency, 3); // a
    }
}
