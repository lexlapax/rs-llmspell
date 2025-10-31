//! ABOUTME: Cached embedding service wrapper with LRU cache and SHA-256 hashing
//!
//! Optimizes embedding generation by caching results based on content hash,
//! avoiding redundant computations for identical or similar content.

use crate::embeddings::EmbeddingService;
use crate::error::Result;
use lru::LruCache;
use parking_lot::Mutex;
use sha2::{Digest, Sha256};
use std::num::NonZeroUsize;
use std::sync::Arc;
use tracing::{debug, info};

/// Cached embedding service with LRU cache and SHA-256 content hashing
///
/// Wraps an existing `EmbeddingService` with transparent caching layer.
/// Uses SHA-256 to hash content for cache keys, ensuring identical content
/// gets the same embedding without regeneration.
///
/// # Example
///
/// ```rust
/// use llmspell_memory::embeddings::{EmbeddingService, CachedEmbeddingService};
/// use llmspell_core::traits::embedding::EmbeddingProvider;
/// use std::sync::Arc;
///
/// # struct MockProvider;
/// # impl EmbeddingProvider for MockProvider {
/// #     fn name(&self) -> &str { "mock" }
/// #     async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, llmspell_core::LLMSpellError> {
/// #         Ok(vec![])
/// #     }
/// #     fn embedding_dimensions(&self) -> usize { 384 }
/// #     fn embedding_model(&self) -> Option<&str> { None }
/// # }
/// #
/// # async fn example() {
/// let provider: Arc<dyn EmbeddingProvider> = Arc::new(MockProvider);
/// let service = Arc::new(EmbeddingService::new(provider));
///
/// // Wrap with caching (10,000 entry capacity)
/// let cached = Arc::new(CachedEmbeddingService::new(service, 10_000));
/// # }
/// ```
#[derive(Clone)]
pub struct CachedEmbeddingService {
    /// Underlying embedding service
    inner: Arc<EmbeddingService>,

    /// LRU cache: SHA-256 hash → embedding vector
    cache: Arc<Mutex<LruCache<String, Vec<f32>>>>,

    /// Cache statistics
    stats: Arc<Mutex<CacheStats>>,
}

impl std::fmt::Debug for CachedEmbeddingService {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CachedEmbeddingService")
            .field("provider", &self.inner.provider_name())
            .field("cache_capacity", &self.cache.lock().cap())
            .field("cache_size", &self.cache.lock().len())
            .field("stats", &*self.stats.lock())
            .finish()
    }
}

impl CachedEmbeddingService {
    /// Create new cached embedding service
    ///
    /// # Arguments
    ///
    /// * `inner` - Underlying embedding service to wrap
    /// * `capacity` - Maximum number of embeddings to cache (LRU eviction)
    ///
    /// # Panics
    ///
    /// Panics if capacity is 0
    pub fn new(inner: Arc<EmbeddingService>, capacity: usize) -> Self {
        info!(
            "Creating CachedEmbeddingService: provider={}, cache_capacity={}",
            inner.provider_name(),
            capacity
        );

        Self {
            inner,
            cache: Arc::new(Mutex::new(
                LruCache::new(NonZeroUsize::new(capacity).expect("Cache capacity must be non-zero")),
            )),
            stats: Arc::new(Mutex::new(CacheStats::default())),
        }
    }

    /// Generate embedding for single text with caching
    ///
    /// Checks cache first using SHA-256 content hash. On miss, generates
    /// embedding via inner service and stores in cache.
    ///
    /// # Errors
    ///
    /// Returns error if embedding generation fails or provider is unavailable
    pub async fn embed_single(&self, text: &str) -> Result<Vec<f32>> {
        let content_hash = Self::hash_content(text);

        // Check cache
        {
            let mut cache = self.cache.lock();
            if let Some(embedding) = cache.get(&content_hash) {
                debug!(
                    "Cache hit for content hash: {}... (length: {})",
                    &content_hash[..8],
                    text.len()
                );
                self.stats.lock().record_hit();
                return Ok(embedding.clone());
            }
        }

        debug!(
            "Cache miss for content hash: {}... (length: {})",
            &content_hash[..8],
            text.len()
        );
        self.stats.lock().record_miss();

        // Generate embedding via inner service
        let embedding = self.inner.embed_single(text).await?;

        // Store in cache
        {
            let mut cache = self.cache.lock();
            cache.put(content_hash, embedding.clone());
        }

        Ok(embedding)
    }

    /// Generate embeddings for multiple texts with batch caching
    ///
    /// Checks cache for each text, generates only cache misses in batch,
    /// then assembles results in original order.
    ///
    /// # Errors
    ///
    /// Returns error if embedding generation fails or provider is unavailable
    #[allow(clippy::significant_drop_tightening)]
    pub async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        info!(
            "Generating batch of {} embeddings with caching",
            texts.len()
        );

        let mut results = Vec::with_capacity(texts.len());
        let mut to_generate = Vec::new();
        let mut to_generate_indices = Vec::new();

        // Check cache for each text
        {
            let mut cache = self.cache.lock();
            for (i, text) in texts.iter().enumerate() {
                let content_hash = Self::hash_content(text);

                if let Some(embedding) = cache.get(&content_hash) {
                    results.push((i, embedding.clone()));
                    self.stats.lock().record_hit();
                } else {
                    to_generate.push(text.clone());
                    to_generate_indices.push((i, content_hash));
                    self.stats.lock().record_miss();
                }
            }
        }

        debug!(
            "Batch cache stats: hits={}, misses={}",
            results.len(),
            to_generate.len()
        );

        // Generate missing embeddings
        if !to_generate.is_empty() {
            let generated = self.inner.embed_batch(&to_generate).await?;

            // Store in cache and add to results
            {
                let mut cache = self.cache.lock();
                for ((idx, content_hash), embedding) in
                    to_generate_indices.into_iter().zip(generated.into_iter())
                {
                    cache.put(content_hash, embedding.clone());
                    results.push((idx, embedding));
                }
            } // Drop cache lock after updates
        }

        // Sort by original index and extract embeddings
        results.sort_unstable_by_key(|(idx, _)| *idx);
        let embeddings = results.into_iter().map(|(_, emb)| emb).collect();

        Ok(embeddings)
    }

    /// Get embedding dimensions
    #[must_use]
    pub fn dimensions(&self) -> usize {
        self.inner.dimensions()
    }

    /// Get provider name
    #[must_use]
    pub fn provider_name(&self) -> &str {
        self.inner.provider_name()
    }

    /// Get cache statistics
    #[must_use]
    pub fn stats(&self) -> CacheStats {
        *self.stats.lock()
    }

    /// Clear cache and reset statistics
    pub fn clear(&self) {
        {
            let mut cache = self.cache.lock();
            cache.clear();
        } // Drop cache lock early

        {
            let mut stats = self.stats.lock();
            *stats = CacheStats::default();
        } // Drop stats lock early

        info!("Cache cleared and statistics reset");
    }

    /// Hash content using SHA-256 for cache key
    fn hash_content(content: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(content.as_bytes());
        let result = hasher.finalize();
        format!("{result:x}")
    }
}

/// Cache statistics
#[derive(Debug, Clone, Copy, Default)]
pub struct CacheStats {
    /// Total cache hits
    pub hits: u64,

    /// Total cache misses
    pub misses: u64,
}

impl CacheStats {
    /// Record cache hit
    const fn record_hit(&mut self) {
        self.hits += 1;
    }

    /// Record cache miss
    const fn record_miss(&mut self) {
        self.misses += 1;
    }

    /// Calculate cache hit rate (0.0 to 1.0)
    #[must_use]
    pub const fn hit_rate(&self) -> f64 {
        let total = self.hits + self.misses;
        if total == 0 {
            0.0
        } else {
            #[allow(clippy::cast_precision_loss)]
            {
                self.hits as f64 / total as f64
            }
        }
    }

    /// Get total requests
    #[must_use]
    pub const fn total(&self) -> u64 {
        self.hits + self.misses
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use llmspell_core::error::LLMSpellError;
    use llmspell_core::traits::embedding::EmbeddingProvider;

    /// Mock embedding provider for testing
    struct MockEmbeddingProvider;

    #[async_trait]
    impl EmbeddingProvider for MockEmbeddingProvider {
        fn name(&self) -> &str {
            "mock-cached"
        }

        async fn embed(&self, texts: &[String]) -> std::result::Result<Vec<Vec<f32>>, LLMSpellError> {
            // Generate deterministic embeddings based on text length
            Ok(texts
                .iter()
                .map(|t| vec![t.len() as f32, (t.len() * 2) as f32, (t.len() * 3) as f32])
                .collect())
        }

        fn embedding_dimensions(&self) -> usize {
            3
        }

        fn supports_dimension_reduction(&self) -> bool {
            false
        }

        fn set_embedding_dimensions(&mut self, _dims: usize) -> std::result::Result<(), LLMSpellError> {
            Err(LLMSpellError::Provider {
                message: "Dimension configuration not supported".to_string(),
                provider: Some(self.name().to_string()),
                source: None,
            })
        }

        fn embedding_model(&self) -> Option<&str> {
            Some("mock-model")
        }

        fn embedding_cost_per_token(&self) -> Option<f64> {
            None
        }
    }

    #[tokio::test]
    async fn test_cache_hit() {
        let provider = Arc::new(MockEmbeddingProvider);
        let service = Arc::new(EmbeddingService::new(provider));
        let cached = CachedEmbeddingService::new(service, 100);

        // First call - cache miss
        let embedding1 = cached.embed_single("test").await.unwrap();
        assert_eq!(embedding1, vec![4.0, 8.0, 12.0]); // len=4

        // Second call - cache hit
        let embedding2 = cached.embed_single("test").await.unwrap();
        assert_eq!(embedding1, embedding2);

        // Check stats
        let stats = cached.stats();
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.misses, 1);
        assert_eq!(stats.hit_rate(), 0.5);
    }

    #[tokio::test]
    async fn test_cache_miss_different_content() {
        let provider = Arc::new(MockEmbeddingProvider);
        let service = Arc::new(EmbeddingService::new(provider));
        let cached = CachedEmbeddingService::new(service, 100);

        let embedding1 = cached.embed_single("test").await.unwrap();
        let embedding2 = cached.embed_single("longer test").await.unwrap();

        // Different content should have different embeddings (due to length)
        assert_ne!(embedding1, embedding2);

        // Both should be cache misses
        let stats = cached.stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 2);
    }

    #[tokio::test]
    async fn test_batch_caching() {
        let provider = Arc::new(MockEmbeddingProvider);
        let service = Arc::new(EmbeddingService::new(provider));
        let cached = CachedEmbeddingService::new(service, 100);

        // First batch
        let texts1 = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let embeddings1 = cached.embed_batch(&texts1).await.unwrap();
        assert_eq!(embeddings1.len(), 3);

        // Second batch with some overlap
        let texts2 = vec!["a".to_string(), "b".to_string(), "d".to_string()];
        let embeddings2 = cached.embed_batch(&texts2).await.unwrap();
        assert_eq!(embeddings2.len(), 3);

        // "a" and "b" should be cache hits
        assert_eq!(embeddings1[0], embeddings2[0]); // "a"
        assert_eq!(embeddings1[1], embeddings2[1]); // "b"

        // Check stats: 6 requests, 2 hits (a, b in second batch)
        let stats = cached.stats();
        assert_eq!(stats.total(), 6);
        assert_eq!(stats.hits, 2);
        assert_eq!(stats.misses, 4);
    }

    #[tokio::test]
    async fn test_clear_cache() {
        let provider = Arc::new(MockEmbeddingProvider);
        let service = Arc::new(EmbeddingService::new(provider));
        let cached = CachedEmbeddingService::new(service, 100);

        cached.embed_single("test").await.unwrap();
        assert_eq!(cached.stats().misses, 1);

        cached.clear();

        let stats = cached.stats();
        assert_eq!(stats.hits, 0);
        assert_eq!(stats.misses, 0);

        // After clear, should be cache miss again
        cached.embed_single("test").await.unwrap();
        assert_eq!(cached.stats().misses, 1);
    }

    #[test]
    fn test_cache_stats() {
        let mut stats = CacheStats::default();
        assert_eq!(stats.hit_rate(), 0.0);

        stats.record_miss();
        assert_eq!(stats.hit_rate(), 0.0);

        stats.record_hit();
        assert_eq!(stats.hit_rate(), 0.5);

        stats.record_hit();
        assert_eq!(stats.hit_rate(), 2.0 / 3.0);
    }

    #[test]
    fn test_hash_content() {
        let hash1 = CachedEmbeddingService::hash_content("test");
        let hash2 = CachedEmbeddingService::hash_content("test");
        let hash3 = CachedEmbeddingService::hash_content("test2");

        // Same content should have same hash
        assert_eq!(hash1, hash2);

        // Different content should have different hash
        assert_ne!(hash1, hash3);

        // Hash should be 64 hex characters (SHA-256)
        assert_eq!(hash1.len(), 64);
    }
}
