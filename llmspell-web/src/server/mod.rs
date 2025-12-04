use crate::config::WebConfig;
use crate::handlers::health::health_check;
use anyhow::Result;
use axum::{routing::get, Router};
use tokio::net::TcpListener;
use tokio::signal;

pub struct WebServer;

impl WebServer {
    pub async fn run(config: WebConfig) -> Result<()> {
        let app = Router::new().route("/health", get(health_check));

        let addr = format!("{}:{}", config.host, config.port);
        println!("Listening on {}", addr);

        let listener = TcpListener::bind(&addr).await?;
        axum::serve(listener, app)
            .with_graceful_shutdown(shutdown_signal())
            .await?;

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
