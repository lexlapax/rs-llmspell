//! PostgreSQL vector storage with multi-dimension support (Phase 13b.4.2)
//!
//! Routes vectors to appropriate table based on dimension:
//! - 384 dims → vector_embeddings_384
//! - 768 dims → vector_embeddings_768
//! - 1536 dims → vector_embeddings_1536
//! - 3072 dims → vector_embeddings_3072
//!
//! Uses pgvector HNSW indices for similarity search (384, 768, 1536 only -
//! 3072 has no vector index due to pgvector's 2000-dimension limit).

use super::backend::PostgresBackend;
use crate::vector_storage::{
    ScopedStats, StorageStats, VectorEntry, VectorQuery, VectorResult, VectorStorage,
};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use llmspell_core::state::StateScope;
use pgvector::Vector;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

/// PostgreSQL vector storage with multi-dimension support
///
/// Automatically routes vectors to the correct table based on their dimensions.
/// Supports 384, 768, 1536, and 3072-dimensional vectors via separate tables.
///
/// # RLS Integration
///
/// Tenant isolation enforced via PostgreSQL Row-Level Security (RLS).
/// Tenant context inherited from PostgresBackend connection pool.
pub struct PostgreSQLVectorStorage {
    backend: Arc<PostgresBackend>,
}

impl PostgreSQLVectorStorage {
    /// Create a new PostgreSQL vector storage
    pub fn new(backend: Arc<PostgresBackend>) -> Self {
        Self { backend }
    }

    /// Map dimension to table name
    ///
    /// # Errors
    ///
    /// Returns error if dimension is not one of: 384, 768, 1536, 3072
    fn get_table_name(dimension: usize) -> Result<&'static str> {
        match dimension {
            384 => Ok("vector_embeddings_384"),
            768 => Ok("vector_embeddings_768"),
            1536 => Ok("vector_embeddings_1536"),
            3072 => Ok("vector_embeddings_3072"),
            _ => Err(anyhow!(
                "Unsupported dimension: {}. Supported dimensions: 384, 768, 1536, 3072",
                dimension
            )),
        }
    }
}

#[async_trait]
impl VectorStorage for PostgreSQLVectorStorage {
    async fn insert(&self, vectors: Vec<VectorEntry>) -> Result<Vec<String>> {
        let client = self.backend.get_client().await?;
        let mut ids = Vec::new();

        for entry in vectors {
            let dimension = entry.embedding.len();
            let table = Self::get_table_name(dimension)?;

            // Tenant context automatically applied via RLS
            let query = format!(
                "INSERT INTO llmspell.{} (id, tenant_id, scope, embedding, metadata, created_at, updated_at)
                 VALUES ($1, current_setting('app.current_tenant_id', true), $2, $3, $4, $5, $6)
                 RETURNING id",
                table
            );

            // Convert String ID to UUID
            let id_uuid = uuid::Uuid::parse_str(&entry.id)
                .map_err(|e| anyhow!("Invalid UUID for id '{}': {}", entry.id, e))?;

            // Use database's now() for timestamps instead of client-side timestamps
            let row = client
                .query_one(
                    &query,
                    &[
                        &id_uuid,
                        &entry.scope.to_string(),
                        &Vector::from(entry.embedding),
                        &serde_json::to_value(&entry.metadata)?,
                        &std::time::SystemTime::now(),
                        &std::time::SystemTime::now(),
                    ],
                )
                .await?;

            let id: uuid::Uuid = row.get(0);
            ids.push(id.to_string());
        }

        Ok(ids)
    }

    async fn search(&self, query: &VectorQuery) -> Result<Vec<VectorResult>> {
        // Default to scope-based search if scope is provided
        if let Some(ref scope) = query.scope {
            return self.search_scoped(query, scope).await;
        }

        // No scope filtering - search within current tenant (via RLS)
        let dimension = query.vector.len();
        let table = Self::get_table_name(dimension)?;
        let client = self.backend.get_client().await?;

        // Build query (RLS automatically filters by tenant)
        let sql = format!(
            "SELECT id, scope, embedding, metadata,
                    embedding <=> $1::vector AS distance
             FROM llmspell.{}
             ORDER BY distance
             LIMIT $2",
            table
        );

        let rows = client
            .query(
                &sql,
                &[&Vector::from(query.vector.clone()), &(query.k as i64)],
            )
            .await?;

        let results = rows
            .into_iter()
            .filter_map(|row| {
                let distance: f64 = row.get("distance");
                let distance_f32 = distance as f32;

                // Apply threshold filter
                let similarity = 1.0 - distance_f32;
                if let Some(threshold) = query.threshold {
                    if similarity < threshold {
                        return None;
                    }
                }

                let id_uuid: uuid::Uuid = row.get("id");
                let id = id_uuid.to_string();
                let _scope_str: String = row.get("scope");
                let embedding: Vector = row.get("embedding");
                let metadata_value: Value = row.get("metadata");

                Some(VectorResult {
                    id,
                    score: similarity,
                    vector: Some(embedding.to_vec()),
                    metadata: if query.include_metadata {
                        metadata_value
                            .as_object()
                            .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                    } else {
                        None
                    },
                    distance: distance_f32,
                })
            })
            .collect();

        Ok(results)
    }

    async fn search_scoped(
        &self,
        query: &VectorQuery,
        scope: &StateScope,
    ) -> Result<Vec<VectorResult>> {
        let dimension = query.vector.len();
        let table = Self::get_table_name(dimension)?;
        let client = self.backend.get_client().await?;

        // Build query with scope filtering (in addition to RLS tenant filtering)
        let sql = format!(
            "SELECT id, scope, embedding, metadata,
                    embedding <=> $1::vector AS distance
             FROM llmspell.{}
             WHERE scope = $2
             ORDER BY distance
             LIMIT $3",
            table
        );

        let rows = client
            .query(
                &sql,
                &[
                    &Vector::from(query.vector.clone()),
                    &scope.to_string(),
                    &(query.k as i64),
                ],
            )
            .await?;

        let results = rows
            .into_iter()
            .filter_map(|row| {
                let distance: f64 = row.get("distance");
                let distance_f32 = distance as f32;

                // Apply threshold filter
                let similarity = 1.0 - distance_f32;
                if let Some(threshold) = query.threshold {
                    if similarity < threshold {
                        return None;
                    }
                }

                let id_uuid: uuid::Uuid = row.get("id");
                let id = id_uuid.to_string();
                let _scope_str: String = row.get("scope");
                let embedding: Vector = row.get("embedding");
                let metadata_value: Value = row.get("metadata");

                Some(VectorResult {
                    id,
                    score: similarity,
                    vector: Some(embedding.to_vec()),
                    metadata: if query.include_metadata {
                        metadata_value
                            .as_object()
                            .map(|obj| obj.iter().map(|(k, v)| (k.clone(), v.clone())).collect())
                    } else {
                        None
                    },
                    distance: distance_f32,
                })
            })
            .collect();

        Ok(results)
    }

    async fn update_metadata(&self, id: &str, metadata: HashMap<String, Value>) -> Result<()> {
        let client = self.backend.get_client().await?;

        // Convert ID to UUID
        let id_uuid = uuid::Uuid::parse_str(id)
            .map_err(|e| anyhow!("Invalid UUID for id '{}': {}", id, e))?;

        // Try updating in all 4 tables (we don't know which dimension without querying)
        for dimension in [384, 768, 1536, 3072] {
            let table = Self::get_table_name(dimension)?;
            let query = format!(
                "UPDATE llmspell.{} SET metadata = $1, updated_at = now() WHERE id = $2",
                table
            );

            let rows_affected = client
                .execute(&query, &[&serde_json::to_value(&metadata)?, &id_uuid])
                .await?;

            if rows_affected > 0 {
                return Ok(()); // Found and updated
            }
        }

        Err(anyhow!("Vector with ID {} not found", id))
    }

    async fn delete(&self, ids: &[String]) -> Result<()> {
        let client = self.backend.get_client().await?;

        // Try deleting from all 4 tables (vectors could be in any dimension table)
        for dimension in [384, 768, 1536, 3072] {
            let table = Self::get_table_name(dimension)?;
            let query = format!("DELETE FROM llmspell.{} WHERE id = ANY($1)", table);
            let _ = client.execute(&query, &[&ids]).await; // Ignore errors (ID might not exist in this table)
        }

        Ok(())
    }

    async fn delete_scope(&self, scope: &StateScope) -> Result<usize> {
        let client = self.backend.get_client().await?;
        let mut total_deleted = 0;

        // Delete from all 4 tables
        for dimension in [384, 768, 1536, 3072] {
            let table = Self::get_table_name(dimension)?;
            let query = format!("DELETE FROM llmspell.{} WHERE scope = $1", table);

            let rows_affected = client.execute(&query, &[&scope.to_string()]).await?;
            total_deleted += rows_affected as usize;
        }

        Ok(total_deleted)
    }

    async fn stats(&self) -> Result<StorageStats> {
        let client = self.backend.get_client().await?;
        let mut total_vectors = 0;

        // Aggregate stats from all 4 tables
        for dimension in [384, 768, 1536, 3072] {
            let table = Self::get_table_name(dimension)?;
            let query = format!(
                "SELECT COUNT(*) FROM llmspell.{}
                 WHERE tenant_id = current_setting('app.current_tenant_id', true)",
                table
            );

            let row = client.query_one(&query, &[]).await?;
            let count: i64 = row.get(0);
            total_vectors += count as usize;
        }

        Ok(StorageStats {
            total_vectors,
            storage_bytes: 0,        // TODO: Calculate from pg_total_relation_size
            namespace_count: 1,      // Single tenant via RLS
            avg_query_time_ms: None, // TODO: Track query performance
            dimensions: None,        // Multiple dimensions
            index_build_time_ms: None,
        })
    }

    async fn stats_for_scope(&self, scope: &StateScope) -> Result<ScopedStats> {
        let client = self.backend.get_client().await?;
        let mut total_vectors = 0;

        // Aggregate stats from all 4 tables for this scope
        for dimension in [384, 768, 1536, 3072] {
            let table = Self::get_table_name(dimension)?;
            let query = format!(
                "SELECT COUNT(*) FROM llmspell.{}
                 WHERE tenant_id = current_setting('app.current_tenant_id', true)
                   AND scope = $1",
                table
            );

            let row = client.query_one(&query, &[&scope.to_string()]).await?;
            let count: i64 = row.get(0);
            total_vectors += count as usize;
        }

        Ok(ScopedStats {
            scope: scope.clone(),
            vector_count: total_vectors,
            storage_bytes: 0,
            query_count: 0,
            tokens_processed: 0,
            estimated_cost: 0.0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_table_name_supported_dimensions() {
        assert_eq!(
            PostgreSQLVectorStorage::get_table_name(384).unwrap(),
            "vector_embeddings_384"
        );
        assert_eq!(
            PostgreSQLVectorStorage::get_table_name(768).unwrap(),
            "vector_embeddings_768"
        );
        assert_eq!(
            PostgreSQLVectorStorage::get_table_name(1536).unwrap(),
            "vector_embeddings_1536"
        );
        assert_eq!(
            PostgreSQLVectorStorage::get_table_name(3072).unwrap(),
            "vector_embeddings_3072"
        );
    }

    #[test]
    fn test_get_table_name_unsupported_dimension() {
        let result = PostgreSQLVectorStorage::get_table_name(999);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unsupported dimension: 999"));
    }
}
