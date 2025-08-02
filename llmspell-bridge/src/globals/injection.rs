//! ABOUTME: High-performance global injection system with caching
//! ABOUTME: Handles batch injection of globals into script engines

use super::registry::GlobalRegistry;
use super::types::{GlobalContext, InjectionMetrics};
use llmspell_core::{LLMSpellError, Result};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;

#[cfg(feature = "lua")]
use mlua::Lua;

#[cfg(feature = "javascript")]
use boa_engine::Context;

/// Cache for compiled global injections
pub struct InjectionCache {
    /// Cached Lua injection functions
    #[cfg(feature = "lua")]
    lua_cache: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    /// Cache hit/miss statistics
    hits: Arc<RwLock<u64>>,
    misses: Arc<RwLock<u64>>,
}

impl InjectionCache {
    /// Create a new injection cache
    #[must_use]
    pub fn new() -> Self {
        Self {
            #[cfg(feature = "lua")]
            lua_cache: Arc::new(RwLock::new(HashMap::new())),
            hits: Arc::new(RwLock::new(0)),
            misses: Arc::new(RwLock::new(0)),
        }
    }

    /// Get cache hit rate
    #[must_use]
    pub fn hit_rate(&self) -> f64 {
        let hits = *self.hits.read();
        let misses = *self.misses.read();
        let total = hits + misses;
        if total == 0 {
            0.0
        } else {
            #[allow(clippy::cast_precision_loss)]
            let rate = hits as f64 / total as f64;
            rate
        }
    }

    /// Clear the cache
    pub fn clear(&self) {
        #[cfg(feature = "lua")]
        self.lua_cache.write().clear();
        *self.hits.write() = 0;
        *self.misses.write() = 0;
    }
}

impl Default for InjectionCache {
    fn default() -> Self {
        Self::new()
    }
}

/// High-performance global injector
pub struct GlobalInjector {
    registry: Arc<GlobalRegistry>,
    cache: Arc<InjectionCache>,
}

impl GlobalInjector {
    /// Create a new global injector
    #[must_use]
    pub fn new(registry: Arc<GlobalRegistry>) -> Self {
        Self {
            registry,
            cache: Arc::new(InjectionCache::new()),
        }
    }

    /// Inject all globals into a Lua environment
    #[cfg(feature = "lua")]
    pub fn inject_lua(&self, lua: &Lua, context: &GlobalContext) -> Result<InjectionMetrics> {
        let start = Instant::now();
        let mut metrics = InjectionMetrics::default();
        let globals = self.registry.get_all_ordered();

        // Initialize all globals first
        for global in &globals {
            global.initialize(context)?;
        }

        // Batch inject all globals
        for global in &globals {
            let global_start = Instant::now();
            let metadata = global.metadata();

            // Try to inject the global
            global
                .inject_lua(lua, context)
                .map_err(|e| LLMSpellError::Component {
                    message: format!("Failed to inject global '{}': {}", metadata.name, e),
                    source: Some(Box::new(e)),
                })?;

            let elapsed = u64::try_from(global_start.elapsed().as_micros()).unwrap_or(u64::MAX);
            metrics
                .per_global_times
                .insert(metadata.name.clone(), elapsed);
        }

        metrics.total_injection_time_us =
            u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX);
        metrics.globals_injected = globals.len();
        metrics.cache_hit_rate = self.cache.hit_rate();

        Ok(metrics)
    }

    /// Inject all globals into a JavaScript environment
    #[cfg(feature = "javascript")]
    pub fn inject_javascript(
        &self,
        ctx: &mut Context,
        context: &GlobalContext,
    ) -> Result<InjectionMetrics> {
        let start = Instant::now();
        let mut metrics = InjectionMetrics::default();
        let globals = self.registry.get_all_ordered();

        // Initialize all globals first
        for global in &globals {
            global.initialize(context)?;
        }

        // Batch inject all globals
        for global in &globals {
            let global_start = Instant::now();
            let metadata = global.metadata();

            // Try to inject the global
            global
                .inject_javascript(ctx, context)
                .map_err(|e| LLMSpellError::Component {
                    message: format!("Failed to inject global '{}': {}", metadata.name, e),
                    source: Some(Box::new(e)),
                })?;

            let elapsed = u64::try_from(global_start.elapsed().as_micros()).unwrap_or(u64::MAX);
            metrics
                .per_global_times
                .insert(metadata.name.clone(), elapsed);
        }

        metrics.total_injection_time_us =
            u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX);
        metrics.globals_injected = globals.len();
        metrics.cache_hit_rate = self.cache.hit_rate();

        Ok(metrics)
    }

    /// Clean up all globals
    pub fn cleanup(&self) -> Result<()> {
        let globals = self.registry.get_all_ordered();

        // Clean up in reverse order
        for global in globals.iter().rev() {
            global.cleanup()?;
        }

        self.cache.clear();
        Ok(())
    }

    /// Get injection metrics from the last injection
    #[must_use]
    pub fn get_metrics(&self) -> &InjectionMetrics {
        self.registry.metrics()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_injection_cache() {
        let cache = InjectionCache::new();

        // Initially, hit rate should be 0
        assert_eq!(cache.hit_rate(), 0.0);

        // Simulate some hits and misses
        *cache.hits.write() = 7;
        *cache.misses.write() = 3;

        // Hit rate should be 0.7
        assert_eq!(cache.hit_rate(), 0.7);

        // Clear should reset everything
        cache.clear();
        assert_eq!(cache.hit_rate(), 0.0);
    }
    #[test]
    fn test_injection_metrics() {
        let metrics = InjectionMetrics {
            total_injection_time_us: 3000, // 3ms
            globals_injected: 10,
            ..Default::default()
        };

        assert!(metrics.is_within_bounds());
        assert_eq!(metrics.average_time_us(), 300);
    }
    #[test]
    fn test_injection_metrics_failure_case() {
        // Test failure case
        let metrics = InjectionMetrics {
            total_injection_time_us: 6000, // 6ms
            globals_injected: 10,
            ..Default::default()
        };
        assert!(!metrics.is_within_bounds());
    }
}
