//! ABOUTME: Conditional workflow types and structures
//! ABOUTME: Defines condition evaluation, branching logic, and conditional step types

use llmspell_core::ComponentId;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Conditional workflow branch containing steps to execute when condition is met
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalBranch {
    /// Unique identifier for this branch
    pub id: ComponentId,
    /// Human-readable name for the branch
    pub name: String,
    /// Condition that must be true to execute this branch
    pub condition: BasicCondition,
    /// Steps to execute when condition is met
    pub steps: Vec<super::super::traits::BasicWorkflowStep>,
    /// Whether this is the default branch (executes if no other conditions match)
    pub is_default: bool,
}

impl ConditionalBranch {
    /// Create a new conditional branch
    pub fn new(name: String, condition: BasicCondition) -> Self {
        Self {
            id: ComponentId::new(),
            name,
            condition,
            steps: Vec::new(),
            is_default: false,
        }
    }

    /// Create a default branch (executes when no conditions match)
    pub fn default(name: String) -> Self {
        Self {
            id: ComponentId::new(),
            name,
            condition: BasicCondition::Always, // Always true condition
            steps: Vec::new(),
            is_default: true,
        }
    }

    /// Add a step to this branch
    pub fn with_step(mut self, step: super::super::traits::BasicWorkflowStep) -> Self {
        self.steps.push(step);
        self
    }

    /// Add multiple steps to this branch
    pub fn with_steps(mut self, steps: Vec<super::super::traits::BasicWorkflowStep>) -> Self {
        self.steps.extend(steps);
        self
    }
}

/// Basic condition types for conditional workflows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BasicCondition {
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
    And { conditions: Vec<BasicCondition> },
    /// Logical OR of multiple conditions
    Or { conditions: Vec<BasicCondition> },
    /// Logical NOT of a condition
    Not { condition: Box<BasicCondition> },
    /// Custom condition with JavaScript-like expression
    Custom {
        expression: String,
        description: String,
    },
}

impl BasicCondition {
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
    pub fn and(conditions: Vec<BasicCondition>) -> Self {
        Self::And { conditions }
    }

    /// Create an OR condition  
    pub fn or(conditions: Vec<BasicCondition>) -> Self {
        Self::Or { conditions }
    }

    /// Create a NOT condition
    pub fn not_condition(condition: BasicCondition) -> Self {
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
    pub step_results: HashMap<ComponentId, super::super::traits::BasicStepResult>,
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
    pub fn with_step_results(
        mut self,
        step_results: HashMap<ComponentId, super::super::traits::BasicStepResult>,
    ) -> Self {
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
    pub fn get_step_result(
        &self,
        step_id: ComponentId,
    ) -> Option<&super::super::traits::BasicStepResult> {
        self.step_results.get(&step_id)
    }
}

/// Branch execution result
#[derive(Debug, Clone)]
pub struct BranchExecutionResult {
    /// Branch that was executed
    pub branch_id: ComponentId,
    /// Branch name
    pub branch_name: String,
    /// Condition evaluation result
    pub condition_result: ConditionResult,
    /// Results from executed steps
    pub step_results: Vec<super::super::traits::BasicStepResult>,
    /// Whether the branch execution was successful
    pub success: bool,
    /// Total execution time for the branch
    pub duration: std::time::Duration,
}

impl BranchExecutionResult {
    /// Create a successful branch execution result
    pub fn success(
        branch_id: ComponentId,
        branch_name: String,
        condition_result: ConditionResult,
        step_results: Vec<super::super::traits::BasicStepResult>,
        duration: std::time::Duration,
    ) -> Self {
        let success = step_results.iter().all(|r| r.success);
        Self {
            branch_id,
            branch_name,
            condition_result,
            step_results,
            success,
            duration,
        }
    }
}

/// Configuration for conditional workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalWorkflowConfig {
    /// Whether to execute all matching branches or just the first one
    pub execute_all_matching: bool,
    /// Whether to execute the default branch if no conditions match
    pub execute_default_on_no_match: bool,
    /// Maximum number of branches to evaluate (prevents infinite loops)
    pub max_branches_to_evaluate: usize,
    /// Timeout for condition evaluation
    pub condition_evaluation_timeout_ms: u64,
    /// Whether to short-circuit evaluation (stop on first true condition)
    pub short_circuit_evaluation: bool,
}

impl Default for ConditionalWorkflowConfig {
    fn default() -> Self {
        Self {
            execute_all_matching: false, // Execute only first matching branch
            execute_default_on_no_match: true,
            max_branches_to_evaluate: 100,
            condition_evaluation_timeout_ms: 1000, // 1 second
            short_circuit_evaluation: true,
        }
    }
}
