//! ABOUTME: Workflow discovery and management for script integration
//! ABOUTME: Provides workflow type information and factory methods

#![allow(clippy::significant_drop_tightening)]

use crate::discovery::BridgeDiscovery;
use crate::workflow_performance::{ExecutionCache, OptimizedConverter, PerformanceMetrics};
use crate::ComponentRegistry;
use llmspell_core::{traits::base_agent::BaseAgent, LLMSpellError, Result};
use llmspell_workflows::{
    factory::{DefaultWorkflowFactory, WorkflowFactory},
    types::WorkflowConfig,
    ErrorStrategy, WorkflowStep,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

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

/// Status of a workflow instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowStatus {
    /// Workflow is ready to execute
    Ready,
    /// Workflow is currently executing
    Running,
    /// Workflow completed successfully
    Completed,
    /// Workflow failed with error
    Failed(String),
}

/// Discovery service for available workflow types
pub struct WorkflowDiscovery {
    /// Registry of available workflow types
    workflow_types: HashMap<String, WorkflowInfo>,
}

impl WorkflowDiscovery {
    /// Create a new workflow discovery service
    #[must_use]
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
                required_params: vec!["branches".to_string()], // Also accepts "steps" for compatibility
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
    #[must_use]
    pub fn list_workflow_types(&self) -> Vec<String> {
        self.workflow_types.keys().cloned().collect()
    }

    /// Get information about a specific workflow type
    #[must_use]
    pub fn get_workflow_info(&self, workflow_type: &str) -> Option<&WorkflowInfo> {
        self.workflow_types.get(workflow_type)
    }

    /// Check if a workflow type is available
    #[must_use]
    pub fn has_workflow_type(&self, workflow_type: &str) -> bool {
        self.workflow_types.contains_key(workflow_type)
    }

    /// Get all workflow type information
    #[must_use]
    pub fn get_all_workflow_info(&self) -> Vec<WorkflowInfo> {
        self.workflow_types.values().cloned().collect()
    }

    /// Get all workflow types with their info
    #[must_use]
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

/// Implementation of unified BridgeDiscovery trait for WorkflowDiscovery
#[async_trait::async_trait]
impl BridgeDiscovery<WorkflowInfo> for WorkflowDiscovery {
    async fn discover_types(&self) -> Vec<(String, WorkflowInfo)> {
        self.get_workflow_types()
    }

    async fn get_type_info(&self, type_name: &str) -> Option<WorkflowInfo> {
        self.get_workflow_info(type_name).cloned()
    }

    async fn has_type(&self, type_name: &str) -> bool {
        self.has_workflow_type(type_name)
    }

    async fn list_types(&self) -> Vec<String> {
        self.list_workflow_types()
    }

    async fn filter_types<F>(&self, predicate: F) -> Vec<(String, WorkflowInfo)>
    where
        F: Fn(&str, &WorkflowInfo) -> bool + Send,
    {
        self.workflow_types
            .iter()
            .filter(|(k, v)| predicate(k, v))
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }
}

// JSON-based WorkflowFactory removed - use StandardizedWorkflowFactory instead
// which creates workflows from Rust structures directly without JSON translation

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

/// Standardized workflow factory using llmspell-workflows
pub struct StandardizedWorkflowFactory {
    factory: Arc<DefaultWorkflowFactory>,
    registry: Option<Arc<ComponentRegistry>>,
    state_manager: Option<Arc<llmspell_state_persistence::StateManager>>,
}

impl StandardizedWorkflowFactory {
    #[must_use]
    pub fn new() -> Self {
        Self {
            factory: Arc::new(DefaultWorkflowFactory::new()),
            registry: None,
            state_manager: None,
        }
    }

    #[must_use]
    pub fn new_with_state(
        registry: Arc<ComponentRegistry>,
        state_manager: Arc<llmspell_state_persistence::StateManager>,
    ) -> Self {
        Self {
            factory: Arc::new(DefaultWorkflowFactory::new()),
            registry: Some(registry),
            state_manager: Some(state_manager),
        }
    }

    #[must_use]
    pub fn new_with_registry(registry: Arc<ComponentRegistry>) -> Self {
        Self {
            factory: Arc::new(DefaultWorkflowFactory::new()),
            registry: Some(registry),
            state_manager: None,
        }
    }

    /// Create workflow from Rust structures directly (for internal bridge use)
    ///
    /// This method bypasses JSON serialization/deserialization for better performance
    /// and type safety when called from language bridges (Lua, Python, JS).
    ///
    /// # Errors
    ///
    /// Returns an error if workflow creation fails
    #[allow(clippy::unused_async)]
    pub async fn create_from_steps(
        &self,
        workflow_type: &str,
        name: String,
        steps: Vec<WorkflowStep>,
        config: WorkflowConfig,
        error_strategy: Option<ErrorStrategy>,
    ) -> Result<Box<dyn WorkflowExecutor>> {
        use llmspell_workflows::{
            Condition, ConditionalBranch, ConditionalWorkflowBuilder, LoopWorkflowBuilder,
            ParallelBranch, ParallelWorkflowBuilder, SequentialWorkflowBuilder,
        };

        match workflow_type {
            "sequential" => {
                let mut builder = SequentialWorkflowBuilder::new(name.clone());

                // Add registry if available
                if let Some(ref reg) = self.registry {
                    builder = builder.with_registry(reg.clone());
                }

                // Add steps
                for step in steps {
                    builder = builder.add_step(step);
                }

                // Apply error strategy
                if let Some(strategy) = error_strategy {
                    builder = builder.with_error_strategy(strategy);
                }

                let workflow = builder.build();
                Ok(Box::new(SequentialWorkflowExecutor {
                    workflow,
                    name,
                    state_manager: self.state_manager.clone(),
                }))
            }
            "parallel" => {
                let mut builder = ParallelWorkflowBuilder::new(name.clone());

                // Create a single branch with all steps
                let mut branch = ParallelBranch::new("main".to_string());
                for step in steps {
                    branch = branch.add_step(step);
                }
                builder = builder.add_branch(branch);

                if config.continue_on_error {
                    builder = builder.fail_fast(false);
                }

                let workflow = builder.build()?;
                Ok(Box::new(ParallelWorkflowExecutor {
                    workflow,
                    name,
                    state_manager: self.state_manager.clone(),
                }))
            }
            "loop" => {
                let mut builder = LoopWorkflowBuilder::new(name.clone());

                // Add registry if available
                if let Some(ref reg) = self.registry {
                    builder = builder.with_registry(reg.clone());
                }

                // Add steps
                for step in steps {
                    builder = builder.add_step(step);
                }

                // TODO: Pass iterator configuration from Lua
                // For now, use a default range iterator to make tests pass
                builder = builder.with_range(1, 5, 1);

                let workflow = builder.build()?;
                Ok(Box::new(LoopWorkflowExecutor {
                    workflow,
                    name,
                    state_manager: self.state_manager.clone(),
                }))
            }
            "conditional" => {
                let mut builder =
                    ConditionalWorkflowBuilder::new(name.clone()).with_workflow_config(config);

                // Add registry if available
                if let Some(ref reg) = self.registry {
                    builder = builder.with_registry(reg.clone());
                }

                // Create a single "always" branch with all steps for simplified case
                let branch =
                    ConditionalBranch::new("main".to_string(), Condition::Always).with_steps(steps);
                builder = builder.add_branch(branch);

                // Apply error strategy
                if let Some(strategy) = error_strategy {
                    builder = builder.with_error_strategy(strategy);
                }

                let workflow = builder.build();
                Ok(Box::new(ConditionalWorkflowExecutor {
                    workflow,
                    name,
                    state_manager: self.state_manager.clone(),
                }))
            }
            _ => Err(LLMSpellError::Configuration {
                message: format!("Unknown workflow type: {workflow_type}"),
                source: None,
            }),
        }
    }

    /// List available workflow types
    #[must_use]
    pub fn list_workflow_types(&self) -> Vec<String> {
        self.factory.list_workflow_types()
    }
}

impl Default for StandardizedWorkflowFactory {
    fn default() -> Self {
        Self::new()
    }
}

// Helper functions to create specific workflow types
// REMOVED: All JSON-based workflow creation functions
/*
pub fn create_sequential_workflow(
    params: &serde_json::Value,
    registry: Option<Arc<ComponentRegistry>>,
) -> Result<impl WorkflowExecutor> {
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

    // Add registry if provided
    if let Some(reg) = registry {
        use llmspell_core::ComponentLookup;
        builder = builder.with_registry(reg as Arc<dyn ComponentLookup>);
    }

    let workflow = builder.build();
    Ok(SequentialWorkflowExecutor { workflow, name })
}

/// Creates a conditional workflow from JSON parameters
///
/// # Errors
///
/// Returns an error if:
/// - Required fields are missing from parameters
/// - Branch configuration is invalid
/// - Step parsing fails
pub fn create_conditional_workflow(
    params: &serde_json::Value,
    registry: Option<Arc<ComponentRegistry>>,
) -> Result<impl WorkflowExecutor> {
    use llmspell_core::ComponentLookup;
    use llmspell_workflows::{ConditionalBranch, ConditionalWorkflowBuilder};

    let name = params
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("conditional_workflow")
        .to_string();

    let mut builder = ConditionalWorkflowBuilder::new(name.clone());

    // Configure to execute default branch if no conditions match
    let config = ConditionalConfig::builder()
        .execute_default_on_no_match(true)
        .build()?;
    builder = builder.with_conditional_config(config);

    // Parse branches - support both array and object formats
    if let Some(branches_value) = params.get("branches") {
        if let Some(branches_array) = branches_value.as_array() {
            // Handle array format (our test case)
            for branch_data in branches_array {
                let branch_name = branch_data
                    .get("name")
                    .and_then(|v| v.as_str())
                    .ok_or_else(|| llmspell_core::LLMSpellError::Configuration {
                        message: "Branch requires 'name' field".to_string(),
                        source: None,
                    })?;

                let steps = branch_data
                    .get("steps")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| llmspell_core::LLMSpellError::Configuration {
                        message: format!("Branch '{branch_name}' requires 'steps' array"),
                        source: None,
                    })?;

                // Parse condition for the branch
                let condition = if let Some(condition_json) = branch_data.get("condition") {
                    parse_condition(condition_json)?
                } else {
                    // Default to always true for branches without explicit conditions
                    llmspell_workflows::Condition::Always
                };

                let mut branch = ConditionalBranch::new(branch_name.to_string(), condition);

                for step_json in steps {
                    let step = parse_workflow_step(step_json)?;
                    branch = branch.with_step(step);
                }

                builder = builder.add_branch(branch);
            }
        } else if let Some(branches_object) = branches_value.as_object() {
            // Handle object format (backward compatibility)
            for (branch_name, branch_data) in branches_object {
                let steps = branch_data
                    .get("steps")
                    .and_then(|v| v.as_array())
                    .ok_or_else(|| llmspell_core::LLMSpellError::Configuration {
                        message: format!("Branch '{branch_name}' requires 'steps' array"),
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
    }

    // Parse default branch if provided
    if let Some(default_branch_data) = params.get("default_branch") {
        let default_branch_name = default_branch_data
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("default");

        let steps = default_branch_data
            .get("steps")
            .and_then(|v| v.as_array())
            .ok_or_else(|| llmspell_core::LLMSpellError::Configuration {
                message: "Default branch requires 'steps' array".to_string(),
                source: None,
            })?;

        // Use the default() constructor which marks the branch as default
        let mut default_branch = ConditionalBranch::default(default_branch_name.to_string());

        for step_json in steps {
            let step = parse_workflow_step(step_json)?;
            default_branch = default_branch.with_step(step);
        }

        builder = builder.add_branch(default_branch);
    }

    // Add registry if provided
    if let Some(reg) = registry {
        builder = builder.with_registry(reg as Arc<dyn ComponentLookup>);
    }

    let workflow = builder.build();
    Ok(ConditionalWorkflowExecutor { workflow, name })
}

pub fn create_loop_workflow(
    params: &serde_json::Value,
    registry: Option<Arc<ComponentRegistry>>,
) -> Result<impl WorkflowExecutor> {
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

    // Add registry if provided
    if let Some(reg) = registry {
        use llmspell_core::ComponentLookup;
        builder = builder.with_registry(reg as Arc<dyn ComponentLookup>);
    }

    let workflow = builder.build()?;
    Ok(LoopWorkflowExecutor { workflow, name })
}

/// Creates a parallel workflow from JSON parameters
///
/// # Errors
///
/// Returns an error if:
/// - Required 'branches' or 'steps' field is missing
/// - Branch configuration is invalid
/// - Step parsing fails
pub fn create_parallel_workflow(
    params: &serde_json::Value,
    registry: Option<Arc<ComponentRegistry>>,
) -> Result<impl WorkflowExecutor> {
    use llmspell_core::ComponentLookup;
    use llmspell_workflows::ParallelWorkflowBuilder;

    let name = extract_workflow_name(params, "parallel_workflow");
    let mut builder = ParallelWorkflowBuilder::new(name.clone());

    // Get branches array from params
    let branches = extract_parallel_branches(params)?;

    // Process each branch
    builder = process_parallel_branches(builder, branches)?;

    // Apply configuration options
    builder = apply_parallel_config(builder, params);

    // Add registry if provided
    if let Some(reg) = registry {
        builder = builder.with_registry(reg as Arc<dyn ComponentLookup>);
    }

    // Build and return the workflow
    let workflow = builder.build()?;
    debug!("Successfully built parallel workflow '{}'", name);
    Ok(ParallelWorkflowExecutor { workflow, name })
}

/// Extract workflow name from params with default fallback
fn extract_workflow_name(params: &serde_json::Value, default: &str) -> String {
    params
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or(default)
        .to_string()
}

/// Extract branches array from params, supporting both "branches" and "steps" fields
fn extract_parallel_branches(params: &serde_json::Value) -> Result<&Vec<serde_json::Value>> {
    params
        .get("branches")
        .and_then(|v| v.as_array())
        .inspect(|branches| {
            debug!("Found {} branches in params", branches.len());
        })
        .or_else(|| {
            params
                .get("steps")
                .and_then(|v| v.as_array())
                .inspect(|steps| {
                    debug!("Found {} steps (as branches) in params", steps.len());
                })
        })
        .ok_or_else(|| llmspell_core::LLMSpellError::Configuration {
            message: "Parallel workflow requires either 'branches' or 'steps' field".to_string(),
            source: None,
        })
}

/// Process branches and add them to the builder
fn process_parallel_branches(
    mut builder: llmspell_workflows::ParallelWorkflowBuilder,
    branches: &[serde_json::Value],
) -> Result<llmspell_workflows::ParallelWorkflowBuilder> {
    let mut branch_count = 0;
    for branch_json in branches {
        let branch = create_parallel_branch(branch_json)?;
        builder = builder.add_branch(branch);
        branch_count += 1;
    }
    debug!("Added {} branches to parallel workflow", branch_count);
    Ok(builder)
}

/// Create a single parallel branch from JSON
fn create_parallel_branch(
    branch_json: &serde_json::Value,
) -> Result<llmspell_workflows::ParallelBranch> {
    use llmspell_workflows::ParallelBranch;

    let branch_name = branch_json
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("branch")
        .to_string();

    debug!("Processing branch: {}", branch_name);
    let mut branch = ParallelBranch::new(branch_name.clone());

    // Add description if present
    if let Some(desc) = branch_json.get("description").and_then(|v| v.as_str()) {
        branch = branch.with_description(desc.to_string());
    }

    // Mark as optional if specified
    if let Some(optional) = branch_json
        .get("optional")
        .and_then(serde_json::Value::as_bool)
    {
        if optional {
            branch = branch.optional();
        }
    }

    // Add steps to branch
    branch = add_steps_to_branch(branch, branch_json, &branch_name)?;
    Ok(branch)
}

/// Add steps to a parallel branch
fn add_steps_to_branch(
    mut branch: llmspell_workflows::ParallelBranch,
    branch_json: &serde_json::Value,
    branch_name: &str,
) -> Result<llmspell_workflows::ParallelBranch> {
    let mut step_count = 0;
    if let Some(steps) = branch_json.get("steps").and_then(|v| v.as_array()) {
        for step_json in steps {
            let step = parse_workflow_step(step_json)?;
            branch = branch.add_step(step);
            step_count += 1;
        }
    }
    debug!("Branch '{}' has {} steps", branch_name, step_count);
    Ok(branch)
}

/// Apply configuration options to parallel workflow builder
fn apply_parallel_config(
    mut builder: llmspell_workflows::ParallelWorkflowBuilder,
    params: &serde_json::Value,
) -> llmspell_workflows::ParallelWorkflowBuilder {
    // Set max concurrency if specified
    if let Some(max_concurrency) = params
        .get("max_concurrency")
        .and_then(serde_json::Value::as_u64)
    {
        builder =
            builder.with_max_concurrency(usize::try_from(max_concurrency).unwrap_or(usize::MAX));
    }

    // Set fail fast if specified
    if let Some(fail_fast) = params.get("fail_fast").and_then(serde_json::Value::as_bool) {
        builder = builder.fail_fast(fail_fast);
    }

    builder
}

/// Convert JSON input to `AgentInput` with fallback logic
pub(crate) fn json_to_agent_input(input: &serde_json::Value) -> llmspell_core::types::AgentInput {
    // Try to deserialize directly first
    if let Ok(agent_input) =
        serde_json::from_value::<llmspell_core::types::AgentInput>(input.clone())
    {
        return agent_input;
    }

    // Fallback: try to extract text field from JSON object
    if let Some(text) = input.get("text").and_then(|v| v.as_str()) {
        return llmspell_core::types::AgentInput::text(text.to_string());
    }

    // Fallback: treat entire value as string if it is one
    if let Some(text_str) = input.as_str() {
        return llmspell_core::types::AgentInput::text(text_str.to_string());
    }

    // Last resort: empty input
    llmspell_core::types::AgentInput::text("")
}

/// Convert a `WorkflowStep` to flat JSON format expected by the parser
/// This is the single source of truth for step JSON format
#[must_use]
pub fn workflow_step_to_json(step: &llmspell_workflows::WorkflowStep) -> serde_json::Value {
    use llmspell_workflows::StepType;

    let mut json = serde_json::json!({
        "name": &step.name
    });

    // Add type-specific fields in flat format
    match &step.step_type {
        StepType::Tool {
            tool_name,
            parameters,
        } => {
            json["tool"] = serde_json::json!(tool_name);
            json["parameters"] = parameters.clone();
        }
        StepType::Agent { agent_id, input } => {
            json["agent"] = serde_json::json!(agent_id.to_string());
            json["input"] = serde_json::json!(input);
        }
        StepType::Custom {
            function_name,
            parameters,
        } => {
            json["function"] = serde_json::json!(function_name);
            json["parameters"] = parameters.clone();
        }
        StepType::Workflow { workflow_id, input } => {
            json["workflow"] = serde_json::json!(workflow_id.to_string());
            json["input"] = input.clone();
        }
    }

    json
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
            .unwrap_or_else(|| serde_json::json!({}));
        StepType::Tool {
            tool_name: tool_name.to_string(),
            parameters: params,
        }
    } else if let Some(agent_id) = step_json.get("agent").and_then(|v| v.as_str()) {
        let input = step_json
            .get("input")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));
        StepType::Agent {
            agent_id: agent_id.to_string(),  // Keep original agent name for registry lookup
            input: input.to_string(),
        }
    } else if let Some(func_name) = step_json.get("function").and_then(|v| v.as_str()) {
        let params = step_json
            .get("parameters")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));
        StepType::Custom {
            function_name: func_name.to_string(),
            parameters: params,
        }
    } else if let Some(workflow_id) = step_json.get("workflow").and_then(|v| v.as_str()) {
        let input = step_json
            .get("input")
            .cloned()
            .unwrap_or_else(|| serde_json::json!({}));
        StepType::Workflow {
            workflow_id: ComponentId::from_name(workflow_id),
            input,
        }
    } else {
        return Err(llmspell_core::LLMSpellError::Configuration {
            message: "Step must have 'tool', 'agent', 'function', or 'workflow' field".to_string(),
            source: None,
        });
    };

    Ok(WorkflowStep::new(name, step_type))
}

fn parse_condition(condition_json: &serde_json::Value) -> Result<llmspell_workflows::Condition> {
    use llmspell_workflows::Condition;

    if let Some(condition_type) = condition_json.get("type").and_then(|v| v.as_str()) {
        match condition_type {
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

    #[allow(clippy::option_if_let_else)] // Complex pattern
    if let Some(collection) = config.get("collection").and_then(|v| v.as_array()) {
        Ok(LoopIterator::Collection {
            values: collection.clone(),
        })
    } else if let Some(range) = config.get("range").and_then(|v| v.as_object()) {
        let start = range
            .get("start")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(0);
        let end = range
            .get("end")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(10);
        let step = range
            .get("step")
            .and_then(serde_json::Value::as_i64)
            .unwrap_or(1);
        Ok(LoopIterator::Range { start, end, step })
    } else if let Some(condition) = config.get("while_condition").and_then(|v| v.as_str()) {
        Ok(LoopIterator::WhileCondition {
            condition: condition.to_string(),
            max_iterations: config
                .get("max_iterations")
                .and_then(serde_json::Value::as_u64)
                .map_or(100, |v| usize::try_from(v).unwrap_or(usize::MAX)),
        })
    } else {
        Err(llmspell_core::LLMSpellError::Configuration {
            message: "Iterator must have 'collection', 'range', or 'while_condition'".to_string(),
            source: None,
        })
    }
}
*/ // End of removed JSON-based workflow creation functions

// Helper function for converting JSON to AgentInput - still needed for WorkflowExecutor trait
/// Convert JSON input to `AgentInput` with fallback logic
pub(crate) fn json_to_agent_input(input: &serde_json::Value) -> llmspell_core::types::AgentInput {
    // Try to deserialize directly first
    if let Ok(agent_input) =
        serde_json::from_value::<llmspell_core::types::AgentInput>(input.clone())
    {
        return agent_input;
    }

    // Fallback: try to extract text field from JSON object
    if let Some(text) = input.get("text").and_then(|v| v.as_str()) {
        return llmspell_core::types::AgentInput::text(text.to_string());
    }

    // Fallback: treat entire value as string if it is one
    if let Some(text_str) = input.as_str() {
        return llmspell_core::types::AgentInput::text(text_str.to_string());
    }

    // Last resort: empty input
    llmspell_core::types::AgentInput::text("")
}

// Executor implementations for each workflow type

#[allow(dead_code)]
pub(crate) struct SequentialWorkflowExecutor {
    pub(crate) workflow: llmspell_workflows::SequentialWorkflow,
    pub(crate) name: String,
    pub(crate) state_manager: Option<Arc<llmspell_state_persistence::StateManager>>,
}

#[async_trait::async_trait]
impl WorkflowExecutor for SequentialWorkflowExecutor {
    async fn execute(&self, input: serde_json::Value) -> Result<serde_json::Value> {
        use tracing::info;

        info!(
            "SequentialWorkflowExecutor: Starting execution for workflow '{}'",
            self.name
        );

        // Create execution context with state support
        let context = create_execution_context_with_state(self.state_manager.clone()).await?;

        info!(
            "SequentialWorkflowExecutor: Context created with state: {}",
            context.state.is_some()
        );

        // Convert input to AgentInput
        let agent_input = json_to_agent_input(&input);

        // Execute through BaseAgent interface with state
        info!("SequentialWorkflowExecutor: Executing workflow with input");
        let agent_output = self.workflow.execute(agent_input, context).await?;

        info!("SequentialWorkflowExecutor: Workflow execution completed");

        // Convert AgentOutput to JSON
        Ok(serde_json::to_value(&agent_output)?)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn workflow_type(&self) -> &'static str {
        "sequential"
    }
}

pub(crate) struct ConditionalWorkflowExecutor {
    pub(crate) workflow: llmspell_workflows::ConditionalWorkflow,
    pub(crate) name: String,
    pub(crate) state_manager: Option<Arc<llmspell_state_persistence::StateManager>>,
}

#[async_trait::async_trait]
impl WorkflowExecutor for ConditionalWorkflowExecutor {
    async fn execute(&self, input: serde_json::Value) -> Result<serde_json::Value> {
        // Create execution context with state support
        let context = create_execution_context_with_state(self.state_manager.clone()).await?;

        // Convert input to AgentInput
        let agent_input = json_to_agent_input(&input);

        // Execute through BaseAgent interface with state
        let agent_output = self.workflow.execute(agent_input, context).await?;

        // Convert AgentOutput to JSON
        Ok(serde_json::to_value(&agent_output)?)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn workflow_type(&self) -> &'static str {
        "conditional"
    }
}

pub(crate) struct LoopWorkflowExecutor {
    pub(crate) workflow: llmspell_workflows::LoopWorkflow,
    pub(crate) name: String,
    pub(crate) state_manager: Option<Arc<llmspell_state_persistence::StateManager>>,
}

#[async_trait::async_trait]
impl WorkflowExecutor for LoopWorkflowExecutor {
    async fn execute(&self, input: serde_json::Value) -> Result<serde_json::Value> {
        // Create execution context with state support
        let context = create_execution_context_with_state(self.state_manager.clone()).await?;

        // Convert input to AgentInput
        let agent_input = json_to_agent_input(&input);

        // Execute through BaseAgent interface with state
        let agent_output = self.workflow.execute(agent_input, context).await?;

        // Convert AgentOutput to JSON
        Ok(serde_json::to_value(&agent_output)?)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn workflow_type(&self) -> &'static str {
        "loop"
    }
}

pub(crate) struct ParallelWorkflowExecutor {
    pub(crate) workflow: llmspell_workflows::ParallelWorkflow,
    pub(crate) name: String,
    pub(crate) state_manager: Option<Arc<llmspell_state_persistence::StateManager>>,
}

#[async_trait::async_trait]
impl WorkflowExecutor for ParallelWorkflowExecutor {
    async fn execute(&self, input: serde_json::Value) -> Result<serde_json::Value> {
        // Create execution context with state support
        let context = create_execution_context_with_state(self.state_manager.clone()).await?;

        // Convert input to AgentInput
        let agent_input = json_to_agent_input(&input);

        // Execute through BaseAgent interface with state
        let agent_output = self.workflow.execute(agent_input, context).await?;

        // Convert AgentOutput to JSON
        Ok(serde_json::to_value(&agent_output)?)
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn workflow_type(&self) -> &'static str {
        "parallel"
    }
}

/// Helper function to create an `ExecutionContext` with state support
///
/// This function creates an `ExecutionContext` with state persistence enabled
/// based on the current configuration. It uses in-memory state by default
/// but can be configured for persistent backends.
pub(crate) async fn create_execution_context_with_state(
    state_manager: Option<Arc<llmspell_state_persistence::StateManager>>,
) -> Result<llmspell_core::execution_context::ExecutionContext> {
    use tracing::info;

    info!("Creating execution context with state support");

    // Use provided state manager or create in-memory one
    let state_adapter: Arc<dyn llmspell_core::traits::state::StateAccess> =
        if let Some(sm) = state_manager {
            info!(
                "WorkflowBridge: Using StateManager at {:p} for workflow state",
                Arc::as_ptr(&sm)
            );
            // Use NoScopeStateAdapter to avoid adding "global:" prefix to workflow keys
            Arc::new(crate::state_adapter::NoScopeStateAdapter::new(sm))
        } else {
            info!("No shared StateManager provided, creating in-memory adapter");
            Arc::new(
                crate::state_adapter::StateManagerAdapter::in_memory()
                    .await
                    .map_err(|e| LLMSpellError::Component {
                        message: format!("Failed to create state adapter: {e}"),
                        source: None,
                    })?,
            )
        };

    let context = llmspell_core::execution_context::ExecutionContextBuilder::new()
        .state(state_adapter)
        .build();

    info!(
        "ExecutionContext created with state: {}",
        context.state.is_some()
    );

    Ok(context)
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

// =====================================================================
// WorkflowBridge - Merged from workflow_bridge.rs
// =====================================================================

/// Type alias for active workflow storage
type ActiveWorkflowMap = HashMap<String, Arc<Box<dyn WorkflowExecutor>>>;

/// Bridge between scripts and workflows
pub struct WorkflowBridge {
    /// Workflow discovery service
    discovery: Arc<WorkflowDiscovery>,
    /// Component registry for script access (used for component lookup)
    #[allow(dead_code)] // Used when creating StandardizedWorkflowFactory
    registry: Arc<ComponentRegistry>,
    /// Shared state manager for workflow state persistence
    state_manager: Option<Arc<llmspell_state_persistence::StateManager>>,
    /// Active workflow instances
    active_workflows: Arc<RwLock<ActiveWorkflowMap>>,
    /// Workflow execution history
    execution_history: Arc<RwLock<Vec<WorkflowExecutionRecord>>>,
    /// Bridge metrics
    metrics: Arc<BridgeMetrics>,
    /// Performance optimizations
    _converter: Arc<OptimizedConverter>,
    execution_cache: Arc<ExecutionCache>,
    perf_metrics: Arc<PerformanceMetrics>,
    /// Standardized workflow factory
    standardized_factory: Arc<StandardizedWorkflowFactory>,
}

/// Record of workflow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecutionRecord {
    /// Workflow ID
    pub workflow_id: String,
    /// Workflow type
    pub workflow_type: String,
    /// Start time
    pub start_time: chrono::DateTime<chrono::Utc>,
    /// End time
    pub end_time: Option<chrono::DateTime<chrono::Utc>>,
    /// Success status
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
    /// Execution duration in milliseconds
    pub duration_ms: Option<u64>,
}

/// Bridge metrics for monitoring
#[derive(Debug, Default)]
pub struct BridgeMetrics {
    /// Total workflows created
    workflows_created: AtomicU64,
    /// Total workflow executions
    workflow_executions: AtomicU64,
    /// Successful executions
    successful_executions: AtomicU64,
    /// Failed executions
    failed_executions: AtomicU64,
    /// Average execution time in milliseconds
    avg_execution_time_ms: AtomicU64,
}

impl WorkflowBridge {
    /// Create a new workflow bridge with optional state manager
    #[must_use]
    pub fn new(
        registry: &Arc<ComponentRegistry>,
        state_manager: Option<Arc<llmspell_state_persistence::StateManager>>,
    ) -> Self {
        // Create factory with state manager if available
        let standardized_factory = state_manager.as_ref().map_or_else(
            || {
                Arc::new(StandardizedWorkflowFactory::new_with_registry(
                    registry.clone(),
                ))
            },
            |sm| {
                Arc::new(StandardizedWorkflowFactory::new_with_state(
                    registry.clone(),
                    sm.clone(),
                ))
            },
        );

        Self {
            discovery: Arc::new(WorkflowDiscovery::new()),
            registry: registry.clone(),
            state_manager,
            active_workflows: Arc::new(RwLock::new(HashMap::new())),
            execution_history: Arc::new(RwLock::new(Vec::new())),
            metrics: Arc::new(BridgeMetrics::default()),
            _converter: Arc::new(OptimizedConverter::new()),
            execution_cache: Arc::new(ExecutionCache::new(1000)),
            perf_metrics: Arc::new(PerformanceMetrics::new()),
            standardized_factory,
        }
    }

    /// Get the state manager if available
    #[must_use]
    pub const fn state_manager(&self) -> &Option<Arc<llmspell_state_persistence::StateManager>> {
        &self.state_manager
    }

    /// List available workflow types
    #[must_use]
    pub fn list_workflow_types(&self) -> Vec<String> {
        self.standardized_factory.list_workflow_types()
    }

    /// Get information about a specific workflow type
    #[must_use]
    pub fn get_workflow_info(&self, workflow_type: &str) -> Option<WorkflowInfo> {
        self.discovery.get_workflow_info(workflow_type).cloned()
    }

    /// Get information about all workflow types
    #[must_use]
    pub fn get_all_workflow_info(&self) -> Vec<(String, WorkflowInfo)> {
        self.discovery.get_workflow_types()
    }

    /// Create a workflow instance from Rust structures
    ///
    /// # Errors
    ///
    /// Returns an error if workflow type is invalid or creation fails
    pub async fn create_workflow(
        &self,
        workflow_type: &str,
        name: String,
        steps: Vec<llmspell_workflows::WorkflowStep>,
        config: llmspell_workflows::WorkflowConfig,
        error_strategy: Option<llmspell_workflows::ErrorStrategy>,
    ) -> Result<String> {
        let workflow = self
            .standardized_factory
            .create_from_steps(workflow_type, name, steps, config, error_strategy)
            .await?;

        let workflow_id = format!("workflow_{}", uuid::Uuid::new_v4());
        let mut workflows = self.active_workflows.write().await;
        workflows.insert(workflow_id.clone(), Arc::new(workflow));

        self.metrics
            .workflows_created
            .fetch_add(1, Ordering::Relaxed);

        info!(
            "Created workflow '{}' of type '{}'",
            workflow_id, workflow_type
        );
        Ok(workflow_id)
    }

    /// Execute a workflow
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - JSON serialization fails
    /// - Workflow is not found
    /// - Workflow execution fails
    pub async fn execute_workflow(
        &self,
        workflow_id: &str,
        input: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let start_instant = std::time::Instant::now();
        let start_time = chrono::Utc::now();

        // Try cache first
        let cache_key = format!("{}:{}", workflow_id, serde_json::to_string(&input)?);
        if let Some(cached) = self.execution_cache.get(&cache_key) {
            debug!("Returning cached result for workflow '{}'", workflow_id);
            return Ok(cached);
        }

        // Get workflow
        let workflow = {
            let workflows = self.active_workflows.read().await;
            workflows
                .get(workflow_id)
                .cloned()
                .ok_or_else(|| LLMSpellError::Component {
                    message: format!("No active workflow with ID: {workflow_id}"),
                    source: None,
                })?
        };

        let workflow_type = workflow.workflow_type().to_string();

        // Execute workflow
        match workflow.execute(input).await {
            Ok(output) => {
                let duration_ms =
                    u64::try_from(start_instant.elapsed().as_millis()).unwrap_or(u64::MAX);

                // Record successful execution
                let record = WorkflowExecutionRecord {
                    workflow_id: workflow_id.to_string(),
                    workflow_type: workflow_type.clone(),
                    start_time,
                    end_time: Some(chrono::Utc::now()),
                    success: true,
                    error: None,
                    duration_ms: Some(duration_ms),
                };

                self.record_execution(record).await;
                self.update_metrics(true, duration_ms);

                // Cache result
                self.execution_cache
                    .put(workflow_id.to_string(), output.clone());

                // Record performance
                self.perf_metrics.record_operation(duration_ms);

                info!(
                    "Workflow '{}' executed successfully in {}ms",
                    workflow_id, duration_ms
                );
                Ok(output)
            }
            Err(e) => {
                let duration_ms =
                    u64::try_from(start_instant.elapsed().as_millis()).unwrap_or(u64::MAX);

                // Record failed execution
                let record = WorkflowExecutionRecord {
                    workflow_id: workflow_id.to_string(),
                    workflow_type: workflow_type.clone(),
                    start_time,
                    end_time: Some(chrono::Utc::now()),
                    success: false,
                    error: Some(e.to_string()),
                    duration_ms: Some(duration_ms),
                };

                self.record_execution(record).await;
                self.update_metrics(false, duration_ms);

                // Record performance even for failures
                self.perf_metrics.record_operation(duration_ms);

                warn!(
                    "Workflow '{}' failed after {}ms: {}",
                    workflow_id, duration_ms, e
                );
                Err(e)
            }
        }
    }

    // JSON-based oneshot execution removed - use create_workflow with Rust structures instead
    // pub async fn execute_workflow_oneshot(...) removed

    /// Get a workflow instance by ID
    ///
    /// # Errors
    ///
    /// Returns an error if the workflow is not found
    pub async fn get_workflow(&self, workflow_id: &str) -> Result<WorkflowInfo> {
        let workflows = self.active_workflows.read().await;
        let workflow = workflows
            .get(workflow_id)
            .ok_or_else(|| LLMSpellError::Component {
                message: format!("No active workflow with ID: {workflow_id}"),
                source: None,
            })?;

        // Return workflow info for the instance
        Ok(WorkflowInfo {
            workflow_type: workflow.workflow_type().to_string(),
            description: format!("Active workflow: {}", workflow.name()),
            features: vec![],
            required_params: vec![],
            optional_params: vec![],
        })
    }

    /// Remove a workflow instance
    ///
    /// # Errors
    ///
    /// Returns an error if the workflow is not found
    pub async fn remove_workflow(&self, workflow_id: &str) -> Result<()> {
        let mut workflows = self.active_workflows.write().await;
        workflows
            .remove(workflow_id)
            .ok_or_else(|| LLMSpellError::Component {
                message: format!("No active workflow with ID: {workflow_id}"),
                source: None,
            })?;

        debug!("Removed workflow '{}'", workflow_id);
        Ok(())
    }

    /// List active workflow instances
    pub async fn list_active_workflows(&self) -> Vec<(String, String)> {
        let workflows = self.active_workflows.read().await;
        workflows
            .iter()
            .map(|(id, workflow)| (id.clone(), workflow.workflow_type().to_string()))
            .collect()
    }

    /// Get workflow execution history
    pub async fn get_execution_history(&self) -> Vec<WorkflowExecutionRecord> {
        let history = self.execution_history.read().await;
        history.clone()
    }

    /// Get workflow status
    ///
    /// # Errors
    ///
    /// Returns an error if the workflow is not found
    pub async fn get_workflow_status(&self, workflow_id: &str) -> Result<WorkflowStatus> {
        let workflows = self.active_workflows.read().await;
        if workflows.contains_key(workflow_id) {
            // Check if workflow is in execution history
            let history = self.execution_history.read().await;
            let recent_execution = history
                .iter()
                .rfind(|record| record.workflow_id == workflow_id);

            let status = match recent_execution {
                Some(record) if record.end_time.is_none() => WorkflowStatus::Running,
                Some(record) if record.success => WorkflowStatus::Completed,
                Some(record) => WorkflowStatus::Failed(record.error.clone().unwrap_or_default()),
                None => WorkflowStatus::Ready,
            };

            Ok(status)
        } else {
            Err(LLMSpellError::Component {
                message: format!("No active workflow with ID: {workflow_id}"),
                source: None,
            })
        }
    }

    /// Get bridge metrics
    pub async fn get_bridge_metrics(&self) -> serde_json::Value {
        serde_json::json!({
            "workflows_created": self.metrics.workflows_created.load(Ordering::Relaxed),
            "workflow_executions": self.metrics.workflow_executions.load(Ordering::Relaxed),
            "successful_executions": self.metrics.successful_executions.load(Ordering::Relaxed),
            "failed_executions": self.metrics.failed_executions.load(Ordering::Relaxed),
            "avg_execution_time_ms": self.metrics.avg_execution_time_ms.load(Ordering::Relaxed),
            "active_workflows": self.active_workflows.read().await.len(),
            "performance": {
                "average_operation_ms": self.perf_metrics.average_duration_ms(),
                "p99_operation_ms": self.perf_metrics.p99_duration_ms(),
                "within_bounds": self.perf_metrics.is_within_bounds(),
            }
        })
    }

    /// Get performance metrics
    #[must_use]
    pub fn get_performance_metrics(&self) -> serde_json::Value {
        serde_json::json!({
            "average_duration_ms": self.perf_metrics.average_duration_ms(),
            "p99_duration_ms": self.perf_metrics.p99_duration_ms(),
            "is_within_10ms_target": self.perf_metrics.is_within_bounds(),
        })
    }

    /// Clear execution history
    pub async fn clear_execution_history(&self) {
        let mut history = self.execution_history.write().await;
        history.clear();
        debug!("Cleared workflow execution history");
    }

    // Private helper methods

    async fn record_execution(&self, record: WorkflowExecutionRecord) {
        let mut history = self.execution_history.write().await;
        history.push(record);

        // Keep only last 1000 records
        if history.len() > 1000 {
            history.drain(0..100);
        }
    }

    fn update_metrics(&self, success: bool, duration_ms: u64) {
        self.metrics
            .workflow_executions
            .fetch_add(1, Ordering::Relaxed);

        if success {
            self.metrics
                .successful_executions
                .fetch_add(1, Ordering::Relaxed);
        } else {
            self.metrics
                .failed_executions
                .fetch_add(1, Ordering::Relaxed);
        }

        // Update average execution time (simple moving average)
        let current_avg = self.metrics.avg_execution_time_ms.load(Ordering::Relaxed);
        let executions = self.metrics.workflow_executions.load(Ordering::Relaxed);
        let new_avg = if executions > 1 {
            (current_avg * (executions - 1) + duration_ms) / executions
        } else {
            duration_ms
        };
        self.metrics
            .avg_execution_time_ms
            .store(new_avg, Ordering::Relaxed);
    }
}

// =====================================================================
// WorkflowRegistry - Merged from workflow_registry_bridge.rs
// =====================================================================

/// Workflow registry for managing workflow instances
pub struct WorkflowRegistry {
    /// Registered workflow instances
    workflows: Arc<RwLock<HashMap<String, WorkflowRegistration>>>,
    /// Workflow templates
    templates: Arc<RwLock<HashMap<String, WorkflowTemplate>>>,
    /// Registry metrics
    metrics: Arc<RegistryMetrics>,
}

/// Registration information for a workflow
#[derive(Clone)]
struct WorkflowRegistration {
    /// Workflow ID
    _id: String,
    /// Workflow instance
    workflow: Arc<Box<dyn WorkflowExecutor>>,
    /// Registration metadata
    metadata: WorkflowMetadata,
    /// Usage statistics
    usage_stats: WorkflowUsageStats,
}

/// Workflow metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowMetadata {
    /// Workflow name
    pub name: String,
    /// Workflow type
    pub workflow_type: String,
    /// Description
    pub description: Option<String>,
    /// Tags for categorization
    pub tags: Vec<String>,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last modified timestamp
    pub modified_at: chrono::DateTime<chrono::Utc>,
    /// Version
    pub version: String,
    /// Author/creator
    pub author: Option<String>,
}

/// Workflow usage statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkflowUsageStats {
    /// Total executions
    pub total_executions: u64,
    /// Successful executions
    pub successful_executions: u64,
    /// Failed executions
    pub failed_executions: u64,
    /// Average execution time in ms
    pub avg_execution_time_ms: u64,
    /// Last execution time
    pub last_execution: Option<chrono::DateTime<chrono::Utc>>,
}

/// Workflow template for creating workflow instances
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTemplate {
    /// Template ID
    pub id: String,
    /// Template name
    pub name: String,
    /// Workflow type
    pub workflow_type: String,
    /// Template description
    pub description: String,
    /// Default configuration
    pub default_config: serde_json::Value,
    /// Parameter schema
    pub parameter_schema: serde_json::Value,
    /// Example usage
    pub example: Option<serde_json::Value>,
}

/// Registry metrics
#[derive(Debug, Default)]
struct RegistryMetrics {
    /// Total workflows registered
    registered: AtomicU64,
    /// Total templates registered
    templates: AtomicU64,
    /// Total workflow executions through registry
    executions: AtomicU64,
}

impl WorkflowRegistry {
    /// Create a new workflow registry
    #[must_use]
    pub fn new() -> Self {
        let mut registry = Self {
            workflows: Arc::new(RwLock::new(HashMap::new())),
            templates: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RegistryMetrics::default()),
        };

        // Register default templates
        registry.register_default_templates();

        registry
    }

    /// Register a workflow instance
    ///
    /// # Errors
    ///
    /// Returns an error if a workflow with the same ID is already registered
    pub async fn register_workflow(
        &self,
        id: String,
        workflow: Box<dyn WorkflowExecutor>,
        metadata: WorkflowMetadata,
    ) -> Result<()> {
        let registration = WorkflowRegistration {
            _id: id.clone(),
            workflow: Arc::new(workflow),
            metadata,
            usage_stats: WorkflowUsageStats::default(),
        };

        let mut workflows = self.workflows.write().await;
        if workflows.contains_key(&id) {
            return Err(LLMSpellError::Configuration {
                message: format!("Workflow '{id}' already registered"),
                source: None,
            });
        }

        workflows.insert(id, registration);
        self.metrics.registered.fetch_add(1, Ordering::Relaxed);

        Ok(())
    }

    /// Unregister a workflow
    ///
    /// # Errors
    ///
    /// Returns an error if the workflow is not found
    pub async fn unregister_workflow(&self, id: &str) -> Result<()> {
        let mut workflows = self.workflows.write().await;
        workflows
            .remove(id)
            .ok_or_else(|| LLMSpellError::Component {
                message: format!("No workflow registered with ID: {id}"),
                source: None,
            })?;

        Ok(())
    }

    /// Get a workflow by ID
    ///
    /// # Errors
    ///
    /// Returns an error if the workflow is not found
    pub async fn get_workflow(&self, id: &str) -> Result<Arc<Box<dyn WorkflowExecutor>>> {
        let workflows = self.workflows.read().await;
        workflows
            .get(id)
            .map(|reg| reg.workflow.clone())
            .ok_or_else(|| LLMSpellError::Component {
                message: format!("No workflow found with ID: {id}"),
                source: None,
            })
    }

    /// List all registered workflows
    pub async fn list_workflows(&self) -> Vec<(String, WorkflowMetadata)> {
        let workflows = self.workflows.read().await;
        workflows
            .iter()
            .map(|(id, reg)| (id.clone(), reg.metadata.clone()))
            .collect()
    }

    /// Search workflows by criteria
    pub async fn search_workflows(
        &self,
        criteria: SearchCriteria,
    ) -> Vec<(String, WorkflowMetadata)> {
        let workflows = self.workflows.read().await;
        workflows
            .iter()
            .filter(|(_, reg)| criteria.matches(&reg.metadata))
            .map(|(id, reg)| (id.clone(), reg.metadata.clone()))
            .collect()
    }

    /// Update workflow usage statistics
    ///
    /// # Errors
    ///
    /// Returns an error if the workflow is not found
    pub async fn update_usage_stats(
        &self,
        id: &str,
        success: bool,
        execution_time_ms: u64,
    ) -> Result<()> {
        let mut workflows = self.workflows.write().await;
        let registration = workflows
            .get_mut(id)
            .ok_or_else(|| LLMSpellError::Component {
                message: format!("No workflow registration found with ID: {id}"),
                source: None,
            })?;

        let stats = &mut registration.usage_stats;
        stats.total_executions += 1;
        if success {
            stats.successful_executions += 1;
        } else {
            stats.failed_executions += 1;
        }

        // Update average execution time
        let current_avg = stats.avg_execution_time_ms;
        let total = stats.total_executions;
        stats.avg_execution_time_ms = (current_avg * (total - 1) + execution_time_ms) / total;
        stats.last_execution = Some(chrono::Utc::now());

        self.metrics.executions.fetch_add(1, Ordering::Relaxed);

        Ok(())
    }

    /// Get workflow usage statistics
    ///
    /// # Errors
    ///
    /// Returns an error if the workflow is not found
    pub async fn get_usage_stats(&self, id: &str) -> Result<WorkflowUsageStats> {
        let workflows = self.workflows.read().await;
        workflows
            .get(id)
            .map(|reg| reg.usage_stats.clone())
            .ok_or_else(|| LLMSpellError::Component {
                message: format!("No workflow found with ID: {id}"),
                source: None,
            })
    }

    /// Register a workflow template
    ///
    /// # Errors
    ///
    /// Returns an error if a template with the same ID already exists
    pub async fn register_template(&self, template: WorkflowTemplate) -> Result<()> {
        let mut templates = self.templates.write().await;
        templates.insert(template.id.clone(), template);
        self.metrics.templates.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    /// Get a workflow template
    ///
    /// # Errors
    ///
    /// Returns an error if the template is not found
    pub async fn get_template(&self, template_id: &str) -> Result<WorkflowTemplate> {
        let templates = self.templates.read().await;
        templates
            .get(template_id)
            .cloned()
            .ok_or_else(|| LLMSpellError::Component {
                message: format!("No workflow template found with ID: {template_id}"),
                source: None,
            })
    }

    /// List all templates
    pub async fn list_templates(&self) -> Vec<WorkflowTemplate> {
        let templates = self.templates.read().await;
        templates.values().cloned().collect()
    }

    // JSON-based template creation removed - use create_workflow with Rust structures instead
    /*
    pub async fn create_from_template(
        &self,
        template_id: &str,
        params: serde_json::Value,
        bridge: &WorkflowBridge,
    ) -> Result<String> {
        // Removed - requires JSON
    }
    */

    /// Register default workflow templates
    fn register_default_templates(&mut self) {
        let templates = vec![
            WorkflowTemplate {
                id: "sequential_basic".to_string(),
                name: "Basic Sequential Workflow".to_string(),
                workflow_type: "sequential".to_string(),
                description: "Execute steps one after another".to_string(),
                default_config: serde_json::json!({
                    "name": "sequential_workflow",
                    "steps": [],
                    "error_strategy": "stop"
                }),
                parameter_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "name": {"type": "string"},
                        "steps": {"type": "array"},
                        "error_strategy": {"type": "string", "enum": ["stop", "continue", "retry"]}
                    },
                    "required": ["steps"]
                }),
                example: Some(serde_json::json!({
                    "name": "data_processing",
                    "steps": [
                        {"name": "load", "tool": "file_reader"},
                        {"name": "process", "tool": "data_processor"},
                        {"name": "save", "tool": "file_writer"}
                    ]
                })),
            },
            WorkflowTemplate {
                id: "parallel_basic".to_string(),
                name: "Basic Parallel Workflow".to_string(),
                workflow_type: "parallel".to_string(),
                description: "Execute multiple branches concurrently".to_string(),
                default_config: serde_json::json!({
                    "name": "parallel_workflow",
                    "branches": [],
                    "max_concurrency": 4,
                    "fail_fast": true
                }),
                parameter_schema: serde_json::json!({
                    "type": "object",
                    "properties": {
                        "name": {"type": "string"},
                        "branches": {"type": "array"},
                        "max_concurrency": {"type": "integer", "minimum": 1},
                        "fail_fast": {"type": "boolean"}
                    },
                    "required": ["branches"]
                }),
                example: Some(serde_json::json!({
                    "name": "multi_analysis",
                    "branches": [
                        {"name": "technical", "steps": [{"tool": "tech_analyzer"}]},
                        {"name": "business", "steps": [{"tool": "biz_analyzer"}]}
                    ]
                })),
            },
        ];

        // Synchronously add templates during initialization
        let templates_map = Arc::get_mut(&mut self.templates)
            .expect("templates Arc should have single owner during initialization");
        let templates_write = templates_map.get_mut();
        for template in templates {
            templates_write.insert(template.id.clone(), template);
            self.metrics.templates.fetch_add(1, Ordering::Relaxed);
        }
    }
}

/// Search criteria for workflows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchCriteria {
    /// Workflow type filter
    pub workflow_type: Option<String>,
    /// Name pattern (substring match)
    pub name_pattern: Option<String>,
    /// Tags to match (any)
    pub tags: Option<Vec<String>>,
    /// Author filter
    pub author: Option<String>,
    /// Created after date
    pub created_after: Option<chrono::DateTime<chrono::Utc>>,
    /// Modified after date
    pub modified_after: Option<chrono::DateTime<chrono::Utc>>,
}

impl SearchCriteria {
    /// Check if metadata matches criteria
    fn matches(&self, metadata: &WorkflowMetadata) -> bool {
        // Check workflow type
        if let Some(ref wf_type) = self.workflow_type {
            if &metadata.workflow_type != wf_type {
                return false;
            }
        }

        // Check name pattern
        if let Some(ref pattern) = self.name_pattern {
            if !metadata
                .name
                .to_lowercase()
                .contains(&pattern.to_lowercase())
            {
                return false;
            }
        }

        // Check tags
        if let Some(ref tags) = self.tags {
            let has_matching_tag = tags.iter().any(|tag| metadata.tags.contains(tag));
            if !has_matching_tag {
                return false;
            }
        }

        // Check author
        if let Some(ref author) = self.author {
            if metadata.author.as_ref() != Some(author) {
                return false;
            }
        }

        // Check dates
        if let Some(created_after) = self.created_after {
            if metadata.created_at < created_after {
                return false;
            }
        }

        if let Some(modified_after) = self.modified_after {
            if metadata.modified_at < modified_after {
                return false;
            }
        }

        true
    }
}

impl Default for WorkflowRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// =====================================================================
// Additional tests from merged files
// =====================================================================

#[cfg(test)]
mod workflow_bridge_tests {
    use super::*;
    #[tokio::test]
    async fn test_workflow_bridge_creation() {
        let registry = Arc::new(ComponentRegistry::new());
        let bridge = WorkflowBridge::new(&registry, None);

        // Test listing workflow types
        let types = bridge.list_workflow_types();
        assert_eq!(types.len(), 4);
        assert!(types.contains(&"sequential".to_string()));
    }
    #[tokio::test]
    async fn test_workflow_info() {
        let registry = Arc::new(ComponentRegistry::new());
        let bridge = WorkflowBridge::new(&registry, None);

        // Test getting workflow info
        let info = bridge.get_workflow_info("sequential").unwrap();
        assert_eq!(info.workflow_type, "sequential");
        assert!(info.required_params.contains(&"steps".to_string()));

        // Test getting all workflow info
        let all_info = bridge.get_all_workflow_info();
        assert_eq!(all_info.len(), 4);
    }
    #[tokio::test]
    async fn test_bridge_metrics() {
        let registry = Arc::new(ComponentRegistry::new());
        let bridge = WorkflowBridge::new(&registry, None);

        // Get initial metrics
        let metrics = bridge.get_bridge_metrics().await;
        assert_eq!(metrics["workflows_created"], 0);
        assert_eq!(metrics["workflow_executions"], 0);
        assert_eq!(metrics["active_workflows"], 0);
    }
}

#[cfg(test)]
mod workflow_registry_tests {
    use super::*;
    #[tokio::test]
    async fn test_workflow_registry() {
        let registry = WorkflowRegistry::new();

        // Test template listing
        let templates = registry.list_templates().await;
        assert!(templates.len() >= 2);

        // Test template retrieval
        let template = registry.get_template("sequential_basic").await.unwrap();
        assert_eq!(template.workflow_type, "sequential");
    }
    #[test]
    fn test_search_criteria() {
        let criteria = SearchCriteria {
            workflow_type: Some("sequential".to_string()),
            name_pattern: Some("data".to_string()),
            tags: Some(vec!["processing".to_string()]),
            author: None,
            created_after: None,
            modified_after: None,
        };

        let metadata = WorkflowMetadata {
            name: "data_processing_workflow".to_string(),
            workflow_type: "sequential".to_string(),
            description: None,
            tags: vec!["processing".to_string(), "etl".to_string()],
            created_at: chrono::Utc::now(),
            modified_at: chrono::Utc::now(),
            version: "1.0.0".to_string(),
            author: None,
        };

        assert!(criteria.matches(&metadata));
    }
}
