//! Protocol adapter for `ExecutionManager`
//!
//! Wraps the existing `ExecutionManager` to implement the `DebugCapability` trait,
//! enabling protocol-based access to execution and breakpoint management.

use crate::execution_bridge::{
    Breakpoint, DebugCommand, DebugState as BridgeDebugState, ExecutionManager,
    PauseReason as BridgePauseReason, StackFrame,
};
use async_trait::async_trait;
use llmspell_core::{
    debug::{
        BreakpointInfo, DebugCapability, DebugRequest, DebugResponse, DebugState, LocationInfo,
        PauseReason, StackFrameInfo, StepType,
    },
    Result,
};
use std::sync::Arc;

/// Adapter that wraps `ExecutionManager` to provide `DebugCapability`
pub struct ExecutionManagerAdapter {
    execution_manager: Arc<ExecutionManager>,
    session_id: String,
}

impl ExecutionManagerAdapter {
    /// Create a new adapter wrapping an `ExecutionManager`
    #[must_use]
    pub const fn new(execution_manager: Arc<ExecutionManager>, session_id: String) -> Self {
        Self {
            execution_manager,
            session_id,
        }
    }

    /// Convert bridge `DebugState` to protocol `DebugState`
    fn convert_debug_state(state: &BridgeDebugState) -> DebugState {
        match state {
            BridgeDebugState::Running => DebugState::Running,
            BridgeDebugState::Terminated => DebugState::Terminated { exit_code: None },
            BridgeDebugState::Paused { reason, location } => DebugState::Paused {
                reason: Self::convert_pause_reason(reason),
                location: Some(LocationInfo {
                    source: location.source.clone(),
                    line: location.line,
                    column: location.column,
                    function: None,
                }),
            },
        }
    }

    /// Convert bridge `PauseReason` to protocol `PauseReason`
    fn convert_pause_reason(reason: &BridgePauseReason) -> PauseReason {
        match reason {
            BridgePauseReason::Breakpoint => PauseReason::Breakpoint {
                id: "unknown".to_string(),
            },
            BridgePauseReason::Step => PauseReason::Step,
            BridgePauseReason::Pause => PauseReason::PauseRequest,
            BridgePauseReason::Exception(msg) => PauseReason::Exception {
                message: msg.clone(),
            },
            BridgePauseReason::Entry => PauseReason::Entry,
        }
    }

    /// Convert `StackFrame` to `StackFrameInfo`
    fn convert_stack_frame(frame: &StackFrame, index: usize) -> StackFrameInfo {
        StackFrameInfo {
            index,
            name: frame.name.clone(),
            location: LocationInfo {
                source: frame.source.clone(),
                line: frame.line,
                column: frame.column,
                function: Some(frame.name.clone()),
            },
            locals: frame.locals.iter().map(|v| v.name.clone()).collect(),
            is_user_code: frame.is_user_code,
        }
    }

    /// Convert `Breakpoint` to `BreakpointInfo`
    fn convert_breakpoint(bp: &Breakpoint) -> BreakpointInfo {
        BreakpointInfo {
            id: bp.id.clone(),
            source: bp.source.clone(),
            line: bp.line,
            condition: bp.condition.clone(),
            verified: bp.enabled,
            hit_count: bp.current_hits,
        }
    }

    /// Convert `StepType` to `DebugCommand`
    const fn step_type_to_command(step_type: StepType) -> DebugCommand {
        match step_type {
            StepType::StepIn => DebugCommand::StepInto,
            StepType::StepOver => DebugCommand::StepOver,
            StepType::StepOut => DebugCommand::StepOut,
        }
    }
}

#[async_trait]
impl DebugCapability for ExecutionManagerAdapter {
    async fn process_debug_request(&self, request: DebugRequest) -> Result<DebugResponse> {
        match request {
            DebugRequest::CreateSession { .. } => {
                // Session is created when adapter is instantiated
                Ok(DebugResponse::SessionCreated {
                    session_id: self.session_id.clone(),
                    capabilities: self.capabilities(),
                })
            }

            DebugRequest::SetBreakpoints {
                source,
                breakpoints,
            } => {
                let mut bp_infos = Vec::new();
                for (line, condition) in breakpoints {
                    let mut bp = Breakpoint::new(source.clone(), line);
                    if let Some(cond) = condition {
                        bp = bp.with_condition(cond);
                    }
                    let id = self.execution_manager.add_breakpoint(bp.clone()).await;
                    bp.id = id; // Update with assigned ID
                    bp_infos.push(Self::convert_breakpoint(&bp));
                }
                Ok(DebugResponse::BreakpointsSet {
                    breakpoints: bp_infos,
                })
            }

            DebugRequest::RemoveBreakpoints { ids } => {
                let mut count = 0;
                for id in ids {
                    if self.execution_manager.remove_breakpoint(&id).await {
                        count += 1;
                    }
                }
                Ok(DebugResponse::BreakpointsRemoved { count })
            }

            DebugRequest::Step { step_type } => {
                let command = Self::step_type_to_command(step_type);
                self.execution_manager.send_command(command).await;
                let state = self.execution_manager.get_state().await;
                Ok(DebugResponse::ExecutionState(Self::convert_debug_state(
                    &state,
                )))
            }

            DebugRequest::Continue => {
                self.execution_manager
                    .send_command(DebugCommand::Continue)
                    .await;
                let state = self.execution_manager.get_state().await;
                Ok(DebugResponse::ExecutionState(Self::convert_debug_state(
                    &state,
                )))
            }

            DebugRequest::Pause => {
                self.execution_manager
                    .send_command(DebugCommand::Pause)
                    .await;
                let state = self.execution_manager.get_state().await;
                Ok(DebugResponse::ExecutionState(Self::convert_debug_state(
                    &state,
                )))
            }

            DebugRequest::GetDebugState => {
                let state = self.execution_manager.get_state().await;
                Ok(DebugResponse::DebugStateInfo(Self::convert_debug_state(
                    &state,
                )))
            }

            DebugRequest::GetStackTrace => {
                let frames = self.execution_manager.get_stack_trace().await;
                let frame_infos: Vec<StackFrameInfo> = frames
                    .iter()
                    .enumerate()
                    .map(|(i, f)| Self::convert_stack_frame(f, i))
                    .collect();
                Ok(DebugResponse::StackTrace(frame_infos))
            }

            DebugRequest::Terminate { .. } => {
                self.execution_manager
                    .send_command(DebugCommand::Terminate)
                    .await;
                Ok(DebugResponse::SessionTerminated)
            }

            _ => Ok(DebugResponse::Error {
                message: "Request should be handled by different capability".to_string(),
                details: Some(format!("Request type: {request:?}")),
            }),
        }
    }

    fn capabilities(&self) -> Vec<String> {
        vec![
            "breakpoints".to_string(),
            "stepping".to_string(),
            "pause".to_string(),
            "continue".to_string(),
            "terminate".to_string(),
            "stack_trace".to_string(),
            "execution_state".to_string(),
        ]
    }

    fn name(&self) -> &'static str {
        "execution_manager"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::debug_state_cache::SharedDebugStateCache;

    #[tokio::test]
    async fn test_execution_manager_adapter() {
        let cache = Arc::new(SharedDebugStateCache::new());
        let exec_mgr = Arc::new(ExecutionManager::new(cache));
        let session_id = "test_session".to_string();
        let adapter = ExecutionManagerAdapter::new(exec_mgr, session_id);

        // Test capabilities
        assert!(adapter.capabilities().contains(&"breakpoints".to_string()));
        assert_eq!(adapter.name(), "execution_manager");

        // Test session creation
        let response = adapter
            .process_debug_request(DebugRequest::CreateSession {
                script: "test.lua".to_string(),
                args: vec![],
            })
            .await
            .unwrap();

        if let DebugResponse::SessionCreated { session_id, .. } = response {
            assert!(!session_id.is_empty());
        } else {
            panic!("Expected SessionCreated response");
        }
    }
}
