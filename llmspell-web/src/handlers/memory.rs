use crate::error::WebError;
use crate::state::AppState;
use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, utoipa::IntoParams)]
pub struct SearchMemoryParams {
    pub query: String,
    pub limit: Option<usize>,
}

#[derive(Serialize, utoipa::ToSchema)]
pub struct MemoryEntryResponse {
    pub id: String,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[utoipa::path(
    get,
    path = "/api/memory/search",
    tag = "memory",
    params(SearchMemoryParams),
    responses(
        (status = 200, description = "Search memory", body = Vec<MemoryEntryResponse>)
    )
)]
pub async fn search_memory(
    State(state): State<AppState>,
    Query(params): Query<SearchMemoryParams>,
) -> Result<Json<Vec<MemoryEntryResponse>>, WebError> {
    let kernel = state.kernel.lock().await;

    let memory_manager = kernel
        .memory_manager()
        .ok_or_else(|| WebError::Internal("Memory manager not available".to_string()))?;

    let episodic = memory_manager.episodic();

    let results = episodic
        .search(&params.query, params.limit.unwrap_or(10))
        .await
        .map_err(|e| WebError::Internal(e.to_string()))?;

    let response = results
        .into_iter()
        .map(|entry| MemoryEntryResponse {
            id: entry.id.to_string(),
            session_id: entry.session_id,
            role: entry.role,
            content: entry.content,
            timestamp: entry.timestamp,
        })
        .collect();

    Ok(Json(response))
}
