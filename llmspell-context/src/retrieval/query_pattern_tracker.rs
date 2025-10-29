//! ABOUTME: Query pattern tracking for consolidation priority
//! ABOUTME: Tracks retrieval frequency of episodic entries to inform consolidation priority

use std::collections::HashMap;
use std::sync::RwLock;
use tracing::{debug, info};

/// Tracks query patterns to inform consolidation priority
///
/// Records how frequently episodic entries are retrieved during hybrid searches.
/// Frequently-retrieved entries are marked as high-value candidates for
/// consolidation to semantic memory.
///
/// # Thread Safety
///
/// Uses `RwLock` for concurrent access from multiple retrieval threads.
///
/// # Example
///
/// ```ignore
/// let tracker = QueryPatternTracker::new();
///
/// // Record retrievals
/// tracker.record_retrieval(&["entry-1", "entry-2"]);
/// tracker.record_retrieval(&["entry-1"]); // entry-1 now count=2
///
/// // Get consolidation candidates (min 2 retrievals)
/// let candidates = tracker.get_consolidation_candidates(2);
/// assert_eq!(candidates, vec!["entry-1"]);
/// ```
#[derive(Debug)]
pub struct QueryPatternTracker {
    /// Maps `entry_id` → retrieval count
    retrieval_counts: RwLock<HashMap<String, usize>>,
}

impl Default for QueryPatternTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl QueryPatternTracker {
    /// Create a new query pattern tracker
    #[must_use]
    pub fn new() -> Self {
        debug!("Created QueryPatternTracker");
        Self {
            retrieval_counts: RwLock::new(HashMap::new()),
        }
    }

    /// Record retrieval of episodic entries
    ///
    /// Increments the retrieval count for each entry ID.
    ///
    /// # Arguments
    ///
    /// * `entry_ids` - Slice of entry IDs that were retrieved
    ///
    /// # Panics
    ///
    /// Panics if the `RwLock` is poisoned (should never happen in practice)
    ///
    /// # Example
    ///
    /// ```ignore
    /// tracker.record_retrieval(&["entry-1", "entry-2", "entry-1"]);
    /// // entry-1: count=2, entry-2: count=1
    /// ```
    pub fn record_retrieval(&self, entry_ids: &[String]) {
        if entry_ids.is_empty() {
            return;
        }

        let mut counts = self.retrieval_counts.write().unwrap();
        for id in entry_ids {
            *counts.entry(id.clone()).or_insert(0) += 1;
        }
        let total_tracked = counts.len();
        drop(counts);

        debug!(
            "Recorded {} entry retrievals (total tracked: {})",
            entry_ids.len(),
            total_tracked
        );
    }

    /// Get consolidation candidates based on retrieval frequency
    ///
    /// Returns entry IDs that have been retrieved at least `min_retrievals` times.
    /// These are high-value candidates for consolidation to semantic memory.
    ///
    /// # Arguments
    ///
    /// * `min_retrievals` - Minimum number of retrievals to qualify as candidate
    ///
    /// # Returns
    ///
    /// Vector of entry IDs sorted by retrieval count (descending)
    ///
    /// # Panics
    ///
    /// Panics if the `RwLock` is poisoned (should never happen in practice)
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Get entries retrieved 5+ times
    /// let candidates = tracker.get_consolidation_candidates(5);
    /// ```
    pub fn get_consolidation_candidates(&self, min_retrievals: usize) -> Vec<String> {
        let counts = self.retrieval_counts.read().unwrap();

        let mut candidates: Vec<_> = counts
            .iter()
            .filter(|(_, count)| **count >= min_retrievals)
            .map(|(id, count)| (id.clone(), *count))
            .collect();

        // Sort by count descending (highest frequency first)
        candidates.sort_by(|a, b| b.1.cmp(&a.1));

        let total_tracked = counts.len();
        drop(counts);

        info!(
            "Found {} consolidation candidates (min_retrievals={}, total_tracked={})",
            candidates.len(),
            min_retrievals,
            total_tracked
        );

        candidates.into_iter().map(|(id, _)| id).collect()
    }

    /// Get current retrieval count for an entry
    ///
    /// # Arguments
    ///
    /// * `entry_id` - Entry ID to query
    ///
    /// # Returns
    ///
    /// Current retrieval count, or 0 if never retrieved
    ///
    /// # Panics
    ///
    /// Panics if the `RwLock` is poisoned (should never happen in practice)
    #[must_use]
    pub fn get_count(&self, entry_id: &str) -> usize {
        let counts = self.retrieval_counts.read().unwrap();
        counts.get(entry_id).copied().unwrap_or(0)
    }

    /// Clear all tracking data
    ///
    /// Resets all retrieval counts to zero.
    ///
    /// # Panics
    ///
    /// Panics if the `RwLock` is poisoned (should never happen in practice)
    pub fn clear(&self) {
        self.retrieval_counts.write().unwrap().clear();
        debug!("Cleared all query pattern tracking data");
    }

    /// Get total number of tracked entries
    ///
    /// # Panics
    ///
    /// Panics if the `RwLock` is poisoned (should never happen in practice)
    #[must_use]
    pub fn tracked_count(&self) -> usize {
        let counts = self.retrieval_counts.read().unwrap();
        counts.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_tracker() {
        let tracker = QueryPatternTracker::new();
        assert_eq!(tracker.tracked_count(), 0);
    }

    #[test]
    fn test_record_retrieval() {
        let tracker = QueryPatternTracker::new();

        tracker.record_retrieval(&["entry-1".to_string(), "entry-2".to_string()]);
        assert_eq!(tracker.get_count("entry-1"), 1);
        assert_eq!(tracker.get_count("entry-2"), 1);

        tracker.record_retrieval(&["entry-1".to_string()]);
        assert_eq!(tracker.get_count("entry-1"), 2);
        assert_eq!(tracker.get_count("entry-2"), 1);
    }

    #[test]
    fn test_record_retrieval_empty() {
        let tracker = QueryPatternTracker::new();
        tracker.record_retrieval(&[]);
        assert_eq!(tracker.tracked_count(), 0);
    }

    #[test]
    fn test_get_consolidation_candidates() {
        let tracker = QueryPatternTracker::new();

        // Record entries with different frequencies
        tracker.record_retrieval(&["entry-1".to_string()]); // count=1
        tracker.record_retrieval(&["entry-1".to_string()]); // count=2
        tracker.record_retrieval(&["entry-2".to_string()]); // count=1
        tracker.record_retrieval(&["entry-3".to_string()]); // count=1
        tracker.record_retrieval(&["entry-3".to_string()]); // count=2
        tracker.record_retrieval(&["entry-3".to_string()]); // count=3

        // Get candidates with min 2 retrievals
        let candidates = tracker.get_consolidation_candidates(2);
        assert_eq!(candidates.len(), 2);
        assert!(candidates.contains(&"entry-1".to_string()));
        assert!(candidates.contains(&"entry-3".to_string()));

        // Get candidates with min 3 retrievals
        let candidates = tracker.get_consolidation_candidates(3);
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0], "entry-3");
    }

    #[test]
    fn test_get_consolidation_candidates_sorted() {
        let tracker = QueryPatternTracker::new();

        // Record entries with different frequencies
        for _ in 0..5 {
            tracker.record_retrieval(&["entry-1".to_string()]);
        }
        for _ in 0..3 {
            tracker.record_retrieval(&["entry-2".to_string()]);
        }
        for _ in 0..7 {
            tracker.record_retrieval(&["entry-3".to_string()]);
        }

        // Should be sorted by count descending
        let candidates = tracker.get_consolidation_candidates(1);
        assert_eq!(candidates[0], "entry-3"); // 7 retrievals
        assert_eq!(candidates[1], "entry-1"); // 5 retrievals
        assert_eq!(candidates[2], "entry-2"); // 3 retrievals
    }

    #[test]
    fn test_clear() {
        let tracker = QueryPatternTracker::new();

        tracker.record_retrieval(&["entry-1".to_string(), "entry-2".to_string()]);
        assert_eq!(tracker.tracked_count(), 2);

        tracker.clear();
        assert_eq!(tracker.tracked_count(), 0);
        assert_eq!(tracker.get_count("entry-1"), 0);
    }

    #[test]
    fn test_get_count_nonexistent() {
        let tracker = QueryPatternTracker::new();
        assert_eq!(tracker.get_count("nonexistent"), 0);
    }
}
