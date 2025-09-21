//! Mock implementation of multi-language debug traits

use crate::error::LLMSpellError;
use crate::traits::debug::{
    Breakpoint, DebugSession, DebugState, MultiLanguageDebug, StackFrame, WatchExpression,
};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Mock multi-language debug for testing
#[derive(Debug, Default)]
pub struct MockMultiLanguageDebug {
    sessions: Arc<RwLock<HashMap<String, DebugSession>>>,
    watches: Arc<RwLock<HashMap<String, Vec<WatchExpression>>>>,
}

impl MockMultiLanguageDebug {
    /// Create a new mock debug implementation
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Get a session by ID
    pub async fn get_session(&self, session_id: &str) -> Option<DebugSession> {
        let sessions = self.sessions.read().await;
        sessions.get(session_id).cloned()
    }
}

#[async_trait]
impl MultiLanguageDebug for MockMultiLanguageDebug {
    async fn start_debug_session(
        &self,
        language: &str,
        script_path: &str,
    ) -> Result<DebugSession, LLMSpellError> {
        let session = DebugSession {
            id: uuid::Uuid::new_v4().to_string(),
            language: language.to_string(),
            breakpoints: vec![],
            state: DebugState::Idle,
            variables: HashMap::new(),
            call_stack: vec![StackFrame {
                id: "frame-1".to_string(),
                function_name: "main".to_string(),
                file: script_path.to_string(),
                line: 1,
                column: Some(1),
                locals: HashMap::new(),
            }],
            started_at: chrono::Utc::now(),
        };

        let mut sessions = self.sessions.write().await;
        sessions.insert(session.id.clone(), session.clone());

        Ok(session)
    }

    async fn set_breakpoint(
        &self,
        session_id: &str,
        file: &str,
        line: usize,
        condition: Option<String>,
    ) -> Result<String, LLMSpellError> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| LLMSpellError::Resource {
                message: "Session not found".to_string(),
                resource_type: Some("debug_session".to_string()),
                source: None,
            })?;

        let breakpoint = Breakpoint {
            id: uuid::Uuid::new_v4().to_string(),
            file: file.to_string(),
            line,
            condition,
            hit_count: 0,
            enabled: true,
            log_message: None,
        };

        let bp_id = breakpoint.id.clone();
        session.breakpoints.push(breakpoint);
        Ok(bp_id)
    }

    async fn remove_breakpoint(
        &self,
        session_id: &str,
        breakpoint_id: &str,
    ) -> Result<(), LLMSpellError> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| LLMSpellError::Resource {
                message: "Session not found".to_string(),
                resource_type: Some("debug_session".to_string()),
                source: None,
            })?;

        let initial_len = session.breakpoints.len();
        session.breakpoints.retain(|bp| bp.id != breakpoint_id);

        if session.breakpoints.len() == initial_len {
            return Err(LLMSpellError::Resource {
                message: "Breakpoint not found".to_string(),
                resource_type: Some("breakpoint".to_string()),
                source: None,
            });
        }

        Ok(())
    }

    async fn continue_execution(&self, session_id: &str) -> Result<DebugState, LLMSpellError> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| LLMSpellError::Resource {
                message: "Session not found".to_string(),
                resource_type: Some("debug_session".to_string()),
                source: None,
            })?;

        session.state = DebugState::Running;
        Ok(session.state)
    }

    async fn step_over(&self, session_id: &str) -> Result<DebugState, LLMSpellError> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| LLMSpellError::Resource {
                message: "Session not found".to_string(),
                resource_type: Some("debug_session".to_string()),
                source: None,
            })?;

        session.state = DebugState::Stepping;
        Ok(session.state)
    }

    async fn step_into(&self, session_id: &str) -> Result<DebugState, LLMSpellError> {
        self.step_over(session_id).await
    }

    async fn step_out(&self, session_id: &str) -> Result<DebugState, LLMSpellError> {
        self.step_over(session_id).await
    }

    async fn evaluate(
        &self,
        session_id: &str,
        expression: &str,
    ) -> Result<serde_json::Value, LLMSpellError> {
        let sessions = self.sessions.read().await;
        if !sessions.contains_key(session_id) {
            return Err(LLMSpellError::Resource {
                message: "Session not found".to_string(),
                resource_type: Some("debug_session".to_string()),
                source: None,
            });
        }

        // Mock evaluation - return expression as string
        Ok(serde_json::json!({
            "expression": expression,
            "value": "mock_value",
            "type": "string"
        }))
    }

    async fn get_variables(
        &self,
        session_id: &str,
        _frame_id: Option<&str>,
    ) -> Result<HashMap<String, serde_json::Value>, LLMSpellError> {
        let sessions = self.sessions.read().await;
        let session = sessions
            .get(session_id)
            .ok_or_else(|| LLMSpellError::Resource {
                message: "Session not found".to_string(),
                resource_type: Some("debug_session".to_string()),
                source: None,
            })?;

        Ok(session.variables.clone())
    }

    async fn get_call_stack(&self, session_id: &str) -> Result<Vec<StackFrame>, LLMSpellError> {
        let sessions = self.sessions.read().await;
        let session = sessions
            .get(session_id)
            .ok_or_else(|| LLMSpellError::Resource {
                message: "Session not found".to_string(),
                resource_type: Some("debug_session".to_string()),
                source: None,
            })?;

        Ok(session.call_stack.clone())
    }

    async fn add_watch(&self, session_id: &str, expression: &str) -> Result<String, LLMSpellError> {
        let sessions = self.sessions.read().await;
        if !sessions.contains_key(session_id) {
            return Err(LLMSpellError::Resource {
                message: "Session not found".to_string(),
                resource_type: Some("debug_session".to_string()),
                source: None,
            });
        }

        let watch = WatchExpression {
            id: uuid::Uuid::new_v4().to_string(),
            expression: expression.to_string(),
            value: None,
            enabled: true,
        };

        let mut watches = self.watches.write().await;
        let session_watches = watches.entry(session_id.to_string()).or_default();
        let watch_id = watch.id.clone();
        session_watches.push(watch);

        Ok(watch_id)
    }

    async fn remove_watch(&self, session_id: &str, watch_id: &str) -> Result<(), LLMSpellError> {
        let mut watches = self.watches.write().await;
        let session_watches =
            watches
                .get_mut(session_id)
                .ok_or_else(|| LLMSpellError::Resource {
                    message: "Session not found".to_string(),
                    resource_type: Some("debug_session".to_string()),
                    source: None,
                })?;

        let initial_len = session_watches.len();
        session_watches.retain(|w| w.id != watch_id);

        if session_watches.len() == initial_len {
            return Err(LLMSpellError::Resource {
                message: "Watch not found".to_string(),
                resource_type: Some("watch_expression".to_string()),
                source: None,
            });
        }

        Ok(())
    }

    async fn get_watches(&self, session_id: &str) -> Result<Vec<WatchExpression>, LLMSpellError> {
        let watches = self.watches.read().await;
        Ok(watches.get(session_id).cloned().unwrap_or_default())
    }

    async fn terminate_session(&self, session_id: &str) -> Result<(), LLMSpellError> {
        let mut sessions = self.sessions.write().await;
        let session = sessions
            .get_mut(session_id)
            .ok_or_else(|| LLMSpellError::Resource {
                message: "Session not found".to_string(),
                resource_type: Some("debug_session".to_string()),
                source: None,
            })?;

        session.state = DebugState::Terminated;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_debug_session_lifecycle() {
        let debug = MockMultiLanguageDebug::new();

        // Start session
        let session = debug.start_debug_session("lua", "test.lua").await.unwrap();
        assert_eq!(session.language, "lua");
        assert_eq!(session.state, DebugState::Idle);

        // Set breakpoint
        let bp_id = debug
            .set_breakpoint(&session.id, "test.lua", 10, None)
            .await
            .unwrap();
        assert!(!bp_id.is_empty());

        // Continue execution
        let state = debug.continue_execution(&session.id).await.unwrap();
        assert_eq!(state, DebugState::Running);

        // Step over
        let state = debug.step_over(&session.id).await.unwrap();
        assert_eq!(state, DebugState::Stepping);

        // Terminate
        debug.terminate_session(&session.id).await.unwrap();
        let session = debug.get_session(&session.id).await.unwrap();
        assert_eq!(session.state, DebugState::Terminated);
    }

    #[tokio::test]
    async fn test_breakpoint_management() {
        let debug = MockMultiLanguageDebug::new();

        let session = debug
            .start_debug_session("javascript", "app.js")
            .await
            .unwrap();

        // Set multiple breakpoints
        let bp1 = debug
            .set_breakpoint(&session.id, "app.js", 10, None)
            .await
            .unwrap();
        let _bp2 = debug
            .set_breakpoint(&session.id, "app.js", 20, Some("x > 5".to_string()))
            .await
            .unwrap();

        let session = debug.get_session(&session.id).await.unwrap();
        assert_eq!(session.breakpoints.len(), 2);

        // Remove breakpoint
        debug.remove_breakpoint(&session.id, &bp1).await.unwrap();
        let session = debug.get_session(&session.id).await.unwrap();
        assert_eq!(session.breakpoints.len(), 1);

        // Try to remove non-existent breakpoint
        assert!(debug.remove_breakpoint(&session.id, &bp1).await.is_err());
    }

    #[tokio::test]
    async fn test_watch_expressions() {
        let debug = MockMultiLanguageDebug::new();

        let session = debug
            .start_debug_session("python", "main.py")
            .await
            .unwrap();

        // Add watches
        let w1 = debug.add_watch(&session.id, "x + y").await.unwrap();
        let _w2 = debug.add_watch(&session.id, "len(array)").await.unwrap();

        let watches = debug.get_watches(&session.id).await.unwrap();
        assert_eq!(watches.len(), 2);

        // Remove watch
        debug.remove_watch(&session.id, &w1).await.unwrap();
        let watches = debug.get_watches(&session.id).await.unwrap();
        assert_eq!(watches.len(), 1);

        // Evaluate expression
        let result = debug.evaluate(&session.id, "2 + 2").await.unwrap();
        assert!(result.get("expression").is_some());
    }
}
