//! Execution memory hook for capturing kernel executions as episodic memories
//!
//! This hook captures code input and execution results as episodic memory entries
//! when the kernel executes code. Each execution creates two entries:
//! - User entry: The code/command submitted
//! - Assistant entry: The execution result
//!
//! Phase 13.7.3: Kernel Execution-Memory Linking

use crate::hooks::{Hook, HookContext, HookResult};
use anyhow::Result;
use llmspell_memory::types::EpisodicEntry;
use llmspell_memory::MemoryManager;
use serde_json::json;
use std::sync::Arc;
use tracing::{debug, error, warn};

/// Hook that captures kernel executions as episodic memories
///
/// Listens to `PostCodeExecution` events and creates two episodic entries:
/// 1. User entry with the code input
/// 2. Assistant entry with the execution result
///
/// Embeddings are generated asynchronously by the `ConsolidationDaemon`.
pub struct ExecutionMemoryHook {
    memory_manager: Arc<dyn MemoryManager>,
}

impl std::fmt::Debug for ExecutionMemoryHook {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExecutionMemoryHook")
            .field("memory_manager", &"Arc<dyn MemoryManager>")
            .finish()
    }
}

impl ExecutionMemoryHook {
    /// Create a new execution memory hook
    pub fn new(memory_manager: Arc<dyn MemoryManager>) -> Self {
        Self { memory_manager }
    }
}

#[async_trait::async_trait]
impl Hook for ExecutionMemoryHook {
    async fn execute(&self, ctx: &mut HookContext) -> Result<HookResult> {
        // Extract data from context
        let Some(session_id) = ctx.data.get("session_id").and_then(|v| v.as_str()) else {
            warn!("ExecutionMemoryHook: session_id not in context, skipping");
            return Ok(HookResult::Continue);
        };

        let Some(code) = ctx.data.get("code").and_then(|v| v.as_str()) else {
            warn!("ExecutionMemoryHook: code not in context, skipping");
            return Ok(HookResult::Continue);
        };

        // Check if execution was successful
        let success = ctx
            .data
            .get("success")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false);

        // Get result or error
        let result_content = if success {
            ctx.data
                .get("result")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string()
        } else {
            ctx.data
                .get("error")
                .and_then(|v| v.as_str())
                .map_or_else(|| "Unknown error".to_string(), |s| format!("Error: {s}"))
        };

        // Create user entry (input) using EpisodicEntry::new()
        let mut user_entry =
            EpisodicEntry::new(session_id.to_string(), "user".to_string(), code.to_string());
        user_entry.metadata = json!({
            "type": "execution_input",
            "execution_id": ctx.data.get("execution_id"),
        });

        // Create assistant entry (output) using EpisodicEntry::new()
        let mut assistant_entry = EpisodicEntry::new(
            session_id.to_string(),
            "assistant".to_string(),
            result_content,
        );
        assistant_entry.metadata = json!({
            "type": "execution_output",
            "execution_id": ctx.data.get("execution_id"),
            "success": success,
        });

        // Add entries to episodic memory
        match self.memory_manager.episodic().add(user_entry).await {
            Ok(_) => {
                debug!(
                    "ExecutionMemoryHook: Added user entry for session {}",
                    session_id
                );
            }
            Err(e) => {
                error!(
                    "ExecutionMemoryHook: Failed to add user entry for session {}: {}",
                    session_id, e
                );
                return Err(e.into());
            }
        }

        match self.memory_manager.episodic().add(assistant_entry).await {
            Ok(_) => {
                debug!(
                    "ExecutionMemoryHook: Added assistant entry for session {}",
                    session_id
                );
            }
            Err(e) => {
                error!(
                    "ExecutionMemoryHook: Failed to add assistant entry for session {}: {}",
                    session_id, e
                );
                return Err(e.into());
            }
        }

        debug!(
            "ExecutionMemoryHook: Captured execution as episodic memory for session {}",
            session_id
        );

        Ok(HookResult::Continue)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hooks::{ComponentId, ComponentType, HookPoint};
    use llmspell_memory::DefaultMemoryManager;

    #[tokio::test]
    async fn test_execution_memory_hook_success() {
        let memory =
            DefaultMemoryManager::new_in_memory().expect("Failed to create memory manager");
        let memory_arc = Arc::new(memory);
        let hook = ExecutionMemoryHook::new(memory_arc.clone());

        let component_id = ComponentId::new(ComponentType::System, "test".to_string());
        let mut ctx = HookContext::new(HookPoint::SystemStartup, component_id);
        ctx.data
            .insert("session_id".to_string(), json!("test-session"));
        ctx.data.insert("code".to_string(), json!("print('hello')"));
        ctx.data.insert("result".to_string(), json!("hello"));
        ctx.data.insert("success".to_string(), json!(true));
        ctx.data
            .insert("execution_id".to_string(), json!("exec-123"));

        let result = hook.execute(&mut ctx).await;
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), HookResult::Continue));

        // Verify entries were added
        let entries = memory_arc
            .episodic()
            .get_session("test-session")
            .await
            .expect("Failed to get entries");
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].role, "user");
        assert_eq!(entries[0].content, "print('hello')");
        assert_eq!(entries[1].role, "assistant");
        assert_eq!(entries[1].content, "hello");
    }

    #[tokio::test]
    async fn test_execution_memory_hook_error() {
        let memory =
            DefaultMemoryManager::new_in_memory().expect("Failed to create memory manager");
        let memory_arc = Arc::new(memory);
        let hook = ExecutionMemoryHook::new(memory_arc.clone());

        let component_id = ComponentId::new(ComponentType::System, "test".to_string());
        let mut ctx = HookContext::new(HookPoint::SystemStartup, component_id);
        ctx.data
            .insert("session_id".to_string(), json!("test-session"));
        ctx.data.insert("code".to_string(), json!("bad code"));
        ctx.data.insert("error".to_string(), json!("Syntax error"));
        ctx.data.insert("success".to_string(), json!(false));
        ctx.data
            .insert("execution_id".to_string(), json!("exec-456"));

        let result = hook.execute(&mut ctx).await;
        assert!(result.is_ok());

        // Verify entries were added
        let entries = memory_arc
            .episodic()
            .get_session("test-session")
            .await
            .expect("Failed to get entries");
        assert_eq!(entries.len(), 2);
        assert_eq!(entries[0].role, "user");
        assert_eq!(entries[0].content, "bad code");
        assert_eq!(entries[1].role, "assistant");
        assert!(entries[1].content.contains("Syntax error"));
    }

    #[tokio::test]
    async fn test_execution_memory_hook_missing_session_id() {
        let memory =
            DefaultMemoryManager::new_in_memory().expect("Failed to create memory manager");
        let memory_arc = Arc::new(memory);
        let hook = ExecutionMemoryHook::new(memory_arc.clone());

        let component_id = ComponentId::new(ComponentType::System, "test".to_string());
        let mut ctx = HookContext::new(HookPoint::SystemStartup, component_id);
        // Missing session_id
        ctx.data.insert("code".to_string(), json!("print('hello')"));

        let result = hook.execute(&mut ctx).await;
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), HookResult::Continue));
    }
}
