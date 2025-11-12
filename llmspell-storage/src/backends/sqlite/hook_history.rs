//! SQLite hook history storage (Phase 13c.2.7)
//!
//! Hook execution history with compression and replay capabilities.
//! Mirrors StorageBackend API from llmspell-hooks without depending on it.
//!
//! # Integration with llmspell-hooks
//!
//! Applications using llmspell-hooks can use SqliteHookHistoryStorage directly:
//! ```rust,ignore
//! let hook_storage = SqliteHookHistoryStorage::new(sqlite_backend);
//! hook_storage.store_execution(&serialized_execution).await?;
//! ```

use super::backend::SqliteBackend;
use super::error::SqliteError;
use anyhow::Context;
use chrono::{DateTime, TimeZone, Utc};
use lz4_flex::{compress_prepend_size, decompress_size_prepended};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use uuid::Uuid;

/// Hook execution storage statistics
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HookHistoryStats {
    /// Total number of stored executions
    pub total_executions: u64,
    /// Storage size in bytes (approximate)
    pub storage_size_bytes: u64,
    /// Oldest execution timestamp
    pub oldest_execution: Option<DateTime<Utc>>,
    /// Newest execution timestamp
    pub newest_execution: Option<DateTime<Utc>>,
    /// Executions by hook ID
    pub executions_by_hook: HashMap<String, u64>,
    /// Executions by hook type
    pub executions_by_type: HashMap<String, u64>,
    /// Average execution duration (milliseconds)
    pub avg_duration_ms: f64,
}

/// Serialized hook execution for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedHookExecution {
    pub execution_id: Uuid,
    pub hook_id: String,
    pub hook_type: String,
    pub correlation_id: Uuid,
    pub hook_context: Vec<u8>, // Compressed serialized HookContext
    pub result_data: Value,    // Serialized HookResult (JSON)
    pub timestamp: DateTime<Utc>,
    pub duration_ms: i32,
    pub triggering_component: String,
    pub component_id: String,
    pub modified_operation: bool,
    pub tags: Vec<String>,
    pub retention_priority: i32,
    pub context_size: i32,
    pub contains_sensitive_data: bool,
    pub metadata: Value,
}

/// SQLite hook history storage with compression
///
/// Mirrors StorageBackend trait API with:
/// - LZ4 compression for hook_context (3-10x reduction)
/// - Application-level tenant isolation
/// - Optimized indexes for hook_id/correlation_id/type queries
/// - Retention policy support via archive_executions
///
/// # Performance Targets
/// - store_execution: <10ms
/// - load_execution: <5ms (primary key lookup + decompression)
/// - get_executions_by_correlation_id: <50ms
/// - get_executions_by_hook_id: <100ms
pub struct SqliteHookHistoryStorage {
    backend: Arc<SqliteBackend>,
    tenant_id: String,
}

impl SqliteHookHistoryStorage {
    /// Create new hook history storage
    pub fn new(backend: Arc<SqliteBackend>, tenant_id: String) -> Self {
        Self { backend, tenant_id }
    }

    /// Store a hook execution with compressed context
    ///
    /// Compresses hook_context using LZ4 before storage.
    pub async fn store_execution(&self, execution: &SerializedHookExecution) -> anyhow::Result<()> {
        let conn = self.backend.get_connection().await?;
        let tenant_id = self.tenant_id.clone();

        // Compress hook_context if not already compressed
        let compressed_context = if execution.hook_context.is_empty() {
            Vec::new()
        } else {
            compress_prepend_size(&execution.hook_context)
        };

        // Serialize tags as JSON array
        let tags_json = serde_json::to_string(&execution.tags)
            .context("Failed to serialize tags")?;

        // Serialize result_data and metadata
        let result_json = serde_json::to_string(&execution.result_data)
            .context("Failed to serialize result_data")?;
        let metadata_json = serde_json::to_string(&execution.metadata)
            .context("Failed to serialize metadata")?;

        // Insert execution
        conn.execute(
            "INSERT INTO hook_history
             (execution_id, tenant_id, hook_id, hook_type, correlation_id,
              hook_context, result_data, timestamp, duration_ms,
              triggering_component, component_id, modified_operation,
              tags, retention_priority, context_size, contains_sensitive_data, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)",
            vec![
                libsql::Value::Text(execution.execution_id.to_string()),
                libsql::Value::Text(tenant_id),
                libsql::Value::Text(execution.hook_id.clone()),
                libsql::Value::Text(execution.hook_type.clone()),
                libsql::Value::Text(execution.correlation_id.to_string()),
                libsql::Value::Blob(compressed_context),
                libsql::Value::Text(result_json),
                libsql::Value::Integer(execution.timestamp.timestamp()),
                libsql::Value::Integer(execution.duration_ms as i64),
                libsql::Value::Text(execution.triggering_component.clone()),
                libsql::Value::Text(execution.component_id.clone()),
                libsql::Value::Integer(if execution.modified_operation { 1 } else { 0 }),
                libsql::Value::Text(tags_json),
                libsql::Value::Integer(execution.retention_priority as i64),
                libsql::Value::Integer(execution.context_size as i64),
                libsql::Value::Integer(if execution.contains_sensitive_data { 1 } else { 0 }),
                libsql::Value::Text(metadata_json),
            ],
        )
        .await
        .map_err(|e| SqliteError::Query(format!("Failed to insert hook execution: {}", e)))?;

        Ok(())
    }

    /// Load a hook execution by execution_id
    ///
    /// Decompresses hook_context using LZ4 after retrieval.
    pub async fn load_execution(
        &self,
        execution_id: &Uuid,
    ) -> anyhow::Result<Option<SerializedHookExecution>> {
        let conn = self.backend.get_connection().await?;
        let tenant_id = self.tenant_id.clone();

        let mut rows = conn
            .query(
                "SELECT execution_id, hook_id, hook_type, correlation_id,
                        hook_context, result_data, timestamp, duration_ms,
                        triggering_component, component_id, modified_operation,
                        tags, retention_priority, context_size, contains_sensitive_data, metadata
                 FROM hook_history
                 WHERE tenant_id = ?1 AND execution_id = ?2",
                vec![
                    libsql::Value::Text(tenant_id),
                    libsql::Value::Text(execution_id.to_string()),
                ],
            )
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to query hook_history: {}", e)))?;

        if let Some(row) = rows
            .next()
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to fetch row: {}", e)))?
        {
            Ok(Some(self.execution_from_row(&row)?))
        } else {
            Ok(None)
        }
    }

    /// Get executions by correlation_id
    ///
    /// Returns all executions with matching correlation_id, ordered by timestamp.
    pub async fn get_executions_by_correlation_id(
        &self,
        correlation_id: &Uuid,
        limit: Option<i32>,
    ) -> anyhow::Result<Vec<SerializedHookExecution>> {
        let conn = self.backend.get_connection().await?;
        let tenant_id = self.tenant_id.clone();

        let limit_val = limit.unwrap_or(100);

        let mut rows = conn
            .query(
                "SELECT execution_id, hook_id, hook_type, correlation_id,
                        hook_context, result_data, timestamp, duration_ms,
                        triggering_component, component_id, modified_operation,
                        tags, retention_priority, context_size, contains_sensitive_data, metadata
                 FROM hook_history
                 WHERE tenant_id = ?1 AND correlation_id = ?2
                 ORDER BY timestamp DESC
                 LIMIT ?3",
                vec![
                    libsql::Value::Text(tenant_id),
                    libsql::Value::Text(correlation_id.to_string()),
                    libsql::Value::Integer(limit_val as i64),
                ],
            )
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to query hook_history: {}", e)))?;

        let mut executions = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to fetch row: {}", e)))?
        {
            executions.push(self.execution_from_row(&row)?);
        }

        Ok(executions)
    }

    /// Get executions by hook_id within time range
    ///
    /// Returns executions for a specific hook, filtered by time range.
    pub async fn get_executions_by_hook_id(
        &self,
        hook_id: &str,
        from_time: Option<DateTime<Utc>>,
        to_time: Option<DateTime<Utc>>,
        limit: Option<i32>,
    ) -> anyhow::Result<Vec<SerializedHookExecution>> {
        let conn = self.backend.get_connection().await?;
        let tenant_id = self.tenant_id.clone();

        let from_ts = from_time.map(|dt| dt.timestamp()).unwrap_or(0);
        let to_ts = to_time.map(|dt| dt.timestamp()).unwrap_or(i64::MAX);
        let limit_val = limit.unwrap_or(100);

        let mut rows = conn
            .query(
                "SELECT execution_id, hook_id, hook_type, correlation_id,
                        hook_context, result_data, timestamp, duration_ms,
                        triggering_component, component_id, modified_operation,
                        tags, retention_priority, context_size, contains_sensitive_data, metadata
                 FROM hook_history
                 WHERE tenant_id = ?1 AND hook_id = ?2
                   AND timestamp >= ?3 AND timestamp <= ?4
                 ORDER BY timestamp DESC
                 LIMIT ?5",
                vec![
                    libsql::Value::Text(tenant_id),
                    libsql::Value::Text(hook_id.to_string()),
                    libsql::Value::Integer(from_ts),
                    libsql::Value::Integer(to_ts),
                    libsql::Value::Integer(limit_val as i64),
                ],
            )
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to query hook_history: {}", e)))?;

        let mut executions = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to fetch row: {}", e)))?
        {
            executions.push(self.execution_from_row(&row)?);
        }

        Ok(executions)
    }

    /// Get executions by hook_type within time range
    ///
    /// Returns executions for a specific hook type, filtered by time range.
    pub async fn get_executions_by_type(
        &self,
        hook_type: &str,
        from_time: Option<DateTime<Utc>>,
        to_time: Option<DateTime<Utc>>,
        limit: Option<i32>,
    ) -> anyhow::Result<Vec<SerializedHookExecution>> {
        let conn = self.backend.get_connection().await?;
        let tenant_id = self.tenant_id.clone();

        let from_ts = from_time.map(|dt| dt.timestamp()).unwrap_or(0);
        let to_ts = to_time.map(|dt| dt.timestamp()).unwrap_or(i64::MAX);
        let limit_val = limit.unwrap_or(100);

        let mut rows = conn
            .query(
                "SELECT execution_id, hook_id, hook_type, correlation_id,
                        hook_context, result_data, timestamp, duration_ms,
                        triggering_component, component_id, modified_operation,
                        tags, retention_priority, context_size, contains_sensitive_data, metadata
                 FROM hook_history
                 WHERE tenant_id = ?1 AND hook_type = ?2
                   AND timestamp >= ?3 AND timestamp <= ?4
                 ORDER BY timestamp DESC
                 LIMIT ?5",
                vec![
                    libsql::Value::Text(tenant_id),
                    libsql::Value::Text(hook_type.to_string()),
                    libsql::Value::Integer(from_ts),
                    libsql::Value::Integer(to_ts),
                    libsql::Value::Integer(limit_val as i64),
                ],
            )
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to query hook_history: {}", e)))?;

        let mut executions = Vec::new();
        while let Some(row) = rows
            .next()
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to fetch row: {}", e)))?
        {
            executions.push(self.execution_from_row(&row)?);
        }

        Ok(executions)
    }

    /// Archive (delete) old hook executions for retention policy
    ///
    /// Deletes executions older than before_timestamp with retention_priority
    /// less than or equal to min_retention_priority.
    pub async fn archive_executions(
        &self,
        before_timestamp: DateTime<Utc>,
        min_retention_priority: i32,
    ) -> anyhow::Result<u64> {
        let conn = self.backend.get_connection().await?;
        let tenant_id = self.tenant_id.clone();

        let before_ts = before_timestamp.timestamp();

        let affected = conn
            .execute(
                "DELETE FROM hook_history
                 WHERE tenant_id = ?1
                   AND timestamp < ?2
                   AND retention_priority <= ?3",
                vec![
                    libsql::Value::Text(tenant_id),
                    libsql::Value::Integer(before_ts),
                    libsql::Value::Integer(min_retention_priority as i64),
                ],
            )
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to archive hook executions: {}", e)))?;

        Ok(affected)
    }

    /// Get storage statistics
    ///
    /// Returns comprehensive statistics about hook execution storage.
    pub async fn get_statistics(&self) -> anyhow::Result<HookHistoryStats> {
        let conn = self.backend.get_connection().await?;
        let tenant_id = self.tenant_id.clone();

        // Get basic stats (count, size, time range, avg duration)
        let mut rows = conn
            .query(
                "SELECT COUNT(*) as total_executions,
                        SUM(context_size) as storage_size_bytes,
                        MIN(timestamp) as oldest_execution,
                        MAX(timestamp) as newest_execution,
                        AVG(duration_ms) as avg_duration_ms
                 FROM hook_history
                 WHERE tenant_id = ?1",
                vec![libsql::Value::Text(tenant_id.clone())],
            )
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to query statistics: {}", e)))?;

        let (total_executions, storage_size_bytes, oldest_execution, newest_execution, avg_duration_ms) =
            if let Some(row) = rows
                .next()
                .await
                .map_err(|e| SqliteError::Query(format!("Failed to fetch stats: {}", e)))?
            {
                let total: i64 = row.get(0).unwrap_or(0);
                let size: i64 = row.get(1).unwrap_or(0);
                let oldest: Option<i64> = row.get::<i64>(2).ok();
                let newest: Option<i64> = row.get::<i64>(3).ok();
                let avg_dur: f64 = row.get(4).unwrap_or(0.0);

                let oldest_dt = oldest.map(|ts| Utc.timestamp_opt(ts, 0).unwrap());
                let newest_dt = newest.map(|ts| Utc.timestamp_opt(ts, 0).unwrap());

                (total as u64, size as u64, oldest_dt, newest_dt, avg_dur)
            } else {
                (0, 0, None, None, 0.0)
            };

        // Get executions by hook_id
        let mut executions_by_hook = HashMap::new();
        let mut hook_rows = conn
            .query(
                "SELECT hook_id, COUNT(*) as count
                 FROM hook_history
                 WHERE tenant_id = ?1
                 GROUP BY hook_id",
                vec![libsql::Value::Text(tenant_id.clone())],
            )
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to query hook_id stats: {}", e)))?;

        while let Some(row) = hook_rows
            .next()
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to fetch hook_id row: {}", e)))?
        {
            let hook_id: String = row.get(0).unwrap_or_default();
            let count: i64 = row.get(1).unwrap_or(0);
            executions_by_hook.insert(hook_id, count as u64);
        }

        // Get executions by hook_type
        let mut executions_by_type = HashMap::new();
        let mut type_rows = conn
            .query(
                "SELECT hook_type, COUNT(*) as count
                 FROM hook_history
                 WHERE tenant_id = ?1
                 GROUP BY hook_type",
                vec![libsql::Value::Text(tenant_id)],
            )
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to query hook_type stats: {}", e)))?;

        while let Some(row) = type_rows
            .next()
            .await
            .map_err(|e| SqliteError::Query(format!("Failed to fetch hook_type row: {}", e)))?
        {
            let hook_type: String = row.get(0).unwrap_or_default();
            let count: i64 = row.get(1).unwrap_or(0);
            executions_by_type.insert(hook_type, count as u64);
        }

        Ok(HookHistoryStats {
            total_executions,
            storage_size_bytes,
            oldest_execution,
            newest_execution,
            executions_by_hook,
            executions_by_type,
            avg_duration_ms,
        })
    }

    /// Convert SQLite row to SerializedHookExecution
    fn execution_from_row(
        &self,
        row: &libsql::Row,
    ) -> anyhow::Result<SerializedHookExecution> {
        let execution_id_str: String = row.get(0).context("Missing execution_id")?;
        let execution_id = Uuid::parse_str(&execution_id_str)
            .context("Failed to parse execution_id")?;

        let hook_id: String = row.get(1).unwrap_or_default();
        let hook_type: String = row.get(2).unwrap_or_default();

        let correlation_id_str: String = row.get(3).context("Missing correlation_id")?;
        let correlation_id = Uuid::parse_str(&correlation_id_str)
            .context("Failed to parse correlation_id")?;

        // Decompress hook_context
        let compressed_context: Vec<u8> = row.get(4).unwrap_or_default();
        let hook_context = if compressed_context.is_empty() {
            Vec::new()
        } else {
            decompress_size_prepended(&compressed_context)
                .context("Failed to decompress hook_context")?
        };

        let result_json: String = row.get(5).unwrap_or_else(|_| "{}".to_string());
        let result_data: Value = serde_json::from_str(&result_json)
            .context("Failed to parse result_data")?;

        let timestamp_i64: i64 = row.get(6).unwrap_or(0);
        let timestamp = Utc.timestamp_opt(timestamp_i64, 0).unwrap();

        let duration_ms: i64 = row.get(7).unwrap_or(0);

        let triggering_component: String = row.get(8).unwrap_or_default();
        let component_id: String = row.get(9).unwrap_or_default();

        let modified_operation_i64: i64 = row.get(10).unwrap_or(0);
        let modified_operation = modified_operation_i64 != 0;

        let tags_json: String = row.get(11).unwrap_or_else(|_| "[]".to_string());
        let tags: Vec<String> = serde_json::from_str(&tags_json)
            .context("Failed to parse tags")?;

        let retention_priority: i64 = row.get(12).unwrap_or(0);
        let context_size: i64 = row.get(13).unwrap_or(0);

        let contains_sensitive_data_i64: i64 = row.get(14).unwrap_or(0);
        let contains_sensitive_data = contains_sensitive_data_i64 != 0;

        let metadata_json: String = row.get(15).unwrap_or_else(|_| "{}".to_string());
        let metadata: Value = serde_json::from_str(&metadata_json)
            .context("Failed to parse metadata")?;

        Ok(SerializedHookExecution {
            execution_id,
            hook_id,
            hook_type,
            correlation_id,
            hook_context,
            result_data,
            timestamp,
            duration_ms: duration_ms as i32,
            triggering_component,
            component_id,
            modified_operation,
            tags,
            retention_priority: retention_priority as i32,
            context_size: context_size as i32,
            contains_sensitive_data,
            metadata,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backends::sqlite::SqliteConfig;
    use chrono::Duration;

    async fn setup() -> (tempfile::TempDir, Arc<SqliteBackend>, String) {
        let temp_dir = tempfile::tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let config = SqliteConfig::new(db_path.to_str().unwrap());
        let backend = Arc::new(SqliteBackend::new(config).await.unwrap());
        let tenant_id = format!("test_hook_{}", Uuid::new_v4());

        backend.set_tenant_context(&tenant_id).await.unwrap();

        // Run migrations
        let conn = backend.get_connection().await.unwrap();

        // V1: Initial setup
        conn.execute(
            "CREATE TABLE IF NOT EXISTS _migrations (version INTEGER PRIMARY KEY)",
            Vec::<libsql::Value>::new(),
        )
        .await
        .unwrap();

        // V13: hook_history table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS hook_history (
                id TEXT PRIMARY KEY DEFAULT (lower(hex(randomblob(16)))),
                execution_id TEXT NOT NULL,
                tenant_id TEXT NOT NULL,
                hook_id TEXT NOT NULL,
                hook_type TEXT NOT NULL,
                correlation_id TEXT NOT NULL,
                hook_context BLOB NOT NULL,
                result_data TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                duration_ms INTEGER NOT NULL,
                triggering_component TEXT NOT NULL,
                component_id TEXT NOT NULL,
                modified_operation INTEGER NOT NULL DEFAULT 0,
                tags TEXT DEFAULT '[]',
                retention_priority INTEGER NOT NULL DEFAULT 0,
                context_size INTEGER NOT NULL,
                contains_sensitive_data INTEGER NOT NULL DEFAULT 0,
                metadata TEXT NOT NULL DEFAULT '{}',
                created_at INTEGER NOT NULL DEFAULT (strftime('%s', 'now')),
                UNIQUE(tenant_id, execution_id)
            )",
            Vec::<libsql::Value>::new(),
        )
        .await
        .unwrap();

        drop(conn);

        (temp_dir, backend, tenant_id)
    }

    #[tokio::test]
    async fn test_hook_history_store_and_load() {
        let (_temp_dir, backend, tenant_id) = setup().await;
        let storage = SqliteHookHistoryStorage::new(backend, tenant_id);

        let execution_id = Uuid::new_v4();
        let correlation_id = Uuid::new_v4();
        let hook_context = b"test hook context data".to_vec();

        let execution = SerializedHookExecution {
            execution_id,
            hook_id: "test_rate_limiter".to_string(),
            hook_type: "rate_limit".to_string(),
            correlation_id,
            hook_context: hook_context.clone(),
            result_data: serde_json::json!({
                "success": true,
                "message": "Rate limit check passed"
            }),
            timestamp: Utc::now(),
            duration_ms: 25,
            triggering_component: "Agent".to_string(),
            component_id: "agent-123".to_string(),
            modified_operation: false,
            tags: vec!["production".to_string(), "critical".to_string()],
            retention_priority: 10,
            context_size: hook_context.len() as i32,
            contains_sensitive_data: false,
            metadata: serde_json::json!({"env": "prod"}),
        };

        // Store execution
        storage.store_execution(&execution).await.unwrap();

        // Load execution
        let loaded = storage
            .load_execution(&execution_id)
            .await
            .unwrap()
            .expect("Execution should exist");

        assert_eq!(loaded.execution_id, execution_id);
        assert_eq!(loaded.hook_id, "test_rate_limiter");
        assert_eq!(loaded.hook_type, "rate_limit");
        assert_eq!(loaded.correlation_id, correlation_id);
        assert_eq!(loaded.hook_context, hook_context);
        assert_eq!(loaded.duration_ms, 25);
        assert_eq!(loaded.tags, vec!["production", "critical"]);
        assert_eq!(loaded.retention_priority, 10);
    }

    #[tokio::test]
    async fn test_hook_history_correlation_query() {
        let (_temp_dir, backend, tenant_id) = setup().await;
        let storage = SqliteHookHistoryStorage::new(backend, tenant_id);

        let correlation_id = Uuid::new_v4();

        // Store 3 executions with same correlation_id
        for i in 0..3 {
            let execution = SerializedHookExecution {
                execution_id: Uuid::new_v4(),
                hook_id: format!("hook_{}", i),
                hook_type: "test_type".to_string(),
                correlation_id,
                hook_context: vec![],
                result_data: serde_json::json!({"index": i}),
                timestamp: Utc::now(),
                duration_ms: 10 + i,
                triggering_component: "Test".to_string(),
                component_id: "test-123".to_string(),
                modified_operation: false,
                tags: vec![],
                retention_priority: 0,
                context_size: 0,
                contains_sensitive_data: false,
                metadata: serde_json::json!({}),
            };
            storage.store_execution(&execution).await.unwrap();
        }

        // Query by correlation_id
        let executions = storage
            .get_executions_by_correlation_id(&correlation_id, None)
            .await
            .unwrap();

        assert_eq!(executions.len(), 3);
        assert_eq!(executions[0].correlation_id, correlation_id);
    }

    #[tokio::test]
    async fn test_hook_history_hook_id_query() {
        let (_temp_dir, backend, tenant_id) = setup().await;
        let storage = SqliteHookHistoryStorage::new(backend, tenant_id);

        let hook_id = "test_hook";
        let now = Utc::now();

        // Store 2 executions for hook_id
        for i in 0..2 {
            let execution = SerializedHookExecution {
                execution_id: Uuid::new_v4(),
                hook_id: hook_id.to_string(),
                hook_type: "test_type".to_string(),
                correlation_id: Uuid::new_v4(),
                hook_context: vec![],
                result_data: serde_json::json!({"index": i}),
                timestamp: now - Duration::hours(i),
                duration_ms: 10,
                triggering_component: "Test".to_string(),
                component_id: "test-123".to_string(),
                modified_operation: false,
                tags: vec![],
                retention_priority: 0,
                context_size: 0,
                contains_sensitive_data: false,
                metadata: serde_json::json!({}),
            };
            storage.store_execution(&execution).await.unwrap();
        }

        // Query by hook_id
        let executions = storage
            .get_executions_by_hook_id(hook_id, None, None, None)
            .await
            .unwrap();

        assert_eq!(executions.len(), 2);
        assert_eq!(executions[0].hook_id, hook_id);
    }

    #[tokio::test]
    async fn test_hook_history_type_query() {
        let (_temp_dir, backend, tenant_id) = setup().await;
        let storage = SqliteHookHistoryStorage::new(backend, tenant_id);

        let hook_type = "security_check";

        // Store 2 executions of same type
        for i in 0..2 {
            let execution = SerializedHookExecution {
                execution_id: Uuid::new_v4(),
                hook_id: format!("hook_{}", i),
                hook_type: hook_type.to_string(),
                correlation_id: Uuid::new_v4(),
                hook_context: vec![],
                result_data: serde_json::json!({"index": i}),
                timestamp: Utc::now(),
                duration_ms: 10,
                triggering_component: "Test".to_string(),
                component_id: "test-123".to_string(),
                modified_operation: false,
                tags: vec![],
                retention_priority: 0,
                context_size: 0,
                contains_sensitive_data: false,
                metadata: serde_json::json!({}),
            };
            storage.store_execution(&execution).await.unwrap();
        }

        // Query by type
        let executions = storage
            .get_executions_by_type(hook_type, None, None, None)
            .await
            .unwrap();

        assert_eq!(executions.len(), 2);
        assert_eq!(executions[0].hook_type, hook_type);
    }

    #[tokio::test]
    async fn test_hook_history_archive() {
        let (_temp_dir, backend, tenant_id) = setup().await;
        let storage = SqliteHookHistoryStorage::new(backend, tenant_id);

        let now = Utc::now();
        let old_time = now - Duration::days(100);

        // Store old execution with low priority
        let old_execution = SerializedHookExecution {
            execution_id: Uuid::new_v4(),
            hook_id: "old_hook".to_string(),
            hook_type: "test_type".to_string(),
            correlation_id: Uuid::new_v4(),
            hook_context: vec![],
            result_data: serde_json::json!({}),
            timestamp: old_time,
            duration_ms: 10,
            triggering_component: "Test".to_string(),
            component_id: "test-123".to_string(),
            modified_operation: false,
            tags: vec![],
            retention_priority: 0,
            context_size: 0,
            contains_sensitive_data: false,
            metadata: serde_json::json!({}),
        };
        storage.store_execution(&old_execution).await.unwrap();

        // Store recent execution
        let new_execution = SerializedHookExecution {
            execution_id: Uuid::new_v4(),
            hook_id: "new_hook".to_string(),
            hook_type: "test_type".to_string(),
            correlation_id: Uuid::new_v4(),
            hook_context: vec![],
            result_data: serde_json::json!({}),
            timestamp: now,
            duration_ms: 10,
            triggering_component: "Test".to_string(),
            component_id: "test-123".to_string(),
            modified_operation: false,
            tags: vec![],
            retention_priority: 0,
            context_size: 0,
            contains_sensitive_data: false,
            metadata: serde_json::json!({}),
        };
        storage.store_execution(&new_execution).await.unwrap();

        // Archive executions older than 90 days with priority <= 0
        let cutoff = now - Duration::days(90);
        let archived = storage.archive_executions(cutoff, 0).await.unwrap();

        assert_eq!(archived, 1); // Only old execution should be archived

        // Verify new execution still exists
        let loaded = storage
            .load_execution(&new_execution.execution_id)
            .await
            .unwrap();
        assert!(loaded.is_some());

        // Verify old execution is gone
        let loaded = storage
            .load_execution(&old_execution.execution_id)
            .await
            .unwrap();
        assert!(loaded.is_none());
    }

    #[tokio::test]
    async fn test_hook_history_statistics() {
        let (_temp_dir, backend, tenant_id) = setup().await;
        let storage = SqliteHookHistoryStorage::new(backend, tenant_id);

        // Store executions with different hooks and types
        for i in 0..5 {
            let execution = SerializedHookExecution {
                execution_id: Uuid::new_v4(),
                hook_id: if i < 3 { "hook_a" } else { "hook_b" }.to_string(),
                hook_type: if i < 2 { "type_x" } else { "type_y" }.to_string(),
                correlation_id: Uuid::new_v4(),
                hook_context: vec![0u8; 100], // 100 bytes
                result_data: serde_json::json!({}),
                timestamp: Utc::now(),
                duration_ms: 10 + i,
                triggering_component: "Test".to_string(),
                component_id: "test-123".to_string(),
                modified_operation: false,
                tags: vec![],
                retention_priority: 0,
                context_size: 100,
                contains_sensitive_data: false,
                metadata: serde_json::json!({}),
            };
            storage.store_execution(&execution).await.unwrap();
        }

        // Get statistics
        let stats = storage.get_statistics().await.unwrap();

        assert_eq!(stats.total_executions, 5);
        assert_eq!(stats.storage_size_bytes, 500); // 5 * 100 bytes
        assert!(stats.oldest_execution.is_some());
        assert!(stats.newest_execution.is_some());
        assert_eq!(stats.executions_by_hook.get("hook_a"), Some(&3));
        assert_eq!(stats.executions_by_hook.get("hook_b"), Some(&2));
        assert_eq!(stats.executions_by_type.get("type_x"), Some(&2));
        assert_eq!(stats.executions_by_type.get("type_y"), Some(&3));
        assert!(stats.avg_duration_ms > 0.0);
    }

    #[tokio::test]
    async fn test_hook_history_compression() {
        let (_temp_dir, backend, tenant_id) = setup().await;
        let storage = SqliteHookHistoryStorage::new(backend, tenant_id);

        let execution_id = Uuid::new_v4();
        let large_context = vec![42u8; 10000]; // 10KB of data

        let execution = SerializedHookExecution {
            execution_id,
            hook_id: "compression_test".to_string(),
            hook_type: "test".to_string(),
            correlation_id: Uuid::new_v4(),
            hook_context: large_context.clone(),
            result_data: serde_json::json!({}),
            timestamp: Utc::now(),
            duration_ms: 10,
            triggering_component: "Test".to_string(),
            component_id: "test-123".to_string(),
            modified_operation: false,
            tags: vec![],
            retention_priority: 0,
            context_size: large_context.len() as i32,
            contains_sensitive_data: false,
            metadata: serde_json::json!({}),
        };

        // Store with compression
        storage.store_execution(&execution).await.unwrap();

        // Load and verify decompression
        let loaded = storage
            .load_execution(&execution_id)
            .await
            .unwrap()
            .expect("Execution should exist");

        assert_eq!(loaded.hook_context, large_context);
        assert_eq!(loaded.context_size, 10000);
    }
}
