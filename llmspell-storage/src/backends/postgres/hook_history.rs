//! ABOUTME: PostgreSQL hook history storage (Phase 13b.12.2)
//! ABOUTME: Hook execution history with compression and replay capabilities
//!
//! This module provides PostgreSQL-specific hook history storage implementation
//! optimized for hook execution tracking and replay. It mirrors the StorageBackend
//! trait API from llmspell-hooks but is specialized for PostgreSQL's schema.
//!
//! # Integration with llmspell-hooks
//!
//! Applications using llmspell-hooks can use PostgresHookHistoryStorage directly:
//! ```rust,ignore
//! let hook_storage = PostgresHookHistoryStorage::new(postgres_backend);
//! hook_storage.store_execution(&serialized_execution, &metadata).await?;
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
    pub result_data: Value,    // Serialized HookResult (JSONB)
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

/// Hook metadata for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HookMetadata {
    pub retention_priority: i32,
    pub tags: Vec<String>,
    pub contains_sensitive_data: bool,
    pub metadata: Value,
}

/// PostgreSQL hook history storage
///
/// Implements hook execution history with:
/// - Compression support (BYTEA for hook_context)
/// - RLS for tenant isolation
/// - Optimized indexes for hook_id/correlation_id/type queries
/// - Cleanup functions for retention policies
///
/// # Performance Targets
/// - store_execution: <10ms
/// - load_execution: <5ms (primary key lookup)
/// - get_executions_by_correlation_id: <50ms
/// - get_executions_by_hook_id: <100ms
#[derive(Debug, Clone)]
pub struct PostgresHookHistoryStorage {
    backend: Arc<PostgresBackend>,
}

impl PostgresHookHistoryStorage {
    /// Create new hook history storage
    ///
    /// # Arguments
    /// * `backend` - PostgreSQL backend with connection pool
    ///
    /// # Example
    /// ```rust,no_run
    /// use llmspell_storage::backends::postgres::{PostgresBackend, PostgresConfig, PostgresHookHistoryStorage};
    /// use std::sync::Arc;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let config = PostgresConfig::new("postgresql://localhost/llmspell");
    /// let backend = Arc::new(PostgresBackend::new(config).await?);
    /// let storage = PostgresHookHistoryStorage::new(backend);
    /// # Ok(())
    /// # }
    /// ```
    pub fn new(backend: Arc<PostgresBackend>) -> Self {
        Self { backend }
    }

    /// Get tenant context with error if not set
    async fn get_tenant_context(&self) -> Result<String> {
        self.backend
            .get_tenant_context()
            .await
            .ok_or_else(|| anyhow!("Tenant context not set - call set_tenant_context() first"))
    }

    /// Convert PostgreSQL row to SerializedHookExecution
    fn execution_from_row(row: &Row) -> Result<SerializedHookExecution> {
        let execution_id: Uuid = row.get("execution_id");
        let hook_id: String = row.get("hook_id");
        let hook_type: String = row.get("hook_type");
        let correlation_id: Uuid = row.get("correlation_id");
        let hook_context: Vec<u8> = row.get("hook_context");
        let result_data: Value = row.get("result_data");
        let timestamp: DateTime<Utc> = row.get("timestamp");
        let duration_ms: i32 = row.get("duration_ms");
        let triggering_component: String = row.get("triggering_component");
        let component_id: String = row.get("component_id");
        let modified_operation: bool = row.get("modified_operation");
        let tags: Vec<String> = row.get("tags");
        let retention_priority: i32 = row.get("retention_priority");
        let context_size: i32 = row.get("context_size");
        let contains_sensitive_data: bool = row.get("contains_sensitive_data");
        let metadata: Value = row.get("metadata");

        Ok(SerializedHookExecution {
            execution_id,
            hook_id,
            hook_type,
            correlation_id,
            hook_context,
            result_data,
            timestamp,
            duration_ms,
            triggering_component,
            component_id,
            modified_operation,
            tags,
            retention_priority,
            context_size,
            contains_sensitive_data,
            metadata,
        })
    }

    /// Store a hook execution
    ///
    /// # Arguments
    /// * `execution` - Serialized hook execution with compressed context
    ///
    /// # Errors
    /// Returns error if insert fails or tenant context not set
    pub async fn store_execution(&self, execution: &SerializedHookExecution) -> Result<()> {
        let tenant_id = self
            .get_tenant_context()
            .await
            .map_err(|e| anyhow!("Tenant context error: {}", e))?;

        let client = self
            .backend
            .get_client()
            .await
            .map_err(|e| anyhow!("Database connection failed: {}", e))?;

        // Insert execution with all fields
        client
            .execute(
                "INSERT INTO llmspell.hook_history
                 (execution_id, tenant_id, hook_id, hook_type, correlation_id,
                  hook_context, result_data, timestamp, duration_ms,
                  triggering_component, component_id, modified_operation,
                  tags, retention_priority, context_size, contains_sensitive_data, metadata)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)",
                &[
                    &execution.execution_id,
                    &tenant_id,
                    &execution.hook_id,
                    &execution.hook_type,
                    &execution.correlation_id,
                    &execution.hook_context,
                    &execution.result_data,
                    &execution.timestamp,
                    &execution.duration_ms,
                    &execution.triggering_component,
                    &execution.component_id,
                    &execution.modified_operation,
                    &execution.tags,
                    &execution.retention_priority,
                    &execution.context_size,
                    &execution.contains_sensitive_data,
                    &execution.metadata,
                ],
            )
            .await
            .map_err(|e| anyhow!("Hook execution insert failed: {}", e))?;

        Ok(())
    }

    /// Load a hook execution by ID
    ///
    /// # Arguments
    /// * `execution_id` - Execution ID to load
    ///
    /// # Returns
    /// SerializedHookExecution if found, None otherwise
    ///
    /// # Performance
    /// Uses primary key index for <5ms lookup
    pub async fn load_execution(
        &self,
        execution_id: &Uuid,
    ) -> Result<Option<SerializedHookExecution>> {
        let tenant_id = self
            .get_tenant_context()
            .await
            .map_err(|e| anyhow!("Tenant context error: {}", e))?;

        let client = self
            .backend
            .get_client()
            .await
            .map_err(|e| anyhow!("Database connection failed: {}", e))?;

        // Primary key lookup
        let row = client
            .query_opt(
                "SELECT execution_id, hook_id, hook_type, correlation_id,
                        hook_context, result_data, timestamp, duration_ms,
                        triggering_component, component_id, modified_operation,
                        tags, retention_priority, context_size, contains_sensitive_data, metadata
                 FROM llmspell.hook_history
                 WHERE tenant_id = $1 AND execution_id = $2",
                &[&tenant_id, execution_id],
            )
            .await
            .map_err(|e| anyhow!("Execution load failed: {}", e))?;

        if let Some(row) = row {
            Ok(Some(Self::execution_from_row(&row)?))
        } else {
            Ok(None)
        }
    }

    /// Retrieve executions by correlation ID
    ///
    /// # Arguments
    /// * `correlation_id` - Correlation ID to search for
    ///
    /// # Returns
    /// Vector of executions sorted by timestamp
    ///
    /// # Performance
    /// Uses idx_hook_history_correlation index for <50ms lookup
    pub async fn get_executions_by_correlation_id(
        &self,
        correlation_id: &Uuid,
    ) -> Result<Vec<SerializedHookExecution>> {
        let tenant_id = self
            .get_tenant_context()
            .await
            .map_err(|e| anyhow!("Tenant context error: {}", e))?;

        let client = self
            .backend
            .get_client()
            .await
            .map_err(|e| anyhow!("Database connection failed: {}", e))?;

        // Uses idx_hook_history_correlation index
        let rows = client
            .query(
                "SELECT execution_id, hook_id, hook_type, correlation_id,
                        hook_context, result_data, timestamp, duration_ms,
                        triggering_component, component_id, modified_operation,
                        tags, retention_priority, context_size, contains_sensitive_data, metadata
                 FROM llmspell.hook_history
                 WHERE tenant_id = $1 AND correlation_id = $2
                 ORDER BY timestamp ASC",
                &[&tenant_id, correlation_id],
            )
            .await
            .map_err(|e| anyhow!("Correlation query failed: {}", e))?;

        let mut executions = Vec::with_capacity(rows.len());
        for row in &rows {
            executions.push(Self::execution_from_row(row)?);
        }

        Ok(executions)
    }

    /// Retrieve executions by hook ID
    ///
    /// # Arguments
    /// * `hook_id` - Hook ID to search for
    /// * `limit` - Maximum number of executions to return (most recent)
    ///
    /// # Returns
    /// Vector of executions sorted by timestamp (newest first)
    ///
    /// # Performance
    /// Uses idx_hook_history_hook_time index for <100ms lookup
    pub async fn get_executions_by_hook_id(
        &self,
        hook_id: &str,
        limit: Option<i64>,
    ) -> Result<Vec<SerializedHookExecution>> {
        let tenant_id = self
            .get_tenant_context()
            .await
            .map_err(|e| anyhow!("Tenant context error: {}", e))?;

        let client = self
            .backend
            .get_client()
            .await
            .map_err(|e| anyhow!("Database connection failed: {}", e))?;

        // Uses idx_hook_history_hook_time index
        let rows = if let Some(limit) = limit {
            client
                .query(
                    "SELECT execution_id, hook_id, hook_type, correlation_id,
                            hook_context, result_data, timestamp, duration_ms,
                            triggering_component, component_id, modified_operation,
                            tags, retention_priority, context_size, contains_sensitive_data, metadata
                     FROM llmspell.hook_history
                     WHERE tenant_id = $1 AND hook_id = $2
                     ORDER BY timestamp DESC
                     LIMIT $3",
                    &[&tenant_id, &hook_id, &limit],
                )
                .await
                .map_err(|e| anyhow!("Hook ID query failed: {}", e))?
        } else {
            client
                .query(
                    "SELECT execution_id, hook_id, hook_type, correlation_id,
                            hook_context, result_data, timestamp, duration_ms,
                            triggering_component, component_id, modified_operation,
                            tags, retention_priority, context_size, contains_sensitive_data, metadata
                     FROM llmspell.hook_history
                     WHERE tenant_id = $1 AND hook_id = $2
                     ORDER BY timestamp DESC",
                    &[&tenant_id, &hook_id],
                )
                .await
                .map_err(|e| anyhow!("Hook ID query failed: {}", e))?
        };

        let mut executions = Vec::with_capacity(rows.len());
        for row in &rows {
            executions.push(Self::execution_from_row(row)?);
        }

        Ok(executions)
    }

    /// Retrieve executions by hook type
    ///
    /// # Arguments
    /// * `hook_type` - Hook type to search for
    /// * `limit` - Maximum number of executions to return (most recent)
    ///
    /// # Returns
    /// Vector of executions sorted by timestamp (newest first)
    ///
    /// # Performance
    /// Uses idx_hook_history_type index
    pub async fn get_executions_by_type(
        &self,
        hook_type: &str,
        limit: Option<i64>,
    ) -> Result<Vec<SerializedHookExecution>> {
        let tenant_id = self
            .get_tenant_context()
            .await
            .map_err(|e| anyhow!("Tenant context error: {}", e))?;

        let client = self
            .backend
            .get_client()
            .await
            .map_err(|e| anyhow!("Database connection failed: {}", e))?;

        // Uses idx_hook_history_type index
        let rows = if let Some(limit) = limit {
            client
                .query(
                    "SELECT execution_id, hook_id, hook_type, correlation_id,
                            hook_context, result_data, timestamp, duration_ms,
                            triggering_component, component_id, modified_operation,
                            tags, retention_priority, context_size, contains_sensitive_data, metadata
                     FROM llmspell.hook_history
                     WHERE tenant_id = $1 AND hook_type = $2
                     ORDER BY timestamp DESC
                     LIMIT $3",
                    &[&tenant_id, &hook_type, &limit],
                )
                .await
                .map_err(|e| anyhow!("Hook type query failed: {}", e))?
        } else {
            client
                .query(
                    "SELECT execution_id, hook_id, hook_type, correlation_id,
                            hook_context, result_data, timestamp, duration_ms,
                            triggering_component, component_id, modified_operation,
                            tags, retention_priority, context_size, contains_sensitive_data, metadata
                     FROM llmspell.hook_history
                     WHERE tenant_id = $1 AND hook_type = $2
                     ORDER BY timestamp DESC",
                    &[&tenant_id, &hook_type],
                )
                .await
                .map_err(|e| anyhow!("Hook type query failed: {}", e))?
        };

        let mut executions = Vec::with_capacity(rows.len());
        for row in &rows {
            executions.push(Self::execution_from_row(row)?);
        }

        Ok(executions)
    }

    /// Archive (delete) old hook executions
    ///
    /// # Arguments
    /// * `before_date` - Delete executions older than this timestamp
    /// * `min_retention_priority` - Only delete executions with priority <= this value
    ///
    /// # Returns
    /// Number of executions deleted
    ///
    /// # Note
    /// Calls PostgreSQL function cleanup_old_hook_executions() which respects
    /// retention_priority to preserve important executions.
    pub async fn archive_executions(
        &self,
        before_date: DateTime<Utc>,
        min_retention_priority: i32,
    ) -> Result<usize> {
        let client = self
            .backend
            .get_client()
            .await
            .map_err(|e| anyhow!("Database connection failed: {}", e))?;

        // Call PostgreSQL cleanup function
        let row = client
            .query_one(
                "SELECT llmspell.cleanup_old_hook_executions($1, $2)",
                &[&before_date, &min_retention_priority],
            )
            .await
            .map_err(|e| anyhow!("Archive cleanup failed: {}", e))?;

        let deleted_count: i32 = row.get(0);
        Ok(deleted_count as usize)
    }

    /// Get storage statistics
    ///
    /// # Returns
    /// Statistics including total executions, storage size, timestamp range,
    /// executions by hook/type, and average duration
    ///
    /// # Note
    /// Calls PostgreSQL function get_hook_history_stats() which aggregates
    /// stats for the current tenant.
    pub async fn get_statistics(&self) -> Result<HookHistoryStats> {
        let client = self
            .backend
            .get_client()
            .await
            .map_err(|e| anyhow!("Database connection failed: {}", e))?;

        // Call PostgreSQL stats function (tenant-scoped)
        // Cast NUMERIC to DOUBLE PRECISION for direct f64 conversion
        let row = client
            .query_one(
                "SELECT
                    total_executions,
                    total_size_bytes,
                    executions_by_hook,
                    executions_by_type,
                    oldest_execution,
                    newest_execution,
                    avg_duration_ms::DOUBLE PRECISION as avg_duration_ms
                 FROM llmspell.get_hook_history_stats()",
                &[],
            )
            .await
            .map_err(|e| anyhow!("Stats query failed: {}", e))?;

        let total_executions: i64 = row.get("total_executions");
        let storage_size_bytes: i64 = row.get("total_size_bytes");
        let executions_by_hook: Value = row.get("executions_by_hook");
        let executions_by_type: Value = row.get("executions_by_type");
        let oldest_execution: Option<DateTime<Utc>> = row.get("oldest_execution");
        let newest_execution: Option<DateTime<Utc>> = row.get("newest_execution");
        let avg_duration_ms: f64 = row.get("avg_duration_ms");

        // Convert JSONB to HashMap
        let executions_by_hook_map: HashMap<String, u64> = executions_by_hook
            .as_object()
            .map(|obj| {
                obj.iter()
                    .map(|(k, v)| (k.clone(), v.as_u64().unwrap_or(0)))
                    .collect()
            })
            .unwrap_or_default();

        let executions_by_type_map: HashMap<String, u64> = executions_by_type
            .as_object()
            .map(|obj| {
                obj.iter()
                    .map(|(k, v)| (k.clone(), v.as_u64().unwrap_or(0)))
                    .collect()
            })
            .unwrap_or_default();

        Ok(HookHistoryStats {
            total_executions: total_executions as u64,
            storage_size_bytes: storage_size_bytes as u64,
            oldest_execution,
            newest_execution,
            executions_by_hook: executions_by_hook_map,
            executions_by_type: executions_by_type_map,
            avg_duration_ms,
        })
    }
}
