//! Interactive session management with integrated REPL and debug
//!
//! Consolidates REPL functionality from Phase-9 llmspell-repl and debug
//! capabilities from llmspell-debug into a unified interactive experience.

use crate::debug::{DebugCoordinator, DebugSession};
use crate::execution::IntegratedKernel;
use crate::protocols::jupyter::JupyterProtocol;
use crate::repl::commands::{DebugCommand, MetaCommand, ReplCommand};
use crate::repl::readline::ReplReadline;
use crate::repl::state::{Breakpoint, ReplState};
use anyhow::Result;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// REPL session configuration
#[derive(Debug, Clone)]
pub struct ReplSessionConfig {
    /// Enable performance monitoring
    pub enable_performance_monitoring: bool,
    /// Enable debug commands
    pub enable_debug_commands: bool,
    /// History file path
    pub history_file: Option<PathBuf>,
    /// Maximum execution time (seconds)
    pub execution_timeout_secs: u64,
    /// Enable session persistence
    pub enable_persistence: bool,
}

impl Default for ReplSessionConfig {
    fn default() -> Self {
        Self {
            enable_performance_monitoring: true,
            enable_debug_commands: true,
            history_file: dirs::cache_dir().map(|d| d.join("llmspell_history")),
            execution_timeout_secs: 300,
            enable_persistence: true,
        }
    }
}

/// Interactive session combining REPL and debug
pub struct InteractiveSession {
    /// Integrated kernel
    kernel: IntegratedKernel<JupyterProtocol>,
    /// Debug coordinator (optional)
    debug_coordinator: Option<Arc<DebugCoordinator>>,
    /// REPL state
    state: Arc<RwLock<ReplState>>,
    /// Session configuration
    config: ReplSessionConfig,
    /// Execution count
    execution_count: u32,
    /// Session start time
    start_time: Instant,
    /// Performance monitoring enabled
    perf_monitoring: bool,
    /// Current debug session (if debugging)
    debug_session: Option<DebugSession>,
    /// Readline interface (optional - falls back to stdin if not available)
    readline: Option<ReplReadline>,
    /// Multi-line input buffer
    multiline_buffer: Vec<String>,
    /// Flag to track if we're executing
    executing: Arc<AtomicBool>,
}

impl InteractiveSession {
    /// Create new interactive session
    ///
    /// # Errors
    ///
    /// Returns an error if the session cannot be created
    pub async fn new(
        kernel: IntegratedKernel<JupyterProtocol>,
        config: ReplSessionConfig,
    ) -> Result<Self> {
        let mut state = ReplState::new();

        // Load history if configured
        if let Some(ref history_file) = config.history_file {
            if let Err(e) = state.history.load_from_file(history_file) {
                debug!("Failed to load history: {}", e);
            }
        }

        // Create shared state
        let state_arc = Arc::new(RwLock::new(state));

        // Create readline interface
        let readline = match ReplReadline::new(state_arc.clone()).await {
            Ok(mut rl) => {
                // Load history file if configured
                if let Some(ref history_file) = config.history_file {
                    let _ = rl.load_history(history_file);
                }
                Some(rl)
            }
            Err(e) => {
                warn!(
                    "Failed to initialize readline, falling back to stdin: {}",
                    e
                );
                None
            }
        };

        // Create debug coordinator if debug commands are enabled
        let debug_coordinator = if config.enable_debug_commands {
            let session_id = format!("repl-{}", uuid::Uuid::new_v4());
            Some(Arc::new(DebugCoordinator::new(session_id)))
        } else {
            None
        };

        Ok(Self {
            kernel,
            debug_coordinator,
            state: state_arc,
            execution_count: 0,
            start_time: Instant::now(),
            perf_monitoring: config.enable_performance_monitoring,
            debug_session: None,
            readline,
            multiline_buffer: Vec::new(),
            executing: Arc::new(AtomicBool::new(false)),
            config,
        })
    }

    /// Run the REPL loop
    ///
    /// # Errors
    ///
    /// Returns an error if the REPL fails
    pub async fn run_repl(&mut self) -> Result<()> {
        info!("Starting interactive REPL session");

        // Set up signal handler for Ctrl-C
        self.setup_signal_handler();

        // Print welcome message
        self.print_welcome();

        // Main REPL loop
        loop {
            // Determine prompt based on multi-line state
            let prompt = if self.multiline_buffer.is_empty() {
                "> "
            } else {
                "... "
            };

            // Get user input
            let input = self.read_input(prompt).await;

            // Check if interrupted
            if input.is_empty() && self.multiline_buffer.is_empty() {
                // Ctrl-C at prompt - just show new prompt
                continue;
            }

            // Handle multi-line input
            let full_input = if !self.multiline_buffer.is_empty() {
                // Check for empty line to execute accumulated buffer
                if input.trim().is_empty() {
                    let code = self.multiline_buffer.join("\n");
                    self.multiline_buffer.clear();
                    code
                } else {
                    // Add to buffer and check if complete
                    self.multiline_buffer.push(input.clone());
                    let code = self.multiline_buffer.join("\n");

                    // Check if expression is complete
                    if self.is_complete_expression(&code) {
                        self.multiline_buffer.clear();
                        code
                    } else {
                        // Continue accumulating
                        continue;
                    }
                }
            } else if input.trim().is_empty() {
                // Empty line - skip
                continue;
            } else if self.looks_like_multiline_start(&input) {
                // Start multi-line accumulation
                self.multiline_buffer.push(input);
                continue;
            } else {
                input
            };

            // Parse and handle command
            match ReplCommand::parse(&full_input) {
                Ok(ReplCommand::Empty) => {}
                Ok(ReplCommand::Meta(MetaCommand::Exit)) => {
                    self.cleanup().await?;
                    break;
                }
                Ok(command) => {
                    if let Err(e) = self.handle_command(command).await {
                        error!("Command execution error: {e}");
                    }
                }
                Err(e) => {
                    error!("Command parse error: {e}");
                }
            }
        }

        // Save history on exit
        if let Some(ref mut readline) = self.readline {
            if let Some(ref history_file) = self.config.history_file {
                if let Err(e) = readline.save_history(history_file) {
                    debug!("Failed to save history: {}", e);
                }
            }
        }

        info!("REPL session ended");
        Ok(())
    }

    /// Handle a parsed command
    async fn handle_command(&mut self, command: ReplCommand) -> Result<()> {
        match command {
            ReplCommand::Execute(code) => self.execute_code(&code).await,
            ReplCommand::Meta(meta) => self.handle_meta_command(meta).await,
            ReplCommand::Debug(debug) => self.handle_debug_command(debug).await,
            ReplCommand::Empty => Ok(()),
        }
    }

    /// Execute code in the kernel
    async fn execute_code(&mut self, code: &str) -> Result<()> {
        self.execution_count += 1;

        // Add to history
        {
            let mut state = self.state.write().await;
            state.history.add(code.to_string());
        }

        // Performance monitoring
        let start = if self.perf_monitoring {
            Some(Instant::now())
        } else {
            None
        };

        // Set executing flag
        self.executing.store(true, Ordering::Relaxed);

        // Execute through kernel's direct execution
        let result = self.execute_via_kernel(code).await;

        // Clear executing flag
        self.executing.store(false, Ordering::Relaxed);

        // Print result
        println!("{result}");

        // Check performance
        if let Some(start_time) = start {
            let duration = start_time.elapsed();
            let millis = duration.as_millis();
            if self.perf_monitoring {
                println!("⏱️ {millis} ms");
            }
            if duration.as_secs() > 1 {
                warn!("Slow execution: {:.2}s", duration.as_secs_f64());
            }
        }

        Ok(())
    }

    /// Execute code via the kernel's direct execution method
    async fn execute_via_kernel(&mut self, code: &str) -> String {
        // Execute code directly through the kernel
        match self.kernel.execute_direct(code).await {
            Ok(result) => result,
            Err(e) => format!("Error: {e}"),
        }
    }

    /// Handle meta commands
    async fn handle_meta_command(&mut self, command: MetaCommand) -> Result<()> {
        match command {
            MetaCommand::Help => {
                println!("{}", MetaCommand::help_text());
                if self.config.enable_debug_commands {
                    println!("\n{}", DebugCommand::help_text());
                }
            }
            MetaCommand::Clear => {
                // Clear screen (would be handled by CLI layer)
                print!("\x1B[2J\x1B[1;1H");
            }
            MetaCommand::Save(path) => {
                self.save_session(&path).await?;
                let path_display = path.display();
                println!("Session saved to {path_display}");
            }
            MetaCommand::Load(path) => {
                self.load_session(&path).await?;
                let path_display = path.display();
                println!("Session loaded from {path_display}");
            }
            MetaCommand::History => {
                let state = self.state.read().await;
                for (i, cmd) in state.history.entries().iter().enumerate() {
                    println!("{:4}: {cmd}", i + 1);
                }
            }
            MetaCommand::ClearHistory => {
                let mut state = self.state.write().await;
                state.history.clear();
                println!("History cleared");
            }
            MetaCommand::Variables => {
                let state = self.state.read().await;
                if state.variables.is_empty() {
                    println!("No variables set");
                } else {
                    for (name, value) in &state.variables {
                        println!("{name} = {value}");
                    }
                }
            }
            MetaCommand::Set(name, value) => {
                let mut state = self.state.write().await;
                state.variables.insert(name.clone(), value.clone());
                println!("Set {name} = {value}");
            }
            MetaCommand::Unset(name) => {
                let mut state = self.state.write().await;
                if state.variables.remove(&name).is_some() {
                    println!("Unset {name}");
                } else {
                    println!("Variable {name} not found");
                }
            }
            MetaCommand::Cd(path) => {
                std::env::set_current_dir(&path)?;
                let mut state = self.state.write().await;
                state.working_dir.clone_from(&path);
                let path_display = path.display();
                println!("Changed directory to {path_display}");
            }
            MetaCommand::Pwd => {
                let state = self.state.read().await;
                let dir = state.working_dir.display();
                println!("{dir}");
            }
            MetaCommand::Ls(path) => {
                let dir = path.unwrap_or_else(|| std::env::current_dir().unwrap());
                for entry in std::fs::read_dir(dir)? {
                    let entry = entry?;
                    println!("{}", entry.file_name().to_string_lossy());
                }
            }
            MetaCommand::Info => {
                self.print_session_info().await;
            }
            MetaCommand::Reset => {
                self.reset_session().await?;
                println!("Session reset");
            }
            MetaCommand::Run { file, args } => {
                self.execute_script_file(&file, args).await?;
            }
            MetaCommand::Perf { enabled } => {
                self.perf_monitoring = enabled;
                println!(
                    "Performance monitoring {}",
                    if enabled { "enabled" } else { "disabled" }
                );
            }
            MetaCommand::Exit => unreachable!(), // Handled in run_repl
        }
        Ok(())
    }

    /// Handle breakpoint-related debug commands
    async fn handle_breakpoint_command(&mut self, command: DebugCommand) -> Result<Option<()>> {
        match command {
            DebugCommand::Break(spec) => {
                let mut state = self.state.write().await;
                let id = state.breakpoints.len();
                let mut breakpoint = Breakpoint::new(id, spec.line);

                if let Some(file) = spec.file {
                    breakpoint = breakpoint.with_file(file);
                }
                if let Some(condition) = spec.condition {
                    breakpoint = breakpoint.with_condition(condition);
                }

                state.add_breakpoint(breakpoint.clone());
                println!("Breakpoint #{id} set at line {}", spec.line);
                Ok(Some(()))
            }
            DebugCommand::Delete(id) => {
                let mut state = self.state.write().await;
                if state.remove_breakpoint(id).is_some() {
                    println!("Breakpoint #{id} deleted");
                } else {
                    println!("Breakpoint #{id} not found");
                }
                Ok(Some(()))
            }
            DebugCommand::List => {
                let state = self.state.read().await;
                if state.breakpoints.is_empty() {
                    println!("No breakpoints set");
                } else {
                    for bp in state.list_breakpoints() {
                        let status = if bp.enabled { "enabled" } else { "disabled" };
                        let location = bp.file.as_deref().unwrap_or("<current>");
                        println!(
                            "#{} {} {location}:{} (hits: {})",
                            bp.id, status, bp.line, bp.hit_count
                        );
                        if let Some(ref cond) = bp.condition {
                            println!("    Condition: {cond}");
                        }
                    }
                }
                Ok(Some(()))
            }
            DebugCommand::Enable(id) => {
                let mut state = self.state.write().await;
                if let Some(bp) = state.breakpoints.get_mut(id) {
                    bp.enabled = true;
                    println!("Breakpoint #{id} enabled");
                } else {
                    println!("Breakpoint #{id} not found");
                }
                Ok(Some(()))
            }
            DebugCommand::Disable(id) => {
                let mut state = self.state.write().await;
                if let Some(bp) = state.breakpoints.get_mut(id) {
                    bp.enabled = false;
                    println!("Breakpoint #{id} disabled");
                } else {
                    println!("Breakpoint #{id} not found");
                }
                Ok(Some(()))
            }
            _ => Ok(None), // Not a breakpoint command
        }
    }

    /// Handle execution control commands
    async fn handle_execution_command(
        &mut self,
        command: DebugCommand,
        coordinator: &Arc<DebugCoordinator>,
    ) -> Result<Option<()>> {
        match command {
            DebugCommand::Step => {
                coordinator.step_into().await?;
                println!("Stepped into");
                Ok(Some(()))
            }
            DebugCommand::Next => {
                coordinator.step_over().await?;
                println!("Stepped over");
                Ok(Some(()))
            }
            DebugCommand::Finish => {
                coordinator.step_out().await?;
                println!("Stepped out");
                Ok(Some(()))
            }
            DebugCommand::Continue => {
                coordinator.continue_execution().await?;
                println!("Continuing execution");
                Ok(Some(()))
            }
            DebugCommand::Pause => {
                coordinator.pause().await?;
                println!("Execution paused");
                Ok(Some(()))
            }
            _ => Ok(None), // Not an execution command
        }
    }

    /// Handle information display commands
    async fn handle_info_command(&mut self, command: DebugCommand) -> Result<Option<()>> {
        match command {
            DebugCommand::Locals => {
                let state = self.state.read().await;
                if let Some(ref ctx) = state.debug_context {
                    if ctx.locals.is_empty() {
                        println!("No local variables");
                    } else {
                        for (name, value) in &ctx.locals {
                            println!("{name} = {value}");
                        }
                    }
                } else {
                    println!("Not in debug context");
                }
                Ok(Some(()))
            }
            DebugCommand::Backtrace | DebugCommand::Where => {
                let state = self.state.read().await;
                if let Some(ref ctx) = state.debug_context {
                    for (i, frame) in ctx.stack_frames.iter().enumerate() {
                        let marker = if i == ctx.current_frame { ">" } else { " " };
                        let location = frame.file.as_deref().unwrap_or("<unknown>");
                        println!(
                            "{marker} #{} {} at {location}:{}",
                            frame.id,
                            frame.name,
                            frame.line.unwrap_or(0)
                        );
                    }
                } else {
                    println!("Not in debug context");
                }
                Ok(Some(()))
            }
            DebugCommand::Frame(n) => {
                let mut state = self.state.write().await;
                if let Some(ref mut ctx) = state.debug_context {
                    if n < ctx.stack_frames.len() {
                        ctx.current_frame = n;
                        println!("Selected frame #{n}");
                    } else {
                        println!("Invalid frame number");
                    }
                } else {
                    println!("Not in debug context");
                }
                Ok(Some(()))
            }
            _ => Ok(None), // Not an info command
        }
    }

    /// Handle expression evaluation commands
    async fn handle_expression_command(&mut self, command: DebugCommand) -> Option<()> {
        match command {
            DebugCommand::Print(expr) => {
                // Evaluate expression in current context
                let result = self.execute_via_kernel(&expr).await;
                println!("{result}");
                Some(())
            }
            DebugCommand::Watch(expr) => {
                // Add watch expression (would need watch list in state)
                println!("Watch expression added: {expr}");
                Some(())
            }
            DebugCommand::Unwatch(id) => {
                println!("Watch #{id} removed");
                Some(())
            }
            _ => None, // Not an expression command
        }
    }

    /// Handle debug commands
    async fn handle_debug_command(&mut self, command: DebugCommand) -> Result<()> {
        if !self.config.enable_debug_commands {
            return Err(anyhow::anyhow!("Debug commands are disabled"));
        }

        // Try each command handler in order
        if let Some(()) = self.handle_breakpoint_command(command.clone()).await? {
            return Ok(());
        }

        // For execution commands, we need the coordinator
        if matches!(
            command,
            DebugCommand::Step
                | DebugCommand::Next
                | DebugCommand::Finish
                | DebugCommand::Continue
                | DebugCommand::Pause
        ) {
            let coordinator = self
                .debug_coordinator
                .as_ref()
                .ok_or_else(|| anyhow::anyhow!("Debug coordinator not initialized"))?
                .clone();
            if let Some(()) = self
                .handle_execution_command(command.clone(), &coordinator)
                .await?
            {
                return Ok(());
            }
        }

        if let Some(()) = self.handle_info_command(command.clone()).await? {
            return Ok(());
        }
        if let Some(()) = self.handle_expression_command(command).await {
            return Ok(());
        }

        Err(anyhow::anyhow!("Unknown debug command"))
    }

    /// Print welcome message
    fn print_welcome(&self) {
        println!("LLMSpell Interactive REPL");
        println!("Type .help for commands, .exit to quit");
        if self.config.enable_debug_commands {
            println!("Debug commands enabled - use 'db:' prefix or .break");
        }
        println!();
    }

    /// Print session info
    async fn print_session_info(&self) {
        let uptime = self.start_time.elapsed();
        let state = self.state.read().await;

        println!("Session Information:");
        println!("  Uptime: {:.1}s", uptime.as_secs_f64());
        println!("  Executions: {}", self.execution_count);
        println!("  History entries: {}", state.history.entries().len());
        println!("  Variables: {}", state.variables.len());
        println!("  Breakpoints: {}", state.breakpoints.len());
        println!("  Debug mode: {}", self.debug_session.is_some());
    }

    /// Save session to file
    async fn save_session(&self, path: &std::path::Path) -> Result<()> {
        let state = self.state.read().await;
        let content = serde_json::to_string_pretty(&*state)?;
        std::fs::write(path, content)?;

        // Also save history separately if configured
        if let Some(ref history_file) = self.config.history_file {
            state.history.save_to_file(history_file)?;
        }

        Ok(())
    }

    /// Load session from file
    async fn load_session(&mut self, path: &std::path::Path) -> Result<()> {
        let content = std::fs::read_to_string(path)?;
        let loaded_state: ReplState = serde_json::from_str(&content)?;

        let mut state = self.state.write().await;
        *state = loaded_state;

        Ok(())
    }

    /// Reset session
    async fn reset_session(&mut self) -> Result<()> {
        let mut state = self.state.write().await;
        *state = ReplState::new();
        self.execution_count = 0;
        self.debug_session = None;
        Ok(())
    }

    /// Cleanup on exit
    async fn cleanup(&self) -> Result<()> {
        // Save history if configured
        if let Some(ref history_file) = self.config.history_file {
            let state = self.state.read().await;
            if let Err(e) = state.history.save_to_file(history_file) {
                warn!("Failed to save history: {e}");
            }
        }

        Ok(())
    }

    /// Read user input using readline or fallback to stdin
    async fn read_input(&mut self, prompt: &str) -> String {
        if let Some(ref mut readline) = self.readline {
            // Use readline interface
            match readline.readline(prompt).await {
                Ok(line) => line,
                Err(e) => {
                    // Handle interrupts and EOF
                    if e.to_string().contains("Interrupted") {
                        // Ctrl-C pressed - return empty line to continue
                        String::new()
                    } else {
                        // EOF or other error - exit
                        ".exit".to_string()
                    }
                }
            }
        } else {
            // Fallback to stdin
            use std::io::{self, Write};

            // Print prompt
            print!("{prompt}");
            io::stdout().flush().unwrap();

            // Read line from stdin
            let mut input = String::new();
            if io::stdin().read_line(&mut input).is_ok() {
                input
            } else {
                // On error or EOF, return exit command
                ".exit".to_string()
            }
        }
    }

    /// Set up signal handler for Ctrl-C
    fn setup_signal_handler(&self) {
        let executing = self.executing.clone();

        tokio::spawn(async move {
            loop {
                match tokio::signal::ctrl_c().await {
                    Ok(()) => {
                        if executing.load(Ordering::Relaxed) {
                            // Interrupt current execution
                            println!(
                                "\n^C Interrupted (execution interruption not fully implemented)"
                            );
                            executing.store(false, Ordering::Relaxed);
                        } else {
                            // At prompt or in multi-line - handled by readline
                            // The readline library will handle this case
                        }
                    }
                    Err(e) => {
                        error!("Failed to listen for Ctrl-C: {}", e);
                        break;
                    }
                }
            }
        });
    }

    /// Check if an expression looks like it needs multi-line input
    fn looks_like_multiline_start(&self, input: &str) -> bool {
        #![allow(clippy::unused_self)]
        let trimmed = input.trim();
        // Common patterns that suggest multi-line input
        trimmed.ends_with("function")
            || trimmed.ends_with("do")
            || trimmed.ends_with("then")
            || trimmed.ends_with("else")
            || trimmed.ends_with("repeat")
            || trimmed.ends_with('{')
            || trimmed.ends_with('[')
            || trimmed.ends_with('(')
            || (trimmed.starts_with("function") && !trimmed.contains("end"))
            || (trimmed.starts_with("if") && !trimmed.contains("end"))
            || (trimmed.starts_with("for") && !trimmed.contains("end"))
            || (trimmed.starts_with("while") && !trimmed.contains("end"))
    }

    /// Check if an expression is complete (can be executed)
    fn is_complete_expression(&self, code: &str) -> bool {
        #![allow(clippy::unused_self)]
        // For now, use simple heuristics for Lua
        // In the future, this should delegate to ScriptEngineBridge

        // Count opening and closing keywords/brackets
        let opens = code.matches("function").count()
            + code.matches(" do").count()
            + code.matches(" then").count()
            + code.matches(" repeat").count();
        let closes = code.matches("end").count() + code.matches("until").count();

        // Count brackets
        let open_braces = code.chars().filter(|&c| c == '{').count();
        let close_braces = code.chars().filter(|&c| c == '}').count();
        let open_brackets = code.chars().filter(|&c| c == '[').count();
        let close_brackets = code.chars().filter(|&c| c == ']').count();
        let open_parens = code.chars().filter(|&c| c == '(').count();
        let close_parens = code.chars().filter(|&c| c == ')').count();

        // Check for unclosed strings (simple check)
        let mut in_string = false;
        let mut escape = false;
        let mut quote_char = ' ';
        for c in code.chars() {
            if escape {
                escape = false;
                continue;
            }
            if c == '\\' {
                escape = true;
                continue;
            }
            if !in_string && (c == '"' || c == '\'' || c == '[' && code.contains("[[")) {
                in_string = true;
                quote_char = c;
            } else if in_string && c == quote_char {
                in_string = false;
            }
        }

        // Expression is complete if all are balanced and no unclosed strings
        !in_string
            && opens <= closes
            && open_braces == close_braces
            && open_brackets == close_brackets
            && open_parens == close_parens
    }

    /// Execute a script file with arguments
    async fn execute_script_file(
        &mut self,
        file: &std::path::Path,
        args: Vec<String>,
    ) -> Result<()> {
        // Resolve the file path (add .lua if needed)
        let resolved_file = if !file.exists() && file.extension().is_none() {
            let mut lua_file = file.to_path_buf();
            lua_file.set_extension("lua");
            if lua_file.exists() {
                lua_file
            } else {
                return Err(anyhow::anyhow!("File not found: {}", file.display()));
            }
        } else if !file.exists() {
            return Err(anyhow::anyhow!("File not found: {}", file.display()));
        } else {
            file.to_path_buf()
        };

        // Read the script content
        let script = tokio::fs::read_to_string(&resolved_file)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to read {}: {}", resolved_file.display(), e))?;

        // Save current directory and change to script's directory
        let original_dir = std::env::current_dir()?;
        if let Some(parent) = resolved_file.parent() {
            std::env::set_current_dir(parent)?;
        }

        println!("Running {}...", resolved_file.display());
        if !args.is_empty() {
            println!("Arguments: {args:?}");
        }

        // TODO: Pass args to script when ScriptEngineBridge supports it
        // For now, just execute the script
        let start = if self.perf_monitoring {
            Some(Instant::now())
        } else {
            None
        };

        let result = self.execute_via_kernel(&script).await;
        println!("{result}");

        // Show performance
        if let Some(start_time) = start {
            let duration = start_time.elapsed();
            println!("⏱️ Script execution time: {} ms", duration.as_millis());
        }

        // Restore original directory
        std::env::set_current_dir(original_dir)?;

        Ok(())
    }
}
