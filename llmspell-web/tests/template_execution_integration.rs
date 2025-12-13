use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use llmspell_bridge::ScriptRuntime;
use llmspell_config::LLMSpellConfig;
use llmspell_kernel::api::{start_embedded_kernel_with_executor, KernelExecutionMode};
use llmspell_web::config::WebConfig;
use llmspell_web::server::WebServer;
use llmspell_web::state::AppState;
use metrics_exporter_prometheus::PrometheusBuilder;
use serde_json::{json, Value};
use std::sync::Arc;
use tower::ServiceExt; // for `oneshot`

// Helper to setup a real kernel environment
async fn setup_env() -> (AppState, tempfile::TempDir) {
    // 1. Setup temp storage
    let temp_dir = tempfile::tempdir().expect("Failed to create temp dir");
    // 2. Configure LLMSpell
    let config = LLMSpellConfig::default();

    let runtime = ScriptRuntime::new(config.clone())
        .await
        .expect("Failed to create runtime");
    let executor = Arc::new(runtime);

    // Extract registries
    let tool_registry = executor.tool_registry().clone();
    let agent_registry = executor.agent_registry().clone();
    let workflow_factory = executor.workflow_factory().clone();
    // provider_manager returns Arc<bridge::ProviderManager>
    let bridge_provider_manager = executor.provider_manager();
    // Convert to core manager
    let provider_manager = bridge_provider_manager
        .create_core_manager_arc()
        .await
        .expect("Failed to create core provider manager");
    let provider_config = Arc::new(config.providers.clone());

    let kernel = start_embedded_kernel_with_executor(
        config.clone(),
        executor,
        KernelExecutionMode::Transport,
    )
    .await
    .expect("Failed to start kernel");

    // Setup Metrics
    let recorder_handle = PrometheusBuilder::new()
        .install_recorder()
        .expect("Failed to install Prometheus recorder");

    let runtime_config = Arc::new(tokio::sync::RwLock::new(
        llmspell_config::env::EnvRegistry::new(),
    ));
    let web_config = WebConfig::default();

    let state = AppState {
        kernel: Arc::new(tokio::sync::Mutex::new(kernel)),
        metrics_recorder: recorder_handle,
        config: web_config,
        runtime_config,
        static_config_path: None,
        config_store: None,
        tool_registry: Some(tool_registry),
        agent_registry: Some(agent_registry),
        workflow_factory: Some(workflow_factory),
        provider_manager: Some(provider_manager),
        provider_config: Some(provider_config),
    };

    (state, temp_dir)
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_template_launch_and_execution_flow() {
    // 1. Setup AppState
    let (state, _temp_dir) = setup_env().await;
    let app = WebServer::build_app(state);

    // 2. Launch a template (using a simple template if available, or just testing the flow logic)
    // We assume "research-assistant" or similar exists, or we mock it.
    // Since we are running in `llmspell-web` context, we rely on registered templates.
    // If no templates are registered by default in test env, we might need to register one manually?
    // AppState registration logic handles this.

    // Check available templates first to pick one
    let list_req = Request::builder()
        .uri("/api/templates")
        .body(Body::empty())
        .unwrap();
    let list_res = app.clone().oneshot(list_req).await.unwrap();
    assert_eq!(list_res.status(), StatusCode::OK);

    let body_bytes = axum::body::to_bytes(list_res.into_body(), usize::MAX)
        .await
        .unwrap();
    let templates: Vec<Value> = serde_json::from_slice(&body_bytes).unwrap();

    if templates.is_empty() {
        println!("No templates available for testing. Skipping execution test.");
        return;
    }

    let template_id = templates[0]["id"].as_str().unwrap();
    println!("Testing with template: {}", template_id);

    // 3. Launch Template
    let launch_payload = json!({
        "params": {
            "description": "Create a hello world function"
        },
        "session_id": "test-session-integration"
    });

    let launch_req = Request::builder()
        .method("POST")
        .uri(format!("/api/templates/{}/launch", template_id))
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_vec(&launch_payload).unwrap()))
        .unwrap();

    let launch_res = app.clone().oneshot(launch_req).await.unwrap();
    assert_eq!(launch_res.status(), StatusCode::OK);

    let launch_body: Value = serde_json::from_slice(
        &axum::body::to_bytes(launch_res.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();

    let session_id = launch_body["session_id"].as_str().unwrap();
    assert!(!session_id.is_empty());
    println!("Created session: {}", session_id);

    // 4. Poll Session Details
    // We expect status to be "running" then "completed" (or failed).
    // Execution happens in background tokio task.


    let mut attempts = 0;
    while attempts < 10 {
        // 10 * 500ms = 5s timeout
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;

        let details_req = Request::builder()
            .uri(format!("/api/sessions/{}/details", session_id))
            .body(Body::empty())
            .unwrap();

        let details_res = app.clone().oneshot(details_req).await.unwrap();
        if details_res.status() == StatusCode::OK {
            let details: Value = serde_json::from_slice(
                &axum::body::to_bytes(details_res.into_body(), usize::MAX)
                    .await
                    .unwrap(),
            )
            .unwrap();

            let status = details["status"].as_str().unwrap().to_string();
            println!("Poll {}: Status = {}", attempts, status);

            // Check workflow existence
            if let Some(workflow) = details.get("workflow") {
                if !workflow.is_null() {
                    let nodes = workflow["nodes"].as_array().unwrap();
                    println!("Workflow has {} nodes", nodes.len());
                }
            }

            if status == "completed" || status == "failed" {
                break;
            }
        }
        attempts += 1;
    }

    assert!(attempts < 10, "Timed out waiting for execution completion");

    // 5. Verify Workflow Graph
    let details_req = Request::builder()
        .uri(format!("/api/sessions/{}/details", session_id))
        .body(Body::empty())
        .unwrap();
    let details_res = app.clone().oneshot(details_req).await.unwrap();
    let details: Value = serde_json::from_slice(
        &axum::body::to_bytes(details_res.into_body(), usize::MAX)
            .await
            .unwrap(),
    )
    .unwrap();

    let workflow = details["workflow"].as_object().unwrap();
    let nodes = workflow["nodes"].as_array().unwrap();
    assert!(
        !nodes.is_empty(),
        "Workflow graph should have at least the root node"
    );

    println!("Test passed successfully!");
}
