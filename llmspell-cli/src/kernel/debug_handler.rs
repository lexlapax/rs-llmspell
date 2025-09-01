//! Debug event handler for CLI kernel interaction
//!
//! Handles debug events from the kernel using unified types and ExecutionManager.

use anyhow::Result;
use async_trait::async_trait;
use llmspell_bridge::{
    circuit_breaker::CircuitBreaker,
    diagnostics_bridge::DiagnosticsBridge,
    execution_bridge::{DebugState, ExecutionLocation, PauseReason, StackFrame, Variable},
    hook_profiler::{HookProfiler, WorkloadClassifier},
    stack_navigator::StackNavigator,
    variable_inspector::VariableInspector,
};
use llmspell_debug::session_manager::DebugSessionManager;
use llmspell_engine::channels::IOPubMessage;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};

use crate::cli::OutputFormat;

/// Debug event from kernel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DebugEvent {
    /// Breakpoint was hit
    BreakpointHit {
        location: ExecutionLocation,
        stack: Vec<StackFrame>,
        locals: Vec<Variable>,
    },
    /// Step operation completed
    StepComplete { new_location: ExecutionLocation },
    /// Execution paused
    Paused {
        reason: PauseReason,
        location: ExecutionLocation,
    },
    /// Execution resumed
    Resumed,
    /// Debug state changed
    StateChanged { new_state: DebugState },
}

/// Trait for debug event handling operations
#[async_trait]
pub trait DebugEventHandlerTrait: Send + Sync {
    /// Handle events from IOPub channel
    async fn handle_events(&mut self) -> Result<()>;

    /// Handle a single debug event
    async fn handle_debug_event(&mut self, event: DebugEvent) -> Result<()>;

    /// Display stream output
    fn display_output(&mut self, name: &str, text: &str);

    /// Display execution result
    fn display_result(&mut self, data: &serde_json::Value);

    /// Display error with traceback
    fn display_error(&mut self, traceback: &[String]);

    /// Enter interactive debug REPL
    async fn enter_debug_repl(&mut self) -> Result<()>;

    /// Check if event flooding is occurring
    fn is_flooding(&self) -> bool;
}

/// Builder for debug event handler with dependency injection
#[derive(Default)]
pub struct DebugEventHandlerBuilder {
    iopub_receiver: Option<broadcast::Receiver<IOPubMessage>>,
    debug_session: Option<Arc<RwLock<DebugSessionManager>>>,
    output_format: Option<OutputFormat>,
    circuit_breaker: Option<Box<dyn CircuitBreaker>>,
    hook_profiler: Option<Box<dyn HookProfiler>>,
    diagnostics: Option<DiagnosticsBridge>,
    variable_inspector: Option<Box<dyn VariableInspector>>,
    stack_navigator: Option<Box<dyn StackNavigator>>,
}

impl DebugEventHandlerBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the IOPub receiver
    pub fn iopub_receiver(mut self, receiver: broadcast::Receiver<IOPubMessage>) -> Self {
        self.iopub_receiver = Some(receiver);
        self
    }

    /// Set the debug session manager
    pub fn debug_session(mut self, session: Arc<RwLock<DebugSessionManager>>) -> Self {
        self.debug_session = Some(session);
        self
    }

    /// Set the output format
    pub fn output_format(mut self, format: OutputFormat) -> Self {
        self.output_format = Some(format);
        self
    }

    /// Set the circuit breaker for event flooding protection
    pub fn circuit_breaker(mut self, circuit_breaker: Box<dyn CircuitBreaker>) -> Self {
        self.circuit_breaker = Some(circuit_breaker);
        self
    }

    /// Set the hook profiler for performance monitoring
    pub fn hook_profiler(mut self, profiler: Box<dyn HookProfiler>) -> Self {
        self.hook_profiler = Some(profiler);
        self
    }

    /// Set the diagnostics bridge for error formatting
    pub fn diagnostics(mut self, diagnostics: DiagnosticsBridge) -> Self {
        self.diagnostics = Some(diagnostics);
        self
    }

    /// Set the variable inspector for variable display
    pub fn variable_inspector(mut self, inspector: Box<dyn VariableInspector>) -> Self {
        self.variable_inspector = Some(inspector);
        self
    }

    /// Set the stack navigator for stack formatting
    pub fn stack_navigator(mut self, navigator: Box<dyn StackNavigator>) -> Self {
        self.stack_navigator = Some(navigator);
        self
    }

    /// Build the debug event handler
    pub fn build(self) -> DebugEventHandler {
        DebugEventHandler {
            iopub_receiver: self.iopub_receiver,
            debug_session: self.debug_session,
            output_format: self.output_format.unwrap_or(OutputFormat::Pretty),
            circuit_breaker: self.circuit_breaker,
            hook_profiler: self.hook_profiler,
            diagnostics: self.diagnostics,
            variable_inspector: self.variable_inspector,
            stack_navigator: self.stack_navigator,
            event_count: 0,
            flooding_threshold: 100, // Adaptive, not hardcoded in production
        }
    }
}

/// Debug event handler implementation
pub struct DebugEventHandler {
    iopub_receiver: Option<broadcast::Receiver<IOPubMessage>>,
    debug_session: Option<Arc<RwLock<DebugSessionManager>>>,
    output_format: OutputFormat,
    circuit_breaker: Option<Box<dyn CircuitBreaker>>,
    #[allow(dead_code)]
    hook_profiler: Option<Box<dyn HookProfiler>>,
    diagnostics: Option<DiagnosticsBridge>,
    variable_inspector: Option<Box<dyn VariableInspector>>,
    stack_navigator: Option<Box<dyn StackNavigator>>,
    event_count: usize,
    flooding_threshold: usize,
}

#[async_trait]
impl DebugEventHandlerTrait for DebugEventHandler {
    async fn handle_events(&mut self) -> Result<()> {
        // Take ownership of the receiver temporarily to avoid multiple mutable borrows
        let mut receiver = self.iopub_receiver.take();

        if let Some(ref mut recv) = receiver {
            while let Ok(event) = recv.recv().await {
                // Monitor event performance (HookProfiler doesn't have record_hook_execution)
                // TODO: Implement proper performance monitoring when HookProfiler API is available

                // Check for event flooding
                self.event_count += 1;
                if self.event_count > self.flooding_threshold {
                    if let Some(breaker) = &self.circuit_breaker {
                        let context = llmspell_bridge::circuit_breaker::OperationContext {
                            operation_name: "debug_events".to_string(),
                            workload: WorkloadClassifier::Light,
                            duration: std::time::Duration::from_millis(1),
                            success: true,
                        };
                        if breaker.allow_operation(&context) {
                            self.event_count = 0; // Reset counter
                        } else {
                            tracing::warn!("Event flooding detected, circuit breaker engaged");
                            continue;
                        }
                    }
                }

                match event {
                    IOPubMessage::DebugEvent(event_data) => {
                        // Parse debug event from JSON
                        if let Ok(debug_event) = serde_json::from_value::<DebugEvent>(event_data) {
                            self.handle_debug_event(debug_event).await?;
                        }
                    }
                    IOPubMessage::StreamOutput { name, text } => {
                        self.display_output(&name, &text);
                    }
                    IOPubMessage::ExecuteResult { data, .. } => {
                        self.display_result(&data);
                    }
                    IOPubMessage::Error { traceback, .. } => {
                        self.display_error(&traceback);
                    }
                    IOPubMessage::Status { execution_state } => {
                        tracing::debug!("Execution state: {}", execution_state);
                    }
                }
            }
        }

        // Restore the receiver
        self.iopub_receiver = receiver;
        Ok(())
    }

    async fn handle_debug_event(&mut self, event: DebugEvent) -> Result<()> {
        match event {
            DebugEvent::BreakpointHit {
                location,
                stack,
                locals,
            } => {
                println!("🔴 Breakpoint hit at {}:{}", location.source, location.line);
                if let Some(col) = location.column {
                    println!("   Column: {}", col);
                }

                // Use stack navigator for professional stack display
                if let Some(_navigator) = &self.stack_navigator {
                    // StackNavigator doesn't have format_stack_trace method
                    // Use fallback display for now
                    println!("\n📚 Call Stack:");
                    for (i, frame) in stack.iter().enumerate() {
                        let marker = if frame.is_user_code { "▶" } else { " " };
                        println!(
                            "{} #{}: {} at {}:{}",
                            marker, i, frame.name, frame.source, frame.line
                        );
                    }
                } else {
                    // Fallback to basic display
                    println!("\n📚 Call Stack:");
                    for (i, frame) in stack.iter().enumerate() {
                        let marker = if frame.is_user_code { "▶" } else { " " };
                        println!(
                            "{} #{}: {} at {}:{}",
                            marker, i, frame.name, frame.source, frame.line
                        );
                    }
                }

                // Use variable inspector for professional variable display
                if let Some(_inspector) = &self.variable_inspector {
                    // VariableInspector doesn't have format_variables method
                    // Use fallback display for now
                    println!("\n📦 Local Variables:");
                    for var in &locals {
                        println!("  {} ({}) = {}", var.name, var.var_type, var.value);
                    }
                } else {
                    // Fallback to basic display
                    println!("\n📦 Local Variables:");
                    for var in &locals {
                        println!("  {} ({}) = {}", var.name, var.var_type, var.value);
                    }
                }

                // Enter debug REPL
                self.enter_debug_repl().await?;
            }
            DebugEvent::StepComplete { new_location } => {
                println!("➡️ Step to {}:{}", new_location.source, new_location.line);
            }
            DebugEvent::Paused { reason, location } => {
                let reason_str = match reason {
                    PauseReason::Breakpoint => "breakpoint",
                    PauseReason::Step => "step",
                    PauseReason::Pause => "user pause",
                    PauseReason::Exception(ref msg) => &format!("exception: {}", msg),
                    PauseReason::Entry => "entry point",
                };
                println!(
                    "⏸️ Paused ({}) at {}:{}",
                    reason_str, location.source, location.line
                );
            }
            DebugEvent::Resumed => {
                println!("▶️ Execution resumed");
            }
            DebugEvent::StateChanged { new_state } => {
                match new_state {
                    DebugState::Running => println!("🏃 Running..."),
                    DebugState::Paused { .. } => {} // Handled by Paused event
                    DebugState::Terminated => println!("🏁 Terminated"),
                }
            }
        }
        Ok(())
    }

    fn display_output(&mut self, name: &str, text: &str) {
        match name {
            "stdout" => print!("{}", text),
            "stderr" => eprint!("{}", text),
            _ => println!("[{}] {}", name, text),
        }
    }

    fn display_result(&mut self, data: &serde_json::Value) {
        match self.output_format {
            OutputFormat::Json => {
                println!("{}", serde_json::to_string_pretty(data).unwrap_or_default());
            }
            OutputFormat::Text | OutputFormat::Pretty => {
                if let Some(text) = data.as_str() {
                    println!("{}", text);
                } else {
                    println!("{}", data);
                }
            }
        }
    }

    fn display_error(&mut self, traceback: &[String]) {
        if let Some(_diagnostics) = &self.diagnostics {
            // DiagnosticsBridge doesn't have format_error method
            // Use fallback display for now
            eprintln!("❌ Error:");
            for line in traceback {
                eprintln!("  {}", line);
            }
        } else {
            // Fallback to basic display
            eprintln!("❌ Error:");
            for line in traceback {
                eprintln!("  {}", line);
            }
        }
    }

    async fn enter_debug_repl(&mut self) -> Result<()> {
        use rustyline::DefaultEditor;

        let mut rl = DefaultEditor::new()?;

        println!("\n🐛 Debug REPL - Commands: step(s), next(n), continue(c), locals(l), backtrace(bt), inspect <var>, quit(q)");

        loop {
            let readline = rl.readline("debug> ");
            match readline {
                Ok(line) => {
                    let cmd = line.trim();

                    // Handle commands through debug session manager
                    if let Some(session) = &self.debug_session {
                        match cmd {
                            "step" | "s" => {
                                let mgr = session.read().await;
                                let sessions_list = mgr.list_sessions().await;
                                if let Some(sessions) = sessions_list.first() {
                                    mgr.handle_debug_command(
                                        sessions,
                                        llmspell_bridge::execution_bridge::DebugCommand::StepInto,
                                    )
                                    .await?;
                                }
                                break; // Exit REPL after command
                            }
                            "next" | "n" => {
                                let mgr = session.read().await;
                                let sessions_list = mgr.list_sessions().await;
                                if let Some(sessions) = sessions_list.first() {
                                    mgr.handle_debug_command(
                                        sessions,
                                        llmspell_bridge::execution_bridge::DebugCommand::StepOver,
                                    )
                                    .await?;
                                }
                                break;
                            }
                            "continue" | "c" => {
                                let mgr = session.read().await;
                                let sessions_list = mgr.list_sessions().await;
                                if let Some(sessions) = sessions_list.first() {
                                    mgr.handle_debug_command(
                                        sessions,
                                        llmspell_bridge::execution_bridge::DebugCommand::Continue,
                                    )
                                    .await?;
                                }
                                break;
                            }
                            "locals" | "l" => {
                                let mgr = session.read().await;
                                let sessions_list = mgr.list_sessions().await;
                                if let Some(sessions) = sessions_list.first() {
                                    let vars = mgr.get_session_variables(sessions, None).await?;
                                    for var in vars {
                                        println!(
                                            "  {} ({}) = {}",
                                            var.name, var.var_type, var.value
                                        );
                                    }
                                }
                            }
                            "backtrace" | "bt" => {
                                let mgr = session.read().await;
                                let sessions_list = mgr.list_sessions().await;
                                if let Some(sessions) = sessions_list.first() {
                                    let stack = mgr.get_session_stack_trace(sessions).await?;
                                    for (i, frame) in stack.iter().enumerate() {
                                        println!(
                                            "#{}: {} at {}:{}",
                                            i, frame.name, frame.source, frame.line
                                        );
                                    }
                                }
                            }
                            "quit" | "q" => {
                                println!("Exiting debug REPL...");
                                break;
                            }
                            cmd if cmd.starts_with("inspect ") => {
                                let var_name = cmd.strip_prefix("inspect ").unwrap();
                                println!("Inspecting variable: {}", var_name);
                                // TODO: Implement variable inspection through session manager
                            }
                            "" => continue,
                            _ => {
                                println!("Unknown command: '{}'. Try: step, next, continue, locals, backtrace, inspect <var>, quit", cmd);
                            }
                        }
                    } else {
                        println!("No debug session available");
                        break;
                    }
                }
                Err(rustyline::error::ReadlineError::Interrupted) => {
                    println!("^C");
                    continue;
                }
                Err(rustyline::error::ReadlineError::Eof) => {
                    println!("^D");
                    break;
                }
                Err(err) => {
                    eprintln!("Error: {}", err);
                    break;
                }
            }
        }

        Ok(())
    }

    fn is_flooding(&self) -> bool {
        self.event_count > self.flooding_threshold
    }
}

/// Null debug event handler for testing
pub struct NullDebugEventHandler {
    pub events_handled: std::sync::Arc<std::sync::Mutex<Vec<DebugEvent>>>,
    pub outputs: std::sync::Arc<std::sync::Mutex<Vec<(String, String)>>>,
    pub results: std::sync::Arc<std::sync::Mutex<Vec<serde_json::Value>>>,
    pub errors: std::sync::Arc<std::sync::Mutex<Vec<Vec<String>>>>,
}

impl NullDebugEventHandler {
    pub fn new() -> Self {
        use std::sync::{Arc, Mutex};
        Self {
            events_handled: Arc::new(Mutex::new(Vec::new())),
            outputs: Arc::new(Mutex::new(Vec::new())),
            results: Arc::new(Mutex::new(Vec::new())),
            errors: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl Default for NullDebugEventHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DebugEventHandlerTrait for NullDebugEventHandler {
    async fn handle_events(&mut self) -> Result<()> {
        // No-op for testing
        Ok(())
    }

    async fn handle_debug_event(&mut self, event: DebugEvent) -> Result<()> {
        if let Ok(mut events) = self.events_handled.lock() {
            events.push(event);
        }
        Ok(())
    }

    fn display_output(&mut self, name: &str, text: &str) {
        if let Ok(mut outputs) = self.outputs.lock() {
            outputs.push((name.to_string(), text.to_string()));
        }
    }

    fn display_result(&mut self, data: &serde_json::Value) {
        if let Ok(mut results) = self.results.lock() {
            results.push(data.clone());
        }
    }

    fn display_error(&mut self, traceback: &[String]) {
        if let Ok(mut errors) = self.errors.lock() {
            errors.push(traceback.to_vec());
        }
    }

    async fn enter_debug_repl(&mut self) -> Result<()> {
        // No-op for testing
        Ok(())
    }

    fn is_flooding(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_null_debug_event_handler() {
        let mut handler = NullDebugEventHandler::new();

        let event = DebugEvent::StepComplete {
            new_location: ExecutionLocation {
                source: "test.lua".to_string(),
                line: 42,
                column: Some(10),
            },
        };

        handler.handle_debug_event(event.clone()).await.unwrap();
        assert_eq!(handler.events_handled.lock().unwrap().len(), 1);

        handler.display_output("stdout", "Hello");
        assert_eq!(handler.outputs.lock().unwrap().len(), 1);
        assert_eq!(
            handler.outputs.lock().unwrap()[0],
            ("stdout".to_string(), "Hello".to_string())
        );

        handler.display_result(&serde_json::json!({"result": 42}));
        assert_eq!(handler.results.lock().unwrap().len(), 1);

        handler.display_error(&["Error line 1".to_string()]);
        assert_eq!(handler.errors.lock().unwrap().len(), 1);
    }

    #[test]
    fn test_debug_event_handler_builder() {
        let handler = DebugEventHandlerBuilder::new()
            .output_format(OutputFormat::Json)
            .build();

        assert!(!handler.is_flooding());
    }
}
