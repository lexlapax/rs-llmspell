//! Script-agnostic debug state cache for debugging infrastructure
//!
//! This module provides trait definitions for debug state caching that work
//! across any supported script language. Implementations are provided in the
//! respective language modules (lua/, js/, python/, etc.).

use crate::condition_evaluator::CompiledCondition;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Arc;

/// Debug mode for execution
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugMode {
    /// No debug hooks installed at all
    Disabled,
    /// Minimal overhead - check periodically
    Minimal {
        /// Check every N instructions
        check_interval: u32,
    },
    /// Full line-by-line debugging (only when actively debugging)
    Full,
}

/// Step debugging mode
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StepMode {
    /// Not stepping
    None,
    /// Step into next line (enter functions)
    StepIn { depth: i32 },
    /// Step over next line (skip function calls)
    StepOver { target_depth: i32 },
    /// Step out of current function
    StepOut { target_depth: i32 },
}

/// Variable entry in the cache
#[derive(Debug, Clone)]
pub struct CachedVariable {
    /// Variable name
    pub name: String,
    /// Variable value as JSON
    pub value: JsonValue,
    /// Generation when cached
    pub generation: u64,
    /// Last access time (for LRU eviction)
    pub last_access: std::time::Instant,
}

/// Script-agnostic debug state cache trait
pub trait DebugStateCache: Send + Sync {
    // ===== Core Debug State =====

    /// Check if debugging is active at all (fastest check)
    fn is_debug_active(&self) -> bool;

    /// Check if we might break at a location (fast path)
    fn might_break_at(&self, source: &str, line: u32) -> bool;

    /// Update breakpoints from execution manager (called rarely)
    fn update_breakpoints(&self, breakpoints: Vec<(String, u32)>);

    /// Get current debug mode
    fn get_debug_mode(&self) -> DebugMode;

    /// Set debug mode
    fn set_debug_mode(&self, mode: DebugMode);

    /// Clear all cached state
    fn clear(&self);

    // ===== Condition Support =====

    /// Check if breakpoint has a condition (fast path)
    fn has_condition(&self, source: &str, line: u32) -> bool;

    /// Set a compiled condition for a breakpoint
    fn set_condition(&self, source: String, line: u32, condition: CompiledCondition);

    /// Remove condition for a breakpoint
    fn remove_condition(&self, source: &str, line: u32);

    /// Get cached condition result (slow path)
    fn get_cached_condition(&self, source: &str, line: u32) -> Option<(bool, u64)>;

    /// Cache condition evaluation result (slow path)
    fn cache_condition_result(&self, source: &str, line: u32, result: bool);

    /// Get compiled condition for evaluation (slow path)
    fn get_condition(&self, source: &str, line: u32) -> Option<Arc<CompiledCondition>>;

    /// Invalidate all condition caches (called on variable changes)
    fn invalidate_condition_cache(&self);

    // ===== Step Debugging =====

    /// Check if we're currently stepping (fast path)
    fn is_stepping(&self) -> bool;

    /// Start stepping with mode save (slow path)
    fn start_stepping(&self, mode: StepMode, current_mode: DebugMode);

    /// Stop stepping and get saved mode for restoration
    fn stop_stepping(&self) -> Option<DebugMode>;

    /// Get the current step mode
    fn get_step_mode(&self) -> StepMode;

    /// Get saved debug mode (for restoration)
    fn get_saved_mode(&self) -> Option<DebugMode>;

    /// Update current stack depth for step operations
    fn set_current_depth(&self, depth: i32);

    /// Get current stack depth
    fn get_current_depth(&self) -> i32;

    // ===== Variable Caching =====

    /// Cache a variable (slow path only)
    fn cache_variable(&self, name: String, value: JsonValue);

    /// Get a cached variable (slow path only)
    fn get_cached_variable(&self, name: &str) -> Option<JsonValue>;

    /// Get all cached variables that are still valid
    fn get_cached_variables(&self) -> Vec<CachedVariable>;

    /// Add a variable to the watch list
    fn add_to_watch_list(&self, name: String);

    /// Remove a variable from the watch list
    fn remove_from_watch_list(&self, name: &str);

    /// Get the current watch list
    fn get_watch_list(&self) -> Vec<String>;

    /// Check if a variable is watched
    fn is_watched(&self, name: &str) -> bool;

    /// Invalidate variable cache (called when context changes)
    fn invalidate_variable_cache(&self);

    /// Get cache generation (for invalidation checks)
    fn generation(&self) -> u64;

    // ===== Watch Expressions (Script-Agnostic Storage) =====

    /// Add a watch expression (slow path only)
    fn add_watch(&self, expr: String) -> String;

    /// Remove a watch expression by value (slow path only)
    fn remove_watch(&self, expr: &str) -> bool;

    /// Get all current watch expressions
    fn get_watch_expressions(&self) -> Vec<String>;

    /// Get cached watch result if current generation (slow path only)
    fn get_watch_result(&self, expr: &str) -> Option<String>;

    /// Get all cached watch results that are current
    fn get_all_watch_results(&self) -> HashMap<String, String>;

    // ===== Performance Monitoring =====

    /// Record a hot location (for performance monitoring)
    fn record_hot_location(&self, source: String, line: u32);
}

/// Factory for creating script-specific debug state caches
pub trait DebugStateCacheFactory {
    /// Create a debug state cache for the specified script engine
    fn create_cache(&self) -> Box<dyn DebugStateCache>;
}

/// Shared implementation of `DebugStateCache` for all script engines
pub struct SharedDebugStateCache {
    /// Whether any debugging is active (fast atomic check)
    debug_active: std::sync::atomic::AtomicBool,
    /// Current debug mode
    debug_mode: parking_lot::RwLock<DebugMode>,
    /// Breakpoint lines by source file (using `HashSet` for O(1) lookup)
    breakpoint_lines: Arc<parking_lot::RwLock<HashMap<String, std::collections::HashSet<u32>>>>,
    /// Breakpoint conditions (lockless concurrent access)
    breakpoint_conditions: Arc<dashmap::DashMap<(String, u32), Arc<CompiledCondition>>>,
    /// Condition evaluation cache: (result, generation when cached)
    condition_cache: Arc<dashmap::DashMap<(String, u32), (bool, u64)>>,
    /// Cache generation for invalidation
    generation: std::sync::atomic::AtomicU64,
    /// Recent hot locations (for performance monitoring)
    hot_locations: Arc<parking_lot::RwLock<Vec<(String, u32, std::time::Instant)>>>,
    /// Maximum hot locations to track
    max_hot_locations: usize,
    /// Whether we're currently stepping (fast atomic check)
    is_stepping: std::sync::atomic::AtomicBool,
    /// Current step mode (slow path only)
    step_mode: Arc<parking_lot::RwLock<StepMode>>,
    /// Saved debug mode for restoration after stepping
    saved_debug_mode: Arc<parking_lot::RwLock<Option<DebugMode>>>,
    /// Current stack depth for step operations
    current_depth: Arc<parking_lot::RwLock<i32>>,
    /// Variable cache for frequently accessed variables (slow path only)
    variable_cache: Arc<dashmap::DashMap<String, CachedVariable>>,
    /// Watch list - variables to always cache
    watch_list: Arc<parking_lot::RwLock<std::collections::HashSet<String>>>,
    /// Maximum variables to cache
    max_cached_variables: usize,
    /// Watch expressions for slow path evaluation
    watch_expressions: Arc<parking_lot::RwLock<Vec<String>>>,
    /// Watch expression evaluation results: (result, generation)
    watch_results: Arc<dashmap::DashMap<String, (String, u64)>>,
    /// Next watch ID counter
    next_watch_id: std::sync::atomic::AtomicUsize,
}

impl SharedDebugStateCache {
    /// Create a new shared debug state cache
    #[must_use]
    pub fn new() -> Self {
        Self {
            debug_active: std::sync::atomic::AtomicBool::new(false),
            debug_mode: parking_lot::RwLock::new(DebugMode::Disabled),
            breakpoint_lines: Arc::new(parking_lot::RwLock::new(HashMap::new())),
            breakpoint_conditions: Arc::new(dashmap::DashMap::new()),
            condition_cache: Arc::new(dashmap::DashMap::new()),
            generation: std::sync::atomic::AtomicU64::new(0),
            hot_locations: Arc::new(parking_lot::RwLock::new(Vec::with_capacity(100))),
            max_hot_locations: 100,
            is_stepping: std::sync::atomic::AtomicBool::new(false),
            step_mode: Arc::new(parking_lot::RwLock::new(StepMode::None)),
            saved_debug_mode: Arc::new(parking_lot::RwLock::new(None)),
            current_depth: Arc::new(parking_lot::RwLock::new(0)),
            variable_cache: Arc::new(dashmap::DashMap::new()),
            watch_list: Arc::new(parking_lot::RwLock::new(std::collections::HashSet::new())),
            max_cached_variables: 1000,
            watch_expressions: Arc::new(parking_lot::RwLock::new(Vec::new())),
            watch_results: Arc::new(dashmap::DashMap::new()),
            next_watch_id: std::sync::atomic::AtomicUsize::new(0),
        }
    }

    /// Evict least recently used variables
    fn evict_lru_variables(&self) {
        // Get all entries with their last access times
        let mut entries: Vec<_> = self
            .variable_cache
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().last_access))
            .collect();

        // Sort by last access time (oldest first)
        entries.sort_by_key(|(_, time)| *time);

        // Remove oldest entries (keep watch list variables)
        let evict_count = self.max_cached_variables / 4; // Evict 25% at a time
        let watch_list = self.watch_list.read();
        for (name, _) in entries.iter().take(evict_count) {
            if !watch_list.contains(name) {
                self.variable_cache.remove(name);
            }
        }
    }
}

impl Default for SharedDebugStateCache {
    fn default() -> Self {
        Self::new()
    }
}

impl DebugStateCache for SharedDebugStateCache {
    // Core Debug State
    fn is_debug_active(&self) -> bool {
        self.debug_active.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn might_break_at(&self, source: &str, line: u32) -> bool {
        // Fast atomic check first - most common case
        if !self.debug_active.load(std::sync::atomic::Ordering::Relaxed) {
            return false;
        }

        // Check if this source file has any breakpoints
        self.breakpoint_lines
            .read()
            .get(source)
            .is_some_and(|lines| lines.contains(&line))
    }

    fn update_breakpoints(&self, breakpoints: Vec<(String, u32)>) {
        let mut map = HashMap::new();

        for (source, line) in breakpoints {
            map.entry(source)
                .or_insert_with(std::collections::HashSet::new)
                .insert(line);
        }

        let is_empty = map.is_empty();
        *self.breakpoint_lines.write() = map;
        self.debug_active
            .store(!is_empty, std::sync::atomic::Ordering::Relaxed);
        self.generation
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    fn get_debug_mode(&self) -> DebugMode {
        *self.debug_mode.read()
    }

    fn set_debug_mode(&self, mode: DebugMode) {
        *self.debug_mode.write() = mode;

        // Update active flag based on mode
        let active = !matches!(mode, DebugMode::Disabled);
        self.debug_active
            .store(active, std::sync::atomic::Ordering::Relaxed);
    }

    fn clear(&self) {
        self.breakpoint_lines.write().clear();
        self.breakpoint_conditions.clear();
        self.condition_cache.clear();
        self.hot_locations.write().clear();
        self.debug_active
            .store(false, std::sync::atomic::Ordering::Relaxed);
        *self.debug_mode.write() = DebugMode::Disabled;
        self.generation
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        // Clear stepping state
        self.is_stepping
            .store(false, std::sync::atomic::Ordering::Relaxed);
        *self.step_mode.write() = StepMode::None;
        *self.saved_debug_mode.write() = None;
        *self.current_depth.write() = 0;
        // Clear variable cache
        self.variable_cache.clear();
        self.watch_list.write().clear();
        // Clear watch expressions
        self.watch_expressions.write().clear();
        self.watch_results.clear();
        self.next_watch_id
            .store(0, std::sync::atomic::Ordering::Relaxed);
    }

    // Condition Support
    fn has_condition(&self, source: &str, line: u32) -> bool {
        self.breakpoint_conditions
            .contains_key(&(source.to_string(), line))
    }

    fn set_condition(&self, source: String, line: u32, condition: CompiledCondition) {
        self.breakpoint_conditions
            .insert((source, line), Arc::new(condition));
        // Invalidate cache when condition is set/changed
        self.generation
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    fn remove_condition(&self, source: &str, line: u32) {
        self.breakpoint_conditions
            .remove(&(source.to_string(), line));
        self.condition_cache.remove(&(source.to_string(), line));
        self.generation
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    fn get_cached_condition(&self, source: &str, line: u32) -> Option<(bool, u64)> {
        self.condition_cache
            .get(&(source.to_string(), line))
            .map(|entry| *entry)
    }

    fn cache_condition_result(&self, source: &str, line: u32, result: bool) {
        let generation = self.generation.load(std::sync::atomic::Ordering::Relaxed);
        self.condition_cache
            .insert((source.to_string(), line), (result, generation));
    }

    fn get_condition(&self, source: &str, line: u32) -> Option<Arc<CompiledCondition>> {
        self.breakpoint_conditions
            .get(&(source.to_string(), line))
            .map(|entry| entry.clone())
    }

    fn invalidate_condition_cache(&self) {
        self.condition_cache.clear();
        self.generation
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    // Step Debugging
    fn is_stepping(&self) -> bool {
        self.is_stepping.load(std::sync::atomic::Ordering::Relaxed)
    }

    fn start_stepping(&self, mode: StepMode, current_mode: DebugMode) {
        // Save current mode for restoration
        *self.saved_debug_mode.write() = Some(current_mode);
        // Set the step mode
        *self.step_mode.write() = mode;
        // Mark as stepping
        self.is_stepping
            .store(true, std::sync::atomic::Ordering::Release);
    }

    fn stop_stepping(&self) -> Option<DebugMode> {
        // Stop stepping
        self.is_stepping
            .store(false, std::sync::atomic::Ordering::Release);
        // Reset step mode
        *self.step_mode.write() = StepMode::None;
        // Get and clear saved mode
        self.saved_debug_mode.write().take()
    }

    fn get_step_mode(&self) -> StepMode {
        self.step_mode.read().clone()
    }

    fn get_saved_mode(&self) -> Option<DebugMode> {
        *self.saved_debug_mode.read()
    }

    fn set_current_depth(&self, depth: i32) {
        *self.current_depth.write() = depth;
    }

    fn get_current_depth(&self) -> i32 {
        *self.current_depth.read()
    }

    // Variable Caching
    fn cache_variable(&self, name: String, value: JsonValue) {
        let generation = self.generation.load(std::sync::atomic::Ordering::Relaxed);
        let cached = CachedVariable {
            name: name.clone(),
            value,
            generation,
            last_access: std::time::Instant::now(),
        };

        // Check if we need to evict old entries
        if self.variable_cache.len() >= self.max_cached_variables {
            self.evict_lru_variables();
        }

        self.variable_cache.insert(name, cached);
    }

    fn get_cached_variable(&self, name: &str) -> Option<JsonValue> {
        let generation = self.generation.load(std::sync::atomic::Ordering::Relaxed);
        self.variable_cache.get_mut(name).and_then(|mut entry| {
            // Check if cache is still valid
            if entry.generation == generation {
                entry.last_access = std::time::Instant::now();
                Some(entry.value.clone())
            } else {
                None
            }
        })
    }

    fn get_cached_variables(&self) -> Vec<CachedVariable> {
        let generation = self.generation.load(std::sync::atomic::Ordering::Relaxed);
        self.variable_cache
            .iter()
            .filter(|entry| entry.generation == generation)
            .map(|entry| entry.value().clone())
            .collect()
    }

    fn add_to_watch_list(&self, name: String) {
        self.watch_list.write().insert(name);
    }

    fn remove_from_watch_list(&self, name: &str) {
        self.watch_list.write().remove(name);
    }

    fn get_watch_list(&self) -> Vec<String> {
        self.watch_list.read().iter().cloned().collect()
    }

    fn is_watched(&self, name: &str) -> bool {
        self.watch_list.read().contains(name)
    }

    fn invalidate_variable_cache(&self) {
        // Increment generation to invalidate all cached variables
        self.generation
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    }

    fn generation(&self) -> u64 {
        self.generation.load(std::sync::atomic::Ordering::Relaxed)
    }

    // Watch Expressions (Script-Agnostic Storage)
    fn add_watch(&self, expr: String) -> String {
        let id = format!(
            "watch_{}",
            self.next_watch_id
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        );
        self.watch_expressions.write().push(expr);
        id
    }

    fn remove_watch(&self, expr: &str) -> bool {
        let mut expressions = self.watch_expressions.write();
        expressions
            .iter()
            .position(|e| e == expr)
            .is_some_and(|pos| {
                expressions.remove(pos);
                drop(expressions);
                // Remove cached result
                self.watch_results.remove(expr);
                true
            })
    }

    fn get_watch_expressions(&self) -> Vec<String> {
        self.watch_expressions.read().clone()
    }

    fn get_watch_result(&self, expr: &str) -> Option<String> {
        if let Some(entry) = self.watch_results.get(expr) {
            let current_gen = self.generation.load(std::sync::atomic::Ordering::Relaxed);
            let (result, gen) = entry.value();
            if *gen == current_gen {
                return Some(result.clone());
            }
        }
        None
    }

    fn get_all_watch_results(&self) -> HashMap<String, String> {
        let current_gen = self.generation.load(std::sync::atomic::Ordering::Relaxed);
        self.watch_results
            .iter()
            .filter(|entry| entry.value().1 == current_gen)
            .map(|entry| (entry.key().clone(), entry.value().0.clone()))
            .collect()
    }

    // Performance Monitoring
    fn record_hot_location(&self, source: String, line: u32) {
        let mut locations = self.hot_locations.write();

        // Keep only recent locations
        if locations.len() >= self.max_hot_locations {
            locations.drain(0..self.max_hot_locations / 2);
        }

        locations.push((source, line, std::time::Instant::now()));
    }
}

// Additional methods for SharedDebugStateCache that are not part of the trait
impl SharedDebugStateCache {
    /// Cache watch expression result with generation (slow path only)
    pub fn cache_watch_result(&self, expr: String, result: String, generation: u64) {
        self.watch_results.insert(expr, (result, generation));
    }
}
