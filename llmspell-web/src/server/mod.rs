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

impl WebServer {
    pub async fn run(config: WebConfig, kernel: KernelHandle) -> Result<()> {
        let state = AppState {
            kernel: Arc::new(Mutex::new(kernel)),
        };

        let app = Router::new()
            .route("/health", get(health_check))
            .route("/api/scripts/execute", post(handlers::scripts::execute_script))
            .with_state(state);

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
