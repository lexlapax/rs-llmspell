use anyhow::Result;
use llmspell_bridge::ScriptRuntime;
use llmspell_config::LLMSpellConfig;
use llmspell_kernel::api::{start_embedded_kernel_with_executor, KernelExecutionMode};
use llmspell_web::{
    config::WebConfig,
    server::{WebDependencies, WebServer},
};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    // 1. Create Config
    let config = LLMSpellConfig::default();

    // 2. Create Runtime
    let runtime = ScriptRuntime::new(config.clone())
        .await
        .expect("Failed to create runtime");
    let executor = Arc::new(runtime);

    // 3. Start Kernel (Transport mode for web interface)
    let kernel_handle = start_embedded_kernel_with_executor(
        config.clone(),
        executor.clone(),
        KernelExecutionMode::Transport, // Web uses Transport mode
    )
    .await
    .expect("Failed to start kernel");

    // 4. Configure Web Server
    // Extract registries
    let dependencies = WebDependencies {
        tool_registry: Some(executor.tool_registry().clone()),
        agent_registry: Some(executor.agent_registry().clone()),
        workflow_factory: Some(executor.workflow_factory().clone()),
        provider_manager: Some(
            executor
                .provider_manager()
                .create_core_manager_arc()
                .await
                .expect("Failed to create provider manager"),
        ),
        provider_config: Some(Arc::new(config.providers.clone())),
    };

    let port = 3000;
    let host = "127.0.0.1".to_string();

    // 4. Configure Web Server
    let web_config = WebConfig {
        port,
        host,
        cors_origins: vec!["http://localhost:5173".to_string()],
        auth_secret: "dev_secret_do_not_use_in_prod".to_string(),
        api_keys: vec!["dev-key-123".to_string()],
        dev_mode: true,
    };

    println!(
        "Starting dev server on http://{}:{}",
        web_config.host, web_config.port
    );

    // 5. Run Server
    WebServer::run(web_config, kernel_handle, None, dependencies).await?;

    Ok(())
}
