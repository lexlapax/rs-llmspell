// ABOUTME: TTL (Time To Live) cache implementation with automatic expiration and cleanup
// ABOUTME: Provides time-based cache entry expiration with configurable cleanup intervals

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};

/// TTL cache entry with expiration time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TtlEntry<T> {
    pub value: T,
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub access_count: u64,
    pub last_accessed: DateTime<Utc>,
}

impl<T> TtlEntry<T> {
    /// Create a new TTL entry
    pub fn new(value: T, ttl: Duration) -> Self {
        let now = Utc::now();
        let expires_at = now
            + chrono::Duration::from_std(ttl).unwrap_or_else(|_| chrono::Duration::seconds(300));

        Self {
            value,
            created_at: now,
            expires_at,
            access_count: 1,
            last_accessed: now,
        }
    }

    /// Check if the entry has expired
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expires_at
    }

    /// Get the remaining TTL
    pub fn remaining_ttl(&self) -> Duration {
        let now = Utc::now();
        if now >= self.expires_at {
            Duration::ZERO
        } else {
            (self.expires_at - now).to_std().unwrap_or(Duration::ZERO)
        }
    }

    /// Mark the entry as accessed
    pub fn mark_accessed(&mut self) {
        self.last_accessed = Utc::now();
        self.access_count += 1;
    }

    /// Extend the TTL from now
    pub fn extend_ttl(&mut self, additional_ttl: Duration) {
        let now = Utc::now();
        let additional_duration = chrono::Duration::from_std(additional_ttl)
            .unwrap_or_else(|_| chrono::Duration::seconds(300));
        self.expires_at = now + additional_duration;
    }

    /// Reset TTL from creation time
    pub fn reset_ttl(&mut self, new_ttl: Duration) {
        let ttl_duration =
            chrono::Duration::from_std(new_ttl).unwrap_or_else(|_| chrono::Duration::seconds(300));
        self.expires_at = self.created_at + ttl_duration;
    }
}

/// TTL cache configuration
#[derive(Debug, Clone)]
pub struct TtlCacheConfig {
    /// Default TTL for entries
    pub default_ttl: Duration,
    /// Maximum number of entries
    pub max_entries: usize,
    /// Cleanup interval for expired entries
    pub cleanup_interval: Duration,
    /// Whether to extend TTL on access
    pub extend_on_access: bool,
    /// Additional TTL to add on access (if extend_on_access is true)
    pub access_extension: Duration,
}

impl Default for TtlCacheConfig {
    fn default() -> Self {
        Self {
            default_ttl: Duration::from_secs(300), // 5 minutes
            max_entries: 1000,
            cleanup_interval: Duration::from_secs(60), // 1 minute
            extend_on_access: false,
            access_extension: Duration::from_secs(60), // 1 minute
        }
    }
}

/// TTL cache statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TtlCacheStats {
    pub total_entries: usize,
    pub expired_entries: u64,
    pub evicted_entries: u64,
    pub total_gets: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub total_puts: u64,
    pub cleanup_runs: u64,
    pub last_cleanup: Option<DateTime<Utc>>,
}

impl TtlCacheStats {
    pub fn hit_ratio(&self) -> f64 {
        if self.total_gets == 0 {
            0.0
        } else {
            self.cache_hits as f64 / self.total_gets as f64
        }
    }

    pub fn miss_ratio(&self) -> f64 {
        1.0 - self.hit_ratio()
    }
}

/// Thread-safe TTL-based cache
#[derive(Debug)]
pub struct TtlCache<K, V>
where
    K: Clone + Eq + std::hash::Hash,
    V: Clone,
{
    /// Cache storage
    entries: Arc<RwLock<HashMap<K, TtlEntry<V>>>>,
    /// Cache configuration
    config: TtlCacheConfig,
    /// Cache statistics
    stats: Arc<RwLock<TtlCacheStats>>,
    /// Last cleanup time
    last_cleanup: Arc<RwLock<Instant>>,
}

impl<K, V> TtlCache<K, V>
where
    K: Clone + Eq + std::hash::Hash,
    V: Clone,
{
    /// Create a new TTL cache with default configuration
    pub fn new() -> Self {
        Self::with_config(TtlCacheConfig::default())
    }

    /// Create a new TTL cache with custom configuration
    pub fn with_config(config: TtlCacheConfig) -> Self {
        Self {
            entries: Arc::new(RwLock::new(HashMap::new())),
            config,
            stats: Arc::new(RwLock::new(TtlCacheStats::default())),
            last_cleanup: Arc::new(RwLock::new(Instant::now())),
        }
    }

    /// Get a value from the cache
    pub fn get(&self, key: &K) -> Option<V> {
        let mut stats = self.stats.write().unwrap();
        stats.total_gets += 1;
        drop(stats);

        let mut entries = self.entries.write().unwrap();

        if let Some(entry) = entries.get_mut(key) {
            // Check if expired
            if entry.is_expired() {
                entries.remove(key);

                let mut stats = self.stats.write().unwrap();
                stats.cache_misses += 1;
                stats.expired_entries += 1;
                stats.total_entries = entries.len();

                return None;
            }

            // Mark as accessed
            entry.mark_accessed();

            // Extend TTL if configured
            if self.config.extend_on_access {
                entry.extend_ttl(self.config.access_extension);
            }

            let mut stats = self.stats.write().unwrap();
            stats.cache_hits += 1;

            Some(entry.value.clone())
        } else {
            let mut stats = self.stats.write().unwrap();
            stats.cache_misses += 1;
            None
        }
    }

    /// Put a value into the cache with default TTL
    pub fn put(&self, key: K, value: V) -> bool {
        self.put_with_ttl(key, value, self.config.default_ttl)
    }

    /// Put a value into the cache with custom TTL
    pub fn put_with_ttl(&self, key: K, value: V, ttl: Duration) -> bool {
        let mut entries = self.entries.write().unwrap();

        // Check if we need to make space
        if entries.len() >= self.config.max_entries && !entries.contains_key(&key) {
            // Try cleanup first
            self.cleanup_expired_entries_internal(&mut entries);

            // If still over limit, we can't add
            if entries.len() >= self.config.max_entries {
                let mut stats = self.stats.write().unwrap();
                stats.evicted_entries += 1;
                return false;
            }
        }

        let entry = TtlEntry::new(value, ttl);
        entries.insert(key, entry);

        let mut stats = self.stats.write().unwrap();
        stats.total_puts += 1;
        stats.total_entries = entries.len();

        // Periodic cleanup
        drop(entries);
        self.cleanup_if_needed();

        true
    }

    /// Remove a key from the cache
    pub fn remove(&self, key: &K) -> Option<V> {
        let mut entries = self.entries.write().unwrap();

        if let Some(entry) = entries.remove(key) {
            let mut stats = self.stats.write().unwrap();
            stats.total_entries = entries.len();
            Some(entry.value)
        } else {
            None
        }
    }

    /// Check if the cache contains a key (and it's not expired)
    pub fn contains_key(&self, key: &K) -> bool {
        let mut entries = self.entries.write().unwrap();

        if let Some(entry) = entries.get(key) {
            if entry.is_expired() {
                entries.remove(key);
                let mut stats = self.stats.write().unwrap();
                stats.expired_entries += 1;
                stats.total_entries = entries.len();
                false
            } else {
                true
            }
        } else {
            false
        }
    }

    /// Get the current size of the cache
    pub fn len(&self) -> usize {
        self.entries.read().unwrap().len()
    }

    /// Check if the cache is empty
    pub fn is_empty(&self) -> bool {
        self.entries.read().unwrap().is_empty()
    }

    /// Clear all entries from the cache
    pub fn clear(&self) {
        let mut entries = self.entries.write().unwrap();
        entries.clear();

        let mut stats = self.stats.write().unwrap();
        stats.total_entries = 0;
    }

    /// Get cache statistics
    pub fn stats(&self) -> TtlCacheStats {
        self.stats.read().unwrap().clone()
    }

    /// Force cleanup of expired entries
    pub fn cleanup_expired(&self) -> u64 {
        let mut entries = self.entries.write().unwrap();
        self.cleanup_expired_entries_internal(&mut entries)
    }

    /// Get all non-expired keys
    pub fn keys(&self) -> Vec<K> {
        let mut entries = self.entries.write().unwrap();
        let mut keys = Vec::new();
        let mut expired_keys = Vec::new();

        for (key, entry) in entries.iter() {
            if entry.is_expired() {
                expired_keys.push(key.clone());
            } else {
                keys.push(key.clone());
            }
        }

        // Remove expired entries
        for key in &expired_keys {
            entries.remove(key);
        }

        if !expired_keys.is_empty() {
            let mut stats = self.stats.write().unwrap();
            stats.expired_entries += expired_keys.len() as u64;
            stats.total_entries = entries.len();
        }

        keys
    }

    /// Cleanup expired entries if cleanup interval has passed
    fn cleanup_if_needed(&self) {
        let now = Instant::now();
        let should_cleanup = {
            let last_cleanup = self.last_cleanup.read().unwrap();
            now.duration_since(*last_cleanup) >= self.config.cleanup_interval
        };

        if should_cleanup {
            let mut entries = self.entries.write().unwrap();
            self.cleanup_expired_entries_internal(&mut entries);
            *self.last_cleanup.write().unwrap() = now;
        }
    }

    /// Internal cleanup implementation
    fn cleanup_expired_entries_internal(&self, entries: &mut HashMap<K, TtlEntry<V>>) -> u64 {
        let mut expired_keys = Vec::new();

        for (key, entry) in entries.iter() {
            if entry.is_expired() {
                expired_keys.push(key.clone());
            }
        }

        let expired_count = expired_keys.len() as u64;

        for key in expired_keys {
            entries.remove(&key);
        }

        if expired_count > 0 {
            let mut stats = self.stats.write().unwrap();
            stats.expired_entries += expired_count;
            stats.cleanup_runs += 1;
            stats.last_cleanup = Some(Utc::now());
            stats.total_entries = entries.len();
        }

        expired_count
    }
}

impl<K, V> Default for TtlCache<K, V>
where
    K: Clone + Eq + std::hash::Hash,
    V: Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<K, V> Clone for TtlCache<K, V>
where
    K: Clone + Eq + std::hash::Hash,
    V: Clone,
{
    fn clone(&self) -> Self {
        let entries = self.entries.read().unwrap().clone();
        let new_cache = Self::with_config(self.config.clone());
        *new_cache.entries.write().unwrap() = entries;
        *new_cache.stats.write().unwrap() = self.stats.read().unwrap().clone();
        new_cache
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration as StdDuration;

    #[test]
    fn test_ttl_entry_creation() {
        let entry = TtlEntry::new("test_value", Duration::from_secs(60));

        assert_eq!(entry.value, "test_value");
        assert_eq!(entry.access_count, 1);
        assert!(!entry.is_expired());
        assert!(entry.remaining_ttl() > Duration::from_secs(50));
    }

    #[test]
    fn test_ttl_entry_expiration() {
        let mut entry = TtlEntry::new("test_value", Duration::from_millis(100));

        assert!(!entry.is_expired());

        thread::sleep(StdDuration::from_millis(150));
        assert!(entry.is_expired());
        assert_eq!(entry.remaining_ttl(), Duration::ZERO);

        // Test access tracking
        entry.mark_accessed();
        assert_eq!(entry.access_count, 2);
    }

    #[test]
    fn test_ttl_entry_extension() {
        let mut entry = TtlEntry::new("test_value", Duration::from_millis(100));

        thread::sleep(StdDuration::from_millis(50));
        entry.extend_ttl(Duration::from_secs(1));

        assert!(!entry.is_expired());
        assert!(entry.remaining_ttl() > Duration::from_millis(500));
    }

    #[test]
    fn test_ttl_cache_basic_operations() {
        let cache = TtlCache::new();

        // Test put and get
        assert!(cache.put("key1", "value1"));
        assert_eq!(cache.get(&"key1"), Some("value1"));
        assert_eq!(cache.len(), 1);

        // Test miss
        assert_eq!(cache.get(&"nonexistent"), None);

        // Test contains
        assert!(cache.contains_key(&"key1"));
        assert!(!cache.contains_key(&"nonexistent"));

        // Test remove
        assert_eq!(cache.remove(&"key1"), Some("value1"));
        assert_eq!(cache.get(&"key1"), None);
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_ttl_cache_expiration() {
        let config = TtlCacheConfig {
            default_ttl: Duration::from_millis(100),
            ..Default::default()
        };
        let cache = TtlCache::with_config(config);

        cache.put("key1", "value1");
        assert_eq!(cache.get(&"key1"), Some("value1"));

        thread::sleep(StdDuration::from_millis(150));
        assert_eq!(cache.get(&"key1"), None);

        let stats = cache.stats();
        assert_eq!(stats.expired_entries, 1);
    }

    #[test]
    fn test_ttl_cache_custom_ttl() {
        let cache = TtlCache::new();

        cache.put_with_ttl("short", "value", Duration::from_millis(50));
        cache.put_with_ttl("long", "value", Duration::from_secs(60));

        thread::sleep(StdDuration::from_millis(100));

        assert_eq!(cache.get(&"short"), None);
        assert_eq!(cache.get(&"long"), Some("value"));
    }

    #[test]
    fn test_ttl_cache_extend_on_access() {
        let config = TtlCacheConfig {
            default_ttl: Duration::from_millis(100),
            extend_on_access: true,
            access_extension: Duration::from_millis(100),
            ..Default::default()
        };
        let cache = TtlCache::with_config(config);

        cache.put("key1", "value1");

        // Access after 50ms
        thread::sleep(StdDuration::from_millis(50));
        assert_eq!(cache.get(&"key1"), Some("value1"));

        // Should still be valid after original TTL due to extension
        thread::sleep(StdDuration::from_millis(75));
        assert_eq!(cache.get(&"key1"), Some("value1"));
    }

    #[test]
    fn test_ttl_cache_max_entries() {
        let config = TtlCacheConfig {
            max_entries: 2,
            ..Default::default()
        };
        let cache = TtlCache::with_config(config);

        assert!(cache.put("key1", "value1"));
        assert!(cache.put("key2", "value2"));
        assert!(!cache.put("key3", "value3")); // Should fail due to limit

        assert_eq!(cache.len(), 2);
        assert!(cache.contains_key(&"key1"));
        assert!(cache.contains_key(&"key2"));
        assert!(!cache.contains_key(&"key3"));
    }

    #[test]
    fn test_ttl_cache_cleanup() {
        let config = TtlCacheConfig {
            default_ttl: Duration::from_millis(50),
            cleanup_interval: Duration::from_millis(10),
            ..Default::default()
        };
        let cache = TtlCache::with_config(config);

        cache.put("key1", "value1");
        cache.put("key2", "value2");
        assert_eq!(cache.len(), 2);

        thread::sleep(StdDuration::from_millis(100));

        let expired_count = cache.cleanup_expired();
        assert_eq!(expired_count, 2);
        assert_eq!(cache.len(), 0);
    }

    #[test]
    fn test_ttl_cache_keys() {
        let cache = TtlCache::new();

        cache.put("key1", "value1");
        cache.put("key2", "value2");

        let keys = cache.keys();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"key1"));
        assert!(keys.contains(&"key2"));
    }

    #[test]
    fn test_ttl_cache_clear() {
        let cache = TtlCache::new();

        cache.put("key1", "value1");
        cache.put("key2", "value2");
        assert_eq!(cache.len(), 2);

        cache.clear();
        assert_eq!(cache.len(), 0);
        assert!(cache.is_empty());
    }

    #[test]
    fn test_ttl_cache_stats() {
        let cache = TtlCache::new();

        // Test initial stats
        let stats = cache.stats();
        assert_eq!(stats.total_gets, 0);
        assert_eq!(stats.hit_ratio(), 0.0);

        // Test operations
        cache.put("key1", "value1");
        cache.get(&"key1"); // hit
        cache.get(&"nonexistent"); // miss

        let stats = cache.stats();
        assert_eq!(stats.total_gets, 2);
        assert_eq!(stats.cache_hits, 1);
        assert_eq!(stats.cache_misses, 1);
        assert_eq!(stats.hit_ratio(), 0.5);
        assert_eq!(stats.total_puts, 1);
    }

    #[test]
    fn test_ttl_cache_clone() {
        let cache1 = TtlCache::new();
        cache1.put("key1", "value1");

        let cache2 = cache1.clone();
        assert_eq!(cache2.get(&"key1"), Some("value1"));

        // They should be independent
        cache1.put("key2", "value2");
        assert!(!cache2.contains_key(&"key2"));
    }
}
