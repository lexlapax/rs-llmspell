use crate::config::WebConfig;
use crate::handlers::{self, health::health_check};
use crate::state::AppState;
use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};
use llmspell_core::traits::storage::StorageBackend;
use llmspell_kernel::api::KernelHandle;
use std::sync::Arc;
use tokio::signal;
use tokio::sync::Mutex;

pub struct WebServer;

use crate::middleware::metrics::track_metrics;
use metrics_exporter_prometheus::PrometheusBuilder;

impl WebServer {
    pub async fn run(
        config: WebConfig,
        kernel: KernelHandle,
        config_path: Option<std::path::PathBuf>,
    ) -> Result<()> {
        Self::run_with_custom_setup(config, kernel, config_path).await
    }

    pub async fn run_with_custom_setup(
        config: WebConfig,
        kernel: KernelHandle,
        config_path: Option<std::path::PathBuf>,
    ) -> Result<()> {
        // Setup metrics - this will panic if called twice (e.g. in multiple tests running in parallel if not careful)
        // For production usage this is fine.
        // For integration tests, we should construct State manually and call build_app directly.
        let recorder_handle = PrometheusBuilder::new()
            .install_recorder()
            .expect("failed to install Prometheus recorder");

        // Initialize Shared Runtime Config
        let runtime_config = llmspell_config::env::EnvRegistry::new();

        // Register known variables for Web UI Management (Task 14.5.1e)
        use llmspell_config::env::{EnvCategory, EnvVarDefBuilder};
        let _ = runtime_config.register_var(
            EnvVarDefBuilder::new("RUST_LOG")
                .description("Rust logging level/filter")
                .category(EnvCategory::Runtime)
                .default("info")
                .build(),
        );
        let _ = runtime_config.register_var(
            EnvVarDefBuilder::new("TEST_PERSIST_VAR")
                .description("Test variable for persistence verification")
                .category(EnvCategory::Runtime)
                .build(),
        );

        // Ignoring error on load_from_env as it's best-effort
        let _ = runtime_config.load_from_env();

        // Initialize Storage for Config Persistence (Task 14.5.1e)
        // Assume default DB path for now, matching kernel default if possible
        // Ideally this should come from config or shared constant
        let db_path = "llmspell.db";
        let sqlite_config = llmspell_storage::backends::sqlite::SqliteConfig::new(db_path);

        let config_store =
            match llmspell_storage::backends::sqlite::SqliteBackend::new(sqlite_config).await {
                Ok(backend) => {
                    let backend = Arc::new(backend);
                    // Ensure kv_store tables exist (V7 migration)
                    // In a real app we might want centralized migration management
                    if let Err(e) = backend.run_migrations().await {
                        tracing::warn!("Failed to run storage migrations: {}", e);
                    }

                    let store = Arc::new(llmspell_storage::backends::sqlite::SqliteKVStorage::new(
                        backend,
                        "system".to_string(),
                    ));

                    // Load persisted configuration overrides
                    if let Ok(keys) = store.list_keys("config:").await {
                        if !keys.is_empty() {
                            tracing::info!(
                                "Loading {} persisted configuration overrides from SQLite",
                                keys.len()
                            );
                            if let Ok(values) = store.get_batch(&keys).await {
                                let mut overrides = std::collections::HashMap::new();
                                for (k, v) in values {
                                    if let Ok(val_str) = String::from_utf8(v) {
                                        // Key format "config:VAR_NAME" -> "VAR_NAME"
                                        let var_name = k.trim_start_matches("config:");
                                        std::env::set_var(var_name, &val_str);
                                        overrides.insert(var_name.to_string(), val_str);
                                    }
                                }
                                // Sync registry with persisted overrides
                                let _ = runtime_config.with_overrides(overrides);
                            }
                        }
                    }

                    Some(store)
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to initialize SQLite storage for config persistence: {}",
                        e
                    );
                    None
                }
            };

        let runtime_config = Arc::new(tokio::sync::RwLock::new(runtime_config));

        let state = AppState {
            kernel: Arc::new(Mutex::new(kernel)),
            metrics_recorder: recorder_handle,
            config: config.clone(),
            runtime_config,
            config_store,
            static_config_path: config_path,
        };

        let app = Self::build_app(state);

        let listener =
            tokio::net::TcpListener::bind(format!("{}:{}", config.host, config.port)).await?;
        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal())
            .await?;
        Ok(())
    }

    pub fn build_app(state: AppState) -> Router {
        use crate::middleware::auth::auth_middleware;

        let api_routes = Router::new()
            .route("/scripts/execute", post(handlers::scripts::execute_script))
            .route("/sessions", get(handlers::sessions::list_sessions))
            .route("/sessions/:id", get(handlers::sessions::get_session))
            .route("/memory/search", get(handlers::memory::search_memory))
            .route("/agents", get(handlers::agents::list_agents))
            .route("/agents/:id/execute", post(handlers::agents::execute_agent))
            .route("/tools", get(handlers::tools::list_tools))
            .route("/tools/:id/execute", post(handlers::tools::execute_tool))
            // Templates API
            .route("/templates", get(handlers::templates::list_templates))
            .route("/templates/:id", get(handlers::templates::get_template))
            .route(
                "/templates/:id/launch",
                post(handlers::templates::launch_template),
            )
            // Config API
            .route(
                "/config",
                get(handlers::config::get_config).put(handlers::config::update_config),
            )
            .route(
                "/config/source",
                get(handlers::static_config::get_config_source)
                    .put(handlers::static_config::update_config_source),
            )
            .route(
                "/config/schema",
                get(handlers::static_config::get_config_schema),
            )
            .layer(axum::middleware::from_fn_with_state(
                state.clone(),
                auth_middleware,
            ));

        Router::new()
            .route("/health", get(health_check))
            .route("/metrics", get(handlers::metrics::get_metrics))
            .route("/login", post(handlers::auth::login))
            .route("/ws/stream", get(handlers::ws::ws_handler))
            .nest("/api", api_routes)
            .layer(axum::middleware::from_fn(track_metrics))
            .with_state(state)
            .fallback(handlers::assets::static_handler)
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("Signal received, starting graceful shutdown");
}
