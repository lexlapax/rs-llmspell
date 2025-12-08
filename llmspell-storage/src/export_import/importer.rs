//! Storage importer for loading export data into PostgreSQL or SQLite backends
//!
//! Handles JSON import with type conversion, validation, and transaction safety.

#[cfg(any(feature = "postgres", feature = "sqlite"))]
use super::converters::TypeConverters;
#[cfg(any(feature = "postgres", feature = "sqlite"))]
use super::format::*;
#[cfg(any(feature = "postgres", feature = "sqlite"))]
use anyhow::{anyhow, Context, Result};
#[cfg(any(feature = "postgres", feature = "sqlite"))]
use base64::Engine;
#[cfg(any(feature = "postgres", feature = "sqlite"))]
use std::collections::HashMap;
#[cfg(any(feature = "postgres", feature = "sqlite"))]
use std::sync::Arc;

#[cfg(feature = "postgres")]
use crate::backends::postgres::PostgresBackend;
#[cfg(feature = "sqlite")]
use crate::backends::sqlite::SqliteBackend;

// ============================================================================
// PostgreSQL Importer
// ============================================================================

#[cfg(feature = "postgres")]
/// PostgreSQL data importer
pub struct PostgresImporter {
    backend: Arc<PostgresBackend>,
    #[allow(dead_code)]
    converters: TypeConverters,
}

#[cfg(feature = "postgres")]
impl PostgresImporter {
    /// Create a new PostgreSQL importer
    pub fn new(backend: Arc<PostgresBackend>) -> Self {
        Self {
            backend,
            converters: TypeConverters::new(),
        }
    }

    /// Import data from ExportFormat into PostgreSQL
    #[allow(clippy::field_reassign_with_default)]
    pub async fn import(&self, export: ExportFormat) -> Result<ImportStats> {
        // Validate format version
        if export.version != "1.0" {
            return Err(anyhow!(
                "Unsupported export format version: {}",
                export.version
            ));
        }

        // Get client and start transaction
        let mut client = self.backend.get_client().await?;
        let tx = client
            .transaction()
            .await
            .context("Failed to start transaction")?;

        let mut stats = ImportStats::default();

        // Import in dependency order (entities before relationships, etc.)

        // 1. Vector embeddings (no dependencies)
        stats.vectors = self
            .import_vector_embeddings_tx(&tx, &export.data.vector_embeddings)
            .await?;

        // 2. Knowledge graph entities (no dependencies)
        if let Some(kg) = &export.data.knowledge_graph {
            stats.entities = self.import_entities_tx(&tx, &kg.entities).await?;
            stats.relationships = self.import_relationships_tx(&tx, &kg.relationships).await?;
        }

        // 3. Procedural memory (no dependencies)
        stats.patterns = self
            .import_procedural_memory_tx(&tx, &export.data.procedural_memory)
            .await?;

        // 4. Agent state (no dependencies)
        stats.agent_states = self
            .import_agent_state_tx(&tx, &export.data.agent_state)
            .await?;

        // 5. KV store (no dependencies)
        stats.kv_entries = self.import_kv_store_tx(&tx, &export.data.kv_store).await?;

        // 6. Workflow states (no dependencies)
        stats.workflow_states = self
            .import_workflow_states_tx(&tx, &export.data.workflow_states)
            .await?;

        // 7. Sessions (no dependencies)
        stats.sessions = self.import_sessions_tx(&tx, &export.data.sessions).await?;

        // 8. Artifacts (depends on sessions for session_id FK)
        if let Some(artifacts) = &export.data.artifacts {
            let (content_count, artifact_count) = self.import_artifacts_tx(&tx, artifacts).await?;
            stats.artifact_content = content_count;
            stats.artifacts = artifact_count;
        }

        // 9. Event log (no dependencies)
        stats.events = self
            .import_event_log_tx(&tx, &export.data.event_log)
            .await?;

        // 10. Hook history (no dependencies)
        stats.hooks = self
            .import_hook_history_tx(&tx, &export.data.hook_history)
            .await?;

        // Commit transaction
        tx.commit()
            .await
            .context("Failed to commit import transaction")?;

        Ok(stats)
    }

    async fn import_vector_embeddings_tx(
        &self,
        tx: &tokio_postgres::Transaction<'_>,
        embeddings: &HashMap<usize, Vec<VectorEmbeddingExport>>,
    ) -> Result<usize> {
        let mut count = 0;

        for (dim, vectors) in embeddings {
            let table = format!("llmspell.vector_embeddings_{}", dim);

            for vec_export in vectors {
                // Decode base64 to get original embedding bytes
                // (either JSON format or raw f32 bytes, as stored in source DB)
                let embedding_bytes = base64::engine::general_purpose::STANDARD
                    .decode(&vec_export.embedding)
                    .context("Failed to decode base64 embedding")?;

                // Convert bytes to pgvector text format "[x,y,z,...]"
                // Parse as JSON array if starts with '[', otherwise treat as raw f32 bytes
                let embedding_str = if embedding_bytes.starts_with(b"[") {
                    // Already JSON format - convert to string
                    String::from_utf8(embedding_bytes)
                        .context("Failed to convert JSON embedding to UTF-8")?
                } else {
                    // Raw f32 bytes - convert to JSON array string
                    let floats: Vec<f32> = embedding_bytes
                        .chunks_exact(4)
                        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                        .collect();
                    format!(
                        "[{}]",
                        floats
                            .iter()
                            .map(|f| f.to_string())
                            .collect::<Vec<_>>()
                            .join(",")
                    )
                };

                let query = format!(
                    "INSERT INTO {} (id, tenant_id, scope, embedding, metadata, created_at, updated_at)
                     VALUES ($1, $2, $3, $4, $5, to_timestamp($6), to_timestamp($7))
                     ON CONFLICT (id) DO UPDATE SET
                       embedding = EXCLUDED.embedding,
                       metadata = EXCLUDED.metadata,
                       updated_at = EXCLUDED.updated_at",
                    table
                );

                let metadata_str = vec_export.metadata.to_string();
                let created_at_sec = vec_export.created_at as f64 / 1_000_000.0;
                let updated_at_sec = vec_export.updated_at as f64 / 1_000_000.0;

                tx.execute(
                    &query,
                    &[
                        &uuid::Uuid::parse_str(&vec_export.id)?,
                        &vec_export.tenant_id,
                        &vec_export.scope,
                        &embedding_str,
                        &metadata_str,
                        &created_at_sec,
                        &updated_at_sec,
                    ],
                )
                .await
                .context("Failed to import vector embedding")?;

                count += 1;
            }
        }

        Ok(count)
    }

    async fn import_entities_tx(
        &self,
        tx: &tokio_postgres::Transaction<'_>,
        entities: &[EntityExport],
    ) -> Result<usize> {
        for entity in entities {
            let properties_str = entity.properties.to_string();
            let vts_sec = entity.valid_time_start as f64 / 1_000_000.0;
            let vte_sec = entity.valid_time_end as f64 / 1_000_000.0;
            let tts_sec = entity.transaction_time_start as f64 / 1_000_000.0;
            let tte_sec = entity.transaction_time_end as f64 / 1_000_000.0;
            let created_sec = entity.created_at as f64 / 1_000_000.0;

            tx.execute(
                "INSERT INTO llmspell.entities
                 (entity_id, tenant_id, entity_type, name, properties,
                  valid_time_start, valid_time_end, transaction_time_start, transaction_time_end, created_at)
                 VALUES ($1, $2, $3, $4, $5, to_timestamp($6), to_timestamp($7), to_timestamp($8), to_timestamp($9), to_timestamp($10))
                 ON CONFLICT (entity_id) DO UPDATE SET
                   properties = EXCLUDED.properties,
                   valid_time_end = EXCLUDED.valid_time_end,
                   transaction_time_end = EXCLUDED.transaction_time_end",
                &[
                    &uuid::Uuid::parse_str(&entity.entity_id)?,
                    &entity.tenant_id,
                    &entity.entity_type,
                    &entity.name,
                    &properties_str,
                    &vts_sec,
                    &vte_sec,
                    &tts_sec,
                    &tte_sec,
                    &created_sec,
                ],
            )
            .await
            .context("Failed to import entity")?;
        }

        Ok(entities.len())
    }

    async fn import_relationships_tx(
        &self,
        tx: &tokio_postgres::Transaction<'_>,
        relationships: &[RelationshipExport],
    ) -> Result<usize> {
        for rel in relationships {
            let properties_str = rel.properties.to_string();
            let vts_sec = rel.valid_time_start as f64 / 1_000_000.0;
            let vte_sec = rel.valid_time_end as f64 / 1_000_000.0;
            let tts_sec = rel.transaction_time_start as f64 / 1_000_000.0;
            let tte_sec = rel.transaction_time_end as f64 / 1_000_000.0;
            let created_sec = rel.created_at as f64 / 1_000_000.0;

            tx.execute(
                "INSERT INTO llmspell.relationships
                 (relationship_id, tenant_id, from_entity, to_entity, relationship_type, properties,
                  valid_time_start, valid_time_end, transaction_time_start, transaction_time_end, created_at)
                 VALUES ($1, $2, $3, $4, $5, $6, to_timestamp($7), to_timestamp($8), to_timestamp($9), to_timestamp($10), to_timestamp($11))
                 ON CONFLICT (relationship_id) DO UPDATE SET
                   properties = EXCLUDED.properties,
                   valid_time_end = EXCLUDED.valid_time_end,
                   transaction_time_end = EXCLUDED.transaction_time_end",
                &[
                    &uuid::Uuid::parse_str(&rel.relationship_id)?,
                    &rel.tenant_id,
                    &uuid::Uuid::parse_str(&rel.source_entity_id)?,
                    &uuid::Uuid::parse_str(&rel.target_entity_id)?,
                    &rel.relationship_type,
                    &properties_str,
                    &vts_sec,
                    &vte_sec,
                    &tts_sec,
                    &tte_sec,
                    &created_sec,
                ],
            )
            .await
            .context("Failed to import relationship")?;
        }

        Ok(relationships.len())
    }

    async fn import_procedural_memory_tx(
        &self,
        tx: &tokio_postgres::Transaction<'_>,
        patterns: &[PatternExport],
    ) -> Result<usize> {
        for pattern in patterns {
            let created_sec = pattern.created_at as f64 / 1_000_000.0;
            let updated_sec = pattern.updated_at as f64 / 1_000_000.0;
            let first_seen_sec = pattern.first_seen as f64 / 1_000_000.0;
            let last_seen_sec = pattern.last_seen as f64 / 1_000_000.0;

            tx.execute(
                "INSERT INTO llmspell.procedural_patterns
                 (pattern_id, tenant_id, scope, key, value, frequency, first_seen, last_seen, created_at, updated_at)
                 VALUES ($1, $2, $3, $4, $5, $6, to_timestamp($7), to_timestamp($8), to_timestamp($9), to_timestamp($10))
                 ON CONFLICT (pattern_id) DO UPDATE SET
                   frequency = EXCLUDED.frequency,
                   last_seen = EXCLUDED.last_seen,
                   updated_at = EXCLUDED.updated_at",
                &[
                    &uuid::Uuid::parse_str(&pattern.pattern_id)?,
                    &pattern.tenant_id,
                    &pattern.scope,
                    &pattern.key,
                    &pattern.value,
                    &pattern.frequency,
                    &first_seen_sec,
                    &last_seen_sec,
                    &created_sec,
                    &updated_sec,
                ],
            )
            .await
            .context("Failed to import procedural pattern")?;
        }

        Ok(patterns.len())
    }

    async fn import_agent_state_tx(
        &self,
        tx: &tokio_postgres::Transaction<'_>,
        states: &[AgentStateExport],
    ) -> Result<usize> {
        for state in states {
            let state_data_str = state.state_data.to_string();
            let created_sec = state.created_at as f64 / 1_000_000.0;
            let updated_sec = state.updated_at as f64 / 1_000_000.0;

            tx.execute(
                "INSERT INTO llmspell.agent_states
                 (state_id, tenant_id, agent_id, agent_type, state_data, schema_version, data_version, checksum, created_at, updated_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, to_timestamp($9), to_timestamp($10))
                 ON CONFLICT (state_id) DO UPDATE SET
                   state_data = EXCLUDED.state_data,
                   data_version = EXCLUDED.data_version,
                   checksum = EXCLUDED.checksum,
                   updated_at = EXCLUDED.updated_at",
                &[
                    &uuid::Uuid::parse_str(&state.state_id)?,
                    &state.tenant_id,
                    &state.agent_id,
                    &state.agent_type,
                    &state_data_str,
                    &state.schema_version,
                    &state.data_version,
                    &state.checksum,
                    &created_sec,
                    &updated_sec,
                ],
            )
            .await
            .context("Failed to import agent state")?;
        }

        Ok(states.len())
    }

    async fn import_kv_store_tx(
        &self,
        tx: &tokio_postgres::Transaction<'_>,
        entries: &[KVEntryExport],
    ) -> Result<usize> {
        for entry in entries {
            let value_bytes = base64::engine::general_purpose::STANDARD
                .decode(&entry.value)
                .context("Failed to decode base64 KV value")?;
            let created_sec = entry.created_at as f64 / 1_000_000.0;
            let updated_sec = entry.updated_at as f64 / 1_000_000.0;

            tx.execute(
                "INSERT INTO llmspell.kv_store
                 (kv_id, tenant_id, key, value, metadata, created_at, updated_at)
                 VALUES ($1, $2, $3, $4, $5, to_timestamp($6), to_timestamp($7))
                 ON CONFLICT (kv_id) DO UPDATE SET
                   value = EXCLUDED.value,
                   metadata = EXCLUDED.metadata,
                   updated_at = EXCLUDED.updated_at",
                &[
                    &uuid::Uuid::parse_str(&entry.kv_id)?,
                    &entry.tenant_id,
                    &entry.key,
                    &value_bytes,
                    &entry.metadata,
                    &created_sec,
                    &updated_sec,
                ],
            )
            .await
            .context("Failed to import KV entry")?;
        }

        Ok(entries.len())
    }

    async fn import_workflow_states_tx(
        &self,
        tx: &tokio_postgres::Transaction<'_>,
        workflows: &[WorkflowStateExport],
    ) -> Result<usize> {
        for wf in workflows {
            let state_data_str = wf.state_data.to_string();
            let created_sec = wf.created_at as f64 / 1_000_000.0;
            let last_updated_sec = wf.last_updated as f64 / 1_000_000.0;
            let started_at_sec = wf.started_at.map(|ts| ts as f64 / 1_000_000.0);
            let completed_at_sec = wf.completed_at.map(|ts| ts as f64 / 1_000_000.0);

            tx.execute(
                "INSERT INTO llmspell.workflow_states
                 (tenant_id, workflow_id, workflow_name, state_data, current_step, status, started_at, completed_at, last_updated, created_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, to_timestamp($9), to_timestamp($10))
                 ON CONFLICT (tenant_id, workflow_id) DO UPDATE SET
                   state_data = EXCLUDED.state_data,
                   current_step = EXCLUDED.current_step,
                   status = EXCLUDED.status,
                   completed_at = EXCLUDED.completed_at,
                   last_updated = EXCLUDED.last_updated",
                &[
                    &wf.tenant_id,
                    &wf.workflow_id,
                    &wf.workflow_name,
                    &state_data_str,
                    &wf.current_step,
                    &wf.status,
                    &started_at_sec.map(|s| format!("to_timestamp({})", s)),
                    &completed_at_sec.map(|s| format!("to_timestamp({})", s)),
                    &last_updated_sec,
                    &created_sec,
                ],
            )
            .await
            .context("Failed to import workflow state")?;
        }

        Ok(workflows.len())
    }

    async fn import_sessions_tx(
        &self,
        tx: &tokio_postgres::Transaction<'_>,
        sessions: &[SessionExport],
    ) -> Result<usize> {
        for session in sessions {
            let session_data_str = session.session_data.to_string();
            let created_sec = session.created_at as f64 / 1_000_000.0;
            let last_accessed_sec = session.last_accessed_at as f64 / 1_000_000.0;
            let updated_sec = session.updated_at as f64 / 1_000_000.0;
            let expires_at_sec = session.expires_at.map(|ts| ts as f64 / 1_000_000.0);

            tx.execute(
                "INSERT INTO llmspell.sessions
                 (id, tenant_id, session_id, session_data, status, created_at, last_accessed_at, expires_at, artifact_count, updated_at)
                 VALUES ($1, $2, $3, $4, $5, to_timestamp($6), to_timestamp($7), $8, $9, to_timestamp($10))
                 ON CONFLICT (id) DO UPDATE SET
                   session_data = EXCLUDED.session_data,
                   status = EXCLUDED.status,
                   last_accessed_at = EXCLUDED.last_accessed_at,
                   artifact_count = EXCLUDED.artifact_count,
                   updated_at = EXCLUDED.updated_at",
                &[
                    &uuid::Uuid::parse_str(&session.id)?,
                    &session.tenant_id,
                    &session.session_id,
                    &session_data_str,
                    &session.status,
                    &created_sec,
                    &last_accessed_sec,
                    &expires_at_sec.map(|s| format!("to_timestamp({})", s)),
                    &session.artifact_count,
                    &updated_sec,
                ],
            )
            .await
            .context("Failed to import session")?;
        }

        Ok(sessions.len())
    }

    async fn import_artifacts_tx(
        &self,
        tx: &tokio_postgres::Transaction<'_>,
        artifacts: &ArtifactsExport,
    ) -> Result<(usize, usize)> {
        // Import content first (no dependencies)
        for content in &artifacts.content {
            let data_bytes = if let Some(data_b64) = &content.data {
                Some(
                    base64::engine::general_purpose::STANDARD
                        .decode(data_b64)
                        .context("Failed to decode base64 artifact content")?,
                )
            } else {
                None
            };

            let created_sec = content.created_at as f64 / 1_000_000.0;
            let last_accessed_sec = content.last_accessed_at as f64 / 1_000_000.0;

            tx.execute(
                "INSERT INTO llmspell.artifact_content
                 (tenant_id, content_hash, storage_type, data, size_bytes, is_compressed, original_size_bytes, reference_count, created_at, last_accessed_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, to_timestamp($9), to_timestamp($10))
                 ON CONFLICT (tenant_id, content_hash) DO UPDATE SET
                   reference_count = EXCLUDED.reference_count,
                   last_accessed_at = EXCLUDED.last_accessed_at",
                &[
                    &content.tenant_id,
                    &content.content_hash,
                    &content.storage_type,
                    &data_bytes,
                    &content.size_bytes,
                    &content.is_compressed,
                    &content.original_size_bytes,
                    &content.reference_count,
                    &created_sec,
                    &last_accessed_sec,
                ],
            )
            .await
            .context("Failed to import artifact content")?;
        }

        // Then import artifact metadata (references content)
        for artifact in &artifacts.artifacts {
            let metadata_str = artifact.metadata.to_string();
            let created_sec = artifact.created_at as f64 / 1_000_000.0;
            let stored_sec = artifact.stored_at as f64 / 1_000_000.0;
            let updated_sec = artifact.updated_at as f64 / 1_000_000.0;

            // Convert Vec<String> tags to PostgreSQL array format
            let tags_array = format!("{{{}}}", artifact.tags.join(","));

            tx.execute(
                "INSERT INTO llmspell.artifacts
                 (tenant_id, artifact_id, session_id, sequence, content_hash, metadata, name, artifact_type, mime_type, size_bytes, created_at, created_by, version, parent_artifact_id, tags, stored_at, updated_at)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, to_timestamp($11), $12, $13, $14, $15, to_timestamp($16), to_timestamp($17))
                 ON CONFLICT (tenant_id, artifact_id) DO UPDATE SET
                   metadata = EXCLUDED.metadata,
                   version = EXCLUDED.version,
                   updated_at = EXCLUDED.updated_at",
                &[
                    &artifact.tenant_id,
                    &artifact.artifact_id,
                    &artifact.session_id,
                    &artifact.sequence,
                    &artifact.content_hash,
                    &metadata_str,
                    &artifact.name,
                    &artifact.artifact_type,
                    &artifact.mime_type,
                    &artifact.size_bytes,
                    &created_sec,
                    &artifact.created_by,
                    &artifact.version,
                    &artifact.parent_artifact_id,
                    &tags_array,
                    &stored_sec,
                    &updated_sec,
                ],
            )
            .await
            .context("Failed to import artifact")?;
        }

        Ok((artifacts.content.len(), artifacts.artifacts.len()))
    }

    async fn import_event_log_tx(
        &self,
        tx: &tokio_postgres::Transaction<'_>,
        events: &[EventExport],
    ) -> Result<usize> {
        for event in events {
            let payload_str = event.payload.to_string();
            let timestamp_sec = event.timestamp as f64 / 1_000_000.0;

            tx.execute(
                "INSERT INTO llmspell.event_log
                 (id, tenant_id, event_id, event_type, correlation_id, timestamp, sequence, language, payload)
                 VALUES ($1, $2, $3, $4, $5, to_timestamp($6), $7, $8, $9)
                 ON CONFLICT (id) DO NOTHING",
                &[
                    &uuid::Uuid::parse_str(&event.id)?,
                    &event.tenant_id,
                    &event.event_id,
                    &event.event_type,
                    &event.correlation_id,
                    &timestamp_sec,
                    &event.sequence,
                    &event.language,
                    &payload_str,
                ],
            )
            .await
            .context("Failed to import event")?;
        }

        Ok(events.len())
    }

    async fn import_hook_history_tx(
        &self,
        tx: &tokio_postgres::Transaction<'_>,
        hooks: &[HookExport],
    ) -> Result<usize> {
        for hook in hooks {
            let context_bytes = base64::engine::general_purpose::STANDARD
                .decode(&hook.hook_context)
                .context("Failed to decode base64 hook context")?;
            let result_str = hook.result_data.to_string();
            let timestamp_sec = hook.timestamp as f64 / 1_000_000.0;

            tx.execute(
                "INSERT INTO llmspell.hook_history
                 (id, execution_id, tenant_id, hook_id, hook_type, correlation_id, hook_context, result_data, timestamp, duration_ms)
                 VALUES ($1, $2, $3, $4, $5, $6, $7, $8, to_timestamp($9), $10)
                 ON CONFLICT (id) DO NOTHING",
                &[
                    &uuid::Uuid::parse_str(&hook.id)?,
                    &hook.execution_id,
                    &hook.tenant_id,
                    &hook.hook_id,
                    &hook.hook_type,
                    &hook.correlation_id,
                    &context_bytes,
                    &result_str,
                    &timestamp_sec,
                    &hook.duration_ms,
                ],
            )
            .await
            .context("Failed to import hook history entry")?;
        }

        Ok(hooks.len())
    }

    /// Import from JSON file
    pub async fn import_from_file(&self, path: &str) -> Result<ImportStats> {
        let json = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read export file: {}", path))?;
        let export: ExportFormat =
            serde_json::from_str(&json).context("Failed to parse export JSON")?;
        self.import(export).await
    }
}

// ============================================================================
// SQLite Importer
// ============================================================================

#[cfg(feature = "sqlite")]
/// SQLite data importer
pub struct SqliteImporter {
    backend: Arc<SqliteBackend>,
    #[allow(dead_code)]
    converters: TypeConverters,
}

#[cfg(feature = "sqlite")]
impl SqliteImporter {
    /// Create a new SQLite importer
    pub fn new(backend: Arc<SqliteBackend>) -> Self {
        Self {
            backend,
            converters: TypeConverters::new(),
        }
    }

    /// Import data from ExportFormat into SQLite
    pub async fn import(&self, export: ExportFormat) -> Result<ImportStats> {
        // Validate format version
        if export.version != "1.0" {
            return Err(anyhow!(
                "Unsupported export format version: {}",
                export.version
            ));
        }

        let mut conn = self.backend.get_connection().await?;

        // Start transaction (synchronous)
        let tx = conn.transaction()?;

        let mut stats = ImportStats::default();

        // Import in dependency order (using match to handle errors and rollback)
        // Note: Code block is synchronous because we use rusqlite sync API.
        // We wrap in a closure to catch errors before commit.
        let import_result = (|| {
            // 1. Vector embeddings (no dependencies)
            stats.vectors = self.import_vector_embeddings(&tx, &export.data.vector_embeddings)?;

            // 2. Knowledge graph entities (no dependencies)
            if let Some(kg) = &export.data.knowledge_graph {
                stats.entities = self.import_entities(&tx, &kg.entities)?;
                stats.relationships = self.import_relationships(&tx, &kg.relationships)?;
            }

            // 3. Procedural memory (no dependencies)
            stats.patterns = self.import_procedural_memory(&tx, &export.data.procedural_memory)?;

            // 4. Agent state (no dependencies)
            stats.agent_states = self.import_agent_state(&tx, &export.data.agent_state)?;

            // 5. KV store (no dependencies)
            stats.kv_entries = self.import_kv_store(&tx, &export.data.kv_store)?;

            // 6. Workflow states (no dependencies)
            stats.workflow_states =
                self.import_workflow_states(&tx, &export.data.workflow_states)?;

            // 7. Sessions (no dependencies)
            stats.sessions = self.import_sessions(&tx, &export.data.sessions)?;

            // 8. Artifacts (depends on sessions for session_id FK)
            if let Some(artifacts) = &export.data.artifacts {
                let (content_count, artifact_count) = self.import_artifacts(&tx, artifacts)?;
                stats.artifact_content = content_count;
                stats.artifacts = artifact_count;
            }

            // 9. Event log (no dependencies)
            stats.events = self.import_event_log(&tx, &export.data.event_log)?;

            // 10. Hook history (no dependencies)
            stats.hooks = self.import_hook_history(&tx, &export.data.hook_history)?;

            Ok::<(), anyhow::Error>(())
        })();

        match import_result {
            Ok(_) => {
                tx.commit().context("Failed to commit import transaction")?;
                Ok(stats)
            }
            Err(e) => {
                // Rollback happens automatically when tx is dropped
                Err(e)
            }
        }
    }

    fn import_vector_embeddings(
        &self,
        tx: &rusqlite::Transaction<'_>,
        embeddings: &HashMap<usize, Vec<VectorEmbeddingExport>>,
    ) -> Result<usize> {
        let mut count = 0;

        for (dim, vectors) in embeddings {
            let table = format!("vector_embeddings_{}", dim);

            for vec_export in vectors {
                // Decode base64 embedding to MessagePack bytes
                let embedding_bytes = base64::engine::general_purpose::STANDARD
                    .decode(&vec_export.embedding)
                    .context("Failed to decode base64 embedding")?;

                let metadata_str = vec_export.metadata.to_string();
                let created_at_sec = vec_export.created_at / 1_000_000;
                let updated_at_sec = vec_export.updated_at / 1_000_000;

                let query = format!(
                    "INSERT INTO {} (id, tenant_id, scope, embedding, metadata, created_at, updated_at)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                     ON CONFLICT (id) DO UPDATE SET
                       embedding = excluded.embedding,
                       metadata = excluded.metadata,
                       updated_at = excluded.updated_at",
                    table
                );

                tx.execute(
                    &query,
                    rusqlite::params![
                        vec_export.id.clone(),
                        vec_export.tenant_id.clone(),
                        vec_export.scope.clone(),
                        embedding_bytes,
                        metadata_str,
                        created_at_sec,
                        updated_at_sec,
                    ],
                )
                .context("Failed to import vector embedding")?;

                count += 1;
            }
        }

        Ok(count)
    }

    fn import_entities(
        &self,
        conn: &rusqlite::Transaction<'_>,
        entities: &[EntityExport],
    ) -> Result<usize> {
        for entity in entities {
            let properties_str = entity.properties.to_string();
            let vts_sec = entity.valid_time_start / 1_000_000;
            let vte_sec = entity.valid_time_end / 1_000_000;
            let tts_sec = entity.transaction_time_start / 1_000_000;
            let tte_sec = entity.transaction_time_end / 1_000_000;
            let created_sec = entity.created_at / 1_000_000;

            conn.execute(
                "INSERT INTO entities
                 (entity_id, tenant_id, entity_type, name, properties,
                  valid_time_start, valid_time_end, transaction_time_start, transaction_time_end, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                 ON CONFLICT (entity_id) DO UPDATE SET
                   properties = excluded.properties,
                   valid_time_end = excluded.valid_time_end,
                   transaction_time_end = excluded.transaction_time_end",
                rusqlite::params![
                    entity.entity_id.clone(),
                    entity.tenant_id.clone(),
                    entity.entity_type.clone(),
                    entity.name.clone(),
                    properties_str,
                    vts_sec,
                    vte_sec,
                    tts_sec,
                    tte_sec,
                    created_sec,
                ],
            )
            .context("Failed to import entity")?;
        }

        Ok(entities.len())
    }

    fn import_relationships(
        &self,
        conn: &rusqlite::Transaction<'_>,
        relationships: &[RelationshipExport],
    ) -> Result<usize> {
        for rel in relationships {
            let properties_str = rel.properties.to_string();
            let vts_sec = rel.valid_time_start / 1_000_000;
            let vte_sec = rel.valid_time_end / 1_000_000;
            let tts_sec = rel.transaction_time_start / 1_000_000;
            let tte_sec = rel.transaction_time_end / 1_000_000;
            let created_sec = rel.created_at / 1_000_000;

            conn.execute(
                "INSERT INTO relationships
                 (relationship_id, tenant_id, from_entity, to_entity, relationship_type, properties,
                  valid_time_start, valid_time_end, transaction_time_start, transaction_time_end, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
                 ON CONFLICT (relationship_id) DO UPDATE SET
                   properties = excluded.properties,
                   valid_time_end = excluded.valid_time_end,
                   transaction_time_end = excluded.transaction_time_end",
                rusqlite::params![
                    rel.relationship_id.clone(),
                    rel.tenant_id.clone(),
                    rel.source_entity_id.clone(),
                    rel.target_entity_id.clone(),
                    rel.relationship_type.clone(),
                    properties_str,
                    vts_sec,
                    vte_sec,
                    tts_sec,
                    tte_sec,
                    created_sec,
                ],
            )
            .context("Failed to import relationship")?;
        }

        Ok(relationships.len())
    }

    fn import_procedural_memory(
        &self,
        conn: &rusqlite::Transaction<'_>,
        patterns: &[PatternExport],
    ) -> Result<usize> {
        for pattern in patterns {
            let created_sec = pattern.created_at / 1_000_000;
            let updated_sec = pattern.updated_at / 1_000_000;
            let first_seen_sec = pattern.first_seen / 1_000_000;
            let last_seen_sec = pattern.last_seen / 1_000_000;

            conn.execute(
                "INSERT INTO procedural_patterns
                 (pattern_id, tenant_id, scope, key, value, frequency, first_seen, last_seen, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                 ON CONFLICT (pattern_id) DO UPDATE SET
                   frequency = excluded.frequency,
                   last_seen = excluded.last_seen,
                   updated_at = excluded.updated_at",
                rusqlite::params![
                    pattern.pattern_id.clone(),
                    pattern.tenant_id.clone(),
                    pattern.scope.clone(),
                    pattern.key.clone(),
                    pattern.value.clone(),
                    pattern.frequency,
                    first_seen_sec,
                    last_seen_sec,
                    created_sec,
                    updated_sec,
                ],
            )
            .context("Failed to import procedural pattern")?;
        }

        Ok(patterns.len())
    }

    fn import_agent_state(
        &self,
        conn: &rusqlite::Transaction<'_>,
        states: &[AgentStateExport],
    ) -> Result<usize> {
        for state in states {
            let state_data_str = state.state_data.to_string();
            let created_sec = state.created_at / 1_000_000;
            let updated_sec = state.updated_at / 1_000_000;

            conn.execute(
                "INSERT INTO agent_states
                 (state_id, tenant_id, agent_id, agent_type, state_data, schema_version, data_version, checksum, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                 ON CONFLICT (state_id) DO UPDATE SET
                   state_data = excluded.state_data,
                   data_version = excluded.data_version,
                   checksum = excluded.checksum,
                   updated_at = excluded.updated_at",
                rusqlite::params![
                    state.state_id.clone(),
                    state.tenant_id.clone(),
                    state.agent_id.clone(),
                    state.agent_type.clone(),
                    state_data_str,
                    state.schema_version,
                    state.data_version,
                    state.checksum.clone(),
                    created_sec,
                    updated_sec,
                ],
            )
            .context("Failed to import agent state")?;
        }

        Ok(states.len())
    }

    fn import_kv_store(
        &self,
        conn: &rusqlite::Transaction<'_>,
        entries: &[KVEntryExport],
    ) -> Result<usize> {
        for entry in entries {
            let value_bytes = base64::engine::general_purpose::STANDARD
                .decode(&entry.value)
                .context("Failed to decode base64 KV value")?;
            let created_sec = entry.created_at / 1_000_000;
            let updated_sec = entry.updated_at / 1_000_000;

            conn.execute(
                "INSERT INTO kv_store
                 (kv_id, tenant_id, key, value, metadata, created_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                 ON CONFLICT (kv_id) DO UPDATE SET
                   value = excluded.value,
                   metadata = excluded.metadata,
                   updated_at = excluded.updated_at",
                rusqlite::params![
                    entry.kv_id.clone(),
                    entry.tenant_id.clone(),
                    entry.key.clone(),
                    value_bytes,
                    entry.metadata.clone(),
                    created_sec,
                    updated_sec,
                ],
            )
            .context("Failed to import KV entry")?;
        }

        Ok(entries.len())
    }

    fn import_workflow_states(
        &self,
        tx: &rusqlite::Transaction<'_>,
        workflows: &[WorkflowStateExport],
    ) -> Result<usize> {
        for wf in workflows {
            let state_data_str = wf.state_data.to_string();
            let created_sec = wf.created_at / 1_000_000;
            let last_updated_sec = wf.last_updated / 1_000_000;
            let started_at_sec = wf.started_at.map(|ts| ts / 1_000_000);
            let completed_at_sec = wf.completed_at.map(|ts| ts / 1_000_000);

            tx.execute(
                "INSERT INTO workflow_states
                 (tenant_id, workflow_id, workflow_name, state_data, current_step, status, started_at, completed_at, last_updated, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                 ON CONFLICT (tenant_id, workflow_id) DO UPDATE SET
                   state_data = excluded.state_data,
                   current_step = excluded.current_step,
                   status = excluded.status,
                   completed_at = excluded.completed_at,
                   last_updated = excluded.last_updated",
                rusqlite::params![
                    wf.tenant_id.clone(),
                    wf.workflow_id.clone(),
                    wf.workflow_name.clone(),
                    state_data_str,
                    wf.current_step,
                    wf.status.clone(),
                    started_at_sec,
                    completed_at_sec,
                    last_updated_sec,
                    created_sec,
                ],
            )

            .context("Failed to import workflow state")?;
        }

        Ok(workflows.len())
    }

    fn import_sessions(
        &self,
        tx: &rusqlite::Transaction<'_>,
        sessions: &[SessionExport],
    ) -> Result<usize> {
        for session in sessions {
            let session_data_str = session.session_data.to_string();
            let created_sec = session.created_at / 1_000_000;
            let last_accessed_sec = session.last_accessed_at / 1_000_000;
            let updated_sec = session.updated_at / 1_000_000;
            let expires_at_sec = session.expires_at.map(|ts| ts / 1_000_000);

            tx.execute(
                "INSERT INTO sessions
                 (id, tenant_id, session_id, session_data, status, created_at, last_accessed_at, expires_at, artifact_count, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                 ON CONFLICT (id) DO UPDATE SET
                   session_data = excluded.session_data,
                   status = excluded.status,
                   last_accessed_at = excluded.last_accessed_at,
                   artifact_count = excluded.artifact_count,
                   updated_at = excluded.updated_at",
                rusqlite::params![
                    session.id.clone(),
                    session.tenant_id.clone(),
                    session.session_id.clone(),
                    session_data_str,
                    session.status.clone(),
                    created_sec,
                    last_accessed_sec,
                    expires_at_sec,
                    session.artifact_count,
                    updated_sec,
                ],
            )

            .context("Failed to import session")?;
        }

        Ok(sessions.len())
    }

    fn import_artifacts(
        &self,
        tx: &rusqlite::Transaction<'_>,
        artifacts: &ArtifactsExport,
    ) -> Result<(usize, usize)> {
        // Import content first (no dependencies)
        for content in &artifacts.content {
            let data_bytes = if let Some(data_b64) = &content.data {
                Some(
                    base64::engine::general_purpose::STANDARD
                        .decode(data_b64)
                        .context("Failed to decode base64 artifact content")?,
                )
            } else {
                None
            };

            let created_sec = content.created_at / 1_000_000;
            let last_accessed_sec = content.last_accessed_at / 1_000_000;

            tx.execute(
                "INSERT INTO artifact_content
                 (tenant_id, content_hash, data, size_bytes, is_compressed, original_size_bytes, reference_count, created_at, last_accessed_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                 ON CONFLICT (tenant_id, content_hash) DO UPDATE SET
                   reference_count = excluded.reference_count,
                   last_accessed_at = excluded.last_accessed_at",
                rusqlite::params![
                    content.tenant_id.clone(),
                    content.content_hash.clone(),
                    data_bytes,
                    content.size_bytes,
                    content.is_compressed as i32,
                    content.original_size_bytes,
                    content.reference_count,
                    created_sec,
                    last_accessed_sec,
                ],
            )

            .context("Failed to import artifact content")?;
        }

        // Then import artifact metadata (references content)
        for artifact in &artifacts.artifacts {
            let metadata_str = artifact.metadata.to_string();
            let created_sec = artifact.created_at / 1_000_000;
            let stored_sec = artifact.stored_at / 1_000_000;
            let updated_sec = artifact.updated_at / 1_000_000;

            // Convert Vec<String> tags to JSON array string
            let tags_json =
                serde_json::to_string(&artifact.tags).context("Failed to serialize tags")?;

            tx.execute(
                "INSERT INTO artifacts
                 (tenant_id, artifact_id, session_id, sequence, content_hash, metadata, name, artifact_type, mime_type, size_bytes, created_at, created_by, version, parent_artifact_id, tags, stored_at, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)
                 ON CONFLICT (tenant_id, artifact_id) DO UPDATE SET
                   metadata = excluded.metadata,
                   version = excluded.version,
                   updated_at = excluded.updated_at",
                rusqlite::params![
                    artifact.tenant_id.clone(),
                    artifact.artifact_id.clone(),
                    artifact.session_id.clone(),
                    artifact.sequence,
                    artifact.content_hash.clone(),
                    metadata_str,
                    artifact.name.clone(),
                    artifact.artifact_type.clone(),
                    artifact.mime_type.clone(),
                    artifact.size_bytes,
                    created_sec,
                    artifact.created_by.clone(),
                    artifact.version,
                    artifact.parent_artifact_id.clone(),
                    tags_json,
                    stored_sec,
                    updated_sec,
                ],
            )

            .context("Failed to import artifact")?;
        }

        Ok((artifacts.content.len(), artifacts.artifacts.len()))
    }

    fn import_event_log(
        &self,
        tx: &rusqlite::Transaction<'_>,
        events: &[EventExport],
    ) -> Result<usize> {
        for event in events {
            let payload_str = event.payload.to_string();
            let timestamp_sec = event.timestamp / 1_000_000;

            tx.execute(
                "INSERT INTO event_log
                 (id, tenant_id, event_id, event_type, correlation_id, timestamp, sequence, language, payload)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
                 ON CONFLICT (id) DO NOTHING",
                rusqlite::params![
                    event.id.clone(),
                    event.tenant_id.clone(),
                    event.event_id.clone(),
                    event.event_type.clone(),
                    event.correlation_id.clone(),
                    timestamp_sec,
                    event.sequence,
                    event.language.clone(),
                    payload_str,
                ],
            )

            .context("Failed to import event")?;
        }

        Ok(events.len())
    }

    fn import_hook_history(
        &self,
        tx: &rusqlite::Transaction<'_>,
        hooks: &[HookExport],
    ) -> Result<usize> {
        for hook in hooks {
            let context_bytes = base64::engine::general_purpose::STANDARD
                .decode(&hook.hook_context)
                .context("Failed to decode base64 hook context")?;
            let result_str = hook.result_data.to_string();
            let timestamp_sec = hook.timestamp / 1_000_000;

            tx.execute(
                "INSERT INTO hook_history
                 (id, execution_id, tenant_id, hook_id, hook_type, correlation_id, hook_context, result_data, timestamp, duration_ms)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
                 ON CONFLICT (id) DO NOTHING",
                rusqlite::params![
                    hook.id.clone(),
                    hook.execution_id.clone(),
                    hook.tenant_id.clone(),
                    hook.hook_id.clone(),
                    hook.hook_type.clone(),
                    hook.correlation_id.clone(),
                    context_bytes,
                    result_str,
                    timestamp_sec,
                    hook.duration_ms,
                ],
            )

            .context("Failed to import hook history entry")?;
        }

        Ok(hooks.len())
    }

    /// Import from JSON file
    pub async fn import_from_file(&self, path: &str) -> Result<ImportStats> {
        let json = std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read export file: {}", path))?;
        let export: ExportFormat =
            serde_json::from_str(&json).context("Failed to parse export JSON")?;
        self.import(export).await
    }
}

// ============================================================================
// Import Statistics
// ============================================================================

/// Statistics about imported data
#[derive(Debug, Default, Clone)]
pub struct ImportStats {
    pub vectors: usize,
    pub entities: usize,
    pub relationships: usize,
    pub patterns: usize,
    pub agent_states: usize,
    pub kv_entries: usize,
    pub workflow_states: usize,
    pub sessions: usize,
    pub artifact_content: usize,
    pub artifacts: usize,
    pub events: usize,
    pub hooks: usize,
}

impl ImportStats {
    /// Get total number of records imported
    pub fn total(&self) -> usize {
        self.vectors
            + self.entities
            + self.relationships
            + self.patterns
            + self.agent_states
            + self.kv_entries
            + self.workflow_states
            + self.sessions
            + self.artifact_content
            + self.artifacts
            + self.events
            + self.hooks
    }
}
