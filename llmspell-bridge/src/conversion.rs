//! ABOUTME: Common type conversion traits for script-to-native translations
//! ABOUTME: Defines core traits used by all language-specific conversions

use llmspell_core::Result;
use llmspell_workflows::ErrorStrategy;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Trait for converting Rust types to script values
pub trait ToScriptValue<T> {
    /// Convert this Rust type to a script value
    ///
    /// # Errors
    ///
    /// Returns an error if the conversion fails
    fn to_script_value(&self) -> Result<T>;
}

/// Trait for converting script values to Rust types
pub trait FromScriptValue<T>: Sized {
    /// Convert a script value to this Rust type
    ///
    /// # Errors
    ///
    /// Returns an error if the conversion fails or the value is invalid
    fn from_script_value(value: T) -> Result<Self>;
}

/// Common script value representation for cross-language support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScriptValue {
    /// Null/nil value
    Null,
    /// Boolean value
    Bool(bool),
    /// Numeric value (all numbers as f64)
    Number(f64),
    /// String value
    String(String),
    /// Array of values
    Array(Vec<ScriptValue>),
    /// Object/table with string keys
    Object(HashMap<String, ScriptValue>),
    /// Binary data
    Bytes(Vec<u8>),
}

impl ScriptValue {
    /// Convert to JSON value
    #[must_use]
    pub fn to_json(&self) -> serde_json::Value {
        match self {
            Self::Null => serde_json::Value::Null,
            Self::Bool(b) => serde_json::Value::Bool(*b),
            Self::Number(n) => serde_json::json!(n),
            Self::String(s) => serde_json::Value::String(s.clone()),
            Self::Array(arr) => serde_json::Value::Array(arr.iter().map(Self::to_json).collect()),
            Self::Object(obj) => {
                let map: serde_json::Map<String, serde_json::Value> =
                    obj.iter().map(|(k, v)| (k.clone(), v.to_json())).collect();
                serde_json::Value::Object(map)
            }
            Self::Bytes(bytes) => {
                // Encode bytes as base64 string
                serde_json::Value::String(base64::Engine::encode(
                    &base64::engine::general_purpose::STANDARD,
                    bytes,
                ))
            }
        }
    }

    /// Convert from JSON value
    pub fn from_json(value: &serde_json::Value) -> Self {
        match value {
            serde_json::Value::Null => Self::Null,
            serde_json::Value::Bool(b) => Self::Bool(*b),
            serde_json::Value::Number(n) => Self::Number(n.as_f64().unwrap_or(0.0)),
            serde_json::Value::String(s) => Self::String(s.clone()),
            serde_json::Value::Array(arr) => Self::Array(arr.iter().map(Self::from_json).collect()),
            serde_json::Value::Object(obj) => {
                let map = obj
                    .iter()
                    .map(|(k, v)| (k.clone(), Self::from_json(v)))
                    .collect();
                Self::Object(map)
            }
        }
    }
}

// Implement conversions for primitive types
impl From<bool> for ScriptValue {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

impl From<i32> for ScriptValue {
    fn from(n: i32) -> Self {
        Self::Number(f64::from(n))
    }
}

impl From<i64> for ScriptValue {
    fn from(n: i64) -> Self {
        // Note: i64 to f64 conversion may lose precision for very large numbers
        #[allow(clippy::cast_precision_loss)]
        let f = n as f64;
        Self::Number(f)
    }
}

impl From<f32> for ScriptValue {
    fn from(n: f32) -> Self {
        Self::Number(f64::from(n))
    }
}

impl From<f64> for ScriptValue {
    fn from(n: f64) -> Self {
        Self::Number(n)
    }
}

impl From<String> for ScriptValue {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<&str> for ScriptValue {
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}

impl<T> From<Option<T>> for ScriptValue
where
    T: Into<Self>,
{
    fn from(opt: Option<T>) -> Self {
        opt.map_or(Self::Null, std::convert::Into::into)
    }
}

impl<T> From<Vec<T>> for ScriptValue
where
    T: Into<Self>,
{
    fn from(vec: Vec<T>) -> Self {
        Self::Array(vec.into_iter().map(std::convert::Into::into).collect())
    }
}

impl From<Vec<u8>> for ScriptValue {
    fn from(bytes: Vec<u8>) -> Self {
        Self::Bytes(bytes)
    }
}

/// Conversion utilities
pub struct ConversionUtils;

impl ConversionUtils {
    /// Check if a `ScriptValue` is truthy (for conditional evaluation)
    #[must_use]
    pub fn is_truthy(value: &ScriptValue) -> bool {
        match value {
            ScriptValue::Null => false,
            ScriptValue::Bool(b) => *b,
            ScriptValue::Number(n) => *n != 0.0,
            ScriptValue::String(s) => !s.is_empty(),
            ScriptValue::Array(arr) => !arr.is_empty(),
            ScriptValue::Object(obj) => !obj.is_empty(),
            ScriptValue::Bytes(bytes) => !bytes.is_empty(),
        }
    }

    /// Get a nested value from an object by dot-separated path
    #[must_use]
    pub fn get_nested<'a>(value: &'a ScriptValue, path: &str) -> Option<&'a ScriptValue> {
        let parts: Vec<&str> = path.split('.').collect();
        let mut current = value;

        for part in parts {
            match current {
                ScriptValue::Object(obj) => {
                    current = obj.get(part)?;
                }
                _ => return None,
            }
        }

        Some(current)
    }
}

// ===== Workflow conversions =====

/// Convert error strategy from string
#[must_use]
pub fn parse_error_strategy(strategy: &str) -> ErrorStrategy {
    match strategy.to_lowercase().as_str() {
        "continue" => ErrorStrategy::Continue,
        "retry" => ErrorStrategy::Retry {
            max_attempts: 3,
            backoff_ms: 1000,
        },
        _ => ErrorStrategy::FailFast,
    }
}

/// Generic workflow parameter structure
#[derive(Debug, Clone)]
pub struct WorkflowParams {
    pub name: String,
    pub workflow_type: String,
    pub config: serde_json::Value,
}

/// Convert JSON value to workflow parameters
///
/// # Errors
///
/// Returns an error if the JSON value is not an object or is missing required fields
pub fn json_to_workflow_params(value: serde_json::Value) -> Result<WorkflowParams> {
    let obj = value
        .as_object()
        .ok_or_else(|| llmspell_core::LLMSpellError::Component {
            message: "Workflow params must be an object".to_string(),
            source: None,
        })?;

    let name = obj
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| llmspell_core::LLMSpellError::Component {
            message: "Missing workflow name".to_string(),
            source: None,
        })?
        .to_string();

    let workflow_type = obj
        .get("type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| llmspell_core::LLMSpellError::Component {
            message: "Missing workflow type".to_string(),
            source: None,
        })?
        .to_string();

    Ok(WorkflowParams {
        name,
        workflow_type,
        config: value,
    })
}

// =====================================================================
// Workflow Result Types and Conversions
// (Moved from workflow_results.rs for consolidation)
// =====================================================================

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
#[must_use]
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

    let error = if result.success {
        None
    } else {
        result.error_message.as_ref().map(|e| WorkflowError {
            error_type: "SequentialExecutionError".to_string(),
            message: e.clone(),
            location: result.failed_steps.first().map(|sr| sr.step_name.clone()),
            details: None,
        })
    };

    ScriptWorkflowResult {
        success: result.success,
        workflow_type: "sequential".to_string(),
        workflow_name: result.workflow_name.clone(),
        duration_ms: u64::try_from(result.duration.as_millis()).unwrap_or(0),
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
#[must_use]
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

    let error = if result.success {
        None
    } else {
        result.error_message.as_ref().map(|e| WorkflowError {
            error_type: "ConditionalExecutionError".to_string(),
            message: e.clone(),
            location: result
                .executed_branches
                .first()
                .map(|br| br.branch_name.clone()),
            details: None,
        })
    };

    ScriptWorkflowResult {
        success: result.success,
        workflow_type: "conditional".to_string(),
        workflow_name: result.workflow_name.clone(),
        duration_ms: u64::try_from(result.duration.as_millis()).unwrap_or(0),
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
#[must_use]
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

    let error = if result.success {
        None
    } else {
        result.error.as_ref().map(|e| WorkflowError {
            error_type: "LoopExecutionError".to_string(),
            message: e.clone(),
            location: Some(format!("iteration {}", result.completed_iterations)),
            details: None,
        })
    };

    ScriptWorkflowResult {
        success: result.success,
        workflow_type: "loop".to_string(),
        workflow_name: result.workflow_name.clone(),
        duration_ms: u64::try_from(result.duration.as_millis()).unwrap_or(0),
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
#[must_use]
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

    let error = if result.success {
        None
    } else {
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
        duration_ms: u64::try_from(result.duration.as_millis()).unwrap_or(0),
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
///
/// # Errors
///
/// Returns an error if the result JSON cannot be parsed into a valid workflow result
pub fn transform_generic_result(
    workflow_type: &str,
    workflow_name: &str,
    result: Value,
) -> Result<ScriptWorkflowResult> {
    // Try to extract common fields
    let success = result
        .get("success")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);

    let duration_ms = result
        .get("duration")
        .and_then(serde_json::Value::as_u64)
        .or_else(|| {
            result
                .get("duration_ms")
                .and_then(serde_json::Value::as_u64)
        })
        .unwrap_or(0);

    let error = if success {
        None
    } else {
        result
            .get("error")
            .and_then(|v| v.as_str())
            .map(|e| WorkflowError {
                error_type: format!("{workflow_type}Error"),
                message: e.to_string(),
                location: None,
                details: None,
            })
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
mod workflow_result_tests {
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
