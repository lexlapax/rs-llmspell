//! DAP (Debug Adapter Protocol) Bridge
//! 
//! Provides a bridge between the Debug Adapter Protocol and the ExecutionManager,
//! enabling IDE debugging support and fixing the .locals command.

use anyhow::Result;
use llmspell_bridge::execution_bridge::{
    Breakpoint, DebugCommand, ExecutionManager,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// DAP Initialize request arguments
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeArguments {
    #[serde(default)]
    pub adapter_id: String,
    #[serde(default)]
    pub locale: String,
    #[serde(default)]
    pub lines_start_at1: bool,
    #[serde(default)]
    pub columns_start_at1: bool,
    #[serde(default)]
    pub path_format: String,
    #[serde(default)]
    pub supports_variable_type: bool,
    #[serde(default)]
    pub supports_variable_paging: bool,
    #[serde(default)]
    pub supports_run_in_terminal_request: bool,
}

/// DAP Initialize response capabilities
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Capabilities {
    pub supports_configuration_done_request: bool,
    pub supports_function_breakpoints: bool,
    pub supports_conditional_breakpoints: bool,
    pub supports_evaluate_for_hovers: bool,
    pub supports_step_back: bool,
    pub supports_set_variable: bool,
    pub supports_restart_frame: bool,
    pub supports_exception_info_request: bool,
    pub supports_set_expression: bool,
    pub supports_terminate_request: bool,
    pub supports_delayed_stack_trace_loading: bool,
    pub supports_loaded_sources_request: bool,
    pub supports_log_points: bool,
    pub supports_terminate_threads_request: bool,
    pub supports_value_formatting_options: bool,
    pub supports_exception_conditions: bool,
    pub supports_clipboard_context: bool,
    pub supports_stepping_granularity: bool,
    pub supports_instruction_breakpoints: bool,
    pub supports_exception_filter_options: bool,
}

/// DAP SetBreakpoints request arguments
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetBreakpointsArguments {
    pub source: Source,
    #[serde(default)]
    pub breakpoints: Vec<SourceBreakpoint>,
    #[serde(default)]
    pub lines: Vec<u32>,
    #[serde(default)]
    pub source_modified: bool,
}

/// DAP Source
#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Source {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_reference: Option<i32>,
}

/// DAP SourceBreakpoint
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceBreakpoint {
    pub line: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub condition: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hit_condition: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_message: Option<String>,
}

/// DAP Breakpoint response
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DapBreakpoint {
    pub id: Option<String>,
    pub verified: bool,
    pub message: Option<String>,
    pub source: Option<Source>,
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub end_line: Option<u32>,
    pub end_column: Option<u32>,
}

/// DAP StackTrace request arguments
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StackTraceArguments {
    pub thread_id: i32,
    #[serde(default)]
    pub start_frame: i32,
    #[serde(default)]
    pub levels: i32,
    #[serde(default)]
    pub format: Option<StackFrameFormat>,
}

/// DAP StackFrame format
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StackFrameFormat {
    #[serde(default)]
    pub parameters: bool,
    #[serde(default)]
    pub parameter_types: bool,
    #[serde(default)]
    pub parameter_names: bool,
    #[serde(default)]
    pub parameter_values: bool,
    #[serde(default)]
    pub line: bool,
    #[serde(default)]
    pub module: bool,
}

/// DAP StackFrame response
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DapStackFrame {
    pub id: i32,
    pub name: String,
    pub source: Option<Source>,
    pub line: u32,
    pub column: u32,
    pub end_line: Option<u32>,
    pub end_column: Option<u32>,
    pub can_restart: bool,
    pub instruction_pointer_reference: Option<String>,
    pub module_id: Option<String>,
    pub presentation_hint: Option<String>,
}

/// DAP Variables request arguments
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VariablesArguments {
    pub variables_reference: i32,
    #[serde(default)]
    pub filter: Option<String>,
    #[serde(default)]
    pub start: Option<i32>,
    #[serde(default)]
    pub count: Option<i32>,
    #[serde(default)]
    pub format: Option<ValueFormat>,
}

/// DAP Value format
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValueFormat {
    #[serde(default)]
    pub hex: bool,
}

/// DAP Variable response
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DapVariable {
    pub name: String,
    pub value: String,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub var_type: Option<String>,
    pub presentation_hint: Option<VariablePresentationHint>,
    pub evaluate_name: Option<String>,
    pub variables_reference: i32,
    pub named_variables: Option<i32>,
    pub indexed_variables: Option<i32>,
    pub memory_reference: Option<String>,
}

/// DAP Variable presentation hint
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VariablePresentationHint {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<String>,
}

/// DAP Request
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub seq: i32,
    #[serde(rename = "type")]
    pub request_type: String,
    pub command: String,
    #[serde(default)]
    pub arguments: Value,
}

/// DAP Response
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub seq: i32,
    #[serde(rename = "type")]
    pub response_type: String,
    pub request_seq: i32,
    pub success: bool,
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<Value>,
}

/// DAP Bridge for connecting ExecutionManager to DAP protocol
pub struct DAPBridge {
    execution_manager: Arc<ExecutionManager>,
    initialized: AtomicBool,
    next_seq: std::sync::atomic::AtomicI32,
}

impl DAPBridge {
    /// Create a new DAP bridge
    pub fn new(execution_manager: Arc<ExecutionManager>) -> Self {
        Self {
            execution_manager,
            initialized: AtomicBool::new(false),
            next_seq: std::sync::atomic::AtomicI32::new(1),
        }
    }

    /// Handle a DAP request
    pub async fn handle_request(&self, request: Value) -> Result<Value> {
        let dap_req: Request = serde_json::from_value(request)?;
        
        let response = match dap_req.command.as_str() {
            "initialize" => self.handle_initialize(dap_req).await,
            "setBreakpoints" => self.handle_set_breakpoints(dap_req).await,
            "stackTrace" => self.handle_stack_trace(dap_req).await,
            "variables" => self.handle_variables(dap_req).await,
            "continue" => self.handle_continue(dap_req).await,
            "next" => self.handle_next(dap_req).await,
            "stepIn" => self.handle_step_in(dap_req).await,
            "stepOut" => self.handle_step_out(dap_req).await,
            "pause" => self.handle_pause(dap_req).await,
            "terminate" => self.handle_terminate(dap_req).await,
            _ => self.handle_unsupported(dap_req),
        }?;
        
        Ok(serde_json::to_value(response)?)
    }

    /// Handle initialize request
    async fn handle_initialize(&self, req: Request) -> Result<Response> {
        self.initialized.store(true, Ordering::Relaxed);
        
        let capabilities = Capabilities {
            supports_configuration_done_request: false,
            supports_function_breakpoints: false,
            supports_conditional_breakpoints: false,
            supports_evaluate_for_hovers: true,
            supports_step_back: false,
            supports_set_variable: false,
            supports_restart_frame: false,
            supports_exception_info_request: false,
            supports_set_expression: false,
            supports_terminate_request: true,
            supports_delayed_stack_trace_loading: false,
            supports_loaded_sources_request: false,
            supports_log_points: false,
            supports_terminate_threads_request: false,
            supports_value_formatting_options: false,
            supports_exception_conditions: false,
            supports_clipboard_context: false,
            supports_stepping_granularity: false,
            supports_instruction_breakpoints: false,
            supports_exception_filter_options: false,
        };

        Ok(Response {
            seq: self.next_seq.fetch_add(1, Ordering::Relaxed),
            response_type: "response".to_string(),
            request_seq: req.seq,
            success: true,
            command: req.command,
            message: None,
            body: Some(serde_json::to_value(capabilities)?),
        })
    }

    /// Handle setBreakpoints request
    async fn handle_set_breakpoints(&self, req: Request) -> Result<Response> {
        let args: SetBreakpointsArguments = serde_json::from_value(req.arguments)?;
        let source_path = args.source.path.unwrap_or_else(|| "unknown".to_string());
        
        let mut dap_breakpoints = Vec::new();
        
        // Clear existing breakpoints for this source
        let existing = self.execution_manager.get_breakpoints().await;
        for bp in existing {
            if bp.source == source_path {
                self.execution_manager.remove_breakpoint(&bp.id).await;
            }
        }
        
        // Add new breakpoints
        for bp in &args.breakpoints {
            let breakpoint = Breakpoint::new(source_path.clone(), bp.line);
            let id = self.execution_manager.add_breakpoint(breakpoint).await;
            
            dap_breakpoints.push(DapBreakpoint {
                id: Some(id),
                verified: true,
                message: None,
                source: Some(Source {
                    name: args.source.name.clone(),
                    path: Some(source_path.clone()),
                    source_reference: None,
                }),
                line: Some(bp.line),
                column: bp.column,
                end_line: None,
                end_column: None,
            });
        }

        Ok(Response {
            seq: self.next_seq.fetch_add(1, Ordering::Relaxed),
            response_type: "response".to_string(),
            request_seq: req.seq,
            success: true,
            command: req.command,
            message: None,
            body: Some(serde_json::json!({
                "breakpoints": dap_breakpoints
            })),
        })
    }

    /// Handle stackTrace request
    async fn handle_stack_trace(&self, req: Request) -> Result<Response> {
        let stack = self.execution_manager.get_stack_trace().await;
        
        let dap_frames: Vec<DapStackFrame> = stack
            .iter()
            .enumerate()
            .map(|(i, frame)| DapStackFrame {
                id: i as i32,
                name: frame.name.clone(),
                source: Some(Source {
                    name: Some(frame.source.clone()),
                    path: Some(frame.source.clone()),
                    source_reference: None,
                }),
                line: frame.line,
                column: frame.column.unwrap_or(0),
                end_line: None,
                end_column: None,
                can_restart: false,
                instruction_pointer_reference: None,
                module_id: None,
                presentation_hint: if frame.is_user_code {
                    Some("normal".to_string())
                } else {
                    Some("subtle".to_string())
                },
            })
            .collect();

        Ok(Response {
            seq: self.next_seq.fetch_add(1, Ordering::Relaxed),
            response_type: "response".to_string(),
            request_seq: req.seq,
            success: true,
            command: req.command,
            message: None,
            body: Some(serde_json::json!({
                "stackFrames": dap_frames,
                "totalFrames": dap_frames.len()
            })),
        })
    }

    /// Handle variables request - THIS FIXES .locals COMMAND!
    async fn handle_variables(&self, req: Request) -> Result<Response> {
        let args: VariablesArguments = serde_json::from_value(req.arguments)?;
        
        // Variables reference convention:
        // 1000 + frame_id = locals for that frame
        // 2000 + frame_id = globals for that frame
        let frame_id = if args.variables_reference >= 1000 && args.variables_reference < 2000 {
            // Local variables
            (args.variables_reference - 1000) as usize
        } else if args.variables_reference >= 2000 {
            // Global variables
            (args.variables_reference - 2000) as usize
        } else {
            0
        };
        
        // Get variables from ExecutionManager
        let stack = self.execution_manager.get_stack_trace().await;
        let frame_id_str = if frame_id < stack.len() {
            stack[frame_id].id.clone()
        } else {
            "current".to_string()
        };
        
        let vars = self.execution_manager.get_variables(Some(&frame_id_str)).await;
        
        let dap_vars: Vec<DapVariable> = vars
            .iter()
            .map(|var| DapVariable {
                name: var.name.clone(),
                value: var.value.clone(),
                var_type: Some(var.var_type.clone()),
                presentation_hint: None,
                evaluate_name: Some(var.name.clone()),
                variables_reference: if var.has_children {
                    // Would need to implement nested variable support
                    0
                } else {
                    0
                },
                named_variables: None,
                indexed_variables: None,
                memory_reference: None,
            })
            .collect();

        Ok(Response {
            seq: self.next_seq.fetch_add(1, Ordering::Relaxed),
            response_type: "response".to_string(),
            request_seq: req.seq,
            success: true,
            command: req.command,
            message: None,
            body: Some(serde_json::json!({
                "variables": dap_vars
            })),
        })
    }

    /// Handle continue request
    async fn handle_continue(&self, req: Request) -> Result<Response> {
        self.execution_manager.send_command(DebugCommand::Continue).await;
        
        Ok(Response {
            seq: self.next_seq.fetch_add(1, Ordering::Relaxed),
            response_type: "response".to_string(),
            request_seq: req.seq,
            success: true,
            command: req.command,
            message: None,
            body: Some(serde_json::json!({
                "allThreadsContinued": true
            })),
        })
    }

    /// Handle next (step over) request
    async fn handle_next(&self, req: Request) -> Result<Response> {
        self.execution_manager.send_command(DebugCommand::StepOver).await;
        
        Ok(Response {
            seq: self.next_seq.fetch_add(1, Ordering::Relaxed),
            response_type: "response".to_string(),
            request_seq: req.seq,
            success: true,
            command: req.command,
            message: None,
            body: None,
        })
    }

    /// Handle stepIn request
    async fn handle_step_in(&self, req: Request) -> Result<Response> {
        self.execution_manager.send_command(DebugCommand::StepInto).await;
        
        Ok(Response {
            seq: self.next_seq.fetch_add(1, Ordering::Relaxed),
            response_type: "response".to_string(),
            request_seq: req.seq,
            success: true,
            command: req.command,
            message: None,
            body: None,
        })
    }

    /// Handle stepOut request
    async fn handle_step_out(&self, req: Request) -> Result<Response> {
        self.execution_manager.send_command(DebugCommand::StepOut).await;
        
        Ok(Response {
            seq: self.next_seq.fetch_add(1, Ordering::Relaxed),
            response_type: "response".to_string(),
            request_seq: req.seq,
            success: true,
            command: req.command,
            message: None,
            body: None,
        })
    }

    /// Handle pause request
    async fn handle_pause(&self, req: Request) -> Result<Response> {
        self.execution_manager.send_command(DebugCommand::Pause).await;
        
        Ok(Response {
            seq: self.next_seq.fetch_add(1, Ordering::Relaxed),
            response_type: "response".to_string(),
            request_seq: req.seq,
            success: true,
            command: req.command,
            message: None,
            body: None,
        })
    }

    /// Handle terminate request
    async fn handle_terminate(&self, req: Request) -> Result<Response> {
        self.execution_manager.send_command(DebugCommand::Terminate).await;
        
        Ok(Response {
            seq: self.next_seq.fetch_add(1, Ordering::Relaxed),
            response_type: "response".to_string(),
            request_seq: req.seq,
            success: true,
            command: req.command,
            message: None,
            body: None,
        })
    }

    /// Handle unsupported command
    fn handle_unsupported(&self, req: Request) -> Result<Response> {
        Ok(Response {
            seq: self.next_seq.fetch_add(1, Ordering::Relaxed),
            response_type: "response".to_string(),
            request_seq: req.seq,
            success: false,
            command: req.command.clone(),
            message: Some(format!("Unsupported command: {}", req.command)),
            body: None,
        })
    }
}