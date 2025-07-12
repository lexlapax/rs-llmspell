//! ABOUTME: Command handler implementations
//! ABOUTME: Executes CLI commands with multi-engine support

pub mod exec;
pub mod info;
pub mod init;
pub mod keys;
pub mod providers;
pub mod repl;
pub mod run;
pub mod validate;

use crate::cli::{Commands, OutputFormat, ScriptEngine};
use anyhow::Result;
use llmspell_bridge::{RuntimeConfig, ScriptRuntime};

/// Execute a command with the given runtime configuration
pub async fn execute_command(
    command: Commands,
    engine: ScriptEngine,
    runtime_config: RuntimeConfig,
    output_format: OutputFormat,
) -> Result<()> {
    match command {
        Commands::Run {
            script,
            stream,
            args,
        } => {
            run::execute_script_file(script, engine, runtime_config, stream, args, output_format)
                .await
        }
        Commands::Exec { code, stream } => {
            exec::execute_inline_script(code, engine, runtime_config, stream, output_format).await
        }
        Commands::Repl { history } => repl::start_repl(engine, runtime_config, history).await,
        Commands::Providers { detailed } => {
            providers::list_providers(runtime_config, detailed, output_format).await
        }
        Commands::Validate { config } => validate::validate_config(config, output_format).await,
        Commands::Info { all } => info::show_engine_info(engine, all, output_format).await,
        Commands::Init { output, force } => init::init_config(output, force).await,
        Commands::Keys(keys_cmd) => keys::KeysCommand { command: keys_cmd }.execute().await,
    }
}

/// Create a script runtime for the specified engine
pub async fn create_runtime(engine: ScriptEngine, config: RuntimeConfig) -> Result<ScriptRuntime> {
    match engine {
        ScriptEngine::Lua => ScriptRuntime::new_with_lua(config)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to create Lua runtime: {}", e)),
        ScriptEngine::Javascript => {
            anyhow::bail!("JavaScript engine not available yet (coming in Phase 5)")
        }
        ScriptEngine::Python => {
            anyhow::bail!("Python engine not available yet (coming in Phase 9)")
        }
    }
}
