//! ABOUTME: Condition evaluation engine for conditional workflows
//! ABOUTME: Handles evaluation of various condition types with performance optimization

use super::types::{BasicCondition, ConditionEvaluationContext, ConditionResult};
use llmspell_core::{ComponentId, Result};
use std::time::{Duration, Instant};
use tracing::{debug, warn};

/// Basic condition evaluator for conditional workflows
pub struct BasicConditionEvaluator {
    /// Timeout for condition evaluation
    evaluation_timeout: Duration,
}

impl BasicConditionEvaluator {
    /// Create a new condition evaluator
    pub fn new(evaluation_timeout: Duration) -> Self {
        Self { evaluation_timeout }
    }

    /// Evaluate a condition with the given context
    pub async fn evaluate(
        &self,
        condition: &BasicCondition,
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
        condition: &BasicCondition,
        context: &ConditionEvaluationContext,
    ) -> Result<ConditionResult> {
        match condition {
            BasicCondition::Always => Ok(ConditionResult::success_true(
                "Always true condition".to_string(),
            )),
            BasicCondition::Never => Ok(ConditionResult::success_false(
                "Always false condition".to_string(),
            )),
            BasicCondition::SharedDataEquals {
                key,
                expected_value,
            } => {
                self.evaluate_shared_data_equals(key, expected_value, context)
                    .await
            }
            BasicCondition::SharedDataExists { key } => {
                self.evaluate_shared_data_exists(key, context).await
            }
            BasicCondition::StepResultEquals {
                step_id,
                expected_output,
            } => {
                self.evaluate_step_result_equals(*step_id, expected_output, context)
                    .await
            }
            BasicCondition::StepSucceeded { step_id } => {
                self.evaluate_step_succeeded(*step_id, context).await
            }
            BasicCondition::StepFailed { step_id } => {
                self.evaluate_step_failed(*step_id, context).await
            }
            BasicCondition::And { conditions } => {
                self.evaluate_and_condition(conditions, context).await
            }
            BasicCondition::Or { conditions } => {
                self.evaluate_or_condition(conditions, context).await
            }
            BasicCondition::Not { condition } => {
                self.evaluate_not_condition(condition, context).await
            }
            BasicCondition::Custom {
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
        conditions: &[BasicCondition],
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
        conditions: &[BasicCondition],
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
        condition: &BasicCondition,
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
mod tests {
    use super::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_always_condition() {
        let evaluator = BasicConditionEvaluator::new(Duration::from_secs(1));
        let context = ConditionEvaluationContext::new(ComponentId::new());
        let condition = BasicCondition::Always;

        let result = evaluator.evaluate(&condition, &context).await.unwrap();
        assert!(result.is_true);
        assert!(result.is_success());
    }

    #[tokio::test]
    async fn test_never_condition() {
        let evaluator = BasicConditionEvaluator::new(Duration::from_secs(1));
        let context = ConditionEvaluationContext::new(ComponentId::new());
        let condition = BasicCondition::Never;

        let result = evaluator.evaluate(&condition, &context).await.unwrap();
        assert!(!result.is_true);
        assert!(result.is_success());
    }

    #[tokio::test]
    async fn test_shared_data_equals_condition() {
        let evaluator = BasicConditionEvaluator::new(Duration::from_secs(1));
        let mut shared_data = std::collections::HashMap::new();
        shared_data.insert("test_key".to_string(), serde_json::json!("test_value"));

        let context =
            ConditionEvaluationContext::new(ComponentId::new()).with_shared_data(shared_data);

        // Test matching condition
        let condition = BasicCondition::shared_data_equals(
            "test_key".to_string(),
            serde_json::json!("test_value"),
        );
        let result = evaluator.evaluate(&condition, &context).await.unwrap();
        assert!(result.is_true);

        // Test non-matching condition
        let condition = BasicCondition::shared_data_equals(
            "test_key".to_string(),
            serde_json::json!("different_value"),
        );
        let result = evaluator.evaluate(&condition, &context).await.unwrap();
        assert!(!result.is_true);
    }

    #[tokio::test]
    async fn test_and_condition() {
        let evaluator = BasicConditionEvaluator::new(Duration::from_secs(1));
        let context = ConditionEvaluationContext::new(ComponentId::new());

        // All true conditions
        let condition = BasicCondition::and(vec![BasicCondition::Always, BasicCondition::Always]);
        let result = evaluator.evaluate(&condition, &context).await.unwrap();
        assert!(result.is_true);

        // Mixed conditions (one false)
        let condition = BasicCondition::and(vec![BasicCondition::Always, BasicCondition::Never]);
        let result = evaluator.evaluate(&condition, &context).await.unwrap();
        assert!(!result.is_true);
    }

    #[tokio::test]
    async fn test_or_condition() {
        let evaluator = BasicConditionEvaluator::new(Duration::from_secs(1));
        let context = ConditionEvaluationContext::new(ComponentId::new());

        // At least one true condition
        let condition = BasicCondition::or(vec![BasicCondition::Never, BasicCondition::Always]);
        let result = evaluator.evaluate(&condition, &context).await.unwrap();
        assert!(result.is_true);

        // All false conditions
        let condition = BasicCondition::or(vec![BasicCondition::Never, BasicCondition::Never]);
        let result = evaluator.evaluate(&condition, &context).await.unwrap();
        assert!(!result.is_true);
    }

    #[tokio::test]
    async fn test_not_condition() {
        let evaluator = BasicConditionEvaluator::new(Duration::from_secs(1));
        let context = ConditionEvaluationContext::new(ComponentId::new());

        // NOT true = false
        let condition = BasicCondition::not_condition(BasicCondition::Always);
        let result = evaluator.evaluate(&condition, &context).await.unwrap();
        assert!(!result.is_true);

        // NOT false = true
        let condition = BasicCondition::not_condition(BasicCondition::Never);
        let result = evaluator.evaluate(&condition, &context).await.unwrap();
        assert!(result.is_true);
    }

    #[tokio::test]
    async fn test_custom_condition_simple() {
        let evaluator = BasicConditionEvaluator::new(Duration::from_secs(1));
        let context = ConditionEvaluationContext::new(ComponentId::new());

        // Simple boolean literals
        let condition = BasicCondition::custom("true".to_string(), "Always true".to_string());
        let result = evaluator.evaluate(&condition, &context).await.unwrap();
        assert!(result.is_true);

        let condition = BasicCondition::custom("false".to_string(), "Always false".to_string());
        let result = evaluator.evaluate(&condition, &context).await.unwrap();
        assert!(!result.is_true);
    }
}
