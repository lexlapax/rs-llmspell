use axum::{extract::State, response::IntoResponse, Json};
use serde_json::json;

use crate::state::AppState;

pub async fn health_check(State(state): State<AppState>) -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "version": env!("CARGO_PKG_VERSION"),
        "dev_mode": state.config.dev_mode
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use axum::{routing::get, Router};
    use tower::util::ServiceExt; // for `oneshot`

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn test_health_check() {
        use crate::config::WebConfig;
        use llmspell_bridge::ScriptRuntime;
        use llmspell_config::LLMSpellConfig;
        use llmspell_kernel::api::{start_embedded_kernel_with_executor, KernelExecutionMode};
        use metrics_exporter_prometheus::PrometheusBuilder;
        use std::sync::Arc;
        use tokio::sync::Mutex;

        // Create test state
        let config = WebConfig::default();
        let llm_config = LLMSpellConfig::default();

        let runtime = ScriptRuntime::new(llm_config.clone())
            .await
            .expect("Failed to create runtime");
        let executor = Arc::new(runtime);

        // Use factory function to create kernel handle
        let kernel = start_embedded_kernel_with_executor(
            llm_config,
            executor,
            KernelExecutionMode::Transport,
        )
        .await
        .expect("Failed to start kernel");

        let metrics_handle = PrometheusBuilder::new()
            .install_recorder()
            .expect("failed to install metrics recorder");

        let state = AppState {
            config: config.clone(),
            kernel: Arc::new(Mutex::new(kernel)),
            metrics_recorder: metrics_handle,
            runtime_config: Arc::new(tokio::sync::RwLock::new(llmspell_config::EnvRegistry::new())),
            config_store: None,
            static_config_path: None,
            tool_registry: None,
            agent_registry: None,
            workflow_factory: None,
            provider_manager: None,
            provider_config: None,
        };

        let app = Router::new()
            .route("/health", get(health_check))
            .with_state(state);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let body_json: serde_json::Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(body_json["status"], "ok");
        assert!(body_json["version"].is_string());
        assert!(body_json["dev_mode"].is_boolean());
    }
}
