// ABOUTME: CachingHook implementation for automatic result caching with TTL and LRU eviction
// ABOUTME: Provides configurable caching of hook results to improve performance and reduce redundant operations

use crate::cache::{Cache, CacheKey, CacheStats};
use crate::context::HookContext;
use crate::result::HookResult;
use crate::traits::{Hook, MetricHook, ReplayableHook};
use crate::types::{HookMetadata, HookPoint, Language, Priority};
use anyhow::Result;
use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Duration;

/// Caching strategy configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum CachingStrategy {
    /// Cache all hook results
    CacheAll,
    /// Only cache successful results (Continue, Modified)
    #[default]
    CacheSuccessOnly,
    /// Only cache specific result types
    CacheSpecificTypes(HashSet<String>),
    /// Custom caching logic based on context
    Custom,
}

/// Configuration for the caching hook
#[derive(Debug, Clone)]
pub struct CachingConfig {
    /// Maximum number of cached entries
    pub max_entries: usize,
    /// Default TTL for cached entries
    pub default_ttl: Duration,
    /// Caching strategy
    pub strategy: CachingStrategy,
    /// Hook points to cache (empty means cache all)
    pub cacheable_hook_points: HashSet<HookPoint>,
    /// Hook points to never cache
    pub non_cacheable_hook_points: HashSet<HookPoint>,
    /// Whether to cache based on full context or just hook point
    pub use_full_context: bool,
    /// Minimum execution time to cache (avoid caching very fast operations)
    pub min_execution_time: Duration,
    /// Whether to update TTL on cache hit
    pub extend_ttl_on_hit: bool,
    /// TTL extension amount on hit
    pub ttl_extension: Duration,
    /// Whether to cache errors
    pub cache_errors: bool,
}

impl Default for CachingConfig {
    fn default() -> Self {
        let mut non_cacheable = HashSet::new();
        // Don't cache security-related hooks by default
        non_cacheable.insert(HookPoint::SecurityViolation);
        non_cacheable.insert(HookPoint::BeforeStateWrite);
        non_cacheable.insert(HookPoint::AfterStateWrite);

        Self {
            max_entries: 1000,
            default_ttl: Duration::from_secs(300), // 5 minutes
            strategy: CachingStrategy::default(),
            cacheable_hook_points: HashSet::new(), // Empty means cache all
            non_cacheable_hook_points: non_cacheable,
            use_full_context: true,
            min_execution_time: Duration::from_millis(10), // Only cache operations > 10ms
            extend_ttl_on_hit: false,
            ttl_extension: Duration::from_secs(60),
            cache_errors: false,
        }
    }
}

/// Caching hook performance metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CachingMetrics {
    pub total_cache_attempts: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub cache_puts: u64,
    pub cache_evictions: u64,
    pub cache_errors: u64,
    pub time_saved_ms: u64,
    pub average_execution_time_ms: f64,
    pub average_cached_time_ms: f64,
}

impl CachingMetrics {
    pub fn hit_ratio(&self) -> f64 {
        if self.total_cache_attempts == 0 {
            0.0
        } else {
            self.cache_hits as f64 / self.total_cache_attempts as f64
        }
    }

    pub fn cache_efficiency(&self) -> f64 {
        if self.cache_hits == 0 {
            0.0
        } else {
            self.time_saved_ms as f64 / self.cache_hits as f64
        }
    }
}

/// Built-in caching hook for automatic result caching
pub struct CachingHook {
    cache: Arc<Cache>,
    config: CachingConfig,
    metrics: Arc<std::sync::RwLock<CachingMetrics>>,
    metadata: HookMetadata,
}

impl CachingHook {
    /// Create a new caching hook with default configuration
    pub fn new() -> Self {
        let config = CachingConfig::default();
        Self {
            cache: Arc::new(Cache::new(config.max_entries, config.default_ttl)),
            config,
            metrics: Arc::new(std::sync::RwLock::new(CachingMetrics::default())),
            metadata: HookMetadata {
                name: "CachingHook".to_string(),
                description: Some("Built-in hook for automatic result caching".to_string()),
                priority: Priority::HIGH, // Run early to check cache
                language: Language::Native,
                tags: vec![
                    "builtin".to_string(),
                    "caching".to_string(),
                    "performance".to_string(),
                ],
                version: "1.0.0".to_string(),
            },
        }
    }

    /// Create a new caching hook with custom configuration
    pub fn with_config(config: CachingConfig) -> Self {
        Self {
            cache: Arc::new(Cache::new(config.max_entries, config.default_ttl)),
            config,
            metrics: Arc::new(std::sync::RwLock::new(CachingMetrics::default())),
            metadata: HookMetadata {
                name: "CachingHook".to_string(),
                description: Some("Built-in hook for automatic result caching".to_string()),
                priority: Priority::HIGH,
                language: Language::Native,
                tags: vec![
                    "builtin".to_string(),
                    "caching".to_string(),
                    "performance".to_string(),
                ],
                version: "1.0.0".to_string(),
            },
        }
    }

    /// Configure caching strategy
    pub fn with_strategy(mut self, strategy: CachingStrategy) -> Self {
        self.config.strategy = strategy;
        self
    }

    /// Set TTL for cached entries
    pub fn with_ttl(mut self, ttl: Duration) -> Self {
        self.config.default_ttl = ttl;
        self
    }

    /// Set maximum cache size
    pub fn with_max_entries(mut self, max_entries: usize) -> Self {
        self.config.max_entries = max_entries;
        self
    }

    /// Enable or disable error caching
    pub fn with_error_caching(mut self, cache_errors: bool) -> Self {
        self.config.cache_errors = cache_errors;
        self
    }

    /// Set minimum execution time for caching
    pub fn with_min_execution_time(mut self, min_time: Duration) -> Self {
        self.config.min_execution_time = min_time;
        self
    }

    /// Get the underlying cache
    pub fn cache(&self) -> Arc<Cache> {
        self.cache.clone()
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> CacheStats {
        self.cache.stats()
    }

    /// Get caching metrics
    pub fn metrics(&self) -> CachingMetrics {
        self.metrics.read().unwrap().clone()
    }

    /// Clear the cache
    pub fn clear_cache(&self) {
        self.cache.clear();
    }

    /// Check if a hook point should be cached
    fn should_cache_hook_point(&self, hook_point: &HookPoint) -> bool {
        // Check non-cacheable list first
        if self.config.non_cacheable_hook_points.contains(hook_point) {
            return false;
        }

        // If specific cacheable points are configured, check them
        if !self.config.cacheable_hook_points.is_empty() {
            return self.config.cacheable_hook_points.contains(hook_point);
        }

        // Default: cache all except non-cacheable
        true
    }

    /// Check if a result should be cached based on strategy
    fn should_cache_result(&self, result: &HookResult) -> bool {
        match &self.config.strategy {
            CachingStrategy::CacheAll => true,
            CachingStrategy::CacheSuccessOnly => {
                matches!(result, HookResult::Continue | HookResult::Modified(_))
            }
            CachingStrategy::CacheSpecificTypes(types) => {
                let result_type = format!("{:?}", std::mem::discriminant(result));
                types.contains(&result_type)
            }
            CachingStrategy::Custom => {
                // For custom strategy, we default to success-only
                // Users can override this by extending the hook
                matches!(result, HookResult::Continue | HookResult::Modified(_))
            }
        }
    }

    /// Check if we should cache based on execution time
    fn should_cache_by_execution_time(&self, execution_time: Duration) -> bool {
        execution_time >= self.config.min_execution_time
    }

    /// Try to get a cached result
    fn try_get_cached(&self, context: &HookContext) -> Option<HookResult> {
        if !self.should_cache_hook_point(&context.point) {
            return None;
        }

        let cache_key = if self.config.use_full_context {
            CacheKey::from_context(context)
        } else {
            // Simple key based on hook point and component
            CacheKey {
                hook_point: format!("{:?}", context.point),
                component_id: format!(
                    "{:?}:{}",
                    context.component_id.component_type, context.component_id.name
                ),
                context_hash: 0, // Ignore context data
                language: format!("{:?}", context.language),
            }
        };

        let mut metrics = self.metrics.write().unwrap();
        metrics.total_cache_attempts += 1;

        if let Some(cached_result) = self.cache.get(&cache_key) {
            metrics.cache_hits += 1;

            // Update cache statistics
            let cache_stats = self.cache.stats();
            metrics.cache_evictions = cache_stats.evictions;

            Some(cached_result)
        } else {
            metrics.cache_misses += 1;
            None
        }
    }

    /// Try to cache a result
    fn try_cache_result(
        &self,
        context: &HookContext,
        result: &HookResult,
        execution_time: Duration,
    ) -> Result<()> {
        if !self.should_cache_hook_point(&context.point) {
            return Ok(());
        }

        if !self.should_cache_result(result) {
            return Ok(());
        }

        if !self.should_cache_by_execution_time(execution_time) {
            return Ok(());
        }

        // Don't cache errors unless explicitly configured
        if result.is_cancelled() && !self.config.cache_errors {
            return Ok(());
        }

        let cache_key = if self.config.use_full_context {
            CacheKey::from_context(context)
        } else {
            CacheKey {
                hook_point: format!("{:?}", context.point),
                component_id: format!(
                    "{:?}:{}",
                    context.component_id.component_type, context.component_id.name
                ),
                context_hash: 0,
                language: format!("{:?}", context.language),
            }
        };

        match self
            .cache
            .put(cache_key, result.clone(), Some(self.config.default_ttl))
        {
            Ok(()) => {
                let mut metrics = self.metrics.write().unwrap();
                metrics.cache_puts += 1;
                metrics.time_saved_ms += execution_time.as_millis() as u64;

                // Update average execution times
                let total_ops = metrics.cache_puts + metrics.cache_hits;
                if total_ops > 0 {
                    let current_avg = metrics.average_execution_time_ms;
                    let new_time = execution_time.as_millis() as f64;
                    metrics.average_execution_time_ms =
                        (current_avg * (total_ops - 1) as f64 + new_time) / total_ops as f64;
                }
            }
            Err(e) => {
                let mut metrics = self.metrics.write().unwrap();
                metrics.cache_errors += 1;
                log::warn!("Failed to cache result: {}", e);
            }
        }

        Ok(())
    }
}

impl Default for CachingHook {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Hook for CachingHook {
    async fn execute(&self, context: &mut HookContext) -> Result<HookResult> {
        let _start_time = std::time::Instant::now();

        // Try to get cached result
        if let Some(cached_result) = self.try_get_cached(context) {
            // Add cache metadata to context
            context.insert_metadata("cache_hit".to_string(), "true".to_string());
            context.insert_metadata("cached_at".to_string(), Utc::now().to_rfc3339());
            context.insert_metadata(
                "caching_hook_version".to_string(),
                self.metadata.version.clone(),
            );

            return Ok(cached_result);
        }

        // No cached result, continue execution
        context.insert_metadata("cache_hit".to_string(), "false".to_string());
        context.insert_metadata("cache_checked_at".to_string(), Utc::now().to_rfc3339());
        context.insert_metadata(
            "caching_hook_version".to_string(),
            self.metadata.version.clone(),
        );

        Ok(HookResult::Continue)
    }

    fn metadata(&self) -> HookMetadata {
        self.metadata.clone()
    }

    fn should_execute(&self, context: &HookContext) -> bool {
        // Always execute to check cache
        self.should_cache_hook_point(&context.point)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[async_trait]
impl MetricHook for CachingHook {
    async fn record_pre_execution(&self, context: &HookContext) -> Result<()> {
        // Pre-execution happens in the main execute method
        // This is called by the hook executor for the actual operation being cached
        if self.should_cache_hook_point(&context.point) {
            log::trace!(
                "CachingHook: Checking cache for hook point {:?}",
                context.point
            );
        }
        Ok(())
    }

    async fn record_post_execution(
        &self,
        context: &HookContext,
        result: &HookResult,
        duration: Duration,
    ) -> Result<()> {
        // This is where we cache the result after the actual operation
        self.try_cache_result(context, result, duration)?;

        log::trace!(
            "CachingHook: Post-execution for hook point {:?}, duration: {:?}, cached: {}",
            context.point,
            duration,
            self.should_cache_result(result) && self.should_cache_by_execution_time(duration)
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{ComponentId, ComponentType, HookPoint};
    use serde_json::json;

    fn create_test_context() -> HookContext {
        let component_id = ComponentId::new(ComponentType::System, "test".to_string());
        HookContext::new(HookPoint::SystemStartup, component_id)
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_caching_hook_basic() {
        let hook = CachingHook::new();
        let mut context = create_test_context();

        let result = hook.execute(&mut context).await.unwrap();
        assert!(matches!(result, HookResult::Continue));

        // Check that cache metadata was added
        assert_eq!(context.get_metadata("cache_hit"), Some("false"));
        assert!(context.get_metadata("cache_checked_at").is_some());
        assert!(context.get_metadata("caching_hook_version").is_some());
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_caching_hook_cache_hit() {
        let hook = CachingHook::new();
        let context = create_test_context();

        // Simulate caching a result
        let cache_key = CacheKey::from_context(&context);
        hook.cache
            .put(cache_key, HookResult::Continue, None)
            .unwrap();

        let mut context_copy = context;
        let result = hook.execute(&mut context_copy).await.unwrap();
        assert!(matches!(result, HookResult::Continue));

        // Check that it was a cache hit
        assert_eq!(context_copy.get_metadata("cache_hit"), Some("true"));
        assert!(context_copy.get_metadata("cached_at").is_some());

        // Check metrics
        let metrics = hook.metrics();
        assert_eq!(metrics.cache_hits, 1);
        assert_eq!(metrics.total_cache_attempts, 1);
        assert_eq!(metrics.hit_ratio(), 1.0);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_caching_config_defaults() {
        let config = CachingConfig::default();
        assert_eq!(config.max_entries, 1000);
        assert_eq!(config.default_ttl, Duration::from_secs(300));
        assert!(matches!(config.strategy, CachingStrategy::CacheSuccessOnly));
        assert!(config.cacheable_hook_points.is_empty());
        assert!(config
            .non_cacheable_hook_points
            .contains(&HookPoint::SecurityViolation));
        assert!(config.use_full_context);
        assert!(!config.cache_errors);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_caching_strategy_success_only() {
        let hook = CachingHook::new().with_strategy(CachingStrategy::CacheSuccessOnly);

        assert!(hook.should_cache_result(&HookResult::Continue));
        assert!(hook.should_cache_result(&HookResult::Modified(json!({"key": "value"}))));
        assert!(!hook.should_cache_result(&HookResult::Cancel("error".to_string())));
        assert!(!hook.should_cache_result(&HookResult::Redirect("/path".to_string())));
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_caching_strategy_cache_all() {
        let hook = CachingHook::new().with_strategy(CachingStrategy::CacheAll);

        assert!(hook.should_cache_result(&HookResult::Continue));
        assert!(hook.should_cache_result(&HookResult::Cancel("error".to_string())));
        assert!(hook.should_cache_result(&HookResult::Redirect("/path".to_string())));
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_should_cache_hook_point() {
        let mut config = CachingConfig::default();
        config
            .non_cacheable_hook_points
            .insert(HookPoint::BeforeAgentInit);
        let hook = CachingHook::with_config(config);

        assert!(hook.should_cache_hook_point(&HookPoint::SystemStartup));
        assert!(!hook.should_cache_hook_point(&HookPoint::SecurityViolation)); // Default non-cacheable
        assert!(!hook.should_cache_hook_point(&HookPoint::BeforeAgentInit)); // Custom non-cacheable
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_should_cache_hook_point_with_whitelist() {
        let mut config = CachingConfig::default();
        config
            .cacheable_hook_points
            .insert(HookPoint::SystemStartup);
        config
            .cacheable_hook_points
            .insert(HookPoint::BeforeAgentInit);
        let hook = CachingHook::with_config(config);

        assert!(hook.should_cache_hook_point(&HookPoint::SystemStartup));
        assert!(hook.should_cache_hook_point(&HookPoint::BeforeAgentInit));
        assert!(!hook.should_cache_hook_point(&HookPoint::BeforeToolExecution));
        // Not in whitelist
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_should_cache_by_execution_time() {
        let hook = CachingHook::new().with_min_execution_time(Duration::from_millis(50));

        assert!(!hook.should_cache_by_execution_time(Duration::from_millis(25)));
        assert!(hook.should_cache_by_execution_time(Duration::from_millis(75)));
        assert!(hook.should_cache_by_execution_time(Duration::from_millis(50)));
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_try_cache_result() {
        let hook = CachingHook::new();
        let context = create_test_context();

        // Cache a successful result
        hook.try_cache_result(&context, &HookResult::Continue, Duration::from_millis(100))
            .unwrap();

        // Check that it was cached
        let cache_key = CacheKey::from_context(&context);
        let cached = hook.cache.get(&cache_key);
        assert!(cached.is_some());
        assert!(matches!(cached.unwrap(), HookResult::Continue));

        // Check metrics
        let metrics = hook.metrics();
        assert_eq!(metrics.cache_puts, 1);
        assert_eq!(metrics.time_saved_ms, 100);
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_cache_error_handling() {
        let hook = CachingHook::new().with_error_caching(false);
        let context = create_test_context();

        // Try to cache an error (should be ignored)
        hook.try_cache_result(
            &context,
            &HookResult::Cancel("error".to_string()),
            Duration::from_millis(100),
        )
        .unwrap();

        // Check that it was not cached
        let cache_key = CacheKey::from_context(&context);
        let cached = hook.cache.get(&cache_key);
        assert!(cached.is_none());

        // Check metrics (should not increment cache_puts)
        let metrics = hook.metrics();
        assert_eq!(metrics.cache_puts, 0);
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_cache_error_caching_enabled() {
        let hook = CachingHook::new()
            .with_error_caching(true)
            .with_strategy(CachingStrategy::CacheAll);
        let context = create_test_context();

        // Cache an error (should work with error caching enabled)
        hook.try_cache_result(
            &context,
            &HookResult::Cancel("error".to_string()),
            Duration::from_millis(100),
        )
        .unwrap();

        // Check that it was cached
        let cache_key = CacheKey::from_context(&context);
        let cached = hook.cache.get(&cache_key);
        assert!(cached.is_some());
        if let Some(result) = cached {
            assert!(matches!(result, HookResult::Cancel(_)));
        }
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_cache_key_from_context() {
        let mut context1 = create_test_context();
        let mut context2 = create_test_context();

        // Same context should produce same key
        let key1 = CacheKey::from_context(&context1);
        let key2 = CacheKey::from_context(&context1);
        assert_eq!(key1, key2);

        // Different context data should produce different keys
        context1.insert_data("key".to_string(), json!("value1"));
        context2.insert_data("key".to_string(), json!("value2"));

        let key1 = CacheKey::from_context(&context1);
        let key2 = CacheKey::from_context(&context2);
        assert_ne!(key1, key2);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_caching_metrics() {
        let hook = CachingHook::new();
        let context = create_test_context();

        // Simulate some cache operations
        hook.try_get_cached(&context); // Miss
        hook.try_get_cached(&context); // Miss

        let cache_key = CacheKey::from_context(&context);
        hook.cache
            .put(cache_key, HookResult::Continue, None)
            .unwrap();

        hook.try_get_cached(&context); // Hit

        let metrics = hook.metrics();
        assert_eq!(metrics.total_cache_attempts, 3);
        assert_eq!(metrics.cache_hits, 1);
        assert_eq!(metrics.cache_misses, 2);
        assert_eq!(metrics.hit_ratio(), 1.0 / 3.0);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_cache_stats_integration() {
        let hook = CachingHook::new();
        let context = create_test_context();
        let cache_key = CacheKey::from_context(&context);

        // Put some entries
        hook.cache
            .put(cache_key.clone(), HookResult::Continue, None)
            .unwrap();

        // Get cache stats
        let stats = hook.cache_stats();
        assert_eq!(stats.current_size, 1);

        // Clear cache
        hook.clear_cache();
        let stats = hook.cache_stats();
        assert_eq!(stats.current_size, 0);
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_metric_hook_trait() {
        let hook = CachingHook::new();
        let context = create_test_context();

        // Test MetricHook implementation
        hook.record_pre_execution(&context).await.unwrap();

        let result = HookResult::Continue;
        hook.record_post_execution(&context, &result, Duration::from_millis(100))
            .await
            .unwrap();

        // Check that result was cached
        let cache_key = CacheKey::from_context(&context);
        let cached = hook.cache.get(&cache_key);
        assert!(cached.is_some());
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_hook_metadata() {
        let hook = CachingHook::new();
        let metadata = hook.metadata();

        assert_eq!(metadata.name, "CachingHook");
        assert!(metadata.description.is_some());
        assert_eq!(metadata.priority, Priority::HIGH);
        assert_eq!(metadata.language, Language::Native);
        assert!(metadata.tags.contains(&"builtin".to_string()));
        assert!(metadata.tags.contains(&"caching".to_string()));
        assert!(metadata.tags.contains(&"performance".to_string()));
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_builder_methods() {
        let hook = CachingHook::new()
            .with_ttl(Duration::from_secs(600))
            .with_max_entries(2000)
            .with_error_caching(true)
            .with_min_execution_time(Duration::from_millis(25));

        assert_eq!(hook.config.default_ttl, Duration::from_secs(600));
        assert_eq!(hook.config.max_entries, 2000);
        assert!(hook.config.cache_errors);
        assert_eq!(hook.config.min_execution_time, Duration::from_millis(25));
    }
}

#[async_trait]
impl ReplayableHook for CachingHook {
    fn is_replayable(&self) -> bool {
        true
    }

    fn serialize_context(&self, ctx: &HookContext) -> Result<Vec<u8>> {
        // Create a serializable version of the context with caching config
        let mut context_data = ctx.data.clone();

        // Add caching configuration for replay
        context_data.insert(
            "_caching_config".to_string(),
            serde_json::json!({
                "max_entries": self.config.max_entries,
                "default_ttl_secs": self.config.default_ttl.as_secs(),
                "strategy": match &self.config.strategy {
                    CachingStrategy::CacheAll => "CacheAll",
                    CachingStrategy::CacheSuccessOnly => "CacheSuccessOnly",
                    CachingStrategy::CacheSpecificTypes(_) => "CacheSpecificTypes",
                    CachingStrategy::Custom => "Custom",
                },
                "use_full_context": self.config.use_full_context,
                "min_execution_time_ms": self.config.min_execution_time.as_millis(),
                "extend_ttl_on_hit": self.config.extend_ttl_on_hit,
                "cache_errors": self.config.cache_errors,
            }),
        );

        // Add cache statistics for debugging
        let stats = self.cache.stats();
        context_data.insert("_cache_stats".to_string(), serde_json::to_value(&stats)?);

        // Add metrics snapshot
        let metrics = self.metrics.read().unwrap();
        context_data.insert(
            "_caching_metrics".to_string(),
            serde_json::json!({
                "hit_ratio": metrics.hit_ratio(),
                "total_attempts": metrics.total_cache_attempts,
                "hits": metrics.cache_hits,
                "misses": metrics.cache_misses,
                "evictions": metrics.cache_evictions,
                "time_saved_ms": metrics.time_saved_ms,
            }),
        );

        let mut replay_context = ctx.clone();
        replay_context.data = context_data;

        Ok(serde_json::to_vec(&replay_context)?)
    }

    fn deserialize_context(&self, data: &[u8]) -> Result<HookContext> {
        let mut context: HookContext = serde_json::from_slice(data)?;

        // Remove the caching-specific data from context
        context.data.remove("_caching_config");
        context.data.remove("_cache_stats");
        context.data.remove("_caching_metrics");

        Ok(context)
    }

    fn replay_id(&self) -> String {
        format!("{}:{}", self.metadata.name, self.metadata.version)
    }
}
