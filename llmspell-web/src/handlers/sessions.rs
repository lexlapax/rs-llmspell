use crate::error::WebError;
use crate::state::AppState;
use crate::workflow_builder::WorkflowExecution;
use axum::{
    extract::{Path, Query, State},
    Json,
};
use llmspell_kernel::sessions::types::{SessionQuery, SessionSortBy, SessionStatus};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct ArtifactInfo {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub type_: String,
    pub size: u64,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Serialize, utoipa::ToSchema)]
pub struct SessionDetailsResponse {
    // Basic session info
    pub id: String,
    pub name: String,
    pub template_id: Option<String>,
    pub template_name: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    #[schema(value_type = String)]
    pub status: String,
    #[schema(value_type = Object)]
    pub metadata: serde_json::Value,

    // Workflow execution data
    #[schema(value_type = Option<Object>)]
    pub workflow: Option<WorkflowExecution>,

    // Logs and artifacts
    pub logs: Vec<String>,
    pub artifacts: Vec<ArtifactInfo>,
}

#[derive(Deserialize, utoipa::IntoParams)]
pub struct ListSessionsParams {
    pub status: Option<SessionStatus>,
    pub created_by: Option<String>,
    pub tags: Option<Vec<String>>,
    pub search: Option<String>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct SessionResponse {
    pub id: String,
    pub name: Option<String>,
    #[schema(value_type = String)]
    pub status: SessionStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub tags: Vec<String>,
    #[schema(value_type = Object)]
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

#[utoipa::path(
    get,
    path = "/api/sessions",
    tag = "sessions",
    params(ListSessionsParams),
    responses(
        (status = 200, description = "List sessions", body = Vec<SessionResponse>)
    )
)]
pub async fn list_sessions(
    State(state): State<AppState>,
    Query(params): Query<ListSessionsParams>,
) -> Result<Json<Vec<SessionResponse>>, WebError> {
    let kernel = state.kernel.lock().await;
    let session_manager = kernel.session_manager();

    let query = SessionQuery {
        status: params.status,
        created_by: params.created_by,
        tags: params.tags.unwrap_or_default(),
        search_text: params.search,
        limit: params.limit,
        offset: params.offset,
        sort_by: SessionSortBy::UpdatedAt,
        sort_desc: true,
        ..Default::default()
    };

    let sessions = session_manager
        .list_sessions(query)
        .await
        .map_err(|e| WebError::Internal(e.to_string()))?;

    let response = sessions
        .into_iter()
        .map(|s| SessionResponse {
            id: s.id.to_string(),
            name: s.name,
            status: s.status,
            created_at: s.created_at,
            updated_at: s.updated_at,
            tags: s.tags,
            metadata: s.custom_metadata,
        })
        .collect();

    Ok(Json(response))
}

#[utoipa::path(
    get,
    path = "/api/sessions/{id}",
    tag = "sessions",
    params(
        ("id" = String, Path, description = "Session ID")
    ),
    responses(
        (status = 200, description = "Get session details", body = SessionResponse),
        (status = 404, description = "Session not found")
    )
)]
pub async fn get_session(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<SessionResponse>, WebError> {
    let kernel = state.kernel.lock().await;
    let session_manager = kernel.session_manager();

    // Parse session ID
    // Note: SessionId::from_str is needed but not directly exposed in types?
    // Let's assume we can pass string to get_session if it takes &str or similar,
    // but looking at manager.rs (which I saw earlier via cat), it takes &SessionId.
    // I need to parse it.
    // Wait, SessionId implements FromStr.

    // Actually, let's check if I can use SessionId directly.
    // I'll try to parse it.
    use std::str::FromStr;
    let session_id = llmspell_kernel::sessions::types::SessionId::from_str(&id)
        .map_err(|_| WebError::BadRequest("Invalid session ID".to_string()))?;

    let session = session_manager
        .get_session(&session_id)
        .await
        .map_err(|e| e.to_string())?;

    let metadata = session.metadata.read().await.clone();

    Ok(Json(SessionResponse {
        id: metadata.id.to_string(),
        name: metadata.name,
        status: metadata.status,
        created_at: metadata.created_at,
        updated_at: metadata.updated_at,
        tags: metadata.tags,
        metadata: metadata.custom_metadata,
    }))
}

#[utoipa::path(
    get,
    path = "/api/sessions/{id}/details",
    tag = "sessions",
    params(
        ("id" = String, Path, description = "Session ID")
    ),
    responses(
        (status = 200, description = "Get full session details", body = SessionDetailsResponse),
        (status = 404, description = "Session not found")
    )
)]
pub async fn get_session_details(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<SessionDetailsResponse>, WebError> {
    let kernel = state.kernel.lock().await;
    let session_manager = kernel.session_manager();

    // Parse ID
    use std::str::FromStr;
    let session_id = llmspell_kernel::sessions::types::SessionId::from_str(&id)
        .map_err(|_| WebError::BadRequest("Invalid session ID".to_string()))?;

    let session = session_manager
        .get_session(&session_id)
        .await
        .map_err(|e| WebError::Internal(format!("Failed to get session: {}", e)))?;

    let metadata = session.metadata.read().await.clone();

    // Extract workflow execution from session state
    let workflow: Option<WorkflowExecution> = session
        .get_state("workflow_execution")
        .await
        .and_then(|v| serde_json::from_value(v).ok());

    // Get artifacts
    let artifact_ids = session.artifact_ids().await;
    let artifacts: Vec<ArtifactInfo> = artifact_ids
        .iter()
        .map(|id| ArtifactInfo {
            id: id.clone(),
            name: format!("artifact_{}", id),
            type_: "unknown".to_string(),
            size: 0,
            created_at: metadata.created_at,
        })
        .collect();

    // Get logs from session state or events
    let logs: Vec<String> = session
        .get_state("logs")
        .await
        .and_then(|v| serde_json::from_value(v).ok())
        .unwrap_or_default();

    let template_id = metadata
        .custom_metadata
        .get("template_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let template_name = metadata
        .custom_metadata
        .get("template_name")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    Ok(Json(SessionDetailsResponse {
        id: metadata.id.to_string(),
        name: metadata
            .name
            .clone()
            .unwrap_or_else(|| "Unnamed Session".to_string()),
        template_id,
        template_name,
        created_at: metadata.created_at,
        updated_at: metadata.updated_at,
        status: metadata.status.to_string(),
        metadata: serde_json::to_value(metadata.custom_metadata).unwrap_or_default(),
        workflow,
        logs,
        artifacts,
    }))
}
