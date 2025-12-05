use axum::{
    body::Body,
    http::{Request, StatusCode},
    routing::{get, post},
    Router,
};
use http_body_util::BodyExt; // for collect()
use llmspell_web::handlers;
// use llmspell_web::state::AppState; // Not needed if we don't use State in handlers
use tower::ServiceExt; // for oneshot
use serde_json::Value;

#[tokio::test]
async fn test_templates_api_list() {
    // 1. Setup Router without state (since current handlers don't use it)
    let app = Router::new()
        .route("/api/templates", get(handlers::templates::list_templates))
        .route("/api/templates/:id", get(handlers::templates::get_template))
        .route("/api/templates/:id/launch", post(handlers::templates::launch_template));

    // 2. Test GET /api/templates
    let response = app
        .oneshot(Request::builder().uri("/api/templates").body(Body::empty()).unwrap())
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

#[tokio::test]
async fn test_config_api_get() {
     // 1. Setup minimal router for config
    let app = Router::new()
        .route("/api/config", get(handlers::config::get_config));

    // 2. Test GET /api/config
    let response = app
        .oneshot(Request::builder().uri("/api/config").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let body_bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body_json: Value = serde_json::from_slice(&body_bytes).unwrap();
    
    // Should be array of ConfigItem
    assert!(body_json.is_array());
}
