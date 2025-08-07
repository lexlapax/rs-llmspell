//! ABOUTME: Monitor Agent template for creating agents that monitor systems, agents, and resources
//! ABOUTME: Provides standardized template for monitoring-focused agents with alerting and metrics collection

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

/// Monitoring scope types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MonitoringScope {
    /// Monitor system resources (CPU, memory, disk, network)
    System,
    /// Monitor other agents
    Agent,
    /// Monitor application services
    Application,
    /// Monitor network endpoints
    Network,
    /// Monitor databases
    Database,
    /// Monitor log files and events
    Logs,
    /// Monitor custom metrics
    Custom(String),
}

impl MonitoringScope {
    #[must_use]
    pub fn name(&self) -> String {
        match self {
            Self::System => "system".to_string(),
            Self::Agent => "agent".to_string(),
            Self::Application => "application".to_string(),
            Self::Network => "network".to_string(),
            Self::Database => "database".to_string(),
            Self::Logs => "logs".to_string(),
            Self::Custom(name) => name.clone(),
        }
    }
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

impl AlertSeverity {
    #[must_use]
    pub fn name(&self) -> String {
        match self {
            Self::Info => "info".to_string(),
            Self::Warning => "warning".to_string(),
            Self::Error => "error".to_string(),
            Self::Critical => "critical".to_string(),
        }
    }
}

/// Monitor Agent configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorAgentConfig {
    /// Monitoring scopes this agent covers
    pub monitoring_scopes: Vec<MonitoringScope>,
    /// Monitoring interval in seconds
    pub monitoring_interval: u64,
    /// Enable alerting
    pub enable_alerting: bool,
    /// Alert thresholds configuration
    pub alert_thresholds: HashMap<String, f64>,
    /// Alert severity mapping
    pub severity_mapping: HashMap<String, AlertSeverity>,
    /// Maximum number of alerts per minute
    pub max_alerts_per_minute: u32,
    /// Enable metrics collection
    pub enable_metrics_collection: bool,
    /// Metrics retention period in seconds
    pub metrics_retention_period: u64,
    /// Enable log monitoring
    pub enable_log_monitoring: bool,
    /// Log file patterns to monitor
    pub log_patterns: Vec<String>,
    /// Enable health checks
    pub enable_health_checks: bool,
    /// Health check timeout in seconds
    pub health_check_timeout: u64,
    /// Maximum concurrent monitoring tasks
    pub max_concurrent_tasks: usize,
}

impl Default for MonitorAgentConfig {
    fn default() -> Self {
        Self {
            monitoring_scopes: vec![MonitoringScope::System, MonitoringScope::Agent],
            monitoring_interval: 30, // 30 seconds
            enable_alerting: true,
            alert_thresholds: HashMap::from([
                ("cpu_usage".to_string(), 80.0),
                ("memory_usage".to_string(), 85.0),
                ("disk_usage".to_string(), 90.0),
                ("response_time".to_string(), 5000.0), // 5 seconds
            ]),
            severity_mapping: HashMap::from([
                ("cpu_usage".to_string(), AlertSeverity::Warning),
                ("memory_usage".to_string(), AlertSeverity::Error),
                ("disk_usage".to_string(), AlertSeverity::Critical),
                ("response_time".to_string(), AlertSeverity::Warning),
            ]),
            max_alerts_per_minute: 10,
            enable_metrics_collection: true,
            metrics_retention_period: 86400, // 24 hours
            enable_log_monitoring: false,
            log_patterns: vec!["*.log".to_string(), "logs/*.log".to_string()],
            enable_health_checks: true,
            health_check_timeout: 10,
            max_concurrent_tasks: 5,
        }
    }
}

/// Monitor Agent template implementation
pub struct MonitorAgentTemplate {
    schema: TemplateSchema,
    config: MonitorAgentConfig,
}

impl MonitorAgentTemplate {
    /// Create new Monitor Agent template
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn new() -> Self {
        let metadata = TemplateMetadata {
            id: "monitor_agent".to_string(),
            name: "Monitor Agent".to_string(),
            version: "1.0.0".to_string(),
            description: "Agent template for monitoring systems, agents, and applications"
                .to_string(),
            author: "rs-llmspell".to_string(),
            license: "MIT".to_string(),
            repository: Some("https://github.com/lexlapax/rs-llmspell".to_string()),
            documentation: Some("https://docs.rs/llmspell-agents".to_string()),
            keywords: vec![
                "monitor".to_string(),
                "observability".to_string(),
                "alerting".to_string(),
                "metrics".to_string(),
            ],
            category: TemplateCategory::Monitoring,
            complexity: ComplexityLevel::Intermediate,
        };

        let mut schema = TemplateSchema::new(metadata);

        // Add parameters
        schema = schema
            .with_parameter(ParameterDefinition {
                name: "agent_name".to_string(),
                description: "Human-readable name for the monitor agent".to_string(),
                param_type: ParameterType::String,
                required: true,
                default_value: None,
                constraints: vec![ParameterConstraint::MinLength(1)],
                examples: vec!["System Monitor".into(), "Agent Health Monitor".into()],
            })
            .with_parameter(ParameterDefinition {
                name: "monitoring_scopes".to_string(),
                description: "Types of monitoring this agent will perform".to_string(),
                param_type: ParameterType::Array(Box::new(ParameterType::Enum(vec![
                    "system".to_string(),
                    "agent".to_string(),
                    "application".to_string(),
                    "network".to_string(),
                    "database".to_string(),
                    "logs".to_string(),
                ]))),
                required: false,
                default_value: Some(vec!["system", "agent"].into()),
                constraints: vec![ParameterConstraint::MinLength(1)],
                examples: vec![
                    vec!["system", "application"].into(),
                    vec!["agent", "network"].into(),
                ],
            })
            .with_parameter(ParameterDefinition {
                name: "monitoring_interval".to_string(),
                description: "How often to perform monitoring checks (seconds)".to_string(),
                param_type: ParameterType::Integer,
                required: false,
                default_value: Some(30.into()),
                constraints: vec![
                    ParameterConstraint::MinValue(5.0),
                    ParameterConstraint::MaxValue(3600.0), // 1 hour max
                ],
                examples: vec![10.into(), 30.into(), 60.into()],
            })
            .with_parameter(ParameterDefinition {
                name: "enable_alerting".to_string(),
                description: "Enable alert generation when thresholds are exceeded".to_string(),
                param_type: ParameterType::Boolean,
                required: false,
                default_value: Some(true.into()),
                constraints: vec![],
                examples: vec![true.into(), false.into()],
            })
            .with_parameter(ParameterDefinition {
                name: "cpu_threshold".to_string(),
                description: "CPU usage alert threshold (percentage)".to_string(),
                param_type: ParameterType::Float,
                required: false,
                default_value: Some(80.0.into()),
                constraints: vec![
                    ParameterConstraint::MinValue(0.0),
                    ParameterConstraint::MaxValue(100.0),
                ],
                examples: vec![70.0.into(), 80.0.into(), 90.0.into()],
            })
            .with_parameter(ParameterDefinition {
                name: "memory_threshold".to_string(),
                description: "Memory usage alert threshold (percentage)".to_string(),
                param_type: ParameterType::Float,
                required: false,
                default_value: Some(85.0.into()),
                constraints: vec![
                    ParameterConstraint::MinValue(0.0),
                    ParameterConstraint::MaxValue(100.0),
                ],
                examples: vec![75.0.into(), 85.0.into(), 95.0.into()],
            })
            .with_parameter(ParameterDefinition {
                name: "disk_threshold".to_string(),
                description: "Disk usage alert threshold (percentage)".to_string(),
                param_type: ParameterType::Float,
                required: false,
                default_value: Some(90.0.into()),
                constraints: vec![
                    ParameterConstraint::MinValue(0.0),
                    ParameterConstraint::MaxValue(100.0),
                ],
                examples: vec![80.0.into(), 90.0.into(), 95.0.into()],
            })
            .with_parameter(ParameterDefinition {
                name: "max_alerts_per_minute".to_string(),
                description: "Maximum number of alerts to generate per minute".to_string(),
                param_type: ParameterType::Integer,
                required: false,
                default_value: Some(10.into()),
                constraints: vec![
                    ParameterConstraint::MinValue(1.0),
                    ParameterConstraint::MaxValue(100.0),
                ],
                examples: vec![5.into(), 10.into(), 20.into()],
            })
            .with_parameter(ParameterDefinition {
                name: "enable_metrics_collection".to_string(),
                description: "Enable collection and storage of historical metrics".to_string(),
                param_type: ParameterType::Boolean,
                required: false,
                default_value: Some(true.into()),
                constraints: vec![],
                examples: vec![true.into(), false.into()],
            })
            .with_parameter(ParameterDefinition {
                name: "metrics_retention_hours".to_string(),
                description: "How long to retain metrics data (hours)".to_string(),
                param_type: ParameterType::Integer,
                required: false,
                default_value: Some(24.into()),
                constraints: vec![
                    ParameterConstraint::MinValue(1.0),
                    ParameterConstraint::MaxValue(8760.0), // 1 year
                ],
                examples: vec![24.into(), 168.into(), 720.into()], // 1 day, 1 week, 1 month
            })
            .with_parameter(ParameterDefinition {
                name: "enable_log_monitoring".to_string(),
                description: "Enable monitoring of log files for patterns and errors".to_string(),
                param_type: ParameterType::Boolean,
                required: false,
                default_value: Some(false.into()),
                constraints: vec![],
                examples: vec![true.into(), false.into()],
            })
            .with_parameter(ParameterDefinition {
                name: "log_patterns".to_string(),
                description: "File patterns for log monitoring".to_string(),
                param_type: ParameterType::Array(Box::new(ParameterType::String)),
                required: false,
                default_value: Some(vec!["*.log", "logs/*.log"].into()),
                constraints: vec![],
                examples: vec![
                    vec!["app.log", "error.log"].into(),
                    vec!["logs/*.log", "system/*.log"].into(),
                ],
            });

        // Add tool dependencies for monitoring
        schema = schema
            .with_tool_dependency(ToolDependency {
                name: "system_monitor".to_string(),
                version: Some("1.0.0".to_string()),
                required: true,
                alternatives: vec!["resource_monitor".to_string()],
                config: HashMap::new(),
            })
            .with_tool_dependency(ToolDependency {
                name: "metrics_collector".to_string(),
                version: Some("1.0.0".to_string()),
                required: false,
                alternatives: vec!["telemetry_collector".to_string()],
                config: HashMap::new(),
            })
            .with_tool_dependency(ToolDependency {
                name: "alert_manager".to_string(),
                version: Some("1.0.0".to_string()),
                required: false,
                alternatives: vec!["notification_service".to_string()],
                config: HashMap::new(),
            })
            .with_tool_dependency(ToolDependency {
                name: "health_checker".to_string(),
                version: Some("1.0.0".to_string()),
                required: false,
                alternatives: vec!["ping_tool".to_string()],
                config: HashMap::new(),
            })
            .with_tool_dependency(ToolDependency {
                name: "log_analyzer".to_string(),
                version: Some("1.0.0".to_string()),
                required: false,
                alternatives: vec!["file_watcher".to_string()],
                config: HashMap::new(),
            });

        // Add capability requirements
        schema = schema
            .with_capability_requirement(CapabilityRequirement {
                name: "system_monitoring".to_string(),
                min_level: 7,
                critical: true,
                usage_description: "Monitor system resources and performance metrics".to_string(),
            })
            .with_capability_requirement(CapabilityRequirement {
                name: "alerting".to_string(),
                min_level: 6,
                critical: false,
                usage_description: "Generate and manage alerts based on thresholds".to_string(),
            })
            .with_capability_requirement(CapabilityRequirement {
                name: "metrics_collection".to_string(),
                min_level: 5,
                critical: false,
                usage_description: "Collect, store, and analyze historical metrics".to_string(),
            })
            .with_capability_requirement(CapabilityRequirement {
                name: "health_monitoring".to_string(),
                min_level: 6,
                critical: true,
                usage_description: "Monitor health and availability of services".to_string(),
            })
            .with_capability_requirement(CapabilityRequirement {
                name: "data_analysis".to_string(),
                min_level: 4,
                critical: false,
                usage_description: "Analyze trends and patterns in monitoring data".to_string(),
            });

        // Set resource requirements
        schema = schema.with_resource_requirements(ResourceRequirements {
            memory: Some(128 * 1024 * 1024), // 128MB
            cpu: Some(20),                   // 20% CPU
            disk: Some(100 * 1024 * 1024),   // 100MB for metrics storage
            network: Some(2 * 1024 * 1024),  // 2MB/s
            max_execution_time: Some(0),     // Continuous operation
        });

        // Add configuration
        schema = schema
            .with_config("agent_type", "monitor_agent".into())
            .with_config("supports_real_time_monitoring", true.into())
            .with_config("supports_historical_analysis", true.into())
            .with_config("supports_alerting", true.into());

        Self {
            schema,
            config: MonitorAgentConfig::default(),
        }
    }

    /// Create Monitor Agent template with custom configuration
    #[must_use]
    pub fn with_config(mut self, config: MonitorAgentConfig) -> Self {
        self.config = config;
        self
    }

    /// Create system monitor template focused on system resources
    #[must_use]
    pub fn system_monitor() -> Self {
        let mut template = Self::new();

        // Update configuration for system monitoring
        template.config.monitoring_scopes = vec![MonitoringScope::System];
        template.config.monitoring_interval = 10; // More frequent for system monitoring
        template.config.enable_log_monitoring = false;

        // Update metadata
        template.schema.metadata.id = "monitor_agent_system".to_string();
        template.schema.metadata.name = "System Monitor Agent".to_string();
        template.schema.metadata.description =
            "Specialized agent for monitoring system resources and performance".to_string();

        // Add system-specific configuration
        template.schema = template
            .schema
            .with_config("system_monitoring_mode", true.into())
            .with_config("focus_area", "system_resources".into());

        template
    }

    /// Create application monitor template for application monitoring
    #[must_use]
    pub fn application_monitor() -> Self {
        let mut template = Self::new();

        // Update configuration for application monitoring
        template.config.monitoring_scopes =
            vec![MonitoringScope::Application, MonitoringScope::Logs];
        template.config.enable_log_monitoring = true;
        template.config.monitoring_interval = 60; // Less frequent for applications
        template.config.health_check_timeout = 30;

        // Update resource requirements for log processing
        template.schema = template
            .schema
            .with_resource_requirements(ResourceRequirements {
                memory: Some(256 * 1024 * 1024), // 256MB for log processing
                cpu: Some(30),                   // 30% CPU
                disk: Some(500 * 1024 * 1024),   // 500MB for log storage
                network: Some(5 * 1024 * 1024),  // 5MB/s
                max_execution_time: Some(0),     // Continuous operation
            });

        // Update metadata
        template.schema.metadata.id = "monitor_agent_application".to_string();
        template.schema.metadata.name = "Application Monitor Agent".to_string();
        template.schema.metadata.description =
            "Specialized agent for monitoring applications and analyzing logs".to_string();

        // Add application-specific configuration
        template.schema = template
            .schema
            .with_config("application_monitoring_mode", true.into())
            .with_config("log_analysis_enabled", true.into())
            .with_config("focus_area", "applications".into());

        template
    }

    /// Create lightweight monitor template for basic monitoring
    #[must_use]
    pub fn lightweight() -> Self {
        let mut template = Self::new();

        // Update configuration for lightweight operation
        template.config.monitoring_scopes = vec![MonitoringScope::System];
        template.config.monitoring_interval = 60;
        template.config.enable_metrics_collection = false;
        template.config.enable_log_monitoring = false;
        template.config.max_concurrent_tasks = 2;

        // Update resource requirements
        template.schema = template
            .schema
            .with_resource_requirements(ResourceRequirements {
                memory: Some(64 * 1024 * 1024), // 64MB
                cpu: Some(10),                  // 10% CPU
                disk: Some(10 * 1024 * 1024),   // 10MB
                network: Some(1024 * 1024),     // 1MB/s
                max_execution_time: Some(0),    // Continuous operation
            });

        // Update metadata
        template.schema.metadata.id = "monitor_agent_lightweight".to_string();
        template.schema.metadata.name = "Lightweight Monitor Agent".to_string();
        template.schema.metadata.description =
            "Lightweight monitoring agent for basic system monitoring".to_string();
        template.schema.metadata.complexity = ComplexityLevel::Basic;

        template
    }

    /// Apply parameters to config
    fn apply_parameters_to_config(
        config: &mut MonitorAgentConfig,
        params: &HashMap<String, serde_json::Value>,
    ) {
        if let Some(scopes) = params.get("monitoring_scopes") {
            if let Some(array) = scopes.as_array() {
                config.monitoring_scopes = array
                    .iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| match s {
                        "system" => MonitoringScope::System,
                        "agent" => MonitoringScope::Agent,
                        "application" => MonitoringScope::Application,
                        "network" => MonitoringScope::Network,
                        "database" => MonitoringScope::Database,
                        "logs" => MonitoringScope::Logs,
                        custom => MonitoringScope::Custom(custom.to_string()),
                    })
                    .collect();
            }
        }

        if let Some(interval) = params.get("monitoring_interval") {
            if let Some(value) = interval.as_u64() {
                config.monitoring_interval = value;
            }
        }

        if let Some(enable_alerting) = params.get("enable_alerting") {
            if let Some(value) = enable_alerting.as_bool() {
                config.enable_alerting = value;
            }
        }

        // Update alert thresholds
        if let Some(cpu_threshold) = params.get("cpu_threshold") {
            if let Some(value) = cpu_threshold.as_f64() {
                config
                    .alert_thresholds
                    .insert("cpu_usage".to_string(), value);
            }
        }

        if let Some(memory_threshold) = params.get("memory_threshold") {
            if let Some(value) = memory_threshold.as_f64() {
                config
                    .alert_thresholds
                    .insert("memory_usage".to_string(), value);
            }
        }

        if let Some(disk_threshold) = params.get("disk_threshold") {
            if let Some(value) = disk_threshold.as_f64() {
                config
                    .alert_thresholds
                    .insert("disk_usage".to_string(), value);
            }
        }

        if let Some(max_alerts) = params.get("max_alerts_per_minute") {
            if let Some(value) = max_alerts.as_u64() {
                #[allow(clippy::cast_possible_truncation)]
                let max_alerts = value as u32;
                config.max_alerts_per_minute = max_alerts;
            }
        }

        if let Some(enable_metrics) = params.get("enable_metrics_collection") {
            if let Some(value) = enable_metrics.as_bool() {
                config.enable_metrics_collection = value;
            }
        }

        if let Some(retention_hours) = params.get("metrics_retention_hours") {
            if let Some(value) = retention_hours.as_u64() {
                config.metrics_retention_period = value * 3600; // Convert hours to seconds
            }
        }

        if let Some(enable_log_monitoring) = params.get("enable_log_monitoring") {
            if let Some(value) = enable_log_monitoring.as_bool() {
                config.enable_log_monitoring = value;
            }
        }

        if let Some(patterns) = params.get("log_patterns") {
            if let Some(array) = patterns.as_array() {
                config.log_patterns = array
                    .iter()
                    .filter_map(|v| v.as_str().map(std::string::ToString::to_string))
                    .collect();
            }
        }
    }
}

impl Default for MonitorAgentTemplate {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AgentTemplate for MonitorAgentTemplate {
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
        Self::apply_parameters_to_config(&mut agent_config, &params.parameters);

        // Build final configuration
        let mut final_config = HashMap::new();
        final_config.insert("agent_type".to_string(), "monitor_agent".into());
        final_config.insert(
            "monitoring_scopes".to_string(),
            agent_config
                .monitoring_scopes
                .iter()
                .map(MonitoringScope::name)
                .collect::<Vec<_>>()
                .into(),
        );
        final_config.insert(
            "monitoring_interval".to_string(),
            agent_config.monitoring_interval.into(),
        );
        final_config.insert(
            "enable_alerting".to_string(),
            agent_config.enable_alerting.into(),
        );
        final_config.insert(
            "max_alerts_per_minute".to_string(),
            u64::from(agent_config.max_alerts_per_minute).into(),
        );
        final_config.insert(
            "enable_metrics_collection".to_string(),
            agent_config.enable_metrics_collection.into(),
        );
        final_config.insert(
            "metrics_retention_period".to_string(),
            agent_config.metrics_retention_period.into(),
        );
        final_config.insert(
            "enable_log_monitoring".to_string(),
            agent_config.enable_log_monitoring.into(),
        );
        final_config.insert(
            "max_concurrent_tasks".to_string(),
            (agent_config.max_concurrent_tasks as u64).into(),
        );

        // Add alert thresholds
        for (key, value) in &agent_config.alert_thresholds {
            final_config.insert(format!("threshold_{key}"), (*value).into());
        }

        // Add log patterns if enabled
        if agent_config.enable_log_monitoring {
            final_config.insert(
                "log_patterns".to_string(),
                agent_config
                    .log_patterns
                    .iter()
                    .map(std::string::String::as_str)
                    .collect::<Vec<_>>()
                    .into(),
            );
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
        // In a real implementation, this would create the actual monitor agent
        let mock_agent =
            MockMonitorAgent::new(params.agent_id.clone(), agent_name, final_config.clone());

        Ok(TemplateInstantiationResult {
            agent: Box::new(mock_agent),
            template_schema: self.schema.clone(),
            applied_parameters: params.parameters,
            applied_config: final_config,
        })
    }

    fn clone_template(&self) -> Box<dyn AgentTemplate> {
        Box::new(Self {
            schema: self.schema.clone(),
            config: self.config.clone(),
        })
    }
}

/// Mock monitor agent for testing (replace with actual implementation)
#[allow(dead_code)]
struct MockMonitorAgent {
    id: String,
    name: String,
    config: HashMap<String, serde_json::Value>,
    metadata: ComponentMetadata,
}

impl MockMonitorAgent {
    fn new(id: String, name: String, config: HashMap<String, serde_json::Value>) -> Self {
        let metadata = ComponentMetadata {
            id: ComponentId::from_name(&id),
            name: name.clone(),
            version: Version::new(1, 0, 0),
            description: "Mock monitor agent for template testing".to_string(),
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
impl BaseAgent for MockMonitorAgent {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute(
        &self,
        _input: AgentInput,
        _context: ExecutionContext,
    ) -> Result<AgentOutput, LLMSpellError> {
        Ok(AgentOutput {
            text: "Mock monitor agent execution result".to_string(),
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
            text: format!("Monitor error handled: {error}"),
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
    async fn test_monitor_agent_template_creation() {
        let template = MonitorAgentTemplate::new();

        assert_eq!(template.schema().metadata.id, "monitor_agent");
        assert_eq!(template.category(), &TemplateCategory::Monitoring);
        assert_eq!(template.complexity(), &ComplexityLevel::Intermediate);

        let required_params = template.schema().required_parameters();
        assert_eq!(required_params.len(), 1);
        assert_eq!(required_params[0].name, "agent_name");
    }
    #[tokio::test]
    async fn test_system_monitor() {
        let template = MonitorAgentTemplate::system_monitor();

        assert_eq!(template.config.monitoring_scopes.len(), 1);
        assert!(matches!(
            template.config.monitoring_scopes[0],
            MonitoringScope::System
        ));
        assert_eq!(template.config.monitoring_interval, 10);
        assert!(!template.config.enable_log_monitoring);

        // Check system-specific configuration
        let system_mode = template
            .schema()
            .template_config
            .get("system_monitoring_mode");
        assert_eq!(system_mode, Some(&true.into()));
    }
    #[tokio::test]
    async fn test_application_monitor() {
        let template = MonitorAgentTemplate::application_monitor();

        assert_eq!(template.config.monitoring_scopes.len(), 2);
        assert!(template
            .config
            .monitoring_scopes
            .contains(&MonitoringScope::Application));
        assert!(template
            .config
            .monitoring_scopes
            .contains(&MonitoringScope::Logs));
        assert!(template.config.enable_log_monitoring);

        // Check application-specific configuration
        let app_mode = template
            .schema()
            .template_config
            .get("application_monitoring_mode");
        assert_eq!(app_mode, Some(&true.into()));
    }
    #[tokio::test]
    async fn test_lightweight_monitor() {
        let template = MonitorAgentTemplate::lightweight();

        assert_eq!(template.config.monitoring_interval, 60);
        assert!(!template.config.enable_metrics_collection);
        assert!(!template.config.enable_log_monitoring);
        assert_eq!(template.config.max_concurrent_tasks, 2);
        assert_eq!(template.complexity(), &ComplexityLevel::Basic);
    }
    #[tokio::test]
    async fn test_monitoring_scopes() {
        assert_eq!(MonitoringScope::System.name(), "system");
        assert_eq!(MonitoringScope::Agent.name(), "agent");
        assert_eq!(MonitoringScope::Custom("test".to_string()).name(), "test");
    }
    #[tokio::test]
    async fn test_alert_severities() {
        assert_eq!(AlertSeverity::Info.name(), "info");
        assert_eq!(AlertSeverity::Warning.name(), "warning");
        assert_eq!(AlertSeverity::Error.name(), "error");
        assert_eq!(AlertSeverity::Critical.name(), "critical");
    }
    #[tokio::test]
    async fn test_parameter_validation() {
        let template = MonitorAgentTemplate::new();

        // Test missing required parameter
        let params = TemplateInstantiationParams::new("test-agent".to_string());
        let result = template.validate_parameters(&params).await;
        assert!(result.is_err());

        // Test valid parameters
        let params = TemplateInstantiationParams::new("test-agent".to_string())
            .with_parameter("agent_name", "Test Monitor".into())
            .with_parameter("monitoring_scopes", vec!["system", "agent"].into())
            .with_parameter("monitoring_interval", 30.into())
            .with_parameter("cpu_threshold", 75.0.into());

        let result = template.validate_parameters(&params).await;
        assert!(result.is_ok());
    }
    #[tokio::test]
    async fn test_threshold_constraints() {
        let template = MonitorAgentTemplate::new();

        // Test invalid CPU threshold (> 100)
        let params = TemplateInstantiationParams::new("test-agent".to_string())
            .with_parameter("agent_name", "Test".into())
            .with_parameter("cpu_threshold", 150.0.into());

        let result = template.validate_parameters(&params).await;
        assert!(result.is_err());

        // Test invalid monitoring interval (too low)
        let params = TemplateInstantiationParams::new("test-agent".to_string())
            .with_parameter("agent_name", "Test".into())
            .with_parameter("monitoring_interval", 1.into()); // < 5 seconds minimum

        let result = template.validate_parameters(&params).await;
        assert!(result.is_err());
    }
    #[tokio::test]
    async fn test_required_tools() {
        let template = MonitorAgentTemplate::new();

        let required_tools = template.required_tools();
        assert!(required_tools.contains(&"system_monitor".to_string()));

        let optional_tools = template.optional_tools();
        assert!(optional_tools.contains(&"metrics_collector".to_string()));
        assert!(optional_tools.contains(&"alert_manager".to_string()));
        assert!(optional_tools.contains(&"health_checker".to_string()));
        assert!(optional_tools.contains(&"log_analyzer".to_string()));
    }
    #[tokio::test]
    async fn test_capability_support() {
        let template = MonitorAgentTemplate::new();

        assert!(template.supports_capability("system_monitoring"));
        assert!(template.supports_capability("alerting"));
        assert!(template.supports_capability("metrics_collection"));
        assert!(template.supports_capability("health_monitoring"));
        assert!(template.supports_capability("data_analysis"));
        assert!(!template.supports_capability("nonexistent_capability"));
    }
}
