//! Variable inspection system for slow path debugging
//!
//! This module provides variable inspection that operates entirely in the slow path,
//! leveraging cached variables from `ContextBatcher` and existing formatting from `output.rs`.

use crate::execution_context::SharedExecutionContext;
use crate::lua::debug_cache::{CachedVariable, ContextBatcher, ContextUpdate, DebugStateCache};
use crate::lua::output::{dump_value, format_simple, DumpOptions};
use crate::lua::sync_utils::block_on_async;
use mlua::{Lua, Value};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, trace, warn};

/// Variable inspector for slow path operations
pub struct VariableInspector {
    /// Debug state cache for variable caching
    cache: Arc<DebugStateCache>,
    /// Shared execution context for variable access
    context: Arc<RwLock<SharedExecutionContext>>,
}

impl VariableInspector {
    /// Create a new variable inspector
    #[must_use]
    pub const fn new(
        cache: Arc<DebugStateCache>,
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

    /// Format a variable for display using output.rs functions
    pub fn format_variable(&self, name: &str, value: &JsonValue, lua: &Lua) -> String {
        // Convert JSON to Lua value for formatting
        match json_to_lua_value(lua, value) {
            Ok(lua_value) => {
                // Use existing output.rs formatting
                if Self::should_use_compact_format(value) {
                    format!("{}: {}", name, format_simple(&lua_value))
                } else {
                    let options = DumpOptions::default();
                    format!("{}:\n{}", name, dump_value(&lua_value, &options))
                }
            }
            Err(e) => {
                warn!("Failed to convert variable '{}' to Lua value: {}", name, e);
                format!("{name}: <error: {e}>")
            }
        }
    }

    /// Determine if compact format should be used
    const fn should_use_compact_format(value: &JsonValue) -> bool {
        matches!(
            value,
            JsonValue::Null | JsonValue::Bool(_) | JsonValue::Number(_) | JsonValue::String(_)
        )
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

/// Convert JSON value to Lua value
fn json_to_lua_value<'lua>(lua: &'lua Lua, json: &JsonValue) -> mlua::Result<Value<'lua>> {
    match json {
        JsonValue::Null => Ok(Value::Nil),
        JsonValue::Bool(b) => Ok(Value::Boolean(*b)),
        JsonValue::Number(n) => Ok(n
            .as_i64()
            .map(Value::Integer)
            .or_else(|| n.as_f64().map(Value::Number))
            .unwrap_or(Value::Nil)),
        JsonValue::String(s) => Ok(Value::String(lua.create_string(s)?)),
        JsonValue::Array(arr) => {
            let table = lua.create_table()?;
            for (i, v) in arr.iter().enumerate() {
                table.set(i + 1, json_to_lua_value(lua, v)?)?;
            }
            Ok(Value::Table(table))
        }
        JsonValue::Object(obj) => {
            let table = lua.create_table()?;
            for (k, v) in obj {
                table.set(k.as_str(), json_to_lua_value(lua, v)?)?;
            }
            Ok(Value::Table(table))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::execution_context::SharedExecutionContext;

    #[test]
    fn test_variable_cache_operations() {
        let cache = Arc::new(DebugStateCache::new());
        let context = Arc::new(RwLock::new(SharedExecutionContext::new()));
        let inspector = VariableInspector::new(cache.clone(), context);

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
    fn test_json_to_lua_conversion() {
        let lua = Lua::new();

        // Test null
        let json = JsonValue::Null;
        let lua_val = json_to_lua_value(&lua, &json).unwrap();
        assert_eq!(lua_val, Value::Nil);

        // Test boolean
        let json = JsonValue::Bool(true);
        let lua_val = json_to_lua_value(&lua, &json).unwrap();
        assert_eq!(lua_val, Value::Boolean(true));

        // Test number
        let json = JsonValue::from(42);
        let lua_val = json_to_lua_value(&lua, &json).unwrap();
        assert_eq!(lua_val, Value::Integer(42));

        // Test string
        let json = JsonValue::String("hello".to_string());
        let lua_val = json_to_lua_value(&lua, &json).unwrap();
        match lua_val {
            Value::String(s) => assert_eq!(s.to_str().unwrap(), "hello"),
            _ => panic!("Expected string"),
        }
    }
}
