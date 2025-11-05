//! ABOUTME: PostgreSQL event log storage with partitioning (Phase 13b.11.2)
//! ABOUTME: Specialized event storage with automatic partition management
//!
//! This module provides a PostgreSQL-specific event storage implementation optimized
//! for time-series event data with monthly partitioning. It mirrors the EventStorage
//! trait API from llmspell-events but is specialized for PostgreSQL's schema.
//!
//! # Integration with llmspell-events
//!
//! Applications using llmspell-events can use PostgresEventLogStorage directly:
//! ```rust,ignore
//! let event_storage = PostgresEventLogStorage::new(postgres_backend);
//! event_storage.store_event(&event).await?;
//! ```

use super::backend::PostgresBackend;
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio_postgres::Row;
use uuid::Uuid;

/// Event storage statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EventStorageStats {
    /// Total number of stored events
    pub total_events: u64,
    /// Storage size in bytes (approximate)
    pub storage_size_bytes: u64,
    /// Oldest event timestamp
    pub oldest_event: Option<DateTime<Utc>>,
    /// Newest event timestamp
    pub newest_event: Option<DateTime<Utc>>,
    /// Events by type
    pub events_by_type: HashMap<String, u64>,
}

/// PostgreSQL event log storage with monthly partitioning
///
/// Implements EventStorage trait with:
/// - Monthly partitioned table (automatic partition creation)
/// - Hybrid schema (extracted columns + JSONB payload)
/// - RLS for tenant isolation
/// - Optimized indexes for pattern/correlation/time queries
///
/// # Performance Targets
/// - store_event: <10ms
/// - get_events_by_correlation_id: <50ms (target from TODO.md:6074)
/// - get_events_by_pattern: <100ms
/// - get_events_by_time_range: <200ms with partition pruning
///
/// # Partition Management
/// - Automatic creation of future partitions (current + next 3 months)
/// - Manual cleanup via cleanup_old_events (drops partitions >90 days old)
#[derive(Debug, Clone)]
pub struct PostgresEventLogStorage {
    backend: Arc<PostgresBackend>,
}

impl PostgresEventLogStorage {
    /// Create new event log storage
    ///
    /// # Arguments
    /// * `backend` - PostgreSQL backend with connection pool
    ///
    /// # Example
    /// ```rust,no_run
    /// use llmspell_storage::backends::postgres::{PostgresBackend, PostgresConfig, PostgresEventLogStorage};
    /// use std::sync::Arc;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = PostgresConfig::new("postgresql://localhost/llmspell");
    /// let backend = Arc::new(PostgresBackend::new(config).await?);
    /// let storage = PostgresEventLogStorage::new(backend);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(backend: Arc<PostgresBackend>) -> Self {
        Self { backend }
    }

    /// Ensure future partitions exist (current + next 3 months)
    ///
    /// Calls PostgreSQL function `ensure_future_event_log_partitions()` which uses
    /// SECURITY DEFINER to create partitions with owner privileges.
    ///
    /// # Performance
    /// - <10ms per partition if not exists
    /// - <1ms per partition if already exists (SKIPPED)
    ///
    /// # Errors
    /// Returns error if partition creation fails
    async fn ensure_partitions(&self) -> Result<()> {
        let client = self
            .backend
            .get_client()
            .await
            .map_err(|e| anyhow!("Database connection failed: {}", e))?;

        client
            .query_one("SELECT llmspell.ensure_future_event_log_partitions()", &[])
            .await
            .map_err(|e| anyhow!("Failed to ensure partitions: {}", e))?;

        Ok(())
    }

    /// Convert PostgreSQL row to Value (JSONB payload)
    ///
    /// Returns the full JSONB payload which contains the serialized event.
    /// Extracted columns (event_type, correlation_id, etc.) are redundant for indexing.
    fn event_from_row(row: Row) -> Result<Value> {
        let payload: Value = row.get("payload");
        Ok(payload)
    }

    /// Get tenant context with error if not set
    async fn get_tenant_context(&self) -> Result<String> {
        self.backend
            .get_tenant_context()
            .await
            .ok_or_else(|| anyhow!("Tenant context not set - call set_tenant_context() first"))
    }

    /// Store an event in the event log
    ///
    /// # Arguments
    /// * `event_payload` - JSONB value containing the serialized event
    ///   Must contain: id, event_type, correlation_id, timestamp, sequence, language
    ///
    /// # Errors
    /// Returns error if partition creation fails or insert fails
    pub async fn store_event(&self, event_payload: &Value) -> Result<()> {
        // Ensure partitions exist before insert
        self.ensure_partitions()
            .await
            .map_err(|e| anyhow!("Partition creation failed: {}", e))?;

        let tenant_id = self
            .get_tenant_context()
            .await
            .map_err(|e| anyhow!("Tenant context error: {}", e))?;

        // Extract required fields from payload
        let event_id = event_payload["id"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing 'id' field in event payload"))?;
        let event_id = Uuid::parse_str(event_id)?;

        let event_type = event_payload["event_type"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing 'event_type' field in event payload"))?;

        let correlation_id = event_payload["metadata"]["correlation_id"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing 'metadata.correlation_id' field in event payload"))?;
        let correlation_id = Uuid::parse_str(correlation_id)?;

        let timestamp_str = event_payload["timestamp"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing 'timestamp' field in event payload"))?;
        let timestamp: DateTime<Utc> = timestamp_str.parse()?;

        let sequence = event_payload["sequence"]
            .as_i64()
            .ok_or_else(|| anyhow!("Missing 'sequence' field in event payload"))?;

        let language = event_payload["language"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing 'language' field in event payload"))?;

        let client = self
            .backend
            .get_client()
            .await
            .map_err(|e| anyhow!("Database connection failed: {}", e))?;

        // Insert with extracted columns for indexing + full payload
        client
            .execute(
                "INSERT INTO llmspell.event_log
                 (tenant_id, event_id, event_type, correlation_id, timestamp, sequence, language, payload)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8)",
                &[
                    &tenant_id,
                    &event_id,
                    &event_type,
                    &correlation_id,
                    &timestamp,
                    &sequence,
                    &language,
                    &event_payload,
                ],
            )
            .await
            .map_err(|e| anyhow!("Event insert failed: {}", e))?;

        Ok(())
    }

    /// Retrieve events by pattern matching on event_type
    ///
    /// # Arguments
    /// * `pattern` - Glob-style pattern (* and ? wildcards supported)
    ///
    /// # Returns
    /// Vector of JSONB payloads (serialized events) sorted by sequence
    pub async fn get_events_by_pattern(&self, pattern: &str) -> Result<Vec<Value>> {
        let tenant_id = self
            .get_tenant_context()
            .await
            .map_err(|e| anyhow!("Tenant context error: {}", e))?;

        let client = self
            .backend
            .get_client()
            .await
            .map_err(|e| anyhow!("Database connection failed: {}", e))?;

        // Convert glob pattern to SQL LIKE pattern
        // "*" -> "%", "?" -> "_"
        let sql_pattern = pattern.replace('*', "%").replace('?', "_");

        let rows = client
            .query(
                "SELECT event_id, event_type, correlation_id, timestamp, sequence, language, payload
                 FROM llmspell.event_log
                 WHERE tenant_id = $1
                   AND event_type LIKE $2
                 ORDER BY sequence ASC",
                &[&tenant_id, &sql_pattern],
            )
            .await
            .map_err(|e| anyhow!("Pattern query failed: {}", e))?;

        let mut events = Vec::with_capacity(rows.len());
        for row in rows {
            events.push(Self::event_from_row(row)?);
        }

        Ok(events)
    }

    /// Retrieve events in a time range
    ///
    /// # Arguments
    /// * `start` - Start of time range (inclusive)
    /// * `end` - End of time range (inclusive)
    ///
    /// # Returns
    /// Vector of JSONB payloads (serialized events) sorted by sequence
    ///
    /// # Performance
    /// Partition pruning automatically applied by PostgreSQL
    pub async fn get_events_by_time_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Value>> {
        let tenant_id = self
            .get_tenant_context()
            .await
            .map_err(|e| anyhow!("Tenant context error: {}", e))?;

        let client = self
            .backend
            .get_client()
            .await
            .map_err(|e| anyhow!("Database connection failed: {}", e))?;

        // Partition pruning automatically applied by PostgreSQL for timestamp range
        let rows = client
            .query(
                "SELECT event_id, event_type, correlation_id, timestamp, sequence, language, payload
                 FROM llmspell.event_log
                 WHERE tenant_id = $1
                   AND timestamp >= $2
                   AND timestamp <= $3
                 ORDER BY sequence ASC",
                &[&tenant_id, &start, &end],
            )
            .await
            .map_err(|e| anyhow!("Time range query failed: {}", e))?;

        let mut events = Vec::with_capacity(rows.len());
        for row in rows {
            events.push(Self::event_from_row(row)?);
        }

        Ok(events)
    }

    /// Retrieve events by correlation ID
    ///
    /// # Arguments
    /// * `correlation_id` - Correlation ID to search for
    ///
    /// # Returns
    /// Vector of JSONB payloads (serialized events) sorted by sequence
    ///
    /// # Performance
    /// Uses idx_event_log_correlation index for fast lookup (<50ms target)
    pub async fn get_events_by_correlation_id(&self, correlation_id: Uuid) -> Result<Vec<Value>> {
        let tenant_id = self
            .get_tenant_context()
            .await
            .map_err(|e| anyhow!("Tenant context error: {}", e))?;

        let client = self
            .backend
            .get_client()
            .await
            .map_err(|e| anyhow!("Database connection failed: {}", e))?;

        // Uses idx_event_log_correlation index for fast lookup
        let rows = client
            .query(
                "SELECT event_id, event_type, correlation_id, timestamp, sequence, language, payload
                 FROM llmspell.event_log
                 WHERE tenant_id = $1
                   AND correlation_id = $2
                 ORDER BY sequence ASC",
                &[&tenant_id, &correlation_id],
            )
            .await
            .map_err(|e| anyhow!("Correlation query failed: {}", e))?;

        let mut events = Vec::with_capacity(rows.len());
        for row in rows {
            events.push(Self::event_from_row(row)?);
        }

        Ok(events)
    }

    /// Cleanup old events by dropping partitions
    ///
    /// # Arguments
    /// * `before` - Delete events older than this timestamp
    ///
    /// # Returns
    /// Number of partitions dropped (approximation for events deleted)
    ///
    /// # Note
    /// Calls PostgreSQL function cleanup_old_event_log_partitions() which uses
    /// SECURITY DEFINER to drop partitions with owner privileges.
    pub async fn cleanup_old_events(&self, before: DateTime<Utc>) -> Result<usize> {
        let client = self
            .backend
            .get_client()
            .await
            .map_err(|e| anyhow!("Database connection failed: {}", e))?;

        // Call PostgreSQL function to drop old partitions (>90 days old)
        // Uses SECURITY DEFINER to execute with owner privileges
        let row = client
            .query_one(
                "SELECT llmspell.cleanup_old_event_log_partitions($1)",
                &[&before],
            )
            .await
            .map_err(|e| anyhow!("Partition cleanup failed: {}", e))?;

        let results: Vec<String> = row.get(0);

        // Count DROPPED results (SKIPPED means no action taken)
        let dropped_count = results.iter().filter(|r| r.starts_with("DROPPED")).count();

        // Each partition can contain many events, but we only know partition count
        // Return dropped partition count as approximation
        Ok(dropped_count)
    }

    /// Get storage statistics
    ///
    /// # Returns
    /// Statistics including total events, storage size, timestamp range, and event counts by type
    pub async fn get_storage_stats(&self) -> Result<EventStorageStats> {
        let tenant_id = self
            .get_tenant_context()
            .await
            .map_err(|e| anyhow!("Tenant context error: {}", e))?;

        let client = self
            .backend
            .get_client()
            .await
            .map_err(|e| anyhow!("Database connection failed: {}", e))?;

        // Aggregate stats across all partitions
        let row = client
            .query_one(
                "SELECT
                    COUNT(*) as total_events,
                    MIN(timestamp) as oldest_event,
                    MAX(timestamp) as newest_event
                 FROM llmspell.event_log
                 WHERE tenant_id = $1",
                &[&tenant_id],
            )
            .await
            .map_err(|e| anyhow!("Stats query failed: {}", e))?;

        let total_events: i64 = row.get("total_events");
        let oldest_event: Option<DateTime<Utc>> = row.get("oldest_event");
        let newest_event: Option<DateTime<Utc>> = row.get("newest_event");

        // Get event counts by type
        let type_rows = client
            .query(
                "SELECT event_type, COUNT(*) as count
                 FROM llmspell.event_log
                 WHERE tenant_id = $1
                 GROUP BY event_type",
                &[&tenant_id],
            )
            .await
            .map_err(|e| anyhow!("Event type stats query failed: {}", e))?;

        let mut events_by_type = HashMap::new();
        for row in type_rows {
            let event_type: String = row.get("event_type");
            let count: i64 = row.get("count");
            events_by_type.insert(event_type, count as u64);
        }

        // Estimate storage size (rough approximation: 1KB per event)
        let storage_size_bytes = (total_events as u64) * 1024;

        Ok(EventStorageStats {
            total_events: total_events as u64,
            storage_size_bytes,
            oldest_event,
            newest_event,
            events_by_type,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backends::postgres::{PostgresBackend, PostgresConfig};

    const TEST_CONNECTION_STRING: &str =
        "postgresql://llmspell_app:llmspell_app_pass@localhost:5432/llmspell_dev";

    fn create_test_event(event_type: &str) -> Value {
        let event_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let timestamp = Utc::now();
        let sequence = chrono::Utc::now().timestamp_millis();

        serde_json::json!({
            "id": event_id.to_string(),
            "event_type": event_type,
            "data": {"test": "data"},
            "language": "rust",
            "timestamp": timestamp.to_rfc3339(),
            "sequence": sequence,
            "metadata": {
                "correlation_id": correlation_id.to_string(),
                "source": null,
                "target": null,
                "tags": [],
                "priority": 0,
                "ttl": null
            },
            "schema_version": "1.0"
        })
    }

    #[tokio::test]
    #[cfg(feature = "postgres")]
    async fn test_event_log_storage_basic_operations() {
        let config = PostgresConfig::new(TEST_CONNECTION_STRING);
        let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
        backend
            .set_tenant_context(&format!("test_event_log_{}", Uuid::new_v4()))
            .await
            .unwrap();

        let storage = PostgresEventLogStorage::new(backend.clone());

        let event = create_test_event("test.event");
        let event_id = Uuid::parse_str(event["id"].as_str().unwrap()).unwrap();
        let correlation_id =
            Uuid::parse_str(event["metadata"]["correlation_id"].as_str().unwrap()).unwrap();

        // Store event
        storage.store_event(&event).await.unwrap();

        // Retrieve by correlation ID
        let events = storage
            .get_events_by_correlation_id(correlation_id)
            .await
            .unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0]["id"].as_str().unwrap(), event_id.to_string());
        assert_eq!(events[0]["event_type"].as_str().unwrap(), "test.event");

        // Cleanup
        let client = backend.get_client().await.unwrap();
        client
            .execute(
                "DELETE FROM llmspell.event_log WHERE event_id = $1",
                &[&event_id],
            )
            .await
            .unwrap();
    }

    #[tokio::test]
    #[cfg(feature = "postgres")]
    async fn test_event_log_pattern_matching() {
        let config = PostgresConfig::new(TEST_CONNECTION_STRING);
        let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
        backend
            .set_tenant_context(&format!("test_pattern_{}", Uuid::new_v4()))
            .await
            .unwrap();

        let storage = PostgresEventLogStorage::new(backend.clone());

        let event1 = create_test_event("agent.state_changed");
        let event2 = create_test_event("agent.action_taken");
        let event3 = create_test_event("system.startup");

        storage.store_event(&event1).await.unwrap();
        storage.store_event(&event2).await.unwrap();
        storage.store_event(&event3).await.unwrap();

        // Pattern: agent.*
        let events = storage.get_events_by_pattern("agent.*").await.unwrap();
        assert_eq!(events.len(), 2);

        // Pattern: system.*
        let events = storage.get_events_by_pattern("system.*").await.unwrap();
        assert_eq!(events.len(), 1);

        // Cleanup
        let client = backend.get_client().await.unwrap();
        for event in [event1, event2, event3] {
            let event_id = Uuid::parse_str(event["id"].as_str().unwrap()).unwrap();
            client
                .execute(
                    "DELETE FROM llmspell.event_log WHERE event_id = $1",
                    &[&event_id],
                )
                .await
                .unwrap();
        }
    }

    #[tokio::test]
    #[cfg(feature = "postgres")]
    async fn test_event_log_time_range_query() {
        let config = PostgresConfig::new(TEST_CONNECTION_STRING);
        let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
        backend
            .set_tenant_context(&format!("test_time_range_{}", Uuid::new_v4()))
            .await
            .unwrap();

        let storage = PostgresEventLogStorage::new(backend.clone());

        let event1 = create_test_event("test.event1");
        let event2 = create_test_event("test.event2");

        let start = Utc::now() - chrono::Duration::seconds(10);
        storage.store_event(&event1).await.unwrap();
        storage.store_event(&event2).await.unwrap();
        let end = Utc::now() + chrono::Duration::seconds(10);

        let events = storage.get_events_by_time_range(start, end).await.unwrap();
        assert!(events.len() >= 2);

        // Cleanup
        let client = backend.get_client().await.unwrap();
        for event in [event1, event2] {
            let event_id = Uuid::parse_str(event["id"].as_str().unwrap()).unwrap();
            client
                .execute(
                    "DELETE FROM llmspell.event_log WHERE event_id = $1",
                    &[&event_id],
                )
                .await
                .unwrap();
        }
    }

    #[tokio::test]
    #[cfg(feature = "postgres")]
    async fn test_event_log_storage_stats() {
        let config = PostgresConfig::new(TEST_CONNECTION_STRING);
        let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
        backend
            .set_tenant_context(&format!("test_stats_{}", Uuid::new_v4()))
            .await
            .unwrap();

        let storage = PostgresEventLogStorage::new(backend.clone());

        let event1 = create_test_event("test.event1");
        let event2 = create_test_event("test.event2");

        storage.store_event(&event1).await.unwrap();
        storage.store_event(&event2).await.unwrap();

        let stats = storage.get_storage_stats().await.unwrap();
        assert_eq!(stats.total_events, 2);
        assert!(stats.oldest_event.is_some());
        assert!(stats.newest_event.is_some());
        assert_eq!(stats.events_by_type["test.event1"], 1);
        assert_eq!(stats.events_by_type["test.event2"], 1);

        // Cleanup
        let client = backend.get_client().await.unwrap();
        for event in [event1, event2] {
            let event_id = Uuid::parse_str(event["id"].as_str().unwrap()).unwrap();
            client
                .execute(
                    "DELETE FROM llmspell.event_log WHERE event_id = $1",
                    &[&event_id],
                )
                .await
                .unwrap();
        }
    }
}
