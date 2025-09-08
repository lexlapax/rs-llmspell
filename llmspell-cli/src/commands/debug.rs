//! Debug command - leverages existing REPL debug infrastructure
//!
//! Implementation strategy: Reuse the working REPL debug system rather than
//! building new infrastructure. This provides all debug functionality through
//! the proven kernel architecture.

use crate::cli::{OutputFormat, ScriptEngine};
use anyhow::Result;
use llmspell_bridge::hook_profiler::WorkloadClassifier;
use llmspell_config::LLMSpellConfig;
use llmspell_repl::{KernelConnection, ReplConfig, ReplSession, WorkloadType};
use rustyline::{error::ReadlineError, Config, Editor};
use serde_json::Value;
use std::path::PathBuf;

/// Handle the debug command using existing kernel + REPL infrastructure
pub async fn handle_debug_command(
    script: PathBuf,
    break_at: Vec<String>,
    port: Option<u16>,
    args: Vec<String>,
    _engine: ScriptEngine,
    mut config: LLMSpellConfig,
    _output_format: OutputFormat,
) -> Result<()> {
    // Validate script file exists
    if !script.exists() {
        anyhow::bail!("Script file not found: {}", script.display());
    }

    println!("🐛 LLMSpell Debug Mode");
    println!("Script: {}", script.display());

    if !args.is_empty() {
        println!("Args: {:?}", args);
    }

    // Enable debug mode in config
    config.debug.enabled = true;
    config.debug.mode = "interactive".to_string();

    // Create kernel connection (reuse existing create_kernel_connection)
    println!("Connecting to kernel...");
    let kernel = super::create_kernel_connection(config.clone(), None).await?;

    // Wrap in adapter for llmspell-repl
    let kernel_adapter = Box::new(DebugKernelAdapter { inner: kernel });

    // Create REPL session with debug commands enabled
    let repl_config = ReplConfig {
        enable_performance_monitoring: config.debug.performance.enabled,
        enable_debug_commands: true, // Enable all debug commands
    };

    let mut session = ReplSession::new(kernel_adapter, repl_config).await?;

    // Set initial breakpoints before script execution
    for bp in &break_at {
        set_breakpoint(&mut session, bp).await?;
    }

    // TODO: DAP server support - when port is specified
    if let Some(_port) = port {
        println!("⚠️  DAP server not yet implemented. Using interactive debug mode.");
        // Future: start_dap_server(&mut session, port).await?;
    }

    // Load script content
    let script_content = tokio::fs::read_to_string(&script).await?;

    // Execute script (will hit breakpoints if set)
    println!("🏃 Starting script execution in debug mode...");
    println!("Type debug commands (.help for help) or press Ctrl+C to interrupt");
    println!();

    // Execute script through REPL to trigger any breakpoints
    match session.handle_input(&script_content).await {
        Ok(response) => {
            let output = response.format();
            if !output.is_empty() {
                println!("{}", output);
            }

            // Check if script completed normally
            if !response.should_exit() {
                println!("✅ Script loaded. Entering debug REPL...");
            }
        }
        Err(e) => {
            println!("❌ Script error: {}", e);
            println!("Entering debug REPL to investigate...");
        }
    }

    // Enter interactive debug REPL (reuse existing REPL terminal I/O)
    start_debug_repl_session(session).await
}

/// Set a breakpoint using REPL debug command format
async fn set_breakpoint(session: &mut ReplSession, breakpoint_spec: &str) -> Result<()> {
    let parts: Vec<&str> = breakpoint_spec.split(':').collect();
    if parts.len() != 2 {
        anyhow::bail!(
            "Invalid breakpoint format: '{}'. Use FILE:LINE",
            breakpoint_spec
        );
    }

    let file = parts[0];
    let line = parts[1];

    // Validate line number
    line.parse::<u32>()
        .map_err(|_| anyhow::anyhow!("Invalid line number: '{}'", line))?;

    // Use REPL's .break command format
    let break_command = format!(".break {} {}", file, line);
    match session.handle_input(&break_command).await {
        Ok(response) => {
            let output = response.format();
            if !output.is_empty() {
                println!("{}", output);
            }
            println!("🔴 Breakpoint set at {}:{}", file, line);
        }
        Err(e) => {
            eprintln!("❌ Failed to set breakpoint at {}: {}", breakpoint_spec, e);
        }
    }

    Ok(())
}

/// Start debug REPL session with terminal I/O (reuse from repl.rs)
async fn start_debug_repl_session(mut session: ReplSession) -> Result<()> {
    // Terminal setup (same as repl.rs)
    let config = Config::builder().max_history_size(1000)?.build();
    let mut editor: Editor<(), _> = Editor::with_config(config)?;

    let history_path = dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".llmspell_debug_history");

    let _ = editor.load_history(&history_path);

    println!("\n🐛 Debug REPL Commands:");
    println!("  .break <file> <line>   - Set breakpoint");
    println!("  .step                  - Step to next line");
    println!("  .continue              - Continue execution");
    println!("  .locals                - Show local variables");
    println!("  .globals               - Show global variables");
    println!("  .stack                 - Show call stack");
    println!("  .help                  - Show all commands");
    println!("  exit                   - Exit debug session");
    println!();

    // I/O loop (same pattern as repl.rs)
    loop {
        match editor.readline("debug> ") {
            Ok(line) => {
                let _ = editor.add_history_entry(&line);

                // Handle input through REPL session
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

    // Save history and disconnect
    let _ = editor.save_history(&history_path);
    session.disconnect().await?;

    println!("Debug session ended.");
    Ok(())
}

/// Adapter to connect kernel_client to llmspell-repl (same pattern as repl.rs)
struct DebugKernelAdapter {
    inner: Box<dyn crate::kernel_client::KernelConnectionTrait>,
}

#[async_trait::async_trait]
impl KernelConnection for DebugKernelAdapter {
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

    fn classify_workload(&self, operation: &str) -> WorkloadType {
        match self.inner.classify_workload(operation) {
            WorkloadClassifier::Micro => WorkloadType::Micro,
            WorkloadClassifier::Light => WorkloadType::Light,
            WorkloadClassifier::Medium => WorkloadType::Medium,
            WorkloadClassifier::Heavy => WorkloadType::Heavy,
        }
    }

    fn execution_manager(&self) -> Option<&dyn std::any::Any> {
        self.inner.execution_manager()
    }
}
