//! Comm channel handler for session management
//!
//! Provides comm-based communication for session operations:
//! - Session state queries and updates
//! - Variable inspection and manipulation
//! - Session persistence control

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::session_persistence::SessionMapper;
use llmspell_sessions::SessionId;

/// Known comm target names
pub const SESSION_COMM_TARGET: &str = "llmspell.session";
pub const STATE_COMM_TARGET: &str = "llmspell.state";

/// Manager for comm channels
#[derive(Clone)]
pub struct CommManager {
    /// Active comm channels
    comms: Arc<RwLock<HashMap<String, CommChannel>>>,
    /// Session mapper for persistence
    session_mapper: Arc<SessionMapper>,
}

/// Individual comm channel
#[derive(Debug, Clone)]
struct CommChannel {
    #[allow(dead_code)]
    comm_id: String,
    target_name: String,
    session_id: Option<SessionId>,
    #[allow(dead_code)]
    created_at: chrono::DateTime<chrono::Utc>,
}

/// Request types for session management comms
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action")]
pub enum SessionCommRequest {
    #[serde(rename = "get_state")]
    GetState { key: String },

    #[serde(rename = "set_state")]
    SetState { key: String, value: Value },

    #[serde(rename = "get_variables")]
    GetVariables,

    #[serde(rename = "set_variables")]
    SetVariables { variables: HashMap<String, Value> },

    #[serde(rename = "get_execution_count")]
    GetExecutionCount,

    #[serde(rename = "suspend_session")]
    SuspendSession,

    #[serde(rename = "activate_session")]
    ActivateSession,
}

/// Response types for session management comms
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status")]
pub enum SessionCommResponse {
    #[serde(rename = "ok")]
    Ok { data: Value },

    #[serde(rename = "error")]
    Error { message: String },
}

impl CommManager {
    /// Create new comm manager
    ///
    /// # Errors
    /// Returns error if comm manager initialization fails
    pub fn new(session_mapper: Arc<SessionMapper>) -> Result<Self> {
        Ok(Self {
            comms: Arc::new(RwLock::new(HashMap::new())),
            session_mapper,
        })
    }

    /// Open a new comm channel
    ///
    /// # Errors
    /// Returns error if session creation fails
    pub async fn open_comm(
        &self,
        comm_id: String,
        target_name: String,
        jupyter_session_id: &str,
        kernel_id: &str,
    ) -> Result<()> {
        // Get or create session for this comm
        let session_id = if target_name == SESSION_COMM_TARGET || target_name == STATE_COMM_TARGET {
            Some(
                self.session_mapper
                    .get_or_create_session(jupyter_session_id, kernel_id)
                    .await?,
            )
        } else {
            None
        };

        let channel = CommChannel {
            comm_id: comm_id.clone(),
            target_name: target_name.clone(),
            session_id,
            created_at: chrono::Utc::now(),
        };

        self.comms.write().await.insert(comm_id, channel);

        Ok(())
    }

    /// Handle comm message
    ///
    /// # Errors
    /// Returns error if message parsing or handling fails
    pub async fn handle_comm_msg(&self, comm_id: &str, data: Value) -> Result<SessionCommResponse> {
        // Get comm channel
        let channel = {
            let comms = self.comms.read().await;
            comms
                .get(comm_id)
                .ok_or_else(|| anyhow::anyhow!("Unknown comm_id: {}", comm_id))?
                .clone()
        };

        // Only handle session management comms
        if channel.target_name != SESSION_COMM_TARGET && channel.target_name != STATE_COMM_TARGET {
            return Ok(SessionCommResponse::Error {
                message: format!("Unsupported target: {}", channel.target_name),
            });
        }

        // Parse request
        let request: SessionCommRequest = serde_json::from_value(data)?;

        // Get session ID
        let session_id = channel
            .session_id
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("No session associated with comm"))?;

        // Handle request
        match request {
            SessionCommRequest::GetState { key } => {
                let value = self
                    .session_mapper
                    .get_kernel_state(session_id, &key)
                    .await?;
                Ok(SessionCommResponse::Ok {
                    data: value.unwrap_or(Value::Null),
                })
            }

            SessionCommRequest::SetState { key, value } => {
                self.session_mapper
                    .store_kernel_state(session_id, &key, value)
                    .await?;
                Ok(SessionCommResponse::Ok {
                    data: Value::Bool(true),
                })
            }

            SessionCommRequest::GetVariables => {
                let vars = self.session_mapper.get_variables(session_id).await?;
                Ok(SessionCommResponse::Ok {
                    data: serde_json::to_value(vars)?,
                })
            }

            SessionCommRequest::SetVariables { variables } => {
                self.session_mapper
                    .store_variables(session_id, variables)
                    .await?;
                Ok(SessionCommResponse::Ok {
                    data: Value::Bool(true),
                })
            }

            SessionCommRequest::GetExecutionCount => {
                let count = self.session_mapper.get_execution_count(session_id).await?;
                Ok(SessionCommResponse::Ok {
                    data: Value::Number(count.into()),
                })
            }

            SessionCommRequest::SuspendSession => {
                self.session_mapper.suspend_session(session_id).await?;
                Ok(SessionCommResponse::Ok {
                    data: Value::Bool(true),
                })
            }

            SessionCommRequest::ActivateSession => {
                self.session_mapper.activate_session(session_id).await?;
                Ok(SessionCommResponse::Ok {
                    data: Value::Bool(true),
                })
            }
        }
    }

    /// Close a comm channel
    ///
    /// # Errors
    /// Returns error if comm channel closing fails
    pub async fn close_comm(&self, comm_id: &str) -> Result<()> {
        self.comms.write().await.remove(comm_id);
        Ok(())
    }

    /// Get information about all open comms
    pub async fn get_comm_info(
        &self,
        target_name: Option<&str>,
    ) -> HashMap<String, crate::jupyter::protocol::CommInfo> {
        let mut info = HashMap::new();

        {
            let comms = self.comms.read().await;
            for (comm_id, channel) in comms.iter() {
                if let Some(target) = target_name {
                    if channel.target_name != target {
                        continue;
                    }
                }

                info.insert(
                    comm_id.clone(),
                    crate::jupyter::protocol::CommInfo {
                        target_name: channel.target_name.clone(),
                    },
                );
            }
        }

        info
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_comm_open_close() {
        let session_mapper = Arc::new(SessionMapper::new().await.unwrap());
        let comm_manager = CommManager::new(session_mapper).unwrap();

        let comm_id = Uuid::new_v4().to_string();
        let jupyter_session = "test-session";
        let kernel_id = "test-kernel";

        // Open comm
        comm_manager
            .open_comm(
                comm_id.clone(),
                SESSION_COMM_TARGET.to_string(),
                jupyter_session,
                kernel_id,
            )
            .await
            .unwrap();

        // Check comm exists
        let info = comm_manager.get_comm_info(None).await;
        assert!(info.contains_key(&comm_id));

        // Close comm
        comm_manager.close_comm(&comm_id).await.unwrap();

        // Check comm removed
        let info = comm_manager.get_comm_info(None).await;
        assert!(!info.contains_key(&comm_id));
    }

    #[tokio::test]
    async fn test_session_comm_messages() {
        let session_mapper = Arc::new(SessionMapper::new().await.unwrap());
        let comm_manager = CommManager::new(session_mapper).unwrap();

        let comm_id = Uuid::new_v4().to_string();
        let jupyter_session = "test-session";
        let kernel_id = "test-kernel";

        // Open comm
        comm_manager
            .open_comm(
                comm_id.clone(),
                SESSION_COMM_TARGET.to_string(),
                jupyter_session,
                kernel_id,
            )
            .await
            .unwrap();

        // Test set state
        let set_request = serde_json::json!({
            "action": "set_state",
            "key": "test_key",
            "value": "test_value"
        });

        let response = comm_manager
            .handle_comm_msg(&comm_id, set_request)
            .await
            .unwrap();

        match response {
            SessionCommResponse::Ok { data } => {
                assert_eq!(data, Value::Bool(true));
            }
            SessionCommResponse::Error { .. } => panic!("Expected Ok response"),
        }

        // Test get state
        let get_request = serde_json::json!({
            "action": "get_state",
            "key": "test_key"
        });

        let response = comm_manager
            .handle_comm_msg(&comm_id, get_request)
            .await
            .unwrap();

        match response {
            SessionCommResponse::Ok { data } => {
                assert_eq!(data, Value::String("test_value".to_string()));
            }
            SessionCommResponse::Error { .. } => panic!("Expected Ok response with test_value"),
        }
    }

    #[tokio::test]
    async fn test_comm_message_validation() {
        let session_mapper = Arc::new(SessionMapper::new().await.unwrap());
        let comm_manager = CommManager::new(session_mapper).unwrap();

        let comm_id = Uuid::new_v4().to_string();
        let jupyter_session = "test-session";
        let kernel_id = "test-kernel";

        // Open comm with non-session target
        comm_manager
            .open_comm(
                comm_id.clone(),
                "unknown.target".to_string(),
                jupyter_session,
                kernel_id,
            )
            .await
            .unwrap();

        // Try to send session message to non-session comm
        let request = serde_json::json!({
            "action": "get_state",
            "key": "test"
        });

        let response = comm_manager
            .handle_comm_msg(&comm_id, request)
            .await
            .unwrap();

        match response {
            SessionCommResponse::Error { message } => {
                assert!(message.contains("Unsupported target"));
            }
            SessionCommResponse::Ok { .. } => panic!("Expected error for unsupported target"),
        }
    }
}
