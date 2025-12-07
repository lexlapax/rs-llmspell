use crate::state::AppState;
use axum::extract::State;
use axum::response::IntoResponse;

pub async fn get_metrics(State(state): State<AppState>) -> impl IntoResponse {
    let recorder_handle = state.metrics_recorder.clone();
    recorder_handle.render()
}
