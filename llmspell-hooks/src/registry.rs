// ABOUTME: Thread-safe hook registry with priority-based execution and language filtering
// ABOUTME: Manages hook registration, storage, and efficient lookup by HookPoint

use crate::priority::{PriorityBucket, PriorityComparator};
use crate::traits::{ArcHook, Hook};
use crate::types::{HookMetadata, HookPoint, Language, Priority};
use dashmap::DashMap;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tracing::{debug, info};

/// Configuration for HookRegistry
#[derive(Debug, Clone)]
pub struct HookRegistryConfig {
    /// Initial capacity for the hooks map
    pub initial_capacity: usize,
    /// Whether hooks are globally enabled by default
    pub global_enabled_default: bool,
    /// Whether to track statistics
    pub enable_stats: bool,
}

impl Default for HookRegistryConfig {
    fn default() -> Self {
        Self {
            initial_capacity: 16,
            global_enabled_default: true,
            enable_stats: true,
        }
    }
}

/// Builder for HookRegistryConfig
#[derive(Debug, Clone)]
pub struct HookRegistryConfigBuilder {
    config: HookRegistryConfig,
}

impl HookRegistryConfigBuilder {
    /// Create a new builder with default configuration
    pub fn new() -> Self {
        Self {
            config: HookRegistryConfig::default(),
        }
    }

    /// Set the initial capacity for the hooks map
    #[must_use]
    pub fn initial_capacity(mut self, capacity: usize) -> Self {
        self.config.initial_capacity = capacity;
        self
    }

    /// Set whether hooks are globally enabled by default
    #[must_use]
    pub fn global_enabled_default(mut self, enabled: bool) -> Self {
        self.config.global_enabled_default = enabled;
        self
    }

    /// Set whether to track statistics
    #[must_use]
    pub fn enable_stats(mut self, enable: bool) -> Self {
        self.config.enable_stats = enable;
        self
    }

    /// Build the configuration
    pub fn build(self) -> HookRegistryConfig {
        self.config
    }
}

impl Default for HookRegistryConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Hook entry with metadata
#[derive(Clone, Debug)]
struct HookEntry {
    hook: ArcHook,
    metadata: HookMetadata,
    enabled: bool,
}

/// Hook registry for managing hooks by point
#[derive(Debug)]
pub struct HookRegistry {
    /// Registry configuration
    config: HookRegistryConfig,
    /// Hooks organized by HookPoint
    hooks: Arc<DashMap<HookPoint, Vec<HookEntry>>>,
    /// OPTIMIZATION: Global hook state using atomic for lock-free access
    global_enabled: Arc<AtomicBool>,
    /// Statistics
    stats: Arc<RwLock<RegistryStats>>,
}

/// Registry statistics
#[derive(Debug, Default, Clone)]
pub struct RegistryStats {
    pub total_hooks: usize,
    pub hooks_by_point: HashMap<HookPoint, usize>,
    pub hooks_by_language: HashMap<Language, usize>,
    pub hooks_by_bucket: HashMap<PriorityBucket, usize>,
}

impl HookRegistry {
    /// Create a new hook registry with default configuration
    pub fn new() -> Self {
        Self::with_config(HookRegistryConfig::default())
    }

    /// Create a new hook registry with custom configuration
    pub fn with_config(config: HookRegistryConfig) -> Self {
        let hooks = if config.initial_capacity > 0 {
            Arc::new(DashMap::with_capacity(config.initial_capacity))
        } else {
            Arc::new(DashMap::new())
        };

        Self {
            config: config.clone(),
            hooks,
            global_enabled: Arc::new(AtomicBool::new(config.global_enabled_default)),
            stats: Arc::new(RwLock::new(RegistryStats::default())),
        }
    }

    /// Create a new builder for HookRegistry configuration
    pub fn builder() -> HookRegistryConfigBuilder {
        HookRegistryConfigBuilder::new()
    }

    /// Register a hook for a specific point
    pub fn register(
        &self,
        point: HookPoint,
        hook: impl Hook + 'static,
    ) -> Result<(), RegistryError> {
        self.register_arc(point, Arc::new(hook))
    }

    /// Register an Arc'd hook
    pub fn register_arc(&self, point: HookPoint, hook: ArcHook) -> Result<(), RegistryError> {
        let metadata = hook.metadata();

        if metadata.name.is_empty() {
            return Err(RegistryError::InvalidHookName);
        }

        let entry = HookEntry {
            hook: hook.clone(),
            metadata: metadata.clone(),
            enabled: true,
        };

        // Add to registry
        let mut hooks = self.hooks.entry(point.clone()).or_default();

        // Check for duplicates
        if hooks.iter().any(|e| e.metadata.name == metadata.name) {
            return Err(RegistryError::DuplicateHook(metadata.name));
        }

        hooks.push(entry);

        // Sort by priority
        PriorityComparator::sort_by_priority(&mut hooks, |entry| entry.metadata.priority);

        drop(hooks); // Release the lock before updating stats

        // Update stats
        self.update_stats();

        info!(
            "Registered hook '{}' for {:?} with priority {:?}",
            metadata.name, point, metadata.priority
        );

        Ok(())
    }

    /// Register multiple hooks at once
    pub fn register_bulk(
        &self,
        registrations: Vec<(HookPoint, ArcHook)>,
    ) -> Result<(), RegistryError> {
        for (point, hook) in registrations {
            self.register_arc(point, hook)?;
        }
        Ok(())
    }

    /// Unregister a hook by name
    pub fn unregister(&self, point: &HookPoint, hook_name: &str) -> Result<(), RegistryError> {
        let mut hooks = self
            .hooks
            .get_mut(point)
            .ok_or(RegistryError::HookPointNotFound)?;

        let initial_len = hooks.len();
        hooks.retain(|entry| entry.metadata.name != hook_name);

        if hooks.len() == initial_len {
            return Err(RegistryError::HookNotFound(hook_name.to_string()));
        }

        drop(hooks); // Release the lock before updating stats

        // Update stats
        self.update_stats();

        info!("Unregistered hook '{}' from {:?}", hook_name, point);
        Ok(())
    }

    /// Get all hooks for a point
    pub fn get_hooks(&self, point: &HookPoint) -> Vec<ArcHook> {
        self.get_filtered_hooks(point, |_| true)
    }

    /// Get hooks filtered by language
    pub fn get_hooks_by_language(&self, point: &HookPoint, language: Language) -> Vec<ArcHook> {
        self.get_filtered_hooks(point, |entry| entry.metadata.language == language)
    }

    /// Get hooks filtered by priority range
    pub fn get_hooks_by_priority_range(
        &self,
        point: &HookPoint,
        min: Priority,
        max: Priority,
    ) -> Vec<ArcHook> {
        self.get_filtered_hooks(point, |entry| {
            entry.metadata.priority.0 >= min.0 && entry.metadata.priority.0 <= max.0
        })
    }

    /// Get hooks with custom filter
    fn get_filtered_hooks<F>(&self, point: &HookPoint, filter: F) -> Vec<ArcHook>
    where
        F: Fn(&HookEntry) -> bool,
    {
        // OPTIMIZATION: Use atomic load instead of RwLock read
        if !self.global_enabled.load(Ordering::Relaxed) {
            return Vec::new();
        }

        self.hooks
            .get(point)
            .map(|hooks| {
                // OPTIMIZATION: Use iterator chain to avoid intermediate Vec allocation
                hooks
                    .iter()
                    .filter(|entry| entry.enabled && filter(entry))
                    .map(|entry| entry.hook.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Enable or disable a specific hook
    pub fn set_hook_enabled(
        &self,
        point: &HookPoint,
        hook_name: &str,
        enabled: bool,
    ) -> Result<(), RegistryError> {
        let mut hooks = self
            .hooks
            .get_mut(point)
            .ok_or(RegistryError::HookPointNotFound)?;

        let entry = hooks
            .iter_mut()
            .find(|e| e.metadata.name == hook_name)
            .ok_or_else(|| RegistryError::HookNotFound(hook_name.to_string()))?;

        entry.enabled = enabled;

        debug!(
            "{} hook '{}' for {:?}",
            if enabled { "Enabled" } else { "Disabled" },
            hook_name,
            point
        );

        Ok(())
    }

    /// Enable or disable all hooks globally
    pub fn set_global_enabled(&self, enabled: bool) {
        // OPTIMIZATION: Use atomic store instead of RwLock write
        self.global_enabled.store(enabled, Ordering::Relaxed);
        info!(
            "{} all hooks globally",
            if enabled { "Enabled" } else { "Disabled" }
        );
    }

    /// Check if hooks are globally enabled
    pub fn is_global_enabled(&self) -> bool {
        // OPTIMIZATION: Use atomic load instead of RwLock read
        self.global_enabled.load(Ordering::Relaxed)
    }

    /// Clear all hooks for a specific point
    pub fn clear_point(&self, point: &HookPoint) {
        self.hooks.remove(point);
        self.update_stats();
        info!("Cleared all hooks for {:?}", point);
    }

    /// Clear all hooks
    pub fn clear_all(&self) {
        self.hooks.clear();
        self.update_stats();
        info!("Cleared all hooks from registry");
    }

    /// Get registry statistics
    pub fn stats(&self) -> RegistryStats {
        self.stats.read().clone()
    }

    /// Update statistics
    fn update_stats(&self) {
        if !self.config.enable_stats {
            return;
        }

        let mut stats = RegistryStats::default();

        for entry in self.hooks.iter() {
            let point = entry.key();
            let hooks = entry.value();

            stats.total_hooks += hooks.len();
            stats.hooks_by_point.insert(point.clone(), hooks.len());

            for hook_entry in hooks {
                *stats
                    .hooks_by_language
                    .entry(hook_entry.metadata.language)
                    .or_insert(0) += 1;

                let bucket = PriorityComparator::get_bucket(&hook_entry.metadata.priority);
                *stats.hooks_by_bucket.entry(bucket).or_insert(0) += 1;
            }
        }

        *self.stats.write() = stats;
    }

    /// Get hook names for a point
    pub fn get_hook_names(&self, point: &HookPoint) -> Vec<String> {
        self.hooks
            .get(point)
            .map(|hooks| {
                hooks
                    .iter()
                    .map(|entry| entry.metadata.name.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Check if a hook exists
    pub fn has_hook(&self, point: &HookPoint, hook_name: &str) -> bool {
        self.hooks
            .get(point)
            .map(|hooks| hooks.iter().any(|e| e.metadata.name == hook_name))
            .unwrap_or(false)
    }

    /// Get all registered hook points
    pub fn get_hook_points(&self) -> Vec<HookPoint> {
        self.hooks.iter().map(|entry| entry.key().clone()).collect()
    }
}

impl Default for HookRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for HookRegistry {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            hooks: self.hooks.clone(),
            global_enabled: self.global_enabled.clone(),
            stats: self.stats.clone(),
        }
    }
}

/// Registry errors
#[derive(Debug, thiserror::Error)]
pub enum RegistryError {
    #[error("Hook name cannot be empty")]
    InvalidHookName,

    #[error("Hook '{0}' already registered")]
    DuplicateHook(String),

    #[error("Hook point not found")]
    HookPointNotFound,

    #[error("Hook '{0}' not found")]
    HookNotFound(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::HookContext;
    use crate::result::HookResult;
    use crate::traits::FnHook;
    #[test]
    fn test_hook_registration() {
        let registry = HookRegistry::new();

        let hook = FnHook::new("test_hook", |_ctx: &mut HookContext| {
            Ok(HookResult::Continue)
        });

        registry.register(HookPoint::BeforeAgentInit, hook).unwrap();

        let hooks = registry.get_hooks(&HookPoint::BeforeAgentInit);
        assert_eq!(hooks.len(), 1);

        let stats = registry.stats();
        assert_eq!(stats.total_hooks, 1);
        assert_eq!(
            stats.hooks_by_point.get(&HookPoint::BeforeAgentInit),
            Some(&1)
        );
    }
    #[test]
    fn test_priority_ordering() {
        let registry = HookRegistry::new();

        // Register hooks with different priorities
        let high_hook =
            FnHook::new("high", |_| Ok(HookResult::Continue)).with_metadata(HookMetadata {
                name: "high".to_string(),
                priority: Priority::HIGH,
                ..Default::default()
            });

        let low_hook =
            FnHook::new("low", |_| Ok(HookResult::Continue)).with_metadata(HookMetadata {
                name: "low".to_string(),
                priority: Priority::LOW,
                ..Default::default()
            });

        let normal_hook =
            FnHook::new("normal", |_| Ok(HookResult::Continue)).with_metadata(HookMetadata {
                name: "normal".to_string(),
                priority: Priority::NORMAL,
                ..Default::default()
            });

        registry
            .register(HookPoint::BeforeAgentInit, low_hook)
            .unwrap();
        registry
            .register(HookPoint::BeforeAgentInit, high_hook)
            .unwrap();
        registry
            .register(HookPoint::BeforeAgentInit, normal_hook)
            .unwrap();

        let names = registry.get_hook_names(&HookPoint::BeforeAgentInit);
        assert_eq!(names, vec!["high", "normal", "low"]);
    }
    #[test]
    fn test_language_filtering() {
        let registry = HookRegistry::new();

        let lua_hook =
            FnHook::new("lua_hook", |_| Ok(HookResult::Continue)).with_metadata(HookMetadata {
                name: "lua_hook".to_string(),
                language: Language::Lua,
                ..Default::default()
            });

        let native_hook =
            FnHook::new("native_hook", |_| Ok(HookResult::Continue)).with_metadata(HookMetadata {
                name: "native_hook".to_string(),
                language: Language::Native,
                ..Default::default()
            });

        registry
            .register(HookPoint::BeforeAgentInit, lua_hook)
            .unwrap();
        registry
            .register(HookPoint::BeforeAgentInit, native_hook)
            .unwrap();

        let lua_hooks = registry.get_hooks_by_language(&HookPoint::BeforeAgentInit, Language::Lua);
        assert_eq!(lua_hooks.len(), 1);

        let native_hooks =
            registry.get_hooks_by_language(&HookPoint::BeforeAgentInit, Language::Native);
        assert_eq!(native_hooks.len(), 1);
    }
    #[test]
    fn test_duplicate_registration() {
        let registry = HookRegistry::new();

        let hook1 = FnHook::new("duplicate", |_| Ok(HookResult::Continue));
        let hook2 = FnHook::new("duplicate", |_| Ok(HookResult::Continue));

        registry
            .register(HookPoint::BeforeAgentInit, hook1)
            .unwrap();
        let result = registry.register(HookPoint::BeforeAgentInit, hook2);

        assert!(matches!(result, Err(RegistryError::DuplicateHook(_))));
    }
    #[test]
    fn test_unregister() {
        let registry = HookRegistry::new();

        let hook = FnHook::new("test_hook", |_| Ok(HookResult::Continue));
        registry.register(HookPoint::BeforeAgentInit, hook).unwrap();

        assert!(registry.has_hook(&HookPoint::BeforeAgentInit, "test_hook"));

        registry
            .unregister(&HookPoint::BeforeAgentInit, "test_hook")
            .unwrap();

        assert!(!registry.has_hook(&HookPoint::BeforeAgentInit, "test_hook"));
        assert_eq!(registry.stats().total_hooks, 0);
    }
    #[test]
    fn test_enable_disable() {
        let registry = HookRegistry::new();

        let hook = FnHook::new("test_hook", |_| Ok(HookResult::Continue));
        registry.register(HookPoint::BeforeAgentInit, hook).unwrap();

        // Disable the hook
        registry
            .set_hook_enabled(&HookPoint::BeforeAgentInit, "test_hook", false)
            .unwrap();
        let hooks = registry.get_hooks(&HookPoint::BeforeAgentInit);
        assert_eq!(hooks.len(), 0);

        // Re-enable the hook
        registry
            .set_hook_enabled(&HookPoint::BeforeAgentInit, "test_hook", true)
            .unwrap();
        let hooks = registry.get_hooks(&HookPoint::BeforeAgentInit);
        assert_eq!(hooks.len(), 1);
    }
    #[test]
    fn test_global_enable() {
        let registry = HookRegistry::new();

        let hook = FnHook::new("test_hook", |_| Ok(HookResult::Continue));
        registry.register(HookPoint::BeforeAgentInit, hook).unwrap();

        // Disable globally
        registry.set_global_enabled(false);
        assert!(!registry.is_global_enabled());

        let hooks = registry.get_hooks(&HookPoint::BeforeAgentInit);
        assert_eq!(hooks.len(), 0);

        // Re-enable globally
        registry.set_global_enabled(true);
        let hooks = registry.get_hooks(&HookPoint::BeforeAgentInit);
        assert_eq!(hooks.len(), 1);
    }
    #[test]
    fn test_bulk_registration() {
        let registry = HookRegistry::new();

        let registrations = vec![
            (
                HookPoint::BeforeAgentInit,
                Arc::new(FnHook::new("hook1", |_| Ok(HookResult::Continue))) as ArcHook,
            ),
            (
                HookPoint::AfterAgentInit,
                Arc::new(FnHook::new("hook2", |_| Ok(HookResult::Continue))) as ArcHook,
            ),
            (
                HookPoint::BeforeAgentExecution,
                Arc::new(FnHook::new("hook3", |_| Ok(HookResult::Continue))) as ArcHook,
            ),
        ];

        registry.register_bulk(registrations).unwrap();

        assert_eq!(registry.stats().total_hooks, 3);
        assert_eq!(registry.get_hook_points().len(), 3);
    }
    #[test]
    fn test_clear_operations() {
        let registry = HookRegistry::new();

        let hook1 = FnHook::new("hook1", |_| Ok(HookResult::Continue));
        let hook2 = FnHook::new("hook2", |_| Ok(HookResult::Continue));

        registry
            .register(HookPoint::BeforeAgentInit, hook1)
            .unwrap();
        registry.register(HookPoint::AfterAgentInit, hook2).unwrap();

        // Clear specific point
        registry.clear_point(&HookPoint::BeforeAgentInit);
        assert_eq!(registry.get_hooks(&HookPoint::BeforeAgentInit).len(), 0);
        assert_eq!(registry.get_hooks(&HookPoint::AfterAgentInit).len(), 1);

        // Clear all
        registry.clear_all();
        assert_eq!(registry.stats().total_hooks, 0);
    }
    #[test]
    fn test_hook_registry_builder() {
        // Test builder pattern
        let registry = HookRegistry::builder()
            .initial_capacity(32)
            .global_enabled_default(false)
            .enable_stats(false)
            .build();

        let registry = HookRegistry::with_config(registry);

        // Check global enabled is false
        assert!(!registry.is_global_enabled());

        // Register a hook
        let hook = FnHook::new("test_hook", |_| Ok(HookResult::Continue));
        registry.register(HookPoint::BeforeAgentInit, hook).unwrap();

        // Since stats are disabled, stats should remain at default
        let stats = registry.stats();
        assert_eq!(stats.total_hooks, 0); // Stats not updated due to enable_stats=false
    }
}
