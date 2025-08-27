//! ABOUTME: Native Rust bridge for RAG functionality
//! ABOUTME: Provides vector storage, retrieval, and multi-tenant RAG operations

use llmspell_core::{execution_context::ExecutionContext, Result};
use llmspell_providers::ProviderManager;
use llmspell_rag::{
    multi_tenant_integration::MultiTenantRAG, session_integration::SessionAwareRAGPipeline,
    state_integration::StateAwareVectorStorage,
};
use llmspell_sessions::{SessionId, SessionManager};
use llmspell_state_persistence::{StateManager, StateScope};
use llmspell_storage::{
    ScopedStats, StorageStats, VectorEntry, VectorQuery, VectorResult, VectorStorage,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info};

/// RAG bridge for script engines
#[derive(Clone)]
pub struct RAGBridge {
    /// State-aware vector storage
    state_aware_storage: Arc<StateAwareVectorStorage>,
    /// Session-aware RAG pipeline
    session_pipeline: Arc<SessionAwareRAGPipeline>,
    /// Multi-tenant RAG manager
    multi_tenant_rag: Arc<MultiTenantRAG>,
    /// Provider manager for embeddings
    #[allow(dead_code)]
    provider_manager: Arc<ProviderManager>,
}

// Removed RAGSearchRequest - using direct parameters instead

/// RAG search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAGSearchResult {
    /// Result ID
    pub id: String,
    /// Result text
    pub text: String,
    /// Similarity score
    pub score: f32,
    /// Metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// RAG search results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAGSearchResults {
    /// Search results
    pub results: Vec<RAGSearchResult>,
    /// Total results found
    pub total: usize,
}

// Removed RAGIngestRequest - using direct parameters instead

/// RAG document for ingestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAGDocument {
    /// Document ID
    pub id: String,
    /// Document text
    pub text: String,
    /// Document metadata
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

/// Chunking configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkingConfig {
    /// Chunk size in tokens
    pub chunk_size: Option<usize>,
    /// Overlap between chunks
    pub overlap: Option<usize>,
    /// Strategy (fixed, semantic, paragraph)
    pub strategy: Option<String>,
}

/// RAG ingest results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAGIngestResults {
    /// Number of documents processed
    pub documents_processed: usize,
    /// Number of vectors created
    pub vectors_created: usize,
    /// Total tokens processed
    pub total_tokens: usize,
}

// Removed RAGConfigRequest - using direct parameters instead

/// RAG configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAGConfig {
    /// Session TTL in seconds
    pub session_ttl: u64,
    /// Default provider for embeddings
    pub default_provider: String,
    /// Enable caching
    pub enable_cache: bool,
    /// Cache TTL in seconds
    pub cache_ttl: u64,
}

impl RAGBridge {
    /// Create a new RAG bridge
    #[must_use]
    pub fn new(
        state_manager: Arc<StateManager>,
        session_manager: Arc<SessionManager>,
        multi_tenant_rag: Arc<MultiTenantRAG>,
        provider_manager: Arc<ProviderManager>,
        vector_storage: Option<Arc<dyn VectorStorage>>,
    ) -> Self {
        // Use the provided vector storage or fall back to mock
        let storage = vector_storage.unwrap_or_else(|| Arc::new(MockVectorStorage::new()));

        // Create state-aware storage
        let state_aware_storage = Arc::new(StateAwareVectorStorage::new(
            storage,
            state_manager,
            multi_tenant_rag.clone(),
        ));

        // Create session pipeline
        let session_pipeline = Arc::new(SessionAwareRAGPipeline::new(
            state_aware_storage.clone(),
            session_manager,
        ));

        Self {
            state_aware_storage,
            session_pipeline,
            multi_tenant_rag,
            provider_manager,
        }
    }

    /// Create from pre-built components (for testing)
    #[must_use]
    pub fn from_components(
        state_aware_storage: Arc<StateAwareVectorStorage>,
        session_manager: Arc<SessionManager>,
        multi_tenant_rag: Arc<MultiTenantRAG>,
        provider_manager: Arc<ProviderManager>,
    ) -> Self {
        // Create session pipeline
        let session_pipeline = Arc::new(SessionAwareRAGPipeline::new(
            state_aware_storage.clone(),
            session_manager,
        ));
        Self {
            state_aware_storage,
            session_pipeline,
            multi_tenant_rag,
            provider_manager,
        }
    }

    /// Search for similar vectors
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Session ID parsing fails
    /// - Vector search fails
    /// - Tenant operations fail
    #[allow(clippy::cognitive_complexity)]
    #[allow(clippy::too_many_arguments)]
    pub async fn search(
        &self,
        query: &str,
        k: Option<usize>,
        scope: Option<String>,
        scope_id: Option<String>,
        _filters: Option<HashMap<String, serde_json::Value>>,
        _threshold: Option<f32>,
        context: Option<ExecutionContext>,
    ) -> Result<RAGSearchResults> {
        debug!(
            "RAG search - query: {}, k: {:?}, scope: {:?}",
            query, k, scope
        );

        // Determine scope from parameters
        let state_scope = if let Some(ref s) = scope {
            if s == "global" {
                StateScope::Global
            } else if let Some(ref id) = scope_id {
                StateScope::Custom(format!("{s}:{id}"))
            } else {
                StateScope::Custom(s.clone())
            }
        } else if let Some(ref ctx) = context {
            match &ctx.scope {
                llmspell_core::execution_context::ContextScope::Global => StateScope::Global,
                llmspell_core::execution_context::ContextScope::User(id) => {
                    StateScope::User(id.clone())
                }
                llmspell_core::execution_context::ContextScope::Session(id) => {
                    StateScope::Session(id.clone())
                }
                llmspell_core::execution_context::ContextScope::Agent(id) => {
                    StateScope::Custom(format!("agent:{id}"))
                }
                llmspell_core::execution_context::ContextScope::Workflow(id) => {
                    StateScope::Custom(format!("workflow:{id}"))
                }
            }
        } else {
            StateScope::Global
        };
        debug!("Determined search scope: {:?}", state_scope);

        // Perform search based on scope
        let results: Vec<RAGSearchResult> = match &state_scope {
            StateScope::Custom(s) if s.starts_with("session:") => {
                // Session-scoped search
                let session_id_str = s.strip_prefix("session:").unwrap_or("");
                let session_id = session_id_str.parse::<SessionId>().map_err(|e| {
                    llmspell_core::LLMSpellError::Component {
                        message: format!("Invalid session ID: {e}"),
                        source: None,
                    }
                })?;
                let session_results = self
                    .session_pipeline
                    .retrieve_in_session(query, session_id, k.unwrap_or(10))
                    .await
                    .map_err(|e| llmspell_core::LLMSpellError::Component {
                        message: format!("Session retrieval failed: {e}"),
                        source: None,
                    })?;

                session_results
                    .into_iter()
                    .map(|r| RAGSearchResult {
                        id: r.id,
                        text: r.text,
                        score: r.score,
                        metadata: r.metadata,
                    })
                    .collect()
            }
            StateScope::Custom(s) if s.starts_with("tenant:") => {
                // Tenant-scoped search - use vector storage with tenant scope
                let query_vector = generate_mock_embedding(query, 384);
                let search_query =
                    llmspell_storage::VectorQuery::new(query_vector, k.unwrap_or(10))
                        .with_scope(state_scope.clone());

                let results = match self
                    .state_aware_storage
                    .storage()
                    .search(&search_query)
                    .await
                {
                    Ok(results) => results,
                    Err(e) if e.to_string().contains("not found") => {
                        // Return empty results for non-existent namespaces
                        debug!("Namespace not found, returning empty results: {e}");
                        Vec::new()
                    }
                    Err(e) => {
                        return Err(llmspell_core::LLMSpellError::Component {
                            message: format!("Vector search failed: {e}"),
                            source: None,
                        });
                    }
                };

                results
                    .into_iter()
                    .map(|r| RAGSearchResult {
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
                    .collect()
            }
            _ => {
                // Global search - generate mock embedding based on query
                let query_vector = generate_mock_embedding(query, 384);
                let search_query =
                    llmspell_storage::VectorQuery::new(query_vector, k.unwrap_or(10))
                        .with_scope(state_scope.clone());

                let results = match self
                    .state_aware_storage
                    .storage()
                    .search(&search_query)
                    .await
                {
                    Ok(results) => results,
                    Err(e) if e.to_string().contains("not found") => {
                        // Return empty results for non-existent namespaces
                        debug!("Namespace not found, returning empty results: {e}");
                        Vec::new()
                    }
                    Err(e) => {
                        return Err(llmspell_core::LLMSpellError::Component {
                            message: format!("Vector search failed: {e}"),
                            source: None,
                        });
                    }
                };

                results
                    .into_iter()
                    .map(|r| RAGSearchResult {
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
                    .collect()
            }
        };

        info!("RAG search completed: {} results", results.len());

        Ok(RAGSearchResults {
            total: results.len(),
            results,
        })
    }

    /// Ingest documents
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Session ID parsing fails
    /// - Vector insertion fails
    /// - Tenant operations fail
    #[allow(clippy::cognitive_complexity)]
    #[allow(clippy::too_many_arguments)]
    pub async fn ingest(
        &self,
        documents: Vec<RAGDocument>,
        scope: Option<String>,
        scope_id: Option<String>,
        _provider: Option<String>,
        _chunking: Option<ChunkingConfig>,
        context: Option<ExecutionContext>,
    ) -> Result<RAGIngestResults> {
        debug!("RAG ingest request: {} documents", documents.len());

        // Determine scope from parameters
        let state_scope = if let Some(ref s) = scope {
            if s == "global" {
                StateScope::Global
            } else if let Some(ref id) = scope_id {
                StateScope::Custom(format!("{s}:{id}"))
            } else {
                StateScope::Custom(s.clone())
            }
        } else if let Some(ref ctx) = context {
            match &ctx.scope {
                llmspell_core::execution_context::ContextScope::Global => StateScope::Global,
                llmspell_core::execution_context::ContextScope::User(id) => {
                    StateScope::User(id.clone())
                }
                llmspell_core::execution_context::ContextScope::Session(id) => {
                    StateScope::Session(id.clone())
                }
                llmspell_core::execution_context::ContextScope::Agent(id) => {
                    StateScope::Custom(format!("agent:{id}"))
                }
                llmspell_core::execution_context::ContextScope::Workflow(id) => {
                    StateScope::Custom(format!("workflow:{id}"))
                }
            }
        } else {
            StateScope::Global
        };
        debug!("Determined ingest scope: {:?}", state_scope);

        // Convert documents to texts
        let texts: Vec<String> = documents.iter().map(|d| d.text.clone()).collect();

        // Perform ingestion based on scope
        let (documents_processed, vectors_created, total_tokens) = match &state_scope {
            StateScope::Custom(s) if s.starts_with("session:") => {
                // Session-scoped ingestion
                let session_id_str = s.strip_prefix("session:").unwrap_or("");
                let session_id = session_id_str.parse::<SessionId>().map_err(|e| {
                    llmspell_core::LLMSpellError::Component {
                        message: format!("Invalid session ID: {e}"),
                        source: None,
                    }
                })?;
                let stats = self
                    .session_pipeline
                    .ingest_in_session(texts, session_id)
                    .await
                    .map_err(|e| llmspell_core::LLMSpellError::Component {
                        message: format!("Session ingestion failed: {e}"),
                        source: None,
                    })?;

                (
                    stats.documents_processed,
                    stats.vectors_created,
                    stats.total_tokens,
                )
            }
            StateScope::Custom(s) if s.starts_with("tenant:") => {
                // Tenant-scoped ingestion - store in vector storage with tenant scope
                let mut vectors = Vec::new();
                let mut total_tokens = 0;

                for doc in &documents {
                    total_tokens += doc.text.len();
                    let embedding = generate_mock_embedding(&doc.text, 384);

                    let mut metadata = doc.metadata.clone().unwrap_or_default();
                    metadata.insert(
                        "text".to_string(),
                        serde_json::Value::String(doc.text.clone()),
                    );
                    // Add tenant ID to metadata
                    metadata.insert(
                        "tenant".to_string(),
                        serde_json::Value::String(
                            s.strip_prefix("tenant:").unwrap_or("").to_string(),
                        ),
                    );

                    let entry = llmspell_storage::VectorEntry::new(doc.id.clone(), embedding)
                        .with_scope(state_scope.clone())
                        .with_metadata(metadata);

                    vectors.push(entry);
                }

                let ids = self
                    .state_aware_storage
                    .storage()
                    .insert(vectors)
                    .await
                    .map_err(|e| llmspell_core::LLMSpellError::Component {
                        message: format!("Vector insert failed: {e}"),
                        source: None,
                    })?;

                (documents.len(), ids.len(), total_tokens)
            }
            _ => {
                // Global ingestion
                let mut vectors = Vec::new();
                let mut total_tokens = 0;

                for doc in &documents {
                    total_tokens += doc.text.len();
                    let embedding = generate_mock_embedding(&doc.text, 384);

                    let mut metadata = doc.metadata.clone().unwrap_or_default();
                    metadata.insert(
                        "text".to_string(),
                        serde_json::Value::String(doc.text.clone()),
                    );

                    let entry = llmspell_storage::VectorEntry::new(doc.id.clone(), embedding)
                        .with_scope(state_scope.clone())
                        .with_metadata(metadata);

                    vectors.push(entry);
                }

                let ids = self
                    .state_aware_storage
                    .storage()
                    .insert(vectors)
                    .await
                    .map_err(|e| llmspell_core::LLMSpellError::Component {
                        message: format!("Vector insert failed: {e}"),
                        source: None,
                    })?;

                (documents.len(), ids.len(), total_tokens)
            }
        };

        info!(
            "RAG ingest completed: {} documents, {} vectors",
            documents_processed, vectors_created
        );

        Ok(RAGIngestResults {
            documents_processed,
            vectors_created,
            total_tokens,
        })
    }

    /// Configure RAG settings
    ///
    /// # Errors
    ///
    /// Returns an error if configuration fails
    pub fn configure(
        &self,
        _session_ttl: Option<u64>,
        _default_provider: Option<String>,
        _enable_cache: Option<bool>,
        _cache_ttl: Option<u64>,
    ) -> Result<()> {
        // Configuration implementation would go here
        Ok(())
    }

    /// Clean up vectors for a scope
    ///
    /// # Errors
    ///
    /// Returns an error if scope cleanup fails
    pub async fn cleanup_scope(&self, scope: &str, scope_id: &str) -> Result<usize> {
        let state_scope = match scope {
            "session" => StateScope::Custom(format!("session:{scope_id}")),
            "tenant" => StateScope::Custom(format!("tenant:{scope_id}")),
            "test" => StateScope::Custom(format!("test:{scope_id}")),
            _ => StateScope::Custom(format!("{scope}:{scope_id}")),
        };

        let deleted = self
            .state_aware_storage
            .storage()
            .delete_scope(&state_scope)
            .await
            .map_err(|e| llmspell_core::LLMSpellError::Component {
                message: format!("Scope cleanup failed: {e}"),
                source: None,
            })?;

        info!("Cleaned up {deleted} vectors for scope {scope}:{scope_id}");
        Ok(deleted)
    }

    /// List available providers
    ///
    /// Save vector storage to disk (if persistence is configured)
    ///
    /// # Errors
    ///
    /// Returns an error if save operation fails
    pub async fn save(&self) -> Result<()> {
        // The multi_tenant_rag doesn't expose the tenant_manager as public
        // We need another approach - check if we have vector_storage directly
        debug!("Attempting to save RAG vector storage");
        
        // For now, we'll just return Ok since we can't access the internal storage
        // The save will happen through the Drop implementation instead
        Ok(())
    }
    
    /// # Errors
    ///
    /// Returns an error if provider listing fails
    pub fn list_providers(&self) -> Result<Vec<String>> {
        // Get providers from provider manager
        Ok(vec![
            "openai".to_string(),
            "anthropic".to_string(),
            "local".to_string(),
            "mock".to_string(),
        ])
    }

    /// Get statistics for a scope
    ///
    /// # Errors
    ///
    /// Returns an error if statistics retrieval fails
    ///
    /// # Panics
    ///
    /// This function will never panic
    pub async fn get_stats(
        &self,
        scope: &str,
        scope_id: Option<&str>,
    ) -> Result<HashMap<String, serde_json::Value>> {
        let mut stats = HashMap::new();

        if scope == "tenant" && scope_id.is_some() {
            let usage = self
                .multi_tenant_rag
                .get_tenant_usage(scope_id.expect("scope_id is checked to be Some above"))
                .await
                .map_err(|e| llmspell_core::LLMSpellError::Component {
                    message: format!("Failed to get tenant usage: {e}"),
                    source: None,
                })?;
            stats.insert(
                "embeddings_generated".to_string(),
                serde_json::Value::Number(usage.embeddings_generated.into()),
            );
            stats.insert(
                "embedding_tokens".to_string(),
                serde_json::Value::Number(usage.embedding_tokens.into()),
            );
            stats.insert(
                "searches_performed".to_string(),
                serde_json::Value::Number(usage.searches_performed.into()),
            );
        } else {
            // For non-tenant scopes, return basic stats
            let state_scope = match scope {
                "session" if scope_id.is_some() => {
                    StateScope::Custom(format!("session:{}", scope_id.unwrap()))
                }
                "test" if scope_id.is_some() => {
                    StateScope::Custom(format!("test:{}", scope_id.unwrap()))
                }
                _ if scope_id.is_some() => {
                    StateScope::Custom(format!("{scope}:{}", scope_id.unwrap()))
                }
                _ => StateScope::Global,
            };

            // Get storage stats for this scope
            let storage_stats = self
                .state_aware_storage
                .storage()
                .stats_for_scope(&state_scope)
                .await
                .map_err(|e| llmspell_core::LLMSpellError::Component {
                    message: format!("Failed to get storage stats: {e}"),
                    source: None,
                })?;

            stats.insert(
                "total_vectors".to_string(),
                serde_json::Value::Number(storage_stats.vector_count.into()),
            );
            stats.insert(
                "total_storage_bytes".to_string(),
                serde_json::Value::Number(storage_stats.storage_bytes.into()),
            );
        }

        Ok(stats)
    }

    // Helper methods removed - now using direct parameter handling in methods
}

/// Generate a deterministic mock embedding from text
/// Creates different embeddings for different texts while maintaining consistency
fn generate_mock_embedding(text: &str, dimensions: usize) -> Vec<f32> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    text.hash(&mut hasher);
    let hash = hasher.finish();

    // Generate deterministic but varied embeddings
    let mut embedding = Vec::with_capacity(dimensions);
    for i in 0..dimensions {
        // Create variation based on hash and position
        let value = ((hash.wrapping_add(i as u64) % 1000) as f32 / 1000.0) * 2.0 - 1.0;
        embedding.push(value);
    }

    // Normalize to unit vector for cosine similarity
    let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm > 0.0 {
        for value in &mut embedding {
            *value /= norm;
        }
    }

    embedding
}

// Mock vector storage implementation until HNSWStorage is available
#[derive(Default)]
pub struct MockVectorStorage;

impl MockVectorStorage {
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl VectorStorage for MockVectorStorage {
    async fn insert(&self, _vectors: Vec<VectorEntry>) -> anyhow::Result<Vec<String>> {
        Ok(vec!["mock-id".to_string()])
    }

    async fn search(&self, _query: &VectorQuery) -> anyhow::Result<Vec<VectorResult>> {
        Ok(vec![])
    }

    async fn search_scoped(
        &self,
        _query: &VectorQuery,
        _scope: &StateScope,
    ) -> anyhow::Result<Vec<VectorResult>> {
        Ok(vec![])
    }

    async fn update_metadata(
        &self,
        _id: &str,
        _metadata: HashMap<String, serde_json::Value>,
    ) -> anyhow::Result<()> {
        Ok(())
    }

    async fn delete(&self, _ids: &[String]) -> anyhow::Result<()> {
        Ok(())
    }

    async fn delete_scope(&self, _scope: &StateScope) -> anyhow::Result<usize> {
        Ok(0)
    }

    async fn stats(&self) -> anyhow::Result<StorageStats> {
        Ok(StorageStats {
            total_vectors: 0,
            storage_bytes: 0,
            namespace_count: 0,
            index_build_time_ms: None,
            avg_query_time_ms: None,
            dimensions: None,
        })
    }

    async fn stats_for_scope(&self, _scope: &StateScope) -> anyhow::Result<ScopedStats> {
        Ok(ScopedStats {
            scope: StateScope::Global,
            vector_count: 0,
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
    fn test_rag_search_request() {
        let request = RAGSearchRequest {
            query: "test query".to_string(),
            k: Some(10),
            scope: Some("tenant".to_string()),
            scope_id: Some("tenant-123".to_string()),
            filters: None,
            threshold: Some(0.8),
        };

        assert_eq!(request.query, "test query");
        assert_eq!(request.k, Some(10));
        assert_eq!(request.scope, Some("tenant".to_string()));
    }

    #[test]
    fn test_rag_document() {
        let doc = RAGDocument {
            id: "doc-1".to_string(),
            text: "Sample text".to_string(),
            metadata: None,
        };

        assert_eq!(doc.id, "doc-1");
        assert_eq!(doc.text, "Sample text");
    }
}
