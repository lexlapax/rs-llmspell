//! Storage exporter for data migration
//!
//! Exports data from PostgreSQL or SQLite backends to standardized JSON format
//! for bidirectional migration and backup.

use super::converters::TypeConverters;
use super::format::*;
use anyhow::{Context, Result};
use base64::Engine;
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

// ============================================================================
// PostgreSQL Exporter
// ============================================================================

#[cfg(feature = "postgres")]
use crate::backends::postgres::PostgresBackend;

#[cfg(feature = "postgres")]
pub struct PostgresExporter {
    backend: Arc<PostgresBackend>,
    #[allow(dead_code)]
    converters: TypeConverters,
}

#[cfg(feature = "postgres")]
impl PostgresExporter {
    pub fn new(backend: Arc<PostgresBackend>) -> Self {
        Self {
            backend,
            converters: TypeConverters::new(),
        }
    }

    /// Export all data to JSON format
    pub async fn export_all(&self) -> Result<ExportFormat> {
        let migrations = self.detect_migrations().await?;
        let mut export = ExportFormat::new("postgresql".to_string(), migrations.clone());

        tracing::info!("Starting PostgreSQL export");

        // Export each data type
        if migrations.contains(&"V3".to_string()) {
            export.data.vector_embeddings = self.export_vector_embeddings().await?;
        }
        if migrations.contains(&"V4".to_string()) {
            export.data.knowledge_graph = Some(self.export_knowledge_graph().await?);
        }
        if migrations.contains(&"V5".to_string()) {
            export.data.procedural_memory = self.export_procedural_memory().await?;
        }
        if migrations.contains(&"V6".to_string()) {
            export.data.agent_state = self.export_agent_state().await?;
        }
        if migrations.contains(&"V7".to_string()) {
            export.data.kv_store = self.export_kv_store().await?;
        }
        if migrations.contains(&"V8".to_string()) {
            export.data.workflow_states = self.export_workflow_states().await?;
        }
        if migrations.contains(&"V9".to_string()) {
            export.data.sessions = self.export_sessions().await?;
        }
        if migrations.contains(&"V10".to_string()) {
            export.data.artifacts = Some(self.export_artifacts().await?);
        }
        if migrations.contains(&"V11".to_string()) {
            export.data.event_log = self.export_event_log().await?;
        }
        if migrations.contains(&"V13".to_string()) {
            export.data.hook_history = self.export_hook_history().await?;
        }

        tracing::info!("PostgreSQL export completed");
        Ok(export)
    }

    async fn detect_migrations(&self) -> Result<Vec<String>> {
        let client = self.backend.get_client().await?;
        let rows = client
            .query(
                "SELECT version FROM llmspell.refinery_schema_history ORDER BY version",
                &[],
            )
            .await?;

        Ok(rows
            .iter()
            .map(|row| {
                let version: i32 = row.get(0);
                format!("V{}", version)
            })
            .collect())
    }

    async fn export_vector_embeddings(&self) -> Result<HashMap<usize, Vec<VectorEmbeddingExport>>> {
        let mut result: HashMap<usize, Vec<VectorEmbeddingExport>> = HashMap::new();
        let client = self.backend.get_client().await?;

        for dim in [384, 768, 1536, 3072] {
            let table = format!("llmspell.vector_embeddings_{}", dim);
            let query = format!(
                "SELECT id, tenant_id, scope, embedding::text, metadata,
                 EXTRACT(EPOCH FROM created_at) * 1000000,
                 EXTRACT(EPOCH FROM updated_at) * 1000000
                 FROM {}",
                table
            );

            let rows = client.query(&query, &[]).await?;
            let mut vectors = Vec::new();

            for row in rows {
                let id: uuid::Uuid = row.get(0);
                let metadata_str: String = row.get(4);
                let metadata: serde_json::Value = serde_json::from_str(&metadata_str)?;

                vectors.push(VectorEmbeddingExport {
                    id: id.to_string(),
                    tenant_id: row.get(1),
                    scope: row.get(2),
                    dimension: dim,
                    embedding: row.get::<_, String>(3), // Vector as text
                    metadata,
                    created_at: row.get::<_, f64>(5) as i64,
                    updated_at: row.get::<_, f64>(6) as i64,
                });
            }

            if !vectors.is_empty() {
                result.insert(dim, vectors);
            }
        }

        Ok(result)
    }

    async fn export_knowledge_graph(&self) -> Result<KnowledgeGraphExport> {
        let client = self.backend.get_client().await?;

        // Export entities
        let entity_rows = client
            .query(
                "SELECT entity_id, tenant_id, entity_type, name, properties,
                 EXTRACT(EPOCH FROM valid_time_start) * 1000000,
                 EXTRACT(EPOCH FROM valid_time_end) * 1000000,
                 EXTRACT(EPOCH FROM transaction_time_start) * 1000000,
                 EXTRACT(EPOCH FROM transaction_time_end) * 1000000,
                 EXTRACT(EPOCH FROM created_at) * 1000000
                 FROM llmspell.entities",
                &[],
            )
            .await?;

        let mut entities = Vec::new();
        for row in entity_rows {
            let id: uuid::Uuid = row.get(0);
            let props_str: String = row.get(4);
            let properties: serde_json::Value = serde_json::from_str(&props_str)?;

            entities.push(EntityExport {
                entity_id: id.to_string(),
                tenant_id: row.get(1),
                entity_type: row.get(2),
                name: row.get(3),
                properties,
                valid_time_start: row.get::<_, f64>(5) as i64,
                valid_time_end: row.get::<_, f64>(6) as i64,
                transaction_time_start: row.get::<_, f64>(7) as i64,
                transaction_time_end: row.get::<_, f64>(8) as i64,
                created_at: row.get::<_, f64>(9) as i64,
            });
        }

        // Export relationships
        let rel_rows = client
            .query(
                "SELECT relationship_id, tenant_id, from_entity, to_entity,
                 relationship_type, properties,
                 EXTRACT(EPOCH FROM valid_time_start) * 1000000,
                 EXTRACT(EPOCH FROM valid_time_end) * 1000000,
                 EXTRACT(EPOCH FROM transaction_time_start) * 1000000,
                 EXTRACT(EPOCH FROM transaction_time_end) * 1000000,
                 EXTRACT(EPOCH FROM created_at) * 1000000
                 FROM llmspell.relationships",
                &[],
            )
            .await?;

        let mut relationships = Vec::new();
        for row in rel_rows {
            let id: uuid::Uuid = row.get(0);
            let source: uuid::Uuid = row.get(2);
            let target: uuid::Uuid = row.get(3);
            let props_str: String = row.get(5);
            let properties: serde_json::Value = serde_json::from_str(&props_str)?;

            relationships.push(RelationshipExport {
                relationship_id: id.to_string(),
                tenant_id: row.get(1),
                source_entity_id: source.to_string(),
                target_entity_id: target.to_string(),
                relationship_type: row.get(4),
                properties,
                valid_time_start: row.get::<_, f64>(6) as i64,
                valid_time_end: row.get::<_, f64>(7) as i64,
                transaction_time_start: row.get::<_, f64>(8) as i64,
                transaction_time_end: row.get::<_, f64>(9) as i64,
                created_at: row.get::<_, f64>(10) as i64,
            });
        }

        Ok(KnowledgeGraphExport {
            entities,
            relationships,
        })
    }

    async fn export_procedural_memory(&self) -> Result<Vec<PatternExport>> {
        let client = self.backend.get_client().await?;
        let rows = client
            .query(
                "SELECT pattern_id, tenant_id, scope, key, value, frequency,
                 EXTRACT(EPOCH FROM first_seen) * 1000000,
                 EXTRACT(EPOCH FROM last_seen) * 1000000,
                 EXTRACT(EPOCH FROM created_at) * 1000000,
                 EXTRACT(EPOCH FROM updated_at) * 1000000
                 FROM llmspell.procedural_patterns",
                &[],
            )
            .await?;

        let mut patterns = Vec::new();
        for row in rows {
            let id: uuid::Uuid = row.get(0);

            patterns.push(PatternExport {
                pattern_id: id.to_string(),
                tenant_id: row.get(1),
                scope: row.get(2),
                key: row.get(3),
                value: row.get(4),
                frequency: row.get(5),
                first_seen: row.get::<_, f64>(6) as i64,
                last_seen: row.get::<_, f64>(7) as i64,
                created_at: row.get::<_, f64>(8) as i64,
                updated_at: row.get::<_, f64>(9) as i64,
            });
        }

        Ok(patterns)
    }

    async fn export_agent_state(&self) -> Result<Vec<AgentStateExport>> {
        let client = self.backend.get_client().await?;
        let rows = client
            .query(
                "SELECT state_id, tenant_id, agent_id, agent_type, state_data,
                 schema_version, data_version, checksum,
                 EXTRACT(EPOCH FROM created_at) * 1000000,
                 EXTRACT(EPOCH FROM updated_at) * 1000000
                 FROM llmspell.agent_states",
                &[],
            )
            .await?;

        let mut states = Vec::new();
        for row in rows {
            let state_id: uuid::Uuid = row.get(0);
            let agent_id: uuid::Uuid = row.get(2);
            let data_str: String = row.get(4);
            let state_data: serde_json::Value = serde_json::from_str(&data_str)?;

            states.push(AgentStateExport {
                state_id: state_id.to_string(),
                tenant_id: row.get(1),
                agent_id: agent_id.to_string(),
                agent_type: row.get(3),
                state_data,
                schema_version: row.get(5),
                data_version: row.get(6),
                checksum: row.get(7),
                created_at: row.get::<_, f64>(8) as i64,
                updated_at: row.get::<_, f64>(9) as i64,
            });
        }

        Ok(states)
    }

    async fn export_kv_store(&self) -> Result<Vec<KVEntryExport>> {
        let client = self.backend.get_client().await?;
        let rows = client
            .query(
                "SELECT kv_id, tenant_id, key, value, metadata,
                 EXTRACT(EPOCH FROM created_at) * 1000000,
                 EXTRACT(EPOCH FROM updated_at) * 1000000
                 FROM llmspell.kv_store",
                &[],
            )
            .await?;

        let mut entries = Vec::new();
        for row in rows {
            let kv_id: uuid::Uuid = row.get(0);
            let value_bytes: Vec<u8> = row.get(3);
            let value_base64 = base64::engine::general_purpose::STANDARD.encode(&value_bytes);
            let metadata_opt: Option<String> = row.get(4);

            entries.push(KVEntryExport {
                kv_id: kv_id.to_string(),
                tenant_id: row.get(1),
                key: row.get(2),
                value: value_base64,
                metadata: metadata_opt,
                created_at: row.get::<_, f64>(5) as i64,
                updated_at: row.get::<_, f64>(6) as i64,
            });
        }

        Ok(entries)
    }

    async fn export_workflow_states(&self) -> Result<Vec<WorkflowStateExport>> {
        let client = self.backend.get_client().await?;
        let rows = client
            .query(
                "SELECT tenant_id, workflow_id, workflow_name, state_data, current_step, status,
                 started_at, completed_at,
                 EXTRACT(EPOCH FROM last_updated) * 1000000,
                 EXTRACT(EPOCH FROM created_at) * 1000000
                 FROM llmspell.workflow_states",
                &[],
            )
            .await?;

        let mut states = Vec::new();
        for row in rows {
            let data_str: String = row.get(3);
            let state_data: serde_json::Value = serde_json::from_str(&data_str)?;
            let started_opt: Option<i64> = row.get(6);
            let completed_opt: Option<i64> = row.get(7);

            states.push(WorkflowStateExport {
                tenant_id: row.get(0),
                workflow_id: row.get(1),
                workflow_name: row.get(2),
                state_data,
                current_step: row.get(4),
                status: row.get(5),
                started_at: started_opt,
                completed_at: completed_opt,
                last_updated: row.get::<_, f64>(8) as i64,
                created_at: row.get::<_, f64>(9) as i64,
            });
        }

        Ok(states)
    }

    async fn export_sessions(&self) -> Result<Vec<SessionExport>> {
        let client = self.backend.get_client().await?;
        let rows = client
            .query(
                "SELECT id, tenant_id, session_id, session_data, status,
                 EXTRACT(EPOCH FROM created_at) * 1000000,
                 EXTRACT(EPOCH FROM last_accessed_at) * 1000000,
                 expires_at, artifact_count,
                 EXTRACT(EPOCH FROM updated_at) * 1000000
                 FROM llmspell.sessions",
                &[],
            )
            .await?;

        let mut sessions = Vec::new();
        for row in rows {
            let id: uuid::Uuid = row.get(0);
            let session_id: String = row.get(2);
            let data_str: String = row.get(3);
            let session_data: serde_json::Value = serde_json::from_str(&data_str)?;
            let expires_opt: Option<i64> = row.get(7);

            sessions.push(SessionExport {
                id: id.to_string(),
                tenant_id: row.get(1),
                session_id,
                session_data,
                status: row.get(4),
                created_at: row.get::<_, f64>(5) as i64,
                last_accessed_at: row.get::<_, f64>(6) as i64,
                expires_at: expires_opt,
                artifact_count: row.get(8),
                updated_at: row.get::<_, f64>(9) as i64,
            });
        }

        Ok(sessions)
    }

    async fn export_artifacts(&self) -> Result<ArtifactsExport> {
        let client = self.backend.get_client().await?;

        // Export artifact content
        let content_rows = client
            .query(
                "SELECT tenant_id, content_hash, storage_type, data, large_object_oid,
                 size_bytes, is_compressed, original_size_bytes, reference_count,
                 EXTRACT(EPOCH FROM created_at) * 1000000,
                 EXTRACT(EPOCH FROM last_accessed_at) * 1000000
                 FROM llmspell.artifact_content",
                &[],
            )
            .await?;

        let mut content = Vec::new();
        for row in content_rows {
            let storage_type: String = row.get(2);
            let data = if storage_type == "bytea" {
                let bytes: Vec<u8> = row.get(3);
                Some(base64::Engine::encode(
                    &base64::engine::general_purpose::STANDARD,
                    bytes,
                ))
            } else {
                // Large object handling would go here in full implementation
                None
            };

            content.push(ArtifactContentExport {
                tenant_id: row.get(0),
                content_hash: row.get(1),
                storage_type,
                data,
                size_bytes: row.get(5),
                is_compressed: row.get(6),
                original_size_bytes: row.get(7),
                reference_count: row.get(8),
                created_at: row.get::<_, f64>(9) as i64,
                last_accessed_at: row.get::<_, f64>(10) as i64,
            });
        }

        // Export artifact metadata
        let meta_rows = client
            .query(
                "SELECT tenant_id, artifact_id, session_id, sequence, content_hash,
                 metadata, name, artifact_type, mime_type, size_bytes,
                 EXTRACT(EPOCH FROM created_at) * 1000000,
                 created_by, version, parent_artifact_id, tags,
                 EXTRACT(EPOCH FROM stored_at) * 1000000,
                 EXTRACT(EPOCH FROM updated_at) * 1000000
                 FROM llmspell.artifacts",
                &[],
            )
            .await?;

        let mut artifacts = Vec::new();
        for row in meta_rows {
            let session_id: uuid::Uuid = row.get(2);
            let meta_str: String = row.get(5);
            let metadata: serde_json::Value = serde_json::from_str(&meta_str)?;
            let tags: Vec<String> = row.get(14);

            artifacts.push(ArtifactMetadataExport {
                tenant_id: row.get(0),
                artifact_id: row.get(1),
                session_id: session_id.to_string(),
                sequence: row.get(3),
                content_hash: row.get(4),
                metadata,
                name: row.get(6),
                artifact_type: row.get(7),
                mime_type: row.get(8),
                size_bytes: row.get(9),
                created_at: row.get::<_, f64>(10) as i64,
                created_by: row.get(11),
                version: row.get(12),
                parent_artifact_id: row.get(13),
                tags,
                stored_at: row.get::<_, f64>(15) as i64,
                updated_at: row.get::<_, f64>(16) as i64,
            });
        }

        Ok(ArtifactsExport { content, artifacts })
    }

    async fn export_event_log(&self) -> Result<Vec<EventExport>> {
        let client = self.backend.get_client().await?;
        let rows = client
            .query(
                "SELECT id, tenant_id, event_id, event_type, correlation_id,
                 EXTRACT(EPOCH FROM timestamp) * 1000000,
                 sequence, language, payload
                 FROM llmspell.event_log",
                &[],
            )
            .await?;

        let mut events = Vec::new();
        for row in rows {
            let id: uuid::Uuid = row.get(0);
            let payload_str: String = row.get(8);
            let payload: serde_json::Value = serde_json::from_str(&payload_str)?;

            events.push(EventExport {
                id: id.to_string(),
                tenant_id: row.get(1),
                event_id: row.get(2),
                event_type: row.get(3),
                correlation_id: row.get(4),
                timestamp: row.get::<_, f64>(5) as i64,
                sequence: row.get(6),
                language: row.get(7),
                payload,
            });
        }

        Ok(events)
    }

    async fn export_hook_history(&self) -> Result<Vec<HookExport>> {
        let client = self.backend.get_client().await?;
        let rows = client
            .query(
                "SELECT id, execution_id, tenant_id, hook_id, hook_type, correlation_id,
                 hook_context, result_data,
                 EXTRACT(EPOCH FROM timestamp) * 1000000,
                 duration_ms
                 FROM llmspell.hook_history",
                &[],
            )
            .await?;

        let mut hooks = Vec::new();
        for row in rows {
            let id: uuid::Uuid = row.get(0);
            let context_bytes: Vec<u8> = row.get(6);
            let hook_context_base64 =
                base64::engine::general_purpose::STANDARD.encode(&context_bytes);
            let result_str: String = row.get(7);
            let result_data: serde_json::Value = serde_json::from_str(&result_str)?;

            hooks.push(HookExport {
                id: id.to_string(),
                execution_id: row.get(1),
                tenant_id: row.get(2),
                hook_id: row.get(3),
                hook_type: row.get(4),
                correlation_id: row.get(5),
                hook_context: hook_context_base64,
                result_data,
                timestamp: row.get::<_, f64>(8) as i64,
                duration_ms: row.get(9),
            });
        }

        Ok(hooks)
    }

    pub async fn export_to_file<P: AsRef<Path>>(&self, output_path: P) -> Result<()> {
        let export = self.export_all().await?;
        let json = serde_json::to_string_pretty(&export)?;
        std::fs::write(output_path.as_ref(), json)?;
        tracing::info!("Export written to: {}", output_path.as_ref().display());
        Ok(())
    }

    pub async fn export_to_json(&self) -> Result<String> {
        let export = self.export_all().await?;
        serde_json::to_string_pretty(&export).context("Failed to serialize export data")
    }
}

// ============================================================================
// SQLite Exporter
// ============================================================================

#[cfg(feature = "sqlite")]
use crate::backends::sqlite::SqliteBackend;

#[cfg(feature = "sqlite")]
pub struct SqliteExporter {
    backend: Arc<SqliteBackend>,
    #[allow(dead_code)]
    converters: TypeConverters,
}

#[cfg(feature = "sqlite")]
impl SqliteExporter {
    pub fn new(backend: Arc<SqliteBackend>) -> Self {
        Self {
            backend,
            converters: TypeConverters::new(),
        }
    }

    /// Export all data to JSON format
    pub async fn export_all(&self) -> Result<ExportFormat> {
        let migrations = self.detect_migrations().await?;
        let mut export = ExportFormat::new("sqlite".to_string(), migrations.clone());

        tracing::info!("Starting SQLite export");

        // Export each data type
        if migrations.contains(&"V3".to_string()) {
            export.data.vector_embeddings = self.export_vector_embeddings().await?;
        }
        if migrations.contains(&"V4".to_string()) {
            export.data.knowledge_graph = Some(self.export_knowledge_graph().await?);
        }
        if migrations.contains(&"V5".to_string()) {
            export.data.procedural_memory = self.export_procedural_memory().await?;
        }
        if migrations.contains(&"V6".to_string()) {
            export.data.agent_state = self.export_agent_state().await?;
        }
        if migrations.contains(&"V7".to_string()) {
            export.data.kv_store = self.export_kv_store().await?;
        }
        if migrations.contains(&"V8".to_string()) {
            export.data.workflow_states = self.export_workflow_states().await?;
        }
        if migrations.contains(&"V9".to_string()) {
            export.data.sessions = self.export_sessions().await?;
        }
        if migrations.contains(&"V10".to_string()) {
            export.data.artifacts = Some(self.export_artifacts().await?);
        }
        if migrations.contains(&"V11".to_string()) {
            export.data.event_log = self.export_event_log().await?;
        }
        if migrations.contains(&"V13".to_string()) {
            export.data.hook_history = self.export_hook_history().await?;
        }

        tracing::info!("SQLite export completed");
        Ok(export)
    }

    async fn detect_migrations(&self) -> Result<Vec<String>> {
        // SQLite doesn't have refinery_schema_history, but we can detect tables
        Ok(vec![
            "V3".to_string(),
            "V4".to_string(),
            "V5".to_string(),
            "V6".to_string(),
            "V7".to_string(),
            "V8".to_string(),
            "V9".to_string(),
            "V10".to_string(),
            "V11".to_string(),
            "V13".to_string(),
        ])
    }

    async fn export_vector_embeddings(&self) -> Result<HashMap<usize, Vec<VectorEmbeddingExport>>> {
        let conn = self.backend.get_connection().await?;

        // SQLite stores vectors in vector_metadata table
        let stmt = conn
            .prepare("SELECT id, tenant_id, scope, dimension, metadata, created_at, updated_at FROM vector_metadata")
            .await?;

        let mut rows = stmt.query(libsql::params![]).await?;
        let mut all_vectors: HashMap<usize, Vec<VectorEmbeddingExport>> = HashMap::new();

        while let Some(row) = rows.next().await? {
            let id: String = row.get(0)?;
            let dimension: i32 = row.get(3)?;
            let dim = dimension as usize;
            let metadata_str: String = row.get(4)?;
            let metadata: serde_json::Value = serde_json::from_str(&metadata_str)?;

            let entry = all_vectors.entry(dim).or_default();
            entry.push(VectorEmbeddingExport {
                id,
                tenant_id: row.get(1)?,
                scope: row.get(2)?,
                dimension: dim,
                embedding: String::new(), // Embedding stored separately in vectorlite
                metadata,
                created_at: row.get::<i64>(5)?,
                updated_at: row.get::<i64>(6)?,
            });
        }

        Ok(all_vectors)
    }

    async fn export_knowledge_graph(&self) -> Result<KnowledgeGraphExport> {
        let conn = self.backend.get_connection().await?;

        // Export entities
        let stmt = conn
            .prepare(
                "SELECT entity_id, tenant_id, entity_type, name, properties,
                     valid_time_start, valid_time_end, transaction_time_start,
                     transaction_time_end, created_at FROM entities",
            )
            .await?;

        let mut rows = stmt.query(libsql::params![]).await?;
        let mut entities = Vec::new();

        while let Some(row) = rows.next().await? {
            let props_str: String = row.get(4)?;
            let properties: serde_json::Value = serde_json::from_str(&props_str)?;

            entities.push(EntityExport {
                entity_id: row.get(0)?,
                tenant_id: row.get(1)?,
                entity_type: row.get(2)?,
                name: row.get(3)?,
                properties,
                valid_time_start: row.get(5)?,
                valid_time_end: row.get(6)?,
                transaction_time_start: row.get(7)?,
                transaction_time_end: row.get(8)?,
                created_at: row.get(9)?,
            });
        }

        // Export relationships
        let stmt = conn
            .prepare(
                "SELECT relationship_id, tenant_id, from_entity, to_entity,
                     relationship_type, properties, valid_time_start, valid_time_end,
                     transaction_time_start, transaction_time_end, created_at FROM relationships",
            )
            .await?;

        let mut rows = stmt.query(libsql::params![]).await?;
        let mut relationships = Vec::new();

        while let Some(row) = rows.next().await? {
            let props_str: String = row.get(5)?;
            let properties: serde_json::Value = serde_json::from_str(&props_str)?;

            relationships.push(RelationshipExport {
                relationship_id: row.get(0)?,
                tenant_id: row.get(1)?,
                source_entity_id: row.get(2)?,
                target_entity_id: row.get(3)?,
                relationship_type: row.get(4)?,
                properties,
                valid_time_start: row.get(6)?,
                valid_time_end: row.get(7)?,
                transaction_time_start: row.get(8)?,
                transaction_time_end: row.get(9)?,
                created_at: row.get(10)?,
            });
        }

        Ok(KnowledgeGraphExport {
            entities,
            relationships,
        })
    }

    async fn export_procedural_memory(&self) -> Result<Vec<PatternExport>> {
        let conn = self.backend.get_connection().await?;
        let stmt = conn
            .prepare(
                "SELECT pattern_id, tenant_id, scope, key, value, frequency,
                     first_seen, last_seen, created_at, updated_at FROM procedural_patterns",
            )
            .await?;

        let mut rows = stmt.query(libsql::params![]).await?;
        let mut patterns = Vec::new();

        while let Some(row) = rows.next().await? {
            patterns.push(PatternExport {
                pattern_id: row.get(0)?,
                tenant_id: row.get(1)?,
                scope: row.get(2)?,
                key: row.get(3)?,
                value: row.get(4)?,
                frequency: row.get(5)?,
                first_seen: row.get(6)?,
                last_seen: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            });
        }

        Ok(patterns)
    }

    async fn export_agent_state(&self) -> Result<Vec<AgentStateExport>> {
        let conn = self.backend.get_connection().await?;
        let stmt = conn
            .prepare(
                "SELECT state_id, tenant_id, agent_id, agent_type, state_data,
                     schema_version, data_version, checksum, created_at, updated_at FROM agent_states",
            )
            .await?;

        let mut rows = stmt.query(libsql::params![]).await?;
        let mut states = Vec::new();

        while let Some(row) = rows.next().await? {
            let data_str: String = row.get(4)?;
            let state_data: serde_json::Value = serde_json::from_str(&data_str)?;

            states.push(AgentStateExport {
                state_id: row.get(0)?,
                tenant_id: row.get(1)?,
                agent_id: row.get(2)?,
                agent_type: row.get(3)?,
                state_data,
                schema_version: row.get(5)?,
                data_version: row.get(6)?,
                checksum: row.get(7)?,
                created_at: row.get(8)?,
                updated_at: row.get(9)?,
            });
        }

        Ok(states)
    }

    async fn export_kv_store(&self) -> Result<Vec<KVEntryExport>> {
        let conn = self.backend.get_connection().await?;
        let stmt = conn
            .prepare("SELECT kv_id, tenant_id, key, value, metadata, created_at, updated_at FROM kv_store")
            .await?;

        let mut rows = stmt.query(libsql::params![]).await?;
        let mut entries = Vec::new();

        while let Some(row) = rows.next().await? {
            let value_bytes: Vec<u8> = row.get(3)?;
            let value_base64 = base64::engine::general_purpose::STANDARD.encode(&value_bytes);
            let metadata_opt: Option<String> = row.get(4)?;

            entries.push(KVEntryExport {
                kv_id: row.get(0)?,
                tenant_id: row.get(1)?,
                key: row.get(2)?,
                value: value_base64,
                metadata: metadata_opt,
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
            });
        }

        Ok(entries)
    }

    async fn export_workflow_states(&self) -> Result<Vec<WorkflowStateExport>> {
        let conn = self.backend.get_connection().await?;
        let stmt = conn
            .prepare(
                "SELECT tenant_id, workflow_id, workflow_name, state_data, current_step, status,
                     started_at, completed_at, last_updated, created_at FROM workflow_states",
            )
            .await?;

        let mut rows = stmt.query(libsql::params![]).await?;
        let mut states = Vec::new();

        while let Some(row) = rows.next().await? {
            let data_str: String = row.get(3)?;
            let state_data: serde_json::Value = serde_json::from_str(&data_str)?;
            let started_opt: Option<i64> = row.get(6)?;
            let completed_opt: Option<i64> = row.get(7)?;

            states.push(WorkflowStateExport {
                tenant_id: row.get(0)?,
                workflow_id: row.get(1)?,
                workflow_name: row.get(2)?,
                state_data,
                current_step: row.get(4)?,
                status: row.get(5)?,
                started_at: started_opt,
                completed_at: completed_opt,
                last_updated: row.get(8)?,
                created_at: row.get(9)?,
            });
        }

        Ok(states)
    }

    async fn export_sessions(&self) -> Result<Vec<SessionExport>> {
        let conn = self.backend.get_connection().await?;
        let stmt = conn
            .prepare(
                "SELECT id, tenant_id, session_id, session_data, status, created_at,
                     last_accessed_at, expires_at, artifact_count, updated_at FROM sessions",
            )
            .await?;

        let mut rows = stmt.query(libsql::params![]).await?;
        let mut sessions = Vec::new();

        while let Some(row) = rows.next().await? {
            let data_str: String = row.get(3)?;
            let session_data: serde_json::Value = serde_json::from_str(&data_str)?;
            let expires_opt: Option<i64> = row.get(7)?;

            sessions.push(SessionExport {
                id: row.get(0)?,
                tenant_id: row.get(1)?,
                session_id: row.get(2)?,
                session_data,
                status: row.get(4)?,
                created_at: row.get(5)?,
                last_accessed_at: row.get(6)?,
                expires_at: expires_opt,
                artifact_count: row.get(8)?,
                updated_at: row.get(9)?,
            });
        }

        Ok(sessions)
    }

    async fn export_artifacts(&self) -> Result<ArtifactsExport> {
        let conn = self.backend.get_connection().await?;

        // Export content
        let stmt = conn
            .prepare(
                "SELECT tenant_id, content_hash, data, size_bytes, is_compressed,
                     original_size_bytes, reference_count, created_at, last_accessed_at
                     FROM artifact_content",
            )
            .await?;

        let mut rows = stmt.query(libsql::params![]).await?;
        let mut content = Vec::new();

        while let Some(row) = rows.next().await? {
            let data_blob: Vec<u8> = row.get(2)?;
            let data = Some(base64::Engine::encode(
                &base64::engine::general_purpose::STANDARD,
                data_blob,
            ));

            content.push(ArtifactContentExport {
                tenant_id: row.get(0)?,
                content_hash: row.get(1)?,
                storage_type: "blob".to_string(), // SQLite stores all as BLOB
                data,
                size_bytes: row.get(3)?,
                is_compressed: row.get(4)?,
                original_size_bytes: row.get(5)?,
                reference_count: row.get(6)?,
                created_at: row.get(7)?,
                last_accessed_at: row.get(8)?,
            });
        }

        // Export metadata
        let stmt = conn
            .prepare(
                "SELECT tenant_id, artifact_id, session_id, sequence, content_hash, metadata,
                     name, artifact_type, mime_type, size_bytes, created_at, created_by, version,
                     parent_artifact_id, tags, stored_at, updated_at FROM artifacts",
            )
            .await?;

        let mut rows = stmt.query(libsql::params![]).await?;
        let mut artifacts = Vec::new();

        while let Some(row) = rows.next().await? {
            let meta_str: String = row.get(5)?;
            let metadata: serde_json::Value = serde_json::from_str(&meta_str)?;
            let tags_str: String = row.get(14)?;
            let tags: Vec<String> = serde_json::from_str(&tags_str)?;

            artifacts.push(ArtifactMetadataExport {
                tenant_id: row.get(0)?,
                artifact_id: row.get(1)?,
                session_id: row.get(2)?,
                sequence: row.get(3)?,
                content_hash: row.get(4)?,
                metadata,
                name: row.get(6)?,
                artifact_type: row.get(7)?,
                mime_type: row.get(8)?,
                size_bytes: row.get(9)?,
                created_at: row.get(10)?,
                created_by: row.get(11)?,
                version: row.get(12)?,
                parent_artifact_id: row.get(13)?,
                tags,
                stored_at: row.get(15)?,
                updated_at: row.get(16)?,
            });
        }

        Ok(ArtifactsExport { content, artifacts })
    }

    async fn export_event_log(&self) -> Result<Vec<EventExport>> {
        let conn = self.backend.get_connection().await?;
        let stmt = conn
            .prepare(
                "SELECT id, tenant_id, event_id, event_type, correlation_id,
                     timestamp, sequence, language, payload FROM event_log",
            )
            .await?;

        let mut rows = stmt.query(libsql::params![]).await?;
        let mut events = Vec::new();

        while let Some(row) = rows.next().await? {
            let payload_str: String = row.get(8)?;
            let payload: serde_json::Value = serde_json::from_str(&payload_str)?;

            events.push(EventExport {
                id: row.get(0)?,
                tenant_id: row.get(1)?,
                event_id: row.get(2)?,
                event_type: row.get(3)?,
                correlation_id: row.get(4)?,
                timestamp: row.get(5)?,
                sequence: row.get(6)?,
                language: row.get(7)?,
                payload,
            });
        }

        Ok(events)
    }

    async fn export_hook_history(&self) -> Result<Vec<HookExport>> {
        let conn = self.backend.get_connection().await?;
        let stmt = conn
            .prepare(
                "SELECT id, execution_id, tenant_id, hook_id, hook_type, correlation_id,
                     hook_context, result_data, timestamp, duration_ms FROM hook_history",
            )
            .await?;

        let mut rows = stmt.query(libsql::params![]).await?;
        let mut hooks = Vec::new();

        while let Some(row) = rows.next().await? {
            let context_bytes: Vec<u8> = row.get(6)?;
            let hook_context_base64 =
                base64::engine::general_purpose::STANDARD.encode(&context_bytes);
            let result_str: String = row.get(7)?;
            let result_data: serde_json::Value = serde_json::from_str(&result_str)?;

            hooks.push(HookExport {
                id: row.get(0)?,
                execution_id: row.get(1)?,
                tenant_id: row.get(2)?,
                hook_id: row.get(3)?,
                hook_type: row.get(4)?,
                correlation_id: row.get(5)?,
                hook_context: hook_context_base64,
                result_data,
                timestamp: row.get(8)?,
                duration_ms: row.get(9)?,
            });
        }

        Ok(hooks)
    }

    pub async fn export_to_file<P: AsRef<Path>>(&self, output_path: P) -> Result<()> {
        let export = self.export_all().await?;
        let json = serde_json::to_string_pretty(&export)?;
        std::fs::write(output_path.as_ref(), json)?;
        tracing::info!("Export written to: {}", output_path.as_ref().display());
        Ok(())
    }

    pub async fn export_to_json(&self) -> Result<String> {
        let export = self.export_all().await?;
        serde_json::to_string_pretty(&export).context("Failed to serialize export data")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    #[cfg(feature = "sqlite")]
    async fn test_sqlite_exporter_creation() {
        use crate::backends::sqlite::{SqliteBackend, SqliteConfig};

        let config = SqliteConfig::in_memory();
        let backend = Arc::new(SqliteBackend::new(config).await.unwrap());

        // Run migrations to create tables
        backend.run_migrations().await.unwrap();

        let exporter = SqliteExporter::new(backend);

        // Should be able to create export format
        let export = exporter.export_all().await.unwrap();
        assert_eq!(export.source_backend, "sqlite");
        assert_eq!(export.version, "1.0");
    }

    #[tokio::test]
    #[cfg(feature = "postgres")]
    async fn test_postgres_exporter_creation() {
        use crate::backends::postgres::{PostgresBackend, PostgresConfig};

        // This test requires a running PostgreSQL instance
        // Skip if not available
        if std::env::var("DATABASE_URL").is_err() {
            return;
        }

        let config = PostgresConfig::new(&std::env::var("DATABASE_URL").unwrap());
        let backend = Arc::new(PostgresBackend::new(config).await.unwrap());
        let exporter = PostgresExporter::new(backend);

        // Should be able to create export format
        let export = exporter.export_all().await.unwrap();
        assert_eq!(export.source_backend, "postgresql");
        assert_eq!(export.version, "1.0");
    }
}
