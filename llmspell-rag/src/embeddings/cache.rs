//! Embedding cache for reducing API calls and costs

use anyhow::Result;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

/// Configuration for the embedding cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Maximum number of entries in the cache
    pub max_entries: usize,

    /// Maximum memory size in bytes
    pub max_memory_bytes: usize,

    /// TTL for cache entries
    pub ttl: Option<Duration>,

    /// Whether to persist cache to disk
    pub persist_to_disk: bool,

    /// Path for disk persistence
    pub cache_path: Option<String>,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_entries: 10_000,
            max_memory_bytes: 100 * 1024 * 1024,  // 100MB
            ttl: Some(Duration::from_secs(3600)), // 1 hour
            persist_to_disk: false,
            cache_path: None,
        }
    }
}

/// Cache entry for embeddings
#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry {
    /// The embedding vector
    embedding: Vec<f32>,

    /// When this entry was created
    created_at: SystemTime,

    /// Number of times this entry was accessed
    access_count: usize,

    /// Last access time
    last_accessed: SystemTime,
}

/// LRU cache for embeddings
#[derive(Debug)]
pub struct EmbeddingCache {
    entries: Arc<RwLock<HashMap<String, CacheEntry>>>,
    config: CacheConfig,
    stats: Arc<RwLock<CacheStats>>,
}

/// Cache statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct CacheStats {
    hits: usize,
    misses: usize,
    evictions: usize,
    total_entries: usize,
    total_memory_bytes: usize,
}

impl EmbeddingCache {
    /// Create a new embedding cache
    #[must_use]
    pub fn new(config: CacheConfig) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            config,
            stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }

    /// Get an embedding from cache
    #[must_use]
    #[allow(clippy::significant_drop_tightening)]
    pub fn get(&self, key: &str) -> Option<Vec<f32>> {
        let mut entries = self.entries.write();
        let mut stats = self.stats.write();

        if let Some(entry) = entries.get_mut(key) {
            // Check TTL
            if let Some(ttl) = self.config.ttl {
                if entry.created_at.elapsed().unwrap_or(Duration::MAX) > ttl {
                    entries.remove(key);
                    stats.evictions += 1;
                    stats.misses += 1;
                    return None;
                }
            }

            // Update access tracking
            entry.access_count += 1;
            entry.last_accessed = SystemTime::now();

            stats.hits += 1;
            Some(entry.embedding.clone())
        } else {
            stats.misses += 1;
            None
        }
    }

    /// Put an embedding in cache
    ///
    /// # Errors
    ///
    /// Currently always returns Ok, but may return errors in future implementations
    #[allow(clippy::significant_drop_tightening)]
    pub fn put(&self, key: String, embedding: Vec<f32>) -> Result<()> {
        let mut entries = self.entries.write();
        let mut stats = self.stats.write();

        // Check cache size limits
        if entries.len() >= self.config.max_entries {
            // Simple eviction: remove least recently used
            // TODO: Implement proper LRU eviction
            if let Some(oldest_key) = entries
                .iter()
                .min_by_key(|(_, e)| e.last_accessed)
                .map(|(k, _)| k.clone())
            {
                entries.remove(&oldest_key);
                stats.evictions += 1;
            }
        }

        let entry = CacheEntry {
            embedding,
            created_at: SystemTime::now(),
            access_count: 1,
            last_accessed: SystemTime::now(),
        };

        entries.insert(key, entry);
        stats.total_entries = entries.len();

        Ok(())
    }

    /// Clear the cache
    #[allow(clippy::significant_drop_tightening)]
    pub fn clear(&self) {
        let mut entries = self.entries.write();
        let mut stats = self.stats.write();

        entries.clear();
        stats.total_entries = 0;
        stats.total_memory_bytes = 0;
    }

    /// Get cache statistics
    #[must_use]
    pub fn stats(&self) -> (usize, usize, f64) {
        let stats = self.stats.read();
        let total = stats.hits + stats.misses;
        let hit_rate = if total > 0 {
            #[allow(clippy::cast_precision_loss)]
            {
                stats.hits as f64 / total as f64
            }
        } else {
            0.0
        };

        (stats.hits, stats.misses, hit_rate)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_basic_operations() {
        let cache = EmbeddingCache::new(CacheConfig::default());

        // Test put and get
        cache.put("test".to_string(), vec![1.0, 2.0, 3.0]).unwrap();
        let result = cache.get("test");
        assert!(result.is_some());
        assert_eq!(result.unwrap(), vec![1.0, 2.0, 3.0]);

        // Test miss
        assert!(cache.get("missing").is_none());

        // Check stats
        let (hits, misses, _) = cache.stats();
        assert_eq!(hits, 1);
        assert_eq!(misses, 1);
    }
}
