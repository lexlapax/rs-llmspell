use anyhow::Result;
use async_trait::async_trait;
use llmspell_config::{providers::ProviderConfig, LLMSpellConfig};
use llmspell_core::traits::script_executor::{ScriptExecutionOutput, ScriptExecutor};
use llmspell_core::LLMSpellError;
use llmspell_events::bus::EventBus;
use llmspell_hooks::{HookExecutor, HookRegistry};
use llmspell_kernel::api::{
    start_embedded_kernel_with_executor, KernelExecutionMode, KernelHandle,
};
use llmspell_kernel::sessions::config::SessionManagerConfig;
use llmspell_kernel::sessions::SessionManager;
use llmspell_storage::MemoryBackend;
use llmspell_testing::state_helpers::create_test_state_manager;
use llmspell_web::config::WebConfig;
use llmspell_web::server::WebServer;
use llmspell_web::state::AppState;
use serde_json::json;
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;

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
            let _sm_clone = sm.clone();
        }
    }
    fn get_session_manager_any(&self) -> Option<Arc<dyn Any + Send + Sync>> {
        let guard = futures::executor::block_on(self.session_manager.lock());
        guard
            .as_ref()
            .map(|m| m.clone() as Arc<dyn Any + Send + Sync>)
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}

async fn setup_kernel_with_provider() -> Result<KernelHandle> {
    let state_manager = create_test_state_manager().await;
    let storage_backend = Arc::new(MemoryBackend::new());
    let hook_registry = Arc::new(HookRegistry::new());
    let hook_executor = Arc::new(HookExecutor::new());
    let event_bus = Arc::new(EventBus::new());
    let session_config_kernel = SessionManagerConfig {
        storage_path: std::path::PathBuf::from("/tmp/test-sessions-providers"),
        ..Default::default()
    };

    let session_manager = Arc::new(SessionManager::new(
        state_manager,
        storage_backend,
        hook_registry,
        hook_executor,
        &event_bus,
        session_config_kernel,
    )?);

    let executor = Arc::new(DummyScriptExecutor::new());
    executor.set_session_manager(session_manager.clone()).await;

    // Config with a test provider
    let mut config = LLMSpellConfig::default();

    let mut providers_map = HashMap::new();
    providers_map.insert(
        "test-provider".to_string(),
        ProviderConfig {
            provider_type: "ollama".to_string(), // Use omni/ollama as it doesn't need API key
            enabled: true,
            default_model: Some("test-model".to_string()),
            base_url: Some("http://localhost:11434".to_string()),
            api_key: None,
            api_key_env: None,
            timeout_seconds: Some(30),
            max_retries: Some(3),
            rate_limit: None,
            retry: None,
            temperature: None,
            max_tokens: None,
            options: HashMap::new(),
            name: "test-provider".to_string(),
        },
    );

    config.providers.providers = providers_map;

    start_embedded_kernel_with_executor(config, executor, KernelExecutionMode::Transport).await
}

#[tokio::test]
async fn test_list_providers_api() -> Result<()> {
    // 1. Setup Kernel with Provider
    let handle = setup_kernel_with_provider().await?;
    let handle_mutex = Arc::new(Mutex::new(handle));

    // Web Config
    let config = WebConfig {
        host: "127.0.0.1".to_string(),
        port: 0,
        auth_secret: "secret".to_string(),
        api_keys: vec!["test-key".to_string()],
        cors_origins: vec![],
        dev_mode: true,
    };

    // 2. Start Web Server
    let (tx, _rx) = tokio::sync::oneshot::channel();
    let listener = TcpListener::bind("127.0.0.1:0").await?;
    let addr = listener.local_addr()?;

    tokio::spawn(async move {
        let recorder_handle = metrics_exporter_prometheus::PrometheusBuilder::new()
            .install_recorder()
            .expect("Failed to install recorder");

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

    // 3. Authenticate
    let client = reqwest::Client::new();
    let base_url = format!("http://{}", addr);

    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    let login_resp = client
        .post(format!("{}/api/login", base_url))
        .json(&json!({ "api_key": "test-key" }))
        .send()
        .await?;

    let login_body: serde_json::Value = login_resp.json().await?;
    let token = login_body["token"].as_str().expect("Token missing");

    // 4. Test List Providers
    let resp = client
        .get(format!("{}/api/providers", base_url))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await?;

    assert!(resp.status().is_success());

    let body: serde_json::Value = resp.json().await?;
    println!("Providers response: {:?}", body);

    // Check status
    assert_eq!(body["status"], "ok");

    // Check providers list structure
    let providers = body["providers"]
        .as_array()
        .expect("providers is not an array");

    // Note: We cannot guarantee 'test-provider' is available because query_capabilities
    // might fail if the backend (e.g. Ollama) is not running.
    // However, getting "status": "ok" and an array confirms the API -> Kernel -> ProviderManager path works.
    println!("Found {} providers", providers.len());

    Ok(())
}
