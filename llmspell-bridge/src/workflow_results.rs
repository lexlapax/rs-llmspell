//! ABOUTME: Workflow result transformation and formatting
//! ABOUTME: Handles conversion of workflow execution results for script consumption

use llmspell_core::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Unified workflow result format for scripts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptWorkflowResult {
    /// Whether the workflow completed successfully
    pub success: bool,
    /// Workflow type that was executed
    pub workflow_type: String,
    /// Workflow name or ID
    pub workflow_name: String,
    /// Execution duration in milliseconds
    pub duration_ms: u64,
    /// Result data specific to workflow type
    pub data: Value,
    /// Error information if failed
    pub error: Option<WorkflowError>,
    /// Execution metadata
    pub metadata: WorkflowMetadata,
}

/// Workflow error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowError {
    /// Error type/category
    pub error_type: String,
    /// Human-readable error message
    pub message: String,
    /// Step or branch where error occurred
    pub location: Option<String>,
    /// Additional error details
    pub details: Option<Value>,
}

/// Workflow execution metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowMetadata {
    /// Start time of execution
    pub start_time: String,
    /// End time of execution
    pub end_time: String,
    /// Number of steps executed
    pub steps_executed: Option<usize>,
    /// Number of steps that succeeded
    pub steps_succeeded: Option<usize>,
    /// Number of steps that failed
    pub steps_failed: Option<usize>,
    /// Additional workflow-specific metadata
    pub extra: Option<Value>,
}

/// Transform sequential workflow result
pub fn transform_sequential_result(
    result: &llmspell_workflows::SequentialWorkflowResult,
) -> ScriptWorkflowResult {
    let metadata = WorkflowMetadata {
        start_time: chrono::Utc::now().to_rfc3339(), // Would be better from actual result
        end_time: chrono::Utc::now().to_rfc3339(),
        steps_executed: Some(result.successful_steps.len() + result.failed_steps.len()),
        steps_succeeded: Some(result.successful_steps.len()),
        steps_failed: Some(result.failed_steps.len()),
        extra: Some(serde_json::json!({
            "step_results": result.successful_steps.iter().chain(result.failed_steps.iter()).map(|sr| {
                serde_json::json!({
                    "step_name": sr.step_name,
                    "success": sr.success,
                    "duration_ms": sr.duration.as_millis(),
                    "output": sr.output,
                    "error": sr.error,
                })
            }).collect::<Vec<_>>(),
        })),
    };

    let error = if !result.success {
        result.error_message.as_ref().map(|e| WorkflowError {
            error_type: "SequentialExecutionError".to_string(),
            message: e.clone(),
            location: result.failed_steps.first().map(|sr| sr.step_name.clone()),
            details: None,
        })
    } else {
        None
    };

    ScriptWorkflowResult {
        success: result.success,
        workflow_type: "sequential".to_string(),
        workflow_name: result.workflow_name.clone(),
        duration_ms: result.duration.as_millis() as u64,
        data: serde_json::json!({
            "steps_executed": result.successful_steps.len() + result.failed_steps.len(),
            "successful_steps": result.successful_steps.len(),
            "failed_steps": result.failed_steps.len(),
            "final_output": result.successful_steps.last()
                .map(|sr| sr.output.clone())
                .unwrap_or_default(),
        }),
        error,
        metadata,
    }
}

/// Transform conditional workflow result
pub fn transform_conditional_result(
    result: &llmspell_workflows::ConditionalWorkflowResult,
) -> ScriptWorkflowResult {
    let metadata = WorkflowMetadata {
        start_time: chrono::Utc::now().to_rfc3339(),
        end_time: chrono::Utc::now().to_rfc3339(),
        steps_executed: None,
        steps_succeeded: None,
        steps_failed: None,
        extra: Some(serde_json::json!({
            "executed_branches": result.executed_branches.iter().map(|br| {
                serde_json::json!({
                    "branch_name": br.branch_name,
                    "success": br.success,
                    "steps_executed": br.step_results.len(),
                    "duration_ms": br.duration.as_millis(),
                })
            }).collect::<Vec<_>>(),
            "matched_branches": result.matched_branches,
            "total_branches": result.total_branches,
        })),
    };

    let error = if !result.success {
        result.error_message.as_ref().map(|e| WorkflowError {
            error_type: "ConditionalExecutionError".to_string(),
            message: e.clone(),
            location: result
                .executed_branches
                .first()
                .map(|br| br.branch_name.clone()),
            details: None,
        })
    } else {
        None
    };

    ScriptWorkflowResult {
        success: result.success,
        workflow_type: "conditional".to_string(),
        workflow_name: result.workflow_name.clone(),
        duration_ms: result.duration.as_millis() as u64,
        data: serde_json::json!({
            "executed_branches": result.executed_branches.len(),
            "matched_branches": result.matched_branches,
            "total_branches": result.total_branches,
            "branch_output": result.executed_branches.last()
                .and_then(|br| br.step_results.last())
                .map(|sr| sr.output.clone())
                .unwrap_or_default(),
        }),
        error,
        metadata,
    }
}

/// Transform loop workflow result
pub fn transform_loop_result(
    result: &llmspell_workflows::LoopWorkflowResult,
) -> ScriptWorkflowResult {
    let metadata = WorkflowMetadata {
        start_time: chrono::Utc::now().to_rfc3339(),
        end_time: chrono::Utc::now().to_rfc3339(),
        steps_executed: Some(result.completed_iterations),
        steps_succeeded: None,
        steps_failed: None,
        extra: Some(serde_json::json!({
            "total_iterations": result.total_iterations,
            "completed_iterations": result.completed_iterations,
            "break_reason": result.break_reason,
        })),
    };

    let error = if !result.success {
        result.error.as_ref().map(|e| WorkflowError {
            error_type: "LoopExecutionError".to_string(),
            message: e.clone(),
            location: Some(format!("iteration {}", result.completed_iterations)),
            details: None,
        })
    } else {
        None
    };

    ScriptWorkflowResult {
        success: result.success,
        workflow_type: "loop".to_string(),
        workflow_name: result.workflow_name.clone(),
        duration_ms: result.duration.as_millis() as u64,
        data: serde_json::json!({
            "total_iterations": result.total_iterations,
            "completed_iterations": result.completed_iterations,
            "aggregated_results": result.aggregated_results,
            "break_reason": result.break_reason,
        }),
        error,
        metadata,
    }
}

/// Transform parallel workflow result
pub fn transform_parallel_result(
    result: &llmspell_workflows::ParallelWorkflowResult,
) -> ScriptWorkflowResult {
    let metadata = WorkflowMetadata {
        start_time: chrono::Utc::now().to_rfc3339(),
        end_time: chrono::Utc::now().to_rfc3339(),
        steps_executed: Some(
            result
                .branch_results
                .iter()
                .map(|br| br.step_results.len())
                .sum(),
        ),
        steps_succeeded: None,
        steps_failed: None,
        extra: Some(serde_json::json!({
            "total_branches": result.branch_results.len(),
            "successful_branches": result.successful_branches,
            "failed_branches": result.failed_branches,
            "stopped_early": result.stopped_early,
            "branch_details": result.branch_results.iter().map(|br| {
                serde_json::json!({
                    "branch_name": br.branch_name,
                    "success": br.success,
                    "required": br.required,
                    "duration_ms": br.duration.as_millis(),
                    "error": br.error,
                })
            }).collect::<Vec<_>>(),
        })),
    };

    let error = if !result.success {
        result.error.as_ref().map(|e| WorkflowError {
            error_type: "ParallelExecutionError".to_string(),
            message: e.clone(),
            location: result
                .branch_results
                .iter()
                .find(|br| !br.success && br.required)
                .map(|br| br.branch_name.clone()),
            details: Some(serde_json::json!({
                "failed_branches": result.branch_results.iter()
                    .filter(|br| !br.success)
                    .map(|br| br.branch_name.clone())
                    .collect::<Vec<_>>(),
            })),
        })
    } else {
        None
    };

    // Collect all branch outputs
    let branch_outputs: serde_json::Map<String, Value> = result
        .branch_results
        .iter()
        .filter_map(|br| {
            br.step_results.last().map(|sr| {
                (
                    br.branch_name.clone(),
                    serde_json::Value::String(sr.output.clone()),
                )
            })
        })
        .collect();

    ScriptWorkflowResult {
        success: result.success,
        workflow_type: "parallel".to_string(),
        workflow_name: result.workflow_name.clone(),
        duration_ms: result.duration.as_millis() as u64,
        data: serde_json::json!({
            "successful_branches": result.successful_branches,
            "failed_branches": result.failed_branches,
            "branch_outputs": branch_outputs,
        }),
        error,
        metadata,
    }
}

/// Transform generic workflow result from JSON
pub fn transform_generic_result(
    workflow_type: &str,
    workflow_name: &str,
    result: Value,
) -> Result<ScriptWorkflowResult> {
    // Try to extract common fields
    let success = result
        .get("success")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let duration_ms = result
        .get("duration")
        .and_then(|v| v.as_u64())
        .or_else(|| result.get("duration_ms").and_then(|v| v.as_u64()))
        .unwrap_or(0);

    let error = if !success {
        result
            .get("error")
            .and_then(|v| v.as_str())
            .map(|e| WorkflowError {
                error_type: format!("{}Error", workflow_type),
                message: e.to_string(),
                location: None,
                details: None,
            })
    } else {
        None
    };

    let metadata = WorkflowMetadata {
        start_time: chrono::Utc::now().to_rfc3339(),
        end_time: chrono::Utc::now().to_rfc3339(),
        steps_executed: None,
        steps_succeeded: None,
        steps_failed: None,
        extra: Some(result.clone()),
    };

    Ok(ScriptWorkflowResult {
        success,
        workflow_type: workflow_type.to_string(),
        workflow_name: workflow_name.to_string(),
        duration_ms,
        data: result,
        error,
        metadata,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_workflow_result_serialization() {
        let result = ScriptWorkflowResult {
            success: true,
            workflow_type: "sequential".to_string(),
            workflow_name: "test_workflow".to_string(),
            duration_ms: 1500,
            data: serde_json::json!({
                "steps_executed": 3,
                "final_output": {"value": 42}
            }),
            error: None,
            metadata: WorkflowMetadata {
                start_time: "2023-01-01T00:00:00Z".to_string(),
                end_time: "2023-01-01T00:00:01.5Z".to_string(),
                steps_executed: Some(3),
                steps_succeeded: Some(3),
                steps_failed: Some(0),
                extra: None,
            },
        };

        // Test serialization
        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["workflow_type"], "sequential");
        assert_eq!(json["duration_ms"], 1500);
    }
}
