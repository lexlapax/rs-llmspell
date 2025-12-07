use crate::config::WebConfig;
use crate::handlers::{self, health::health_check};
use crate::state::AppState;
use anyhow::Result;
use axum::{routing::{get, post}, Router};
use llmspell_kernel::api::KernelHandle;
use std::sync::Arc;
use tokio::signal;
use tokio::sync::Mutex;

pub struct WebServer;

use metrics_exporter_prometheus::PrometheusBuilder;
use crate::middleware::metrics::track_metrics;

impl WebServer {
    pub async fn run(config: WebConfig, kernel: KernelHandle) -> Result<()> {
        Self::run_with_custom_setup(config, kernel).await
    }

    pub async fn run_with_custom_setup(config: WebConfig, kernel: KernelHandle) -> Result<()> {
         // Setup metrics - this will panic if called twice (e.g. in multiple tests running in parallel if not careful)
         // For production usage this is fine.
         // For integration tests, we should construct State manually and call build_app directly.
         let recorder_handle = PrometheusBuilder::new()
            .install_recorder()
            .expect("failed to install Prometheus recorder");

        let state = AppState {
            kernel: Arc::new(Mutex::new(kernel)),
            metrics_recorder: recorder_handle,
            config: config.clone(),
        };

        let app = Self::build_app(state);

        let listener = tokio::net::TcpListener::bind(format!("{}:{}", config.host, config.port)).await?;
        axum::serve(listener, app).with_graceful_shutdown(shutdown_signal()).await?;
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
            .route("/templates/:id/launch", post(handlers::templates::launch_template))
            // Config API
            .route("/config", get(handlers::config::get_config).put(handlers::config::update_config))
            .layer(axum::middleware::from_fn_with_state(state.clone(), auth_middleware));

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
