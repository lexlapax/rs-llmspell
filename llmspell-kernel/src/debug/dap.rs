//! DAP (Debug Adapter Protocol) Bridge
//!
//! Provides a bridge between the Debug Adapter Protocol and the `ExecutionManager`,
//! enabling IDE debugging support for VS Code and other DAP-compliant editors.
//!
//! Migrated from Phase-9 branch (originally 743 lines)

use super::execution_bridge::{ExecutionManager, StepMode, Variable, VariableScope};
use anyhow::Result;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicI32, Ordering};
use std::sync::Arc;
use tracing::{debug, info, instrument, warn};

/// DAP Initialize request arguments
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::struct_excessive_bools)]
pub struct InitializeArguments {
    /// Adapter ID
    #[serde(default)]
    pub adapter_id: String,
    /// Locale for messages
    #[serde(default)]
    pub locale: String,
    /// Whether lines start at 1 or 0
    #[serde(default = "default_true")]
    pub lines_start_at1: bool,
    /// Whether columns start at 1 or 0
    #[serde(default = "default_true")]
    pub columns_start_at1: bool,
    /// Path format (path or uri)
    #[serde(default)]
    pub path_format: String,
    /// Whether client supports variable types
    #[serde(default)]
    pub supports_variable_type: bool,
    /// Whether client supports variable paging
    #[serde(default)]
    pub supports_variable_paging: bool,
    /// Whether client supports run in terminal request
    #[serde(default)]
    pub supports_run_in_terminal_request: bool,
}

fn default_true() -> bool {
    true
}

/// DAP Capabilities response
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[allow(clippy::struct_excessive_bools)]
pub struct Capabilities {
    /// Supports configuration done request
    pub supports_configuration_done_request: bool,
    /// Supports function breakpoints
    pub supports_function_breakpoints: bool,
    /// Supports conditional breakpoints
    pub supports_conditional_breakpoints: bool,
    /// Supports hit conditional breakpoints
    pub supports_hit_conditional_breakpoints: bool,
    /// Supports evaluate for hovers
    pub supports_evaluate_for_hovers: bool,
    /// Supports step back
    pub supports_step_back: bool,
    /// Supports set variable
    pub supports_set_variable: bool,
    /// Supports restart frame
    pub supports_restart_frame: bool,
    /// Supports step in targets request
    pub supports_step_in_targets_request: bool,
    /// Supports delayed stack trace loading
    pub supports_delayed_stack_trace_loading: bool,
    /// Supports loaded sources request
    pub supports_loaded_sources_request: bool,
    /// Supports log points
    pub supports_log_points: bool,
    /// Supports terminate request
    pub supports_terminate_request: bool,
    /// Supports set expression
    pub supports_set_expression: bool,
    /// Supports terminate threads request
    pub supports_terminate_threads_request: bool,
    /// Supports read memory request
    pub supports_read_memory_request: bool,
    /// Supports write memory request
    pub supports_write_memory_request: bool,
    /// Supports stepping granularity
    pub supports_stepping_granularity: bool,
    /// Supports instruction breakpoints
    pub supports_instruction_breakpoints: bool,
    /// Supports exception info request
    pub supports_exception_info_request: bool,
    /// Supports exception conditions
    pub supports_exception_conditions: bool,
    /// Supports exception filter options
    pub supports_exception_filter_options: bool,
    /// Supports value formatting options
    pub supports_value_formatting_options: bool,
    /// Supports clipboard context
    pub supports_clipboard_context: bool,
}

impl Default for Capabilities {
    fn default() -> Self {
        Self {
            supports_configuration_done_request: true,
            supports_function_breakpoints: false,
            supports_conditional_breakpoints: true,
            supports_hit_conditional_breakpoints: true,
            supports_evaluate_for_hovers: true,
            supports_step_back: false,
            supports_set_variable: true,
            supports_restart_frame: false,
            supports_step_in_targets_request: false,
            supports_delayed_stack_trace_loading: false,
            supports_loaded_sources_request: true,
            supports_log_points: false,
            supports_terminate_request: true,
            supports_set_expression: false,
            supports_terminate_threads_request: false,
            supports_read_memory_request: false,
            supports_write_memory_request: false,
            supports_stepping_granularity: false,
            supports_instruction_breakpoints: false,
            supports_exception_info_request: false,
            supports_exception_conditions: false,
            supports_exception_filter_options: false,
            supports_value_formatting_options: false,
            supports_clipboard_context: false,
        }
    }
}

/// DAP Source reference
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    /// Name of the source
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Path to the source
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Source reference for virtual sources
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_reference: Option<i32>,
    /// Presentation hint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presentation_hint: Option<String>,
    /// Origin of the source
    #[serde(skip_serializing_if = "Option::is_none")]
    pub origin: Option<String>,
}

/// DAP Source breakpoint
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceBreakpoint {
    /// Line number
    pub line: u32,
    /// Optional column
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<u32>,
    /// Optional condition
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    /// Optional hit condition
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hit_condition: Option<String>,
    /// Optional log message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_message: Option<String>,
}

/// DAP Breakpoint
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DapBreakpoint {
    /// Breakpoint ID
    pub id: Option<String>,
    /// Whether breakpoint is verified
    pub verified: bool,
    /// Optional message
    pub message: Option<String>,
    /// Source location
    pub source: Option<Source>,
    /// Line number
    pub line: Option<u32>,
    /// Column number
    pub column: Option<u32>,
    /// End line
    pub end_line: Option<u32>,
    /// End column
    pub end_column: Option<u32>,
}

/// DAP Stack frame
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DapStackFrame {
    /// Frame ID
    pub id: i32,
    /// Frame name
    pub name: String,
    /// Source location
    pub source: Option<Source>,
    /// Line number
    pub line: i32,
    /// Column number
    pub column: i32,
    /// End line
    pub end_line: Option<i32>,
    /// End column
    pub end_column: Option<i32>,
    /// Module ID
    pub module_id: Option<String>,
    /// Presentation hint
    pub presentation_hint: Option<String>,
}

/// DAP Variable
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DapVariable {
    /// Variable name
    pub name: String,
    /// Variable value
    pub value: String,
    /// Variable type
    #[serde(skip_serializing_if = "Option::is_none")]
    pub var_type: Option<String>,
    /// Presentation hint
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presentation_hint: Option<String>,
    /// Evaluate name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evaluate_name: Option<String>,
    /// Variables reference for structured variables
    pub variables_reference: i32,
    /// Named variables count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub named_variables: Option<i32>,
    /// Indexed variables count
    #[serde(skip_serializing_if = "Option::is_none")]
    pub indexed_variables: Option<i32>,
}

/// Source reference for IDE mapping
#[derive(Debug, Clone)]
pub struct SourceReference {
    /// Script ID
    pub script_id: u32,
    /// File path
    pub path: String,
    /// Source content (for virtual sources)
    pub content: Option<String>,
}

/// DAP Bridge for Debug Adapter Protocol support
pub struct DAPBridge {
    /// Execution manager
    execution_manager: Option<Arc<ExecutionManager>>,
    /// Whether initialized
    initialized: AtomicBool,
    /// Sequence number for messages
    next_seq: AtomicI32,
    /// Source mapping
    source_map: Arc<RwLock<HashMap<u32, SourceReference>>>,
    /// Variable references
    variable_refs: Arc<RwLock<HashMap<i32, Vec<Variable>>>>,
    /// Next variable reference ID
    next_var_ref: AtomicI32,
    /// Session ID
    _session_id: String,
}

impl DAPBridge {
    /// Create a new DAP bridge
    pub fn new(session_id: String) -> Self {
        Self {
            execution_manager: None,
            initialized: AtomicBool::new(false),
            next_seq: AtomicI32::new(1),
            source_map: Arc::new(RwLock::new(HashMap::new())),
            variable_refs: Arc::new(RwLock::new(HashMap::new())),
            next_var_ref: AtomicI32::new(1000),
            _session_id: session_id,
        }
    }

    /// Connect to execution manager
    pub fn connect_execution_manager(&mut self, manager: Arc<ExecutionManager>) {
        self.execution_manager = Some(manager);
        info!("Connected execution manager to DAP bridge");
    }

    /// Check if execution manager is connected
    pub fn is_connected(&self) -> bool {
        self.execution_manager.is_some()
    }

    /// Handle generic DAP request
    ///
    /// # Errors
    ///
    /// Returns an error if the request cannot be handled
    #[instrument(level = "debug", skip(self, request))]
    #[allow(clippy::too_many_lines)]
    pub fn handle_request(&self, request: &Value) -> Result<Value> {
        // Extract command from request
        let command = request
            .get("command")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown");

        debug!("Handling DAP request: {}", command);

        // Route to appropriate handler based on command
        let response = match command {
            "initialize" => {
                let args: InitializeArguments =
                    serde_json::from_value(request.get("arguments").cloned().unwrap_or_default())?;
                let capabilities = self.handle_initialize(args)?;
                serde_json::json!({
                    "type": "response",
                    "command": "initialize",
                    "success": true,
                    "body": capabilities
                })
            }
            "launch" => {
                let args = request.get("arguments").cloned().unwrap_or_default();
                self.handle_launch(&args)?;
                serde_json::json!({
                    "type": "response",
                    "command": "launch",
                    "success": true
                })
            }
            "setBreakpoints" => {
                let args = request.get("arguments").unwrap_or(&Value::Null);
                let source: Source =
                    serde_json::from_value(args.get("source").cloned().unwrap_or_default())?;
                let breakpoints: Vec<SourceBreakpoint> =
                    serde_json::from_value(args.get("breakpoints").cloned().unwrap_or_default())?;
                let dap_breakpoints = self.handle_set_breakpoints(&source, &breakpoints)?;
                serde_json::json!({
                    "type": "response",
                    "command": "setBreakpoints",
                    "success": true,
                    "body": {
                        "breakpoints": dap_breakpoints
                    }
                })
            }
            "stackTrace" => {
                let thread_id = request
                    .get("arguments")
                    .and_then(|a| a.get("threadId"))
                    .and_then(serde_json::Value::as_i64)
                    .unwrap_or(1) as i32;
                let frames = self.handle_stack_trace(thread_id)?;
                serde_json::json!({
                    "type": "response",
                    "command": "stackTrace",
                    "success": true,
                    "body": {
                        "stackFrames": frames
                    }
                })
            }
            "scopes" => {
                let frame_id = request
                    .get("arguments")
                    .and_then(|a| a.get("frameId"))
                    .and_then(serde_json::Value::as_i64)
                    .unwrap_or(0) as i32;
                let scopes = self.handle_scopes(frame_id)?;
                serde_json::json!({
                    "type": "response",
                    "command": "scopes",
                    "success": true,
                    "body": {
                        "scopes": scopes
                    }
                })
            }
            "variables" => {
                let var_ref = request
                    .get("arguments")
                    .and_then(|a| a.get("variablesReference"))
                    .and_then(serde_json::Value::as_i64)
                    .unwrap_or(0) as i32;
                let variables = self.handle_variables(var_ref)?;
                serde_json::json!({
                    "type": "response",
                    "command": "variables",
                    "success": true,
                    "body": {
                        "variables": variables
                    }
                })
            }
            "continue" => {
                self.handle_continue()?;
                serde_json::json!({
                    "type": "response",
                    "command": "continue",
                    "success": true
                })
            }
            "next" => {
                self.handle_next()?;
                serde_json::json!({
                    "type": "response",
                    "command": "next",
                    "success": true
                })
            }
            "stepIn" => {
                self.handle_step_in()?;
                serde_json::json!({
                    "type": "response",
                    "command": "stepIn",
                    "success": true
                })
            }
            "stepOut" => {
                self.handle_step_out()?;
                serde_json::json!({
                    "type": "response",
                    "command": "stepOut",
                    "success": true
                })
            }
            "pause" => {
                self.handle_pause()?;
                serde_json::json!({
                    "type": "response",
                    "command": "pause",
                    "success": true
                })
            }
            "evaluate" => {
                let expression = request
                    .get("arguments")
                    .and_then(|a| a.get("expression"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let frame_id = request
                    .get("arguments")
                    .and_then(|a| a.get("frameId"))
                    .and_then(serde_json::Value::as_i64)
                    .map(|id| id as i32);
                let result = self.handle_evaluate(expression, &frame_id)?;
                serde_json::json!({
                    "type": "response",
                    "command": "evaluate",
                    "success": true,
                    "body": {
                        "result": result,
                        "variablesReference": 0
                    }
                })
            }
            "disconnect" => {
                self.handle_disconnect()?;
                serde_json::json!({
                    "type": "response",
                    "command": "disconnect",
                    "success": true
                })
            }
            _ => {
                warn!("Unknown DAP command: {}", command);
                serde_json::json!({
                    "type": "response",
                    "command": command,
                    "success": false,
                    "message": format!("Unknown command: {}", command)
                })
            }
        };

        Ok(response)
    }

    /// Map script to source reference
    pub fn map_script_to_source(&self, script_id: u32) -> Option<SourceReference> {
        self.source_map.read().get(&script_id).cloned()
    }

    /// Add source mapping
    pub fn add_source_mapping(&self, script_id: u32, path: String, content: Option<String>) {
        let source_ref = SourceReference {
            script_id,
            path,
            content,
        };
        self.source_map.write().insert(script_id, source_ref);
    }

    /// Handle DAP initialize request
    ///
    /// # Errors
    ///
    /// Returns an error if initialization fails
    #[instrument(level = "debug", skip(self, _args))]
    pub fn handle_initialize(&self, _args: InitializeArguments) -> Result<Capabilities> {
        self.initialized.store(true, Ordering::Relaxed);
        debug!("DAP bridge initialized");
        Ok(Capabilities::default())
    }

    /// Handle DAP launch request
    ///
    /// # Errors
    ///
    /// Returns an error if launch fails
    #[instrument(level = "debug", skip(self, args))]
    pub fn handle_launch(&self, args: &Value) -> Result<()> {
        debug!("Handling launch request: {:?}", args);
        // Extract program path and arguments
        if let Some(program) = args.get("program").and_then(|v| v.as_str()) {
            info!("Launching program: {}", program);
            // TODO: Actually launch the program
        }
        Ok(())
    }

    /// Handle DAP set breakpoints request
    ///
    /// # Errors
    ///
    /// Returns an error if execution manager is not connected
    #[instrument(level = "debug", skip(self))]
    pub fn handle_set_breakpoints(
        &self,
        source: &Source,
        breakpoints: &[SourceBreakpoint],
    ) -> Result<Vec<DapBreakpoint>> {
        let Some(ref manager) = self.execution_manager else {
            return Err(anyhow::anyhow!("Execution manager not connected"));
        };

        let source_path = source.path.clone().unwrap_or_else(|| "unknown".to_string());
        let mut dap_breakpoints = Vec::new();

        for bp in breakpoints {
            match manager.set_breakpoint(source_path.clone(), bp.line) {
                Ok(breakpoint) => {
                    dap_breakpoints.push(DapBreakpoint {
                        id: Some(breakpoint.id),
                        verified: true,
                        message: None,
                        source: Some(source.clone()),
                        line: Some(bp.line),
                        column: bp.column,
                        end_line: None,
                        end_column: None,
                    });
                }
                Err(e) => {
                    warn!("Failed to set breakpoint: {}", e);
                    dap_breakpoints.push(DapBreakpoint {
                        id: None,
                        verified: false,
                        message: Some(e.to_string()),
                        source: Some(source.clone()),
                        line: Some(bp.line),
                        column: bp.column,
                        end_line: None,
                        end_column: None,
                    });
                }
            }
        }

        Ok(dap_breakpoints)
    }

    /// Handle DAP stack trace request
    ///
    /// # Errors
    ///
    /// Returns an error if execution manager is not connected
    #[instrument(level = "debug", skip(self))]
    #[allow(clippy::used_underscore_binding)]
    pub fn handle_stack_trace(&self, _thread_id: i32) -> Result<Vec<DapStackFrame>> {
        let Some(ref manager) = self.execution_manager else {
            return Err(anyhow::anyhow!("Execution manager not connected"));
        };

        let frames = manager.get_stack_frames();
        let mut dap_frames = Vec::new();

        for (i, frame) in frames.iter().enumerate() {
            let source = self.map_script_to_source(frame.line).map(|src| Source {
                name: Some(frame.source.clone()),
                path: Some(src.path),
                source_reference: None,
                presentation_hint: None,
                origin: None,
            });

            dap_frames.push(DapStackFrame {
                id: i32::try_from(i).unwrap_or(i32::MAX),
                name: frame.name.clone(),
                source,
                line: i32::try_from(frame.line).unwrap_or(i32::MAX),
                column: i32::try_from(frame.column.unwrap_or(0)).unwrap_or(0),
                end_line: None,
                end_column: None,
                module_id: None,
                presentation_hint: None,
            });
        }

        Ok(dap_frames)
    }

    /// Handle DAP scopes request
    ///
    /// # Errors
    ///
    /// Returns an error if execution manager is not connected
    #[instrument(level = "debug", skip(self))]
    pub fn handle_scopes(&self, frame_id: i32) -> Result<Vec<Scope>> {
        let Some(ref manager) = self.execution_manager else {
            return Err(anyhow::anyhow!("Execution manager not connected"));
        };

        let mut scopes = Vec::new();

        // Local scope
        let locals = manager.get_variables(&VariableScope::Local, Some(&frame_id.to_string()));
        if !locals.is_empty() {
            let var_ref = self.next_var_ref.fetch_add(1, Ordering::Relaxed);
            self.variable_refs.write().insert(var_ref, locals);
            scopes.push(Scope {
                name: "Locals".to_string(),
                variables_reference: var_ref,
                expensive: false,
            });
        }

        // Global scope
        let globals = manager.get_variables(&VariableScope::Global, None);
        if !globals.is_empty() {
            let var_ref = self.next_var_ref.fetch_add(1, Ordering::Relaxed);
            self.variable_refs.write().insert(var_ref, globals);
            scopes.push(Scope {
                name: "Globals".to_string(),
                variables_reference: var_ref,
                expensive: false,
            });
        }

        Ok(scopes)
    }

    /// Handle DAP variables request
    ///
    /// # Errors
    ///
    /// Returns an error if variables cannot be retrieved
    #[instrument(level = "debug", skip(self))]
    pub fn handle_variables(&self, variables_reference: i32) -> Result<Vec<DapVariable>> {
        let var_refs = self.variable_refs.read();
        let Some(variables) = var_refs.get(&variables_reference) else {
            return Ok(Vec::new());
        };

        let mut dap_vars = Vec::new();
        for var in variables {
            let var_ref = if var.has_children {
                // TODO: Store child variables
                self.next_var_ref.fetch_add(1, Ordering::Relaxed)
            } else {
                0
            };

            dap_vars.push(DapVariable {
                name: var.name.clone(),
                value: var.value.clone(),
                var_type: Some(var.var_type.clone()),
                presentation_hint: None,
                evaluate_name: Some(var.name.clone()),
                variables_reference: var_ref,
                named_variables: None,
                indexed_variables: None,
            });
        }

        Ok(dap_vars)
    }

    /// Handle DAP continue request
    ///
    /// # Errors
    ///
    /// Returns an error if execution manager is not connected
    #[instrument(level = "debug", skip(self))]
    pub fn handle_continue(&self) -> Result<()> {
        let Some(ref manager) = self.execution_manager else {
            return Err(anyhow::anyhow!("Execution manager not connected"));
        };

        manager.resume(StepMode::Continue);
        Ok(())
    }

    /// Handle DAP next (step over) request
    ///
    /// # Errors
    ///
    /// Returns an error if execution manager is not connected
    #[instrument(level = "debug", skip(self))]
    pub fn handle_next(&self) -> Result<()> {
        let Some(ref manager) = self.execution_manager else {
            return Err(anyhow::anyhow!("Execution manager not connected"));
        };

        manager.resume(StepMode::StepOver);
        Ok(())
    }

    /// Handle DAP step in request
    ///
    /// # Errors
    ///
    /// Returns an error if execution manager is not connected
    #[instrument(level = "debug", skip(self))]
    pub fn handle_step_in(&self) -> Result<()> {
        let Some(ref manager) = self.execution_manager else {
            return Err(anyhow::anyhow!("Execution manager not connected"));
        };

        manager.resume(StepMode::StepIn);
        Ok(())
    }

    /// Handle DAP step out request
    ///
    /// # Errors
    ///
    /// Returns an error if execution manager is not connected
    #[instrument(level = "debug", skip(self))]
    pub fn handle_step_out(&self) -> Result<()> {
        let Some(ref manager) = self.execution_manager else {
            return Err(anyhow::anyhow!("Execution manager not connected"));
        };

        manager.resume(StepMode::StepOut);
        Ok(())
    }

    /// Handle DAP pause request
    ///
    /// # Errors
    ///
    /// Returns an error if execution manager is not connected
    #[instrument(level = "debug", skip(self))]
    pub fn handle_pause(&self) -> Result<()> {
        let Some(ref manager) = self.execution_manager else {
            return Err(anyhow::anyhow!("Execution manager not connected"));
        };

        manager.pause();
        Ok(())
    }

    /// Handle DAP evaluate request
    ///
    /// # Errors
    ///
    /// Returns an error if evaluation fails
    #[instrument(level = "debug", skip(self))]
    #[allow(clippy::used_underscore_binding)]
    pub fn handle_evaluate(&self, expression: &str, _frame_id: &Option<i32>) -> Result<String> {
        // Simplified evaluation - would integrate with actual script runtime
        debug!("Evaluating expression: {}", expression);
        Ok(format!("<evaluated: {expression}>"))
    }

    /// Handle DAP disconnect request
    ///
    /// # Errors
    ///
    /// Returns an error if disconnection fails
    #[instrument(level = "debug", skip(self))]
    pub fn handle_disconnect(&self) -> Result<()> {
        self.initialized.store(false, Ordering::Relaxed);
        info!("DAP bridge disconnected");
        Ok(())
    }

    /// Check if initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized.load(Ordering::Relaxed)
    }

    /// Get next sequence number
    pub fn next_sequence(&self) -> i32 {
        self.next_seq.fetch_add(1, Ordering::Relaxed)
    }
}

/// DAP Scope
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Scope {
    /// Scope name
    pub name: String,
    /// Variables reference
    pub variables_reference: i32,
    /// Whether scope is expensive to retrieve
    pub expensive: bool,
}

/// Language-agnostic debug adapter for Phase 18 preparation
pub trait DebugAdapter: Send + Sync {
    /// Get language name
    fn language(&self) -> &str;
    /// Initialize the adapter
    ///
    /// # Errors
    ///
    /// Returns an error if initialization fails
    fn initialize(&mut self, args: InitializeArguments) -> Result<Capabilities>;
    /// Launch a program
    ///
    /// # Errors
    ///
    /// Returns an error if launch fails
    fn launch(&mut self, args: Value) -> Result<()>;
    /// Attach to a running program
    ///
    /// # Errors
    ///
    /// Returns an error if attach fails
    fn attach(&mut self, args: Value) -> Result<()>;
    /// Set breakpoints
    ///
    /// # Errors
    ///
    /// Returns an error if breakpoint setting fails
    fn set_breakpoints(
        &self,
        source: Source,
        breakpoints: Vec<SourceBreakpoint>,
    ) -> Result<Vec<DapBreakpoint>>;
    /// Get stack trace
    ///
    /// # Errors
    ///
    /// Returns an error if stack trace retrieval fails
    fn stack_trace(&self, thread_id: i32) -> Result<Vec<DapStackFrame>>;
    /// Get scopes
    ///
    /// # Errors
    ///
    /// Returns an error if scope retrieval fails
    fn scopes(&self, frame_id: i32) -> Result<Vec<Scope>>;
    /// Get variables
    ///
    /// # Errors
    ///
    /// Returns an error if variable retrieval fails
    fn variables(&self, reference: i32) -> Result<Vec<DapVariable>>;
    /// Continue execution
    ///
    /// # Errors
    ///
    /// Returns an error if continuation fails
    fn continue_execution(&self) -> Result<()>;
    /// Step over
    ///
    /// # Errors
    ///
    /// Returns an error if step over fails
    fn next(&self) -> Result<()>;
    /// Step in
    ///
    /// # Errors
    ///
    /// Returns an error if step in fails
    fn step_in(&self) -> Result<()>;
    /// Step out
    ///
    /// # Errors
    ///
    /// Returns an error if step out fails
    fn step_out(&self) -> Result<()>;
    /// Pause execution
    ///
    /// # Errors
    ///
    /// Returns an error if pause fails
    fn pause(&self) -> Result<()>;
    /// Evaluate expression
    ///
    /// # Errors
    ///
    /// Returns an error if evaluation fails
    fn evaluate(&self, expression: String, frame_id: Option<i32>) -> Result<String>;
    /// Disconnect
    ///
    /// # Errors
    ///
    /// Returns an error if disconnection fails
    fn disconnect(&self) -> Result<()>;
}

/// Lua debug adapter
pub struct LuaDebugAdapter {
    bridge: DAPBridge,
}

impl LuaDebugAdapter {
    /// Create a new Lua debug adapter
    pub fn new(session_id: String) -> Self {
        Self {
            bridge: DAPBridge::new(session_id),
        }
    }

    /// Connect to execution manager
    pub fn connect_execution_manager(&mut self, manager: Arc<ExecutionManager>) {
        self.bridge.connect_execution_manager(manager);
    }
}

impl DebugAdapter for LuaDebugAdapter {
    fn language(&self) -> &'static str {
        "lua"
    }

    fn initialize(&mut self, args: InitializeArguments) -> Result<Capabilities> {
        self.bridge.handle_initialize(args)
    }

    fn launch(&mut self, args: Value) -> Result<()> {
        self.bridge.handle_launch(&args)
    }

    fn attach(&mut self, _args: Value) -> Result<()> {
        Err(anyhow::anyhow!("Attach not supported for Lua"))
    }

    fn set_breakpoints(
        &self,
        source: Source,
        breakpoints: Vec<SourceBreakpoint>,
    ) -> Result<Vec<DapBreakpoint>> {
        self.bridge.handle_set_breakpoints(&source, &breakpoints)
    }

    fn stack_trace(&self, thread_id: i32) -> Result<Vec<DapStackFrame>> {
        self.bridge.handle_stack_trace(thread_id)
    }

    fn scopes(&self, frame_id: i32) -> Result<Vec<Scope>> {
        self.bridge.handle_scopes(frame_id)
    }

    fn variables(&self, reference: i32) -> Result<Vec<DapVariable>> {
        self.bridge.handle_variables(reference)
    }

    fn continue_execution(&self) -> Result<()> {
        self.bridge.handle_continue()
    }

    fn next(&self) -> Result<()> {
        self.bridge.handle_next()
    }

    fn step_in(&self) -> Result<()> {
        self.bridge.handle_step_in()
    }

    fn step_out(&self) -> Result<()> {
        self.bridge.handle_step_out()
    }

    fn pause(&self) -> Result<()> {
        self.bridge.handle_pause()
    }

    fn evaluate(&self, expression: String, frame_id: Option<i32>) -> Result<String> {
        self.bridge.handle_evaluate(&expression, &frame_id)
    }

    fn disconnect(&self) -> Result<()> {
        self.bridge.handle_disconnect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dap_bridge_creation() {
        let bridge = DAPBridge::new("test-session".to_string());
        assert!(!bridge.is_initialized());
    }

    #[test]
    fn test_dap_initialize() {
        let bridge = DAPBridge::new("test-session".to_string());
        let args = InitializeArguments {
            adapter_id: "llmspell".to_string(),
            locale: "en".to_string(),
            lines_start_at1: true,
            columns_start_at1: true,
            path_format: "path".to_string(),
            supports_variable_type: true,
            supports_variable_paging: false,
            supports_run_in_terminal_request: false,
        };

        let capabilities = bridge.handle_initialize(args).unwrap();
        assert!(bridge.is_initialized());
        assert!(capabilities.supports_conditional_breakpoints);
        assert!(capabilities.supports_configuration_done_request);
    }

    #[test]
    fn test_source_mapping() {
        let bridge = DAPBridge::new("test-session".to_string());

        bridge.add_source_mapping(1, "test.lua".to_string(), None);

        let source_ref = bridge.map_script_to_source(1);
        assert!(source_ref.is_some());
        assert_eq!(source_ref.unwrap().path, "test.lua");
    }

    #[test]
    fn test_lua_debug_adapter() {
        let mut adapter = LuaDebugAdapter::new("test-session".to_string());

        assert_eq!(adapter.language(), "lua");

        let args = InitializeArguments {
            adapter_id: "lua".to_string(),
            locale: "en".to_string(),
            lines_start_at1: true,
            columns_start_at1: true,
            path_format: "path".to_string(),
            supports_variable_type: true,
            supports_variable_paging: false,
            supports_run_in_terminal_request: false,
        };

        let capabilities = adapter.initialize(args).unwrap();
        assert!(capabilities.supports_conditional_breakpoints);
    }

    #[test]
    fn test_sequence_numbers() {
        let bridge = DAPBridge::new("test-session".to_string());

        let seq1 = bridge.next_sequence();
        let seq2 = bridge.next_sequence();
        let seq3 = bridge.next_sequence();

        assert_eq!(seq1, 1);
        assert_eq!(seq2, 2);
        assert_eq!(seq3, 3);
    }
}
