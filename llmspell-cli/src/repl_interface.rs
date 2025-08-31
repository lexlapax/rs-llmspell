//! Interactive REPL interface with kernel integration
//!
//! Provides the main REPL loop with dependency injection and workload-aware performance.

use crate::kernel_connection::KernelConnectionTrait;
use anyhow::Result;
use llmspell_bridge::{diagnostics_bridge::DiagnosticsBridge, hook_profiler::WorkloadClassifier};
use llmspell_config::LLMSpellConfig;
use llmspell_repl::protocol::LDPRequest;
use rustyline::error::ReadlineError;
use rustyline::DefaultEditor;
use std::path::PathBuf;

/// REPL interface builder with dependency injection
#[derive(Default)]
pub struct CLIReplInterfaceBuilder {
    kernel: Option<Box<dyn KernelConnectionTrait>>,
    diagnostics: Option<DiagnosticsBridge>,
    history_file: Option<PathBuf>,
    config: Option<LLMSpellConfig>,
}

impl CLIReplInterfaceBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the kernel connection
    pub fn kernel(mut self, kernel: Box<dyn KernelConnectionTrait>) -> Self {
        self.kernel = Some(kernel);
        self
    }

    /// Set the diagnostics bridge
    pub fn diagnostics(mut self, diagnostics: DiagnosticsBridge) -> Self {
        self.diagnostics = Some(diagnostics);
        self
    }

    /// Set the history file
    pub fn history_file(mut self, path: PathBuf) -> Self {
        self.history_file = Some(path);
        self
    }

    /// Set the configuration
    pub fn config(mut self, config: LLMSpellConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Build the REPL interface
    pub fn build(self) -> Result<CLIReplInterface> {
        let kernel = self
            .kernel
            .ok_or_else(|| anyhow::anyhow!("Kernel connection required"))?;

        Ok(CLIReplInterface {
            kernel,
            diagnostics: self.diagnostics,
            history_file: self.history_file,
            config: self.config,
            editor: None,
        })
    }
}

/// CLI REPL interface
pub struct CLIReplInterface {
    kernel: Box<dyn KernelConnectionTrait>,
    diagnostics: Option<DiagnosticsBridge>,
    history_file: Option<PathBuf>,
    config: Option<LLMSpellConfig>,
    #[allow(dead_code)]
    editor: Option<DefaultEditor>,
}

impl CLIReplInterface {
    /// Create a builder for the REPL interface
    pub fn builder() -> CLIReplInterfaceBuilder {
        CLIReplInterfaceBuilder::new()
    }

    /// Run the interactive REPL loop
    pub async fn run_interactive_loop(&mut self) -> Result<()> {
        // Initialize editor
        let mut editor = DefaultEditor::new()?;

        // Load history if available
        if let Some(history_path) = &self.history_file {
            let _ = editor.load_history(history_path);
        }

        println!("LLMSpell REPL - Connected to kernel");
        println!("Type '.help' for commands, 'exit' or press Ctrl+D to quit");
        println!();

        loop {
            let readline = editor.readline("llmspell> ");

            match readline {
                Ok(line) => {
                    // Add to history
                    let _ = editor.add_history_entry(&line);

                    // Handle the input
                    if line.trim() == "exit" {
                        break;
                    }

                    if let Err(e) = self.handle_input(&line).await {
                        eprintln!("Error: {}", e);
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
        if let Some(history_path) = &self.history_file {
            let _ = editor.save_history(history_path);
        }

        // Disconnect from kernel
        self.kernel.disconnect().await?;

        Ok(())
    }

    /// Handle user input
    async fn handle_input(&mut self, input: &str) -> Result<()> {
        let input = input.trim();

        if input.is_empty() {
            return Ok(());
        }

        // Check if it's a command
        if input.starts_with('.') {
            self.handle_command(input).await
        } else {
            // Execute as code
            self.execute_code(input).await
        }
    }

    /// Handle REPL commands
    async fn handle_command(&mut self, command: &str) -> Result<()> {
        let parts: Vec<&str> = command.split_whitespace().collect();
        let cmd = parts.first().copied().unwrap_or("");

        // Classify workload for performance monitoring
        let _workload = self.kernel.classify_workload(cmd);

        match cmd {
            ".help" => {
                self.print_help();
                Ok(())
            }
            ".break" => self.handle_breakpoint_command(&parts).await,
            ".step" => self.handle_step_command().await,
            ".continue" => self.handle_continue_command().await,
            ".locals" => self.handle_locals_command().await,
            ".stack" => self.handle_stack_command().await,
            ".watch" => self.handle_watch_command(&parts).await,
            ".info" => self.handle_info_command().await,
            _ => {
                eprintln!("Unknown command: {}", cmd);
                Ok(())
            }
        }
    }

    /// Execute code on the kernel
    async fn execute_code(&mut self, code: &str) -> Result<()> {
        // Classify workload
        let workload = if code.lines().count() > 10 {
            self.kernel.classify_workload("execute_block")
        } else {
            self.kernel.classify_workload("execute_line")
        };

        // Record performance if diagnostics available
        let start = std::time::Instant::now();

        let result = self.kernel.execute(code).await?;

        let duration = start.elapsed();

        // Check if performance meets workload expectations
        if let Some(_diagnostics) = &self.diagnostics {
            // Log performance metrics based on workload classification
            match workload {
                WorkloadClassifier::Micro => {
                    if duration.as_millis() > 10 {
                        tracing::warn!("Micro operation took {}ms", duration.as_millis());
                    }
                }
                WorkloadClassifier::Light => {
                    if duration.as_millis() > 100 {
                        tracing::warn!("Light operation took {}ms", duration.as_millis());
                    }
                }
                WorkloadClassifier::Medium => {
                    if duration.as_millis() > 1000 {
                        tracing::warn!("Medium operation took {}ms", duration.as_millis());
                    }
                }
                WorkloadClassifier::Heavy => {
                    if duration.as_secs() > 10 {
                        tracing::warn!("Heavy operation took {}s", duration.as_secs());
                    }
                }
            }
        }

        // Display result
        println!("{}", serde_json::to_string_pretty(&result)?);

        Ok(())
    }

    /// Handle breakpoint command
    async fn handle_breakpoint_command(&mut self, parts: &[&str]) -> Result<()> {
        if parts.len() < 3 {
            eprintln!("Usage: .break <file> <line>");
            return Ok(());
        }

        let file = parts[1];
        let line: u32 = parts[2].parse()?;

        let request = LDPRequest::SetBreakpointRequest {
            file: file.to_string(),
            line,
            condition: None,
            hit_count: None,
            ignore_count: None,
        };

        let response = self.kernel.send_debug_command(request).await?;
        println!("Breakpoint response: {:?}", response);

        Ok(())
    }

    /// Handle step command
    async fn handle_step_command(&mut self) -> Result<()> {
        let request = LDPRequest::StepRequest;

        let response = self.kernel.send_debug_command(request).await?;
        println!("Step response: {:?}", response);

        Ok(())
    }

    /// Handle continue command
    async fn handle_continue_command(&mut self) -> Result<()> {
        let request = LDPRequest::ContinueRequest;

        let response = self.kernel.send_debug_command(request).await?;
        println!("Continue response: {:?}", response);

        Ok(())
    }

    /// Handle locals command
    async fn handle_locals_command(&mut self) -> Result<()> {
        if let Some(_exec_mgr) = self.kernel.execution_manager() {
            // Get variables from execution manager
            // This would be properly implemented with the actual execution manager
            println!("Local variables: (not yet implemented)");
        } else {
            eprintln!("Execution manager not available");
        }

        Ok(())
    }

    /// Handle stack command
    async fn handle_stack_command(&mut self) -> Result<()> {
        let request = LDPRequest::StackTraceRequest {
            thread_id: None,
            start_frame: Some(0),
            levels: Some(20),
        };

        let response = self.kernel.send_debug_command(request).await?;
        println!("Stack trace: {:?}", response);

        Ok(())
    }

    /// Handle watch command
    async fn handle_watch_command(&mut self, parts: &[&str]) -> Result<()> {
        if parts.len() < 2 {
            eprintln!("Usage: .watch <expression>");
            return Ok(());
        }

        let expression = parts[1..].join(" ");
        println!("Watching expression: {}", expression);

        Ok(())
    }

    /// Handle info command
    async fn handle_info_command(&mut self) -> Result<()> {
        println!(
            "Kernel connection status: {}",
            if self.kernel.is_connected() {
                "Connected"
            } else {
                "Disconnected"
            }
        );

        if let Some(_config) = &self.config {
            println!("Configuration loaded");
        }

        Ok(())
    }

    /// Print help message
    fn print_help(&self) {
        println!("Available commands:");
        println!("  .help              - Show this help message");
        println!("  .break <file> <ln> - Set breakpoint");
        println!("  .step              - Step to next line");
        println!("  .continue          - Continue execution");
        println!("  .locals            - Show local variables");
        println!("  .stack             - Show call stack");
        println!("  .watch <expr>      - Watch expression");
        println!("  .info              - Show connection info");
        println!("  exit               - Exit REPL");
        println!();
        println!("Enter any other text to execute as code");
    }
}
