//! Session-aware RAG with artifact storage and TTL management
//!
//! This module provides session-scoped vector storage with automatic cleanup,
//! artifact tracking, and tenant isolation.

use anyhow::Result;
use llmspell_sessions::{SessionId, SessionManager};
use llmspell_state_traits::StateScope;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tracing::{debug, error, info};
use uuid::Uuid;

use crate::state_integration::StateAwareVectorStorage;

/// Session-aware RAG pipeline with artifact tracking
pub struct SessionAwareRAGPipeline {
    /// State-aware vector storage
    state_aware_storage: Arc<StateAwareVectorStorage>,
    /// Session manager for lifecycle integration
    session_manager: Arc<SessionManager>,
    /// Active session collections
    session_collections: Arc<RwLock<HashMap<SessionId, SessionVectorCollection>>>,
    /// TTL cleanup handler
    cleanup_handler: Arc<RwLock<Option<tokio::task::JoinHandle<()>>>>,
}

impl std::fmt::Debug for SessionAwareRAGPipeline {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SessionAwareRAGPipeline")
            .field("state_aware_storage", &self.state_aware_storage)
            .field("session_manager", &"Arc<SessionManager>")
            .field("session_collections", &"Arc<RwLock<HashMap<...>>>")
            .field("cleanup_handler", &"Arc<RwLock<Option<JoinHandle>>>")
            .finish()
    }
}

/// Session-bound vector collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionVectorCollection {
    /// Session ID this collection belongs to
    pub session_id: SessionId,
    /// Namespace for vector storage
    pub namespace: String,
    /// Number of vectors in collection
    pub vector_count: usize,
    /// Total tokens processed
    pub total_tokens: usize,
    /// Creation timestamp
    pub created_at: SystemTime,
    /// Expiration time if TTL is set
    pub expires_at: Option<SystemTime>,
    /// Associated tenant ID if applicable
    pub tenant_id: Option<String>,
}

/// Ingestion statistics for session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IngestStats {
    /// Number of documents processed
    pub documents_processed: usize,
    /// Number of vectors created
    pub vectors_created: usize,
    /// Total tokens used
    pub total_tokens: usize,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Session vector operation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionVectorResult {
    /// Operation ID
    pub id: String,
    /// Vector text
    pub text: String,
    /// Similarity score
    pub score: f32,
    /// Metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl SessionAwareRAGPipeline {
    /// Create a new session-aware RAG pipeline
    #[must_use]
    pub fn new(
        state_aware_storage: Arc<StateAwareVectorStorage>,
        session_manager: Arc<SessionManager>,
    ) -> Self {
        Self {
            state_aware_storage,
            session_manager,
            session_collections: Arc::new(RwLock::new(HashMap::new())),
            cleanup_handler: Arc::new(RwLock::new(None)),
        }
    }

    /// Create session-bound vector collection
    ///
    /// # Errors
    /// Returns an error if session is invalid or collection creation fails
    pub async fn create_session_collection(
        &self,
        session_id: SessionId,
        ttl_seconds: Option<u64>,
    ) -> Result<SessionVectorCollection> {
        // Validate session is active
        let session = self.session_manager.get_session(&session_id).await?;
        let status = session.status().await;
        if !status.is_active() {
            return Err(anyhow::anyhow!("Session is not active"));
        }

        // Extract tenant ID from session metadata if available
        let tenant_id = {
            let metadata = session.metadata.read().await;
            metadata
                .custom_metadata
                .get("tenant_id")
                .and_then(|v| v.as_str())
                .map(String::from)
        };

        // Create collection
        let collection = SessionVectorCollection {
            session_id,
            namespace: format!("session_{session_id}"),
            vector_count: 0,
            total_tokens: 0,
            created_at: SystemTime::now(),
            expires_at: ttl_seconds.map(|ttl| SystemTime::now() + Duration::from_secs(ttl)),
            tenant_id,
        };

        // Store collection
        {
            let mut collections = self.session_collections.write().await;
            collections.insert(session_id, collection.clone());
        }

        // Schedule TTL cleanup if needed
        if let Some(ttl) = ttl_seconds {
            self.schedule_ttl_cleanup(session_id, ttl).await?;
        }

        info!(
            "Created session vector collection for session {}",
            session_id
        );
        Ok(collection)
    }

    /// Retrieve vectors within session context
    ///
    /// # Errors
    /// Returns an error if session is invalid or search fails
    pub async fn retrieve_in_session(
        &self,
        query: &str,
        session_id: SessionId,
        k: usize,
    ) -> Result<Vec<SessionVectorResult>> {
        // Validate session access
        let session = self.session_manager.get_session(&session_id).await?;
        let status = session.status().await;
        if !status.is_active() {
            return Err(anyhow::anyhow!("Session is not active"));
        }

        // Create session scope for search
        let scope = StateScope::Custom(format!("session:{session_id}"));

        // Perform scoped search
        let query_vector = vec![0.1; 384]; // Mock embedding - would be generated
        let search_query =
            llmspell_storage::VectorQuery::new(query_vector, k).with_scope(scope.clone());

        let results = self
            .state_aware_storage
            .storage()
            .search(&search_query)
            .await?;

        // Convert to session results
        let session_results: Vec<SessionVectorResult> = results
            .into_iter()
            .map(|r| SessionVectorResult {
                id: r.id,
                text: r
                    .metadata
                    .as_ref()
                    .and_then(|m| m.get("text"))
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string(),
                score: r.score,
                metadata: r.metadata.unwrap_or_default(),
            })
            .collect();

        // Update collection stats
        if let Some(collections) = self.session_collections.write().await.get_mut(&session_id) {
            collections.total_tokens += query.len();
        }

        debug!(
            "Retrieved {} results in session {}",
            session_results.len(),
            session_id
        );
        Ok(session_results)
    }

    /// Ingest documents into session-bound storage
    ///
    /// # Errors
    /// Returns an error if session is invalid or ingestion fails
    pub async fn ingest_in_session(
        &self,
        texts: Vec<String>,
        session_id: SessionId,
    ) -> Result<IngestStats> {
        let start = std::time::Instant::now();

        // Validate session
        let session = self.session_manager.get_session(&session_id).await?;
        let status = session.status().await;
        if !status.is_active() {
            return Err(anyhow::anyhow!("Session is not active"));
        }

        let scope = StateScope::Custom(format!("session:{session_id}"));

        // Generate embeddings (mock for now)
        let mut vectors = Vec::new();
        let mut total_tokens = 0;

        for text in &texts {
            total_tokens += text.len();
            let embedding = vec![0.1; 384]; // Mock embedding

            let mut metadata = HashMap::new();
            metadata.insert("text".to_string(), serde_json::Value::String(text.clone()));
            metadata.insert(
                "session_id".to_string(),
                serde_json::Value::String(session_id.to_string()),
            );

            let entry = llmspell_storage::VectorEntry::new(Uuid::new_v4().to_string(), embedding)
                .with_scope(scope.clone())
                .with_metadata(metadata);

            vectors.push(entry);
        }

        // Store vectors
        let ids = self
            .state_aware_storage
            .storage()
            .insert(vectors.clone())
            .await?;

        // Update collection stats
        {
            let mut collections = self.session_collections.write().await;
            if let Some(collection) = collections.get_mut(&session_id) {
                collection.vector_count += ids.len();
                collection.total_tokens += total_tokens;
            }
        }

        let ingest_stats = IngestStats {
            documents_processed: texts.len(),
            vectors_created: ids.len(),
            total_tokens,
            processing_time_ms: start.elapsed().as_millis().try_into().unwrap_or(u64::MAX),
        };

        info!(
            "Ingested {} documents in session {}",
            texts.len(),
            session_id
        );
        Ok(ingest_stats)
    }

    /// Clean up session vectors
    ///
    /// # Errors
    /// Returns an error if cleanup fails
    pub async fn cleanup_session(&self, session_id: SessionId) -> Result<usize> {
        let scope = StateScope::Custom(format!("session:{session_id}"));

        // Delete vectors
        let deleted = self
            .state_aware_storage
            .storage()
            .delete_scope(&scope)
            .await?;

        // Remove collection
        {
            let mut collections = self.session_collections.write().await;
            collections.remove(&session_id);
        }

        info!("Cleaned up {} vectors for session {}", deleted, session_id);
        Ok(deleted)
    }

    /// Schedule TTL-based cleanup
    async fn schedule_ttl_cleanup(&self, session_id: SessionId, ttl_seconds: u64) -> Result<()> {
        let pipeline = self.clone();
        let handle = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_secs(ttl_seconds)).await;

            if let Err(e) = pipeline.cleanup_session(session_id).await {
                error!("Failed to cleanup session {} vectors: {}", session_id, e);
            } else {
                info!("TTL cleanup completed for session {}", session_id);
            }
        });

        // Store cleanup handler
        {
            let mut cleanup = self.cleanup_handler.write().await;
            *cleanup = Some(handle);
        }

        Ok(())
    }

    /// Get session collection info
    pub async fn get_collection(&self, session_id: &SessionId) -> Option<SessionVectorCollection> {
        let collections = self.session_collections.read().await;
        collections.get(session_id).cloned()
    }

    /// List all active session collections
    pub async fn list_collections(&self) -> Vec<SessionVectorCollection> {
        let collections = self.session_collections.read().await;
        collections.values().cloned().collect()
    }

    /// Migrate session vectors to tenant long-term storage
    ///
    /// # Errors
    /// Returns an error if migration fails
    pub async fn migrate_to_tenant(&self, session_id: SessionId, tenant_id: &str) -> Result<usize> {
        // Get session vectors
        let session_scope = StateScope::Custom(format!("session:{session_id}"));
        let tenant_scope = StateScope::Custom(format!("tenant:{tenant_id}"));

        // Search all vectors in session (mock implementation)
        let query = llmspell_storage::VectorQuery::new(vec![0.0; 384], 10000)
            .with_scope(session_scope.clone());

        let results = self.state_aware_storage.storage().search(&query).await?;

        // Re-insert with tenant scope
        let mut migrated_vectors = Vec::new();
        for result in &results {
            let entry = llmspell_storage::VectorEntry::new(
                result.id.clone(),
                vec![0.1; 384], // Would need to retrieve actual embedding
            )
            .with_scope(tenant_scope.clone())
            .with_metadata(result.metadata.clone().unwrap_or_default());

            migrated_vectors.push(entry);
        }

        if !migrated_vectors.is_empty() {
            self.state_aware_storage
                .storage()
                .insert(migrated_vectors)
                .await?;
        }

        // Clean up session vectors
        self.cleanup_session(session_id).await?;

        info!(
            "Migrated {} vectors from session to tenant {}",
            results.len(),
            tenant_id
        );
        Ok(results.len())
    }
}

// Clone implementation for Arc-based struct
impl Clone for SessionAwareRAGPipeline {
    fn clone(&self) -> Self {
        Self {
            state_aware_storage: self.state_aware_storage.clone(),
            session_manager: self.session_manager.clone(),
            session_collections: self.session_collections.clone(),
            cleanup_handler: self.cleanup_handler.clone(),
        }
    }
}

/// Helper functions for `StateScope` tenant patterns
pub mod scope_helpers {
    use llmspell_state_traits::StateScope;

    /// Create a tenant scope
    #[must_use]
    pub fn create_tenant_scope(tenant_id: &str) -> StateScope {
        StateScope::Custom(format!("tenant:{tenant_id}"))
    }

    /// Create a session scope
    #[must_use]
    pub fn create_session_scope(session_id: &str) -> StateScope {
        StateScope::Custom(format!("session:{session_id}"))
    }

    /// Extract tenant ID from scope
    #[must_use]
    pub fn extract_tenant_from_scope(scope: &StateScope) -> Option<String> {
        match scope {
            StateScope::Custom(s) if s.starts_with("tenant:") => {
                s.strip_prefix("tenant:").map(String::from)
            }
            _ => None,
        }
    }

    /// Extract session ID from scope
    #[must_use]
    pub fn extract_session_from_scope(scope: &StateScope) -> Option<String> {
        match scope {
            StateScope::Custom(s) if s.starts_with("session:") => {
                s.strip_prefix("session:").map(String::from)
            }
            _ => None,
        }
    }

    /// Check if scope is tenant-scoped
    #[must_use]
    pub fn is_tenant_scope(scope: &StateScope) -> bool {
        matches!(scope, StateScope::Custom(s) if s.starts_with("tenant:"))
    }

    /// Check if scope is session-scoped
    #[must_use]
    pub fn is_session_scope(scope: &StateScope) -> bool {
        matches!(scope, StateScope::Custom(s) if s.starts_with("session:"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scope_helpers() {
        // Test tenant scope
        let tenant_scope = scope_helpers::create_tenant_scope("test-tenant");
        assert!(scope_helpers::is_tenant_scope(&tenant_scope));
        assert_eq!(
            scope_helpers::extract_tenant_from_scope(&tenant_scope),
            Some("test-tenant".to_string())
        );

        // Test session scope
        let session_scope = scope_helpers::create_session_scope("test-session");
        assert!(scope_helpers::is_session_scope(&session_scope));
        assert_eq!(
            scope_helpers::extract_session_from_scope(&session_scope),
            Some("test-session".to_string())
        );

        // Test non-matching scopes
        assert!(!scope_helpers::is_tenant_scope(&StateScope::Global));
        assert!(!scope_helpers::is_session_scope(&StateScope::Global));
    }

    #[tokio::test]
    async fn test_session_vector_collection() {
        let collection = SessionVectorCollection {
            session_id: SessionId::new(),
            namespace: "test_namespace".to_string(),
            vector_count: 10,
            total_tokens: 1000,
            created_at: SystemTime::now(),
            expires_at: Some(SystemTime::now() + Duration::from_secs(3600)),
            tenant_id: Some("test-tenant".to_string()),
        };

        assert_eq!(collection.vector_count, 10);
        assert_eq!(collection.total_tokens, 1000);
        assert!(collection.expires_at.is_some());
        assert_eq!(collection.tenant_id, Some("test-tenant".to_string()));
    }

    #[test]
    fn test_ingest_stats() {
        let stats = IngestStats {
            documents_processed: 5,
            vectors_created: 5,
            total_tokens: 500,
            processing_time_ms: 100,
        };

        assert_eq!(stats.documents_processed, 5);
        assert_eq!(stats.vectors_created, 5);
        assert_eq!(stats.total_tokens, 500);
        assert_eq!(stats.processing_time_ms, 100);
    }
}
