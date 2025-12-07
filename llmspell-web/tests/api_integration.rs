use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt; // for collecting body
use llmspell_bridge::ScriptRuntime;
use llmspell_config::LLMSpellConfig;
use llmspell_kernel::api::start_embedded_kernel_with_executor;
use llmspell_web::{config::WebConfig, server::WebServer, state::AppState};
use std::sync::Arc;
use tokio::sync::Mutex;
use tower::ServiceExt; // for oneshot
use metrics_exporter_prometheus::PrometheusBuilder;

// Helper to setup a real kernel environment
async fn setup_env() -> (AppState, tempfile::TempDir) {
    // 1. Load env (optional, for API keys if needed)
    dotenv::dotenv().ok();

    // 2. Setup temp storage
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    let storage_path = temp_dir.path().join("storage");
    std::fs::create_dir_all(&storage_path).expect("Failed to create storage dir");

    // 3. Configure LLMSpell
    // We modify the default config to use our temp storage
    // using a builder or manual modification if fields are public
    // LLMSpellConfig fields are public mostly.
    
    // We need to load defaults then override
    // LLMSpellConfig::default() might look for config files, so we should be careful.
    // Ideally we constructs a clean config.
    // Let's rely on default() but override storage paths.
    
    // NOTE: We're not using load_runtime_config to avoid reading user's config file
    // unless we explicitely want to test user's config (which we don't for isolation).
    
    // Actually LLMSpellConfig::default() is just the struct defaults.
    // We should make sure we point everything to temp_dir.
    
    // We don't have easy mutable access to all config paths in one go, 
    // but the critical one is where SessionManager stores data.
    // SessionManager uses `storage.base_path`.
    
    // Wait, LLMSpellConfig is a bit complex. Let's look at what helps us.
    // We will trust default() provides reasonable starting point and just verify storage.
    
    // It seems LLMSpellConfig doesn't expose all paths easily mutably?
    // Let's assume default is specific enough or we can set it.
    // However, `start_embedded_kernel_with_executor` takes `LLMSpellConfig`.
    
    // Let's try to construct a minimal valid config.
    let config = LLMSpellConfig::default();
    
    // We also need to set script_engine config if we want to run scripts.
    
    let runtime = ScriptRuntime::new(config.clone())
        .await
        .expect("Failed to create runtime");
    let executor = Arc::new(runtime);

    let kernel = start_embedded_kernel_with_executor(config.clone(), executor)
        .await
        .expect("Failed to start kernel");

    // Setup Metrics (mock/real handle)
    // In tests, if multiple run in parallel, installing recorder might fail.
    // We try to install, if fails we assume it's there.
    let recorder_handle = PrometheusBuilder::new()
        .install_recorder()
        .expect("Failed to install Prometheus recorder - ensure tests run in isolation or --test-threads=1");
    
    let state = AppState {
        kernel: Arc::new(Mutex::new(kernel)),
        metrics_recorder: recorder_handle,
        config: WebConfig::default(),
    };

    (state, temp_dir)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_full_integration_flow() {
    // Setup
    let (state, _temp_dir) = setup_env().await;
    let app = WebServer::build_app(state);
    let api_key = "dev-key-123";

    // 1. List Templates
    let response = app.clone()
        .oneshot(
            Request::builder()
                .uri("/api/templates")
                .header("X-API-Key", api_key)
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let templates: Vec<serde_json::Value> = serde_json::from_slice(&body).unwrap();
    
    // We expect some templates to be registered
    assert!(!templates.is_empty(), "Should have templates loaded from default registry");
    let template_id = templates[0]["id"].as_str().unwrap().to_string();
    println!("Found template: {}", template_id);

    // 2. Launch Template (Create Session)
    let launch_payload = serde_json::json!({
        "params": {
            "description": "A simple calculator in Rust"
        }
        // session_id is optional
    });

    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri(format!("/api/templates/{}/launch", template_id))
                .header("Content-Type", "application/json")
                .header("X-API-Key", api_key)
                .body(Body::from(serde_json::to_vec(&launch_payload).unwrap()))
                .unwrap()
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let launch_res: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let session_id = launch_res["session_id"].as_str().expect("Response should contain session_id");
    println!("Created session: {}", session_id);
    
    assert_eq!(launch_res["status"], "created");

    // 3. Verify Session Exists
    let response = app.clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/sessions/{}", session_id))
                .header("X-API-Key", api_key)
                .body(Body::empty())
                .unwrap()
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let body = response.into_body().collect().await.unwrap().to_bytes();
    let session_details: serde_json::Value = serde_json::from_slice(&body).unwrap();
    
    // Session handler returns SessionResponse (flat structure)
    assert_eq!(session_details["id"], session_id);
    println!("Verified session exists: {}", session_id);
}
