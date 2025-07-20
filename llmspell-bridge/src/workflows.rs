//! ABOUTME: Workflow discovery and management for script integration
//! ABOUTME: Provides workflow type information and factory methods

use llmspell_core::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Information about a workflow type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowInfo {
    /// Workflow type name (e.g., "sequential", "conditional", "loop", "parallel")
    pub workflow_type: String,
    /// Human-readable description
    pub description: String,
    /// Supported features
    pub features: Vec<String>,
    /// Required parameters for creation
    pub required_params: Vec<String>,
    /// Optional parameters
    pub optional_params: Vec<String>,
}

/// Discovery service for available workflow types
pub struct WorkflowDiscovery {
    /// Registry of available workflow types
    workflow_types: HashMap<String, WorkflowInfo>,
}

impl WorkflowDiscovery {
    /// Create a new workflow discovery service
    pub fn new() -> Self {
        let mut workflow_types = HashMap::new();

        // Register Sequential workflow
        workflow_types.insert(
            "sequential".to_string(),
            WorkflowInfo {
                workflow_type: "sequential".to_string(),
                description: "Execute steps one after another in order".to_string(),
                features: vec![
                    "ordered_execution".to_string(),
                    "state_passing".to_string(),
                    "error_handling".to_string(),
                ],
                required_params: vec!["steps".to_string()],
                optional_params: vec![
                    "name".to_string(),
                    "timeout".to_string(),
                    "error_strategy".to_string(),
                ],
            },
        );

        // Register Conditional workflow
        workflow_types.insert(
            "conditional".to_string(),
            WorkflowInfo {
                workflow_type: "conditional".to_string(),
                description: "Execute different branches based on conditions".to_string(),
                features: vec![
                    "branching".to_string(),
                    "condition_evaluation".to_string(),
                    "default_branch".to_string(),
                ],
                required_params: vec!["condition".to_string(), "branches".to_string()],
                optional_params: vec![
                    "name".to_string(),
                    "default_branch".to_string(),
                    "evaluation_mode".to_string(),
                ],
            },
        );

        // Register Loop workflow
        workflow_types.insert(
            "loop".to_string(),
            WorkflowInfo {
                workflow_type: "loop".to_string(),
                description: "Execute steps repeatedly based on iteration criteria".to_string(),
                features: vec![
                    "collection_iteration".to_string(),
                    "range_iteration".to_string(),
                    "while_condition".to_string(),
                    "break_conditions".to_string(),
                    "result_aggregation".to_string(),
                ],
                required_params: vec!["iterator".to_string(), "body".to_string()],
                optional_params: vec![
                    "name".to_string(),
                    "max_iterations".to_string(),
                    "break_condition".to_string(),
                    "aggregation".to_string(),
                    "timeout".to_string(),
                ],
            },
        );

        // Register Parallel workflow
        workflow_types.insert(
            "parallel".to_string(),
            WorkflowInfo {
                workflow_type: "parallel".to_string(),
                description: "Execute multiple branches concurrently".to_string(),
                features: vec![
                    "concurrent_execution".to_string(),
                    "fork_join".to_string(),
                    "concurrency_limits".to_string(),
                    "fail_fast".to_string(),
                    "optional_branches".to_string(),
                ],
                required_params: vec!["branches".to_string()],
                optional_params: vec![
                    "name".to_string(),
                    "max_concurrency".to_string(),
                    "fail_fast".to_string(),
                    "timeout".to_string(),
                ],
            },
        );

        Self { workflow_types }
    }

    /// List all available workflow types
    pub fn list_workflow_types(&self) -> Vec<String> {
        self.workflow_types.keys().cloned().collect()
    }

    /// Get information about a specific workflow type
    pub fn get_workflow_info(&self, workflow_type: &str) -> Option<&WorkflowInfo> {
        self.workflow_types.get(workflow_type)
    }

    /// Check if a workflow type is available
    pub fn has_workflow_type(&self, workflow_type: &str) -> bool {
        self.workflow_types.contains_key(workflow_type)
    }

    /// Get all workflow type information
    pub fn get_all_workflow_info(&self) -> Vec<WorkflowInfo> {
        self.workflow_types.values().cloned().collect()
    }

    /// Get all workflow types with their info
    pub fn get_workflow_types(&self) -> Vec<(String, WorkflowInfo)> {
        self.workflow_types
            .iter()
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
}

impl Default for WorkflowDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

/// Factory for creating workflow instances
pub struct WorkflowFactory;

impl WorkflowFactory {
    /// Create a workflow instance based on type and parameters
    pub async fn create_workflow(
        workflow_type: &str,
        params: serde_json::Value,
    ) -> Result<Box<dyn WorkflowExecutor>> {
        match workflow_type {
            "sequential" => {
                let workflow = create_sequential_workflow(params).await?;
                Ok(Box::new(workflow))
            }
            "conditional" => {
                let workflow = create_conditional_workflow(params).await?;
                Ok(Box::new(workflow))
            }
            "loop" => {
                let workflow = create_loop_workflow(params).await?;
                Ok(Box::new(workflow))
            }
            "parallel" => {
                let workflow = create_parallel_workflow(params).await?;
                Ok(Box::new(workflow))
            }
            _ => Err(llmspell_core::LLMSpellError::Configuration {
                message: format!("Unknown workflow type: {}", workflow_type),
                source: None,
            }),
        }
    }
}

/// Trait for workflow execution through the bridge
#[async_trait::async_trait]
pub trait WorkflowExecutor: Send + Sync {
    /// Execute the workflow
    async fn execute(&self, input: serde_json::Value) -> Result<serde_json::Value>;

    /// Get workflow name
    fn name(&self) -> &str;

    /// Get workflow type
    fn workflow_type(&self) -> &str;
}

// Helper functions to create specific workflow types

async fn create_sequential_workflow(params: serde_json::Value) -> Result<impl WorkflowExecutor> {
    use llmspell_workflows::SequentialWorkflowBuilder;

    let name = params
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("sequential_workflow")
        .to_string();

    let steps = params
        .get("steps")
        .and_then(|v| v.as_array())
        .ok_or_else(|| llmspell_core::LLMSpellError::Configuration {
            message: "Sequential workflow requires 'steps' array".to_string(),
            source: None,
        })?;

    let mut builder = SequentialWorkflowBuilder::new(name.clone());

    // Add steps from params
    for step_json in steps {
        let step = parse_workflow_step(step_json)?;
        builder = builder.add_step(step);
    }

    let workflow = builder.build();
    Ok(SequentialWorkflowExecutor { workflow, name })
}

async fn create_conditional_workflow(params: serde_json::Value) -> Result<impl WorkflowExecutor> {
    use llmspell_workflows::{ConditionalBranch, ConditionalWorkflowBuilder};

    let name = params
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("conditional_workflow")
        .to_string();

    let mut builder = ConditionalWorkflowBuilder::new(name.clone());

    // Parse branches
    if let Some(branches) = params.get("branches").and_then(|v| v.as_object()) {
        for (branch_name, branch_data) in branches {
            let steps = branch_data
                .get("steps")
                .and_then(|v| v.as_array())
                .ok_or_else(|| llmspell_core::LLMSpellError::Configuration {
                    message: format!("Branch '{}' requires 'steps' array", branch_name),
                    source: None,
                })?;

            // Parse condition for the branch
            let condition = if let Some(condition_json) = branch_data.get("condition") {
                parse_condition(condition_json)?
            } else {
                // Default to always true for branches without explicit conditions
                llmspell_workflows::Condition::Always
            };

            let mut branch = ConditionalBranch::new(branch_name.clone(), condition);

            for step_json in steps {
                let step = parse_workflow_step(step_json)?;
                branch = branch.with_step(step);
            }

            builder = builder.add_branch(branch);
        }
    }

    let workflow = builder.build();
    Ok(ConditionalWorkflowExecutor { workflow, name })
}

async fn create_loop_workflow(params: serde_json::Value) -> Result<impl WorkflowExecutor> {
    use llmspell_workflows::{LoopIterator, LoopWorkflowBuilder};

    let name = params
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("loop_workflow")
        .to_string();

    let mut builder = LoopWorkflowBuilder::new(name.clone());

    // Parse iterator configuration
    if let Some(iterator_config) = params.get("iterator") {
        let iterator = parse_loop_iterator(iterator_config)?;
        // Set iterator based on its type
        builder = match iterator {
            LoopIterator::Collection { values } => builder.with_collection(values),
            LoopIterator::Range { start, end, step } => builder.with_range(start, end, step),
            LoopIterator::WhileCondition {
                condition,
                max_iterations,
            } => builder.with_while_condition(condition, max_iterations),
        };
    }

    // Parse body steps
    if let Some(body_steps) = params.get("body").and_then(|v| v.as_array()) {
        for step_json in body_steps {
            let step = parse_workflow_step(step_json)?;
            builder = builder.add_step(step);
        }
    }

    let workflow = builder.build()?;
    Ok(LoopWorkflowExecutor { workflow, name })
}

async fn create_parallel_workflow(params: serde_json::Value) -> Result<impl WorkflowExecutor> {
    use llmspell_workflows::{ParallelBranch, ParallelWorkflowBuilder};

    let name = params
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("parallel_workflow")
        .to_string();

    let mut builder = ParallelWorkflowBuilder::new(name.clone());

    // Parse branches
    if let Some(branches) = params.get("branches").and_then(|v| v.as_array()) {
        for branch_json in branches {
            let branch_name = branch_json
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("branch")
                .to_string();

            let mut branch = ParallelBranch::new(branch_name);

            if let Some(desc) = branch_json.get("description").and_then(|v| v.as_str()) {
                branch = branch.with_description(desc.to_string());
            }

            if let Some(optional) = branch_json.get("optional").and_then(|v| v.as_bool()) {
                if optional {
                    branch = branch.optional();
                }
            }

            if let Some(steps) = branch_json.get("steps").and_then(|v| v.as_array()) {
                for step_json in steps {
                    let step = parse_workflow_step(step_json)?;
                    branch = branch.add_step(step);
                }
            }

            builder = builder.add_branch(branch);
        }
    }

    // Parse config options
    if let Some(max_concurrency) = params.get("max_concurrency").and_then(|v| v.as_u64()) {
        builder = builder.with_max_concurrency(max_concurrency as usize);
    }

    if let Some(fail_fast) = params.get("fail_fast").and_then(|v| v.as_bool()) {
        builder = builder.fail_fast(fail_fast);
    }

    let workflow = builder.build()?;
    Ok(ParallelWorkflowExecutor { workflow, name })
}

fn parse_workflow_step(step_json: &serde_json::Value) -> Result<llmspell_workflows::WorkflowStep> {
    use llmspell_core::ComponentId;
    use llmspell_workflows::{StepType, WorkflowStep};

    let name = step_json
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("step")
        .to_string();

    let step_type = if let Some(tool_name) = step_json.get("tool").and_then(|v| v.as_str()) {
        let params = step_json
            .get("parameters")
            .cloned()
            .unwrap_or(serde_json::json!({}));
        StepType::Tool {
            tool_name: tool_name.to_string(),
            parameters: params,
        }
    } else if let Some(agent_id) = step_json.get("agent").and_then(|v| v.as_str()) {
        let input = step_json
            .get("input")
            .cloned()
            .unwrap_or(serde_json::json!({}));
        StepType::Agent {
            agent_id: ComponentId::from_name(agent_id),
            input: input.to_string(),
        }
    } else if let Some(func_name) = step_json.get("function").and_then(|v| v.as_str()) {
        let params = step_json
            .get("parameters")
            .cloned()
            .unwrap_or(serde_json::json!({}));
        StepType::Custom {
            function_name: func_name.to_string(),
            parameters: params,
        }
    } else {
        return Err(llmspell_core::LLMSpellError::Configuration {
            message: "Step must have 'tool', 'agent', or 'function' field".to_string(),
            source: None,
        });
    };

    Ok(WorkflowStep::new(name, step_type))
}

fn parse_condition(condition_json: &serde_json::Value) -> Result<llmspell_workflows::Condition> {
    use llmspell_workflows::Condition;

    if let Some(condition_type) = condition_json.get("type").and_then(|v| v.as_str()) {
        match condition_type {
            "always" => Ok(Condition::Always),
            "never" => Ok(Condition::Never),
            "shared_data_equals" => {
                let key = condition_json
                    .get("key")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| llmspell_core::LLMSpellError::Configuration {
                        message: "shared_data_equals condition requires 'key'".to_string(),
                        source: None,
                    })?;
                let expected_value = condition_json
                    .get("expected_value")
                    .cloned()
                    .unwrap_or(serde_json::Value::Null);
                Ok(Condition::SharedDataEquals {
                    key: key.to_string(),
                    expected_value,
                })
            }
            "shared_data_exists" => {
                let key = condition_json
                    .get("key")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| llmspell_core::LLMSpellError::Configuration {
                        message: "shared_data_exists condition requires 'key'".to_string(),
                        source: None,
                    })?;
                Ok(Condition::SharedDataExists {
                    key: key.to_string(),
                })
            }
            _ => Ok(Condition::Always), // Default to always true for unknown types
        }
    } else {
        // If no type specified, default to always true
        Ok(Condition::Always)
    }
}

fn parse_loop_iterator(config: &serde_json::Value) -> Result<llmspell_workflows::LoopIterator> {
    use llmspell_workflows::LoopIterator;

    if let Some(collection) = config.get("collection").and_then(|v| v.as_array()) {
        Ok(LoopIterator::Collection {
            values: collection.to_vec(),
        })
    } else if let Some(range) = config.get("range").and_then(|v| v.as_object()) {
        let start = range.get("start").and_then(|v| v.as_i64()).unwrap_or(0);
        let end = range.get("end").and_then(|v| v.as_i64()).unwrap_or(10);
        let step = range.get("step").and_then(|v| v.as_i64()).unwrap_or(1);
        Ok(LoopIterator::Range { start, end, step })
    } else if let Some(condition) = config.get("while_condition").and_then(|v| v.as_str()) {
        Ok(LoopIterator::WhileCondition {
            condition: condition.to_string(),
            max_iterations: config
                .get("max_iterations")
                .and_then(|v| v.as_u64())
                .unwrap_or(100) as usize,
        })
    } else {
        Err(llmspell_core::LLMSpellError::Configuration {
            message: "Iterator must have 'collection', 'range', or 'while_condition'".to_string(),
            source: None,
        })
    }
}

// Executor implementations for each workflow type

struct SequentialWorkflowExecutor {
    workflow: llmspell_workflows::SequentialWorkflow,
    name: String,
}

#[async_trait::async_trait]
impl WorkflowExecutor for SequentialWorkflowExecutor {
    async fn execute(&self, _input: serde_json::Value) -> Result<serde_json::Value> {
        let result = self.workflow.execute().await?;
        let script_result = crate::workflow_results::transform_sequential_result(&result);
        Ok(serde_json::to_value(&script_result)?)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn workflow_type(&self) -> &str {
        "sequential"
    }
}

struct ConditionalWorkflowExecutor {
    workflow: llmspell_workflows::ConditionalWorkflow,
    name: String,
}

#[async_trait::async_trait]
impl WorkflowExecutor for ConditionalWorkflowExecutor {
    async fn execute(&self, _input: serde_json::Value) -> Result<serde_json::Value> {
        let result = self.workflow.execute().await?;
        let script_result = crate::workflow_results::transform_conditional_result(&result);
        Ok(serde_json::to_value(&script_result)?)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn workflow_type(&self) -> &str {
        "conditional"
    }
}

struct LoopWorkflowExecutor {
    workflow: llmspell_workflows::LoopWorkflow,
    name: String,
}

#[async_trait::async_trait]
impl WorkflowExecutor for LoopWorkflowExecutor {
    async fn execute(&self, _input: serde_json::Value) -> Result<serde_json::Value> {
        let result = self.workflow.execute().await?;
        let script_result = crate::workflow_results::transform_loop_result(&result);
        Ok(serde_json::to_value(&script_result)?)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn workflow_type(&self) -> &str {
        "loop"
    }
}

struct ParallelWorkflowExecutor {
    workflow: llmspell_workflows::ParallelWorkflow,
    name: String,
}

#[async_trait::async_trait]
impl WorkflowExecutor for ParallelWorkflowExecutor {
    async fn execute(&self, _input: serde_json::Value) -> Result<serde_json::Value> {
        let result = self.workflow.execute().await?;
        let script_result = crate::workflow_results::transform_parallel_result(&result);
        Ok(serde_json::to_value(&script_result)?)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn workflow_type(&self) -> &str {
        "parallel"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workflow_discovery() {
        let discovery = WorkflowDiscovery::new();

        // Test listing workflow types
        let types = discovery.list_workflow_types();
        assert_eq!(types.len(), 4);
        assert!(types.contains(&"sequential".to_string()));
        assert!(types.contains(&"conditional".to_string()));
        assert!(types.contains(&"loop".to_string()));
        assert!(types.contains(&"parallel".to_string()));

        // Test getting workflow info
        let seq_info = discovery.get_workflow_info("sequential").unwrap();
        assert_eq!(seq_info.workflow_type, "sequential");
        assert!(seq_info.required_params.contains(&"steps".to_string()));

        // Test checking workflow existence
        assert!(discovery.has_workflow_type("parallel"));
        assert!(!discovery.has_workflow_type("unknown"));
    }
}
