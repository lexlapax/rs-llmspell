use anyhow::Result;
use llmspell_bridge::ScriptRuntime;
use llmspell_config::LLMSpellConfig;
use llmspell_kernel::api::{start_embedded_kernel_with_executor, KernelExecutionMode};
use llmspell_web::{
    config::WebConfig,
    server::{WebDependencies, WebServer},
};
use std::sync::Arc;
use tokio::signal;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // 1. Create Config
    let config = LLMSpellConfig::default();

    // Perform startup pre-flight check on database
    if let Some(db_path) = &config.storage.database_path {
        tracing::info!("Performing startup database check on: {:?}", db_path);
        if let Err(e) = perform_startup_check(db_path) {
            tracing::error!("Startup check failed: {}", e);
            // We continue anyway, hoping the main pool can recover, or fail loudly there
        } else {
            tracing::info!("Startup database check passed");
        }
    }

    // 2. Create Runtime
    let runtime = ScriptRuntime::new(config.clone())
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create runtime: {}", e))?;

    let runtime = Arc::new(runtime);

    // 3. Start Kernel (Transport mode for web interface)
    let kernel_handle = start_embedded_kernel_with_executor(
        config.clone(),
        runtime.clone(), // runtime implements ScriptExecutor
        KernelExecutionMode::Transport,
    )
    .await
    .map_err(|e| anyhow::anyhow!("Failed to start kernel: {}", e))?;

    // 4. Extract registries and create dependencies
    // Use runtime accessors properly and extract core manager for WebDependencies
    let core_provider_manager = runtime
        .provider_manager()
        .create_core_manager_arc()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to create core provider manager: {}", e))?;

    let dependencies = WebDependencies {
        provider_manager: Some(core_provider_manager),
        tool_registry: Some(runtime.tool_registry().clone()),
        agent_registry: Some(runtime.agent_registry().clone()),
        workflow_factory: Some(runtime.workflow_factory().clone()),
        provider_config: Some(Arc::new(config.providers.clone())),
    };

    // 5. Start Server with Graceful Shutdown
    tracing::info!("Starting dev server...");
    let server_config = WebConfig {
        port: 3000,
        host: "127.0.0.1".to_string(),
        cors_origins: vec!["http://localhost:5173".to_string()],
        auth_secret: "dev_secret_do_not_use_in_prod".to_string(),
        api_keys: vec!["dev-key-123".to_string()],
        dev_mode: true,
    };

    tracing::info!(
        "Starting dev server on http://{}:{}",
        server_config.host,
        server_config.port
    );

    tokio::select! {
        result = WebServer::run_with_dependencies(server_config, kernel_handle, None, dependencies) => {
            if let Err(e) = result {
                tracing::error!("Server error: {}", e);
            }
        }
        _ = signal::ctrl_c() => {
            tracing::info!("Shutdown signal received");
        }
    }

    tracing::info!("Shutting down...");

    Ok(())
}

/// Perform low-level startup check and repair on SQLite database
fn perform_startup_check(path: &std::path::Path) -> anyhow::Result<()> {
    // Ensure directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Connect with rusqlite directly
    let conn = rusqlite::Connection::open(path)?;

    // Set explicit busy timeout (5s) to allow recovery from stale locks
    conn.busy_timeout(std::time::Duration::from_millis(5000))?;

    // Force WAL mode (idempotent, ensures correct mode)
    let journal_mode: String = conn.query_row("PRAGMA journal_mode=WAL;", [], |row| row.get(0))?;

    if journal_mode != "wal" {
        tracing::warn!("Failed to set WAL mode, got: {}", journal_mode);
    }

    // Perform passive checkpoint to clean up WAL and synchronize
    // This helps resolve "database is locked" if previous process crashed leaving WAL state
    conn.execute_batch("PRAGMA wal_checkpoint(PASSIVE);")?;

    Ok(())
}
