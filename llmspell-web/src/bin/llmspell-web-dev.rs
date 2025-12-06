use anyhow::Result;
use llmspell_bridge::ScriptRuntime;
use llmspell_config::LLMSpellConfig;
use llmspell_kernel::api::start_embedded_kernel_with_executor;
use llmspell_web::config::WebConfig;
use llmspell_web::server::WebServer;
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

    // 3. Start Kernel
    let kernel_handle = start_embedded_kernel_with_executor(config, executor)
        .await
        .expect("Failed to start kernel");

    // 4. Configure Web Server
    // 4. Configure Web Server
    let web_config = WebConfig {
        port: 3000,
        host: "127.0.0.1".to_string(),
        cors_origins: vec!["http://localhost:5173".to_string()],
        auth_secret: "dev_secret_do_not_use_in_prod".to_string(),
        api_keys: vec!["dev-key-123".to_string()],
    };

    println!("Starting dev server on http://{}:{}", web_config.host, web_config.port);

    // 5. Run Server
    WebServer::run(web_config, kernel_handle).await?;

    Ok(())
}
