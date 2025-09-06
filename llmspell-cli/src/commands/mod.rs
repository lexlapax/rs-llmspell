//! ABOUTME: Command handler implementations
//! ABOUTME: Executes CLI commands with multi-engine support

pub mod apps;
pub mod backup;
pub mod debug;
pub mod exec;
pub mod info;
pub mod init;
pub mod kernel;
pub mod keys;
pub mod providers;
pub mod repl;
pub mod run;
pub mod setup;
pub mod validate;

use crate::cli::{Commands, OutputFormat, ScriptEngine};
use crate::kernel_client::KernelConnectionTrait;
use anyhow::Result;
use llmspell_config::LLMSpellConfig;
use std::path::PathBuf;

/// RAG configuration options from command line
#[derive(Debug, Default)]
pub struct RagOptions {
    pub rag: bool,
    pub no_rag: bool,
    pub rag_config: Option<PathBuf>,
    pub rag_dims: Option<usize>,
    pub rag_backend: Option<String>,
}

impl RagOptions {
    /// Apply RAG options to configuration
    pub async fn apply_to_config(&self, config: &mut LLMSpellConfig) -> Result<()> {
        // Handle explicit enable/disable
        if self.no_rag {
            config.rag.enabled = false;
        } else if self.rag {
            config.rag.enabled = true;
        }

        // Load custom RAG config file if specified
        if let Some(rag_config_path) = &self.rag_config {
            let rag_config_str = tokio::fs::read_to_string(rag_config_path).await?;
            let rag_config: llmspell_config::RAGConfig = toml::from_str(&rag_config_str)
                .map_err(|e| anyhow::anyhow!("Failed to parse RAG config: {}", e))?;
            config.rag = rag_config;
        }

        // Apply individual overrides
        if let Some(dims) = self.rag_dims {
            config.rag.vector_storage.dimensions = dims;
        }

        if let Some(backend) = &self.rag_backend {
            config.rag.vector_storage.backend = match backend.to_lowercase().as_str() {
                "hnsw" => llmspell_config::VectorBackend::HNSW,
                _ => anyhow::bail!("Unknown RAG backend: {}. Only 'hnsw' is available", backend),
            };
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
            debug,
            rag,
            no_rag,
            rag_config,
            rag_dims,
            rag_backend,
            args,
        } => {
            // Apply RAG options to config
            let mut runtime_config = runtime_config;
            let rag_options = RagOptions {
                rag,
                no_rag,
                rag_config,
                rag_dims,
                rag_backend,
            };
            rag_options.apply_to_config(&mut runtime_config).await?;

            run::execute_script_file(
                script,
                engine,
                runtime_config,
                connect,
                stream,
                args,
                output_format,
                debug,
            )
            .await
        }
        Commands::Exec {
            code,
            connect,
            stream,
            debug,
            rag,
            no_rag,
            rag_config,
            rag_dims,
            rag_backend,
        } => {
            // Apply RAG options to config
            let mut runtime_config = runtime_config;
            let rag_options = RagOptions {
                rag,
                no_rag,
                rag_config,
                rag_dims,
                rag_backend,
            };
            rag_options.apply_to_config(&mut runtime_config).await?;

            exec::execute_inline_script(
                code,
                engine,
                runtime_config,
                connect,
                stream,
                debug,
                output_format,
            )
            .await
        }
        Commands::Repl { connect, history } => {
            repl::start_repl(engine, runtime_config, connect, history).await
        }
        Commands::Providers { detailed } => {
            providers::list_providers(runtime_config, detailed, output_format).await
        }
        Commands::Validate { config } => validate::validate_config(config, output_format).await,
        Commands::Info { all } => info::show_engine_info(engine, all, output_format).await,
        Commands::Init { output, force } => init::init_config(output, force).await,
        Commands::Keys(keys_cmd) => keys::KeysCommand { command: keys_cmd }.execute().await,
        Commands::Backup(backup_cmd) => {
            backup::execute_backup(backup_cmd, &runtime_config, output_format).await
        }
        Commands::Apps { app } => {
            apps::execute_apps_command(app, engine, runtime_config, output_format).await
        }
        Commands::Setup { force } => setup::run_interactive_setup(force).await,
        Commands::Debug { script, args } => {
            // Use dedicated DebugBridge architecture for debug command
            debug::handle_debug_command(script, args, engine, runtime_config, output_format).await
        }
        Commands::Kernel {
            port,
            id,
            connection_file,
        } => kernel::start_kernel(engine, port, id, connection_file, runtime_config).await,
    }
}

/// Create a kernel connection based on the connect flag
/// Uses embedded kernel by default, or connects to external kernel if specified
pub async fn create_kernel_connection(
    config: LLMSpellConfig,
    connect: Option<String>,
) -> Result<Box<dyn crate::kernel_client::KernelConnectionTrait>> {
    use std::sync::Arc;

    if let Some(connection) = connect {
        // TODO: Connect to existing external kernel via ZeroMQ
        // This will require implementing ZmqKernelClient properly
        anyhow::bail!(
            "External kernel connection not yet implemented.\n\
            Connection string provided: {}",
            connection
        );
    } else {
        // Default: Use embedded kernel (runs in same process)
        let mut kernel = crate::kernel_client::EmbeddedKernel::new(Arc::new(config)).await?;
        kernel.connect_or_start().await?;
        Ok(Box::new(kernel))
    }
}
