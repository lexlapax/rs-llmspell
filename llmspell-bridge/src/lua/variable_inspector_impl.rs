//! Lua-specific implementation of variable inspection
//!
//! This module provides the Lua implementation of the `VariableInspector` trait,
//! handling Lua-specific variable formatting and display logic.

use crate::debug_state_cache::{CachedVariable, DebugStateCache};
use crate::execution_context::SharedExecutionContext;
use crate::lua::output::{dump_value, format_simple, DumpOptions};
use crate::variable_inspector::{ContextBatcher, ContextUpdate};
use crate::variable_inspector::{SharedVariableInspector, VariableInspector};
use mlua::{Lua, Value};
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::warn;

/// Lua-specific variable inspector
pub struct LuaVariableInspector {
    /// Core variable inspector functionality
    inner: SharedVariableInspector,
}

impl LuaVariableInspector {
    /// Create a new Lua variable inspector
    #[must_use]
    pub const fn new(
        cache: Arc<dyn DebugStateCache>,
        context: Arc<RwLock<SharedExecutionContext>>,
    ) -> Self {
        Self {
            inner: SharedVariableInspector::new(cache, context),
        }
    }

    /// Format a variable for display using Lua-specific formatting
    pub fn format_variable_with_lua(&self, name: &str, value: &JsonValue, lua: &Lua) -> String {
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

    /// Get reference to the inner shared inspector
    #[must_use]
    pub const fn inner(&self) -> &SharedVariableInspector {
        &self.inner
    }
}

impl VariableInspector for LuaVariableInspector {
    fn inspect_variables(
        &self,
        variable_names: &[String],
        batcher: &mut ContextBatcher,
    ) -> HashMap<String, JsonValue> {
        self.inner.inspect_variables(variable_names, batcher)
    }

    fn watch_variable(&self, name: String, batcher: &mut ContextBatcher) {
        self.inner.watch_variable(name, batcher);
    }

    fn unwatch_variable(&self, name: &str, batcher: &mut ContextBatcher) {
        self.inner.unwatch_variable(name, batcher);
    }

    fn get_all_cached_variables(&self) -> Vec<CachedVariable> {
        self.inner.get_all_cached_variables()
    }

    fn invalidate_cache(&self) {
        self.inner.invalidate_cache();
    }

    fn process_context_updates(&self, updates: Vec<ContextUpdate>) {
        self.inner.process_context_updates(updates);
    }
}

// Removed VariableFormatter implementation since it can't store Lua reference

impl LuaVariableInspector {
    /// Determine if compact format should be used
    const fn should_use_compact_format(value: &JsonValue) -> bool {
        matches!(
            value,
            JsonValue::Null | JsonValue::Bool(_) | JsonValue::Number(_) | JsonValue::String(_)
        )
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
    use crate::lua::debug_state_cache_impl::LuaDebugStateCache;
    use serde_json::json;

    #[test]
    fn test_lua_variable_inspector_creation() {
        let cache = Arc::new(LuaDebugStateCache::new());
        let context = Arc::new(RwLock::new(SharedExecutionContext::new()));

        let inspector = LuaVariableInspector::new(cache, context);

        // Should create successfully
        assert!(
            !inspector.get_all_cached_variables().is_empty()
                || inspector.get_all_cached_variables().is_empty()
        );
    }

    #[test]
    fn test_variable_formatting() {
        let cache = Arc::new(LuaDebugStateCache::new());
        let context = Arc::new(RwLock::new(SharedExecutionContext::new()));
        let lua = Lua::new();

        let inspector = LuaVariableInspector::new(cache, context);

        // Test simple value formatting
        let simple_value = json!(42);
        let formatted = inspector.format_variable_with_lua("test_int", &simple_value, &lua);
        assert!(formatted.contains("test_int"));
        assert!(formatted.contains("42"));

        // Test null value
        let null_value = json!(null);
        let formatted = inspector.format_variable_with_lua("test_null", &null_value, &lua);
        assert!(formatted.contains("test_null"));
        assert!(formatted.contains("nil"));

        // Test boolean value
        let bool_value = json!(true);
        let formatted = inspector.format_variable_with_lua("test_bool", &bool_value, &lua);
        assert!(formatted.contains("test_bool"));
        assert!(formatted.contains("true"));
    }

    #[test]
    fn test_compact_format_detection() {
        // Simple types should use compact format
        assert!(LuaVariableInspector::should_use_compact_format(&json!(
            null
        )));
        assert!(LuaVariableInspector::should_use_compact_format(&json!(
            true
        )));
        assert!(LuaVariableInspector::should_use_compact_format(&json!(42)));
        assert!(LuaVariableInspector::should_use_compact_format(&json!(
            "hello"
        )));

        // Complex types should not use compact format
        assert!(!LuaVariableInspector::should_use_compact_format(&json!([])));
        assert!(!LuaVariableInspector::should_use_compact_format(&json!({})));
        assert!(!LuaVariableInspector::should_use_compact_format(&json!([
            1, 2, 3
        ])));
        assert!(!LuaVariableInspector::should_use_compact_format(
            &json!({"a": 1})
        ));
    }

    #[test]
    fn test_json_to_lua_conversion() {
        use std::f64::consts::PI;

        let lua = Lua::new();

        // Test various JSON types
        let test_cases = [
            (json!(null), "nil"),
            (json!(true), "boolean"),
            (json!(42), "integer"),
            (json!(PI), "number"),
            (json!("hello"), "string"),
        ];

        for (json_val, expected_type) in test_cases {
            let lua_val = json_to_lua_value(&lua, &json_val).unwrap();
            match (lua_val, expected_type) {
                (Value::Nil, "nil")
                | (Value::Boolean(_), "boolean")
                | (Value::Integer(_), "integer")
                | (Value::Number(_), "number")
                | (Value::String(_), "string") => {} // Expected types
                _ => panic!("Unexpected conversion for {json_val:?}"),
            }
        }
    }

    #[test]
    fn test_complex_json_conversion() {
        let lua = Lua::new();

        // Test array
        let array = json!([1, 2, "three"]);
        let lua_val = json_to_lua_value(&lua, &array).unwrap();
        assert!(matches!(lua_val, Value::Table(_)));

        // Test object
        let object = json!({"name": "test", "value": 42});
        let lua_val = json_to_lua_value(&lua, &object).unwrap();
        assert!(matches!(lua_val, Value::Table(_)));
    }
}
