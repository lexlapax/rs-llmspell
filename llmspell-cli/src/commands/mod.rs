//! ABOUTME: Command handler implementations
//! ABOUTME: Executes CLI commands with multi-engine support

pub mod apps;
pub mod backup;
pub mod config;
pub mod debug;
pub mod exec;
pub mod info;
pub mod init;
pub mod kernel;
pub mod keys;
pub mod providers;
pub mod repl;
pub mod run;
pub mod session;
pub mod setup;
pub mod state;
pub mod validate;

use crate::cli::{Commands, OutputFormat, ScriptEngine};
use anyhow::Result;
use llmspell_config::LLMSpellConfig;

/// RAG configuration options from command line
#[derive(Debug, Default)]
pub struct RagOptions {
    pub rag_profile: Option<String>,
}

impl RagOptions {
    /// Apply RAG options to configuration
    pub async fn apply_to_config(&self, config: &mut LLMSpellConfig) -> Result<()> {
        // Apply RAG profile if specified
        if let Some(profile_name) = &self.rag_profile {
            if let Some(profile) = config.rag.profiles.get(profile_name).cloned() {
                profile.apply_to_config(&mut config.rag);

                // Load custom config file if profile specifies one
                if let Some(config_file) = &profile.config_file {
                    let rag_config_str = tokio::fs::read_to_string(config_file).await?;
                    let rag_config: llmspell_config::RAGConfig = toml::from_str(&rag_config_str)
                        .map_err(|e| anyhow::anyhow!("Failed to parse RAG config: {}", e))?;
                    config.rag = rag_config;
                }
            } else {
                anyhow::bail!("RAG profile '{}' not found in configuration", profile_name);
            }
        }

        Ok(())
    }
}

/// Execute a command with the given runtime configuration
pub async fn execute_command(
    command: Commands,
    engine: ScriptEngine,
    runtime_config: LLMSpellConfig,
    output_format: OutputFormat,
) -> Result<()> {
    // Use LLMSpellConfig directly now that bridge accepts it

    match command {
        Commands::Run {
            script,
            connect,
            stream,
            rag_profile,
            args,
        } => {
            // Apply RAG options to config
            let mut runtime_config = runtime_config;
            let rag_options = RagOptions { rag_profile };
            rag_options.apply_to_config(&mut runtime_config).await?;

            run::execute_script_file(
                script,
                engine,
                runtime_config,
                connect,
                stream,
                args,
                output_format,
                false, // debug mode removed - use dedicated Debug command
            )
            .await
        }
        Commands::Exec {
            code,
            connect,
            stream,
            rag_profile,
        } => {
            // Apply RAG options to config
            let mut runtime_config = runtime_config;
            let rag_options = RagOptions { rag_profile };
            rag_options.apply_to_config(&mut runtime_config).await?;

            exec::execute_inline_script(
                code,
                engine,
                runtime_config,
                connect,
                stream,
                false, // debug mode removed - use dedicated Debug command
                output_format,
            )
            .await
        }
        Commands::Repl {
            connect,
            history,
            rag_profile,
        } => {
            // Apply RAG options to config
            let mut runtime_config = runtime_config;
            let rag_options = RagOptions { rag_profile };
            rag_options.apply_to_config(&mut runtime_config).await?;

            repl::start_repl(engine, runtime_config, connect, history).await
        }
        Commands::Providers { detailed } => {
            providers::list_providers(runtime_config, detailed, output_format).await
        }
        Commands::Info { all } => info::show_engine_info(engine, all, output_format).await,
        Commands::Keys(keys_cmd) => keys::KeysCommand { command: keys_cmd }.execute().await,
        Commands::Backup(backup_cmd) => {
            backup::execute_backup(backup_cmd, &runtime_config, output_format).await
        }
        Commands::Apps { app } => {
            apps::execute_apps_command(app, engine, runtime_config, output_format).await
        }
        Commands::Setup { force } => setup::run_interactive_setup(force).await,
        Commands::State { command } => {
            state::handle_state_command(command, runtime_config, output_format).await
        }
        Commands::Session { command } => {
            session::handle_session_command(command, runtime_config, output_format).await
        }
        Commands::Config { command } => config::handle_config_command(command, output_format).await,
        Commands::Debug {
            script,
            break_at,
            port,
            rag_profile,
            args,
        } => {
            // Apply RAG options to config
            let mut runtime_config = runtime_config;
            let rag_options = RagOptions { rag_profile };
            rag_options.apply_to_config(&mut runtime_config).await?;

            // Use dedicated DebugBridge architecture for debug command
            debug::handle_debug_command(
                script,
                break_at,
                port,
                args,
                engine,
                runtime_config,
                output_format,
            )
            .await
        }
        Commands::Kernel { command } => {
            kernel::handle_kernel_command(command, engine, runtime_config, output_format).await
        }
    }
}

/// Create a kernel connection based on the connect flag
/// Uses embedded kernel by default, or connects to external kernel if specified
pub async fn create_kernel_connection(
    config: LLMSpellConfig,
    connect: Option<String>,
) -> Result<Box<dyn crate::kernel_client::KernelConnectionTrait>> {
    use crate::kernel_client::UnifiedKernelClient;
    use std::sync::Arc;

    let kernel = if let Some(connection) = connect {
        // Connect to external kernel
        UnifiedKernelClient::connect_external(connection).await?
    } else {
        // Start embedded kernel
        UnifiedKernelClient::start_embedded(Arc::new(config)).await?
    };

    Ok(Box::new(kernel))
}
