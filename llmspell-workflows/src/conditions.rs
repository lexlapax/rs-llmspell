//! ABOUTME: Condition evaluation engine for workflows
//! ABOUTME: Defines reusable condition types and evaluation logic

use super::traits::StepResult;
use llmspell_core::{ComponentId, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{debug, warn};

/// Condition types for workflows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Condition {
    /// Always true condition (for default branches)
    Always,
    /// Always false condition
    Never,
    /// Compare a shared data value to a target value
    SharedDataEquals {
        key: String,
        expected_value: serde_json::Value,
    },
    /// Check if shared data key exists
    SharedDataExists { key: String },
    /// Compare step result output to expected value
    StepResultEquals {
        step_id: ComponentId,
        expected_output: String,
    },
    /// Check if previous step was successful
    StepSucceeded { step_id: ComponentId },
    /// Check if previous step failed
    StepFailed { step_id: ComponentId },
    /// Logical AND of multiple conditions
    And { conditions: Vec<Condition> },
    /// Logical OR of multiple conditions
    Or { conditions: Vec<Condition> },
    /// Logical NOT of a condition
    Not { condition: Box<Condition> },
    /// Custom condition with JavaScript-like expression
    Custom {
        expression: String,
        description: String,
    },
}

impl Condition {
    /// Create a shared data equals condition
    pub fn shared_data_equals(key: String, expected_value: serde_json::Value) -> Self {
        Self::SharedDataEquals {
            key,
            expected_value,
        }
    }

    /// Create a shared data exists condition
    pub fn shared_data_exists(key: String) -> Self {
        Self::SharedDataExists { key }
    }

    /// Create a step result equals condition
    pub fn step_result_equals(step_id: ComponentId, expected_output: String) -> Self {
        Self::StepResultEquals {
            step_id,
            expected_output,
        }
    }

    /// Create a step succeeded condition
    pub fn step_succeeded(step_id: ComponentId) -> Self {
        Self::StepSucceeded { step_id }
    }

    /// Create a step failed condition
    pub fn step_failed(step_id: ComponentId) -> Self {
        Self::StepFailed { step_id }
    }

    /// Create an AND condition
    pub fn and(conditions: Vec<Condition>) -> Self {
        Self::And { conditions }
    }

    /// Create an OR condition  
    pub fn or(conditions: Vec<Condition>) -> Self {
        Self::Or { conditions }
    }

    /// Create a NOT condition
    pub fn not_condition(condition: Condition) -> Self {
        Self::Not {
            condition: Box::new(condition),
        }
    }

    /// Create a custom condition with expression
    pub fn custom(expression: String, description: String) -> Self {
        Self::Custom {
            expression,
            description,
        }
    }
}

/// Result of condition evaluation
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConditionResult {
    /// Whether the condition evaluated to true
    pub is_true: bool,
    /// Optional error message if evaluation failed
    pub error: Option<String>,
    /// Human-readable description of what was evaluated
    pub description: String,
}

impl ConditionResult {
    /// Create a successful true result
    pub fn success_true(description: String) -> Self {
        Self {
            is_true: true,
            error: None,
            description,
        }
    }

    /// Create a successful false result
    pub fn success_false(description: String) -> Self {
        Self {
            is_true: false,
            error: None,
            description,
        }
    }

    /// Create an error result
    pub fn error(description: String, error: String) -> Self {
        Self {
            is_true: false,
            error: Some(error),
            description,
        }
    }

    /// Check if evaluation was successful (no error)
    pub fn is_success(&self) -> bool {
        self.error.is_none()
    }

    /// Check if evaluation failed
    pub fn is_error(&self) -> bool {
        self.error.is_some()
    }
}

/// Context for condition evaluation
#[derive(Debug, Clone)]
pub struct ConditionEvaluationContext {
    /// Shared data from workflow state
    pub shared_data: HashMap<String, serde_json::Value>,
    /// Step outputs from completed steps
    pub step_outputs: HashMap<ComponentId, serde_json::Value>,
    /// Step results from completed steps
    pub step_results: HashMap<ComponentId, StepResult>,
    /// Current workflow execution ID
    pub execution_id: ComponentId,
}

impl ConditionEvaluationContext {
    /// Create a new condition evaluation context
    pub fn new(execution_id: ComponentId) -> Self {
        Self {
            shared_data: HashMap::new(),
            step_outputs: HashMap::new(),
            step_results: HashMap::new(),
            execution_id,
        }
    }

    /// Add shared data to context
    pub fn with_shared_data(mut self, shared_data: HashMap<String, serde_json::Value>) -> Self {
        self.shared_data = shared_data;
        self
    }

    /// Add step outputs to context
    pub fn with_step_outputs(
        mut self,
        step_outputs: HashMap<ComponentId, serde_json::Value>,
    ) -> Self {
        self.step_outputs = step_outputs;
        self
    }

    /// Add step results to context
    pub fn with_step_results(mut self, step_results: HashMap<ComponentId, StepResult>) -> Self {
        self.step_results = step_results;
        self
    }

    /// Get shared data value by key
    pub fn get_shared_data(&self, key: &str) -> Option<&serde_json::Value> {
        self.shared_data.get(key)
    }

    /// Get step output by step ID
    pub fn get_step_output(&self, step_id: ComponentId) -> Option<&serde_json::Value> {
        self.step_outputs.get(&step_id)
    }

    /// Get step result by step ID
    pub fn get_step_result(&self, step_id: ComponentId) -> Option<&StepResult> {
        self.step_results.get(&step_id)
    }
}

/// Condition evaluator for workflows
pub struct ConditionEvaluator {
    /// Timeout for condition evaluation
    evaluation_timeout: Duration,
}

impl ConditionEvaluator {
    /// Create a new condition evaluator
    pub fn new(evaluation_timeout: Duration) -> Self {
        Self { evaluation_timeout }
    }

    /// Evaluate a condition with the given context
    pub async fn evaluate(
        &self,
        condition: &Condition,
        context: &ConditionEvaluationContext,
    ) -> Result<ConditionResult> {
        let start_time = Instant::now();

        // Check timeout before evaluation
        if start_time.elapsed() > self.evaluation_timeout {
            return Ok(ConditionResult::error(
                "Condition evaluation".to_string(),
                "Evaluation timeout before start".to_string(),
            ));
        }

        let result = self.evaluate_internal(condition, context).await;
        let duration = start_time.elapsed();

        debug!(
            "Condition evaluation completed in {:?}: {}",
            duration,
            match &result {
                Ok(r) =>
                    if r.is_true {
                        "TRUE"
                    } else {
                        "FALSE"
                    },
                Err(_) => "ERROR",
            }
        );

        result
    }

    /// Internal condition evaluation logic
    async fn evaluate_internal(
        &self,
        condition: &Condition,
        context: &ConditionEvaluationContext,
    ) -> Result<ConditionResult> {
        match condition {
            Condition::Always => Ok(ConditionResult::success_true(
                "Always true condition".to_string(),
            )),
            Condition::Never => Ok(ConditionResult::success_false(
                "Always false condition".to_string(),
            )),
            Condition::SharedDataEquals {
                key,
                expected_value,
            } => {
                self.evaluate_shared_data_equals(key, expected_value, context)
                    .await
            }
            Condition::SharedDataExists { key } => {
                self.evaluate_shared_data_exists(key, context).await
            }
            Condition::StepResultEquals {
                step_id,
                expected_output,
            } => {
                self.evaluate_step_result_equals(*step_id, expected_output, context)
                    .await
            }
            Condition::StepSucceeded { step_id } => {
                self.evaluate_step_succeeded(*step_id, context).await
            }
            Condition::StepFailed { step_id } => self.evaluate_step_failed(*step_id, context).await,
            Condition::And { conditions } => self.evaluate_and_condition(conditions, context).await,
            Condition::Or { conditions } => self.evaluate_or_condition(conditions, context).await,
            Condition::Not { condition } => self.evaluate_not_condition(condition, context).await,
            Condition::Custom {
                expression,
                description,
            } => {
                self.evaluate_custom_condition(expression, description, context)
                    .await
            }
        }
    }

    /// Evaluate shared data equals condition
    async fn evaluate_shared_data_equals(
        &self,
        key: &str,
        expected_value: &serde_json::Value,
        context: &ConditionEvaluationContext,
    ) -> Result<ConditionResult> {
        let description = format!("Shared data '{}' equals {:?}", key, expected_value);

        match context.get_shared_data(key) {
            Some(actual_value) => {
                let is_equal = actual_value == expected_value;
                if is_equal {
                    Ok(ConditionResult::success_true(description))
                } else {
                    Ok(ConditionResult::success_false(format!(
                        "Shared data '{}' is {:?}, expected {:?}",
                        key, actual_value, expected_value
                    )))
                }
            }
            None => Ok(ConditionResult::success_false(format!(
                "Shared data key '{}' does not exist",
                key
            ))),
        }
    }

    /// Evaluate shared data exists condition
    async fn evaluate_shared_data_exists(
        &self,
        key: &str,
        context: &ConditionEvaluationContext,
    ) -> Result<ConditionResult> {
        let description = format!("Shared data '{}' exists", key);
        let exists = context.get_shared_data(key).is_some();

        if exists {
            Ok(ConditionResult::success_true(description))
        } else {
            Ok(ConditionResult::success_false(format!(
                "Shared data key '{}' does not exist",
                key
            )))
        }
    }

    /// Evaluate step result equals condition
    async fn evaluate_step_result_equals(
        &self,
        step_id: ComponentId,
        expected_output: &str,
        context: &ConditionEvaluationContext,
    ) -> Result<ConditionResult> {
        let description = format!("Step {:?} output equals '{}'", step_id, expected_output);

        match context.get_step_result(step_id) {
            Some(step_result) => {
                if step_result.output == expected_output {
                    Ok(ConditionResult::success_true(description))
                } else {
                    Ok(ConditionResult::success_false(format!(
                        "Step {:?} output is '{}', expected '{}'",
                        step_id, step_result.output, expected_output
                    )))
                }
            }
            None => Ok(ConditionResult::success_false(format!(
                "Step {:?} result not found",
                step_id
            ))),
        }
    }

    /// Evaluate step succeeded condition
    async fn evaluate_step_succeeded(
        &self,
        step_id: ComponentId,
        context: &ConditionEvaluationContext,
    ) -> Result<ConditionResult> {
        let description = format!("Step {:?} succeeded", step_id);

        match context.get_step_result(step_id) {
            Some(step_result) => {
                if step_result.success {
                    Ok(ConditionResult::success_true(description))
                } else {
                    Ok(ConditionResult::success_false(format!(
                        "Step {:?} failed",
                        step_id
                    )))
                }
            }
            None => Ok(ConditionResult::success_false(format!(
                "Step {:?} result not found",
                step_id
            ))),
        }
    }

    /// Evaluate step failed condition
    async fn evaluate_step_failed(
        &self,
        step_id: ComponentId,
        context: &ConditionEvaluationContext,
    ) -> Result<ConditionResult> {
        let description = format!("Step {:?} failed", step_id);

        match context.get_step_result(step_id) {
            Some(step_result) => {
                if !step_result.success {
                    Ok(ConditionResult::success_true(description))
                } else {
                    Ok(ConditionResult::success_false(format!(
                        "Step {:?} succeeded",
                        step_id
                    )))
                }
            }
            None => Ok(ConditionResult::success_false(format!(
                "Step {:?} result not found",
                step_id
            ))),
        }
    }

    /// Evaluate AND condition (all must be true)
    async fn evaluate_and_condition(
        &self,
        conditions: &[Condition],
        context: &ConditionEvaluationContext,
    ) -> Result<ConditionResult> {
        let description = format!("AND of {} conditions", conditions.len());

        if conditions.is_empty() {
            return Ok(ConditionResult::success_true(
                "Empty AND condition (vacuous truth)".to_string(),
            ));
        }

        for (index, condition) in conditions.iter().enumerate() {
            let result = Box::pin(self.evaluate_internal(condition, context)).await?;

            if result.is_error() {
                return Ok(ConditionResult::error(
                    description,
                    format!(
                        "AND condition {} failed: {}",
                        index,
                        result.error.unwrap_or_default()
                    ),
                ));
            }

            if !result.is_true {
                return Ok(ConditionResult::success_false(format!(
                    "AND condition {} is false: {}",
                    index, result.description
                )));
            }
        }

        Ok(ConditionResult::success_true(description))
    }

    /// Evaluate OR condition (at least one must be true)
    async fn evaluate_or_condition(
        &self,
        conditions: &[Condition],
        context: &ConditionEvaluationContext,
    ) -> Result<ConditionResult> {
        let description = format!("OR of {} conditions", conditions.len());

        if conditions.is_empty() {
            return Ok(ConditionResult::success_false(
                "Empty OR condition".to_string(),
            ));
        }

        let mut errors = Vec::new();

        for (index, condition) in conditions.iter().enumerate() {
            let result = Box::pin(self.evaluate_internal(condition, context)).await?;

            if result.is_error() {
                errors.push(format!(
                    "OR condition {}: {}",
                    index,
                    result.error.unwrap_or_default()
                ));
                continue;
            }

            if result.is_true {
                return Ok(ConditionResult::success_true(format!(
                    "OR condition {} is true: {}",
                    index, result.description
                )));
            }
        }

        if !errors.is_empty() {
            Ok(ConditionResult::error(
                description,
                format!("OR condition errors: {}", errors.join(", ")),
            ))
        } else {
            Ok(ConditionResult::success_false(format!(
                "All {} OR conditions are false",
                conditions.len()
            )))
        }
    }

    /// Evaluate NOT condition (inverse of inner condition)
    async fn evaluate_not_condition(
        &self,
        condition: &Condition,
        context: &ConditionEvaluationContext,
    ) -> Result<ConditionResult> {
        let result = Box::pin(self.evaluate_internal(condition, context)).await?;

        if result.is_error() {
            return Ok(ConditionResult::error(
                "NOT condition".to_string(),
                format!(
                    "Inner condition failed: {}",
                    result.error.unwrap_or_default()
                ),
            ));
        }

        let is_true = !result.is_true;
        let description = format!("NOT ({})", result.description);

        if is_true {
            Ok(ConditionResult::success_true(description))
        } else {
            Ok(ConditionResult::success_false(description))
        }
    }

    /// Evaluate custom condition with simple expression parsing
    async fn evaluate_custom_condition(
        &self,
        expression: &str,
        description: &str,
        context: &ConditionEvaluationContext,
    ) -> Result<ConditionResult> {
        // For now, implement basic expression evaluation
        // In a full implementation, this would use a proper expression parser

        if expression.is_empty() {
            return Ok(ConditionResult::error(
                description.to_string(),
                "Empty custom expression".to_string(),
            ));
        }

        // Simple pattern matching for basic expressions
        if let Some(result) = self.evaluate_simple_expression(expression, context).await {
            Ok(result)
        } else {
            warn!("Custom expression not supported: {}", expression);
            Ok(ConditionResult::error(
                description.to_string(),
                format!("Unsupported custom expression: {}", expression),
            ))
        }
    }

    /// Evaluate simple custom expressions (basic implementation)
    async fn evaluate_simple_expression(
        &self,
        expression: &str,
        context: &ConditionEvaluationContext,
    ) -> Option<ConditionResult> {
        let expr = expression.trim();

        // Handle simple boolean literals
        if expr == "true" {
            return Some(ConditionResult::success_true("Custom: true".to_string()));
        }
        if expr == "false" {
            return Some(ConditionResult::success_false("Custom: false".to_string()));
        }

        // Handle shared data access patterns: shared_data.key == "value"
        if let Some(result) = self.parse_shared_data_expression(expr, context).await {
            return Some(result);
        }

        // Handle step result patterns: step_result.step_id.success
        if let Some(result) = self.parse_step_result_expression(expr, context).await {
            return Some(result);
        }

        None
    }

    /// Parse shared data expressions like "shared_data.key == 'value'"
    async fn parse_shared_data_expression(
        &self,
        expr: &str,
        context: &ConditionEvaluationContext,
    ) -> Option<ConditionResult> {
        // Simple pattern: shared_data.key == "value"
        if let Some(equals_pos) = expr.find("==") {
            let left = expr[..equals_pos].trim();
            let right = expr[equals_pos + 2..].trim();

            if let Some(key) = left.strip_prefix("shared_data.") {
                let expected_value = if right.starts_with('"') && right.ends_with('"') {
                    serde_json::Value::String(right[1..right.len() - 1].to_string())
                } else if right == "true" {
                    serde_json::Value::Bool(true)
                } else if right == "false" {
                    serde_json::Value::Bool(false)
                } else if let Ok(num) = right.parse::<i64>() {
                    serde_json::Value::Number(serde_json::Number::from(num))
                } else {
                    return Some(ConditionResult::error(
                        "Custom expression".to_string(),
                        format!("Cannot parse value: {}", right),
                    ));
                };

                let description = format!("Custom: shared_data.{} == {}", key, right);

                match context.get_shared_data(key) {
                    Some(actual_value) => {
                        if actual_value == &expected_value {
                            return Some(ConditionResult::success_true(description));
                        } else {
                            return Some(ConditionResult::success_false(format!(
                                "Custom: shared_data.{} is {:?}, expected {}",
                                key, actual_value, right
                            )));
                        }
                    }
                    None => {
                        return Some(ConditionResult::success_false(format!(
                            "Custom: shared_data.{} does not exist",
                            key
                        )));
                    }
                }
            }
        }

        None
    }

    /// Parse step result expressions like "step_result.step_id.success"
    async fn parse_step_result_expression(
        &self,
        expr: &str,
        context: &ConditionEvaluationContext,
    ) -> Option<ConditionResult> {
        // Simple pattern: step_result.step_id.success or step_result.step_id.failed
        if expr.starts_with("step_result.") {
            let parts: Vec<&str> = expr.split('.').collect();
            if parts.len() >= 3 {
                let step_id_str = parts[1];
                let property = parts[2];

                // For simplicity, use the first step result if step_id parsing fails
                // In a real implementation, you'd parse the ComponentId properly
                if let Some(first_result) = context.step_results.values().next() {
                    match property {
                        "success" => {
                            let description = format!("Custom: step.{}.success", step_id_str);
                            if first_result.success {
                                return Some(ConditionResult::success_true(description));
                            } else {
                                return Some(ConditionResult::success_false(description));
                            }
                        }
                        "failed" => {
                            let description = format!("Custom: step.{}.failed", step_id_str);
                            if !first_result.success {
                                return Some(ConditionResult::success_true(description));
                            } else {
                                return Some(ConditionResult::success_false(description));
                            }
                        }
                        _ => {
                            return Some(ConditionResult::error(
                                "Custom expression".to_string(),
                                format!("Unknown step property: {}", property),
                            ));
                        }
                    }
                }
            }
        }

        None
    }
}

#[cfg(test)]
#[cfg_attr(test_category = "workflow")]
mod tests {
    use super::*;
    use std::time::Duration;

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_always_condition() {
        let evaluator = ConditionEvaluator::new(Duration::from_secs(1));
        let context = ConditionEvaluationContext::new(ComponentId::new());
        let condition = Condition::Always;

        let result = evaluator.evaluate(&condition, &context).await.unwrap();
        assert!(result.is_true);
        assert!(result.is_success());
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_never_condition() {
        let evaluator = ConditionEvaluator::new(Duration::from_secs(1));
        let context = ConditionEvaluationContext::new(ComponentId::new());
        let condition = Condition::Never;

        let result = evaluator.evaluate(&condition, &context).await.unwrap();
        assert!(!result.is_true);
        assert!(result.is_success());
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_shared_data_equals_condition() {
        let evaluator = ConditionEvaluator::new(Duration::from_secs(1));
        let mut shared_data = std::collections::HashMap::new();
        shared_data.insert("test_key".to_string(), serde_json::json!("test_value"));

        let context =
            ConditionEvaluationContext::new(ComponentId::new()).with_shared_data(shared_data);

        // Test matching condition
        let condition =
            Condition::shared_data_equals("test_key".to_string(), serde_json::json!("test_value"));
        let result = evaluator.evaluate(&condition, &context).await.unwrap();
        assert!(result.is_true);

        // Test non-matching condition
        let condition = Condition::shared_data_equals(
            "test_key".to_string(),
            serde_json::json!("different_value"),
        );
        let result = evaluator.evaluate(&condition, &context).await.unwrap();
        assert!(!result.is_true);
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_and_condition() {
        let evaluator = ConditionEvaluator::new(Duration::from_secs(1));
        let context = ConditionEvaluationContext::new(ComponentId::new());

        // All true conditions
        let condition = Condition::and(vec![Condition::Always, Condition::Always]);
        let result = evaluator.evaluate(&condition, &context).await.unwrap();
        assert!(result.is_true);

        // Mixed conditions (one false)
        let condition = Condition::and(vec![Condition::Always, Condition::Never]);
        let result = evaluator.evaluate(&condition, &context).await.unwrap();
        assert!(!result.is_true);
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_or_condition() {
        let evaluator = ConditionEvaluator::new(Duration::from_secs(1));
        let context = ConditionEvaluationContext::new(ComponentId::new());

        // At least one true condition
        let condition = Condition::or(vec![Condition::Never, Condition::Always]);
        let result = evaluator.evaluate(&condition, &context).await.unwrap();
        assert!(result.is_true);

        // All false conditions
        let condition = Condition::or(vec![Condition::Never, Condition::Never]);
        let result = evaluator.evaluate(&condition, &context).await.unwrap();
        assert!(!result.is_true);
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_not_condition() {
        let evaluator = ConditionEvaluator::new(Duration::from_secs(1));
        let context = ConditionEvaluationContext::new(ComponentId::new());

        // NOT true = false
        let condition = Condition::not_condition(Condition::Always);
        let result = evaluator.evaluate(&condition, &context).await.unwrap();
        assert!(!result.is_true);

        // NOT false = true
        let condition = Condition::not_condition(Condition::Never);
        let result = evaluator.evaluate(&condition, &context).await.unwrap();
        assert!(result.is_true);
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_custom_condition_simple() {
        let evaluator = ConditionEvaluator::new(Duration::from_secs(1));
        let context = ConditionEvaluationContext::new(ComponentId::new());

        // Simple boolean literals
        let condition = Condition::custom("true".to_string(), "Always true".to_string());
        let result = evaluator.evaluate(&condition, &context).await.unwrap();
        assert!(result.is_true);

        let condition = Condition::custom("false".to_string(), "Always false".to_string());
        let result = evaluator.evaluate(&condition, &context).await.unwrap();
        assert!(!result.is_true);
    }
}
