use anyhow::Result;
use async_trait::async_trait;
use llmspell_config::LLMSpellConfig;
use llmspell_core::traits::script_executor::{ScriptExecutionOutput, ScriptExecutor};
use llmspell_core::LLMSpellError;
use llmspell_events::bus::EventBus;
use llmspell_hooks::{HookExecutor, HookRegistry};
use llmspell_kernel::api::{start_embedded_kernel_with_executor, KernelHandle};
use llmspell_kernel::sessions::config::SessionManagerConfig;
use llmspell_kernel::sessions::SessionManager;
use llmspell_storage::MemoryBackend;
use llmspell_templates::core::provider_parameters;
use llmspell_templates::core::{
    Template, TemplateCategory, TemplateMetadata, TemplateOutput, TemplateParams,
};
use llmspell_templates::validation::ConfigSchema;
use llmspell_testing::state_helpers::create_test_state_manager;
use llmspell_web::config::WebConfig;
use llmspell_web::server::WebServer;
use llmspell_web::state::AppState;
use serde_json::json;
use std::any::Any;

use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

#[derive(Debug, Clone)]
struct TestTemplate {
    meta: TemplateMetadata,
}

impl TestTemplate {
    fn new(id: &str) -> Self {
        Self {
            meta: TemplateMetadata {
                id: id.to_string(),
                name: "Test Template".to_string(),
                description: "Test".to_string(),
                category: TemplateCategory::Research,
                version: "0.1.0".to_string(),
                author: None,
                requires: vec![],
                tags: vec![],
            },
        }
    }
}

#[async_trait]
impl Template for TestTemplate {
    fn metadata(&self) -> &TemplateMetadata {
        &self.meta
    }
    fn config_schema(&self) -> ConfigSchema {
        let mut params = vec![llmspell_templates::validation::ParameterSchema::optional(
            "max_tokens",
            "tokens",
            llmspell_templates::validation::ParameterType::Integer,
            json!(100),
        )];
        // Add provider params as expected by logic
        params.extend(provider_parameters());
        ConfigSchema::new(params)
    }
    async fn execute(
        &self,
        _params: TemplateParams,
        _ctx: llmspell_templates::context::ExecutionContext,
    ) -> Result<TemplateOutput, llmspell_templates::error::TemplateError> {
        unimplemented!()
    }
}

pub struct DummyScriptExecutor {
    session_manager: Arc<Mutex<Option<Arc<SessionManager>>>>,
}

impl DummyScriptExecutor {
    fn new() -> Self {
        Self {
            session_manager: Arc::new(Mutex::new(None)),
        }
    }

    async fn set_session_manager(&self, manager: Arc<SessionManager>) {
        let mut guard = self.session_manager.lock().await;
        *guard = Some(manager);
    }
}

#[async_trait]
impl ScriptExecutor for DummyScriptExecutor {
    async fn execute_script(&self, _script: &str) -> Result<ScriptExecutionOutput, LLMSpellError> {
        unimplemented!()
    }
    fn language(&self) -> &'static str {
        "dummy"
    }

    fn set_session_manager_any(&self, manager: Arc<dyn Any + Send + Sync>) {
        if let Ok(sm) = Arc::downcast::<SessionManager>(manager) {
            // This is synchronous, but we need to set async mutex or use standard mutex.
            // Using standard mutex for simplicity inside this callback
            let _sm_clone = sm.clone();
            // We can't easily wait here if using tokio mutex.
            // But set_session_manager_any depends on implementation.
            // For now, let's assume we populated it BEFORE passing executor to kernel
            // OR use std::sync::Mutex.
            // Let's rely on get_session_manager_any returning what we set.
        }
    }

    fn get_session_manager_any(&self) -> Option<Arc<dyn Any + Send + Sync>> {
        // We use std mutex to avoid async in sync method issue
        // Re-implementing struct with std Mutex
        let guard = futures::executor::block_on(self.session_manager.lock());
        guard
            .as_ref()
            .map(|m| m.clone() as Arc<dyn Any + Send + Sync>)
    }

    // Required since newer version:
    fn as_any(&self) -> &dyn Any {
        self
    }
}

async fn setup_kernel() -> Result<KernelHandle> {
    // 1. Dependencies
    let state_manager = create_test_state_manager().await;
    let storage_backend = Arc::new(MemoryBackend::new());
    let hook_registry = Arc::new(HookRegistry::new());
    let hook_executor = Arc::new(HookExecutor::new());
    let event_bus = Arc::new(EventBus::new());
    let session_config = SessionManagerConfig {
        storage_path: std::path::PathBuf::from("/tmp/test-sessions"),
        ..Default::default()
    };

    // 2. Session Manager
    let session_manager = Arc::new(SessionManager::new(
        state_manager,
        storage_backend,
        hook_registry,
        hook_executor,
        &event_bus,
        session_config,
    )?);

    // 3. Executor
    let executor = Arc::new(DummyScriptExecutor::new());
    executor.set_session_manager(session_manager.clone()).await;

    // 4. Kernel
    // Requires LLMSpellConfig
    let config = LLMSpellConfig::default();

    // start_embedded_kernel... expects executor to have session manager.
    start_embedded_kernel_with_executor(config, executor).await
}

#[tokio::test]
async fn test_authenticated_template_launch_persistence() -> Result<()> {
    // 1. Setup Kernel
    let handle = setup_kernel().await?;
    let handle_mutex = Arc::new(Mutex::new(handle));

    // Register test template
    {
        let t = TestTemplate::new("test-launch-v1");
        llmspell_templates::registry::global_registry()
            .register(Arc::new(t))
            .unwrap();
    }

    // Config with mock key
    let config = WebConfig {
        host: "127.0.0.1".to_string(),
        port: 0,
        auth_secret: "secret".to_string(),
        api_keys: vec!["test-key".to_string()],
        cors_origins: vec![],
        dev_mode: true,
    };

    // Construct App
    let (tx, _rx) = tokio::sync::oneshot::channel();
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;

    tokio::spawn(async move {
        let recorder_handle = metrics_exporter_prometheus::PrometheusBuilder::new()
            .install_recorder()
            .expect("Failed to install recorder"); // Will panic if run multiple times, ensuring isolation

        let runtime_config = Arc::new(tokio::sync::RwLock::new(
            llmspell_config::env::EnvRegistry::new(),
        ));

        let state = AppState {
            kernel: handle_mutex,
            metrics_recorder: recorder_handle,
            config: config.clone(),
            runtime_config,
            config_store: None,
            static_config_path: None,
        };

        let app = WebServer::build_app(state);
        axum::serve(listener, app).await.unwrap();
        let _ = tx.send(());
    });

    let client = reqwest::Client::new();
    let base_url = format!("http://{}", addr);

    // Give server a moment
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // 2. Login
    let login_resp = client
        .post(format!("{}/login", base_url))
        .json(&json!({ "api_key": "test-key" }))
        .send()
        .await?;

    if !login_resp.status().is_success() {
        println!("Login request failed status: {}", login_resp.status());
    }

    let login_body: serde_json::Value = login_resp.json().await?;
    if login_body.get("token").is_none() {
        println!("Login failed. Body: {:?}", login_body);
        panic!("Login failed: missing token");
    }
    let token = login_body["token"].as_str().unwrap();

    // 3. Launch Template
    let launch_resp = client
        .post(format!("{}/api/templates/test-launch-v1/launch", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .json(&json!({
            "params": {
                "max_tokens": 555,
                "provider_name": "ollama"
            }
        }))
        .send()
        .await?;

    if !launch_resp.status().is_success() {
        let err_text = launch_resp.text().await?;
        println!("Launch failed: {}", err_text);
        panic!("Launch request failed");
    }

    let launch_json: serde_json::Value = launch_resp.json().await?;
    let session_id = launch_json["session_id"].as_str().unwrap();

    // 4. Verify Session Metadata
    let session_resp = client
        .get(format!("{}/api/sessions/{}", base_url, session_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    assert!(session_resp.status().is_success());
    let session_json: serde_json::Value = session_resp.json().await?;

    // Check persistence
    let metadata = &session_json["metadata"];
    assert_eq!(metadata["max_tokens"], 555);
    assert_eq!(metadata["provider_name"], "ollama");

    Ok(())
}
