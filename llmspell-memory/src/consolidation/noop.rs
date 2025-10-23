//! No-op consolidation engine (placeholder)

use async_trait::async_trait;

use super::ConsolidationEngine;
use crate::error::Result;
use crate::types::{ConsolidationResult, EpisodicEntry};

/// No-op consolidation engine stub
///
/// Does not perform any actual consolidation.
/// Used as placeholder when consolidation is disabled.
#[derive(Debug, Clone, Copy)]
pub struct NoopConsolidationEngine;

impl NoopConsolidationEngine {
    /// Create new no-op consolidation engine
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Default for NoopConsolidationEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ConsolidationEngine for NoopConsolidationEngine {
    async fn consolidate(
        &self,
        _session_ids: &[&str],
        _entries: &mut [EpisodicEntry],
    ) -> Result<ConsolidationResult> {
        Ok(ConsolidationResult {
            entries_processed: 0,
            entities_added: 0,
            entities_updated: 0,
            entities_deleted: 0,
            entries_skipped: 0,
            entries_failed: 0,
            duration_ms: 0,
        })
    }

    fn is_ready(&self) -> bool {
        false // No-op engine is never "ready" for real consolidation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_noop_consolidation() {
        let engine = NoopConsolidationEngine::new();

        let mut entries = vec![EpisodicEntry::new(
            "session-1".to_string(),
            "user".to_string(),
            "test content".to_string(),
        )];

        let result = engine
            .consolidate(&["session-1"], &mut entries)
            .await
            .unwrap();

        assert_eq!(result.entries_processed, 0);
        assert_eq!(result.entities_added, 0);
        assert!(!engine.is_ready());
    }
}
