// ABOUTME: SQLite procedural pattern storage (Phase 13c.2.5)
//! ABOUTME: Storage layer for state transition pattern tracking with tenant isolation

use super::backend::SqliteBackend;
use super::error::{Result, SqliteError};
use rusqlite::params;
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

/// SQLite-backed procedural pattern storage
///
/// Stores learned patterns from repeated state transitions with:
/// - Tenant isolation via application-level filtering
/// - Unique constraint on (tenant_id, scope, key, value)
/// - Frequency tracking with first_seen/last_seen timestamps
/// - Performance indexes for pattern queries
///
/// # Performance Target
/// <5ms for pattern insert, <10ms for pattern query (Task 13c.2.5)
///
/// # Architecture
/// This is a storage layer struct. The memory layer (`llmspell-memory`) wraps this
/// to implement the `ProceduralMemory` trait, following the same pattern as:
/// - `SqliteVectorStorage` → `SqliteEpisodicMemory`
/// - `SqliteGraphStorage` → `GraphSemanticMemory`
#[derive(Clone)]
pub struct SqliteProceduralStorage {
    backend: Arc<SqliteBackend>,
    tenant_id: String,
}

impl SqliteProceduralStorage {
    /// Create new SQLite procedural pattern storage
    ///
    /// # Arguments
    /// * `backend` - SQLite backend with connection pool
    /// * `tenant_id` - Tenant identifier for isolation
    ///
    /// # Example
    /// ```rust,no_run
    /// use llmspell_storage::backends::sqlite::{SqliteBackend, SqliteConfig};
    /// use llmspell_storage::backends::sqlite::SqliteProceduralStorage;
    /// use std::sync::Arc;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = SqliteConfig::new("./llmspell.db");
    /// let backend = Arc::new(SqliteBackend::new(config).await?);
    /// let storage = SqliteProceduralStorage::new(backend, "tenant-123".to_string());
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(backend: Arc<SqliteBackend>, tenant_id: String) -> Self {
        Self { backend, tenant_id }
    }

    /// Get tenant ID for queries
    fn get_tenant_id(&self) -> &str {
        &self.tenant_id
    }

    /// Convert database timestamp (Unix epoch seconds) to milliseconds since epoch
    fn timestamp_to_millis(timestamp: i64) -> u64 {
        // SQLite stores Unix epoch seconds, convert to milliseconds
        (timestamp * 1000) as u64
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
    ///
    /// # Errors
    /// Returns error if database query fails
    pub async fn record_transition(&self, scope: &str, key: &str, to_value: &str) -> Result<u32> {
        // Get tenant context for explicit filtering
        let tenant_id = self.get_tenant_id();

        let conn = self.backend.get_connection().await?;

        // Use INSERT ... ON CONFLICT DO UPDATE to atomically increment frequency
        // This handles both new patterns and updates to existing ones
        let mut stmt = conn
            .prepare(
                "INSERT INTO procedural_patterns
                 (tenant_id, scope, key, value, frequency, last_seen)
                 VALUES (?1, ?2, ?3, ?4, 1, strftime('%s', 'now'))
                 ON CONFLICT(tenant_id, scope, key, value) DO UPDATE SET
                   frequency = frequency + 1,
                   last_seen = strftime('%s', 'now'),
                   updated_at = strftime('%s', 'now')
                 RETURNING frequency",
            )
            .map_err(|e| SqliteError::Query(format!("Failed to prepare pattern insert: {}", e)))?;

        let frequency: u32 = stmt
            .query_row(params![tenant_id, scope, key, to_value], |row| row.get(0))
            .map_err(|e| {
                SqliteError::Query(format!("Failed to record pattern transition: {}", e))
            })?;

        Ok(frequency)
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
    ///
    /// # Errors
    /// Returns error if database query fails
    pub async fn get_pattern_frequency(&self, scope: &str, key: &str, value: &str) -> Result<u32> {
        // Get tenant context for explicit filtering
        let tenant_id = self.get_tenant_id();

        let conn = self.backend.get_connection().await?;

        // Query pattern frequency using composite index (tenant_id, scope, key, value)
        let mut stmt = conn
            .prepare(
                "SELECT frequency FROM procedural_patterns
                 WHERE tenant_id = ?1 AND scope = ?2 AND key = ?3 AND value = ?4",
            )
            .map_err(|e| SqliteError::Query(format!("Failed to prepare frequency query: {}", e)))?;

        let result = stmt.query_row(params![tenant_id, scope, key, value], |row| row.get(0));

        match result {
            Ok(freq) => Ok(freq),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(0),
            Err(e) => Err(SqliteError::Query(format!(
                "Failed to query pattern frequency: {}",
                e
            ))),
        }
    }

    /// Get all learned patterns above minimum frequency threshold
    ///
    /// # Arguments
    /// * `min_frequency` - Minimum occurrences to be considered a pattern (typically 3)
    ///
    /// # Returns
    /// Patterns sorted by frequency (descending)
    ///
    /// # Errors
    /// Returns error if database query fails
    pub async fn get_learned_patterns(&self, min_frequency: u32) -> Result<Vec<StoredPattern>> {
        // Get tenant context for explicit filtering
        let tenant_id = self.get_tenant_id();

        let conn = self.backend.get_connection().await?;

        // Query learned patterns using partial index on frequency >= 3
        // Results ordered by frequency descending for most common patterns first
        let mut stmt = conn
            .prepare(
                "SELECT scope, key, value, frequency, first_seen, last_seen
                 FROM procedural_patterns
                 WHERE tenant_id = ?1 AND frequency >= ?2
                 ORDER BY frequency DESC",
            )
            .map_err(|e| {
                SqliteError::Query(format!("Failed to prepare learned patterns query: {}", e))
            })?;

        let mut rows = stmt
            .query(params![tenant_id, min_frequency as i64])
            .map_err(|e| SqliteError::Query(format!("Failed to query learned patterns: {}", e)))?;

        let mut patterns = Vec::new();
        while let Some(row) = rows.next().map_err(|e| {
            SqliteError::Query(format!("Failed to fetch learned pattern row: {}", e))
        })? {
            let scope: String = row
                .get(0)
                .map_err(|e| SqliteError::Query(format!("Failed to get scope: {}", e)))?;
            let key: String = row
                .get(1)
                .map_err(|e| SqliteError::Query(format!("Failed to get key: {}", e)))?;
            let value: String = row
                .get(2)
                .map_err(|e| SqliteError::Query(format!("Failed to get value: {}", e)))?;
            let frequency: i64 = row
                .get(3)
                .map_err(|e| SqliteError::Query(format!("Failed to get frequency: {}", e)))?;
            let first_seen: i64 = row
                .get(4)
                .map_err(|e| SqliteError::Query(format!("Failed to get first_seen: {}", e)))?;
            let last_seen: i64 = row
                .get(5)
                .map_err(|e| SqliteError::Query(format!("Failed to get last_seen: {}", e)))?;

            patterns.push(StoredPattern {
                scope,
                key,
                value,
                frequency: frequency as u32,
                first_seen: Self::timestamp_to_millis(first_seen),
                last_seen: Self::timestamp_to_millis(last_seen),
            });
        }

        Ok(patterns)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backends::sqlite::{SqliteBackend, SqliteConfig};
    use tempfile::TempDir;

    #[test]
    fn test_timestamp_conversion() {
        // Test epoch conversion (0 seconds = 0 milliseconds)
        assert_eq!(SqliteProceduralStorage::timestamp_to_millis(0), 0);

        // Test with seconds (1234567890 seconds = 1234567890000 milliseconds)
        assert_eq!(
            SqliteProceduralStorage::timestamp_to_millis(1234567890),
            1234567890000
        );
    }

    async fn create_test_storage() -> (TempDir, Arc<SqliteBackend>, SqliteProceduralStorage, String)
    {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let config = SqliteConfig::new(db_path.to_str().unwrap()).with_max_connections(5);
        let backend = Arc::new(SqliteBackend::new(config).await.unwrap());

        // Run migrations manually (V1, V5 for procedural tests)
        // using sync run_migrations
        backend.run_migrations().await.unwrap();

        // Create unique tenant ID
        let tenant_id = format!("test-tenant-{}", uuid::Uuid::new_v4());

        // Set tenant context
        backend.set_tenant_context(&tenant_id).await.unwrap();

        let storage = SqliteProceduralStorage::new(Arc::clone(&backend), tenant_id.clone());

        (temp_dir, backend, storage, tenant_id)
    }

    #[tokio::test]
    async fn test_record_transition_creates_new_pattern() {
        let (_temp, _backend, storage, _tenant_id) = create_test_storage().await;

        // Record first transition
        let freq = storage
            .record_transition("global", "theme", "dark")
            .await
            .unwrap();

        assert_eq!(freq, 1, "First transition should have frequency 1");
    }

    #[tokio::test]
    async fn test_record_transition_increments_frequency() {
        let (_temp, _backend, storage, _tenant_id) = create_test_storage().await;

        // Record same transition multiple times
        let freq1 = storage
            .record_transition("global", "theme", "dark")
            .await
            .unwrap();
        let freq2 = storage
            .record_transition("global", "theme", "dark")
            .await
            .unwrap();
        let freq3 = storage
            .record_transition("global", "theme", "dark")
            .await
            .unwrap();

        assert_eq!(freq1, 1);
        assert_eq!(freq2, 2);
        assert_eq!(freq3, 3);
    }

    #[tokio::test]
    async fn test_get_pattern_frequency_returns_correct_value() {
        let (_temp, _backend, storage, _tenant_id) = create_test_storage().await;

        // Record pattern 5 times
        for _ in 0..5 {
            storage
                .record_transition("session:test", "config.lang", "en")
                .await
                .unwrap();
        }

        let freq = storage
            .get_pattern_frequency("session:test", "config.lang", "en")
            .await
            .unwrap();

        assert_eq!(freq, 5);
    }

    #[tokio::test]
    async fn test_get_pattern_frequency_returns_zero_for_nonexistent() {
        let (_temp, _backend, storage, _tenant_id) = create_test_storage().await;

        let freq = storage
            .get_pattern_frequency("global", "nonexistent", "value")
            .await
            .unwrap();

        assert_eq!(freq, 0);
    }

    #[tokio::test]
    async fn test_get_learned_patterns_with_threshold() {
        let (_temp, _backend, storage, _tenant_id) = create_test_storage().await;

        // Create patterns with different frequencies
        for _ in 0..2 {
            storage
                .record_transition("global", "pattern1", "value1")
                .await
                .unwrap();
        }
        for _ in 0..3 {
            storage
                .record_transition("global", "pattern2", "value2")
                .await
                .unwrap();
        }
        for _ in 0..5 {
            storage
                .record_transition("global", "pattern3", "value3")
                .await
                .unwrap();
        }

        // Get patterns with frequency >= 3
        let patterns = storage.get_learned_patterns(3).await.unwrap();

        assert_eq!(
            patterns.len(),
            2,
            "Should have 2 patterns with frequency >= 3"
        );

        // Verify they're the right patterns (sorted by frequency descending)
        assert_eq!(patterns[0].key, "pattern3");
        assert_eq!(patterns[0].frequency, 5);
        assert_eq!(patterns[1].key, "pattern2");
        assert_eq!(patterns[1].frequency, 3);
    }

    #[tokio::test]
    async fn test_get_learned_patterns_empty_when_none_match() {
        let (_temp, _backend, storage, _tenant_id) = create_test_storage().await;

        // Create pattern with frequency 2
        for _ in 0..2 {
            storage
                .record_transition("global", "pattern", "value")
                .await
                .unwrap();
        }

        // Query for frequency >= 3
        let patterns = storage.get_learned_patterns(3).await.unwrap();

        assert_eq!(patterns.len(), 0, "Should have no patterns with freq >= 3");
    }

    #[tokio::test]
    async fn test_get_learned_patterns_ordered_by_frequency() {
        let (_temp, _backend, storage, _tenant_id) = create_test_storage().await;

        // Create patterns with different frequencies (out of order)
        for _ in 0..3 {
            storage
                .record_transition("global", "pattern-mid", "mid")
                .await
                .unwrap();
        }
        for _ in 0..10 {
            storage
                .record_transition("global", "pattern-high", "high")
                .await
                .unwrap();
        }
        for _ in 0..5 {
            storage
                .record_transition("global", "pattern-med", "med")
                .await
                .unwrap();
        }

        let patterns = storage.get_learned_patterns(1).await.unwrap();

        assert_eq!(patterns.len(), 3);
        assert_eq!(patterns[0].frequency, 10, "Highest frequency first");
        assert_eq!(patterns[1].frequency, 5, "Middle frequency second");
        assert_eq!(patterns[2].frequency, 3, "Lowest frequency third");
    }

    #[tokio::test]
    async fn test_tenant_isolation_patterns_dont_cross() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let config = SqliteConfig::new(db_path.to_str().unwrap()).with_max_connections(5);
        let backend = Arc::new(SqliteBackend::new(config).await.unwrap());

        // Run migrations manually
        backend.run_migrations().await.unwrap();

        let tenant_a = format!("tenant-a-{}", uuid::Uuid::new_v4());
        let tenant_b = format!("tenant-b-{}", uuid::Uuid::new_v4());

        // Record pattern for tenant A
        backend.set_tenant_context(&tenant_a).await.unwrap();
        let storage = SqliteProceduralStorage::new(Arc::clone(&backend), tenant_a.clone());
        for _ in 0..5 {
            storage
                .record_transition("global", "theme", "dark")
                .await
                .unwrap();
        }

        // Record different pattern for tenant B
        backend.set_tenant_context(&tenant_b).await.unwrap();
        let storage_b = SqliteProceduralStorage::new(Arc::clone(&backend), tenant_b.clone());
        for _ in 0..3 {
            storage_b
                .record_transition("global", "theme", "light")
                .await
                .unwrap();
        }

        // Query as tenant A - should only see dark theme
        backend.set_tenant_context(&tenant_a).await.unwrap();
        let freq_a = storage
            .get_pattern_frequency("global", "theme", "dark")
            .await
            .unwrap();
        let freq_a_light = storage
            .get_pattern_frequency("global", "theme", "light")
            .await
            .unwrap();

        assert_eq!(freq_a, 5, "Tenant A should see dark theme with freq 5");
        assert_eq!(
            freq_a_light, 0,
            "Tenant A should not see tenant B's light theme"
        );

        // Query as tenant B - should only see light theme
        backend.set_tenant_context(&tenant_b).await.unwrap();
        let freq_b = storage_b
            .get_pattern_frequency("global", "theme", "light")
            .await
            .unwrap();
        let freq_b_dark = storage_b
            .get_pattern_frequency("global", "theme", "dark")
            .await
            .unwrap();

        assert_eq!(freq_b, 3, "Tenant B should see light theme with freq 3");
        assert_eq!(
            freq_b_dark, 0,
            "Tenant B should not see tenant A's dark theme"
        );
    }

    #[tokio::test]
    async fn test_multiple_values_for_same_scope_key() {
        let (_temp, _backend, storage, _tenant_id) = create_test_storage().await;

        // Record different values for same scope:key
        for _ in 0..3 {
            storage
                .record_transition("global", "theme", "dark")
                .await
                .unwrap();
        }
        for _ in 0..5 {
            storage
                .record_transition("global", "theme", "light")
                .await
                .unwrap();
        }
        for _ in 0..2 {
            storage
                .record_transition("global", "theme", "auto")
                .await
                .unwrap();
        }

        let freq_dark = storage
            .get_pattern_frequency("global", "theme", "dark")
            .await
            .unwrap();
        let freq_light = storage
            .get_pattern_frequency("global", "theme", "light")
            .await
            .unwrap();
        let freq_auto = storage
            .get_pattern_frequency("global", "theme", "auto")
            .await
            .unwrap();

        assert_eq!(freq_dark, 3);
        assert_eq!(freq_light, 5);
        assert_eq!(freq_auto, 2);

        let patterns = storage.get_learned_patterns(3).await.unwrap();
        assert_eq!(
            patterns.len(),
            2,
            "Should have 2 patterns with freq >= 3 (dark and light)"
        );
    }

    #[tokio::test]
    async fn test_pattern_struct_fields() {
        let (_temp, _backend, storage, _tenant_id) = create_test_storage().await;

        // Record pattern
        for _ in 0..3 {
            storage
                .record_transition("session:abc", "config.lang", "en")
                .await
                .unwrap();
        }

        let patterns = storage.get_learned_patterns(3).await.unwrap();
        assert_eq!(patterns.len(), 1);

        let pattern = &patterns[0];
        assert_eq!(pattern.scope, "session:abc");
        assert_eq!(pattern.key, "config.lang");
        assert_eq!(pattern.value, "en");
        assert_eq!(pattern.frequency, 3);
        assert!(pattern.first_seen > 0, "first_seen should be set");
        assert!(pattern.last_seen > 0, "last_seen should be set");
        assert!(
            pattern.last_seen >= pattern.first_seen,
            "last_seen should be >= first_seen"
        );
    }

    #[tokio::test]
    async fn test_first_seen_stays_constant_last_seen_updates() {
        let (_temp, _backend, storage, _tenant_id) = create_test_storage().await;

        // Record first transition
        storage
            .record_transition("global", "test", "value")
            .await
            .unwrap();

        let patterns1 = storage.get_learned_patterns(1).await.unwrap();
        let first_seen_1 = patterns1[0].first_seen;
        let last_seen_1 = patterns1[0].last_seen;

        // Wait a bit (SQLite uses Unix timestamp in seconds, so wait >1s for timestamp to change)
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // Record again
        storage
            .record_transition("global", "test", "value")
            .await
            .unwrap();

        let patterns2 = storage.get_learned_patterns(1).await.unwrap();
        let first_seen_2 = patterns2[0].first_seen;
        let last_seen_2 = patterns2[0].last_seen;

        assert_eq!(
            first_seen_1, first_seen_2,
            "first_seen should remain constant"
        );
        assert!(
            last_seen_2 > last_seen_1,
            "last_seen should update to later timestamp"
        );
    }

    #[tokio::test]
    async fn test_empty_scope_and_key() {
        let (_temp, _backend, storage, _tenant_id) = create_test_storage().await;

        // Record pattern with empty scope
        let freq = storage.record_transition("", "key", "value").await.unwrap();
        assert_eq!(freq, 1);

        // Record pattern with empty key
        let freq = storage
            .record_transition("scope", "", "value")
            .await
            .unwrap();
        assert_eq!(freq, 1);

        // Verify retrieval
        let freq = storage
            .get_pattern_frequency("", "key", "value")
            .await
            .unwrap();
        assert_eq!(freq, 1);

        let freq = storage
            .get_pattern_frequency("scope", "", "value")
            .await
            .unwrap();
        assert_eq!(freq, 1);
    }

    #[tokio::test]
    async fn test_long_strings() {
        let (_temp, _backend, storage, _tenant_id) = create_test_storage().await;

        // Create long strings (SQLite TEXT type has no practical limit)
        let long_scope = "a".repeat(400);
        let long_key = "b".repeat(400);
        let long_value = "c".repeat(1000);

        let freq = storage
            .record_transition(&long_scope, &long_key, &long_value)
            .await
            .unwrap();

        assert_eq!(freq, 1);

        let retrieved_freq = storage
            .get_pattern_frequency(&long_scope, &long_key, &long_value)
            .await
            .unwrap();

        assert_eq!(retrieved_freq, 1);
    }

    #[tokio::test]
    async fn test_special_characters() {
        let (_temp, _backend, storage, _tenant_id) = create_test_storage().await;

        // Test special characters
        let special_scope = "scope:with|pipes|and:colons";
        let special_key = "key.with.dots-and-dashes_underscores";
        let special_value = r#"{"json": "value", "with": ["arrays", "and", "stuff"]}"#;

        let freq = storage
            .record_transition(special_scope, special_key, special_value)
            .await
            .unwrap();

        assert_eq!(freq, 1);

        let retrieved_freq = storage
            .get_pattern_frequency(special_scope, special_key, special_value)
            .await
            .unwrap();

        assert_eq!(retrieved_freq, 1);
    }

    #[tokio::test]
    async fn test_concurrent_updates() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let config = SqliteConfig::new(db_path.to_str().unwrap()).with_max_connections(10);
        let backend = Arc::new(SqliteBackend::new(config).await.unwrap());

        let conn = backend.get_connection().await.unwrap();
        conn.execute_batch(include_str!(
            "../../../migrations/sqlite/V5__procedural_memory.sql"
        ))
        .unwrap();

        let tenant_id = format!("concurrent-{}", uuid::Uuid::new_v4());
        backend.set_tenant_context(&tenant_id).await.unwrap();

        let storage = Arc::new(SqliteProceduralStorage::new(backend, tenant_id));

        // Spawn 10 concurrent tasks each recording the same pattern 10 times
        let mut handles = vec![];
        for _ in 0..10 {
            let mem = Arc::clone(&storage);
            let handle = tokio::spawn(async move {
                for _ in 0..10 {
                    mem.record_transition("global", "concurrent", "test")
                        .await
                        .unwrap();
                }
            });
            handles.push(handle);
        }

        // Wait for all tasks to complete
        for handle in handles {
            handle.await.unwrap();
        }

        // Verify final frequency is 100
        let freq = storage
            .get_pattern_frequency("global", "concurrent", "test")
            .await
            .unwrap();

        assert_eq!(freq, 100, "Concurrent updates should all be counted");
    }

    #[tokio::test]
    async fn test_integration_realistic_workflow() {
        let (_temp, _backend, storage, _tenant_id) = create_test_storage().await;

        // Simulate realistic workflow: user changes theme multiple times
        storage
            .record_transition("user:session", "ui.theme", "light")
            .await
            .unwrap();
        storage
            .record_transition("user:session", "ui.theme", "dark")
            .await
            .unwrap();
        storage
            .record_transition("user:session", "ui.theme", "light")
            .await
            .unwrap();
        storage
            .record_transition("user:session", "ui.theme", "dark")
            .await
            .unwrap();
        storage
            .record_transition("user:session", "ui.theme", "dark")
            .await
            .unwrap();

        // User changes language
        for _ in 0..4 {
            storage
                .record_transition("user:session", "ui.lang", "en")
                .await
                .unwrap();
        }

        // User changes other settings
        storage
            .record_transition("user:session", "ui.font_size", "14")
            .await
            .unwrap();
        storage
            .record_transition("user:session", "ui.font_size", "16")
            .await
            .unwrap();

        // Get learned patterns (frequency >= 3)
        let patterns = storage.get_learned_patterns(3).await.unwrap();

        // Should have 2 learned patterns: dark theme (3x) and en language (4x)
        assert_eq!(patterns.len(), 2);

        // Verify patterns
        let lang_pattern = patterns.iter().find(|p| p.key == "ui.lang").unwrap();
        assert_eq!(lang_pattern.value, "en");
        assert_eq!(lang_pattern.frequency, 4);

        let theme_pattern = patterns.iter().find(|p| p.key == "ui.theme").unwrap();
        assert_eq!(theme_pattern.value, "dark");
        assert_eq!(theme_pattern.frequency, 3);
    }

    #[tokio::test]
    async fn test_performance_pattern_queries() {
        let (_temp, _backend, storage, _tenant_id) = create_test_storage().await;

        // Insert 100 patterns
        for i in 0..100 {
            for _ in 0..3 {
                storage
                    .record_transition("global", &format!("pattern-{}", i), "value")
                    .await
                    .unwrap();
            }
        }

        // Time get_learned_patterns query
        let start = std::time::Instant::now();
        let patterns = storage.get_learned_patterns(3).await.unwrap();
        let duration = start.elapsed();

        assert_eq!(patterns.len(), 100);
        assert!(
            duration.as_millis() < 50,
            "Query should be <50ms, was {}ms",
            duration.as_millis()
        );

        // Time get_pattern_frequency query
        let start = std::time::Instant::now();
        let _freq = storage
            .get_pattern_frequency("global", "pattern-50", "value")
            .await
            .unwrap();
        let duration = start.elapsed();

        assert!(
            duration.as_millis() < 10,
            "Pattern frequency query should be <10ms, was {}ms",
            duration.as_millis()
        );
    }
}
