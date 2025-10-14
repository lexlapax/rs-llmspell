//! Workflow Orchestrator Template
//!
//! User-configurable workflow orchestration with custom patterns:
//! 1. Parse workflow configuration (agents, tools, execution pattern)
//! 2. Build dynamic workflow (parallel, sequential, or hybrid)
//! 3. Execute workflow with state tracking
//! 4. Collect and merge results

use crate::{
    artifacts::Artifact,
    context::ExecutionContext,
    core::{
        CostEstimate, TemplateCategory, TemplateMetadata, TemplateOutput, TemplateParams,
        TemplateResult,
    },
    error::{Result, TemplateError},
    validation::{ConfigSchema, ParameterConstraints, ParameterSchema, ParameterType},
};
use async_trait::async_trait;
use serde_json::json;
use std::time::Instant;
use tracing::{info, warn};

/// Workflow Orchestrator Template
///
/// Flexible workflow orchestration with user-defined patterns:
/// - Configurable agent and tool composition
/// - Support for parallel, sequential, and hybrid execution patterns
/// - Dynamic workflow building from JSON configuration
/// - State tracking and result aggregation across steps
#[derive(Debug)]
pub struct WorkflowOrchestratorTemplate {
    metadata: TemplateMetadata,
}

impl WorkflowOrchestratorTemplate {
    /// Create a new Workflow Orchestrator template instance
    pub fn new() -> Self {
        Self {
            metadata: TemplateMetadata {
                id: "workflow-orchestrator".to_string(),
                name: "Workflow Orchestrator".to_string(),
                description: "Flexible AI workflow orchestration with custom patterns. \
                             Define your own agent and tool compositions with parallel, \
                             sequential, or hybrid execution. Build complex multi-step \
                             workflows with state tracking and result aggregation."
                    .to_string(),
                category: TemplateCategory::Workflow,
                version: "0.1.0".to_string(),
                author: Some("LLMSpell Team".to_string()),
                requires: vec![], // No specific requirements - works with any agents/tools
                tags: vec![
                    "workflow".to_string(),
                    "orchestration".to_string(),
                    "composition".to_string(),
                    "flexible".to_string(),
                    "custom".to_string(),
                ],
            },
        }
    }
}

impl Default for WorkflowOrchestratorTemplate {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl crate::core::Template for WorkflowOrchestratorTemplate {
    fn metadata(&self) -> &TemplateMetadata {
        &self.metadata
    }

    fn config_schema(&self) -> ConfigSchema {
        ConfigSchema::new(vec![
            // workflow_config (required object)
            ParameterSchema::required(
                "workflow_config",
                "Workflow configuration with agents, tools, and execution pattern",
                ParameterType::Object,
            )
            .with_constraints(ParameterConstraints {
                min_length: Some(1),
                ..Default::default()
            }),
            // execution_mode (optional enum with default)
            ParameterSchema::optional(
                "execution_mode",
                "Workflow execution mode (parallel, sequential, hybrid)",
                ParameterType::String,
                json!("sequential"),
            )
            .with_constraints(ParameterConstraints {
                allowed_values: Some(vec![
                    json!("parallel"),
                    json!("sequential"),
                    json!("hybrid"),
                ]),
                ..Default::default()
            }),
            // collect_intermediate (optional boolean with default)
            ParameterSchema::optional(
                "collect_intermediate",
                "Collect intermediate results from each step",
                ParameterType::Boolean,
                json!(true),
            ),
            // max_steps (optional integer with default)
            ParameterSchema::optional(
                "max_steps",
                "Maximum number of workflow steps to execute",
                ParameterType::Integer,
                json!(10),
            )
            .with_constraints(ParameterConstraints {
                min: Some(1.0),
                max: Some(100.0),
                ..Default::default()
            }),
            // model (optional - for agent execution)
            ParameterSchema::optional(
                "model",
                "Default LLM model for agents in workflow",
                ParameterType::String,
                json!("ollama/llama3.2:3b"),
            ),
        ])
    }

    async fn execute(
        &self,
        params: TemplateParams,
        context: ExecutionContext,
    ) -> Result<TemplateOutput> {
        let start_time = Instant::now();

        // Extract and validate parameters
        let workflow_config: serde_json::Value = params.get("workflow_config")?;
        let execution_mode: String = params.get_or("execution_mode", "sequential".to_string());
        let collect_intermediate: bool = params.get_or("collect_intermediate", true);
        let max_steps: i64 = params.get_or("max_steps", 10);
        let model: String = params.get_or("model", "ollama/llama3.2:3b".to_string());

        info!(
            "Starting workflow orchestration (mode={}, max_steps={}, model={})",
            execution_mode, max_steps, model
        );

        // Initialize output
        let mut output = TemplateOutput::new(
            TemplateResult::text(""), // Will be replaced
            self.metadata.id.clone(),
            self.metadata.version.clone(),
            params,
        );

        // Phase 1: Parse and validate workflow configuration
        info!("Phase 1: Parsing workflow configuration...");
        let workflow = self.parse_workflow(&workflow_config)?;

        // Phase 2: Build execution plan
        info!(
            "Phase 2: Building execution plan ({} steps)...",
            workflow.steps.len()
        );
        let execution_plan = self.build_execution_plan(&workflow, &execution_mode, max_steps)?;

        // Phase 3: Execute workflow
        info!("Phase 3: Executing workflow...");
        let execution_result = self
            .execute_workflow(&execution_plan, &model, collect_intermediate, &context)
            .await?;
        output.metrics.agents_invoked = execution_result.agents_executed;
        output.metrics.tools_invoked = execution_result.tools_executed;

        // Phase 4: Aggregate results
        info!("Phase 4: Aggregating results...");
        let final_result = self.aggregate_results(&execution_result);

        // Save artifacts
        if let Some(output_dir) = &context.output_dir {
            self.save_artifacts(
                output_dir,
                &execution_result,
                collect_intermediate,
                &mut output,
            )?;
        }

        // Set result and metrics
        output.result = TemplateResult::text(final_result);
        output.set_duration(start_time.elapsed().as_millis() as u64);
        output.add_metric("workflow_steps", json!(workflow.steps.len()));
        output.add_metric("execution_mode", json!(execution_mode));
        output.add_metric("steps_executed", json!(execution_result.steps_executed));

        info!(
            "Workflow orchestration complete (duration: {}ms, steps: {})",
            output.metrics.duration_ms, execution_result.steps_executed
        );
        Ok(output)
    }

    async fn estimate_cost(&self, params: &TemplateParams) -> CostEstimate {
        // Try to parse workflow config to count steps
        let workflow_config: serde_json::Value = params
            .get("workflow_config")
            .unwrap_or_else(|_| json!({"steps": []}));

        let step_count = workflow_config
            .get("steps")
            .and_then(|s| s.as_array())
            .map(|arr| arr.len())
            .filter(|&len| len > 0)
            .unwrap_or(3); // Default estimate: 3 steps

        // Rough estimates per step:
        // - Agent step: ~1000 tokens
        // - Tool step: minimal tokens
        // Assume 70% agents, 30% tools
        let agent_steps = (step_count as f64 * 0.7) as usize;
        let estimated_tokens = agent_steps * 1000;

        // Assuming $0.10 per 1M tokens (local LLM is cheaper)
        let estimated_cost = (estimated_tokens as f64 / 1_000_000.0) * 0.10;

        // Per step:
        // - Agent: ~3s
        // - Tool: ~1s
        let estimated_duration = (agent_steps * 3000) + ((step_count - agent_steps) * 1000);

        CostEstimate::new(
            estimated_tokens as u64,
            estimated_cost,
            estimated_duration as u64,
            0.5, // Medium-low confidence - highly variable based on workflow
        )
    }
}

impl WorkflowOrchestratorTemplate {
    /// Phase 1: Parse workflow configuration
    fn parse_workflow(&self, config: &serde_json::Value) -> Result<WorkflowDefinition> {
        // TODO: Implement actual workflow parsing from JSON
        // For now, return placeholder workflow
        warn!("Workflow parsing not yet implemented - using placeholder");

        let steps = config
            .get("steps")
            .and_then(|s| s.as_array())
            .map(|arr| arr.len())
            .unwrap_or(3);

        Ok(WorkflowDefinition {
            name: config
                .get("name")
                .and_then(|n| n.as_str())
                .unwrap_or("custom-workflow")
                .to_string(),
            steps: (0..steps)
                .map(|i| WorkflowStep {
                    step_id: format!("step-{}", i + 1),
                    step_type: if i % 2 == 0 {
                        StepType::Agent
                    } else {
                        StepType::Tool
                    },
                    description: format!("Step {} placeholder", i + 1),
                })
                .collect(),
        })
    }

    /// Phase 2: Build execution plan
    fn build_execution_plan(
        &self,
        workflow: &WorkflowDefinition,
        execution_mode: &str,
        max_steps: i64,
    ) -> Result<ExecutionPlan> {
        let limited_steps = workflow
            .steps
            .iter()
            .take(max_steps as usize)
            .cloned()
            .collect();

        Ok(ExecutionPlan {
            workflow_name: workflow.name.clone(),
            steps: limited_steps,
            mode: execution_mode.to_string(),
        })
    }

    /// Phase 3: Execute workflow
    async fn execute_workflow(
        &self,
        plan: &ExecutionPlan,
        _model: &str,
        collect_intermediate: bool,
        _context: &ExecutionContext,
    ) -> Result<ExecutionResult> {
        // TODO: Implement actual workflow execution
        // For now, return placeholder execution
        warn!(
            "Workflow execution not yet implemented - using placeholder (mode: {})",
            plan.mode
        );

        let mut agents_executed = 0;
        let mut tools_executed = 0;
        let mut step_results = Vec::new();

        for step in &plan.steps {
            let result = match step.step_type {
                StepType::Agent => {
                    agents_executed += 1;
                    format!(
                        "Agent step '{}' ({}): Placeholder execution result",
                        step.step_id, step.description
                    )
                }
                StepType::Tool => {
                    tools_executed += 1;
                    format!(
                        "Tool step '{}' ({}): Placeholder execution result",
                        step.step_id, step.description
                    )
                }
            };

            if collect_intermediate {
                step_results.push(StepResult {
                    step_id: step.step_id.clone(),
                    result,
                });
            }
        }

        Ok(ExecutionResult {
            workflow_name: plan.workflow_name.clone(),
            steps_executed: plan.steps.len(),
            agents_executed,
            tools_executed,
            intermediate_results: if collect_intermediate {
                Some(step_results)
            } else {
                None
            },
            final_output: format!(
                "Workflow '{}' completed successfully with {} steps",
                plan.workflow_name,
                plan.steps.len()
            ),
        })
    }

    /// Phase 4: Aggregate results
    fn aggregate_results(&self, execution: &ExecutionResult) -> String {
        let mut report = format!(
            "# Workflow Execution Report\n\n\
             **Workflow**: {}\n\
             **Steps Executed**: {}\n\
             **Agents Invoked**: {}\n\
             **Tools Invoked**: {}\n\n\
             ---\n\n",
            execution.workflow_name,
            execution.steps_executed,
            execution.agents_executed,
            execution.tools_executed
        );

        if let Some(intermediate) = &execution.intermediate_results {
            report.push_str("## Step-by-Step Results\n\n");
            for (idx, step_result) in intermediate.iter().enumerate() {
                report.push_str(&format!(
                    "### Step {}: {}\n\n{}\n\n",
                    idx + 1,
                    step_result.step_id,
                    step_result.result
                ));
            }
            report.push_str("---\n\n");
        }

        report.push_str(&format!(
            "## Final Output\n\n{}\n\n\
             ---\n\n\
             Generated by LLMSpell Workflow Orchestrator Template\n",
            execution.final_output
        ));

        report
    }

    /// Save artifacts to output directory
    fn save_artifacts(
        &self,
        output_dir: &std::path::Path,
        execution: &ExecutionResult,
        collect_intermediate: bool,
        output: &mut TemplateOutput,
    ) -> Result<()> {
        use std::fs;

        // Create output directory if it doesn't exist
        fs::create_dir_all(output_dir).map_err(|e| {
            TemplateError::ExecutionFailed(format!("Failed to create output directory: {}", e))
        })?;

        // Save workflow execution report
        let report = self.aggregate_results(execution);
        let report_path = output_dir.join("workflow_report.md");
        fs::write(&report_path, &report).map_err(|e| {
            TemplateError::ExecutionFailed(format!("Failed to write workflow report: {}", e))
        })?;
        output.add_artifact(Artifact::new(
            report_path.to_string_lossy().to_string(),
            report,
            "text/markdown".to_string(),
        ));

        // Save intermediate results if collected
        if collect_intermediate {
            if let Some(intermediate) = &execution.intermediate_results {
                let intermediate_json =
                    serde_json::to_string_pretty(intermediate).map_err(|e| {
                        TemplateError::ExecutionFailed(format!(
                            "Failed to serialize intermediate results: {}",
                            e
                        ))
                    })?;
                let intermediate_path = output_dir.join("intermediate_results.json");
                fs::write(&intermediate_path, &intermediate_json).map_err(|e| {
                    TemplateError::ExecutionFailed(format!(
                        "Failed to write intermediate results: {}",
                        e
                    ))
                })?;
                output.add_artifact(Artifact::new(
                    intermediate_path.to_string_lossy().to_string(),
                    intermediate_json,
                    "application/json".to_string(),
                ));
            }
        }

        Ok(())
    }
}

/// Workflow definition from configuration
#[derive(Debug, Clone)]
struct WorkflowDefinition {
    /// Workflow name
    name: String,
    /// Workflow steps
    steps: Vec<WorkflowStep>,
}

/// Individual workflow step
#[derive(Debug, Clone)]
struct WorkflowStep {
    /// Step identifier
    step_id: String,
    /// Type of step (agent or tool)
    step_type: StepType,
    /// Step description
    description: String,
}

/// Type of workflow step
#[derive(Debug, Clone, PartialEq)]
enum StepType {
    /// Agent execution step
    Agent,
    /// Tool execution step
    Tool,
}

/// Execution plan built from workflow definition
#[derive(Debug, Clone)]
struct ExecutionPlan {
    /// Workflow name
    workflow_name: String,
    /// Steps to execute
    steps: Vec<WorkflowStep>,
    /// Execution mode
    mode: String,
}

/// Result from a single step
#[derive(Debug, Clone, serde::Serialize)]
struct StepResult {
    /// Step identifier
    step_id: String,
    /// Step result
    result: String,
}

/// Overall execution result
#[derive(Debug, Clone)]
struct ExecutionResult {
    /// Workflow name
    workflow_name: String,
    /// Number of steps executed
    steps_executed: usize,
    /// Number of agents invoked
    agents_executed: usize,
    /// Number of tools invoked
    tools_executed: usize,
    /// Intermediate results (if collected)
    intermediate_results: Option<Vec<StepResult>>,
    /// Final workflow output
    final_output: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Template;

    #[test]
    fn test_template_metadata() {
        let template = WorkflowOrchestratorTemplate::new();
        let metadata = template.metadata();

        assert_eq!(metadata.id, "workflow-orchestrator");
        assert_eq!(metadata.name, "Workflow Orchestrator");
        assert_eq!(metadata.category, TemplateCategory::Workflow);
        assert!(metadata.requires.is_empty()); // Works with any agents/tools
        assert!(metadata.tags.contains(&"workflow".to_string()));
        assert!(metadata.tags.contains(&"orchestration".to_string()));
        assert!(metadata.tags.contains(&"flexible".to_string()));
    }

    #[test]
    fn test_config_schema() {
        let template = WorkflowOrchestratorTemplate::new();
        let schema = template.config_schema();

        assert!(schema.get_parameter("workflow_config").is_some());
        assert!(schema.get_parameter("execution_mode").is_some());
        assert!(schema.get_parameter("collect_intermediate").is_some());
        assert!(schema.get_parameter("max_steps").is_some());
        assert!(schema.get_parameter("model").is_some());

        // workflow_config is required
        let config_param = schema.get_parameter("workflow_config").unwrap();
        assert!(config_param.required);

        // others are optional
        let mode_param = schema.get_parameter("execution_mode").unwrap();
        assert!(!mode_param.required);
    }

    #[tokio::test]
    async fn test_cost_estimate_default() {
        let template = WorkflowOrchestratorTemplate::new();
        let params = TemplateParams::new();

        let estimate = template.estimate_cost(&params).await;
        assert!(estimate.estimated_tokens.is_some());
        assert!(estimate.estimated_cost_usd.is_some());
        assert!(estimate.estimated_duration_ms.is_some());
        // Default estimate: 3 steps, 70% agents = ~2 agent steps = 2000 tokens
        assert_eq!(estimate.estimated_tokens, Some(2000));
    }

    #[tokio::test]
    async fn test_cost_estimate_with_config() {
        let template = WorkflowOrchestratorTemplate::new();
        let mut params = TemplateParams::new();
        params.insert(
            "workflow_config",
            json!({"steps": ["step1", "step2", "step3", "step4", "step5"]}),
        );

        let estimate = template.estimate_cost(&params).await;
        // 5 steps, 70% agents = ~3 agent steps = 3000 tokens
        assert_eq!(estimate.estimated_tokens, Some(3000));
    }

    #[test]
    fn test_parameter_validation_missing_required() {
        let template = WorkflowOrchestratorTemplate::new();
        let schema = template.config_schema();
        let params = std::collections::HashMap::new();

        // Should fail - missing required "workflow_config" parameter
        let result = schema.validate(&params);
        assert!(result.is_err());
    }

    #[test]
    fn test_parameter_validation_invalid_mode() {
        let template = WorkflowOrchestratorTemplate::new();
        let schema = template.config_schema();
        let mut params = std::collections::HashMap::new();
        params.insert("workflow_config".to_string(), json!({"steps": ["step1"]}));
        params.insert("execution_mode".to_string(), json!("invalid_mode"));

        let result = schema.validate(&params);
        assert!(result.is_err());
    }

    #[test]
    fn test_parameter_validation_max_steps_out_of_range() {
        let template = WorkflowOrchestratorTemplate::new();
        let schema = template.config_schema();
        let mut params = std::collections::HashMap::new();
        params.insert("workflow_config".to_string(), json!({"steps": ["step1"]}));
        params.insert("max_steps".to_string(), json!(200)); // > 100

        let result = schema.validate(&params);
        assert!(result.is_err());
    }

    #[test]
    fn test_parameter_validation_success() {
        let template = WorkflowOrchestratorTemplate::new();
        let schema = template.config_schema();
        let mut params = std::collections::HashMap::new();
        params.insert(
            "workflow_config".to_string(),
            json!({"steps": ["step1", "step2"]}),
        );
        params.insert("execution_mode".to_string(), json!("sequential"));
        params.insert("collect_intermediate".to_string(), json!(true));
        params.insert("max_steps".to_string(), json!(10));

        let result = schema.validate(&params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_workflow_placeholder() {
        let template = WorkflowOrchestratorTemplate::new();
        let config = json!({"steps": ["a", "b", "c"]});

        let result = template.parse_workflow(&config);
        assert!(result.is_ok());
        let workflow = result.unwrap();
        assert_eq!(workflow.steps.len(), 3);
    }

    #[test]
    fn test_build_execution_plan() {
        let template = WorkflowOrchestratorTemplate::new();
        let workflow = WorkflowDefinition {
            name: "test".to_string(),
            steps: vec![
                WorkflowStep {
                    step_id: "s1".to_string(),
                    step_type: StepType::Agent,
                    description: "step 1".to_string(),
                },
                WorkflowStep {
                    step_id: "s2".to_string(),
                    step_type: StepType::Tool,
                    description: "step 2".to_string(),
                },
            ],
        };

        let result = template.build_execution_plan(&workflow, "sequential", 10);
        assert!(result.is_ok());
        let plan = result.unwrap();
        assert_eq!(plan.steps.len(), 2);
        assert_eq!(plan.mode, "sequential");
    }

    #[test]
    fn test_build_execution_plan_with_max_steps() {
        let template = WorkflowOrchestratorTemplate::new();
        let workflow = WorkflowDefinition {
            name: "test".to_string(),
            steps: vec![
                WorkflowStep {
                    step_id: "s1".to_string(),
                    step_type: StepType::Agent,
                    description: "step 1".to_string(),
                },
                WorkflowStep {
                    step_id: "s2".to_string(),
                    step_type: StepType::Tool,
                    description: "step 2".to_string(),
                },
                WorkflowStep {
                    step_id: "s3".to_string(),
                    step_type: StepType::Agent,
                    description: "step 3".to_string(),
                },
            ],
        };

        let result = template.build_execution_plan(&workflow, "parallel", 2);
        assert!(result.is_ok());
        let plan = result.unwrap();
        assert_eq!(plan.steps.len(), 2); // Limited to max_steps
    }

    #[tokio::test]
    async fn test_execute_workflow_placeholder() {
        let template = WorkflowOrchestratorTemplate::new();
        let context = ExecutionContext::builder().build();
        if context.is_err() {
            return;
        }
        let context = context.unwrap();

        let plan = ExecutionPlan {
            workflow_name: "test".to_string(),
            steps: vec![
                WorkflowStep {
                    step_id: "s1".to_string(),
                    step_type: StepType::Agent,
                    description: "step 1".to_string(),
                },
                WorkflowStep {
                    step_id: "s2".to_string(),
                    step_type: StepType::Tool,
                    description: "step 2".to_string(),
                },
            ],
            mode: "sequential".to_string(),
        };

        let result = template
            .execute_workflow(&plan, "ollama/llama3.2:3b", true, &context)
            .await;
        assert!(result.is_ok());
        let execution = result.unwrap();
        assert_eq!(execution.steps_executed, 2);
        assert_eq!(execution.agents_executed, 1);
        assert_eq!(execution.tools_executed, 1);
        assert!(execution.intermediate_results.is_some());
    }

    #[test]
    fn test_aggregate_results() {
        let template = WorkflowOrchestratorTemplate::new();
        let execution = ExecutionResult {
            workflow_name: "test".to_string(),
            steps_executed: 2,
            agents_executed: 1,
            tools_executed: 1,
            intermediate_results: Some(vec![
                StepResult {
                    step_id: "s1".to_string(),
                    result: "Result 1".to_string(),
                },
                StepResult {
                    step_id: "s2".to_string(),
                    result: "Result 2".to_string(),
                },
            ]),
            final_output: "Done".to_string(),
        };

        let report = template.aggregate_results(&execution);
        assert!(report.contains("# Workflow Execution Report"));
        assert!(report.contains("test"));
        assert!(report.contains("Step-by-Step Results"));
        assert!(report.contains("s1"));
        assert!(report.contains("s2"));
    }
}
