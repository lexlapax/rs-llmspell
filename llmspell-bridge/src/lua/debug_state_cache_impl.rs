//! Lua-specific implementation of `DebugStateCache`
//!
//! This module provides the Lua-specific implementation of debug state caching,
//! including evaluation of watch expressions using mlua and `LuaConditionEvaluator`.
//!
//! Follows the three-layer bridge architecture:
//! - Bridge Layer: `DebugStateCache` trait (script-agnostic)
//! - Shared Layer: `SharedDebugStateCache` (common implementation)
//! - Script Layer: `LuaDebugStateCache` (Lua-specific evaluation)

use crate::condition_evaluator::{CompiledCondition, DebugContext};
use crate::debug_state_cache::{
    CachedVariable, DebugMode, DebugStateCache, SharedDebugStateCache, StepMode,
};
use crate::lua::condition_evaluator_impl::LuaConditionEvaluator;
use mlua::Lua;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Arc;

/// Lua-specific implementation of `DebugStateCache`
///
/// This struct provides Lua-specific functionality for debug state caching,
/// including evaluation of watch expressions using mlua and `LuaConditionEvaluator`.
/// All script-agnostic operations are delegated to the shared implementation.
pub struct LuaDebugStateCache {
    /// Shared implementation for script-agnostic operations
    shared: SharedDebugStateCache,
}

impl LuaDebugStateCache {
    /// Create a new Lua debug state cache
    #[must_use]
    pub fn new() -> Self {
        Self {
            shared: SharedDebugStateCache::new(),
        }
    }

    /// Evaluate all watch expressions using `LuaConditionEvaluator` (slow path only)
    ///
    /// This method evaluates all watch expressions in a batch and caches the results
    /// with the current generation for invalidation.
    ///
    /// # Arguments
    /// * `lua` - The Lua instance for evaluation
    /// * `context` - The debug context providing variables and location
    /// * `evaluator` - The `LuaConditionEvaluator` for expression evaluation
    ///
    /// # Returns
    /// * `HashMap` mapping expressions to their evaluated results
    pub fn evaluate_watches_with_lua(
        &self,
        lua: &Lua,
        context: &dyn DebugContext,
        evaluator: &LuaConditionEvaluator,
    ) -> HashMap<String, String> {
        let watches = self.shared.get_watch_expressions();
        let mut results = HashMap::new();

        // Evaluate each watch expression
        for watch_expr in watches {
            // Check cache first
            if let Some(cached_result) = self.shared.get_watch_result(&watch_expr) {
                results.insert(watch_expr, cached_result);
                continue;
            }

            // Evaluate the expression as a condition
            let result = evaluator
                .evaluate_condition_with_lua(&watch_expr, None, context, lua)
                .map_or_else(
                    |e| format!("<error: {e}>"),
                    |v| {
                        if v {
                            "true".to_string()
                        } else {
                            "false".to_string()
                        }
                    },
                );

            results.insert(watch_expr.clone(), result.clone());

            // Cache the result with current generation
            let gen = self.shared.generation();
            self.shared.cache_watch_result(watch_expr, result, gen);
        }

        results
    }
}

impl Default for LuaDebugStateCache {
    fn default() -> Self {
        Self::new()
    }
}

// Implement the trait by delegating all methods to the shared implementation
impl DebugStateCache for LuaDebugStateCache {
    // ===== Core Debug State =====

    fn is_debug_active(&self) -> bool {
        self.shared.is_debug_active()
    }

    fn might_break_at(&self, source: &str, line: u32) -> bool {
        self.shared.might_break_at(source, line)
    }

    fn update_breakpoints(&self, breakpoints: Vec<(String, u32)>) {
        self.shared.update_breakpoints(breakpoints);
    }

    fn record_hot_location(&self, source: String, line: u32) {
        self.shared.record_hot_location(source, line);
    }

    fn get_debug_mode(&self) -> DebugMode {
        self.shared.get_debug_mode()
    }

    fn set_debug_mode(&self, mode: DebugMode) {
        self.shared.set_debug_mode(mode);
    }

    fn clear(&self) {
        self.shared.clear();
    }

    // ===== Condition Support =====

    fn has_condition(&self, source: &str, line: u32) -> bool {
        self.shared.has_condition(source, line)
    }

    fn set_condition(&self, source: String, line: u32, condition: CompiledCondition) {
        self.shared.set_condition(source, line, condition);
    }

    fn remove_condition(&self, source: &str, line: u32) {
        self.shared.remove_condition(source, line);
    }

    fn get_cached_condition(&self, source: &str, line: u32) -> Option<(bool, u64)> {
        self.shared.get_cached_condition(source, line)
    }

    fn cache_condition_result(&self, source: &str, line: u32, result: bool) {
        self.shared.cache_condition_result(source, line, result);
    }

    fn get_condition(&self, source: &str, line: u32) -> Option<Arc<CompiledCondition>> {
        self.shared.get_condition(source, line)
    }

    fn invalidate_condition_cache(&self) {
        self.shared.invalidate_condition_cache();
    }

    // ===== Step Debugging =====

    fn is_stepping(&self) -> bool {
        self.shared.is_stepping()
    }

    fn start_stepping(&self, mode: StepMode, current_mode: DebugMode) {
        self.shared.start_stepping(mode, current_mode);
    }

    fn stop_stepping(&self) -> Option<DebugMode> {
        self.shared.stop_stepping()
    }

    fn get_step_mode(&self) -> StepMode {
        self.shared.get_step_mode()
    }

    fn get_saved_mode(&self) -> Option<DebugMode> {
        self.shared.get_saved_mode()
    }

    fn set_current_depth(&self, depth: i32) {
        self.shared.set_current_depth(depth);
    }

    fn get_current_depth(&self) -> i32 {
        self.shared.get_current_depth()
    }

    // ===== Variable Caching =====

    fn cache_variable(&self, name: String, value: JsonValue) {
        self.shared.cache_variable(name, value);
    }

    fn get_cached_variable(&self, name: &str) -> Option<JsonValue> {
        self.shared.get_cached_variable(name)
    }

    fn get_cached_variables(&self) -> Vec<CachedVariable> {
        self.shared.get_cached_variables()
    }

    fn add_to_watch_list(&self, name: String) {
        self.shared.add_to_watch_list(name);
    }

    fn remove_from_watch_list(&self, name: &str) {
        self.shared.remove_from_watch_list(name);
    }

    fn get_watch_list(&self) -> Vec<String> {
        self.shared.get_watch_list()
    }

    fn is_watched(&self, name: &str) -> bool {
        self.shared.is_watched(name)
    }

    fn invalidate_variable_cache(&self) {
        self.shared.invalidate_variable_cache();
    }

    // ===== Watch Expressions (Script-Agnostic Storage) =====

    fn add_watch(&self, expr: String) -> String {
        self.shared.add_watch(expr)
    }

    fn remove_watch(&self, expr: &str) -> bool {
        self.shared.remove_watch(expr)
    }

    fn get_watch_expressions(&self) -> Vec<String> {
        self.shared.get_watch_expressions()
    }

    fn get_watch_result(&self, expr: &str) -> Option<String> {
        self.shared.get_watch_result(expr)
    }

    fn get_all_watch_results(&self) -> HashMap<String, String> {
        self.shared.get_all_watch_results()
    }

    // ===== Performance Monitoring =====

    fn generation(&self) -> u64 {
        self.shared.generation()
    }

    // ===== Stack Navigation =====

    fn get_current_frame_index(&self) -> usize {
        self.shared.get_current_frame_index()
    }

    fn set_current_frame_index(&self, index: usize) {
        self.shared.set_current_frame_index(index);
    }
}
