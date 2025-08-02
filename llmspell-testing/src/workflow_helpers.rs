// ABOUTME: Test utilities for workflow testing including builders and common patterns
// ABOUTME: Provides reusable helpers for workflow unit and integration tests

use llmspell_workflows::{
    conditional::{ConditionalWorkflow, ConditionalWorkflowBuilder},
    conditions::Condition,
    parallel::{ParallelWorkflow, ParallelWorkflowBuilder},
    r#loop::{LoopWorkflow, LoopWorkflowBuilder},
    sequential::{SequentialWorkflow, SequentialWorkflowBuilder},
    traits::{ErrorStrategy, StepType, WorkflowStep},
    types::WorkflowConfig,
};
use serde_json::Value;
use std::time::Duration;

/// Create a test workflow step
pub fn create_test_workflow_step(name: &str) -> WorkflowStep {
    WorkflowStep::new(
        name.to_string(),
        StepType::Tool {
            tool_name: "test_tool".to_string(),
            parameters: serde_json::json!({"test": true}),
        },
    )
}

/// Create a test tool workflow step
pub fn create_test_tool_step(name: &str, tool_name: &str, params: Value) -> WorkflowStep {
    WorkflowStep::new(
        name.to_string(),
        StepType::Tool {
            tool_name: tool_name.to_string(),
            parameters: params,
        },
    )
}

/// Create a test agent workflow step
pub fn create_test_agent_step(name: &str, agent_id: &str, input: String) -> WorkflowStep {
    use llmspell_core::types::ComponentId;
    WorkflowStep::new(
        name.to_string(),
        StepType::Agent {
            agent_id: ComponentId::from_name(agent_id),
            input,
        },
    )
}

/// Create a test sub-workflow step
pub fn create_test_subworkflow_step(name: &str, workflow_id: &str) -> WorkflowStep {
    WorkflowStep::new(
        name.to_string(),
        StepType::Custom {
            function_name: "execute_workflow".to_string(),
            parameters: serde_json::json!({
                "workflow_id": workflow_id
            }),
        },
    )
}

/// Create a test sequential workflow
pub fn create_test_sequential_workflow(name: &str) -> SequentialWorkflow {
    SequentialWorkflow::new(name.to_string(), WorkflowConfig::default())
}

/// Create a test sequential workflow with steps
pub fn create_test_sequential_workflow_with_steps(
    name: &str,
    steps: Vec<WorkflowStep>,
) -> SequentialWorkflow {
    let mut builder = SequentialWorkflow::builder(name.to_string());
    for step in steps {
        builder = builder.add_step(step);
    }
    builder.build()
}

/// Create a test parallel workflow
pub fn create_test_parallel_workflow(name: &str) -> ParallelWorkflow {
    ParallelWorkflowBuilder::new(name).build().unwrap()
}

/// Create a test conditional workflow
pub fn create_test_conditional_workflow(name: &str) -> ConditionalWorkflow {
    ConditionalWorkflow::builder(name.to_string()).build()
}

/// Create a test loop workflow
pub fn create_test_loop_workflow(name: &str, max_iterations: usize) -> LoopWorkflow {
    LoopWorkflowBuilder::new(name)
        .with_range(0, max_iterations as i64, 1)
        .build()
        .unwrap()
}

/// Create a test condition
pub fn create_test_condition(always_true: bool) -> Condition {
    if always_true {
        Condition::Always
    } else {
        Condition::Never
    }
}

/// Create a test workflow config
pub fn create_test_workflow_config() -> WorkflowConfig {
    WorkflowConfig::default()
}

/// Create a test workflow config with retry
pub fn create_test_workflow_config_with_retry(max_retries: u32) -> WorkflowConfig {
    let mut config = WorkflowConfig::default();
    config.max_retry_attempts = max_retries;
    config.exponential_backoff = true;
    config.retry_delay_ms = 100;
    config
}

/// Create a test workflow config with error strategy
pub fn create_test_workflow_config_with_error_strategy(strategy: ErrorStrategy) -> WorkflowConfig {
    let mut config = WorkflowConfig::default();
    config.default_error_strategy = strategy;
    config
}

/// Create multiple test steps
pub fn create_test_steps(count: usize) -> Vec<WorkflowStep> {
    (0..count)
        .map(|i| create_test_workflow_step(&format!("step_{}", i)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_workflow_step() {
        let step = create_test_workflow_step("test");
        assert_eq!(step.name, "test");
        match step.step_type {
            StepType::Tool { tool_name, .. } => {
                assert_eq!(tool_name, "test_tool");
            }
            _ => panic!("Expected tool step"),
        }
    }

    #[test]
    fn test_create_tool_step() {
        let params = serde_json::json!({"operation": "add", "a": 1, "b": 2});
        let step = create_test_tool_step("calc", "calculator", params.clone());
        assert_eq!(step.name, "calc");
        match step.step_type {
            StepType::Tool {
                tool_name,
                parameters,
            } => {
                assert_eq!(tool_name, "calculator");
                assert_eq!(parameters, params);
            }
            _ => panic!("Expected tool step"),
        }
    }

    #[test]
    fn test_create_agent_step() {
        use llmspell_core::types::ComponentId;
        let input = "Process this message";
        let step = create_test_agent_step("chat", "agent-1", input.to_string());
        assert_eq!(step.name, "chat");
        match step.step_type {
            StepType::Agent {
                agent_id,
                input: agent_input,
            } => {
                assert_eq!(agent_id, ComponentId::from_name("agent-1"));
                assert_eq!(agent_input, input);
            }
            _ => panic!("Expected agent step"),
        }
    }

    #[test]
    fn test_create_sequential_workflow() {
        let workflow = create_test_sequential_workflow("test");
        assert_eq!(workflow.name(), "test");
        assert_eq!(workflow.step_count(), 0);
    }

    #[test]
    fn test_create_sequential_workflow_with_steps() {
        let steps = create_test_steps(3);
        let workflow = create_test_sequential_workflow_with_steps("test", steps);
        assert_eq!(workflow.name(), "test");
        assert_eq!(workflow.step_count(), 3);
    }

    #[test]
    fn test_create_condition() {
        let true_condition = create_test_condition(true);
        assert!(matches!(true_condition, Condition::Always));

        let false_condition = create_test_condition(false);
        assert!(matches!(false_condition, Condition::Never));
    }

    #[test]
    fn test_create_workflow_config_with_retry() {
        let config = create_test_workflow_config_with_retry(5);
        assert_eq!(config.max_retry_attempts, 5);
        assert_eq!(config.retry_delay_ms, 100);
        assert!(config.exponential_backoff);
    }

    #[test]
    fn test_create_test_steps() {
        let steps = create_test_steps(5);
        assert_eq!(steps.len(), 5);
        for (i, step) in steps.iter().enumerate() {
            assert_eq!(step.name, format!("step_{}", i));
        }
    }
}
