//! Fast synchronous debug state cache for hot path optimization
//!
//! Provides zero-cost debugging abstractions by keeping hot path synchronous
//! and only using async operations when breakpoints might actually hit.

use dashmap::DashMap;
use parking_lot::RwLock;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

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

/// Compiled condition for fast evaluation
#[derive(Debug, Clone)]
pub struct CompiledCondition {
    /// Original condition expression
    pub expression: String,
    /// Pre-compiled Lua chunk (if using Lua for evaluation)
    pub compiled_chunk: Option<Vec<u8>>,
}

/// Fast synchronous cache for debug state
pub struct DebugStateCache {
    /// Whether any debugging is active (fast atomic check)
    debug_active: AtomicBool,
    /// Current debug mode
    debug_mode: RwLock<DebugMode>,
    /// Breakpoint lines by source file (using `HashSet` for O(1) lookup)
    breakpoint_lines: Arc<RwLock<HashMap<String, HashSet<u32>>>>,
    /// Breakpoint conditions (lockless concurrent access)
    breakpoint_conditions: Arc<DashMap<(String, u32), Arc<CompiledCondition>>>,
    /// Condition evaluation cache: (result, generation when cached)
    condition_cache: Arc<DashMap<(String, u32), (bool, u64)>>,
    /// Cache generation for invalidation
    generation: AtomicU64,
    /// Recent hot locations (for performance monitoring)
    hot_locations: Arc<RwLock<Vec<(String, u32, Instant)>>>,
    /// Maximum hot locations to track
    max_hot_locations: usize,
}

impl DebugStateCache {
    /// Create a new debug state cache
    #[must_use]
    pub fn new() -> Self {
        Self {
            debug_active: AtomicBool::new(false),
            debug_mode: RwLock::new(DebugMode::Disabled),
            breakpoint_lines: Arc::new(RwLock::new(HashMap::new())),
            breakpoint_conditions: Arc::new(DashMap::new()),
            condition_cache: Arc::new(DashMap::new()),
            generation: AtomicU64::new(0),
            hot_locations: Arc::new(RwLock::new(Vec::with_capacity(100))),
            max_hot_locations: 100,
        }
    }

    /// Check if debugging is active at all (fastest check)
    #[inline]
    pub fn is_debug_active(&self) -> bool {
        self.debug_active.load(Ordering::Relaxed)
    }

    /// Check if we might break at a location (fast path)
    ///
    /// This is the hot path - must be as fast as possible
    #[inline]
    pub fn might_break_at(&self, source: &str, line: u32) -> bool {
        // Fast atomic check first - most common case
        if !self.debug_active.load(Ordering::Relaxed) {
            return false;
        }

        // Check if this source file has any breakpoints
        // Using parking_lot RwLock for better performance than std
        self.breakpoint_lines
            .read()
            .get(source)
            .is_some_and(|lines| lines.contains(&line))
    }

    /// Update breakpoints from execution manager (called rarely)
    pub fn update_breakpoints(&self, breakpoints: Vec<(String, u32)>) {
        let mut map = HashMap::new();

        for (source, line) in breakpoints {
            map.entry(source).or_insert_with(HashSet::new).insert(line);
        }

        let is_empty = map.is_empty();
        *self.breakpoint_lines.write() = map;
        self.debug_active.store(!is_empty, Ordering::Relaxed);
        self.generation.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a hot location (for performance monitoring)
    pub fn record_hot_location(&self, source: String, line: u32) {
        let mut locations = self.hot_locations.write();

        // Keep only recent locations
        if locations.len() >= self.max_hot_locations {
            locations.drain(0..self.max_hot_locations / 2);
        }

        locations.push((source, line, Instant::now()));
    }

    /// Check if breakpoint has a condition (FAST PATH - lockless read)
    #[inline]
    pub fn has_condition(&self, source: &str, line: u32) -> bool {
        self.breakpoint_conditions
            .contains_key(&(source.to_string(), line))
    }

    /// Set a compiled condition for a breakpoint
    pub fn set_condition(&self, source: String, line: u32, condition: CompiledCondition) {
        self.breakpoint_conditions
            .insert((source, line), Arc::new(condition));
        // Invalidate cache when condition is set/changed
        self.generation.fetch_add(1, Ordering::Relaxed);
    }

    /// Remove condition for a breakpoint
    pub fn remove_condition(&self, source: &str, line: u32) {
        self.breakpoint_conditions
            .remove(&(source.to_string(), line));
        self.condition_cache.remove(&(source.to_string(), line));
        self.generation.fetch_add(1, Ordering::Relaxed);
    }

    /// Get cached condition result (SLOW PATH)
    pub fn get_cached_condition(&self, source: &str, line: u32) -> Option<(bool, u64)> {
        self.condition_cache
            .get(&(source.to_string(), line))
            .map(|entry| *entry)
    }

    /// Cache condition evaluation result (SLOW PATH)
    pub fn cache_condition_result(&self, source: &str, line: u32, result: bool) {
        let generation = self.generation.load(Ordering::Relaxed);
        self.condition_cache
            .insert((source.to_string(), line), (result, generation));
    }

    /// Get compiled condition for evaluation (SLOW PATH)
    pub fn get_condition(&self, source: &str, line: u32) -> Option<Arc<CompiledCondition>> {
        self.breakpoint_conditions
            .get(&(source.to_string(), line))
            .map(|entry| entry.clone())
    }

    /// Invalidate all condition caches (called on variable changes)
    pub fn invalidate_condition_cache(&self) {
        self.condition_cache.clear();
        self.generation.fetch_add(1, Ordering::Relaxed);
    }

    /// Get current debug mode
    pub fn get_debug_mode(&self) -> DebugMode {
        *self.debug_mode.read()
    }

    /// Set debug mode
    pub fn set_debug_mode(&self, mode: DebugMode) {
        *self.debug_mode.write() = mode;

        // Update active flag based on mode
        let active = !matches!(mode, DebugMode::Disabled);
        self.debug_active.store(active, Ordering::Relaxed);
    }

    /// Clear all cached state
    pub fn clear(&self) {
        self.breakpoint_lines.write().clear();
        self.breakpoint_conditions.clear();
        self.condition_cache.clear();
        self.hot_locations.write().clear();
        self.debug_active.store(false, Ordering::Relaxed);
        *self.debug_mode.write() = DebugMode::Disabled;
        self.generation.fetch_add(1, Ordering::Relaxed);
    }

    /// Get cache generation (for invalidation checks)
    pub fn generation(&self) -> u64 {
        self.generation.load(Ordering::Relaxed)
    }
}

impl Default for DebugStateCache {
    fn default() -> Self {
        Self::new()
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

    #[test]
    fn test_debug_cache_fast_path() {
        let cache = DebugStateCache::new();

        // No debugging active - should be fast
        assert!(!cache.might_break_at("test.lua", 10));

        // Add a breakpoint
        cache.update_breakpoints(vec![("test.lua".to_string(), 10)]);

        // Should detect breakpoint
        assert!(cache.might_break_at("test.lua", 10));
        assert!(!cache.might_break_at("test.lua", 11));
        assert!(!cache.might_break_at("other.lua", 10));
    }

    #[test]
    fn test_debug_mode_switching() {
        let cache = DebugStateCache::new();

        assert_eq!(cache.get_debug_mode(), DebugMode::Disabled);
        assert!(!cache.is_debug_active());

        cache.set_debug_mode(DebugMode::Full);
        assert_eq!(cache.get_debug_mode(), DebugMode::Full);
        assert!(cache.is_debug_active());

        cache.set_debug_mode(DebugMode::Minimal {
            check_interval: 100,
        });
        assert!(matches!(cache.get_debug_mode(), DebugMode::Minimal { .. }));
        assert!(cache.is_debug_active());
    }

    #[test]
    fn test_context_batcher() {
        let mut batcher = ContextBatcher::new();

        // Add updates
        batcher.record_location("test.lua".to_string(), 1);
        batcher.record_location("test.lua".to_string(), 2);

        assert_eq!(batcher.pending_count(), 2);

        // Force flush
        let updates = batcher.flush();
        assert_eq!(updates.len(), 2);
        assert_eq!(batcher.pending_count(), 0);
    }
}
