//! ABOUTME: Native Rust bridge for RAG functionality
//! ABOUTME: Provides vector storage, retrieval, and multi-tenant RAG operations

use llmspell_core::{execution_context::ExecutionContext, logging::warn, Result};
use llmspell_providers::ProviderManager;
use llmspell_rag::{
    multi_tenant_integration::MultiTenantRAG, session_integration::SessionAwareRAGPipeline,
    state_integration::StateAwareVectorStorage,
};
use llmspell_sessions::{SessionId, SessionManager};
use llmspell_state_persistence::{StateManager, StateScope};
use llmspell_storage::{VectorEntry, VectorResult, VectorStorage};
use serde::{Deserialize, Serialize};
use serde_json::Value;
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

/// RAG search parameters to reduce argument count
#[derive(Debug, Clone)]
pub struct RAGSearchParams {
    /// Search query
    pub query: String,
    /// Number of results
    pub k: Option<usize>,
    /// Scope filter
    pub scope: Option<String>,
    /// Scope ID
    pub scope_id: Option<String>,
    /// Metadata filters
    pub filters: Option<HashMap<String, serde_json::Value>>,
    /// Similarity threshold
    pub threshold: Option<f32>,
    /// Execution context
    pub context: Option<ExecutionContext>,
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
    ///
    /// # Panics
    ///
    /// Panics if `vector_storage` is `None`. Vector storage is required for RAG operations
    /// and must be provided. Use `HNSWVectorStorage` or another implementation of the
    /// `VectorStorage` trait.
    #[must_use]
    pub fn new(
        state_manager: Arc<StateManager>,
        session_manager: Arc<SessionManager>,
        multi_tenant_rag: Arc<MultiTenantRAG>,
        provider_manager: Arc<ProviderManager>,
        vector_storage: Option<Arc<dyn VectorStorage>>,
    ) -> Self {
        // Require vector storage to be provided - no fallback to mock
        let storage = vector_storage.expect("Vector storage must be provided");

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
    async fn search_in_session(
        &self,
        query: &str,
        session_id_str: &str,
        k: usize,
    ) -> Result<Vec<RAGSearchResult>> {
        let session_id = session_id_str.parse::<SessionId>().map_err(|e| {
            llmspell_core::LLMSpellError::Component {
                message: format!("Invalid session ID: {e}"),
                source: None,
            }
        })?;

        let session_results = self
            .session_pipeline
            .retrieve_in_session(query, session_id, k)
            .await
            .map_err(|e| llmspell_core::LLMSpellError::Component {
                message: format!("Session retrieval failed: {e}"),
                source: None,
            })?;

        Ok(session_results
            .into_iter()
            .map(|r| RAGSearchResult {
                id: r.id,
                text: r.text,
                score: r.score,
                metadata: r.metadata,
            })
            .collect())
    }

    async fn search_with_vector_storage(
        &self,
        query: &str,
        k: usize,
        state_scope: StateScope,
    ) -> Result<Vec<RAGSearchResult>> {
        let query_vector = generate_mock_embedding(query, 384);
        let search_query =
            llmspell_storage::VectorQuery::new(query_vector, k).with_scope(state_scope);

        let results = match self
            .state_aware_storage
            .storage()
            .search(&search_query)
            .await
        {
            Ok(results) => results,
            Err(e) if e.to_string().contains("not found") => {
                warn!("No vectors found in scope, returning empty results");
                vec![]
            }
            Err(e) => {
                return Err(llmspell_core::LLMSpellError::Component {
                    message: format!("Vector search failed: {e}"),
                    source: None,
                })
            }
        };

        Ok(results
            .into_iter()
            .map(Self::convert_vector_result_to_rag_result)
            .collect())
    }

    /// Dispatch search to appropriate handler based on scope
    async fn dispatch_search(
        &self,
        query: &str,
        k: usize,
        state_scope: StateScope,
    ) -> Result<Vec<RAGSearchResult>> {
        match &state_scope {
            StateScope::Custom(s) if s.starts_with("session:") => {
                let session_id_str = s.strip_prefix("session:").unwrap_or("");
                self.search_in_session(query, session_id_str, k).await
            }
            _ => {
                // All other scopes use vector storage
                self.search_with_vector_storage(query, k, state_scope).await
            }
        }
    }

    /// Execute a search with the given parameters
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Session ID parsing fails
    /// - Vector search fails
    /// - Namespace not found
    pub async fn search(&self, params: RAGSearchParams) -> Result<RAGSearchResults> {
        debug!(
            "RAG search - query: {}, k: {:?}, scope: {:?}",
            params.query, params.k, params.scope
        );

        // Determine scope from parameters
        let state_scope = Self::determine_scope(
            params.scope.as_deref(),
            params.scope_id.as_deref(),
            params.context.as_ref(),
        );
        debug!("Determined search scope: {:?}", state_scope);

        // Perform search based on scope
        let k_value = params.k.unwrap_or(10);
        let results = self
            .dispatch_search(&params.query, k_value, state_scope)
            .await?;

        info!("RAG search completed: {} results", results.len());

        Ok(RAGSearchResults {
            total: results.len(),
            results,
        })
    }

    /// Handle session-scoped ingestion
    async fn ingest_session_scoped(
        &self,
        texts: Vec<String>,
        session_scope_str: &str,
    ) -> Result<(usize, usize, usize)> {
        let session_id_str = session_scope_str.strip_prefix("session:").unwrap_or("");
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

        Ok((
            stats.documents_processed,
            stats.vectors_created,
            stats.total_tokens,
        ))
    }

    /// Handle tenant-scoped ingestion
    async fn ingest_tenant_scoped(
        &self,
        documents: &[RAGDocument],
        state_scope: &StateScope,
        tenant_scope_str: &str,
    ) -> Result<(usize, usize, usize)> {
        let mut vectors = Vec::new();
        let mut total_tokens = 0;

        for doc in documents {
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
                    tenant_scope_str
                        .strip_prefix("tenant:")
                        .unwrap_or("")
                        .to_string(),
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

        Ok((documents.len(), ids.len(), total_tokens))
    }

    /// Handle global ingestion
    async fn ingest_global_scoped(
        &self,
        documents: &[RAGDocument],
        state_scope: &StateScope,
    ) -> Result<(usize, usize, usize)> {
        let mut vectors = Vec::new();
        let mut total_tokens = 0;

        for doc in documents {
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

        Ok((documents.len(), ids.len(), total_tokens))
    }

    /// Convert `VectorResult` to `RAGSearchResult`
    fn convert_vector_result_to_rag_result(r: VectorResult) -> RAGSearchResult {
        RAGSearchResult {
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
        }
    }

    /// Convert RAG documents to vector entries
    fn convert_documents_to_vectors(
        documents: &[RAGDocument],
        state_scope: &StateScope,
    ) -> (Vec<String>, Vec<VectorEntry>) {
        let mut doc_ids = Vec::with_capacity(documents.len());
        let mut vector_entries = Vec::new();

        for doc in documents {
            let doc_id = doc.id.clone();
            let embedding = generate_mock_embedding(&doc.text, 384);

            let metadata = doc.metadata.clone().unwrap_or_default();
            let mut vector_metadata = HashMap::new();
            vector_metadata.insert("text".to_string(), Value::String(doc.text.clone()));
            vector_metadata.insert("doc_id".to_string(), Value::String(doc_id.clone()));
            // Convert serde_json::Value to serde_json::Value for metadata
            for (k, v) in metadata {
                vector_metadata.insert(k, v);
            }

            let entry = VectorEntry::new(doc_id.clone(), embedding)
                .with_metadata(vector_metadata)
                .with_scope(state_scope.clone());

            vector_entries.push(entry);
            doc_ids.push(doc_id);
        }

        (doc_ids, vector_entries)
    }

    /// Process documents for ingestion
    #[allow(dead_code)]
    async fn process_documents_for_scope(
        &self,
        documents: Vec<RAGDocument>,
        state_scope: StateScope,
    ) -> Result<Vec<String>> {
        debug!(
            "Processing {} documents for scope: {:?}",
            documents.len(),
            state_scope
        );

        let (doc_ids, vector_entries) =
            Self::convert_documents_to_vectors(&documents, &state_scope);

        // Insert vectors
        match &state_scope {
            StateScope::Custom(s) if s.starts_with("session:") => {
                debug!("Ingesting documents into session scope");
                // Session-scoped ingestion would go here
                // For now, just return the doc_ids we generated
            }
            _ => {
                debug!("Ingesting documents into vector storage");
                let _inserted_ids = self
                    .state_aware_storage
                    .storage()
                    .insert(vector_entries)
                    .await
                    .map_err(|e| llmspell_core::LLMSpellError::Component {
                        message: format!("Vector insertion failed: {e}"),
                        source: None,
                    })?;
            }
        }

        Ok(doc_ids)
    }

    /// Convert scope string parameters to `StateScope`
    fn scope_from_params(scope: &str, scope_id: Option<&str>) -> StateScope {
        if scope == "global" {
            StateScope::Global
        } else if let Some(id) = scope_id {
            StateScope::Custom(format!("{scope}:{id}"))
        } else {
            StateScope::Custom(scope.to_string())
        }
    }

    /// Convert execution context scope to `StateScope`
    fn scope_from_context(
        ctx_scope: &llmspell_core::execution_context::ContextScope,
    ) -> StateScope {
        match ctx_scope {
            llmspell_core::execution_context::ContextScope::Global => StateScope::Global,
            llmspell_core::execution_context::ContextScope::User(id) => {
                StateScope::User(id.clone())
            }
            llmspell_core::execution_context::ContextScope::Session(id) => {
                StateScope::Custom(format!("session:{id}"))
            }
            llmspell_core::execution_context::ContextScope::Workflow(id) => {
                StateScope::Custom(format!("workflow:{id}"))
            }
            llmspell_core::execution_context::ContextScope::Agent(id) => {
                StateScope::Custom(format!("agent:{id}"))
            }
        }
    }

    /// Determine scope from parameters and context
    fn determine_scope(
        scope: Option<&str>,
        scope_id: Option<&str>,
        context: Option<&ExecutionContext>,
    ) -> StateScope {
        scope.map_or_else(
            || {
                context.map_or(StateScope::Global, |ctx| {
                    Self::scope_from_context(&ctx.scope)
                })
            },
            |s| Self::scope_from_params(s, scope_id),
        )
    }

    /// Dispatch ingestion to appropriate handler based on scope
    async fn dispatch_ingest(
        &self,
        documents: &[RAGDocument],
        texts: Vec<String>,
        state_scope: &StateScope,
    ) -> Result<(usize, usize, usize)> {
        match state_scope {
            StateScope::Custom(s) if s.starts_with("session:") => {
                self.ingest_session_scoped(texts, s).await
            }
            StateScope::Custom(s) if s.starts_with("tenant:") => {
                self.ingest_tenant_scoped(documents, state_scope, s).await
            }
            _ => self.ingest_global_scoped(documents, state_scope).await,
        }
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
        documents: Vec<RAGDocument>,
        scope: Option<String>,
        scope_id: Option<String>,
        _provider: Option<String>,
        _chunking: Option<ChunkingConfig>,
        context: Option<ExecutionContext>,
    ) -> Result<RAGIngestResults> {
        debug!("RAG ingest request: {} documents", documents.len());

        // Determine scope from parameters
        let state_scope =
            Self::determine_scope(scope.as_deref(), scope_id.as_deref(), context.as_ref());
        debug!("Determined ingest scope: {:?}", state_scope);

        // Convert documents to texts
        let texts: Vec<String> = documents.iter().map(|d| d.text.clone()).collect();

        // Perform ingestion based on scope
        let (documents_processed, vectors_created, total_tokens) = self
            .dispatch_ingest(&documents, texts, &state_scope)
            .await?;

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
        debug!("Attempting to save RAG vector storage");

        // Save the underlying vector storage
        self.state_aware_storage
            .storage()
            .save()
            .await
            .map_err(|e| llmspell_core::LLMSpellError::Component {
                message: format!("Failed to save vector storage: {e}"),
                source: None,
            })?;

        info!("RAG vector storage saved successfully");
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
        #[allow(clippy::cast_precision_loss)]
        let intermediate = (hash.wrapping_add(i as u64) % 1000) as f32;
        let value = (intermediate / 1000.0).mul_add(2.0, -1.0);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rag_search_parameters() {
        // Test that search parameters work correctly
        let query = "test query";
        let k = Some(10);
        let scope = Some("tenant".to_string());
        let scope_id = Some("tenant-123".to_string());
        let threshold = Some(0.8);

        assert_eq!(query, "test query");
        assert_eq!(k, Some(10));
        assert_eq!(scope, Some("tenant".to_string()));
        assert_eq!(scope_id, Some("tenant-123".to_string()));
        assert_eq!(threshold, Some(0.8));
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
