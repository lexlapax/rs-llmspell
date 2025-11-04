//! ABOUTME: PostgreSQL procedural pattern storage (Phase 13b.6.2)
//! ABOUTME: Storage layer for state transition pattern tracking with tenant isolation

use super::backend::PostgresBackend;
use super::error::{PostgresError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Learned pattern from repeated state transitions
///
/// Matches the Pattern struct from llmspell-memory but defined here to avoid circular dependency.
/// The memory layer will convert between these types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StoredPattern {
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

/// PostgreSQL-backed procedural pattern storage
///
/// Stores learned patterns from repeated state transitions with:
/// - Tenant isolation via RLS
/// - Unique constraint on (tenant_id, scope, key, value)
/// - Frequency tracking with first_seen/last_seen timestamps
/// - Performance indexes for pattern queries
///
/// # Performance Target
/// <10ms for pattern queries (Task 13b.6.2)
///
/// # Architecture
/// This is a storage layer struct. The memory layer (`llmspell-memory`) wraps this
/// to implement the `ProceduralMemory` trait, following the same pattern as:
/// - `PostgreSQLVectorStorage` → `PostgreSQLEpisodicMemory`
/// - `PostgresGraphStorage` → `GraphSemanticMemory`
#[derive(Debug, Clone)]
pub struct PostgresProceduralStorage {
    backend: Arc<PostgresBackend>,
}

impl PostgresProceduralStorage {
    /// Create new PostgreSQL procedural pattern storage
    ///
    /// # Arguments
    /// * `backend` - PostgreSQL backend with connection pool
    ///
    /// # Example
    /// ```rust,no_run
    /// use llmspell_storage::backends::postgres::{PostgresBackend, PostgresConfig};
    /// use llmspell_storage::backends::postgres::procedural::PostgresProceduralStorage;
    /// use std::sync::Arc;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = PostgresConfig::new("postgresql://localhost/llmspell");
    /// let backend = Arc::new(PostgresBackend::new(config).await?);
    /// let storage = PostgresProceduralStorage::new(backend);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(backend: Arc<PostgresBackend>) -> Self {
        Self { backend }
    }

    /// Convert database timestamp to milliseconds since epoch
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    fn datetime_to_millis(dt: DateTime<Utc>) -> u64 {
        // Cast is acceptable: i64 seconds won't overflow u64 until year ~584 billion AD
        // after converting to milliseconds
        (dt.timestamp() * 1000 + i64::from(dt.timestamp_subsec_millis())) as u64
    }

    /// Record a state transition for pattern learning
    ///
    /// Tracks `scope:key` transitions to `to_value`.
    /// When frequency reaches threshold (≥3), pattern becomes learned.
    ///
    /// # Arguments
    /// * `scope` - State scope (e.g., "global", "session:xyz")
    /// * `key` - State key (e.g., "config.theme")
    /// * `to_value` - New value
    ///
    /// # Returns
    /// New frequency count for this transition
    pub async fn record_transition(
        &self,
        scope: &str,
        key: &str,
        to_value: &str,
    ) -> Result<u32> {
        // Get tenant context for explicit filtering
        let tenant_id = self.backend.get_tenant_context().await.ok_or_else(|| {
            PostgresError::Query(
                "Tenant context not set - call set_tenant_context() first".to_string(),
            )
        })?;

        let client = self.backend.get_client().await?;
        let now = Utc::now();

        // Use INSERT ... ON CONFLICT DO UPDATE to atomically increment frequency
        // This handles both new patterns and updates to existing ones
        let row = client
            .query_one(
                "INSERT INTO llmspell.procedural_patterns
                 (tenant_id, scope, key, value, frequency, first_seen, last_seen)
                 VALUES ($1, $2, $3, $4, 1, $5, $5)
                 ON CONFLICT (tenant_id, scope, key, value)
                 DO UPDATE SET
                   frequency = procedural_patterns.frequency + 1,
                   last_seen = $5
                 RETURNING frequency",
                &[&tenant_id, &scope, &key, &to_value, &now],
            )
            .await
            .map_err(|e| {
                PostgresError::Query(format!("Failed to record pattern transition: {}", e))
            })?;

        let frequency: i32 = row.get(0);
        Ok(frequency as u32)
    }

    /// Get frequency count for a specific state transition
    ///
    /// # Arguments
    /// * `scope` - State scope
    /// * `key` - State key
    /// * `value` - Transition value
    ///
    /// # Returns
    /// Number of times `scope:key → value` transition occurred (0 if not found)
    pub async fn get_pattern_frequency(
        &self,
        scope: &str,
        key: &str,
        value: &str,
    ) -> Result<u32> {
        // Get tenant context for explicit filtering
        let tenant_id = self.backend.get_tenant_context().await.ok_or_else(|| {
            PostgresError::Query(
                "Tenant context not set - call set_tenant_context() first".to_string(),
            )
        })?;

        let client = self.backend.get_client().await?;

        // Query pattern frequency using composite index (tenant_id, scope, key, value)
        let row_opt = client
            .query_opt(
                "SELECT frequency FROM llmspell.procedural_patterns
                 WHERE tenant_id = $1 AND scope = $2 AND key = $3 AND value = $4",
                &[&tenant_id, &scope, &key, &value],
            )
            .await
            .map_err(|e| {
                PostgresError::Query(format!("Failed to query pattern frequency: {}", e))
            })?;

        match row_opt {
            Some(row) => {
                let frequency: i32 = row.get(0);
                Ok(frequency as u32)
            }
            None => Ok(0),
        }
    }

    /// Get all learned patterns above minimum frequency threshold
    ///
    /// # Arguments
    /// * `min_frequency` - Minimum occurrences to be considered a pattern (typically 3)
    ///
    /// # Returns
    /// Patterns sorted by frequency (descending)
    pub async fn get_learned_patterns(&self, min_frequency: u32) -> Result<Vec<StoredPattern>> {
        // Get tenant context for explicit filtering
        let tenant_id = self.backend.get_tenant_context().await.ok_or_else(|| {
            PostgresError::Query(
                "Tenant context not set - call set_tenant_context() first".to_string(),
            )
        })?;

        let client = self.backend.get_client().await?;

        // Query learned patterns using partial index on frequency >= 3
        // Results ordered by frequency descending for most common patterns first
        let rows = client
            .query(
                "SELECT scope, key, value, frequency, first_seen, last_seen
                 FROM llmspell.procedural_patterns
                 WHERE tenant_id = $1 AND frequency >= $2
                 ORDER BY frequency DESC",
                &[&tenant_id, &(min_frequency as i32)],
            )
            .await
            .map_err(|e| {
                PostgresError::Query(format!("Failed to query learned patterns: {}", e))
            })?;

        let patterns = rows
            .iter()
            .map(|row| {
                let scope: String = row.get(0);
                let key: String = row.get(1);
                let value: String = row.get(2);
                let frequency: i32 = row.get(3);
                let first_seen: DateTime<Utc> = row.get(4);
                let last_seen: DateTime<Utc> = row.get(5);

                StoredPattern {
                    scope,
                    key,
                    value,
                    frequency: frequency as u32,
                    first_seen: Self::datetime_to_millis(first_seen),
                    last_seen: Self::datetime_to_millis(last_seen),
                }
            })
            .collect();

        Ok(patterns)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_datetime_conversion() {
        // Test epoch conversion
        let dt = DateTime::from_timestamp(0, 0).unwrap();
        assert_eq!(PostgresProceduralStorage::datetime_to_millis(dt), 0);

        // Test with milliseconds
        let dt = DateTime::from_timestamp(1234567890, 123_000_000).unwrap();
        assert_eq!(
            PostgresProceduralStorage::datetime_to_millis(dt),
            1234567890123
        );
    }
}
