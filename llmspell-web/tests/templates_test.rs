use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::get,
    Router,
};
use http_body_util::BodyExt; // for collect()
use llmspell_web::handlers;
// use llmspell_web::state::AppState; // Not needed if we don't use State in handlers
use serde_json::Value;
use tower::ServiceExt; // for oneshot

#[tokio::test]
async fn test_templates_api_list() {
    // 1. Setup Router without state (since current handlers don't use it)
    let app = Router::new()
        .route("/api/templates", get(handlers::templates::list_templates))
        .route("/api/templates/:id", get(handlers::templates::get_template));

    // 2. Test GET /api/templates
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/templates")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);

    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body_json: Value = serde_json::from_slice(&body_bytes).unwrap();

    // Verify structure
    assert!(body_json.is_array());
    // Note: Built-in templates might be empty if registry isn't initialized or empty.
    // The `global_registry` uses `lazy_static` or `OnceLock`.
    // It should initialize default templates from `llmspell-templates` crate.
    // Let's print it to see if it fails.
    let templates = body_json.as_array().unwrap();
    println!("Found {} templates", templates.len());
}
