//! SQLite event log storage (Phase 13c.2.7)
//!
//! Time-series event storage with hybrid schema for efficient querying.
//! Mirrors EventStorage API from llmspell-events without depending on it.
//!
//! # Integration with llmspell-events
//!
//! Applications using llmspell-events can use SqliteEventLogStorage directly:
//! ```rust,ignore
//! let event_storage = SqliteEventLogStorage::new(sqlite_backend);
//! event_storage.store_event(&event_json).await?;
//! ```

use super::backend::SqliteBackend;
use super::error::SqliteError;
use anyhow::{anyhow, Context};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
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

/// SQLite event log storage with time-series optimization
///
/// Mirrors EventStorage trait API with:
/// - Hybrid schema (extracted columns + JSON payload)
/// - Application-level tenant isolation
/// - Optimized indexes for pattern/correlation/time queries
///
/// # Performance Targets
/// - store_event: <10ms
/// - get_events_by_correlation_id: <50ms
/// - get_events_by_pattern: <100ms
pub struct SqliteEventLogStorage {
    backend: Arc<SqliteBackend>,
    tenant_id: String,
}

impl SqliteEventLogStorage {
    /// Create new event log storage
    pub fn new(backend: Arc<SqliteBackend>, tenant_id: String) -> Self {
        Self { backend, tenant_id }
    }

    /// Store event in log
    ///
    /// Event JSON must contain:
    /// - id: UUID string
    /// - event_type: string
    /// - timestamp: ISO8601 string or Unix timestamp
    /// - metadata.correlation_id: UUID string
    /// - language: string (rust/lua/javascript/python)
    ///
    /// Note: sequence is auto-generated per tenant (not extracted from payload)
    pub async fn store_event(&self, event_payload: &Value) -> anyhow::Result<()> {
        let conn = self.backend.get_connection().await?;
        let tenant_id = self.tenant_id.clone();

        // Extract required fields
        let event_id = event_payload["id"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing event id"))?;
        let event_type = event_payload["event_type"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing event_type"))?;
        let correlation_id = event_payload["metadata"]["correlation_id"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing metadata.correlation_id"))?;
        let language = event_payload["language"]
            .as_str()
            .ok_or_else(|| anyhow!("Missing language"))?;

        // Parse timestamp
        let timestamp = if let Some(ts_str) = event_payload["timestamp"].as_str() {
            DateTime::parse_from_rfc3339(ts_str)
                .context("Failed to parse timestamp")?
                .timestamp()
        } else if let Some(ts_int) = event_payload["timestamp"].as_i64() {
            ts_int
        } else {
            return Err(anyhow!("Invalid timestamp format"));
        };

        // Auto-generate sequence number per tenant
        let mut rows = conn
            .query(
                "SELECT COALESCE(MAX(sequence), -1) + 1 FROM event_log WHERE tenant_id = ?1",
                vec![libsql::Value::Text(tenant_id.clone())],
            )
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to get next sequence: {}", e)))?;

        let sequence: i64 = if let Some(row) = rows
            .next()
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to fetch sequence: {}", e)))?
        {
            row.get(0).unwrap_or(0)
        } else {
            0
        };

        // Serialize full payload
        let payload_json =
            serde_json::to_string(event_payload).context("Failed to serialize event payload")?;

        // Insert event
        conn.execute(
            "INSERT INTO event_log
             (tenant_id, event_id, event_type, correlation_id, timestamp, sequence, language, payload)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            vec![
                libsql::Value::Text(tenant_id),
                libsql::Value::Text(event_id.to_string()),
                libsql::Value::Text(event_type.to_string()),
                libsql::Value::Text(correlation_id.to_string()),
                libsql::Value::Integer(timestamp),
                libsql::Value::Integer(sequence),
                libsql::Value::Text(language.to_string()),
                libsql::Value::Text(payload_json),
            ],
        )
        .await
        .map_err(|e| SqliteError::Query(format!("Failed to insert event: {}", e)))?;

        Ok(())
    }

    /// Retrieve events by pattern
    ///
    /// Pattern uses SQL LIKE syntax (% wildcard)
    pub async fn get_events_by_pattern(&self, pattern: &str) -> anyhow::Result<Vec<Value>> {
        let conn = self.backend.get_connection().await?;
        let tenant_id = self.tenant_id.clone();

        let mut rows = conn
            .query(
                "SELECT payload FROM event_log
                 WHERE tenant_id = ?1 AND event_type LIKE ?2
                 ORDER BY timestamp DESC",
                vec![
                    libsql::Value::Text(tenant_id),
                    libsql::Value::Text(pattern.to_string()),
                ],
            )
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to query events by pattern: {}", e)))?;

        let mut events = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to iterate events: {}", e)))?
        {
            let payload_json: String = row.get(0).context("Failed to get payload")?;
            let event: Value =
                serde_json::from_str(&payload_json).context("Failed to deserialize event")?;
            events.push(event);
        }

        Ok(events)
    }

    /// Retrieve events in time range
    pub async fn get_events_by_time_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> anyhow::Result<Vec<Value>> {
        let conn = self.backend.get_connection().await?;
        let tenant_id = self.tenant_id.clone();

        let start_ts = start.timestamp();
        let end_ts = end.timestamp();

        let mut rows = conn
            .query(
                "SELECT payload FROM event_log
                 WHERE tenant_id = ?1 AND timestamp >= ?2 AND timestamp <= ?3
                 ORDER BY timestamp DESC",
                vec![
                    libsql::Value::Text(tenant_id),
                    libsql::Value::Integer(start_ts),
                    libsql::Value::Integer(end_ts),
                ],
            )
            .await
            .map_err(|e| {
                SqliteError::Query(format!("Failed to query events by time range: {}", e))
            })?;

        let mut events = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to iterate events: {}", e)))?
        {
            let payload_json: String = row.get(0).context("Failed to get payload")?;
            let event: Value =
                serde_json::from_str(&payload_json).context("Failed to deserialize event")?;
            events.push(event);
        }

        Ok(events)
    }

    /// Retrieve events by correlation ID
    pub async fn get_events_by_correlation_id(
        &self,
        correlation_id: Uuid,
    ) -> anyhow::Result<Vec<Value>> {
        let conn = self.backend.get_connection().await?;
        let tenant_id = self.tenant_id.clone();

        let correlation_id_str = correlation_id.to_string();

        let mut rows = conn
            .query(
                "SELECT payload FROM event_log
                 WHERE tenant_id = ?1 AND correlation_id = ?2
                 ORDER BY timestamp DESC",
                vec![
                    libsql::Value::Text(tenant_id),
                    libsql::Value::Text(correlation_id_str),
                ],
            )
            .await
            .map_err(|e| {
                SqliteError::Query(format!("Failed to query events by correlation_id: {}", e))
            })?;

        let mut events = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to iterate events: {}", e)))?
        {
            let payload_json: String = row.get(0).context("Failed to get payload")?;
            let event: Value =
                serde_json::from_str(&payload_json).context("Failed to deserialize event")?;
            events.push(event);
        }

        Ok(events)
    }

    /// Delete events older than specified time
    pub async fn cleanup_old_events(&self, before: DateTime<Utc>) -> anyhow::Result<usize> {
        let conn = self.backend.get_connection().await?;
        let tenant_id = self.tenant_id.clone();

        let before_ts = before.timestamp();

        let rows_affected = conn
            .execute(
                "DELETE FROM event_log
                 WHERE tenant_id = ?1 AND timestamp < ?2",
                vec![
                    libsql::Value::Text(tenant_id),
                    libsql::Value::Integer(before_ts),
                ],
            )
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to cleanup old events: {}", e)))?;

        Ok(rows_affected as usize)
    }

    /// Get storage statistics
    pub async fn get_storage_stats(&self) -> anyhow::Result<EventStorageStats> {
        let conn = self.backend.get_connection().await?;
        let tenant_id = self.tenant_id.clone();

        // Get total count, oldest/newest timestamps
        let mut rows = conn
            .query(
                "SELECT COUNT(*), MIN(timestamp), MAX(timestamp)
                 FROM event_log
                 WHERE tenant_id = ?1",
                vec![libsql::Value::Text(tenant_id.clone())],
            )
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to get storage stats: {}", e)))?;

        let (total_events, oldest_event, newest_event) = if let Some(row) = rows
            .next()
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to fetch stats row: {}", e)))?
        {
            let count: i64 = row.get(0).unwrap_or(0);
            let oldest: Option<i64> = row.get(1).ok();
            let newest: Option<i64> = row.get(2).ok();

            let oldest_event = oldest.and_then(|ts| DateTime::from_timestamp(ts, 0));
            let newest_event = newest.and_then(|ts| DateTime::from_timestamp(ts, 0));

            (count as u64, oldest_event, newest_event)
        } else {
            (0, None, None)
        };

        // Get events by type
        let mut rows = conn
            .query(
                "SELECT event_type, COUNT(*) FROM event_log
                 WHERE tenant_id = ?1
                 GROUP BY event_type",
                vec![libsql::Value::Text(tenant_id)],
            )
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to get events by type: {}", e)))?;

        let mut events_by_type = HashMap::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to iterate event types: {}", e)))?
        {
            let event_type: String = row.get(0).context("Failed to get event_type")?;
            let count: i64 = row.get(1).context("Failed to get count")?;
            events_by_type.insert(event_type, count as u64);
        }

        // Estimate storage size (approximate: 1KB per event)
        let storage_size_bytes = total_events * 1000;

        Ok(EventStorageStats {
            total_events,
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
    use crate::backends::sqlite::SqliteConfig;
    use chrono::Duration;
    use serde_json::json;

    async fn create_test_storage() -> (tempfile::TempDir, Arc<SqliteBackend>, SqliteEventLogStorage)
    {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let config = SqliteConfig::new(db_path.to_str().unwrap());
        let backend = Arc::new(SqliteBackend::new(config).await.unwrap());

        // Set tenant context
        backend.set_tenant_context("test-tenant").await.unwrap();

        // Run migrations
        let conn = backend.get_connection().await.unwrap();

        // V1: Initial setup
        conn.execute(
            "CREATE TABLE IF NOT EXISTS _migrations (version INTEGER PRIMARY KEY)",
            Vec::<libsql::Value>::new(),
        )
        .await
        .unwrap();

        // V11: event_log table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS event_log (
                id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
                tenant_id TEXT NOT NULL,
                event_id TEXT NOT NULL,
                event_type TEXT NOT NULL,
                correlation_id TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                sequence INTEGER NOT NULL,
                language TEXT NOT NULL,
                payload TEXT NOT NULL,
                UNIQUE(tenant_id, event_id),
                UNIQUE(tenant_id, sequence)
            )",
            Vec::<libsql::Value>::new(),
        )
        .await
        .unwrap();

        drop(conn);

        let storage = SqliteEventLogStorage::new(backend.clone(), "test-tenant".to_string());

        (temp_dir, backend, storage)
    }

    fn create_test_event(event_type: &str, correlation_id: Uuid) -> Value {
        json!({
            "id": Uuid::new_v4().to_string(),
            "event_type": event_type,
            "timestamp": Utc::now().to_rfc3339(),
            "sequence": 0,
            "metadata": {
                "correlation_id": correlation_id.to_string()
            },
            "language": "rust",
            "data": {"test": "data"}
        })
    }

    #[tokio::test]
    async fn test_store_event() {
        let (_temp_dir, _backend, storage) = create_test_storage().await;

        let event = create_test_event("test.event", Uuid::new_v4());
        storage.store_event(&event).await.unwrap();

        // Verify stored
        let events = storage.get_events_by_pattern("test.event").await.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0]["event_type"], "test.event");
    }

    #[tokio::test]
    async fn test_get_events_by_pattern() {
        let (_temp_dir, _backend, storage) = create_test_storage().await;

        let event1 = create_test_event("agent.started", Uuid::new_v4());
        let event2 = create_test_event("agent.stopped", Uuid::new_v4());
        let event3 = create_test_event("workflow.completed", Uuid::new_v4());

        storage.store_event(&event1).await.unwrap();
        storage.store_event(&event2).await.unwrap();
        storage.store_event(&event3).await.unwrap();

        // Pattern match "agent.%"
        let events = storage.get_events_by_pattern("agent.%").await.unwrap();
        assert_eq!(events.len(), 2);

        // Pattern match exact
        let events = storage
            .get_events_by_pattern("workflow.completed")
            .await
            .unwrap();
        assert_eq!(events.len(), 1);
    }

    #[tokio::test]
    async fn test_get_events_by_time_range() {
        let (_temp_dir, _backend, storage) = create_test_storage().await;

        let now = Utc::now();
        let event1 = create_test_event("test.old", Uuid::new_v4());
        let event2 = create_test_event("test.new", Uuid::new_v4());

        storage.store_event(&event1).await.unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        storage.store_event(&event2).await.unwrap();

        // Query recent events
        let start = now - Duration::seconds(60);
        let end = now + Duration::seconds(60);
        let events = storage.get_events_by_time_range(start, end).await.unwrap();
        assert_eq!(events.len(), 2);
    }

    #[tokio::test]
    async fn test_get_events_by_correlation_id() {
        let (_temp_dir, _backend, storage) = create_test_storage().await;

        let correlation_id = Uuid::new_v4();
        let event1 = create_test_event("test.1", correlation_id);
        let event2 = create_test_event("test.2", correlation_id);
        let event3 = create_test_event("test.3", Uuid::new_v4());

        storage.store_event(&event1).await.unwrap();
        storage.store_event(&event2).await.unwrap();
        storage.store_event(&event3).await.unwrap();

        let events = storage
            .get_events_by_correlation_id(correlation_id)
            .await
            .unwrap();
        assert_eq!(events.len(), 2);
    }

    #[tokio::test]
    async fn test_cleanup_old_events() {
        let (_temp_dir, _backend, storage) = create_test_storage().await;

        let old_event = create_test_event("test.old", Uuid::new_v4());
        storage.store_event(&old_event).await.unwrap();

        // Cleanup events older than now + 1 second
        let deleted = storage
            .cleanup_old_events(Utc::now() + Duration::seconds(1))
            .await
            .unwrap();
        assert_eq!(deleted, 1);

        let events = storage.get_events_by_pattern("%").await.unwrap();
        assert_eq!(events.len(), 0);
    }

    #[tokio::test]
    async fn test_get_storage_stats() {
        let (_temp_dir, _backend, storage) = create_test_storage().await;

        let event1 = create_test_event("agent.started", Uuid::new_v4());
        let event2 = create_test_event("agent.stopped", Uuid::new_v4());
        let event3 = create_test_event("workflow.completed", Uuid::new_v4());

        storage.store_event(&event1).await.unwrap();
        storage.store_event(&event2).await.unwrap();
        storage.store_event(&event3).await.unwrap();

        let stats = storage.get_storage_stats().await.unwrap();
        assert_eq!(stats.total_events, 3);
        assert!(stats.oldest_event.is_some());
        assert!(stats.newest_event.is_some());
        assert_eq!(stats.events_by_type.get("agent.started"), Some(&1));
        assert_eq!(stats.events_by_type.get("agent.stopped"), Some(&1));
        assert_eq!(stats.events_by_type.get("workflow.completed"), Some(&1));
    }

    #[tokio::test]
    async fn test_tenant_isolation() {
        let (_temp_dir, backend, storage1) = create_test_storage().await;

        // Create second tenant storage
        let storage2 = SqliteEventLogStorage::new(backend.clone(), "tenant-2".to_string());

        // Store events for both tenants
        let event1 = create_test_event("tenant1.event", Uuid::new_v4());
        let event2 = create_test_event("tenant2.event", Uuid::new_v4());

        storage1.store_event(&event1).await.unwrap();
        storage2.store_event(&event2).await.unwrap();

        // Verify isolation
        let events1 = storage1.get_events_by_pattern("%").await.unwrap();
        let events2 = storage2.get_events_by_pattern("%").await.unwrap();

        assert_eq!(events1.len(), 1);
        assert_eq!(events2.len(), 1);
        assert_eq!(events1[0]["event_type"], "tenant1.event");
        assert_eq!(events2[0]["event_type"], "tenant2.event");
    }
}
