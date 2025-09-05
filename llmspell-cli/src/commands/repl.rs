//! ABOUTME: REPL command implementation - thin terminal I/O wrapper
//! ABOUTME: Provides terminal interface, delegates logic to llmspell-repl

use crate::cli::ScriptEngine;
use crate::kernel_client::KernelConnectionTrait;
use anyhow::Result;
use llmspell_bridge::hook_profiler::WorkloadClassifier;
use llmspell_config::LLMSpellConfig;
use llmspell_repl::{ReplConfig, ReplSession};
use rustyline::error::ReadlineError;
use rustyline::{Config, Editor};
use serde_json::Value;
use std::path::PathBuf;

/// Adapter to connect kernel_client to llmspell-repl
struct KernelConnectionAdapter {
    inner: Box<dyn KernelConnectionTrait>,
}

#[async_trait::async_trait]
impl llmspell_repl::KernelConnection for KernelConnectionAdapter {
    async fn connect_or_start(&mut self) -> Result<()> {
        self.inner.connect_or_start().await
    }

    async fn execute(&mut self, code: &str) -> Result<String> {
        self.inner.execute(code).await
    }

    async fn send_debug_command(&mut self, command: Value) -> Result<Value> {
        self.inner.send_debug_command(command).await
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.inner.disconnect().await
    }

    fn is_connected(&self) -> bool {
        self.inner.is_connected()
    }

    fn classify_workload(&self, operation: &str) -> llmspell_repl::WorkloadType {
        match self.inner.classify_workload(operation) {
            WorkloadClassifier::Micro => llmspell_repl::WorkloadType::Micro,
            WorkloadClassifier::Light => llmspell_repl::WorkloadType::Light,
            WorkloadClassifier::Medium => llmspell_repl::WorkloadType::Medium,
            WorkloadClassifier::Heavy => llmspell_repl::WorkloadType::Heavy,
        }
    }

    fn execution_manager(&self) -> Option<&dyn std::any::Any> {
        self.inner.execution_manager()
    }
}

/// Start an interactive REPL session - ONLY terminal I/O
pub async fn start_repl(
    engine: ScriptEngine,
    runtime_config: LLMSpellConfig,
    connect: Option<String>,
    history_file: Option<PathBuf>,
) -> Result<()> {
    println!("LLMSpell REPL - {} engine", engine.as_str());
    println!("Connecting to kernel...");

    // Create kernel connection using the shared function
    let kernel = super::create_kernel_connection(runtime_config.clone(), connect).await?;

    // Wrap in adapter for llmspell-repl
    let kernel_adapter = Box::new(KernelConnectionAdapter { inner: kernel });

    // Create REPL session (business logic)
    let repl_config = ReplConfig {
        enable_performance_monitoring: runtime_config.debug.performance.enabled,
        enable_debug_commands: runtime_config.debug.enabled,
    };
    let mut session = ReplSession::new(kernel_adapter, repl_config).await?;

    // Terminal setup
    let history_path = history_file.unwrap_or_else(|| {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".llmspell_history")
    });

    let config = Config::builder().max_history_size(1000)?.build();
    let mut editor: Editor<(), _> = Editor::with_config(config)?;

    // Load history
    let _ = editor.load_history(&history_path);

    println!("Type '.help' for commands, 'exit' or press Ctrl+D to quit");
    println!();

    // Simple I/O loop - ALL logic delegated to ReplSession
    loop {
        match editor.readline("llmspell> ") {
            Ok(line) => {
                let _ = editor.add_history_entry(&line);

                // Delegate to session
                match session.handle_input(&line).await {
                    Ok(response) => {
                        // Check for exit
                        if response.should_exit() {
                            break;
                        }
                        // Display response
                        let output = response.format();
                        if !output.is_empty() {
                            println!("{}", output);
                        }
                    }
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("^C");
                continue;
            }
            Err(ReadlineError::Eof) => {
                println!("^D");
                break;
            }
            Err(err) => {
                eprintln!("Error: {:?}", err);
                break;
            }
        }
    }

    // Save history
    let _ = editor.save_history(&history_path);

    // Disconnect
    session.disconnect().await?;

    Ok(())
}
