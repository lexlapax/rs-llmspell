//! ABOUTME: Language-agnostic workflow parameter conversion
//! ABOUTME: Provides interfaces for converting between script values and workflow types

use llmspell_workflows::ErrorStrategy;
use serde_json::Value;

/// Convert error strategy from string
pub fn parse_error_strategy(strategy: &str) -> ErrorStrategy {
    match strategy.to_lowercase().as_str() {
        "stop" | "fail_fast" => ErrorStrategy::FailFast,
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
    pub config: Value,
}

/// Convert JSON value to workflow parameters
pub fn json_to_workflow_params(value: Value) -> Result<WorkflowParams, String> {
    let obj = value
        .as_object()
        .ok_or_else(|| "Workflow params must be an object".to_string())?;

    let name = obj
        .get("name")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing workflow name".to_string())?
        .to_string();

    let workflow_type = obj
        .get("type")
        .and_then(|v| v.as_str())
        .ok_or_else(|| "Missing workflow type".to_string())?
        .to_string();

    Ok(WorkflowParams {
        name,
        workflow_type,
        config: value,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_error_strategy() {
        assert!(matches!(
            parse_error_strategy("stop"),
            ErrorStrategy::FailFast
        ));
        assert!(matches!(
            parse_error_strategy("continue"),
            ErrorStrategy::Continue
        ));
        assert!(matches!(
            parse_error_strategy("retry"),
            ErrorStrategy::Retry { .. }
        ));
        assert!(matches!(
            parse_error_strategy("unknown"),
            ErrorStrategy::FailFast
        ));
    }
}
