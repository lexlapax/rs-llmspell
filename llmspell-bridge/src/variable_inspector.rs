//! Variable inspection system for slow path debugging
//!
//! This module provides script-agnostic trait definitions for variable inspection
//! that operates entirely in the slow path, leveraging cached variables from
//! `ContextBatcher`. Script-specific implementations are in the respective
//! language modules (lua/, js/, python/, etc.).

use crate::debug_state_cache::{CachedVariable, DebugStateCache};
use crate::execution_context::SharedExecutionContext;
use crate::lua::sync_utils::block_on_async;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, trace, warn};

/// Script-agnostic variable inspector trait
pub trait VariableInspector: Send + Sync {
    /// Inspect variables (SLOW PATH ONLY)
    ///
    /// This method batches variable reads for efficiency and uses
    /// cached values when available.
    fn inspect_variables(
        &self,
        variable_names: &[String],
        batcher: &mut ContextBatcher,
    ) -> HashMap<String, JsonValue>;

    /// Add a variable to the watch list
    fn watch_variable(&self, name: String, batcher: &mut ContextBatcher);

    /// Remove a variable from the watch list
    fn unwatch_variable(&self, name: &str, batcher: &mut ContextBatcher);

    /// Get all cached variables
    fn get_all_cached_variables(&self) -> Vec<CachedVariable>;

    /// Invalidate all cached variables (called when context changes)
    fn invalidate_cache(&self);

    /// Process batched context updates
    fn process_context_updates(&self, updates: Vec<ContextUpdate>);

    /// Validate API usage in script content
    ///
    /// Returns list of validation errors/warnings for script-specific APIs
    ///
    /// # Arguments
    /// * `script` - The script content to validate
    /// * `context` - Shared execution context for validation
    ///
    /// # Returns
    /// * `Ok(Vec<String>)` - List of validation issues (empty if valid)
    ///
    /// # Errors
    /// * Returns error if validation cannot be performed
    fn validate_api_usage(
        &self,
        script: &str,
        context: &crate::execution_context::SharedExecutionContext,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>>;
}

/// Script-specific variable formatter trait
pub trait VariableFormatter {
    /// Format a variable for display using script-specific formatting
    fn format_variable(&self, name: &str, value: &JsonValue) -> String;
}

/// Concrete variable inspector implementation
pub struct SharedVariableInspector {
    /// Debug state cache for variable caching
    cache: Arc<dyn DebugStateCache>,
    /// Shared execution context for variable access
    context: Arc<RwLock<SharedExecutionContext>>,
}

impl SharedVariableInspector {
    /// Create a new variable inspector
    #[must_use]
    pub const fn new(
        cache: Arc<dyn DebugStateCache>,
        context: Arc<RwLock<SharedExecutionContext>>,
    ) -> Self {
        Self { cache, context }
    }

    /// Inspect variables (SLOW PATH ONLY)
    ///
    /// This method batches variable reads for efficiency and uses
    /// cached values when available.
    pub fn inspect_variables(
        &self,
        variable_names: &[String],
        batcher: &mut ContextBatcher,
    ) -> HashMap<String, JsonValue> {
        let mut result = HashMap::new();

        // Check cache and collect uncached names
        let uncached_names = self.check_cache_and_collect(variable_names, &mut result);

        // If all variables were cached, we're done
        if uncached_names.is_empty() {
            debug!("All {} variables found in cache", variable_names.len());
            return result;
        }

        // Read and cache uncached variables
        self.read_and_cache_variables(&uncached_names, batcher, &mut result);

        // Add watched variables
        self.add_watched_variables(&mut result);

        result
    }

    /// Check cache for variables and collect uncached names
    fn check_cache_and_collect(
        &self,
        variable_names: &[String],
        result: &mut HashMap<String, JsonValue>,
    ) -> Vec<String> {
        let mut uncached_names = Vec::new();

        for name in variable_names {
            if let Some(cached_value) = self.cache.get_cached_variable(name) {
                trace!("Variable '{}' found in cache", name);
                result.insert(name.clone(), cached_value);
            } else {
                uncached_names.push(name.clone());
            }
        }

        uncached_names
    }

    /// Read uncached variables and cache them
    fn read_and_cache_variables(
        &self,
        uncached_names: &[String],
        batcher: &mut ContextBatcher,
        result: &mut HashMap<String, JsonValue>,
    ) {
        debug!("Reading {} uncached variables", uncached_names.len());
        batcher.batch_read_variables(uncached_names.to_vec());

        // Read variables from SharedExecutionContext
        let variables = self.read_variables_from_context(uncached_names);

        // Cache the newly read variables
        for (name, value) in &variables {
            self.cache.cache_variable(name.clone(), value.clone());
            batcher.cache_variable(name.clone(), value.clone());
            result.insert(name.clone(), value.clone());
        }
    }

    /// Add watched variables to result if not already present
    fn add_watched_variables(&self, result: &mut HashMap<String, JsonValue>) {
        for watched_name in self.cache.get_watch_list() {
            if let std::collections::hash_map::Entry::Vacant(e) = result.entry(watched_name.clone())
            {
                if let Some(value) = self.read_single_variable(&watched_name) {
                    self.cache
                        .cache_variable(watched_name.clone(), value.clone());
                    e.insert(value);
                }
            }
        }
    }

    /// Read variables from `SharedExecutionContext` (synchronously via `block_on_async`)
    fn read_variables_from_context(&self, names: &[String]) -> HashMap<String, JsonValue> {
        // Use block_on_async pattern from condition_evaluator
        block_on_async(
            "read_variables",
            {
                let context = self.context.clone();
                let names = names.to_vec();
                async move {
                    let ctx = context.read().await;
                    let mut result = HashMap::new();

                    for name in names {
                        if let Some(value) = ctx.variables.get(&name) {
                            result.insert(name, value.clone());
                        }
                    }

                    Ok::<_, std::io::Error>(result)
                }
            },
            None,
        )
        .unwrap_or_else(|e| {
            warn!("Failed to read variables from context: {}", e);
            HashMap::new()
        })
    }

    /// Read a single variable from context
    fn read_single_variable(&self, name: &str) -> Option<JsonValue> {
        block_on_async(
            "read_single_variable",
            {
                let context = self.context.clone();
                let name = name.to_string();
                async move {
                    let ctx = context.read().await;
                    Ok::<_, std::io::Error>(ctx.variables.get(&name).cloned())
                }
            },
            None,
        )
        .unwrap_or_else(|e| {
            warn!("Failed to read variable '{}': {}", name, e);
            None
        })
    }

    /// Add a variable to the watch list
    pub fn watch_variable(&self, name: String, batcher: &mut ContextBatcher) {
        self.cache.add_to_watch_list(name.clone());
        batcher.watch_variable(name);
    }

    /// Remove a variable from the watch list
    pub fn unwatch_variable(&self, name: &str, batcher: &mut ContextBatcher) {
        self.cache.remove_from_watch_list(name);
        batcher.unwatch_variable(name.to_string());
    }

    /// Get all cached variables
    #[must_use]
    pub fn get_all_cached_variables(&self) -> Vec<CachedVariable> {
        self.cache.get_cached_variables()
    }

    /// Invalidate all cached variables (called when context changes)
    pub fn invalidate_cache(&self) {
        self.cache.invalidate_variable_cache();
    }

    /// Process batched context updates
    pub fn process_context_updates(&self, updates: Vec<ContextUpdate>) {
        for update in updates {
            match update {
                ContextUpdate::ReadVariables(names) => {
                    // Variables are read and cached in inspect_variables
                    debug!("Processing batch read for {} variables", names.len());
                }
                ContextUpdate::CacheVariable { name, value } => {
                    // Cache the variable
                    self.cache.cache_variable(name, value);
                }
                ContextUpdate::WatchVariable(name) => {
                    self.cache.add_to_watch_list(name);
                }
                ContextUpdate::UnwatchVariable(name) => {
                    self.cache.remove_from_watch_list(&name);
                }
                _ => {
                    // Other updates handled elsewhere
                }
            }
        }
    }
}

/// Implementation of `VariableInspector` trait for `SharedVariableInspector`
impl VariableInspector for SharedVariableInspector {
    fn inspect_variables(
        &self,
        variable_names: &[String],
        batcher: &mut ContextBatcher,
    ) -> HashMap<String, JsonValue> {
        let mut result = HashMap::new();

        // Check cache and collect uncached names
        let uncached_names = self.check_cache_and_collect(variable_names, &mut result);

        // If all variables were cached, we're done
        if uncached_names.is_empty() {
            debug!("All {} variables found in cache", variable_names.len());
            return result;
        }

        // Read and cache uncached variables
        self.read_and_cache_variables(&uncached_names, batcher, &mut result);

        // Add watched variables
        self.add_watched_variables(&mut result);

        result
    }

    fn watch_variable(&self, name: String, batcher: &mut ContextBatcher) {
        self.cache.add_to_watch_list(name.clone());
        batcher.watch_variable(name);
    }

    fn unwatch_variable(&self, name: &str, batcher: &mut ContextBatcher) {
        self.cache.remove_from_watch_list(name);
        batcher.unwatch_variable(name.to_string());
    }

    fn get_all_cached_variables(&self) -> Vec<CachedVariable> {
        self.cache.get_cached_variables()
    }

    fn invalidate_cache(&self) {
        self.cache.invalidate_variable_cache();
    }

    fn process_context_updates(&self, updates: Vec<ContextUpdate>) {
        for update in updates {
            match update {
                ContextUpdate::ReadVariables(names) => {
                    // Variables are read and cached in inspect_variables
                    debug!("Processing batch read for {} variables", names.len());
                }
                ContextUpdate::CacheVariable { name, value } => {
                    // Cache the variable
                    self.cache.cache_variable(name, value);
                }
                ContextUpdate::WatchVariable(name) => {
                    self.cache.add_to_watch_list(name);
                }
                ContextUpdate::UnwatchVariable(name) => {
                    self.cache.remove_from_watch_list(&name);
                }
                _ => {
                    // Other updates handled elsewhere
                }
            }
        }
    }

    fn validate_api_usage(
        &self,
        script: &str,
        _context: &crate::execution_context::SharedExecutionContext,
    ) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let mut issues = Vec::new();

        // Basic script-agnostic validation
        // Check for common unsafe patterns
        if script.contains("os.execute") || script.contains("io.popen") {
            issues.push("Potentially unsafe system command execution detected".to_string());
        }

        if script.contains("loadstring") || script.contains("load(") {
            issues.push("Dynamic code execution detected - review for security".to_string());
        }

        // Check for undefined variable patterns (basic heuristic)
        let lines: Vec<&str> = script.lines().collect();
        for (line_num, line) in lines.iter().enumerate() {
            if line.trim_start().starts_with("local ") && line.contains(" = nil") {
                issues.push(format!(
                    "Line {}: Variable initialized to nil",
                    line_num + 1
                ));
            }
        }

        Ok(issues)
    }
}

/// Context update batcher for lazy updates
pub struct ContextBatcher {
    /// Pending updates to batch
    updates: Vec<ContextUpdate>,
    /// Last flush time
    last_flush: Instant,
    /// How often to flush updates
    flush_interval: Duration,
    /// Maximum updates before forced flush
    max_batch_size: usize,
}

/// Types of context updates
#[derive(Debug, Clone)]
pub enum ContextUpdate {
    /// Location update
    Location { source: String, line: u32 },
    /// Performance metric
    ExecutionCount(u64),
    /// Stack frame push
    StackPush {
        name: String,
        source: String,
        line: u32,
    },
    /// Stack frame pop
    StackPop,
    /// Read multiple variables (batched for efficiency)
    ReadVariables(Vec<String>),
    /// Cache a variable value
    CacheVariable {
        name: String,
        value: serde_json::Value,
    },
    /// Add variable to watch list
    WatchVariable(String),
    /// Remove variable from watch list
    UnwatchVariable(String),
}

impl ContextBatcher {
    /// Create a new context batcher
    #[must_use]
    pub fn new() -> Self {
        Self {
            updates: Vec::with_capacity(100),
            last_flush: Instant::now(),
            flush_interval: Duration::from_millis(100),
            max_batch_size: 100,
        }
    }

    /// Record a location update (doesn't flush immediately)
    pub fn record_location(&mut self, source: String, line: u32) {
        self.updates.push(ContextUpdate::Location { source, line });
        self.maybe_flush();
    }

    /// Record an execution count update
    pub fn record_execution_count(&mut self, count: u64) {
        self.updates.push(ContextUpdate::ExecutionCount(count));
        self.maybe_flush();
    }

    /// Record a stack push
    pub fn record_stack_push(&mut self, name: String, source: String, line: u32) {
        self.updates
            .push(ContextUpdate::StackPush { name, source, line });
        self.maybe_flush();
    }

    /// Record a stack pop
    pub fn record_stack_pop(&mut self) {
        self.updates.push(ContextUpdate::StackPop);
        self.maybe_flush();
    }

    /// Batch read multiple variables (slow path only)
    pub fn batch_read_variables(&mut self, names: Vec<String>) {
        if !names.is_empty() {
            self.updates.push(ContextUpdate::ReadVariables(names));
            self.maybe_flush();
        }
    }

    /// Cache a variable value for fast access
    pub fn cache_variable(&mut self, name: String, value: serde_json::Value) {
        self.updates
            .push(ContextUpdate::CacheVariable { name, value });
        self.maybe_flush();
    }

    /// Add a variable to the watch list
    pub fn watch_variable(&mut self, name: String) {
        self.updates.push(ContextUpdate::WatchVariable(name));
        self.maybe_flush();
    }

    /// Remove a variable from the watch list
    pub fn unwatch_variable(&mut self, name: String) {
        self.updates.push(ContextUpdate::UnwatchVariable(name));
        self.maybe_flush();
    }

    /// Check if we should flush
    fn should_flush(&self) -> bool {
        self.updates.len() >= self.max_batch_size
            || self.last_flush.elapsed() >= self.flush_interval
    }

    /// Maybe flush if conditions are met
    fn maybe_flush(&mut self) {
        if self.should_flush() {
            self.flush();
        }
    }

    /// Force flush all pending updates
    pub fn flush(&mut self) -> Vec<ContextUpdate> {
        self.last_flush = Instant::now();
        std::mem::take(&mut self.updates)
    }

    /// Get pending update count
    #[must_use]
    pub const fn pending_count(&self) -> usize {
        self.updates.len()
    }
}

impl Default for ContextBatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution_context::SharedExecutionContext;
    use crate::lua::debug_state_cache_impl::LuaDebugStateCache;

    #[test]
    fn test_variable_cache_operations() {
        let cache = Arc::new(LuaDebugStateCache::new());
        let context = Arc::new(RwLock::new(SharedExecutionContext::new()));
        let inspector = SharedVariableInspector::new(cache.clone(), context);

        // Test caching
        cache.cache_variable("test_var".to_string(), JsonValue::from(42));

        // Test retrieval
        assert_eq!(
            cache.get_cached_variable("test_var"),
            Some(JsonValue::from(42))
        );

        // Test watch list
        inspector.watch_variable("important_var".to_string(), &mut ContextBatcher::new());
        assert!(cache.is_watched("important_var"));

        // Test invalidation
        inspector.invalidate_cache();
        assert_eq!(cache.get_cached_variable("test_var"), None);
    }

    #[test]
    fn test_shared_variable_inspector_interface() {
        let cache = Arc::new(LuaDebugStateCache::new());
        let context = Arc::new(RwLock::new(SharedExecutionContext::new()));
        let inspector = SharedVariableInspector::new(cache, context);

        // Test that trait implementation works
        let mut batcher = ContextBatcher::new();
        let vars = inspector.inspect_variables(&[], &mut batcher);
        assert!(vars.is_empty()); // No variables to inspect
    }
}
