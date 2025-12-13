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

    #[tokio::test]
    async fn test_health_check() {
        use crate::config::WebConfig;

        // Create test state
        let config = WebConfig::default();
        let state = AppState {
            config: config.clone(),
            kernel: std::sync::Arc::new(tokio::sync::Mutex::new(
                llmspell_kernel::KernelHandle::new(llmspell_kernel::IntegratedKernel::new(
                    llmspell_config::LLMSpellConfig::default(),
                ).unwrap())
            )),
            runtime_config: std::sync::Arc::new(tokio::sync::RwLock::new(
                llmspell_config::EnvRegistry::new()
            )),
            static_config_path: None,
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
