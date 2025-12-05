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
        let recorder_handle = PrometheusBuilder::new()
            .install_recorder()
            .expect("failed to install Prometheus recorder");

        let state = AppState {
            kernel: Arc::new(Mutex::new(kernel)),
            metrics_recorder: recorder_handle,
        };

        let app = Router::new()
            .route("/health", get(health_check))
            .route("/metrics", get(handlers::metrics::get_metrics))
            .route("/api/scripts/execute", post(handlers::scripts::execute_script))
            .route("/ws/stream", get(handlers::ws::ws_handler))
            .route("/api/sessions", get(handlers::sessions::list_sessions))
            .route("/api/sessions/:id", get(handlers::sessions::get_session))
            .route("/api/memory/search", get(handlers::memory::search_memory))
            .route("/api/agents", get(handlers::agents::list_agents))
            .route("/api/agents/:id/execute", post(handlers::agents::execute_agent))
            .route("/api/tools", get(handlers::tools::list_tools))
            .route("/api/tools/:id/execute", post(handlers::tools::execute_tool))
            .layer(axum::middleware::from_fn(track_metrics))
            .with_state(state)
            .fallback(handlers::assets::static_handler);

        let listener = tokio::net::TcpListener::bind(format!("{}:{}", config.host, config.port)).await?;
        axum::serve(listener, app).with_graceful_shutdown(shutdown_signal()).await?;
        Ok(())
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
