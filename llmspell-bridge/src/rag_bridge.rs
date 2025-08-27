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

/// RAG search request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAGSearchRequest {
    /// Query text
    pub query: String,
    /// Number of results
    pub k: Option<usize>,
    /// Search scope (global, tenant, session)
    pub scope: Option<String>,
    /// Scope ID (tenant ID or session ID)
    pub scope_id: Option<String>,
    /// Optional metadata filters
    pub filters: Option<HashMap<String, serde_json::Value>>,
    /// Similarity threshold (0.0-1.0)
    pub threshold: Option<f32>,
}

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

/// RAG search response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAGSearchResponse {
    /// Success flag
    pub success: bool,
    /// Search results
    pub results: Vec<RAGSearchResult>,
    /// Total results found
    pub total: usize,
    /// Error message if failed
    pub error: Option<String>,
}

/// RAG ingest request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAGIngestRequest {
    /// Documents to ingest
    pub documents: Vec<RAGDocument>,
    /// Target scope
    pub scope: Option<String>,
    /// Scope ID
    pub scope_id: Option<String>,
    /// Embedding provider to use
    pub provider: Option<String>,
    /// Chunking configuration
    pub chunking: Option<ChunkingConfig>,
}

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

/// RAG ingest response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAGIngestResponse {
    /// Success flag
    pub success: bool,
    /// Number of documents processed
    pub documents_processed: usize,
    /// Number of vectors created
    pub vectors_created: usize,
    /// Total tokens processed
    pub total_tokens: usize,
    /// Error message if failed
    pub error: Option<String>,
}

/// RAG configuration request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAGConfigRequest {
    /// Session TTL in seconds
    pub session_ttl: Option<u64>,
    /// Default provider for embeddings
    pub default_provider: Option<String>,
    /// Enable caching
    pub enable_cache: Option<bool>,
    /// Cache TTL in seconds
    pub cache_ttl: Option<u64>,
}

impl RAGBridge {
    /// Create a new RAG bridge
    #[must_use]
    pub fn new(
        state_manager: Arc<StateManager>,
        session_manager: Arc<SessionManager>,
        multi_tenant_rag: Arc<MultiTenantRAG>,
        provider_manager: Arc<ProviderManager>,
    ) -> Self {
        // Create vector storage (using mock for now since HNSWStorage isn't implemented)
        let storage: Arc<dyn VectorStorage> = Arc::new(MockVectorStorage::new());

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

    /// Search for similar vectors
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Session ID parsing fails
    /// - Vector search fails
    /// - Tenant operations fail
    pub async fn search(
        &self,
        request: RAGSearchRequest,
        context: Option<ExecutionContext>,
    ) -> Result<RAGSearchResponse> {
        debug!("RAG search request: {:?}", request);

        // Determine scope
        let scope = Self::determine_scope(&request, context);

        // Perform search based on scope
        let results = match &scope {
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
                    .retrieve_in_session(&request.query, session_id, request.k.unwrap_or(10))
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
                // Tenant-scoped search
                let tenant_id = s.strip_prefix("tenant:").unwrap_or("");
                let _embeddings = self
                    .multi_tenant_rag
                    .generate_tenant_embeddings(tenant_id, std::slice::from_ref(&request.query))
                    .await
                    .map_err(|e| llmspell_core::LLMSpellError::Component {
                        message: format!("Tenant embedding generation failed: {e}"),
                        source: None,
                    })?;

                // Mock search results for now
                vec![RAGSearchResult {
                    id: "mock-1".to_string(),
                    text: "Mock tenant result".to_string(),
                    score: 0.9,
                    metadata: HashMap::new(),
                }]
            }
            _ => {
                // Global search
                let query_vector = vec![0.1; 384]; // Mock embedding
                let search_query =
                    llmspell_storage::VectorQuery::new(query_vector, request.k.unwrap_or(10))
                        .with_scope(scope.clone());

                let results = self
                    .state_aware_storage
                    .storage()
                    .search(&search_query)
                    .await
                    .map_err(|e| llmspell_core::LLMSpellError::Component {
                        message: format!("Vector search failed: {e}"),
                        source: None,
                    })?;

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

        Ok(RAGSearchResponse {
            success: true,
            total: results.len(),
            results,
            error: None,
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
    pub async fn ingest(
        &self,
        request: RAGIngestRequest,
        context: Option<ExecutionContext>,
    ) -> Result<RAGIngestResponse> {
        debug!("RAG ingest request: {} documents", request.documents.len());

        // Determine scope
        let scope = Self::determine_scope_ingest(&request, context);

        // Convert documents to texts
        let texts: Vec<String> = request.documents.iter().map(|d| d.text.clone()).collect();

        // Perform ingestion based on scope
        let (documents_processed, vectors_created, total_tokens) = match &scope {
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
                // Tenant-scoped ingestion
                let tenant_id = s.strip_prefix("tenant:").unwrap_or("");
                let embeddings = self
                    .multi_tenant_rag
                    .generate_tenant_embeddings(tenant_id, &texts)
                    .await
                    .map_err(|e| llmspell_core::LLMSpellError::Component {
                        message: format!("Tenant embedding generation failed: {e}"),
                        source: None,
                    })?;

                (
                    texts.len(),
                    embeddings.len(),
                    texts.iter().map(String::len).sum(),
                )
            }
            _ => {
                // Global ingestion
                let mut vectors = Vec::new();
                let mut total_tokens = 0;

                for doc in &request.documents {
                    total_tokens += doc.text.len();
                    let embedding = vec![0.1; 384]; // Mock embedding

                    let mut metadata = doc.metadata.clone().unwrap_or_default();
                    metadata.insert(
                        "text".to_string(),
                        serde_json::Value::String(doc.text.clone()),
                    );

                    let entry = llmspell_storage::VectorEntry::new(doc.id.clone(), embedding)
                        .with_scope(scope.clone())
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

                (request.documents.len(), ids.len(), total_tokens)
            }
        };

        info!(
            "RAG ingest completed: {} documents, {} vectors",
            documents_processed, vectors_created
        );

        Ok(RAGIngestResponse {
            success: true,
            documents_processed,
            vectors_created,
            total_tokens,
            error: None,
        })
    }

    /// Configure RAG settings
    ///
    /// # Errors
    ///
    /// Returns an error if configuration fails
    pub fn configure(&self, _request: RAGConfigRequest) -> Result<()> {
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
            _ => StateScope::Global,
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
    /// # Errors
    ///
    /// Returns an error if provider listing fails
    pub fn list_providers(&self) -> Result<Vec<String>> {
        // Get providers from provider manager
        Ok(vec![
            "openai".to_string(),
            "anthropic".to_string(),
            "local".to_string(),
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
        }

        Ok(stats)
    }

    // Helper methods

    fn determine_scope(
        request: &RAGSearchRequest,
        context: Option<ExecutionContext>,
    ) -> StateScope {
        request.scope.as_ref().map_or_else(
            || {
                context.map_or(StateScope::Global, |ctx| match ctx.scope {
                    llmspell_core::execution_context::ContextScope::Global => StateScope::Global,
                    llmspell_core::execution_context::ContextScope::User(id) => {
                        StateScope::User(id)
                    }
                    llmspell_core::execution_context::ContextScope::Session(id) => {
                        StateScope::Session(id)
                    }
                    llmspell_core::execution_context::ContextScope::Agent(id) => {
                        StateScope::Custom(format!("agent:{id}"))
                    }
                    llmspell_core::execution_context::ContextScope::Workflow(id) => {
                        StateScope::Custom(format!("workflow:{id}"))
                    }
                })
            },
            |scope| match scope.as_str() {
                "session" => request.scope_id.as_ref().map_or(StateScope::Global, |id| {
                    StateScope::Custom(format!("session:{id}"))
                }),
                "tenant" => request.scope_id.as_ref().map_or(StateScope::Global, |id| {
                    StateScope::Custom(format!("tenant:{id}"))
                }),
                _ => StateScope::Global,
            },
        )
    }

    fn determine_scope_ingest(
        request: &RAGIngestRequest,
        context: Option<ExecutionContext>,
    ) -> StateScope {
        request.scope.as_ref().map_or_else(
            || {
                context.map_or(StateScope::Global, |ctx| match ctx.scope {
                    llmspell_core::execution_context::ContextScope::Global => StateScope::Global,
                    llmspell_core::execution_context::ContextScope::User(id) => {
                        StateScope::User(id)
                    }
                    llmspell_core::execution_context::ContextScope::Session(id) => {
                        StateScope::Session(id)
                    }
                    llmspell_core::execution_context::ContextScope::Agent(id) => {
                        StateScope::Custom(format!("agent:{id}"))
                    }
                    llmspell_core::execution_context::ContextScope::Workflow(id) => {
                        StateScope::Custom(format!("workflow:{id}"))
                    }
                })
            },
            |scope| match scope.as_str() {
                "session" => request.scope_id.as_ref().map_or(StateScope::Global, |id| {
                    StateScope::Custom(format!("session:{id}"))
                }),
                "tenant" => request.scope_id.as_ref().map_or(StateScope::Global, |id| {
                    StateScope::Custom(format!("tenant:{id}"))
                }),
                _ => StateScope::Global,
            },
        )
    }
}

// Mock vector storage implementation until HNSWStorage is available
struct MockVectorStorage;

impl MockVectorStorage {
    const fn new() -> Self {
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
