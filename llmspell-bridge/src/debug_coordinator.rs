//! Language-Agnostic Debug Coordinator (Layer 1)
//!
//! The `DebugCoordinator` extracts core debug logic from language-specific hooks,
//! providing a clean abstraction layer that can be shared across Lua, JavaScript,
//! Python and other script engines.
//!
//! # Architecture Diagram
//!
//! ```text
//!                   ┌─────────────────────┐
//!                   │   User/REPL/CLI     │
//!                   └──────────┬──────────┘
//!                              │
//!                   ┌──────────▼──────────┐
//!    Layer 1:       │  DebugCoordinator   │  ← Language-agnostic
//!                   │  (this file)        │    Fast path: sync checks
//!                   └──────────┬──────────┘    Slow path: async pause
//!                              │
//!                   ┌──────────▼──────────┐
//!    Layer 2:       │   LuaDebugBridge    │  ← Sync/async boundary
//!                   │  (lua_debug_bridge) │    Marshals Lua context
//!                   └──────────┬──────────┘    Uses block_on_async
//!                              │
//!                   ┌──────────▼──────────┐
//!    Layer 3:       │  LuaExecutionHook   │  ← Language-specific
//!                   │  (execution hook)   │    Lua debug hooks
//!                   └─────────────────────┘    Variable extraction
//! ```
//!
//! # Performance Characteristics
//!
//! | Operation | Path | Time | Frequency | Method |
//! |-----------|------|------|-----------|--------|
//! | Breakpoint Check | Fast | <100ns | 99% | `might_break_at_sync()` |
//! | Breakpoint Hit | Slow | <10ms | 1% | `coordinate_breakpoint_pause()` |
//! | Variable Extract | Slow | <1ms | On pause | Via Layer 2/3 |
//! | State Update | Medium | <100μs | On change | Async with cache |
//!
//! # Communication Examples
//!
//! ## Fast Path (no breakpoint):
//! ```text
//! LuaHook → Bridge.might_break_at_sync() → Coordinator → false (sync, <100ns)
//! ```
//!
//! ## Slow Path (breakpoint hit):
//! ```text
//! LuaHook → Bridge.handle_event() → Extract variables →
//! block_on_async(Coordinator.coordinate_breakpoint_pause()) → Pause (async, <10ms)
//! ```

use crate::execution_bridge::{
    Breakpoint, DebugState, DebugStepType, ExecutionLocation, ExecutionManager, PauseReason,
    StackFrame,
};
use crate::execution_context::{SharedExecutionContext, SourceLocation};
use llmspell_core::debug::{DebugCapability, DebugRequest, DebugResponse};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, trace};

/// Language-agnostic debug coordinator
///
/// Provides the core debug logic that can be shared across different script engines.
/// All language-specific bridges delegate to this coordinator for consistent behavior.
pub struct DebugCoordinator {
    /// Shared execution context for cross-language state
    shared_context: Arc<RwLock<SharedExecutionContext>>,

    /// Debug capabilities registry (variable inspector, etc.)
    capabilities: Arc<RwLock<HashMap<String, Arc<dyn DebugCapability>>>>,

    /// Breakpoints managed by coordinator
    breakpoints: Arc<RwLock<HashMap<String, Breakpoint>>>,

    /// Current debug state
    debug_state: Arc<RwLock<DebugState>>,

    /// Execution manager for delegating debug operations
    execution_manager: Arc<ExecutionManager>,
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
            breakpoints: Arc::new(RwLock::new(HashMap::new())),
            debug_state: Arc::new(RwLock::new(DebugState::Running)),
            execution_manager,
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
    /// - Uses local cache for synchronous access
    /// - No async operations on hot path
    /// - <100ns average case performance
    #[must_use]
    pub fn might_break_at_sync(&self, source: &str, line: u32) -> bool {
        // Fast sync check - try to acquire read lock without blocking
        // NOTE: ExecutionManager doesn't have sync methods, so we maintain our own cache
        self.breakpoints.try_read().is_ok_and(|breakpoints| {
            breakpoints
                .values()
                .any(|bp| bp.enabled && bp.source == source && bp.line == line)
        })
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

        // Also update our local state for fast path checks
        {
            let mut state = self.debug_state.write().await;
            *state = DebugState::Paused {
                reason: PauseReason::Breakpoint,
                location,
            };
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
        {
            let mut breakpoints = self.breakpoints.write().await;
            breakpoints.insert(id.clone(), bp);
        }
        Ok(id)
    }

    /// Remove breakpoint
    ///
    /// # Errors
    ///
    /// Returns an error if the breakpoint is not found
    pub async fn remove_breakpoint(&self, id: &str) -> Result<(), String> {
        trace!("Removing breakpoint through coordinator: {}", id);
        let mut breakpoints = self.breakpoints.write().await;
        if breakpoints.remove(id).is_some() {
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
        let breakpoints = self.breakpoints.read().await;
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

        let coordinator = DebugCoordinator::new(shared_context, capabilities, execution_manager);

        // Initially running
        assert!(!coordinator.is_paused().await);
        assert_eq!(coordinator.get_debug_state().await, DebugState::Running);

        // Test pause coordination
        let location = ExecutionLocation {
            source: "test.lua".to_string(),
            line: 10,
            column: None,
        };
        coordinator
            .coordinate_breakpoint_pause(location.clone(), HashMap::new())
            .await;

        assert!(coordinator.is_paused().await);
        if let DebugState::Paused { reason, .. } = coordinator.get_debug_state().await {
            assert_eq!(reason, PauseReason::Breakpoint);
        } else {
            panic!("Should be paused");
        }

        // Test resume
        coordinator.resume().await;
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
}
