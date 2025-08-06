// ABOUTME: Cache module providing TTL-based and LRU eviction policies for hook result caching
// ABOUTME: Implements thread-safe caching with configurable size limits and expiration policies

pub mod ttl;

use crate::context::HookContext;
use crate::result::HookResult;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

pub use ttl::{TtlCache, TtlEntry};

/// Cache key generated from hook context
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CacheKey {
    pub hook_point: String,
    pub component_id: String,
    pub context_hash: u64,
    pub language: String,
}

impl CacheKey {
    /// Generate a cache key from hook context
    pub fn from_context(context: &HookContext) -> Self {
        let mut hasher = DefaultHasher::new();

        // Hash the context data for uniqueness
        if let Ok(json) = serde_json::to_string(&context.data) {
            json.hash(&mut hasher);
        }

        // Also hash metadata for completeness
        for (key, value) in &context.metadata {
            key.hash(&mut hasher);
            value.hash(&mut hasher);
        }

        Self {
            hook_point: format!("{:?}", context.point),
            component_id: format!(
                "{:?}:{}",
                context.component_id.component_type, context.component_id.name
            ),
            context_hash: hasher.finish(),
            language: format!("{:?}", context.language),
        }
    }

    /// Convert to string representation for storage
    pub fn as_cache_key(&self) -> String {
        format!(
            "{}:{}:{}:{}",
            self.hook_point, self.component_id, self.context_hash, self.language
        )
    }
}

/// Cached entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheEntry {
    pub key: CacheKey,
    pub result: HookResult,
    pub created_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub access_count: u64,
    pub ttl: Duration,
}

impl CacheEntry {
    pub fn new(key: CacheKey, result: HookResult, ttl: Duration) -> Self {
        let now = Utc::now();
        Self {
            key,
            result,
            created_at: now,
            last_accessed: now,
            access_count: 1,
            ttl,
        }
    }

    /// Check if the entry has expired
    pub fn is_expired(&self) -> bool {
        let elapsed = Utc::now().signed_duration_since(self.created_at);
        elapsed.to_std().unwrap_or(Duration::ZERO) > self.ttl
    }

    /// Update access metadata
    pub fn mark_accessed(&mut self) {
        self.last_accessed = Utc::now();
        self.access_count += 1;
    }
}

/// LRU eviction policy
#[derive(Debug)]
pub struct LruEviction {
    /// Access order queue (most recent at back)
    access_order: Arc<RwLock<VecDeque<String>>>,
    /// Maximum number of entries
    max_entries: usize,
}

impl LruEviction {
    pub fn new(max_entries: usize) -> Self {
        Self {
            access_order: Arc::new(RwLock::new(VecDeque::new())),
            max_entries,
        }
    }

    /// Record access to a key
    pub fn record_access(&self, key: &str) {
        let mut queue = self.access_order.write().unwrap();

        // Remove existing entry if present
        if let Some(pos) = queue.iter().position(|k| k == key) {
            queue.remove(pos);
        }

        // Add to back (most recent)
        queue.push_back(key.to_string());
    }

    /// Get keys to evict based on LRU policy
    pub fn get_eviction_candidates(&self, current_size: usize) -> Vec<String> {
        if current_size <= self.max_entries {
            return Vec::new();
        }

        let queue = self.access_order.read().unwrap();
        let evict_count = current_size - self.max_entries;

        // Return least recently used entries (from front)
        queue.iter().take(evict_count).cloned().collect()
    }

    /// Remove key from tracking
    pub fn remove_key(&self, key: &str) {
        let mut queue = self.access_order.write().unwrap();
        if let Some(pos) = queue.iter().position(|k| k == key) {
            queue.remove(pos);
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CacheStats {
    pub total_requests: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub evictions: u64,
    pub expired_entries: u64,
    pub current_size: usize,
    pub max_size: usize,
}

impl CacheStats {
    pub fn hit_ratio(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            #[allow(clippy::cast_precision_loss)]
            let cache_hits_f64 = self.cache_hits as f64;
            #[allow(clippy::cast_precision_loss)]
            let total_requests_f64 = self.total_requests as f64;
            cache_hits_f64 / total_requests_f64
        }
    }

    pub fn miss_ratio(&self) -> f64 {
        1.0 - self.hit_ratio()
    }
}

/// Thread-safe cache with TTL and LRU eviction
#[derive(Debug)]
pub struct Cache {
    /// Cache storage
    entries: Arc<RwLock<HashMap<String, CacheEntry>>>,
    /// LRU eviction policy
    lru: LruEviction,
    /// Cache statistics
    stats: Arc<RwLock<CacheStats>>,
    /// Default TTL for entries
    default_ttl: Duration,
    /// Background cleanup interval
    cleanup_interval: Duration,
    /// Last cleanup time
    last_cleanup: Arc<RwLock<Instant>>,
}

impl Cache {
    /// Create a new cache with specified configuration
    pub fn new(max_entries: usize, default_ttl: Duration) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            lru: LruEviction::new(max_entries),
            stats: Arc::new(RwLock::new(CacheStats {
                max_size: max_entries,
                ..Default::default()
            })),
            default_ttl,
            cleanup_interval: Duration::from_secs(300), // 5 minutes
            last_cleanup: Arc::new(RwLock::new(Instant::now())),
        }
    }

    /// Get an entry from the cache
    pub fn get(&self, key: &CacheKey) -> Option<HookResult> {
        let key_str = key.as_cache_key();

        // Update stats
        {
            let mut stats = self.stats.write().unwrap();
            stats.total_requests += 1;
        }

        // Try to get the entry
        let mut entries = self.entries.write().unwrap();
        if let Some(entry) = entries.get_mut(&key_str) {
            // Check if expired
            if entry.is_expired() {
                entries.remove(&key_str);
                self.lru.remove_key(&key_str);

                let mut stats = self.stats.write().unwrap();
                stats.cache_misses += 1;
                stats.expired_entries += 1;
                stats.current_size = entries.len();

                return None;
            }

            // Update access metadata
            entry.mark_accessed();
            self.lru.record_access(&key_str);

            // Update stats
            {
                let mut stats = self.stats.write().unwrap();
                stats.cache_hits += 1;
            }

            Some(entry.result.clone())
        } else {
            // Cache miss
            let mut stats = self.stats.write().unwrap();
            stats.cache_misses += 1;
            None
        }
    }

    /// Put an entry into the cache
    pub fn put(&self, key: CacheKey, result: HookResult, ttl: Option<Duration>) -> Result<()> {
        let key_str = key.as_cache_key();
        let ttl = ttl.unwrap_or(self.default_ttl);
        let entry = CacheEntry::new(key, result, ttl);

        let mut entries = self.entries.write().unwrap();

        // Insert the entry
        entries.insert(key_str.clone(), entry);
        self.lru.record_access(&key_str);

        // Check if we need to evict entries
        let eviction_candidates = self.lru.get_eviction_candidates(entries.len());
        for evict_key in eviction_candidates {
            entries.remove(&evict_key);
            self.lru.remove_key(&evict_key);

            let mut stats = self.stats.write().unwrap();
            stats.evictions += 1;
        }

        // Update stats
        {
            let mut stats = self.stats.write().unwrap();
            stats.current_size = entries.len();
        }

        // Periodic cleanup of expired entries
        self.cleanup_if_needed();

        Ok(())
    }

    /// Remove an entry from the cache
    pub fn remove(&self, key: &CacheKey) -> bool {
        let key_str = key.as_cache_key();
        let mut entries = self.entries.write().unwrap();

        if entries.remove(&key_str).is_some() {
            self.lru.remove_key(&key_str);
            let mut stats = self.stats.write().unwrap();
            stats.current_size = entries.len();
            true
        } else {
            false
        }
    }

    /// Clear all entries from the cache
    pub fn clear(&self) {
        let mut entries = self.entries.write().unwrap();
        entries.clear();

        let mut stats = self.stats.write().unwrap();
        stats.current_size = 0;

        // Reset LRU tracking
        let access_order = self.lru.access_order.clone();
        let mut queue = access_order.write().unwrap();
        queue.clear();
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        self.stats.read().unwrap().clone()
    }

    /// Get current cache size
    pub fn size(&self) -> usize {
        self.entries.read().unwrap().len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.entries.read().unwrap().is_empty()
    }

    /// Cleanup expired entries if needed
    fn cleanup_if_needed(&self) {
        let now = Instant::now();
        let should_cleanup = {
            let last_cleanup = self.last_cleanup.read().unwrap();
            now.duration_since(*last_cleanup) > self.cleanup_interval
        };

        if should_cleanup {
            self.cleanup_expired_entries();
            *self.last_cleanup.write().unwrap() = now;
        }
    }

    /// Remove all expired entries
    fn cleanup_expired_entries(&self) {
        let mut entries = self.entries.write().unwrap();
        let mut expired_keys = Vec::new();

        for (key, entry) in entries.iter() {
            if entry.is_expired() {
                expired_keys.push(key.clone());
            }
        }

        let expired_count = expired_keys.len();
        for key in expired_keys {
            entries.remove(&key);
            self.lru.remove_key(&key);
        }

        if expired_count > 0 {
            let mut stats = self.stats.write().unwrap();
            #[allow(clippy::cast_possible_truncation)]
            let expired_count_u64 = expired_count as u64;
            stats.expired_entries += expired_count_u64;
            stats.current_size = entries.len();
        }
    }

    /// Force cleanup of all expired entries
    pub fn cleanup(&self) {
        self.cleanup_expired_entries();
    }
}

impl Default for Cache {
    fn default() -> Self {
        Self::new(1000, Duration::from_secs(300)) // 1000 entries, 5 minute TTL
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ComponentId, ComponentType, HookPoint};
    use std::thread;
    use std::time::Duration as StdDuration;

    /// Local test helper to avoid circular dependency with llmspell-testing
    /// (Architectural exception per 7.1.6: foundational crates may have minimal local helpers)
    fn create_test_context() -> HookContext {
        let component_id = ComponentId::new(ComponentType::System, "test".to_string());
        HookContext::new(HookPoint::SystemStartup, component_id)
    }
    #[test]
    fn test_cache_key_generation() {
        let context = create_test_context();
        let key1 = CacheKey::from_context(&context);
        let key2 = CacheKey::from_context(&context);

        // Same context should generate same key
        assert_eq!(key1, key2);
        assert_eq!(key1.hook_point, "SystemStartup");
        assert!(key1.component_id.contains("System:test"));
    }
    #[test]
    fn test_cache_key_uniqueness() {
        let mut context1 = create_test_context();
        let mut context2 = create_test_context();

        context1.insert_data("key".to_string(), serde_json::json!("value1"));
        context2.insert_data("key".to_string(), serde_json::json!("value2"));

        let key1 = CacheKey::from_context(&context1);
        let key2 = CacheKey::from_context(&context2);

        // Different data should generate different keys
        assert_ne!(key1, key2);
        assert_ne!(key1.context_hash, key2.context_hash);
    }
    #[test]
    fn test_cache_entry_expiration() {
        let key = CacheKey {
            hook_point: "test".to_string(),
            component_id: "test".to_string(),
            context_hash: 123,
            language: "Native".to_string(),
        };

        let mut entry = CacheEntry::new(key, HookResult::Continue, Duration::from_millis(100));

        // Should not be expired immediately
        assert!(!entry.is_expired());

        // Wait and check expiration
        thread::sleep(StdDuration::from_millis(150));
        assert!(entry.is_expired());

        // Mark access should update timestamp
        entry.mark_accessed();
        assert_eq!(entry.access_count, 2);
    }
    #[test]
    fn test_lru_eviction() {
        let lru = LruEviction::new(3);

        // Add some keys
        lru.record_access("key1");
        lru.record_access("key2");
        lru.record_access("key3");

        // No eviction needed yet
        assert!(lru.get_eviction_candidates(3).is_empty());

        // Add one more key, should trigger eviction
        lru.record_access("key4");
        let candidates = lru.get_eviction_candidates(4);
        assert_eq!(candidates.len(), 1);
        assert_eq!(candidates[0], "key1"); // Least recently used

        // Access key1 again
        lru.record_access("key1");
        let candidates = lru.get_eviction_candidates(4);
        assert_eq!(candidates[0], "key2"); // Now key2 is least recently used
    }
    #[test]
    fn test_cache_basic_operations() {
        let cache = Cache::new(100, Duration::from_secs(60));
        let context = create_test_context();
        let key = CacheKey::from_context(&context);

        // Cache miss initially
        assert!(cache.get(&key).is_none());
        assert_eq!(cache.stats().cache_misses, 1);

        // Put and get
        cache.put(key.clone(), HookResult::Continue, None).unwrap();
        assert!(cache.get(&key).is_some());
        assert_eq!(cache.stats().cache_hits, 1);

        // Remove
        assert!(cache.remove(&key));
        assert!(cache.get(&key).is_none());
        assert_eq!(cache.stats().cache_misses, 2);
    }
    #[test]
    fn test_cache_ttl_expiration() {
        let cache = Cache::new(100, Duration::from_millis(50));
        let context = create_test_context();
        let key = CacheKey::from_context(&context);

        // Put with short TTL
        cache
            .put(
                key.clone(),
                HookResult::Continue,
                Some(Duration::from_millis(100)),
            )
            .unwrap();
        assert!(cache.get(&key).is_some());

        // Wait for expiration
        thread::sleep(StdDuration::from_millis(150));
        assert!(cache.get(&key).is_none());
    }
    #[test]
    fn test_cache_lru_eviction() {
        let cache = Cache::new(2, Duration::from_secs(60)); // Small cache

        let mut context1 = create_test_context();
        let mut context2 = create_test_context();
        let mut context3 = create_test_context();

        // Make contexts unique by adding different data
        context1.insert_data("unique".to_string(), serde_json::json!("data1"));
        context2.insert_data("unique".to_string(), serde_json::json!("data2"));
        context3.insert_data("unique".to_string(), serde_json::json!("data3"));

        let key1 = CacheKey::from_context(&context1);
        let key2 = CacheKey::from_context(&context2);
        let key3 = CacheKey::from_context(&context3);

        // Fill cache
        cache.put(key1.clone(), HookResult::Continue, None).unwrap();
        cache.put(key2.clone(), HookResult::Continue, None).unwrap();
        assert_eq!(cache.size(), 2);

        // Add third entry, should evict first
        cache.put(key3.clone(), HookResult::Continue, None).unwrap();
        assert_eq!(cache.size(), 2);
        assert!(cache.get(&key1).is_none()); // Should be evicted
        assert!(cache.get(&key2).is_some()); // Should still be there
        assert!(cache.get(&key3).is_some()); // Should still be there
    }
    #[test]
    fn test_cache_stats() {
        let cache = Cache::new(100, Duration::from_secs(60));
        let context = create_test_context();
        let key = CacheKey::from_context(&context);

        // Initial stats
        let stats = cache.stats();
        assert_eq!(stats.total_requests, 0);
        assert_eq!(stats.hit_ratio(), 0.0);

        // Cache miss
        cache.get(&key);
        let stats = cache.stats();
        assert_eq!(stats.total_requests, 1);
        assert_eq!(stats.cache_misses, 1);
        assert_eq!(stats.hit_ratio(), 0.0);

        // Put and hit
        cache.put(key.clone(), HookResult::Continue, None).unwrap();
        cache.get(&key);
        let stats = cache.stats();
        assert_eq!(stats.total_requests, 2);
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.cache_misses, 1);
        assert_eq!(stats.hit_ratio(), 0.5);
    }
    #[test]
    fn test_cache_clear() {
        let cache = Cache::new(100, Duration::from_secs(60));
        let context = create_test_context();
        let key = CacheKey::from_context(&context);

        cache.put(key.clone(), HookResult::Continue, None).unwrap();
        assert_eq!(cache.size(), 1);

        cache.clear();
        assert_eq!(cache.size(), 0);
        assert!(cache.is_empty());
        assert!(cache.get(&key).is_none());
    }
    #[test]
    fn test_cache_cleanup() {
        let cache = Cache::new(100, Duration::from_millis(50));
        let context = create_test_context();
        let key = CacheKey::from_context(&context);

        // Put entry with short TTL
        cache
            .put(
                key.clone(),
                HookResult::Continue,
                Some(Duration::from_millis(100)),
            )
            .unwrap();
        assert_eq!(cache.size(), 1);

        // Wait for expiration
        thread::sleep(StdDuration::from_millis(150));

        // Force cleanup
        cache.cleanup();
        assert_eq!(cache.size(), 0);
    }
}
