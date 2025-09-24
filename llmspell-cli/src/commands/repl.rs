//! ABOUTME: REPL command implementation for interactive scripting
//! ABOUTME: Provides an interactive read-eval-print loop

use crate::cli::{OutputFormat, ScriptEngine};
use crate::execution_context::ExecutionContext;
use anyhow::Result;
use llmspell_kernel::api::{ClientHandle, KernelHandle};
use llmspell_kernel::repl::{InteractiveSession, ReplSessionConfig};
use std::path::PathBuf;

/// Start interactive REPL session
pub async fn start_repl(
    engine: ScriptEngine,
    context: ExecutionContext,
    history: Option<PathBuf>,
    output_format: OutputFormat,
) -> Result<()> {
    println!("LLMSpell REPL - {} engine", engine.as_str());
    println!("Type '.exit' or press Ctrl+D to quit");
    println!();

    // Create session configuration
    let mut session_config = ReplSessionConfig::default();
    if let Some(path) = history {
        session_config.history_file = Some(path);
    }

    // Execute based on context type
    match context {
        ExecutionContext::Embedded { handle, config: _ } => {
            start_embedded_repl(*handle, session_config, engine, output_format).await?;
        }
        ExecutionContext::Connected { handle, address: _ } => {
            start_connected_repl(handle, session_config, engine, output_format).await?;
        }
    }

    Ok(())
}

/// Start REPL with embedded kernel
async fn start_embedded_repl(
    handle: KernelHandle,
    session_config: ReplSessionConfig,
    _engine: ScriptEngine,
    _output_format: OutputFormat,
) -> Result<()> {
    // Create interactive session
    let kernel = handle.into_kernel();
    let mut session = InteractiveSession::new(kernel, session_config).await?;

    // Run REPL loop
    session.run_repl().await?;

    Ok(())
}

/// Start REPL with connected kernel
async fn start_connected_repl(
    _handle: ClientHandle,
    _session_config: ReplSessionConfig,
    engine: ScriptEngine,
    output_format: OutputFormat,
) -> Result<()> {
    // For connected mode, use the client handle differently
    // For now, simulate REPL mode
    println!("Connected REPL mode not yet fully implemented");
    println!("This would connect to the remote kernel for interactive sessions");

    match output_format {
        OutputFormat::Json => {
            println!(
                "{}",
                serde_json::json!({
                    "status": "repl_started",
                    "mode": "connected",
                    "engine": engine.as_str(),
                    "message": "Connected REPL mode placeholder"
                })
            );
        }
        OutputFormat::Yaml => {
            let data = serde_json::json!({
                "status": "repl_started",
                "mode": "connected",
                "engine": engine.as_str(),
                "message": "Connected REPL mode placeholder"
            });
            println!("{}", serde_yaml::to_string(&data)?);
        }
        _ => {
            println!("Starting interactive REPL with connected kernel...");
        }
    }

    Ok(())
}
