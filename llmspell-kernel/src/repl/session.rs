//! Interactive session management with integrated REPL and debug
//!
//! Consolidates REPL functionality from Phase-9 llmspell-repl and debug
//! capabilities from llmspell-debug into a unified interactive experience.

use crate::debug::{DebugCoordinator, ExecutionManager};
use crate::execution::IntegratedKernel;
use crate::protocols::jupyter::JupyterProtocol;
use crate::repl::commands::{DebugCommand, MetaCommand, ReplCommand};
use crate::repl::readline::{ReplReadline, ScriptExecutorCompletionAdapter};
use crate::repl::state::{Breakpoint, ReplState};
use anyhow::Result;
use llmspell_core::traits::debug_context::DebugContext;
use llmspell_core::traits::script_executor::ScriptExecutor;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::{mpsc, RwLock};
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

/// Debug session manager for REPL
///
/// Manages debug state and coordinates between REPL and `ExecutionManager`
pub struct ReplDebugSession {
    /// `ExecutionManager` that implements `DebugContext`
    execution_manager: Arc<ExecutionManager>,
    /// Script executor reference
    #[allow(dead_code)]
    script_executor: Arc<dyn ScriptExecutor>,
    /// Map of breakpoints by file
    breakpoints: std::collections::HashMap<String, Vec<u32>>,
    /// Current stack frame index when paused
    current_frame: Option<usize>,
    /// Flag indicating if execution is paused
    paused: Arc<AtomicBool>,
    /// Current pause location (file, line)
    pause_location: Arc<RwLock<Option<(String, u32)>>>,
    /// Channel to receive stopped events
    stopped_rx: Option<mpsc::UnboundedReceiver<StoppedEvent>>,
    /// Channel to send stopped events
    #[allow(dead_code)]
    stopped_tx: mpsc::UnboundedSender<StoppedEvent>,
}

/// Event sent when execution stops at a breakpoint
#[derive(Debug, Clone)]
pub struct StoppedEvent {
    /// Reason for stopping
    reason: String,
    /// Thread ID (always 1 for single-threaded)
    #[allow(dead_code)]
    thread_id: u32,
    /// File where stopped
    file: String,
    /// Line number where stopped
    line: u32,
}

impl ReplDebugSession {
    /// Create a new debug session
    pub fn new(
        execution_manager: Arc<ExecutionManager>,
        script_executor: Arc<dyn ScriptExecutor>,
    ) -> Self {
        let (stopped_tx, stopped_rx) = mpsc::unbounded_channel();

        Self {
            execution_manager,
            script_executor,
            breakpoints: std::collections::HashMap::new(),
            current_frame: None,
            paused: Arc::new(AtomicBool::new(false)),
            pause_location: Arc::new(RwLock::new(None)),
            stopped_rx: Some(stopped_rx),
            stopped_tx,
        }
    }

    /// Check if execution is paused
    pub fn is_paused(&self) -> bool {
        self.paused.load(Ordering::Relaxed)
    }

    /// Get current pause location
    pub async fn get_pause_location(&self) -> Option<(String, u32)> {
        self.pause_location.read().await.clone()
    }

    /// Start listening for stopped events
    pub fn take_stopped_receiver(&mut self) -> Option<mpsc::UnboundedReceiver<StoppedEvent>> {
        self.stopped_rx.take()
    }
}

/// Session statistics tracking
#[derive(Debug, Clone)]
pub struct SessionStatistics {
    /// Total execution time (milliseconds)
    pub total_execution_time_ms: u128,
    /// Average execution time (milliseconds)
    pub avg_execution_time_ms: u128,
    /// Min execution time (milliseconds)
    pub min_execution_time_ms: u128,
    /// Max execution time (milliseconds)
    pub max_execution_time_ms: u128,
    /// Total commands executed
    pub commands_executed: u32,
    /// Errors encountered
    pub errors_encountered: u32,
    /// Memory snapshots (before, after) for last execution
    pub memory_delta: Option<(u64, u64)>,
    /// Peak memory usage
    pub peak_memory_bytes: u64,
}

impl Default for SessionStatistics {
    fn default() -> Self {
        Self {
            total_execution_time_ms: 0,
            avg_execution_time_ms: 0,
            min_execution_time_ms: u128::MAX,
            max_execution_time_ms: 0,
            commands_executed: 0,
            errors_encountered: 0,
            memory_delta: None,
            peak_memory_bytes: 0,
        }
    }
}

/// Interactive session combining REPL and debug
pub struct InteractiveSession {
    /// Integrated kernel
    kernel: IntegratedKernel<JupyterProtocol>,
    /// Debug coordinator (optional)
    #[allow(dead_code)]
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
    /// Session statistics
    session_stats: SessionStatistics,
    /// Current debug session (if debugging)
    debug_session: Option<ReplDebugSession>,
    /// `ExecutionManager` for debug support
    execution_manager: Option<Arc<ExecutionManager>>,
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
        let mut readline = match ReplReadline::new(state_arc.clone()).await {
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

        // Wire up script completion provider for REPL tab completion
        // This only affects interactive paths, not script execution
        if let Some(ref mut rl) = readline {
            let script_executor = kernel.get_script_executor();
            let completion_adapter = ScriptExecutorCompletionAdapter::new(script_executor);
            let provider = Arc::new(completion_adapter);
            rl.set_script_completion_provider(provider);
            debug!("Script completion provider wired up for REPL");
        }

        // Create debug coordinator and execution manager if debug commands are enabled
        let (debug_coordinator, execution_manager) = if config.enable_debug_commands {
            let session_id = format!("repl-{}", uuid::Uuid::new_v4());
            let exec_mgr = Arc::new(ExecutionManager::new(session_id.clone()));
            let coordinator = Arc::new(DebugCoordinator::new(session_id));
            (Some(coordinator), Some(exec_mgr))
        } else {
            (None, None)
        };

        Ok(Self {
            kernel,
            debug_coordinator,
            state: state_arc,
            execution_count: 0,
            start_time: Instant::now(),
            perf_monitoring: config.enable_performance_monitoring,
            session_stats: SessionStatistics::default(),
            debug_session: None,
            execution_manager,
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
            // Get appropriate prompt
            let prompt = self.get_prompt();

            // Get user input
            let input = self.read_input(&prompt).await;

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
            ReplCommand::Chat(message) => {
                // TODO: Implement in Subtask 12.9.4
                warn!("Chat command not yet implemented: {}", message);
                println!("Chat functionality will be implemented in Subtask 12.9.4");
                Ok(())
            }
            ReplCommand::Meta(meta) => self.handle_meta_command(meta).await,
            ReplCommand::ChatMeta(chat_meta) => {
                // TODO: Implement in Subtask 12.9.4
                warn!("Chat meta command not yet implemented: {:?}", chat_meta);
                println!("Chat meta commands (.system, .model, .tools, .context, .clearchat) will be implemented in Subtask 12.9.4");
                Ok(())
            }
            ReplCommand::Debug(debug) => self.handle_debug_command(debug).await,
            ReplCommand::Empty => Ok(()),
        }
    }

    /// Execute code in the kernel
    async fn execute_code(&mut self, code: &str) -> Result<()> {
        self.execution_count += 1;
        self.session_stats.commands_executed += 1;

        // Add to history
        {
            let mut state = self.state.write().await;
            state.history.add(code.to_string());
        }

        // Memory tracking - before execution
        let mem_before = get_current_memory_usage();

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

        // Track errors
        if result.contains("Error") || result.contains("error") {
            self.session_stats.errors_encountered += 1;
        }

        // Clear executing flag
        self.executing.store(false, Ordering::Relaxed);

        // Print result
        println!("{result}");

        // Memory tracking - after execution
        let mem_after = get_current_memory_usage();
        self.session_stats.memory_delta = Some((mem_before, mem_after));
        if mem_after > self.session_stats.peak_memory_bytes {
            self.session_stats.peak_memory_bytes = mem_after;
        }

        // Check performance and update statistics
        if let Some(start_time) = start {
            let duration = start_time.elapsed();
            let millis = duration.as_millis();

            // Update statistics
            self.session_stats.total_execution_time_ms += millis;
            if millis < self.session_stats.min_execution_time_ms {
                self.session_stats.min_execution_time_ms = millis;
            }
            if millis > self.session_stats.max_execution_time_ms {
                self.session_stats.max_execution_time_ms = millis;
            }
            self.session_stats.avg_execution_time_ms = self.session_stats.total_execution_time_ms
                / u128::from(self.session_stats.commands_executed);

            if self.perf_monitoring {
                println!("⏱️ {millis} ms");

                // Show memory delta if significant
                if let Some((before, after)) = self.session_stats.memory_delta {
                    let delta = i64::try_from(after).unwrap_or(i64::MAX)
                        - i64::try_from(before).unwrap_or(0);
                    if delta.abs() > 1024 * 1024 {
                        // > 1MB change
                        let delta_mb = delta as f64 / 1024.0 / 1024.0;
                        println!("💾 Memory Δ: {delta_mb:+.2} MB");
                    }
                }
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
    #[allow(clippy::too_many_lines)]
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
                    // Calculate max name width for alignment
                    let max_width = state
                        .variables
                        .keys()
                        .map(std::string::String::len)
                        .max()
                        .unwrap_or(0)
                        .min(30); // Cap at 30 chars for readability

                    println!("📦 Session Variables:");
                    println!("{}", "─".repeat(60));

                    // Sort variables by name for consistent display
                    let mut vars: Vec<_> = state.variables.iter().collect();
                    vars.sort_by_key(|(k, _)| k.as_str());

                    for (name, value) in vars {
                        // Detect type based on value content
                        let type_hint = detect_value_type(value);
                        let display_value = format_value(value, 40); // Truncate long values

                        println!("  {name:<max_width$} : {type_hint} {display_value}");
                    }
                    println!("{}", "─".repeat(60));
                    println!(
                        "Total: {} variable{}",
                        state.variables.len(),
                        if state.variables.len() == 1 { "" } else { "s" }
                    );
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
                // Ensure debug session exists
                if self.debug_session.is_none() {
                    self.start_debug_session()?;
                }

                // Get debug session
                let session = self
                    .debug_session
                    .as_mut()
                    .ok_or_else(|| anyhow::anyhow!("Debug session not available"))?;

                // Set breakpoint via ExecutionManager
                let file = spec.file.clone().unwrap_or_else(|| "current".to_string());
                let line =
                    u32::try_from(spec.line).map_err(|_| anyhow::anyhow!("Invalid line number"))?;
                let bp = session
                    .execution_manager
                    .set_breakpoint(file.clone(), line)
                    .map_err(|e| anyhow::anyhow!("Failed to set breakpoint: {}", e))?;

                // Track in session
                session
                    .breakpoints
                    .entry(file.clone())
                    .or_default()
                    .push(line);

                // Also track in state for display
                let mut state = self.state.write().await;
                let bp_id = state.breakpoints.len();
                let mut breakpoint = Breakpoint::new(bp_id, spec.line);

                if let Some(file_path) = spec.file {
                    breakpoint = breakpoint.with_file(file_path);
                }
                if let Some(condition) = spec.condition {
                    breakpoint = breakpoint.with_condition(condition);
                }

                state.add_breakpoint(breakpoint.clone());
                println!("🔴 Breakpoint set at {}:{} (id: {})", file, line, bp.id);
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
    fn handle_execution_command(&mut self, command: &DebugCommand) -> Result<Option<()>> {
        // Get debug session
        let session = self
            .debug_session
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Debug session not active"))?;

        match *command {
            DebugCommand::Step => {
                if session.is_paused() {
                    session
                        .execution_manager
                        .resume(crate::debug::execution_bridge::StepMode::StepIn);
                    println!("➡️  Stepping into...");
                } else {
                    println!("⚠️  Not paused at breakpoint");
                }
                Ok(Some(()))
            }
            DebugCommand::Next => {
                if session.is_paused() {
                    session
                        .execution_manager
                        .resume(crate::debug::execution_bridge::StepMode::StepOver);
                    println!("➡️  Stepping over...");
                } else {
                    println!("⚠️  Not paused at breakpoint");
                }
                Ok(Some(()))
            }
            DebugCommand::Finish => {
                if session.is_paused() {
                    session
                        .execution_manager
                        .resume(crate::debug::execution_bridge::StepMode::StepOut);
                    println!("➡️  Stepping out...");
                } else {
                    println!("⚠️  Not paused at breakpoint");
                }
                Ok(Some(()))
            }
            DebugCommand::Continue => {
                if session.is_paused() {
                    session
                        .execution_manager
                        .resume(crate::debug::execution_bridge::StepMode::Continue);
                    println!("▶️  Continuing execution...");
                } else {
                    println!("⚠️  Not paused at breakpoint");
                }
                Ok(Some(()))
            }
            DebugCommand::Pause => {
                // This would pause a running script - not implemented yet
                println!("⚠️  Pause not yet implemented for running scripts");
                Ok(Some(()))
            }
            _ => Ok(None), // Not an execution command
        }
    }

    /// Handle information display commands
    fn handle_info_command(&mut self, command: &DebugCommand) -> Option<()> {
        match *command {
            DebugCommand::Locals => {
                // Get debug session
                if let Some(ref session) = self.debug_session {
                    if session.is_paused() {
                        let frame_id = session.current_frame.unwrap_or(0);
                        let vars = session.execution_manager.get_variables(
                            &crate::debug::execution_bridge::VariableScope::Local,
                            Some(&frame_id.to_string()),
                        );
                        if vars.is_empty() {
                            println!("No local variables");
                        } else {
                            println!("📦 Local Variables:");
                            for var in vars {
                                println!("  {} = {} ({})", var.name, var.value, var.var_type);
                            }
                        }
                    } else {
                        println!("⚠️  Not paused at breakpoint");
                    }
                } else {
                    println!("⚠️  Debug session not active");
                }
                Some(())
            }
            DebugCommand::Backtrace | DebugCommand::Where => {
                // Get debug session
                if let Some(ref session) = self.debug_session {
                    if session.is_paused() {
                        let frames = session.execution_manager.get_stack_frames();
                        if frames.is_empty() {
                            println!("No stack frames available");
                        } else {
                            println!("📚 Call Stack:");
                            for (i, frame) in frames.iter().enumerate() {
                                let marker = if Some(i) == session.current_frame {
                                    "→"
                                } else {
                                    " "
                                };
                                println!(
                                    "{} #{}: {} at {}:{}",
                                    marker, i, frame.name, frame.source, frame.line
                                );
                            }
                        }
                    } else {
                        println!("⚠️  Not paused at breakpoint");
                    }
                } else {
                    println!("⚠️  Debug session not active");
                }
                Some(())
            }
            DebugCommand::Frame(n) => {
                // Get debug session
                if let Some(ref mut session) = self.debug_session {
                    if session.is_paused() {
                        let frames = session.execution_manager.get_stack_frames();
                        if n < frames.len() {
                            session.current_frame = Some(n);
                            println!("Selected frame #{n}");
                        } else {
                            println!("Invalid frame number (max: {})", frames.len() - 1);
                        }
                    } else {
                        println!("⚠️  Not paused at breakpoint");
                    }
                } else {
                    println!("⚠️  Debug session not active");
                }
                Some(())
            }
            _ => None, // Not an info command
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

        // For execution commands
        if matches!(
            command,
            DebugCommand::Step
                | DebugCommand::Next
                | DebugCommand::Finish
                | DebugCommand::Continue
                | DebugCommand::Pause
        ) {
            if let Some(()) = self.handle_execution_command(&command)? {
                return Ok(());
            }
        }

        if let Some(()) = self.handle_info_command(&command) {
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

        println!("📊 Session Information:");
        println!("{}", "─".repeat(60));

        // Basic info
        println!("📅 Session:");
        println!("  Uptime: {:.1}s", uptime.as_secs_f64());
        println!("  Executions: {}", self.execution_count);
        println!("  History entries: {}", state.history.entries().len());
        println!("  Variables: {}", state.variables.len());
        println!("  Breakpoints: {}", state.breakpoints.len());
        println!("  Debug mode: {}", self.debug_session.is_some());

        // Performance statistics
        if self.session_stats.commands_executed > 0 {
            println!("\n⏱️ Performance:");
            println!(
                "  Total execution time: {} ms",
                self.session_stats.total_execution_time_ms
            );
            println!(
                "  Average time: {} ms",
                self.session_stats.avg_execution_time_ms
            );
            println!(
                "  Min time: {} ms",
                if self.session_stats.min_execution_time_ms == u128::MAX {
                    0
                } else {
                    self.session_stats.min_execution_time_ms
                }
            );
            println!(
                "  Max time: {} ms",
                self.session_stats.max_execution_time_ms
            );
            println!(
                "  Commands executed: {}",
                self.session_stats.commands_executed
            );
            println!(
                "  Errors encountered: {}",
                self.session_stats.errors_encountered
            );
        }

        // Memory statistics
        let current_memory = get_current_memory_usage();
        println!("\n💾 Memory:");
        println!(
            "  Current: {:.2} MB",
            current_memory as f64 / 1024.0 / 1024.0
        );
        if self.session_stats.peak_memory_bytes > 0 {
            println!(
                "  Peak: {:.2} MB",
                self.session_stats.peak_memory_bytes as f64 / 1024.0 / 1024.0
            );
        }
        if let Some((before, after)) = self.session_stats.memory_delta {
            let delta =
                i64::try_from(after).unwrap_or(i64::MAX) - i64::try_from(before).unwrap_or(0);
            println!("  Last delta: {:+.2} MB", delta as f64 / 1024.0 / 1024.0);
        }

        println!("{}", "─".repeat(60));
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

    /// Start a debug session
    fn start_debug_session(&mut self) -> Result<()> {
        if self.debug_session.is_some() {
            println!("Debug session already active");
            return Ok(());
        }

        let exec_mgr = self
            .execution_manager
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Debug commands not enabled"))?
            .clone();

        // Enable debug mode on execution manager
        exec_mgr.enable_debug_mode();

        // Get script executor from kernel
        let script_executor = self.kernel.get_script_executor();

        // Set debug context on script executor
        script_executor.set_debug_context(Some(exec_mgr.clone()));

        // Create debug session
        let mut debug_session = ReplDebugSession::new(exec_mgr, script_executor);

        // Start listening for stopped events
        if let Some(mut stopped_rx) = debug_session.take_stopped_receiver() {
            let paused = debug_session.paused.clone();
            let pause_location = debug_session.pause_location.clone();

            tokio::spawn(async move {
                while let Some(event) = stopped_rx.recv().await {
                    println!("⏸️  Paused at {}:{}", event.file, event.line);
                    println!("   Reason: {}", event.reason);
                    paused.store(true, Ordering::SeqCst);
                    *pause_location.write().await = Some((event.file, event.line));
                }
            });
        }

        self.debug_session = Some(debug_session);
        println!("🔷 Debug session started");
        Ok(())
    }

    /// Stop the debug session
    #[allow(dead_code)]
    fn stop_debug_session(&mut self) {
        if let Some(_session) = self.debug_session.take() {
            // Clear debug context from script executor
            let script_executor = self.kernel.get_script_executor();
            script_executor.set_debug_context(None);

            // Disable debug mode on execution manager
            if let Some(exec_mgr) = &self.execution_manager {
                exec_mgr.disable_debug_mode();
            }

            println!("Debug session stopped");
        } else {
            println!("No debug session active");
        }
    }

    /// Get prompt string based on current state
    fn get_prompt(&self) -> String {
        if let Some(ref session) = self.debug_session {
            if session.is_paused() {
                return "(debug) > ".to_string();
            }
        }

        if self.multiline_buffer.is_empty() {
            "> ".to_string()
        } else {
            "... ".to_string()
        }
    }
}

/// Detect value type based on string content
fn detect_value_type(value: &str) -> &'static str {
    // Try to parse as various types
    if value == "true" || value == "false" {
        "bool"
    } else if value.parse::<i64>().is_ok() {
        "int"
    } else if value.parse::<f64>().is_ok() {
        "float"
    } else if value.starts_with('"') && value.ends_with('"') {
        "string"
    } else if value.starts_with('[') && value.ends_with(']') {
        "array"
    } else if value.starts_with('{') && value.ends_with('}') {
        "object"
    } else if value == "nil" || value == "null" || value == "None" {
        "null"
    } else if value.starts_with('/') || value.contains('\\') {
        "path"
    } else {
        "string"
    }
}

/// Format value for display with truncation
fn format_value(value: &str, max_len: usize) -> String {
    let cleaned = value.trim();

    // Handle special cases
    if cleaned.starts_with('{') && cleaned.ends_with('}') {
        // Try to format as JSON object
        if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(cleaned) {
            let compact = serde_json::to_string(&json_val).unwrap_or_else(|_| cleaned.to_string());
            if compact.len() > max_len {
                format!("{}...}}", &compact[..max_len.saturating_sub(4)])
            } else {
                compact
            }
        } else if cleaned.len() > max_len {
            format!("{}...}}", &cleaned[..max_len.saturating_sub(4)])
        } else {
            cleaned.to_string()
        }
    } else if cleaned.starts_with('[') && cleaned.ends_with(']') {
        // Try to format as JSON array
        if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(cleaned) {
            let compact = serde_json::to_string(&json_val).unwrap_or_else(|_| cleaned.to_string());
            if compact.len() > max_len {
                format!("{}...]", &compact[..max_len.saturating_sub(4)])
            } else {
                compact
            }
        } else if cleaned.len() > max_len {
            format!("{}...]", &cleaned[..max_len.saturating_sub(4)])
        } else {
            cleaned.to_string()
        }
    } else if cleaned.len() > max_len {
        format!("{}...", &cleaned[..max_len.saturating_sub(3)])
    } else {
        cleaned.to_string()
    }
}

/// Get current memory usage in bytes
fn get_current_memory_usage() -> u64 {
    // Try to get memory stats from procfs on Linux
    #[cfg(target_os = "linux")]
    {
        if let Ok(contents) = std::fs::read_to_string("/proc/self/status") {
            for line in contents.lines() {
                if line.starts_with("VmRSS:") {
                    if let Some(kb_str) = line.split_whitespace().nth(1) {
                        if let Ok(kb) = kb_str.parse::<u64>() {
                            return kb * 1024; // Convert KB to bytes
                        }
                    }
                }
            }
        }
    }

    // Try to get memory stats on macOS
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        if let Ok(output) = Command::new("ps")
            .arg("-o")
            .arg("rss=")
            .arg("-p")
            .arg(std::process::id().to_string())
            .output()
        {
            if let Ok(rss_str) = String::from_utf8(output.stdout) {
                if let Ok(kb) = rss_str.trim().parse::<u64>() {
                    return kb * 1024; // Convert KB to bytes
                }
            }
        }
    }

    // Fallback: return 0 if we can't get memory stats
    0
}
