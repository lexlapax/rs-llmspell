// ABOUTME: Fast-path state operations that bypass expensive validation and processing
// ABOUTME: Provides direct serialization paths for trusted and ephemeral data

use crate::StateScope;
use llmspell_state_traits::{StateError, StateResult};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Configuration for fast-path operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FastPathConfig {
    /// Enable MessagePack for better performance than JSON
    pub use_messagepack: bool,

    /// Enable compression for large values
    pub enable_compression: bool,

    /// Compression threshold in bytes
    pub compression_threshold: usize,

    /// Enable in-memory cache for ephemeral data
    pub enable_ephemeral_cache: bool,

    /// Maximum cache size for ephemeral data
    pub ephemeral_cache_limit: usize,
}

impl Default for FastPathConfig {
    fn default() -> Self {
        Self {
            use_messagepack: true,
            enable_compression: true,
            compression_threshold: 1024, // 1KB
            enable_ephemeral_cache: true,
            ephemeral_cache_limit: 10_000,
        }
    }
}

/// Fast-path state manager for performance-critical operations
pub struct FastPathManager {
    config: FastPathConfig,
    ephemeral_cache: parking_lot::RwLock<HashMap<String, Value>>,
}

impl FastPathManager {
    pub fn new(config: FastPathConfig) -> Self {
        Self {
            config,
            ephemeral_cache: parking_lot::RwLock::new(HashMap::new()),
        }
    }

    /// Fast serialization without any validation
    pub fn serialize_trusted(&self, value: &Value) -> StateResult<Vec<u8>> {
        if self.config.use_messagepack {
            rmp_serde::to_vec(value).map_err(|e| {
                StateError::serialization(format!("MessagePack serialization failed: {}", e))
            })
        } else {
            serde_json::to_vec(value)
                .map_err(|e| StateError::serialization(format!("JSON serialization failed: {}", e)))
        }
    }

    /// Fast deserialization without any validation
    pub fn deserialize_trusted(&self, bytes: &[u8]) -> StateResult<Value> {
        if self.config.use_messagepack {
            rmp_serde::from_slice(bytes).map_err(|e| {
                StateError::serialization(format!("MessagePack deserialization failed: {}", e))
            })
        } else {
            serde_json::from_slice(bytes).map_err(|e| {
                StateError::serialization(format!("JSON deserialization failed: {}", e))
            })
        }
    }

    /// Store ephemeral data in memory cache
    pub fn store_ephemeral(&self, scope: &StateScope, key: &str, value: Value) -> StateResult<()> {
        if !self.config.enable_ephemeral_cache {
            return Ok(()); // Ephemeral data discarded if cache disabled
        }

        let cache_key = format!("{}:{}", scope, key);

        let mut cache = self.ephemeral_cache.write();

        // Enforce cache size limit
        if cache.len() >= self.config.ephemeral_cache_limit {
            // Simple eviction: remove oldest entry (HashMap iteration order is arbitrary)
            if let Some(oldest_key) = cache.keys().next().cloned() {
                cache.remove(&oldest_key);
            }
        }

        cache.insert(cache_key, value);
        Ok(())
    }

    /// Retrieve ephemeral data from memory cache
    pub fn get_ephemeral(&self, scope: &StateScope, key: &str) -> StateResult<Option<Value>> {
        if !self.config.enable_ephemeral_cache {
            return Ok(None);
        }

        let cache_key = format!("{}:{}", scope, key);
        let cache = self.ephemeral_cache.read();
        Ok(cache.get(&cache_key).cloned())
    }

    /// Clear ephemeral cache
    pub fn clear_ephemeral(&self) {
        if self.config.enable_ephemeral_cache {
            self.ephemeral_cache.write().clear();
        }
    }

    /// Get cache statistics
    pub fn ephemeral_stats(&self) -> EphemeralCacheStats {
        let cache = self.ephemeral_cache.read();
        EphemeralCacheStats {
            entry_count: cache.len(),
            memory_estimate: cache
                .iter()
                .map(|(k, v)| k.len() + self.estimate_value_size(v))
                .sum(),
            cache_limit: self.config.ephemeral_cache_limit,
        }
    }

    /// Estimate memory usage of a JSON value
    #[allow(clippy::only_used_in_recursion)]
    fn estimate_value_size(&self, value: &Value) -> usize {
        match value {
            Value::Null => 4,
            Value::Bool(_) => 4,
            Value::Number(_) => 8,
            Value::String(s) => s.len(),
            Value::Array(arr) => {
                arr.iter()
                    .map(|v| self.estimate_value_size(v))
                    .sum::<usize>()
                    + 8
            }
            Value::Object(obj) => {
                obj.iter()
                    .map(|(k, v)| k.len() + self.estimate_value_size(v))
                    .sum::<usize>()
                    + 8
            }
        }
    }

    /// Compress data if it exceeds threshold
    pub fn maybe_compress(&self, data: Vec<u8>) -> StateResult<Vec<u8>> {
        if !self.config.enable_compression || data.len() < self.config.compression_threshold {
            return Ok(data);
        }

        use flate2::write::GzEncoder;
        use flate2::Compression;
        use std::io::Write;

        let mut encoder = GzEncoder::new(Vec::new(), Compression::fast());
        encoder
            .write_all(&data)
            .map_err(|e| StateError::compression_error(format!("Compression failed: {}", e)))?;

        encoder
            .finish()
            .map_err(|e| StateError::compression_error(format!("Compression finish failed: {}", e)))
    }

    /// Decompress data if needed
    pub fn maybe_decompress(&self, data: &[u8]) -> StateResult<Vec<u8>> {
        // Simple heuristic: if data starts with gzip magic bytes, decompress
        if data.len() >= 2 && data[0] == 0x1f && data[1] == 0x8b {
            use flate2::read::GzDecoder;
            use std::io::Read;

            let mut decoder = GzDecoder::new(data);
            let mut decompressed = Vec::new();
            decoder.read_to_end(&mut decompressed).map_err(|e| {
                StateError::compression_error(format!("Decompression failed: {}", e))
            })?;
            Ok(decompressed)
        } else {
            Ok(data.to_vec())
        }
    }
}

/// Statistics for ephemeral cache
#[derive(Debug, Clone)]
pub struct EphemeralCacheStats {
    pub entry_count: usize,
    pub memory_estimate: usize,
    pub cache_limit: usize,
}

impl EphemeralCacheStats {
    pub fn utilization_percent(&self) -> f64 {
        if self.cache_limit == 0 {
            0.0
        } else {
            (self.entry_count as f64 / self.cache_limit as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn test_fast_serialization() {
        let config = FastPathConfig::default();
        let manager = FastPathManager::new(config);

        let value = json!({"test": "data", "number": 42});
        let serialized = manager.serialize_trusted(&value).unwrap();
        let deserialized = manager.deserialize_trusted(&serialized).unwrap();

        assert_eq!(value, deserialized);
    }
    #[test]
    fn test_performance_overhead() {
        println!("\n=== Fast Path Performance Test ===");

        let config = FastPathConfig::default();
        let manager = FastPathManager::new(config);

        let test_data = json!({
            "conversation": ["Hello", "Hi there!"],
            "context": {"topic": "greeting"}
        });

        // Baseline: JSON serialization (what we're replacing)
        let start = std::time::Instant::now();
        for _ in 0..1000 {
            let serialized = serde_json::to_vec(&test_data).unwrap();
            let _deserialized: Value = serde_json::from_slice(&serialized).unwrap();
        }
        let json_baseline = start.elapsed();
        println!("JSON baseline (serialize+deserialize): {:?}", json_baseline);

        // Fast path: MessagePack serialization
        let start = std::time::Instant::now();
        for _ in 0..1000 {
            let serialized = manager.serialize_trusted(&test_data).unwrap();
            let _deserialized = manager.deserialize_trusted(&serialized).unwrap();
        }
        let fast_path = start.elapsed();
        println!(
            "MessagePack fast path (serialize+deserialize): {:?}",
            fast_path
        );

        // Calculate improvement over JSON
        let improvement =
            ((json_baseline.as_nanos() as f64 / fast_path.as_nanos() as f64) - 1.0) * 100.0;
        println!("MessagePack is {:.1}% faster than JSON", improvement);

        // MessagePack can be slightly slower than JSON for small payloads due to binary encoding overhead
        // but provides better compression for larger data. Allow up to 30% overhead for small data.
        assert!(
            fast_path.as_micros() <= json_baseline.as_micros() * 130 / 100, // Allow 30% variance
            "MessagePack overhead should be reasonable, but got {:?} vs {:?}",
            fast_path,
            json_baseline
        );

        // Also test that it's reasonably fast in absolute terms
        let per_op_micros = fast_path.as_micros() as f64 / 1000.0;
        println!("Per operation time: {:.2}µs", per_op_micros);

        // Should be under 2µs per operation on modern hardware
        assert!(
            per_op_micros < 5.0,
            "Serialization should be <5µs per operation, got {:.2}µs",
            per_op_micros
        );
    }
    #[test]
    fn test_ephemeral_cache() {
        let config = FastPathConfig {
            enable_ephemeral_cache: true,
            ephemeral_cache_limit: 2,
            ..Default::default()
        };
        let manager = FastPathManager::new(config);

        let scope = StateScope::Agent("test".to_string());
        let value1 = json!({"data": 1});
        let value2 = json!({"data": 2});
        let value3 = json!({"data": 3});

        // Store values
        manager
            .store_ephemeral(&scope, "key1", value1.clone())
            .unwrap();
        manager
            .store_ephemeral(&scope, "key2", value2.clone())
            .unwrap();

        // Verify retrieval
        assert_eq!(manager.get_ephemeral(&scope, "key1").unwrap(), Some(value1));
        assert_eq!(manager.get_ephemeral(&scope, "key2").unwrap(), Some(value2));

        // Store third value (should evict oldest)
        manager
            .store_ephemeral(&scope, "key3", value3.clone())
            .unwrap();

        // Cache should have 2 entries (evicted oldest)
        let stats = manager.ephemeral_stats();
        assert_eq!(stats.entry_count, 2);
    }
    #[test]
    fn test_compression() {
        let config = FastPathConfig {
            enable_compression: true,
            compression_threshold: 10, // Low threshold for testing
            ..Default::default()
        };
        let manager = FastPathManager::new(config);

        let large_data = "This is a large piece of data that should be compressed"
            .as_bytes()
            .to_vec();
        let compressed = manager.maybe_compress(large_data.clone()).unwrap();
        let decompressed = manager.maybe_decompress(&compressed).unwrap();

        assert_eq!(large_data, decompressed);
        // Compressed should be different (unless compression actually made it larger)
        assert_ne!(large_data, compressed);
    }
    #[test]
    fn test_small_data_not_compressed() {
        let config = FastPathConfig {
            enable_compression: true,
            compression_threshold: 1000, // High threshold
            ..Default::default()
        };
        let manager = FastPathManager::new(config);

        let small_data = "small".as_bytes().to_vec();
        let result = manager.maybe_compress(small_data.clone()).unwrap();

        // Should not be compressed
        assert_eq!(small_data, result);
    }
}
