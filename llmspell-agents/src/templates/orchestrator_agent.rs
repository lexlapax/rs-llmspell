//! ABOUTME: Orchestrator Agent template for creating agents that coordinate other agents and workflows
//! ABOUTME: Provides standardized template for orchestration-focused agents with workflow management capabilities

use super::base::{AgentTemplate, TemplateInstantiationParams, TemplateInstantiationResult};
use super::schema::{
    CapabilityRequirement, ComplexityLevel, ParameterConstraint, ParameterDefinition,
    ParameterType, ResourceRequirements, TemplateCategory, TemplateMetadata, TemplateSchema,
    ToolDependency,
};
use anyhow::Result;
use async_trait::async_trait;
use llmspell_core::{
    types::{AgentInput, AgentOutput, ComponentId, ComponentMetadata, OutputMetadata, Version},
    BaseAgent, ExecutionContext, LLMSpellError,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Orchestration strategy types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrchestrationStrategy {
    /// Execute tasks sequentially
    Sequential,
    /// Execute tasks in parallel where possible
    Parallel,
    /// Use conditional logic for task execution
    Conditional,
    /// Pipeline-based execution
    Pipeline,
    /// Event-driven orchestration
    EventDriven,
    /// Custom orchestration strategy
    Custom(String),
}

impl OrchestrationStrategy {
    pub fn name(&self) -> String {
        match self {
            OrchestrationStrategy::Sequential => "sequential".to_string(),
            OrchestrationStrategy::Parallel => "parallel".to_string(),
            OrchestrationStrategy::Conditional => "conditional".to_string(),
            OrchestrationStrategy::Pipeline => "pipeline".to_string(),
            OrchestrationStrategy::EventDriven => "event_driven".to_string(),
            OrchestrationStrategy::Custom(name) => name.clone(),
        }
    }
}

/// Orchestrator Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrchestratorAgentConfig {
    /// Maximum number of managed agents
    pub max_managed_agents: usize,
    /// Default orchestration strategy
    pub default_strategy: OrchestrationStrategy,
    /// Enable workflow persistence
    pub enable_workflow_persistence: bool,
    /// Maximum workflow execution time in seconds
    pub max_workflow_time: u64,
    /// Enable agent health monitoring
    pub enable_health_monitoring: bool,
    /// Health check interval in seconds
    pub health_check_interval: u64,
    /// Maximum concurrent workflows
    pub max_concurrent_workflows: usize,
    /// Enable workflow rollback on failure
    pub enable_rollback: bool,
    /// Workflow retry configuration
    pub max_retries: u32,
    /// Agent discovery patterns
    pub agent_discovery_patterns: Vec<String>,
    /// Workflow templates directory
    pub workflow_templates_dir: Option<String>,
}

impl Default for OrchestratorAgentConfig {
    fn default() -> Self {
        Self {
            max_managed_agents: 20,
            default_strategy: OrchestrationStrategy::Sequential,
            enable_workflow_persistence: true,
            max_workflow_time: 3600, // 1 hour
            enable_health_monitoring: true,
            health_check_interval: 30, // 30 seconds
            max_concurrent_workflows: 3,
            enable_rollback: true,
            max_retries: 3,
            agent_discovery_patterns: vec!["agents/*.toml".to_string()],
            workflow_templates_dir: Some("workflows/".to_string()),
        }
    }
}

/// Orchestrator Agent template implementation
pub struct OrchestratorAgentTemplate {
    schema: TemplateSchema,
    config: OrchestratorAgentConfig,
}

impl OrchestratorAgentTemplate {
    /// Create new Orchestrator Agent template
    pub fn new() -> Self {
        let metadata = TemplateMetadata {
            id: "orchestrator_agent".to_string(),
            name: "Orchestrator Agent".to_string(),
            version: "1.0.0".to_string(),
            description: "Agent template for orchestrating workflows and coordinating other agents"
                .to_string(),
            author: "rs-llmspell".to_string(),
            license: "MIT".to_string(),
            repository: Some("https://github.com/lexlapax/rs-llmspell".to_string()),
            documentation: Some("https://docs.rs/llmspell-agents".to_string()),
            keywords: vec![
                "orchestrator".to_string(),
                "workflow".to_string(),
                "coordination".to_string(),
                "management".to_string(),
            ],
            category: TemplateCategory::Orchestration,
            complexity: ComplexityLevel::Advanced,
        };

        let mut schema = TemplateSchema::new(metadata);

        // Add parameters
        schema = schema
            .with_parameter(ParameterDefinition {
                name: "agent_name".to_string(),
                description: "Human-readable name for the orchestrator agent".to_string(),
                param_type: ParameterType::String,
                required: true,
                default_value: None,
                constraints: vec![ParameterConstraint::MinLength(1)],
                examples: vec!["Workflow Orchestrator".into(), "Task Coordinator".into()],
            })
            .with_parameter(ParameterDefinition {
                name: "max_managed_agents".to_string(),
                description: "Maximum number of agents this orchestrator can manage".to_string(),
                param_type: ParameterType::Integer,
                required: false,
                default_value: Some(20.into()),
                constraints: vec![
                    ParameterConstraint::MinValue(1.0),
                    ParameterConstraint::MaxValue(100.0),
                ],
                examples: vec![5.into(), 20.into(), 50.into()],
            })
            .with_parameter(ParameterDefinition {
                name: "orchestration_strategy".to_string(),
                description: "Default strategy for orchestrating workflows".to_string(),
                param_type: ParameterType::Enum(vec![
                    "sequential".to_string(),
                    "parallel".to_string(),
                    "conditional".to_string(),
                    "pipeline".to_string(),
                    "event_driven".to_string(),
                ]),
                required: false,
                default_value: Some("sequential".into()),
                constraints: vec![],
                examples: vec!["parallel".into(), "pipeline".into()],
            })
            .with_parameter(ParameterDefinition {
                name: "max_workflow_time".to_string(),
                description: "Maximum workflow execution time in seconds".to_string(),
                param_type: ParameterType::Integer,
                required: false,
                default_value: Some(3600.into()),
                constraints: vec![
                    ParameterConstraint::MinValue(60.0),
                    ParameterConstraint::MaxValue(86400.0), // 24 hours
                ],
                examples: vec![1800.into(), 3600.into(), 7200.into()],
            })
            .with_parameter(ParameterDefinition {
                name: "enable_health_monitoring".to_string(),
                description: "Enable health monitoring for managed agents".to_string(),
                param_type: ParameterType::Boolean,
                required: false,
                default_value: Some(true.into()),
                constraints: vec![],
                examples: vec![true.into(), false.into()],
            })
            .with_parameter(ParameterDefinition {
                name: "health_check_interval".to_string(),
                description: "Interval between health checks in seconds".to_string(),
                param_type: ParameterType::Integer,
                required: false,
                default_value: Some(30.into()),
                constraints: vec![
                    ParameterConstraint::MinValue(5.0),
                    ParameterConstraint::MaxValue(300.0),
                ],
                examples: vec![10.into(), 30.into(), 60.into()],
            })
            .with_parameter(ParameterDefinition {
                name: "max_concurrent_workflows".to_string(),
                description: "Maximum number of workflows that can run concurrently".to_string(),
                param_type: ParameterType::Integer,
                required: false,
                default_value: Some(3.into()),
                constraints: vec![
                    ParameterConstraint::MinValue(1.0),
                    ParameterConstraint::MaxValue(20.0),
                ],
                examples: vec![1.into(), 3.into(), 10.into()],
            })
            .with_parameter(ParameterDefinition {
                name: "enable_rollback".to_string(),
                description: "Enable automatic rollback on workflow failures".to_string(),
                param_type: ParameterType::Boolean,
                required: false,
                default_value: Some(true.into()),
                constraints: vec![],
                examples: vec![true.into(), false.into()],
            })
            .with_parameter(ParameterDefinition {
                name: "max_retries".to_string(),
                description: "Maximum number of retry attempts for failed workflows".to_string(),
                param_type: ParameterType::Integer,
                required: false,
                default_value: Some(3.into()),
                constraints: vec![
                    ParameterConstraint::MinValue(0.0),
                    ParameterConstraint::MaxValue(10.0),
                ],
                examples: vec![0.into(), 3.into(), 5.into()],
            })
            .with_parameter(ParameterDefinition {
                name: "workflow_templates_dir".to_string(),
                description: "Directory containing workflow template definitions".to_string(),
                param_type: ParameterType::String,
                required: false,
                default_value: Some("workflows/".into()),
                constraints: vec![],
                examples: vec![
                    "templates/".into(),
                    "workflows/".into(),
                    "orchestration/".into(),
                ],
            })
            .with_parameter(ParameterDefinition {
                name: "managed_agent_types".to_string(),
                description: "Types of agents this orchestrator can manage".to_string(),
                param_type: ParameterType::Array(Box::new(ParameterType::String)),
                required: false,
                default_value: Some(vec!["tool_agent", "monitor_agent"].into()),
                constraints: vec![],
                examples: vec![
                    vec!["tool_agent", "analytics_agent"].into(),
                    vec!["monitor_agent", "communication_agent"].into(),
                ],
            });

        // Add tool dependencies for orchestration
        schema = schema
            .with_tool_dependency(ToolDependency {
                name: "workflow_engine".to_string(),
                version: Some("1.0.0".to_string()),
                required: true,
                alternatives: vec![],
                config: HashMap::new(),
            })
            .with_tool_dependency(ToolDependency {
                name: "agent_manager".to_string(),
                version: Some("1.0.0".to_string()),
                required: true,
                alternatives: vec![],
                config: HashMap::new(),
            })
            .with_tool_dependency(ToolDependency {
                name: "health_monitor".to_string(),
                version: Some("1.0.0".to_string()),
                required: false,
                alternatives: vec!["system_monitor".to_string()],
                config: HashMap::new(),
            })
            .with_tool_dependency(ToolDependency {
                name: "state_manager".to_string(),
                version: Some("1.0.0".to_string()),
                required: true,
                alternatives: vec!["persistence_layer".to_string()],
                config: HashMap::new(),
            });

        // Add capability requirements
        schema = schema
            .with_capability_requirement(CapabilityRequirement {
                name: "workflow_orchestration".to_string(),
                min_level: 8,
                critical: true,
                usage_description: "Core capability for orchestrating complex workflows"
                    .to_string(),
            })
            .with_capability_requirement(CapabilityRequirement {
                name: "agent_management".to_string(),
                min_level: 7,
                critical: true,
                usage_description: "Manage lifecycle and coordination of multiple agents"
                    .to_string(),
            })
            .with_capability_requirement(CapabilityRequirement {
                name: "state_management".to_string(),
                min_level: 6,
                critical: true,
                usage_description: "Maintain and persist workflow and agent state".to_string(),
            })
            .with_capability_requirement(CapabilityRequirement {
                name: "error_recovery".to_string(),
                min_level: 7,
                critical: true,
                usage_description: "Handle failures and implement recovery strategies".to_string(),
            })
            .with_capability_requirement(CapabilityRequirement {
                name: "performance_monitoring".to_string(),
                min_level: 5,
                critical: false,
                usage_description: "Monitor performance of workflows and agents".to_string(),
            });

        // Set resource requirements (higher than tool agents due to orchestration overhead)
        schema = schema.with_resource_requirements(ResourceRequirements {
            memory: Some(512 * 1024 * 1024), // 512MB
            cpu: Some(40),                   // 40% CPU
            disk: Some(100 * 1024 * 1024),   // 100MB
            network: Some(10 * 1024 * 1024), // 10MB/s
            max_execution_time: Some(7200),  // 2 hours
        });

        // Add configuration
        schema = schema
            .with_config("agent_type", "orchestrator_agent".into())
            .with_config("supports_parallel_execution", true.into())
            .with_config("supports_workflow_persistence", true.into())
            .with_config("supports_agent_discovery", true.into())
            .with_config("supports_health_monitoring", true.into());

        Self {
            schema,
            config: OrchestratorAgentConfig::default(),
        }
    }

    /// Create Orchestrator Agent template with custom configuration
    pub fn with_config(mut self, config: OrchestratorAgentConfig) -> Self {
        self.config = config;
        self
    }

    /// Create simple orchestrator template for basic workflow coordination
    pub fn simple() -> Self {
        let mut template = Self::new();

        // Update configuration for simple operation
        template.config.max_managed_agents = 5;
        template.config.max_concurrent_workflows = 1;
        template.config.enable_health_monitoring = false;
        template.config.enable_rollback = false;
        template.config.max_retries = 1;

        // Update resource requirements
        template.schema = template
            .schema
            .with_resource_requirements(ResourceRequirements {
                memory: Some(256 * 1024 * 1024), // 256MB
                cpu: Some(25),                   // 25% CPU
                disk: Some(50 * 1024 * 1024),    // 50MB
                network: Some(5 * 1024 * 1024),  // 5MB/s
                max_execution_time: Some(1800),  // 30 minutes
            });

        // Update metadata
        template.schema.metadata.id = "orchestrator_agent_simple".to_string();
        template.schema.metadata.name = "Simple Orchestrator Agent".to_string();
        template.schema.metadata.description =
            "Simple orchestrator for basic workflow coordination".to_string();
        template.schema.metadata.complexity = ComplexityLevel::Basic;

        template
    }

    /// Create enterprise orchestrator template for complex orchestration scenarios
    pub fn enterprise() -> Self {
        let mut template = Self::new();

        // Update configuration for enterprise operation
        template.config.max_managed_agents = 100;
        template.config.max_concurrent_workflows = 20;
        template.config.enable_health_monitoring = true;
        template.config.health_check_interval = 10;
        template.config.enable_rollback = true;
        template.config.max_retries = 5;
        template.config.max_workflow_time = 86400; // 24 hours

        // Update resource requirements
        template.schema = template
            .schema
            .with_resource_requirements(ResourceRequirements {
                memory: Some(2 * 1024 * 1024 * 1024), // 2GB
                cpu: Some(80),                        // 80% CPU
                disk: Some(500 * 1024 * 1024),        // 500MB
                network: Some(50 * 1024 * 1024),      // 50MB/s
                max_execution_time: Some(86400),      // 24 hours
            });

        // Update metadata
        template.schema.metadata.id = "orchestrator_agent_enterprise".to_string();
        template.schema.metadata.name = "Enterprise Orchestrator Agent".to_string();
        template.schema.metadata.description =
            "Enterprise-grade orchestrator for complex multi-agent workflows".to_string();
        template.schema.metadata.complexity = ComplexityLevel::Expert;

        // Add enterprise-specific configuration
        template.schema = template
            .schema
            .with_config("enterprise_mode", true.into())
            .with_config("high_availability", true.into())
            .with_config("distributed_orchestration", true.into())
            .with_config("audit_logging", true.into());

        template
    }

    /// Create event-driven orchestrator template
    pub fn event_driven() -> Self {
        let mut template = Self::new();

        // Update configuration for event-driven operation
        template.config.default_strategy = OrchestrationStrategy::EventDriven;
        template.config.enable_health_monitoring = true;
        template.config.health_check_interval = 5; // More frequent checks

        // Update metadata
        template.schema.metadata.id = "orchestrator_agent_event_driven".to_string();
        template.schema.metadata.name = "Event-Driven Orchestrator Agent".to_string();
        template.schema.metadata.description =
            "Event-driven orchestrator for reactive workflow management".to_string();

        // Add event-specific tools
        template.schema = template
            .schema
            .with_tool_dependency(ToolDependency {
                name: "event_bus".to_string(),
                version: Some("1.0.0".to_string()),
                required: true,
                alternatives: vec!["message_broker".to_string()],
                config: HashMap::new(),
            })
            .with_tool_dependency(ToolDependency {
                name: "event_processor".to_string(),
                version: Some("1.0.0".to_string()),
                required: true,
                alternatives: vec![],
                config: HashMap::new(),
            });

        // Add event-specific configuration
        template.schema = template
            .schema
            .with_config("event_driven_mode", true.into())
            .with_config("supports_event_sourcing", true.into())
            .with_config("reactive_orchestration", true.into());

        template
    }

    /// Apply parameters to config
    fn apply_parameters_to_config(
        &self,
        config: &mut OrchestratorAgentConfig,
        params: &HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        if let Some(max_agents) = params.get("max_managed_agents") {
            if let Some(value) = max_agents.as_u64() {
                config.max_managed_agents = value as usize;
            }
        }

        if let Some(strategy) = params.get("orchestration_strategy") {
            if let Some(value) = strategy.as_str() {
                config.default_strategy = match value {
                    "sequential" => OrchestrationStrategy::Sequential,
                    "parallel" => OrchestrationStrategy::Parallel,
                    "conditional" => OrchestrationStrategy::Conditional,
                    "pipeline" => OrchestrationStrategy::Pipeline,
                    "event_driven" => OrchestrationStrategy::EventDriven,
                    _ => OrchestrationStrategy::Custom(value.to_string()),
                };
            }
        }

        if let Some(max_time) = params.get("max_workflow_time") {
            if let Some(value) = max_time.as_u64() {
                config.max_workflow_time = value;
            }
        }

        if let Some(enable_monitoring) = params.get("enable_health_monitoring") {
            if let Some(value) = enable_monitoring.as_bool() {
                config.enable_health_monitoring = value;
            }
        }

        if let Some(interval) = params.get("health_check_interval") {
            if let Some(value) = interval.as_u64() {
                config.health_check_interval = value;
            }
        }

        if let Some(max_concurrent) = params.get("max_concurrent_workflows") {
            if let Some(value) = max_concurrent.as_u64() {
                config.max_concurrent_workflows = value as usize;
            }
        }

        if let Some(enable_rollback) = params.get("enable_rollback") {
            if let Some(value) = enable_rollback.as_bool() {
                config.enable_rollback = value;
            }
        }

        if let Some(max_retries) = params.get("max_retries") {
            if let Some(value) = max_retries.as_u64() {
                config.max_retries = value as u32;
            }
        }

        if let Some(templates_dir) = params.get("workflow_templates_dir") {
            if let Some(value) = templates_dir.as_str() {
                config.workflow_templates_dir = Some(value.to_string());
            }
        }

        Ok(())
    }
}

impl Default for OrchestratorAgentTemplate {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AgentTemplate for OrchestratorAgentTemplate {
    fn schema(&self) -> &TemplateSchema {
        &self.schema
    }

    async fn instantiate(
        &self,
        mut params: TemplateInstantiationParams,
    ) -> Result<TemplateInstantiationResult> {
        // Apply defaults
        self.apply_defaults(&mut params).await?;

        // Validate parameters
        self.validate_parameters(&params).await?;

        // Create agent-specific configuration
        let mut agent_config = self.config.clone();
        self.apply_parameters_to_config(&mut agent_config, &params.parameters)?;

        // Build final configuration
        let mut final_config = HashMap::new();
        final_config.insert("agent_type".to_string(), "orchestrator_agent".into());
        final_config.insert(
            "max_managed_agents".to_string(),
            (agent_config.max_managed_agents as u64).into(),
        );
        final_config.insert(
            "orchestration_strategy".to_string(),
            agent_config.default_strategy.name().into(),
        );
        final_config.insert(
            "max_workflow_time".to_string(),
            agent_config.max_workflow_time.into(),
        );
        final_config.insert(
            "enable_health_monitoring".to_string(),
            agent_config.enable_health_monitoring.into(),
        );
        final_config.insert(
            "health_check_interval".to_string(),
            agent_config.health_check_interval.into(),
        );
        final_config.insert(
            "max_concurrent_workflows".to_string(),
            (agent_config.max_concurrent_workflows as u64).into(),
        );
        final_config.insert(
            "enable_rollback".to_string(),
            agent_config.enable_rollback.into(),
        );
        final_config.insert(
            "max_retries".to_string(),
            (agent_config.max_retries as u64).into(),
        );

        if let Some(templates_dir) = &agent_config.workflow_templates_dir {
            final_config.insert(
                "workflow_templates_dir".to_string(),
                templates_dir.clone().into(),
            );
        }

        // Add managed agent types if specified
        if let Some(agent_types) = params.parameters.get("managed_agent_types") {
            final_config.insert("managed_agent_types".to_string(), agent_types.clone());
        }

        // Apply config overrides
        for (key, value) in params.config_overrides {
            final_config.insert(key, value);
        }

        // Get agent name
        let agent_name = params
            .parameters
            .get("agent_name")
            .and_then(|v| v.as_str())
            .unwrap_or(&params.agent_id)
            .to_string();

        // For now, return a mock result since we can't create actual BaseAgent instances
        // In a real implementation, this would create the actual orchestrator agent
        let mock_agent =
            MockOrchestratorAgent::new(params.agent_id.clone(), agent_name, final_config.clone());

        Ok(TemplateInstantiationResult {
            agent: Box::new(mock_agent),
            template_schema: self.schema.clone(),
            applied_parameters: params.parameters,
            applied_config: final_config,
        })
    }

    fn clone_template(&self) -> Box<dyn AgentTemplate> {
        Box::new(OrchestratorAgentTemplate {
            schema: self.schema.clone(),
            config: self.config.clone(),
        })
    }
}

/// Mock orchestrator agent for testing (replace with actual implementation)
#[allow(dead_code)]
struct MockOrchestratorAgent {
    id: String,
    name: String,
    config: HashMap<String, serde_json::Value>,
    metadata: ComponentMetadata,
}

impl MockOrchestratorAgent {
    fn new(id: String, name: String, config: HashMap<String, serde_json::Value>) -> Self {
        let metadata = ComponentMetadata {
            id: ComponentId::from_name(&id),
            name: name.clone(),
            version: Version::new(1, 0, 0),
            description: "Mock orchestrator agent for template testing".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        Self {
            id,
            name,
            config,
            metadata,
        }
    }
}

#[async_trait]
impl BaseAgent for MockOrchestratorAgent {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute(
        &self,
        _input: AgentInput,
        _context: ExecutionContext,
    ) -> Result<AgentOutput, LLMSpellError> {
        Ok(AgentOutput {
            text: "Mock orchestrator agent execution result".to_string(),
            media: vec![],
            tool_calls: vec![],
            metadata: OutputMetadata::default(),
            transfer_to: None,
        })
    }

    async fn validate_input(&self, _input: &AgentInput) -> Result<(), LLMSpellError> {
        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput, LLMSpellError> {
        Ok(AgentOutput {
            text: format!("Orchestrator error handled: {}", error),
            media: vec![],
            tool_calls: vec![],
            metadata: OutputMetadata::default(),
            transfer_to: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_orchestrator_agent_template_creation() {
        let template = OrchestratorAgentTemplate::new();

        assert_eq!(template.schema().metadata.id, "orchestrator_agent");
        assert_eq!(template.category(), &TemplateCategory::Orchestration);
        assert_eq!(template.complexity(), &ComplexityLevel::Advanced);

        let required_params = template.schema().required_parameters();
        assert_eq!(required_params.len(), 1);
        assert_eq!(required_params[0].name, "agent_name");
    }
    #[tokio::test]
    async fn test_simple_orchestrator() {
        let template = OrchestratorAgentTemplate::simple();

        assert_eq!(template.config.max_managed_agents, 5);
        assert_eq!(template.config.max_concurrent_workflows, 1);
        assert!(!template.config.enable_health_monitoring);
        assert_eq!(template.complexity(), &ComplexityLevel::Basic);
    }
    #[tokio::test]
    async fn test_enterprise_orchestrator() {
        let template = OrchestratorAgentTemplate::enterprise();

        assert_eq!(template.config.max_managed_agents, 100);
        assert_eq!(template.config.max_concurrent_workflows, 20);
        assert!(template.config.enable_health_monitoring);
        assert_eq!(template.complexity(), &ComplexityLevel::Expert);

        // Check enterprise-specific configuration
        let enterprise_mode = template.schema().template_config.get("enterprise_mode");
        assert_eq!(enterprise_mode, Some(&true.into()));
    }
    #[tokio::test]
    async fn test_event_driven_orchestrator() {
        let template = OrchestratorAgentTemplate::event_driven();

        assert!(matches!(
            template.config.default_strategy,
            OrchestrationStrategy::EventDriven
        ));
        assert_eq!(template.config.health_check_interval, 5);

        // Check for event-specific tools
        let event_bus_dep = template.schema().get_tool_dependency("event_bus");
        assert!(event_bus_dep.is_some());
        assert!(event_bus_dep.unwrap().required);
    }
    #[tokio::test]
    async fn test_orchestration_strategies() {
        assert_eq!(OrchestrationStrategy::Sequential.name(), "sequential");
        assert_eq!(OrchestrationStrategy::Parallel.name(), "parallel");
        assert_eq!(OrchestrationStrategy::EventDriven.name(), "event_driven");
        assert_eq!(
            OrchestrationStrategy::Custom("test".to_string()).name(),
            "test"
        );
    }
    #[tokio::test]
    async fn test_parameter_validation() {
        let template = OrchestratorAgentTemplate::new();

        // Test missing required parameter
        let params = TemplateInstantiationParams::new("test-agent".to_string());
        let result = template.validate_parameters(&params).await;
        assert!(result.is_err());

        // Test valid parameters
        let params = TemplateInstantiationParams::new("test-agent".to_string())
            .with_parameter("agent_name", "Test Orchestrator".into())
            .with_parameter("max_managed_agents", 10.into())
            .with_parameter("orchestration_strategy", "parallel".into())
            .with_parameter("enable_health_monitoring", true.into());

        let result = template.validate_parameters(&params).await;
        assert!(result.is_ok());
    }
    #[tokio::test]
    async fn test_required_tools() {
        let template = OrchestratorAgentTemplate::new();

        let required_tools = template.required_tools();
        assert!(required_tools.contains(&"workflow_engine".to_string()));
        assert!(required_tools.contains(&"agent_manager".to_string()));
        assert!(required_tools.contains(&"state_manager".to_string()));

        let optional_tools = template.optional_tools();
        assert!(optional_tools.contains(&"health_monitor".to_string()));
    }
    #[tokio::test]
    async fn test_capability_support() {
        let template = OrchestratorAgentTemplate::new();

        assert!(template.supports_capability("workflow_orchestration"));
        assert!(template.supports_capability("agent_management"));
        assert!(template.supports_capability("state_management"));
        assert!(template.supports_capability("error_recovery"));
        assert!(!template.supports_capability("nonexistent_capability"));
    }
    #[tokio::test]
    async fn test_parameter_constraints() {
        let template = OrchestratorAgentTemplate::new();

        // Test invalid max_managed_agents (too high)
        let params = TemplateInstantiationParams::new("test-agent".to_string())
            .with_parameter("agent_name", "Test".into())
            .with_parameter("max_managed_agents", 150.into()); // > 100 max

        let result = template.validate_parameters(&params).await;
        assert!(result.is_err());

        // Test invalid orchestration_strategy
        let params = TemplateInstantiationParams::new("test-agent".to_string())
            .with_parameter("agent_name", "Test".into())
            .with_parameter("orchestration_strategy", "invalid_strategy".into());

        let result = template.validate_parameters(&params).await;
        assert!(result.is_err());
    }
}
