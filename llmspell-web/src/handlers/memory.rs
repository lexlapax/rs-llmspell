use axum::{
    extract::{Query, State},
    Json,
};
use serde::{Deserialize, Serialize};
use crate::state::AppState;

#[derive(Deserialize)]
pub struct SearchMemoryParams {
    pub query: String,
    pub limit: Option<usize>,
}

#[derive(Serialize)]
pub struct MemoryEntryResponse {
    pub id: String,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

pub async fn search_memory(
    State(state): State<AppState>,
    Query(params): Query<SearchMemoryParams>,
) -> Result<Json<Vec<MemoryEntryResponse>>, String> {
    let kernel = state.kernel.lock().await;
    
    let memory_manager = kernel
        .memory_manager()
        .ok_or_else(|| "Memory manager not available".to_string())?;

    let episodic = memory_manager.episodic();

    let results = episodic
        .search(&params.query, params.limit.unwrap_or(10))
        .await
        .map_err(|e| e.to_string())?;

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
