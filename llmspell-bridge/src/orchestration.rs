//! ABOUTME: Workflow-based orchestration patterns for complex agent systems
//! ABOUTME: Provides high-level orchestration capabilities using workflow primitives

#![allow(clippy::significant_drop_tightening)]

use llmspell_core::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Orchestration strategy for complex workflows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrchestrationStrategy {
    /// Static orchestration with predefined flow
    Static,
    /// Dynamic orchestration that adapts based on results
    Dynamic,
    /// Event-driven orchestration responding to triggers
    EventDriven,
    /// Goal-oriented orchestration working backwards from objectives
    GoalOriented,
}

/// Orchestration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationConfig {
    /// Orchestration strategy
    pub strategy: OrchestrationStrategy,
    /// Maximum orchestration depth (nested workflows)
    pub max_depth: usize,
    /// Timeout for entire orchestration
    pub timeout_seconds: u64,
    /// Whether to allow parallel orchestration branches
    pub allow_parallel: bool,
    /// Resource limits for orchestration
    pub resource_limits: ResourceLimits,
}

/// Resource limits for orchestration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum concurrent workflows
    pub max_concurrent_workflows: usize,
    /// Maximum total agent invocations
    pub max_agent_invocations: usize,
    /// Maximum memory usage in MB
    pub max_memory_mb: usize,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_concurrent_workflows: 10,
            max_agent_invocations: 100,
            max_memory_mb: 512,
        }
    }
}

impl ResourceLimits {
    /// Create a new builder for `ResourceLimits`
    #[must_use]
    pub fn builder() -> ResourceLimitsBuilder {
        ResourceLimitsBuilder::new()
    }
}

/// Builder for `ResourceLimits`
#[derive(Debug, Clone)]
pub struct ResourceLimitsBuilder {
    limits: ResourceLimits,
}

impl ResourceLimitsBuilder {
    /// Create a new builder with default limits
    #[must_use]
    pub fn new() -> Self {
        Self {
            limits: ResourceLimits::default(),
        }
    }

    /// Set maximum concurrent workflows
    #[must_use]
    pub const fn max_concurrent_workflows(mut self, max: usize) -> Self {
        self.limits.max_concurrent_workflows = max;
        self
    }

    /// Set maximum total agent invocations
    #[must_use]
    pub const fn max_agent_invocations(mut self, max: usize) -> Self {
        self.limits.max_agent_invocations = max;
        self
    }

    /// Set maximum memory usage in MB
    #[must_use]
    pub const fn max_memory_mb(mut self, max: usize) -> Self {
        self.limits.max_memory_mb = max;
        self
    }

    /// Build the `ResourceLimits`
    #[must_use]
    pub const fn build(self) -> ResourceLimits {
        self.limits
    }
}

impl Default for ResourceLimitsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for OrchestrationConfig {
    fn default() -> Self {
        Self {
            strategy: OrchestrationStrategy::Static,
            max_depth: 5,
            timeout_seconds: 300,
            allow_parallel: true,
            resource_limits: ResourceLimits::default(),
        }
    }
}

impl OrchestrationConfig {
    /// Create a new builder for `OrchestrationConfig`
    #[must_use]
    pub fn builder() -> OrchestrationConfigBuilder {
        OrchestrationConfigBuilder::new()
    }
}

/// Builder for `OrchestrationConfig`
#[derive(Debug, Clone)]
pub struct OrchestrationConfigBuilder {
    config: OrchestrationConfig,
}

impl OrchestrationConfigBuilder {
    /// Create a new builder with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: OrchestrationConfig::default(),
        }
    }

    /// Set the orchestration strategy
    #[must_use]
    pub const fn strategy(mut self, strategy: OrchestrationStrategy) -> Self {
        self.config.strategy = strategy;
        self
    }

    /// Set the maximum orchestration depth
    #[must_use]
    pub const fn max_depth(mut self, depth: usize) -> Self {
        self.config.max_depth = depth;
        self
    }

    /// Set the timeout in seconds
    #[must_use]
    pub const fn timeout_seconds(mut self, timeout: u64) -> Self {
        self.config.timeout_seconds = timeout;
        self
    }

    /// Set whether to allow parallel orchestration branches
    #[must_use]
    pub const fn allow_parallel(mut self, allow: bool) -> Self {
        self.config.allow_parallel = allow;
        self
    }

    /// Set the resource limits
    #[must_use]
    pub const fn resource_limits(mut self, limits: ResourceLimits) -> Self {
        self.config.resource_limits = limits;
        self
    }

    /// Build the `OrchestrationConfig`
    #[must_use]
    pub const fn build(self) -> OrchestrationConfig {
        self.config
    }
}

impl Default for OrchestrationConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Orchestration plan defining workflow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationPlan {
    /// Plan ID
    pub id: String,
    /// Plan name
    pub name: String,
    /// Plan description
    pub description: String,
    /// Root workflow configuration
    pub root_workflow: WorkflowNode,
    /// Global context available to all workflows
    pub global_context: Value,
    /// Success criteria for orchestration
    pub success_criteria: SuccessCriteria,
}

/// Node in orchestration workflow tree
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowNode {
    /// Node ID
    pub id: String,
    /// Workflow type
    pub workflow_type: String,
    /// Workflow configuration
    pub config: Value,
    /// Child nodes (for composite workflows)
    pub children: Vec<WorkflowNode>,
    /// Conditions for executing this node
    pub execution_conditions: Option<ExecutionConditions>,
    /// How to handle node results
    pub result_handling: ResultHandling,
}

/// Conditions for workflow execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConditions {
    /// Required previous nodes to complete
    pub depends_on: Vec<String>,
    /// Condition expression (evaluated against context)
    pub condition: Option<String>,
    /// Whether this node is optional
    pub optional: bool,
}

/// How to handle workflow results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResultHandling {
    /// Whether to store results in context
    pub store_in_context: bool,
    /// Key name for storing results
    pub context_key: Option<String>,
    /// Whether to propagate errors
    pub propagate_errors: bool,
    /// Retry configuration
    pub retry_config: Option<RetryConfig>,
}

/// Retry configuration for workflows
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum retry attempts
    pub max_attempts: usize,
    /// Backoff strategy
    pub backoff_ms: u64,
    /// Whether to use exponential backoff
    pub exponential_backoff: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            backoff_ms: 1000,
            exponential_backoff: false,
        }
    }
}

impl RetryConfig {
    /// Create a new builder for `RetryConfig`
    #[must_use]
    pub fn builder() -> RetryConfigBuilder {
        RetryConfigBuilder::new()
    }
}

/// Builder for `RetryConfig`
#[derive(Debug, Clone)]
pub struct RetryConfigBuilder {
    config: RetryConfig,
}

impl RetryConfigBuilder {
    /// Create a new builder with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: RetryConfig::default(),
        }
    }

    /// Set maximum retry attempts
    #[must_use]
    pub const fn max_attempts(mut self, attempts: usize) -> Self {
        self.config.max_attempts = attempts;
        self
    }

    /// Set backoff in milliseconds
    #[must_use]
    pub const fn backoff_ms(mut self, ms: u64) -> Self {
        self.config.backoff_ms = ms;
        self
    }

    /// Set whether to use exponential backoff
    #[must_use]
    pub const fn exponential_backoff(mut self, exponential: bool) -> Self {
        self.config.exponential_backoff = exponential;
        self
    }

    /// Build the `RetryConfig`
    #[must_use]
    pub const fn build(self) -> RetryConfig {
        self.config
    }
}

impl Default for RetryConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Success criteria for orchestration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessCriteria {
    /// Minimum success rate for workflows
    pub min_success_rate: f64,
    /// Required outputs in final context
    pub required_outputs: Vec<String>,
    /// Custom success expression
    pub success_expression: Option<String>,
}

/// Orchestration runtime for executing plans
pub struct OrchestrationRuntime {
    /// Configuration
    _config: OrchestrationConfig,
    /// Active orchestrations
    active_orchestrations: HashMap<String, OrchestrationState>,
    /// Metrics
    metrics: OrchestrationMetrics,
}

/// State of an active orchestration
#[derive(Debug, Clone)]
struct OrchestrationState {
    /// Plan being executed
    plan: OrchestrationPlan,
    /// Current execution context
    _context: HashMap<String, Value>,
    /// Completed nodes
    completed_nodes: Vec<String>,
    /// Failed nodes
    failed_nodes: Vec<String>,
    /// Start time
    start_time: std::time::Instant,
}

/// Orchestration metrics
#[derive(Debug, Default)]
struct OrchestrationMetrics {
    /// Total orchestrations started
    total_started: std::sync::atomic::AtomicU64,
    /// Total orchestrations completed
    _total_completed: std::sync::atomic::AtomicU64,
    /// Total orchestrations failed
    _total_failed: std::sync::atomic::AtomicU64,
    /// Total workflows executed
    _total_workflows: std::sync::atomic::AtomicU64,
}

impl OrchestrationRuntime {
    /// Create new orchestration runtime
    #[must_use]
    pub fn new(config: OrchestrationConfig) -> Self {
        Self {
            _config: config,
            active_orchestrations: HashMap::new(),
            metrics: OrchestrationMetrics::default(),
        }
    }

    /// Start orchestration from plan
    ///
    /// # Errors
    ///
    /// Returns an error if orchestration startup fails
    pub fn start_orchestration(&mut self, plan: OrchestrationPlan) -> Result<String> {
        let orchestration_id = format!("orch_{}", uuid::Uuid::new_v4());

        // Initialize orchestration state
        let mut context = HashMap::new();
        context.insert("global".to_string(), plan.global_context.clone());

        let state = OrchestrationState {
            plan,
            _context: context,
            completed_nodes: Vec::new(),
            failed_nodes: Vec::new(),
            start_time: std::time::Instant::now(),
        };

        self.active_orchestrations
            .insert(orchestration_id.clone(), state);
        self.metrics
            .total_started
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        Ok(orchestration_id)
    }

    /// Get orchestration status
    pub fn get_orchestration_status(&self, orchestration_id: &str) -> Option<OrchestrationStatus> {
        self.active_orchestrations
            .get(orchestration_id)
            .map(|state| OrchestrationStatus {
                id: orchestration_id.to_string(),
                plan_name: state.plan.name.clone(),
                elapsed_seconds: state.start_time.elapsed().as_secs(),
                completed_nodes: state.completed_nodes.len(),
                failed_nodes: state.failed_nodes.len(),
                total_nodes: count_nodes(&state.plan.root_workflow),
                is_complete: Self::is_orchestration_complete(state),
                is_successful: Self::is_orchestration_successful(state),
            })
    }

    /// Check if orchestration is complete
    fn is_orchestration_complete(state: &OrchestrationState) -> bool {
        let total_nodes = count_nodes(&state.plan.root_workflow);
        state.completed_nodes.len() + state.failed_nodes.len() >= total_nodes
    }

    /// Check if orchestration is successful
    fn is_orchestration_successful(state: &OrchestrationState) -> bool {
        if !Self::is_orchestration_complete(state) {
            return false;
        }

        let total_nodes = count_nodes(&state.plan.root_workflow);
        #[allow(clippy::cast_precision_loss)]
        let success_rate = state.completed_nodes.len() as f64 / total_nodes as f64;

        success_rate >= state.plan.success_criteria.min_success_rate
    }
}

/// Orchestration status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestrationStatus {
    /// Orchestration ID
    pub id: String,
    /// Plan name
    pub plan_name: String,
    /// Elapsed time in seconds
    pub elapsed_seconds: u64,
    /// Number of completed nodes
    pub completed_nodes: usize,
    /// Number of failed nodes
    pub failed_nodes: usize,
    /// Total number of nodes
    pub total_nodes: usize,
    /// Whether orchestration is complete
    pub is_complete: bool,
    /// Whether orchestration is successful
    pub is_successful: bool,
}

/// Count total nodes in workflow tree
fn count_nodes(node: &WorkflowNode) -> usize {
    1 + node.children.iter().map(count_nodes).sum::<usize>()
}

/// Orchestration templates for common patterns
pub struct OrchestrationTemplates;

impl OrchestrationTemplates {
    /// Create a data processing pipeline orchestration
    #[must_use]
    pub fn data_pipeline_orchestration() -> OrchestrationPlan {
        OrchestrationPlan {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Data Processing Pipeline".to_string(),
            description: "Multi-stage data processing with validation".to_string(),
            root_workflow: WorkflowNode {
                id: "root".to_string(),
                workflow_type: "sequential".to_string(),
                config: serde_json::json!({
                    "name": "data_pipeline_root"
                }),
                children: vec![
                    WorkflowNode {
                        id: "ingestion".to_string(),
                        workflow_type: "parallel".to_string(),
                        config: serde_json::json!({
                            "name": "data_ingestion",
                            "max_concurrency": 3
                        }),
                        children: vec![],
                        execution_conditions: None,
                        result_handling: ResultHandling {
                            store_in_context: true,
                            context_key: Some("raw_data".to_string()),
                            propagate_errors: true,
                            retry_config: Some(
                                RetryConfig::builder()
                                    .max_attempts(3)
                                    .backoff_ms(1000)
                                    .exponential_backoff(true)
                                    .build(),
                            ),
                        },
                    },
                    WorkflowNode {
                        id: "validation".to_string(),
                        workflow_type: "conditional".to_string(),
                        config: serde_json::json!({
                            "name": "data_validation",
                            "condition": "raw_data.is_valid"
                        }),
                        children: vec![],
                        execution_conditions: Some(ExecutionConditions {
                            depends_on: vec!["ingestion".to_string()],
                            condition: None,
                            optional: false,
                        }),
                        result_handling: ResultHandling {
                            store_in_context: true,
                            context_key: Some("validated_data".to_string()),
                            propagate_errors: true,
                            retry_config: None,
                        },
                    },
                    WorkflowNode {
                        id: "processing".to_string(),
                        workflow_type: "loop".to_string(),
                        config: serde_json::json!({
                            "name": "batch_processing",
                            "iterator": {"type": "collection", "items": "$validated_data.batches"}
                        }),
                        children: vec![],
                        execution_conditions: Some(ExecutionConditions {
                            depends_on: vec!["validation".to_string()],
                            condition: Some("validated_data.is_valid == true".to_string()),
                            optional: false,
                        }),
                        result_handling: ResultHandling {
                            store_in_context: true,
                            context_key: Some("processed_data".to_string()),
                            propagate_errors: false,
                            retry_config: None,
                        },
                    },
                ],
                execution_conditions: None,
                result_handling: ResultHandling {
                    store_in_context: false,
                    context_key: None,
                    propagate_errors: true,
                    retry_config: None,
                },
            },
            global_context: serde_json::json!({
                "config": {
                    "batch_size": 100,
                    "validation_rules": ["schema_check", "range_check", "format_check"]
                }
            }),
            success_criteria: SuccessCriteria {
                min_success_rate: 0.9,
                required_outputs: vec!["processed_data".to_string()],
                success_expression: None,
            },
        }
    }

    /// Create a multi-agent research orchestration
    #[must_use]
    pub fn research_orchestration() -> OrchestrationPlan {
        OrchestrationPlan {
            id: uuid::Uuid::new_v4().to_string(),
            name: "Research Orchestration".to_string(),
            description: "Coordinated research across multiple agents and sources".to_string(),
            root_workflow: WorkflowNode {
                id: "root".to_string(),
                workflow_type: "sequential".to_string(),
                config: serde_json::json!({
                    "name": "research_root"
                }),
                children: vec![
                    WorkflowNode {
                        id: "topic_analysis".to_string(),
                        workflow_type: "delegation".to_string(),
                        config: serde_json::json!({
                            "coordinator": "research_coordinator",
                            "workers": ["domain_expert_1", "domain_expert_2"]
                        }),
                        children: vec![],
                        execution_conditions: None,
                        result_handling: ResultHandling {
                            store_in_context: true,
                            context_key: Some("research_topics".to_string()),
                            propagate_errors: true,
                            retry_config: None,
                        },
                    },
                    WorkflowNode {
                        id: "literature_review".to_string(),
                        workflow_type: "parallel".to_string(),
                        config: serde_json::json!({
                            "name": "parallel_literature_search"
                        }),
                        children: vec![],
                        execution_conditions: Some(ExecutionConditions {
                            depends_on: vec!["topic_analysis".to_string()],
                            condition: None,
                            optional: false,
                        }),
                        result_handling: ResultHandling {
                            store_in_context: true,
                            context_key: Some("literature_findings".to_string()),
                            propagate_errors: false,
                            retry_config: None,
                        },
                    },
                    WorkflowNode {
                        id: "synthesis".to_string(),
                        workflow_type: "collaboration".to_string(),
                        config: serde_json::json!({
                            "agents": ["analyst_1", "analyst_2", "synthesizer"],
                            "rounds": 3
                        }),
                        children: vec![],
                        execution_conditions: Some(ExecutionConditions {
                            depends_on: vec!["literature_review".to_string()],
                            condition: None,
                            optional: false,
                        }),
                        result_handling: ResultHandling {
                            store_in_context: true,
                            context_key: Some("research_synthesis".to_string()),
                            propagate_errors: true,
                            retry_config: None,
                        },
                    },
                ],
                execution_conditions: None,
                result_handling: ResultHandling {
                    store_in_context: false,
                    context_key: None,
                    propagate_errors: true,
                    retry_config: None,
                },
            },
            global_context: serde_json::json!({
                "research_params": {
                    "depth": "comprehensive",
                    "sources": ["academic", "industry", "news"],
                    "time_range": "last_5_years"
                }
            }),
            success_criteria: SuccessCriteria {
                min_success_rate: 0.8,
                required_outputs: vec![
                    "research_topics".to_string(),
                    "literature_findings".to_string(),
                    "research_synthesis".to_string(),
                ],
                success_expression: Some("research_synthesis.quality_score > 0.8".to_string()),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_orchestration_plan_creation() {
        let plan = OrchestrationTemplates::data_pipeline_orchestration();
        assert_eq!(plan.name, "Data Processing Pipeline");
        assert_eq!(count_nodes(&plan.root_workflow), 4); // root + 3 children
    }
    #[tokio::test]
    async fn test_orchestration_runtime() {
        let _runtime = OrchestrationRuntime::new(OrchestrationConfig::default());
        let plan = OrchestrationTemplates::research_orchestration();

        // Runtime would execute the plan
        // This is a simplified test
        assert_eq!(plan.name, "Research Orchestration");
        assert_eq!(plan.success_criteria.min_success_rate, 0.8);
    }
}
