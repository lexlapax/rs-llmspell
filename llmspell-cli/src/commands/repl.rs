//! ABOUTME: REPL command implementation for interactive scripting
//! ABOUTME: Provides an interactive read-eval-print loop

use crate::cli::ScriptEngine;
use anyhow::Result;
use llmspell_config::LLMSpellConfig;
use llmspell_kernel::api::start_embedded_kernel;
use llmspell_kernel::repl::{InteractiveSession, ReplSessionConfig};
use std::path::PathBuf;

/// Start an interactive REPL session
///
/// # Errors
///
/// Returns an error if the REPL session fails to start
pub async fn start_repl(
    engine: ScriptEngine,
    runtime_config: LLMSpellConfig,
    history_file: Option<PathBuf>,
) -> Result<()> {
    println!("LLMSpell REPL - {} engine", engine.as_str());
    println!("Type '.exit' or press Ctrl+D to quit");
    println!();

    // Create kernel handle
    let kernel_handle = start_embedded_kernel(runtime_config).await?;

    // Create session configuration
    let mut session_config = ReplSessionConfig::default();
    if let Some(path) = history_file {
        session_config.history_file = Some(path);
    }

    // Create interactive session
    let kernel = kernel_handle.into_kernel();
    let mut session = InteractiveSession::new(kernel, session_config)?;

    // Run REPL loop
    session.run_repl().await?;

    Ok(())
}
