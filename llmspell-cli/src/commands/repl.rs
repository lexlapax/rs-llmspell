//! ABOUTME: REPL command implementation for interactive scripting
//! ABOUTME: Provides an interactive read-eval-print loop

use crate::cli::ScriptEngine;
use crate::kernel::{CliCircuitBreaker, CliKernelDiscovery, KernelConnectionBuilder};
use crate::repl_interface::CLIReplInterface;
use anyhow::Result;
use llmspell_bridge::{
    diagnostics_bridge::DiagnosticsBridge,
};
use llmspell_config::LLMSpellConfig;
use std::path::PathBuf;

/// Start an interactive REPL session
pub async fn start_repl(
    engine: ScriptEngine,
    runtime_config: LLMSpellConfig,
    history_file: Option<PathBuf>,
) -> Result<()> {
    println!("LLMSpell REPL - {} engine", engine.as_str());
    println!("Connecting to kernel...");

    // Build kernel connection with dependency injection
    let mut kernel = KernelConnectionBuilder::new()
        .discovery(Box::new(CliKernelDiscovery::new()))
        .circuit_breaker(Box::new(CliCircuitBreaker::new()))
        .diagnostics(DiagnosticsBridge::builder().build())
        .build()
        .await?;

    // Connect to kernel or start new one
    kernel.connect_or_start().await?;

    // Build CLI REPL interface
    let mut cli_client = CLIReplInterface::builder()
        .kernel(kernel)
        .diagnostics(DiagnosticsBridge::builder().build())
        .config(runtime_config)
        .history_file(history_file.unwrap_or_else(|| {
            dirs::home_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join(".llmspell_history")
        }))
        .build()?;

    // Run interactive loop
    cli_client.run_interactive_loop().await
}
