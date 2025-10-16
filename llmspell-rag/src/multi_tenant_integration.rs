//! Multi-tenant RAG integration with cost tracking and tenant isolation

use anyhow::Result;
use llmspell_core::state::StateScope;
use llmspell_tenancy::MultiTenantVectorManager;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};

/// Multi-tenant RAG wrapper with cost tracking and isolation
pub struct MultiTenantRAG {
    /// Tenant manager for isolation and usage tracking
    #[allow(dead_code)]
    tenant_manager: Arc<MultiTenantVectorManager>,
    /// Usage metrics cache
    usage_cache: Arc<RwLock<HashMap<String, TenantUsageMetrics>>>,
}

impl std::fmt::Debug for MultiTenantRAG {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MultiTenantRAG")
            .field("tenant_manager", &"Arc<MultiTenantVectorManager>")
            .field("usage_cache", &"Arc<RwLock<HashMap<...>>>")
            .finish()
    }
}

/// Tenant-specific usage metrics for RAG operations
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TenantUsageMetrics {
    /// Number of embeddings generated
    pub embeddings_generated: u64,
    /// Total tokens processed for embeddings
    pub embedding_tokens: u64,
    /// Number of vector searches performed
    pub searches_performed: u64,
    /// Number of documents indexed
    pub documents_indexed: u64,
    /// Storage usage in bytes
    pub storage_bytes: u64,
    /// Total embedding costs (in cents)
    pub embedding_cost_cents: u64,
}

impl TenantUsageMetrics {
    /// Add embedding usage to the metrics
    pub const fn add_embedding_usage(&mut self, tokens: u64, cost_cents: u64) {
        self.embeddings_generated += 1;
        self.embedding_tokens += tokens;
        self.embedding_cost_cents += cost_cents;
    }

    /// Add search usage
    pub const fn add_search_usage(&mut self) {
        self.searches_performed += 1;
    }

    /// Add document indexing usage
    pub const fn add_document_usage(&mut self, storage_bytes: u64) {
        self.documents_indexed += 1;
        self.storage_bytes += storage_bytes;
    }
}

/// Tenant-aware vector routing configuration
#[derive(Debug, Clone)]
pub struct TenantVectorConfig {
    /// Tenant ID
    pub tenant_id: String,
    /// Namespace prefix for vector storage
    pub namespace_prefix: String,
    /// Maximum vectors allowed for this tenant
    pub max_vectors: Option<u64>,
    /// Maximum storage bytes allowed
    pub max_storage_bytes: Option<u64>,
    /// Cost limits per month in cents
    pub monthly_cost_limit_cents: Option<u64>,
}

impl TenantVectorConfig {
    /// Create configuration for a tenant
    #[must_use]
    pub fn new(tenant_id: impl Into<String>) -> Self {
        let tenant_id = tenant_id.into();
        Self {
            namespace_prefix: format!("tenant:{tenant_id}"),
            tenant_id,
            max_vectors: None,
            max_storage_bytes: None,
            monthly_cost_limit_cents: None,
        }
    }

    /// Set vector limit
    #[must_use]
    pub const fn with_max_vectors(mut self, limit: u64) -> Self {
        self.max_vectors = Some(limit);
        self
    }

    /// Set storage limit
    #[must_use]
    pub const fn with_max_storage_bytes(mut self, limit: u64) -> Self {
        self.max_storage_bytes = Some(limit);
        self
    }

    /// Set monthly cost limit
    #[must_use]
    pub const fn with_monthly_cost_limit_cents(mut self, limit: u64) -> Self {
        self.monthly_cost_limit_cents = Some(limit);
        self
    }
}

impl MultiTenantRAG {
    /// Create a new multi-tenant RAG instance
    #[must_use]
    pub fn new(tenant_manager: Arc<MultiTenantVectorManager>) -> Self {
        Self {
            tenant_manager,
            usage_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get tenant usage metrics
    ///
    /// # Errors
    /// Returns an error if unable to retrieve usage metrics
    pub async fn get_tenant_usage(&self, tenant_id: &str) -> Result<TenantUsageMetrics> {
        let cache = self.usage_cache.read().await;
        Ok(cache.get(tenant_id).cloned().unwrap_or_default())
    }

    /// Update usage metrics for a tenant
    #[allow(clippy::significant_drop_tightening)]
    async fn update_usage_metrics<F>(&self, tenant_id: &str, updater: F) -> Result<()>
    where
        F: FnOnce(&mut TenantUsageMetrics),
    {
        {
            let mut cache = self.usage_cache.write().await;
            let metrics = cache.entry(tenant_id.to_string()).or_default();
            updater(metrics);
        }

        debug!("Updated usage metrics for tenant {tenant_id}");
        Ok(())
    }

    /// Generate embeddings for a tenant with cost tracking
    ///
    /// # Errors
    /// Returns an error if unable to generate embeddings
    pub async fn generate_tenant_embeddings(
        &self,
        tenant_id: &str,
        texts: &[String],
    ) -> Result<Vec<Vec<f32>>> {
        debug!(
            "Generating embeddings for tenant {tenant_id} with {} texts",
            texts.len()
        );

        // Simple mock embedding generation
        let embeddings: Vec<Vec<f32>> = texts
            .iter()
            .map(|_text| {
                // Simple mock embedding - in reality this would call the provider
                vec![0.1; 384] // Standard embedding dimension
            })
            .collect();

        // Calculate costs (simplified cost model)
        let total_tokens: u64 = texts.iter().map(|t| t.len() as u64).sum();
        let cost_per_1k_tokens = 10; // 0.1 cents per 1k tokens
        let total_cost_cents = (total_tokens * cost_per_1k_tokens) / 1000;

        // Update usage metrics
        self.update_usage_metrics(tenant_id, |metrics| {
            metrics.add_embedding_usage(total_tokens, total_cost_cents);
        })
        .await?;

        info!(
            "Generated {} embeddings for tenant {tenant_id} (tokens: {total_tokens}, cost: {total_cost_cents} cents)",
            embeddings.len()
        );

        Ok(embeddings)
    }

    /// Get tenant configuration
    ///
    /// # Errors
    /// Returns an error if unable to retrieve configuration
    pub fn get_tenant_config(&self, tenant_id: &str) -> Result<Option<TenantVectorConfig>> {
        // Return a default config for demonstration
        Ok(Some(TenantVectorConfig::new(tenant_id)))
    }

    /// Check if tenant has exceeded usage limits
    ///
    /// # Errors
    /// Returns an error if unable to check usage limits
    pub async fn check_usage_limits(&self, tenant_id: &str) -> Result<bool> {
        let config = self.get_tenant_config(tenant_id)?;
        let usage = self.get_tenant_usage(tenant_id).await?;

        if let Some(config) = config {
            // Check vector count limit
            if let Some(max_vectors) = config.max_vectors {
                if usage.documents_indexed > max_vectors {
                    return Ok(false);
                }
            }

            // Check storage limit
            if let Some(max_storage_bytes) = config.max_storage_bytes {
                if usage.storage_bytes > max_storage_bytes {
                    return Ok(false);
                }
            }

            // Check cost limit
            if let Some(max_cost_cents) = config.monthly_cost_limit_cents {
                if usage.embedding_cost_cents > max_cost_cents {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    /// Clear tenant data (for tenant deletion)
    ///
    /// # Errors
    /// Returns an error if unable to clear tenant data
    pub async fn clear_tenant_data(&self, tenant_id: &str) -> Result<()> {
        // Clear from usage cache
        {
            let mut cache = self.usage_cache.write().await;
            cache.remove(tenant_id);
        }

        info!("Cleared RAG data for tenant {tenant_id}");
        Ok(())
    }

    /// Create a tenant-scoped `StateScope`
    #[must_use]
    pub fn create_tenant_scope(&self, tenant_id: &str) -> StateScope {
        StateScope::Custom(format!("tenant:{tenant_id}"))
    }

    /// Ingest documents with embeddings into vector storage
    ///
    /// This method combines embedding generation and storage insertion in a single operation.
    /// It generates embeddings for the provided texts, creates `VectorEntry` structs with metadata,
    /// and stores them in the vector storage with proper tenant isolation.
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation and billing
    /// * `texts` - Text documents to embed and store
    /// * `scope` - `StateScope` for organizing vectors (e.g., session, project)
    /// * `metadata_fn` - Optional function to add custom metadata to each vector
    ///
    /// # Returns
    /// Vector of IDs for the stored vectors
    ///
    /// # Errors
    /// Returns error if embedding generation fails or storage insertion fails
    pub async fn ingest_documents<F>(
        &self,
        tenant_id: &str,
        texts: &[String],
        scope: StateScope,
        metadata_fn: Option<F>,
    ) -> Result<Vec<String>>
    where
        F: Fn(usize, &str) -> HashMap<String, serde_json::Value>,
    {
        info!(
            "Ingesting {} documents for tenant {} with scope {:?}",
            texts.len(),
            tenant_id,
            scope
        );

        // Generate embeddings
        let embeddings = self.generate_tenant_embeddings(tenant_id, texts).await?;

        // Create VectorEntry structs
        let mut vectors = Vec::new();
        for (i, (text, embedding)) in texts.iter().zip(embeddings.iter()).enumerate() {
            let id = uuid::Uuid::new_v4().to_string();

            // Build metadata
            let mut metadata = metadata_fn
                .as_ref()
                .map_or_else(HashMap::new, |meta_fn| meta_fn(i, text));

            // Add default metadata
            metadata.insert(
                "text".to_string(),
                serde_json::Value::String(text.clone()),
            );
            metadata.insert(
                "ingested_at".to_string(),
                serde_json::Value::String(
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs()
                        .to_string(),
                ),
            );
            metadata.insert(
                "tenant_id".to_string(),
                serde_json::Value::String(tenant_id.to_string()),
            );

            // Create vector entry
            let entry = llmspell_storage::VectorEntry::new(id, embedding.clone())
                .with_scope(scope.clone())
                .with_metadata(metadata);

            vectors.push(entry);
        }

        // Calculate storage size for metrics
        let storage_bytes: u64 = vectors
            .iter()
            .map(|v| (v.embedding.len() * 4) as u64)
            .sum();

        // Insert into storage
        let ids = self
            .tenant_manager
            .insert_for_tenant(tenant_id, vectors)
            .await?;

        // Update usage metrics
        self.update_usage_metrics(tenant_id, |metrics| {
            metrics.add_document_usage(storage_bytes);
        })
        .await?;

        info!(
            "Successfully ingested {} documents for tenant {} (ids: {:?})",
            texts.len(),
            tenant_id,
            ids
        );

        Ok(ids)
    }

    /// Retrieve context from RAG storage based on query
    ///
    /// This method generates an embedding for the query text and performs similarity search
    /// to find the most relevant documents from the vector storage.
    ///
    /// # Arguments
    /// * `tenant_id` - Tenant identifier for isolation
    /// * `query` - Query text to search for
    /// * `scope` - `StateScope` to restrict search (e.g., specific session)
    /// * `k` - Number of results to return
    ///
    /// # Returns
    /// Vector of retrieval results with text, score, and metadata
    ///
    /// # Errors
    /// Returns error if embedding generation fails or search fails
    pub async fn retrieve_context(
        &self,
        tenant_id: &str,
        query: &str,
        scope: StateScope,
        k: usize,
    ) -> Result<Vec<RetrievalResult>> {
        info!(
            "Retrieving context for tenant {} with query '{}' (k={}, scope={:?})",
            tenant_id, query, k, scope
        );

        // Generate query embedding
        let query_embeddings = self
            .generate_tenant_embeddings(tenant_id, &[query.to_string()])
            .await?;

        let query_embedding = query_embeddings
            .first()
            .ok_or_else(|| anyhow::anyhow!("Failed to generate query embedding"))?;

        // Create vector query
        let vector_query = llmspell_storage::VectorQuery::new(query_embedding.clone(), k)
            .with_scope(scope.clone());

        // Perform search
        let results = self
            .tenant_manager
            .search_for_tenant(tenant_id, vector_query)
            .await?;

        // Convert to retrieval results
        let retrieval_results: Vec<RetrievalResult> = results
            .into_iter()
            .map(|r| RetrievalResult {
                id: r.id,
                text: r
                    .metadata
                    .as_ref()
                    .and_then(|m| m.get("text"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("")
                    .to_string(),
                score: r.score,
                metadata: r.metadata.unwrap_or_default(),
            })
            .collect();

        // Update search metrics
        self.update_usage_metrics(tenant_id, |metrics| {
            metrics.add_search_usage();
        })
        .await?;

        info!(
            "Retrieved {} results for tenant {} query",
            retrieval_results.len(),
            tenant_id
        );

        Ok(retrieval_results)
    }
}

/// Result from RAG retrieval operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalResult {
    /// Vector ID
    pub id: String,
    /// Retrieved text content
    pub text: String,
    /// Similarity score (higher is better)
    pub score: f32,
    /// Associated metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tenant_usage_metrics() {
        let mut metrics = TenantUsageMetrics::default();

        metrics.add_embedding_usage(100, 50);
        assert_eq!(metrics.embeddings_generated, 1);
        assert_eq!(metrics.embedding_tokens, 100);
        assert_eq!(metrics.embedding_cost_cents, 50);

        metrics.add_search_usage();
        assert_eq!(metrics.searches_performed, 1);

        metrics.add_document_usage(1024);
        assert_eq!(metrics.documents_indexed, 1);
        assert_eq!(metrics.storage_bytes, 1024);
    }

    #[tokio::test]
    async fn test_tenant_vector_config() {
        let config = TenantVectorConfig::new("test-tenant")
            .with_max_vectors(10000)
            .with_max_storage_bytes(1024 * 1024 * 100) // 100MB
            .with_monthly_cost_limit_cents(5000); // $50

        assert_eq!(config.tenant_id, "test-tenant");
        assert_eq!(config.namespace_prefix, "tenant:test-tenant");
        assert_eq!(config.max_vectors, Some(10000));
        assert_eq!(config.max_storage_bytes, Some(1024 * 1024 * 100));
        assert_eq!(config.monthly_cost_limit_cents, Some(5000));
    }
}
