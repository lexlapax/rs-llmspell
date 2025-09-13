//! Language-Agnostic Debug Coordinator (Layer 1)
//!
//! The `DebugCoordinator` extracts core debug logic from language-specific hooks,
//! providing a clean abstraction layer that can be shared across Lua, JavaScript,
//! Python and other script engines.
//! As of phase 9.8.9,
//! # Architecture Diagram
//!
//! ```text
//!                   ┌─────────────────────┐
//!                   │   User/REPL/CLI     │
//!                   └──────────┬──────────┘
//!                              │
//!                   ┌──────────▼──────────┐
//!    Layer 1:       │  DebugCoordinator   │  ← Language-agnostic coordinator
//!                   │  (this file)        │    Manages debug state & resume
//!                   │                     │    Synchronizes with ExecutionManager
//!                   └────────┬─┬──────────┘
//!                            │ │
//!                 Breakpoints│ │wait_for_resume()
//!                            │ │
//!                   ┌────────▼─▼──────────┐
//!    Layer 2:       │  ExecutionManager   │  ← Breakpoint storage & checking
//!                   │                     │    Shared between Coordinator & Hooks
//!                   └──────────┬──────────┘
//!                              │
//!                   ┌──────────▼──────────┐
//!    Layer 3:       │ LuaDebugHookAdapter │  ← Implements DebugHook trait
//!                   │                     │    Registers with HookMultiplexer
//!                   └──────────┬──────────┘
//!                              │
//!                   ┌──────────▼──────────┐
//!    Layer 4:       │  HookMultiplexer    │  ← Manages multiple Lua hooks
//!                   │                     │    Priority-based execution
//!                   └──────────┬──────────┘
//!                              │
//!                   ┌──────────▼──────────┐
//!    Layer 5:       │  LuaDebugBridge     │  ← Implements HookHandler trait
//!                   │                     │    Sync/async boundary (block_on_async)
//!                   └──────────┬──────────┘
//!                              │
//!                   ┌──────────▼──────────┐
//!    Layer 6:       │  LuaExecutionHook   │  ← Lua-specific implementation
//!                   │                     │    Fast/slow path optimization
//!                   └─────────────────────┘    Variable extraction
//! ```
//!
//! # Performance Characteristics
//!
//! | Operation | Path | Time | Frequency | Method |
//! |-----------|------|------|-----------|--------|
//! | Hook Dispatch | Fast | <50ns | Every line | `HookMultiplexer` priority sort |
//! | Breakpoint Check | Fast | <100ns | 99% | `might_break_at_sync()` |
//! | Breakpoint Hit | Slow | <10ms | 1% | Full chain through 6 layers |
//! | Variable Extract | Slow | <1ms | On pause | `LuaExecutionHook` via debug API |
//! | Resume Wait | Block | Indefinite | On pause | `wait_for_resume()` blocks |
//! | State Sync | Medium | <100μs | On change | Between Coordinator & `ExecutionManager` |
//!
//! # Communication Examples
//!
//! ## Fast Path (no breakpoint):
//! ```text
//! Lua VM → HookMultiplexer → LuaDebugBridge.handle_event() →
//! Coordinator.might_break_at_sync() → false (sync, <100ns)
//! ```
//!
//! ## Slow Path (breakpoint hit):
//! ```text
//! Lua VM → HookMultiplexer → LuaDebugBridge.handle_event() →
//! LuaExecutionHook.should_break_slow() → ExecutionManager.get_breakpoint_at() →
//! Extract variables → block_on_async(Coordinator.coordinate_breakpoint_pause()) →
//! suspend_for_debugging() → wait_for_resume() [BLOCKS HERE] →
//! User calls resume() → Execution continues
//! ```
//!
//! ## Critical Fix (Task 9.8.9):
//! ```text
//! DebugCoordinator.add_breakpoint() now synchronizes breakpoints to BOTH:
//! 1. DebugCoordinator.breakpoints (for fast path checking)  
//! 2. ExecutionManager.breakpoints (for slow path checking via get_breakpoint_at())
//! ```

use crate::execution_bridge::{
    Breakpoint, DebugState, DebugStepType, ExecutionLocation, ExecutionManager, PauseReason,
    StackFrame,
};
use crate::execution_context::{SharedExecutionContext, SourceLocation};
use llmspell_core::debug::{DebugCapability, DebugRequest, DebugResponse};
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, trace};

/// Map from (`source_file`, `line_number`) to set of breakpoint IDs at that location
type BreakpointLocationMap = HashMap<(String, u32), HashSet<String>>;

/// Language-agnostic debug coordinator
///
/// Provides the core debug logic that can be shared across different script engines.
/// All language-specific bridges delegate to this coordinator for consistent behavior.
pub struct DebugCoordinator {
    /// Shared execution context for cross-language state
    shared_context: Arc<RwLock<SharedExecutionContext>>,

    /// Debug capabilities registry (variable inspector, etc.)
    capabilities: Arc<RwLock<HashMap<String, Arc<dyn DebugCapability>>>>,

    /// Breakpoints indexed by ID for management operations
    breakpoints_by_id: Arc<RwLock<HashMap<String, Breakpoint>>>,

    /// Breakpoint IDs indexed by location for O(1) lookup in hot path
    /// Key: `(source_file, line_number)`, Value: Set of breakpoint IDs at that location
    breakpoints_by_location: Arc<RwLock<BreakpointLocationMap>>,

    /// Set of source files that have at least one breakpoint
    /// Used for fast early-exit in hot path when source has no breakpoints
    sources_with_breakpoints: Arc<RwLock<HashSet<String>>>,

    /// Current debug state
    debug_state: Arc<RwLock<DebugState>>,

    /// Execution manager for delegating debug operations
    execution_manager: Arc<ExecutionManager>,

    /// Atomic flag for ultra-fast check if ANY breakpoints exist
    /// This avoids RwLock acquisition in the common case of no breakpoints
    has_any_breakpoints: Arc<AtomicBool>,
}

impl DebugCoordinator {
    /// Create a new debug coordinator
    #[must_use]
    pub fn new(
        shared_context: Arc<RwLock<SharedExecutionContext>>,
        capabilities: Arc<RwLock<HashMap<String, Arc<dyn DebugCapability>>>>,
        execution_manager: Arc<ExecutionManager>,
    ) -> Self {
        Self {
            shared_context,
            capabilities,
            breakpoints_by_id: Arc::new(RwLock::new(HashMap::new())),
            breakpoints_by_location: Arc::new(RwLock::new(HashMap::new())),
            sources_with_breakpoints: Arc::new(RwLock::new(HashSet::new())),
            debug_state: Arc::new(RwLock::new(DebugState::Running)),
            execution_manager,
            has_any_breakpoints: Arc::new(AtomicBool::new(false)),
        }
    }

    // ========================================
    // FAST PATH METHODS (Sync - No Overhead)
    // ========================================

    /// Check if we might need to break at this location (FAST PATH)
    ///
    /// This is called for every line execution, so it must be extremely fast.
    /// Uses synchronous breakpoint lookup, no async operations.
    ///
    /// # Performance
    /// - Two-level check: first source file, then specific line
    /// - O(1) `HashSet` lookup for source, then O(1) `HashMap` lookup for line
    /// - No string allocations in common case (source not in set)
    /// - <100ns average case performance
    #[must_use]
    pub fn might_break_at_sync(&self, source: &str, line: u32) -> bool {
        // Level 0: Ultra-fast atomic check - are there ANY breakpoints at all?
        if !self.has_any_breakpoints.load(Ordering::Relaxed) {
            return false;
        }

        // Level 1: Super fast check - does this source file have ANY breakpoints?
        let Ok(sources) = self.sources_with_breakpoints.try_read() else {
            return false;
        };

        // Fast path: if this source has no breakpoints at all, exit immediately
        // This avoids string allocation for the HashMap key in 99% of cases
        if !sources.contains(source) {
            return false;
        }

        drop(sources); // Release lock early

        // Level 2: This source has breakpoints, check the specific line
        let Ok(locations) = self.breakpoints_by_location.try_read() else {
            return false;
        };

        // Now we need to build the key, but only for sources that have breakpoints
        let key = (source.to_string(), line);

        // O(1) lookup
        if let Some(bp_ids) = locations.get(&key) {
            if !bp_ids.is_empty() {
                // Found potential breakpoints, now check if any are enabled
                if let Ok(bps) = self.breakpoints_by_id.try_read() {
                    return bp_ids
                        .iter()
                        .any(|id| bps.get(id).is_some_and(|bp| bp.enabled));
                }
            }
        }
        false
    }

    /// Check if we're currently paused (FAST PATH)
    #[must_use]
    pub fn is_paused_sync(&self) -> bool {
        self.debug_state
            .try_read()
            .is_ok_and(|state| matches!(*state, DebugState::Paused { .. }))
    }

    // ========================================
    // SLOW PATH METHODS (Async - Only When Pausing)
    // ========================================

    /// Coordinate breakpoint pause (SLOW PATH - only when actually pausing)
    pub async fn coordinate_breakpoint_pause(
        &self,
        location: ExecutionLocation,
        variables: HashMap<String, serde_json::Value>,
    ) {
        trace!(
            "Coordinating breakpoint pause at {}:{}",
            location.source,
            location.line
        );

        // Update shared context with debugging information
        let mut ctx = self.shared_context.write().await;
        ctx.variables.clone_from(&variables);
        ctx.set_location(SourceLocation {
            source: location.source.clone(),
            line: location.line,
            column: location.column,
        });
        let context = ctx.clone();
        drop(ctx);

        // Delegate to ExecutionManager (preserves existing logic)
        self.execution_manager
            .suspend_for_debugging(location.clone(), context)
            .await;

        // Also update our local state for fast path checks BEFORE waiting
        {
            let mut state = self.debug_state.write().await;
            *state = DebugState::Paused {
                reason: PauseReason::Breakpoint,
                location: location.clone(),
            };
        }

        // Task 9.8.9: Add the missing wait_for_resume() call to actually block execution
        // This completes the debug chain: suspend -> wait -> resume
        self.execution_manager.wait_for_resume().await;

        // Update local state to Running after resume
        {
            let mut state = self.debug_state.write().await;
            *state = DebugState::Running;
        }

        debug!("Breakpoint pause coordinated successfully");
    }

    /// Coordinate step pause (SLOW PATH - only when stepping)
    pub async fn coordinate_step_pause(&self, reason: PauseReason, location: ExecutionLocation) {
        trace!("Coordinating step pause: {:?}", reason);

        // Delegate to ExecutionManager
        self.execution_manager
            .set_state(DebugState::Paused {
                reason: reason.clone(),
                location: location.clone(),
            })
            .await;

        // Also update our local state for fast path checks
        {
            let mut state = self.debug_state.write().await;
            *state = DebugState::Paused { reason, location };
        }

        debug!("Step pause coordinated successfully");
    }

    // ========================================
    // REPL COMMAND DELEGATION
    // ========================================

    /// Add breakpoint
    ///
    /// # Errors
    ///
    /// Returns an error if the breakpoint lock cannot be acquired
    pub async fn add_breakpoint(&self, bp: Breakpoint) -> Result<String, String> {
        trace!(
            "Adding breakpoint through coordinator: {}:{}",
            bp.source,
            bp.line
        );
        let id = bp.id.clone();
        let location_key = (bp.source.clone(), bp.line);

        // Add to ID index
        {
            let mut breakpoints = self.breakpoints_by_id.write().await;
            breakpoints.insert(id.clone(), bp.clone());
        }

        // Add to location index for O(1) lookup
        {
            let mut locations = self.breakpoints_by_location.write().await;
            locations
                .entry(location_key)
                .or_insert_with(HashSet::new)
                .insert(id.clone());
        }

        // Add source to the set of sources with breakpoints
        {
            let mut sources = self.sources_with_breakpoints.write().await;
            sources.insert(bp.source.clone());
        }

        // Update atomic flag - we now have at least one breakpoint
        self.has_any_breakpoints.store(true, Ordering::Relaxed);

        // CRITICAL FIX for Task 9.8.9: Also add to ExecutionManager's collection
        // This ensures get_breakpoint_at() in the slow path can find the breakpoint
        // This was the missing piece preventing breakpoints from actually pausing execution
        self.execution_manager.add_breakpoint(bp).await;

        Ok(id)
    }

    /// Remove breakpoint
    ///
    /// # Errors
    ///
    /// Returns an error if the breakpoint is not found
    pub async fn remove_breakpoint(&self, id: &str) -> Result<(), String> {
        trace!("Removing breakpoint through coordinator: {}", id);

        // Remove from ID index and get the breakpoint info for location cleanup
        let bp_info = {
            let mut breakpoints = self.breakpoints_by_id.write().await;
            breakpoints.remove(id)
        };

        if let Some(bp) = bp_info {
            // Remove from location index
            {
                let mut locations = self.breakpoints_by_location.write().await;
                let location_key = (bp.source.clone(), bp.line);
                if let Some(bp_ids) = locations.get_mut(&location_key) {
                    bp_ids.remove(id);
                    // Clean up empty entries
                    if bp_ids.is_empty() {
                        locations.remove(&location_key);
                    }
                }
            }

            // Check if this source still has any breakpoints
            {
                let still_has_breakpoints = {
                    let locations = self.breakpoints_by_location.read().await;
                    locations.keys().any(|(src, _)| src == &bp.source)
                };

                // If no more breakpoints for this source, remove it from the set
                if !still_has_breakpoints {
                    let mut sources = self.sources_with_breakpoints.write().await;
                    sources.remove(&bp.source);

                    // Check if we have ANY breakpoints left
                    if sources.is_empty() {
                        self.has_any_breakpoints.store(false, Ordering::Relaxed);
                    }
                }
            }

            // Also remove from ExecutionManager's collection to keep them synchronized
            self.execution_manager.remove_breakpoint(id).await;
            Ok(())
        } else {
            Err(format!("Breakpoint {id} not found"))
        }
    }

    /// Resume execution
    pub async fn resume(&self) {
        trace!("Resume through coordinator");
        // Delegate to ExecutionManager
        self.execution_manager.set_state(DebugState::Running).await;

        // Also update our local state
        let mut state = self.debug_state.write().await;
        *state = DebugState::Running;
    }

    /// Step over (delegates to `ExecutionManager`)
    pub async fn step_over(&self) {
        trace!("Step over through coordinator");
        self.execution_manager
            .start_step(DebugStepType::StepOver)
            .await;
    }

    /// Step into (delegates to `ExecutionManager`)
    pub async fn step_into(&self) {
        trace!("Step into through coordinator");
        self.execution_manager
            .start_step(DebugStepType::StepIn)
            .await;
    }

    /// Step out (delegates to `ExecutionManager`)
    pub async fn step_out(&self) {
        trace!("Step out through coordinator");
        self.execution_manager
            .start_step(DebugStepType::StepOut)
            .await;
    }

    /// Get breakpoints
    pub async fn get_breakpoints(&self) -> Vec<Breakpoint> {
        let breakpoints = self.breakpoints_by_id.read().await;
        breakpoints.values().cloned().collect()
    }

    /// Get current execution state
    pub async fn get_debug_state(&self) -> DebugState {
        let state = self.debug_state.read().await;
        state.clone()
    }

    // ========================================
    // STATE QUERY METHODS
    // ========================================

    /// Get current execution position (delegates to `SharedExecutionContext`)
    pub async fn get_current_position(&self) -> Option<ExecutionLocation> {
        let ctx = self.shared_context.read().await;
        ctx.location.as_ref().map(|loc| ExecutionLocation {
            source: loc.source.clone(),
            line: loc.line,
            column: loc.column,
        })
    }

    /// Check if execution is currently paused
    pub async fn is_paused(&self) -> bool {
        let state = self.debug_state.read().await;
        matches!(*state, DebugState::Paused { .. })
    }

    /// Get call stack (delegates to `SharedExecutionContext`)
    pub async fn get_call_stack(&self) -> Vec<StackFrame> {
        let ctx = self.shared_context.read().await;
        ctx.stack.clone()
    }

    /// Inspect local variables (delegates to `SharedExecutionContext`)
    pub async fn inspect_locals(&self) -> HashMap<String, serde_json::Value> {
        let ctx = self.shared_context.read().await;
        ctx.variables.clone()
    }

    /// Process debug capability request (delegates to registered capabilities)
    ///
    /// # Errors
    ///
    /// Returns an error if the requested capability is not registered or if processing fails
    pub async fn process_debug_request(
        &self,
        request: DebugRequest,
    ) -> Result<DebugResponse, String> {
        let capabilities = self.capabilities.read().await;
        let capability_name = request.capability_name();

        if let Some(capability) = capabilities.get(&capability_name) {
            capability
                .process_debug_request(request)
                .await
                .map_err(|e| e.to_string())
        } else {
            Err(format!("No capability registered for: {capability_name}"))
        }
    }

    // ========================================
    // UTILITY METHODS
    // ========================================

    /// Format current debug state for display (optional enhancement)
    pub async fn format_current_state(&self) -> String {
        if let Some(location) = self.get_current_position().await {
            format!("At {}:{}", location.source, location.line)
        } else {
            "No current execution location".to_string()
        }
    }

    /// Get debug coordinator statistics
    #[must_use]
    pub const fn get_statistics() -> DebugCoordinatorStats {
        // Static method - statistics would be tracked via global metrics in production
        DebugCoordinatorStats {
            coordination_calls: 0, // Placeholder - would track from global metrics
        }
    }
}

/// Debug coordinator performance/usage statistics
#[derive(Debug, Clone)]
pub struct DebugCoordinatorStats {
    /// Number of coordination calls made
    pub coordination_calls: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::debug_state_cache::SharedDebugStateCache;

    #[tokio::test]
    async fn test_debug_coordinator_creation() {
        let shared_context = Arc::new(RwLock::new(SharedExecutionContext::new()));
        let capabilities = Arc::new(RwLock::new(HashMap::new()));
        let debug_cache = Arc::new(SharedDebugStateCache::new());
        let execution_manager = Arc::new(ExecutionManager::new(debug_cache));

        let coordinator = DebugCoordinator::new(shared_context, capabilities, execution_manager);

        // Test fast path methods (should not panic)
        assert!(!coordinator.might_break_at_sync("test.lua", 1));
        assert!(!coordinator.is_paused_sync());
    }

    #[tokio::test]
    async fn test_breakpoint_management() {
        let shared_context = Arc::new(RwLock::new(SharedExecutionContext::new()));
        let capabilities = Arc::new(RwLock::new(HashMap::new()));
        let debug_cache = Arc::new(SharedDebugStateCache::new());
        let execution_manager = Arc::new(ExecutionManager::new(debug_cache));

        let coordinator = DebugCoordinator::new(shared_context, capabilities, execution_manager);

        // Test breakpoint management
        let bp = Breakpoint::new("test.lua".to_string(), 10);
        let bp_id = coordinator.add_breakpoint(bp).await.unwrap();

        assert_eq!(coordinator.get_breakpoints().await.len(), 1);
        assert!(coordinator.might_break_at_sync("test.lua", 10));

        coordinator.remove_breakpoint(&bp_id).await.unwrap();
        assert_eq!(coordinator.get_breakpoints().await.len(), 0);
        assert!(!coordinator.might_break_at_sync("test.lua", 10));
    }

    #[tokio::test]
    async fn test_state_management() {
        let shared_context = Arc::new(RwLock::new(SharedExecutionContext::new()));
        let capabilities = Arc::new(RwLock::new(HashMap::new()));
        let debug_cache = Arc::new(SharedDebugStateCache::new());
        let execution_manager = Arc::new(ExecutionManager::new(debug_cache));

        let coordinator = Arc::new(DebugCoordinator::new(
            shared_context,
            capabilities,
            execution_manager,
        ));

        // Initially running
        assert!(!coordinator.is_paused().await);
        assert_eq!(coordinator.get_debug_state().await, DebugState::Running);

        // Test pause coordination - run in separate task to avoid blocking test thread
        let location = ExecutionLocation {
            source: "test.lua".to_string(),
            line: 10,
            column: None,
        };

        let coordinator_clone = coordinator.clone();
        let pause_task = tokio::spawn(async move {
            coordinator_clone
                .coordinate_breakpoint_pause(location, HashMap::new())
                .await;
        });

        // Give the pause task time to start and reach the paused state
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

        // Verify it's actually paused (the fix works!)
        assert!(coordinator.is_paused().await);
        if let DebugState::Paused { reason, .. } = coordinator.get_debug_state().await {
            assert_eq!(reason, PauseReason::Breakpoint);
        } else {
            panic!("Should be paused");
        }

        // Test resume - this unblocks the pause_task
        coordinator.resume().await;

        // Wait for the pause task to complete
        pause_task.await.unwrap();

        // Verify state is back to running
        assert!(!coordinator.is_paused().await);
        assert_eq!(coordinator.get_debug_state().await, DebugState::Running);
    }

    #[tokio::test]
    async fn test_debug_output_formatting() {
        // Task 9.7.5: Verify debug output formatting through all layers
        let shared_context = Arc::new(RwLock::new(SharedExecutionContext::new()));
        let capabilities = Arc::new(RwLock::new(HashMap::new()));
        let debug_cache = Arc::new(SharedDebugStateCache::new());
        let execution_manager = Arc::new(ExecutionManager::new(debug_cache));

        let coordinator = DebugCoordinator::new(
            shared_context.clone(),
            capabilities,
            execution_manager.clone(),
        );

        // Test 1: Basic state formatting
        {
            let mut ctx = shared_context.write().await;
            ctx.set_location(SourceLocation {
                source: "test_format.lua".to_string(),
                line: 100,
                column: Some(15),
            });
        }

        let formatted = coordinator.format_current_state().await;
        assert_eq!(formatted, "At test_format.lua:100");

        // Test 2: No location formatting
        {
            let mut ctx = shared_context.write().await;
            ctx.location = None;
        }

        let formatted = coordinator.format_current_state().await;
        assert_eq!(formatted, "No current execution location");

        // Test 3: Variables are preserved through coordinator
        {
            let mut ctx = shared_context.write().await;
            ctx.variables.insert(
                "test_var".to_string(),
                serde_json::Value::String("formatted_value".to_string()),
            );
            ctx.variables.insert(
                "test_number".to_string(),
                serde_json::Value::Number(serde_json::Number::from(42)),
            );
            ctx.variables
                .insert("test_bool".to_string(), serde_json::Value::Bool(true));
        }

        let locals = coordinator.inspect_locals().await;
        assert_eq!(locals.len(), 3);
        assert_eq!(
            locals.get("test_var").unwrap(),
            &serde_json::Value::String("formatted_value".to_string())
        );
        assert_eq!(
            locals.get("test_number").unwrap(),
            &serde_json::Value::Number(serde_json::Number::from(42))
        );
        assert_eq!(
            locals.get("test_bool").unwrap(),
            &serde_json::Value::Bool(true)
        );

        // Test 4: Stack formatting is preserved
        {
            let mut ctx = shared_context.write().await;
            ctx.push_frame(StackFrame {
                id: "frame1".to_string(),
                name: "main".to_string(),
                source: "test_format.lua".to_string(),
                line: 10,
                column: None,
                locals: vec![],
                is_user_code: true,
            });
            ctx.push_frame(StackFrame {
                id: "frame2".to_string(),
                name: "helper_function".to_string(),
                source: "test_format.lua".to_string(),
                line: 50,
                column: Some(8),
                locals: vec![],
                is_user_code: true,
            });
        }

        let stack = coordinator.get_call_stack().await;
        assert_eq!(stack.len(), 2);
        assert_eq!(stack[0].name, "main");
        assert_eq!(stack[0].line, 10);
        assert_eq!(stack[1].name, "helper_function");
        assert_eq!(stack[1].line, 50);
        assert_eq!(stack[1].column, Some(8));
    }

    #[tokio::test]
    async fn test_state_flows_through_layers() {
        // Task 9.7.4: Verify state flows through all architecture layers
        let shared_context = Arc::new(RwLock::new(SharedExecutionContext::new()));
        let capabilities = Arc::new(RwLock::new(HashMap::new()));
        let debug_cache = Arc::new(SharedDebugStateCache::new());
        let execution_manager = Arc::new(ExecutionManager::new(debug_cache));

        let coordinator = DebugCoordinator::new(
            shared_context.clone(),
            capabilities,
            execution_manager.clone(),
        );

        // Test 1: SharedExecutionContext state flows
        {
            let mut ctx = shared_context.write().await;
            ctx.set_location(SourceLocation {
                source: "state_test.lua".to_string(),
                line: 42,
                column: Some(8),
            });
            ctx.variables.insert(
                "test_var".to_string(),
                serde_json::Value::String("test_value".to_string()),
            );
            ctx.push_frame(StackFrame {
                id: "frame1".to_string(),
                name: "test_function".to_string(),
                source: "state_test.lua".to_string(),
                line: 42,
                column: None,
                locals: vec![],
                is_user_code: true,
            });
        }

        // Verify state is accessible through coordinator
        let position = coordinator.get_current_position().await;
        assert!(position.is_some());
        let pos = position.unwrap();
        assert_eq!(pos.source, "state_test.lua");
        assert_eq!(pos.line, 42);
        assert_eq!(pos.column, Some(8));

        let locals = coordinator.inspect_locals().await;
        assert!(locals.contains_key("test_var"));
        assert_eq!(
            locals.get("test_var").unwrap(),
            &serde_json::Value::String("test_value".to_string())
        );

        let stack = coordinator.get_call_stack().await;
        assert_eq!(stack.len(), 1);
        assert_eq!(stack[0].name, "test_function");

        // Test 2: ExecutionManager state flows
        let bp = Breakpoint::new("state_test.lua".to_string(), 50);
        let bp_id = coordinator.add_breakpoint(bp).await.unwrap();

        let breakpoints = coordinator.get_breakpoints().await;
        assert_eq!(breakpoints.len(), 1);
        assert_eq!(breakpoints[0].line, 50);

        // Test 3: Debug state coordination
        coordinator
            .coordinate_step_pause(
                PauseReason::Step,
                ExecutionLocation {
                    source: "state_test.lua".to_string(),
                    line: 50,
                    column: None,
                },
            )
            .await;

        assert!(coordinator.is_paused().await);
        let state = coordinator.get_debug_state().await;
        assert!(matches!(
            state,
            DebugState::Paused {
                reason: PauseReason::Step,
                ..
            }
        ));

        // Test 4: State query methods work correctly
        let formatted = coordinator.format_current_state().await;
        assert!(formatted.contains("state_test.lua:42"));

        // Clean up
        coordinator.remove_breakpoint(&bp_id).await.unwrap();
        coordinator.resume().await;
        assert!(!coordinator.is_paused().await);
    }

    #[tokio::test]
    async fn test_wait_for_resume_now_works_fix_verified() {
        // Task 9.8.9: This test verifies that the fix now works - coordinate_breakpoint_pause()
        // now blocks execution until resume() is called, completing the missing 15%

        let shared_context = Arc::new(RwLock::new(SharedExecutionContext::new()));
        let capabilities = Arc::new(RwLock::new(HashMap::new()));
        let debug_cache = Arc::new(SharedDebugStateCache::new());
        let execution_manager = Arc::new(ExecutionManager::new(debug_cache));

        let coordinator = Arc::new(DebugCoordinator::new(
            shared_context,
            capabilities,
            execution_manager.clone(),
        ));

        // Add a breakpoint
        let bp = Breakpoint::new("test.lua".to_string(), 10);
        coordinator.add_breakpoint(bp).await.unwrap();

        // Simulate breakpoint hit in a separate task to test blocking behavior
        let location = ExecutionLocation {
            source: "test.lua".to_string(),
            line: 10,
            column: None,
        };

        let coordinator_clone = coordinator.clone();
        let pause_task = tokio::spawn(async move {
            let start = std::time::Instant::now();
            coordinator_clone
                .coordinate_breakpoint_pause(location.clone(), HashMap::new())
                .await;
            start.elapsed()
        });

        // Give the pause task time to start and block
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Verify it's paused and blocked (task should still be running)
        assert!(coordinator.is_paused().await);
        assert!(
            !pause_task.is_finished(),
            "coordinate_breakpoint_pause() should be blocked waiting"
        );

        // Call resume to unblock
        coordinator.resume().await;

        // Now the pause task should complete
        let duration = pause_task.await.unwrap();

        // Verify it was actually blocked for a reasonable time
        assert!(
            duration.as_millis() >= 100,
            "coordinate_breakpoint_pause() should have blocked for at least 100ms"
        );

        // Verify state is back to running
        assert!(!coordinator.is_paused().await);
        assert_eq!(coordinator.get_debug_state().await, DebugState::Running);

        // SUCCESS: The missing 15% is now implemented!
        // coordinate_breakpoint_pause() now properly blocks until resume() is called
    }
}
