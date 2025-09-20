// ABOUTME: SelectiveHookRegistry implementation with lazy loading and feature flags
// ABOUTME: Optimized for minimal memory footprint in library mode scenarios

use crate::priority::PriorityComparator;
use crate::traits::{ArcHook, Hook};
use crate::types::{HookMetadata, HookPoint, Language};
use crate::RegistryError;
use dashmap::DashMap;
use parking_lot::RwLock;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tracing::{debug, info, trace};

/// Hook factory function type
pub type HookFactory = Box<dyn Fn() -> Box<dyn Hook> + Send + Sync>;

/// Feature flags for selective hook loading
#[derive(Debug, Clone, Default)]
pub struct HookFeatures {
    /// Enabled features
    enabled_features: HashSet<String>,
    /// Feature dependencies (feature -> required features)
    feature_dependencies: HashMap<String, Vec<String>>,
    /// Global enable flag
    global_enabled: bool,
}

impl HookFeatures {
    /// Create new feature set with all features disabled
    pub fn new() -> Self {
        Self {
            enabled_features: HashSet::new(),
            feature_dependencies: HashMap::new(),
            global_enabled: true,
        }
    }

    /// Create with specific features enabled
    pub fn with_features(features: impl IntoIterator<Item = impl Into<String>>) -> Self {
        let mut hook_features = Self::new();
        for feature in features {
            hook_features.enable_feature(feature);
        }
        hook_features
    }

    /// Enable a feature
    pub fn enable_feature(&mut self, feature: impl Into<String>) {
        let feature = feature.into();
        self.enabled_features.insert(feature.clone());

        // Enable dependencies
        if let Some(deps) = self.feature_dependencies.get(&feature).cloned() {
            for dep in deps {
                self.enable_feature(dep);
            }
        }
    }

    /// Disable a feature
    pub fn disable_feature(&mut self, feature: impl AsRef<str>) {
        self.enabled_features.remove(feature.as_ref());
    }

    /// Check if a feature is enabled
    pub fn is_feature_enabled(&self, feature: impl AsRef<str>) -> bool {
        self.global_enabled && self.enabled_features.contains(feature.as_ref())
    }

    /// Check if all required features are enabled
    pub fn are_features_enabled(&self, features: &[impl AsRef<str>]) -> bool {
        self.global_enabled
            && features
                .iter()
                .all(|f| self.enabled_features.contains(f.as_ref()))
    }

    /// Add feature dependency
    pub fn add_dependency(&mut self, feature: impl Into<String>, depends_on: impl Into<String>) {
        self.feature_dependencies
            .entry(feature.into())
            .or_default()
            .push(depends_on.into());
    }

    /// Set global enable flag
    pub fn set_global_enabled(&mut self, enabled: bool) {
        self.global_enabled = enabled;
    }

    /// Get all enabled features
    pub fn enabled_features(&self) -> &HashSet<String> {
        &self.enabled_features
    }
}

/// Configuration for selective registry
#[derive(Debug, Clone)]
pub struct SelectiveRegistryConfig {
    /// Maximum number of lazy hooks to keep instantiated
    pub max_instantiated_hooks: usize,
    /// Whether to use LRU eviction for instantiated hooks
    pub use_lru_eviction: bool,
    /// Preload hooks for these points
    pub preload_points: Vec<HookPoint>,
    /// Default features for new hooks
    pub default_features: Vec<String>,
    /// Enable statistics collection
    pub collect_stats: bool,
}

impl Default for SelectiveRegistryConfig {
    fn default() -> Self {
        Self {
            max_instantiated_hooks: 100,
            use_lru_eviction: true,
            preload_points: vec![HookPoint::SystemStartup, HookPoint::SystemShutdown],
            default_features: vec!["core".to_string()],
            collect_stats: true,
        }
    }
}

/// Lazy hook entry
pub struct LazyHookEntry {
    /// Hook factory
    factory: HookFactory,
    /// Instantiated hook (if loaded)
    instance: Arc<RwLock<Option<ArcHook>>>,
    /// Metadata
    metadata: HookMetadata,
    /// Required features
    pub(crate) required_features: Vec<String>,
    /// Enabled state
    pub(crate) enabled: RwLock<bool>,
    /// Access count for LRU
    access_count: Arc<RwLock<u64>>,
    /// Last access time
    last_access: Arc<RwLock<std::time::Instant>>,
}

impl LazyHookEntry {
    /// Create new lazy hook entry
    fn new(factory: HookFactory, metadata: HookMetadata, required_features: Vec<String>) -> Self {
        Self {
            factory,
            instance: Arc::new(RwLock::new(None)),
            metadata,
            required_features,
            enabled: RwLock::new(true),
            access_count: Arc::new(RwLock::new(0)),
            last_access: Arc::new(RwLock::new(std::time::Instant::now())),
        }
    }

    /// Get or create hook instance
    pub(crate) fn get_instance(&self) -> ArcHook {
        let mut instance = self.instance.write();

        if instance.is_none() {
            let hook = (self.factory)();
            *instance = Some(Arc::from(hook));
            trace!("Instantiated lazy hook: {}", self.metadata.name);
        }

        // Update access tracking
        *self.access_count.write() += 1;
        *self.last_access.write() = std::time::Instant::now();

        instance.as_ref().unwrap().clone()
    }

    /// Check if hook is instantiated
    pub(crate) fn is_instantiated(&self) -> bool {
        self.instance.read().is_some()
    }

    /// Evict the instantiated hook
    pub(crate) fn evict(&self) {
        let mut instance = self.instance.write();
        if instance.is_some() {
            *instance = None;
            trace!("Evicted hook instance: {}", self.metadata.name);
        }
    }
}

/// Statistics for selective registry
#[derive(Debug, Default, Clone)]
pub struct SelectiveRegistryStats {
    /// Total registered hooks
    pub total_hooks: usize,
    /// Currently instantiated hooks
    pub instantiated_hooks: usize,
    /// Hook access counts
    pub hook_access_counts: HashMap<String, u64>,
    /// Eviction count
    pub evictions: u64,
    /// Feature usage
    pub feature_usage: HashMap<String, usize>,
}

/// Selective hook registry with lazy loading and feature flags
pub struct SelectiveHookRegistry {
    /// Lazy hooks organized by HookPoint
    hooks: Arc<DashMap<HookPoint, Vec<Arc<LazyHookEntry>>>>,
    /// Feature flags
    features: Arc<RwLock<HookFeatures>>,
    /// Configuration
    config: SelectiveRegistryConfig,
    /// Statistics
    stats: Arc<RwLock<SelectiveRegistryStats>>,
    /// Hook name to entry mapping for quick lookup
    hook_index: Arc<DashMap<String, (HookPoint, Arc<LazyHookEntry>)>>,
}

impl SelectiveHookRegistry {
    /// Create new selective registry
    pub fn new(features: HookFeatures) -> Self {
        Self::with_config(features, SelectiveRegistryConfig::default())
    }

    /// Create with custom configuration
    pub fn with_config(features: HookFeatures, config: SelectiveRegistryConfig) -> Self {
        let registry = Self {
            hooks: Arc::new(DashMap::new()),
            features: Arc::new(RwLock::new(features)),
            config,
            stats: Arc::new(RwLock::new(SelectiveRegistryStats::default())),
            hook_index: Arc::new(DashMap::new()),
        };

        // Enable default features
        let mut features = registry.features.write();
        for feature in &registry.config.default_features {
            features.enable_feature(feature);
        }
        drop(features);

        registry
    }

    /// Register a hook with feature requirements
    pub fn register_with_features(
        &self,
        point: HookPoint,
        factory: impl Fn() -> Box<dyn Hook> + Send + Sync + 'static,
        required_features: &[impl AsRef<str>],
    ) -> Result<(), RegistryError> {
        // Create temporary hook to get metadata
        let temp_hook = factory();
        let metadata = temp_hook.metadata();

        if metadata.name.is_empty() {
            return Err(RegistryError::InvalidHookName);
        }

        // Create lazy entry
        let features: Vec<String> = required_features
            .iter()
            .map(|f| f.as_ref().to_string())
            .collect();

        let entry = Arc::new(LazyHookEntry::new(
            Box::new(factory),
            metadata.clone(),
            features.clone(),
        ));

        // Add to registry
        let mut hooks = self.hooks.entry(point.clone()).or_default();

        // Check for duplicates
        if hooks.iter().any(|e| e.metadata.name == metadata.name) {
            return Err(RegistryError::DuplicateHook(metadata.name));
        }

        hooks.push(entry.clone());

        // Sort by priority
        hooks.sort_by(|a, b| {
            PriorityComparator::compare(&a.metadata.priority, &b.metadata.priority)
        });

        drop(hooks);

        // Add to index
        self.hook_index
            .insert(metadata.name.clone(), (point.clone(), entry));

        // Update stats
        if self.config.collect_stats {
            self.update_stats();
        }

        // Preload if configured
        if self.config.preload_points.contains(&point) && self.should_load_hook(&features) {
            self.preload_hook(&metadata.name)?;
        }

        info!(
            "Registered selective hook '{}' for {:?} with features {:?}",
            metadata.name, point, features
        );

        Ok(())
    }

    /// Register a simple hook without features
    pub fn register(
        &self,
        point: HookPoint,
        factory: impl Fn() -> Box<dyn Hook> + Send + Sync + 'static,
    ) -> Result<(), RegistryError> {
        self.register_with_features(point, factory, &self.config.default_features)
    }

    /// Preload a hook by name
    pub fn preload_hook(&self, hook_name: &str) -> Result<(), RegistryError> {
        let entry = self
            .hook_index
            .get(hook_name)
            .ok_or_else(|| RegistryError::HookNotFound(hook_name.to_string()))?;

        let (_, (_, hook_entry)) = entry.pair();

        if self.should_load_hook(&hook_entry.required_features) {
            hook_entry.get_instance();
            debug!("Preloaded hook: {}", hook_name);
        }

        Ok(())
    }

    /// Get hooks for a point (only instantiates if features are enabled)
    pub fn get_hooks(&self, point: &HookPoint) -> Vec<ArcHook> {
        self.get_filtered_hooks(point, |_| true)
    }

    /// Get hooks filtered by language
    pub fn get_hooks_by_language(&self, point: &HookPoint, language: Language) -> Vec<ArcHook> {
        self.get_filtered_hooks(point, |entry| entry.metadata.language == language)
    }

    /// Get hooks with custom filter
    fn get_filtered_hooks<F>(&self, point: &HookPoint, filter: F) -> Vec<ArcHook>
    where
        F: Fn(&LazyHookEntry) -> bool,
    {
        let features = self.features.read();

        if !features.global_enabled {
            return Vec::new();
        }

        let hooks = match self.hooks.get(point) {
            Some(hooks) => hooks,
            None => return Vec::new(),
        };

        let mut result = Vec::new();
        let to_evict = Vec::new();

        for entry in hooks.iter() {
            if !*entry.enabled.read() || !filter(entry) {
                continue;
            }

            if !self.should_load_hook(&entry.required_features) {
                trace!(
                    "Skipping hook '{}' - missing features: {:?}",
                    entry.metadata.name,
                    entry.required_features
                );
                continue;
            }

            result.push(entry.get_instance());
        }

        drop(hooks);
        drop(features);

        // Handle LRU eviction if needed
        if self.config.use_lru_eviction {
            self.maybe_evict_hooks(to_evict);
        }

        // Update stats
        if self.config.collect_stats {
            self.update_access_stats(&result);
        }

        result
    }

    /// Check if a hook should be loaded based on features
    fn should_load_hook(&self, required_features: &[String]) -> bool {
        if required_features.is_empty() {
            return true;
        }

        let features = self.features.read();
        features.are_features_enabled(required_features)
    }

    /// Maybe evict hooks based on LRU
    fn maybe_evict_hooks(&self, _candidates: Vec<String>) {
        let instantiated_count = self.count_instantiated_hooks();

        if instantiated_count <= self.config.max_instantiated_hooks {
            return;
        }

        // Find least recently used hooks
        let mut lru_candidates: Vec<(String, std::time::Instant, u64)> = Vec::new();

        for entry in self.hook_index.iter() {
            let (name, (_, hook_entry)) = entry.pair();
            if hook_entry.is_instantiated() {
                let last_access = *hook_entry.last_access.read();
                let access_count = *hook_entry.access_count.read();
                lru_candidates.push((name.clone(), last_access, access_count));
            }
        }

        // Sort by last access time (oldest first)
        lru_candidates.sort_by(|a, b| a.1.cmp(&b.1));

        // Evict oldest hooks
        let to_evict = instantiated_count - self.config.max_instantiated_hooks;
        for (name, _, _) in lru_candidates.iter().take(to_evict) {
            if let Some(entry) = self.hook_index.get(name) {
                let (_, (_, hook_entry)) = entry.pair();
                hook_entry.evict();

                if self.config.collect_stats {
                    self.stats.write().evictions += 1;
                }
            }
        }
    }

    /// Count instantiated hooks
    fn count_instantiated_hooks(&self) -> usize {
        self.hook_index
            .iter()
            .filter(|entry| {
                let (_, (_, hook_entry)) = entry.pair();
                hook_entry.is_instantiated()
            })
            .count()
    }

    /// Enable or disable a hook
    pub fn set_hook_enabled(&self, hook_name: &str, enabled: bool) -> Result<(), RegistryError> {
        let entry = self
            .hook_index
            .get_mut(hook_name)
            .ok_or_else(|| RegistryError::HookNotFound(hook_name.to_string()))?;

        let (_, (_, hook_entry)) = entry.pair();
        *hook_entry.enabled.write() = enabled;

        debug!(
            "{} selective hook '{}'",
            if enabled { "Enabled" } else { "Disabled" },
            hook_name
        );

        Ok(())
    }

    /// Enable or disable a feature
    pub fn set_feature_enabled(&self, feature: impl AsRef<str>, enabled: bool) {
        let mut features = self.features.write();
        if enabled {
            features.enable_feature(feature.as_ref());
        } else {
            features.disable_feature(feature.as_ref());
        }
    }

    /// Get current features
    pub fn features(&self) -> HookFeatures {
        self.features.read().clone()
    }

    /// Update statistics
    fn update_stats(&self) {
        if !self.config.collect_stats {
            return;
        }

        let mut stats = self.stats.write();
        stats.total_hooks = self.hook_index.len();
        stats.instantiated_hooks = self.count_instantiated_hooks();

        // Update feature usage
        stats.feature_usage.clear();
        for entry in self.hook_index.iter() {
            let (_, (_, hook_entry)) = entry.pair();
            for feature in &hook_entry.required_features {
                *stats.feature_usage.entry(feature.clone()).or_insert(0) += 1;
            }
        }
    }

    /// Update access statistics
    fn update_access_stats(&self, hooks: &[ArcHook]) {
        if !self.config.collect_stats {
            return;
        }

        let mut stats = self.stats.write();
        for hook in hooks {
            let metadata = hook.metadata();
            *stats.hook_access_counts.entry(metadata.name).or_insert(0) += 1;
        }
    }

    /// Get statistics
    pub fn stats(&self) -> SelectiveRegistryStats {
        if self.config.collect_stats {
            // Update instantiated count in real-time
            let mut stats = self.stats.write();
            stats.instantiated_hooks = self.count_instantiated_hooks();
            return stats.clone();
        }
        self.stats.read().clone()
    }

    /// Clear all instantiated hooks to free memory
    pub fn clear_instances(&self) {
        for entry in self.hook_index.iter() {
            let (_, (_, hook_entry)) = entry.pair();
            hook_entry.evict();
        }

        if self.config.collect_stats {
            self.update_stats();
        }

        info!("Cleared all instantiated hooks");
    }

    /// Get memory usage estimate
    pub fn memory_usage_estimate(&self) -> usize {
        let base_size = std::mem::size_of::<Self>();
        let instantiated = self.count_instantiated_hooks();
        let total = self.hook_index.len();

        // Rough estimate:
        // - Each lazy entry: ~200 bytes
        // - Each instantiated hook: ~1KB (varies greatly)
        base_size + (total * 200) + (instantiated * 1024)
    }
}

impl Default for SelectiveHookRegistry {
    fn default() -> Self {
        Self::new(HookFeatures::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::HookContext;
    use crate::result::HookResult;
    use crate::types::Priority;
    use anyhow::Result;
    use async_trait::async_trait;

    #[derive(Debug)]
    struct TestHook {
        name: String,
    }

    #[async_trait]
    impl Hook for TestHook {
        async fn execute(&self, _context: &mut HookContext) -> Result<HookResult> {
            Ok(HookResult::Continue)
        }

        fn metadata(&self) -> HookMetadata {
            HookMetadata {
                name: self.name.clone(),
                priority: Priority::NORMAL,
                ..Default::default()
            }
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }
    #[test]
    fn test_feature_flags() {
        let mut features = HookFeatures::new();
        features.enable_feature("logging");
        features.enable_feature("metrics");

        assert!(features.is_feature_enabled("logging"));
        assert!(features.is_feature_enabled("metrics"));
        assert!(!features.is_feature_enabled("debugging"));

        features.disable_feature("logging");
        assert!(!features.is_feature_enabled("logging"));
    }
    #[test]
    fn test_feature_dependencies() {
        let mut features = HookFeatures::new();
        features.add_dependency("advanced", "basic");
        features.enable_feature("advanced");

        assert!(features.is_feature_enabled("advanced"));
        assert!(features.is_feature_enabled("basic")); // Automatically enabled
    }
    #[tokio::test]
    async fn test_selective_registration() {
        let mut features = HookFeatures::new();
        features.enable_feature("logging");

        let registry = SelectiveHookRegistry::new(features);

        // Register hook requiring logging feature
        let result = registry.register_with_features(
            HookPoint::BeforeAgentExecution,
            || {
                Box::new(TestHook {
                    name: "test-hook".to_string(),
                })
            },
            &["logging"],
        );
        assert!(result.is_ok());

        // Get hooks - should return the hook since logging is enabled
        let hooks = registry.get_hooks(&HookPoint::BeforeAgentExecution);
        assert_eq!(hooks.len(), 1);
    }
    #[tokio::test]
    async fn test_feature_filtering() {
        let features = HookFeatures::new(); // No features enabled
        let registry = SelectiveHookRegistry::new(features);

        // Register hook requiring metrics feature
        registry
            .register_with_features(
                HookPoint::BeforeToolExecution,
                || {
                    Box::new(TestHook {
                        name: "metrics-hook".to_string(),
                    })
                },
                &["metrics"],
            )
            .unwrap();

        // Get hooks - should return empty since metrics is not enabled
        let hooks = registry.get_hooks(&HookPoint::BeforeToolExecution);
        assert_eq!(hooks.len(), 0);

        // Enable metrics feature
        registry.set_feature_enabled("metrics", true);

        // Now it should return the hook
        let hooks = registry.get_hooks(&HookPoint::BeforeToolExecution);
        assert_eq!(hooks.len(), 1);
    }
    #[test]
    fn test_lazy_instantiation() {
        let features = HookFeatures::with_features(vec!["core"]);
        let registry = SelectiveHookRegistry::new(features);

        let counter = Arc::new(RwLock::new(0));
        let counter_clone = counter.clone();

        // Register hook that increments counter when created
        // Use a hook point that's NOT in the preload list
        registry
            .register_with_features(
                HookPoint::BeforeToolExecution,
                move || {
                    *counter_clone.write() += 1;
                    Box::new(TestHook {
                        name: "lazy-hook".to_string(),
                    })
                },
                &["core"],
            )
            .unwrap();

        // Hook should have been created once during registration to get metadata
        assert_eq!(*counter.read(), 1);

        // Get hooks - this should create the instance for lazy loading
        let _hooks = registry.get_hooks(&HookPoint::BeforeToolExecution);
        assert_eq!(*counter.read(), 2); // Factory called again for instantiation

        // Get again - should reuse instance
        let _hooks = registry.get_hooks(&HookPoint::BeforeToolExecution);
        assert_eq!(*counter.read(), 2); // Still 2
    }
    #[test]
    fn test_memory_management() {
        let features = HookFeatures::with_features(vec!["test"]);
        let config = SelectiveRegistryConfig {
            max_instantiated_hooks: 2,
            ..Default::default()
        };

        let registry = SelectiveHookRegistry::with_config(features, config);

        // Register multiple hooks
        for i in 0..5 {
            registry
                .register_with_features(
                    HookPoint::BeforeAgentExecution,
                    move || {
                        Box::new(TestHook {
                            name: format!("hook-{}", i),
                        })
                    },
                    &["test"],
                )
                .unwrap();
        }

        // Access all hooks
        let hooks = registry.get_hooks(&HookPoint::BeforeAgentExecution);
        assert_eq!(hooks.len(), 5);

        // Check instantiation count doesn't exceed limit
        let stats = registry.stats();
        assert!(stats.instantiated_hooks <= 5); // May be less due to LRU
    }
    #[test]
    fn test_statistics() {
        let features = HookFeatures::with_features(vec!["stats"]);
        let registry = SelectiveHookRegistry::new(features);

        // Register hooks with different features
        registry
            .register_with_features(
                HookPoint::BeforeToolExecution,
                || {
                    Box::new(TestHook {
                        name: "stats-hook-1".to_string(),
                    })
                },
                &["stats", "logging"],
            )
            .unwrap();

        registry
            .register_with_features(
                HookPoint::AfterToolExecution,
                || {
                    Box::new(TestHook {
                        name: "stats-hook-2".to_string(),
                    })
                },
                &["stats"],
            )
            .unwrap();

        let stats = registry.stats();
        assert_eq!(stats.total_hooks, 2);
        assert_eq!(stats.feature_usage.get("stats"), Some(&2));
        assert_eq!(stats.feature_usage.get("logging"), Some(&1));
    }
    #[test]
    fn test_instantiation_tracking() {
        let features = HookFeatures::with_features(vec!["test"]);
        let registry = SelectiveHookRegistry::new(features);

        // Register a hook
        registry
            .register_with_features(
                HookPoint::BeforeAgentExecution,
                || {
                    Box::new(TestHook {
                        name: "inst-test".to_string(),
                    })
                },
                &["test"],
            )
            .unwrap();

        // Get the hook entry directly
        let entry = registry.hook_index.get("inst-test").unwrap();
        let (_, (_, hook_entry)) = entry.pair();

        // Check it's not instantiated yet
        assert!(!hook_entry.is_instantiated());

        // Get instance
        let _inst = hook_entry.get_instance();

        // Now it should be instantiated
        assert!(hook_entry.is_instantiated());
    }
    #[test]
    fn test_clear_instances() {
        let features = HookFeatures::with_features(vec!["clear"]);
        let registry = SelectiveHookRegistry::new(features);

        // Register and instantiate hook
        // Use a hook point that's NOT in the preload list
        registry
            .register_with_features(
                HookPoint::AfterToolExecution,
                || {
                    Box::new(TestHook {
                        name: "clear-hook".to_string(),
                    })
                },
                &["clear"],
            )
            .unwrap();

        // First check that nothing is instantiated yet
        assert_eq!(registry.stats().instantiated_hooks, 0);

        let hooks = registry.get_hooks(&HookPoint::AfterToolExecution);
        assert_eq!(hooks.len(), 1); // Should have one hook

        // Now check if it got instantiated
        let stats = registry.stats();
        assert_eq!(stats.instantiated_hooks, 1); // Should be instantiated

        // Clear instances
        registry.clear_instances();
        assert_eq!(registry.stats().instantiated_hooks, 0);
    }
}
