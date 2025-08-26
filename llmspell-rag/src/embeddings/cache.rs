//! Embedding cache for reducing API calls and costs

use anyhow::Result;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::hash::{Hash, Hasher};
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
    lru_order: Arc<RwLock<VecDeque<String>>>,
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
            entries: Arc::new(RwLock::new(HashMap::with_capacity(config.max_entries))),
            lru_order: Arc::new(RwLock::new(VecDeque::with_capacity(config.max_entries))),
            config,
            stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }

    /// Generate cache key from text content
    pub fn generate_key(text: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Get an embedding from cache
    #[must_use]
    #[allow(clippy::significant_drop_tightening)]
    pub fn get(&self, key: &str) -> Option<Vec<f32>> {
        let mut entries = self.entries.write();
        let mut lru_order = self.lru_order.write();
        let mut stats = self.stats.write();

        if let Some(entry) = entries.get_mut(key) {
            // Check TTL
            if let Some(ttl) = self.config.ttl {
                if entry.created_at.elapsed().unwrap_or(Duration::MAX) > ttl {
                    entries.remove(key);
                    lru_order.retain(|k| k != key);
                    stats.evictions += 1;
                    stats.misses += 1;
                    return None;
                }
            }

            // Update access tracking
            entry.access_count += 1;
            entry.last_accessed = SystemTime::now();

            // Move to front of LRU queue
            lru_order.retain(|k| k != key);
            lru_order.push_front(key.to_string());

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
        let mut lru_order = self.lru_order.write();
        let mut stats = self.stats.write();

        // Check if key already exists
        if entries.contains_key(&key) {
            // Update existing entry
            if let Some(entry) = entries.get_mut(&key) {
                entry.embedding = embedding;
                entry.last_accessed = SystemTime::now();
                entry.access_count += 1;
            }

            // Move to front of LRU
            lru_order.retain(|k| k != &key);
            lru_order.push_front(key);
            return Ok(());
        }

        // Check cache size limits
        while entries.len() >= self.config.max_entries {
            // LRU eviction: remove from back of queue
            if let Some(oldest_key) = lru_order.pop_back() {
                entries.remove(&oldest_key);
                stats.evictions += 1;
            } else {
                break;
            }
        }

        // Check memory limit (rough estimate: 4 bytes per float)
        let embedding_size = embedding.len() * 4;
        let current_memory = stats.total_memory_bytes;
        if current_memory + embedding_size > self.config.max_memory_bytes {
            // Evict entries until we have enough space
            while !lru_order.is_empty()
                && stats.total_memory_bytes + embedding_size > self.config.max_memory_bytes
            {
                if let Some(oldest_key) = lru_order.pop_back() {
                    if let Some(old_entry) = entries.remove(&oldest_key) {
                        stats.total_memory_bytes -= old_entry.embedding.len() * 4;
                        stats.evictions += 1;
                    }
                }
            }
        }

        let entry = CacheEntry {
            embedding,
            created_at: SystemTime::now(),
            access_count: 1,
            last_accessed: SystemTime::now(),
        };

        stats.total_memory_bytes += embedding_size;
        entries.insert(key.clone(), entry);
        lru_order.push_front(key);
        stats.total_entries = entries.len();

        Ok(())
    }

    /// Clear the cache
    #[allow(clippy::significant_drop_tightening)]
    pub fn clear(&self) {
        let mut entries = self.entries.write();
        let mut lru_order = self.lru_order.write();
        let mut stats = self.stats.write();

        entries.clear();
        lru_order.clear();
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

    #[test]
    fn test_lru_eviction() {
        let mut config = CacheConfig::default();
        config.max_entries = 3;
        let cache = EmbeddingCache::new(config);

        // Fill cache
        cache.put("a".to_string(), vec![1.0]).unwrap();
        cache.put("b".to_string(), vec![2.0]).unwrap();
        cache.put("c".to_string(), vec![3.0]).unwrap();

        // Access 'a' to make it more recent
        assert!(cache.get("a").is_some());

        // Add new entry, should evict 'b' (least recently used)
        cache.put("d".to_string(), vec![4.0]).unwrap();

        assert!(cache.get("a").is_some()); // Still present
        assert!(cache.get("b").is_none()); // Evicted
        assert!(cache.get("c").is_some()); // Still present
        assert!(cache.get("d").is_some()); // New entry
    }

    #[test]
    fn test_ttl_expiration() {
        let mut config = CacheConfig::default();
        config.ttl = Some(Duration::from_millis(100));
        let cache = EmbeddingCache::new(config);

        cache.put("test".to_string(), vec![1.0]).unwrap();
        assert!(cache.get("test").is_some());

        // Wait for TTL to expire
        std::thread::sleep(Duration::from_millis(150));
        assert!(cache.get("test").is_none());
    }

    #[test]
    fn test_cache_key_generation() {
        let key1 = EmbeddingCache::generate_key("Hello world");
        let key2 = EmbeddingCache::generate_key("Hello world");
        let key3 = EmbeddingCache::generate_key("Different text");

        assert_eq!(key1, key2); // Same text produces same key
        assert_ne!(key1, key3); // Different text produces different key
    }

    #[test]
    fn test_cache_clear() {
        let cache = EmbeddingCache::new(CacheConfig::default());

        cache.put("a".to_string(), vec![1.0]).unwrap();
        cache.put("b".to_string(), vec![2.0]).unwrap();

        assert!(cache.get("a").is_some());

        cache.clear();

        assert!(cache.get("a").is_none());
        assert!(cache.get("b").is_none());

        let (_hits, misses, _) = cache.stats();
        assert_eq!(misses, 2); // Two misses after clear
    }
}
