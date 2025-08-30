//! Interactive debugging interface - Layer 1 of three-layer architecture
//!
//! Provides high-level interactive debugging interface using `ExecutionBridge` from Phase 9.1

use crate::session_manager::DebugSessionManager;
use crate::{Breakpoint, DebugCommand, DebugState, ExecutionManager, SharedExecutionContext};
use anyhow::Result;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Interactive debugger that coordinates with `ExecutionBridge` architecture
pub struct InteractiveDebugger {
    /// `ExecutionManager` from `execution_bridge.rs` (Phase 9.1)
    execution_manager: Arc<ExecutionManager>,
    /// `SharedExecutionContext` for cross-system enrichment (Phase 9.1)
    shared_context: Arc<RwLock<SharedExecutionContext>>,
    /// Session manager for multi-client debugging
    session_manager: Arc<DebugSessionManager>,
}

impl InteractiveDebugger {
    /// Create new interactive debugger using Phase 9.1 architecture
    #[must_use]
    pub fn new(
        execution_manager: Arc<ExecutionManager>,
        shared_context: Arc<RwLock<SharedExecutionContext>>,
    ) -> Self {
        let session_manager = Arc::new(DebugSessionManager::new(execution_manager.clone()));

        Self {
            execution_manager,
            shared_context,
            session_manager,
        }
    }

    /// Install Lua hooks using existing `lua/globals/execution.rs` (not new `debug_hooks.rs`)
    ///
    /// # Errors
    ///
    /// Returns an error if installing debug hooks fails
    pub fn install_lua_hooks(&self, lua: &mlua::Lua) -> Result<()> {
        // Use existing execution hooks for interactive debugging
        let _hook_handle = llmspell_bridge::lua::globals::execution::install_debug_hooks(
            lua,
            self.execution_manager.clone(),
        )
        .map_err(|e| anyhow::anyhow!("Failed to install debug hooks: {}", e))?;

        // Store the hook handle if needed for later cleanup
        // For now, just return success
        Ok(())
    }

    /// Set a breakpoint using existing `Breakpoint` type (not `ConditionalBreakpoint`)
    ///
    /// # Errors
    ///
    /// Returns an error if setting the breakpoint fails
    pub async fn set_breakpoint(&self, source: String, line: u32) -> Result<String> {
        let breakpoint = Breakpoint::new(source, line);
        let id = self.execution_manager.add_breakpoint(breakpoint).await;
        Ok(id)
    }

    /// Set a conditional breakpoint using enhanced `Breakpoint` type
    ///
    /// # Errors
    ///
    /// Returns an error if setting the conditional breakpoint fails
    pub async fn set_conditional_breakpoint(
        &self,
        source: String,
        line: u32,
        condition: String,
    ) -> Result<String> {
        let breakpoint = Breakpoint::new(source, line).with_condition(condition);
        let id = self.execution_manager.add_breakpoint(breakpoint).await;
        Ok(id)
    }

    /// Remove a breakpoint via `ExecutionManager`
    ///
    /// # Errors
    ///
    /// Returns an error if removing the breakpoint fails
    pub async fn remove_breakpoint(&self, id: &str) -> Result<bool> {
        Ok(self.execution_manager.remove_breakpoint(id).await)
    }

    /// Continue execution via `ExecutionManager`
    ///
    /// # Errors
    ///
    /// Returns an error if continuing execution fails
    pub async fn continue_execution(&self) -> Result<()> {
        self.execution_manager
            .send_command(DebugCommand::Continue)
            .await;
        Ok(())
    }

    /// Step into next statement via `ExecutionManager`
    ///
    /// # Errors
    ///
    /// Returns an error if the step operation fails
    pub async fn step_into(&self) -> Result<()> {
        self.execution_manager
            .send_command(DebugCommand::StepInto)
            .await;
        Ok(())
    }

    /// Step over next statement via `ExecutionManager`
    ///
    /// # Errors
    ///
    /// Returns an error if the step operation fails
    pub async fn step_over(&self) -> Result<()> {
        self.execution_manager
            .send_command(DebugCommand::StepOver)
            .await;
        Ok(())
    }

    /// Step out of current function via `ExecutionManager`
    ///
    /// # Errors
    ///
    /// Returns an error if the step operation fails
    pub async fn step_out(&self) -> Result<()> {
        self.execution_manager
            .send_command(DebugCommand::StepOut)
            .await;
        Ok(())
    }

    /// Get current debug state via `ExecutionManager`
    pub async fn get_debug_state(&self) -> DebugState {
        self.execution_manager.get_state().await
    }

    /// Get stack trace using unified `StackFrame` type via `ExecutionManager`
    pub async fn get_stack_trace(&self) -> Vec<crate::StackFrame> {
        self.execution_manager.get_stack_trace().await
    }

    /// Get variables using unified `Variable` type via `ExecutionManager`
    pub async fn get_variables(&self, frame_id: Option<&str>) -> Vec<crate::Variable> {
        self.execution_manager.get_variables(frame_id).await
    }

    /// Create a debug session for multi-client support
    ///
    /// # Errors
    ///
    /// Returns an error if session creation fails
    pub async fn create_debug_session(&self, client_id: String) -> Result<String> {
        self.session_manager.create_session(client_id).await
    }

    /// Handle debug command for specific session
    ///
    /// # Errors
    ///
    /// Returns an error if the command handling fails
    pub async fn handle_session_debug_command(
        &self,
        session_id: &str,
        command: DebugCommand,
    ) -> Result<()> {
        self.session_manager
            .handle_debug_command(session_id, command)
            .await
    }

    /// Get shared execution context for enrichment
    pub async fn get_shared_context(&self) -> SharedExecutionContext {
        self.shared_context.read().await.clone()
    }

    /// Update shared execution context
    pub async fn update_shared_context(&self, context: SharedExecutionContext) {
        *self.shared_context.write().await = context;
    }
}

impl Default for InteractiveDebugger {
    fn default() -> Self {
        let execution_manager = Arc::new(ExecutionManager::new());
        let shared_context = Arc::new(RwLock::new(SharedExecutionContext::new()));
        Self::new(execution_manager, shared_context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_interactive_debugger_creation() {
        let execution_manager = Arc::new(ExecutionManager::new());
        let shared_context = Arc::new(RwLock::new(SharedExecutionContext::new()));

        let debugger = InteractiveDebugger::new(execution_manager, shared_context);

        // Test basic state retrieval
        let state = debugger.get_debug_state().await;
        assert_eq!(state, DebugState::Terminated);
    }

    #[tokio::test]
    async fn test_breakpoint_management() {
        let debugger = InteractiveDebugger::default();

        // Test setting breakpoint
        let id = debugger
            .set_breakpoint("test.lua".to_string(), 10)
            .await
            .unwrap();
        assert!(!id.is_empty());

        // Test removing breakpoint
        let removed = debugger.remove_breakpoint(&id).await.unwrap();
        assert!(removed);
    }

    #[tokio::test]
    async fn test_conditional_breakpoint() {
        let debugger = InteractiveDebugger::default();

        // Test setting conditional breakpoint
        let id = debugger
            .set_conditional_breakpoint("test.lua".to_string(), 15, "x > 10".to_string())
            .await
            .unwrap();
        assert!(!id.is_empty());
    }

    #[tokio::test]
    async fn test_debug_session_management() {
        let debugger = InteractiveDebugger::default();

        // Test creating debug session
        let session_id = debugger
            .create_debug_session("client1".to_string())
            .await
            .unwrap();
        assert!(!session_id.is_empty());
    }
}
