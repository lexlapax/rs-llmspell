//! Protocol adapter for Debug Session Management
//!
//! Provides a stub implementation of session management for protocol-based debugging.
//! This will be replaced when debug infrastructure is fully consolidated.

use async_trait::async_trait;
use llmspell_core::{
    debug::{DebugCapability, DebugRequest, DebugResponse, DebugState},
    Result,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Simple debug session representation
#[derive(Debug, Clone)]
pub struct DebugSession {
    pub session_id: String,
    pub debug_state: DebugState,
}

/// Stub implementation of session manager for protocol-based debugging
pub struct DebugSessionManagerAdapter {
    sessions: Arc<RwLock<HashMap<String, DebugSession>>>,
}

impl DebugSessionManagerAdapter {
    /// Create a new stub session manager adapter
    #[must_use]
    pub fn new() -> Self {
        Self {
            sessions: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for DebugSessionManagerAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DebugCapability for DebugSessionManagerAdapter {
    async fn process_debug_request(&self, request: DebugRequest) -> Result<DebugResponse> {
        match request {
            DebugRequest::CreateSession { .. } => {
                let session_id = uuid::Uuid::new_v4().to_string();
                let session = DebugSession {
                    session_id: session_id.clone(),
                    debug_state: DebugState::Idle,
                };

                self.sessions
                    .write()
                    .await
                    .insert(session_id.clone(), session);

                Ok(DebugResponse::SessionCreated {
                    session_id,
                    capabilities: vec![
                        "session_management".to_string(),
                        "multi_session".to_string(),
                    ],
                })
            }

            DebugRequest::Terminate { session_id } => {
                let mut sessions = self.sessions.write().await;
                if let Some(id) = session_id {
                    sessions.remove(&id);
                } else {
                    sessions.clear();
                }
                drop(sessions);
                Ok(DebugResponse::SessionTerminated)
            }

            DebugRequest::GetDebugState => {
                let sessions = self.sessions.read().await;
                let state = if let Some((_, session)) = sessions.iter().next() {
                    session.debug_state.clone()
                } else {
                    DebugState::Idle
                };
                drop(sessions);
                Ok(DebugResponse::DebugStateInfo(state))
            }

            _ => Ok(DebugResponse::Error {
                message: "Request should be handled by different capability".to_string(),
                details: Some(format!("Request type: {request:?}")),
            }),
        }
    }

    fn capabilities(&self) -> Vec<String> {
        vec![
            "session_management".to_string(),
            "multi_session".to_string(),
            "session_persistence".to_string(),
        ]
    }

    fn name(&self) -> &'static str {
        "session_manager"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_session_manager_adapter() {
        let adapter = DebugSessionManagerAdapter::new();

        // Test capabilities
        assert!(adapter
            .capabilities()
            .contains(&"session_management".to_string()));
        assert_eq!(adapter.name(), "session_manager");

        // Test session creation
        let response = adapter
            .process_debug_request(DebugRequest::CreateSession {
                script: "test.lua".to_string(),
                args: vec![],
            })
            .await
            .unwrap();

        if let DebugResponse::SessionCreated { session_id, .. } = response {
            assert!(!session_id.is_empty());
        } else {
            panic!("Expected SessionCreated response");
        }
    }
}
