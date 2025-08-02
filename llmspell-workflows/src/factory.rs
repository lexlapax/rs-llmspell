//! ABOUTME: Workflow factory trait and implementations for creating workflows
//! ABOUTME: Provides standardized workflow creation matching agent factory pattern

use crate::{
    conditional::ConditionalWorkflow,
    parallel::{ParallelConfig, ParallelWorkflow},
    r#loop::{LoopConfig, LoopWorkflow},
    sequential::SequentialWorkflow,
    traits::WorkflowStep,
    types::WorkflowConfig,
};
use async_trait::async_trait;
use llmspell_core::{traits::base_agent::BaseAgent, LLMSpellError, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Workflow creation parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowParams {
    /// Name of the workflow
    pub name: String,
    /// Type of workflow to create
    pub workflow_type: WorkflowType,
    /// Base workflow configuration
    pub config: WorkflowConfig,
    /// Type-specific configuration
    pub type_config: serde_json::Value,
}

/// Supported workflow types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowType {
    Sequential,
    Parallel,
    Conditional,
    Loop,
}

/// Factory trait for creating workflows
#[async_trait]
pub trait WorkflowFactory: Send + Sync {
    /// Create a workflow instance based on parameters
    async fn create_workflow(
        &self,
        params: WorkflowParams,
    ) -> Result<Arc<dyn BaseAgent + Send + Sync>>;

    /// List available workflow types
    fn available_types(&self) -> Vec<WorkflowType>;
    
    /// List available workflow types as strings
    fn list_workflow_types(&self) -> Vec<String> {
        self.available_types()
            .into_iter()
            .map(|t| match t {
                WorkflowType::Sequential => "sequential".to_string(),
                WorkflowType::Parallel => "parallel".to_string(),
                WorkflowType::Conditional => "conditional".to_string(),
                WorkflowType::Loop => "loop".to_string(),
            })
            .collect()
    }

    /// Get default configuration for a workflow type
    fn default_config(&self, workflow_type: &WorkflowType) -> WorkflowConfig;
}

/// Default workflow factory implementation
pub struct DefaultWorkflowFactory;

impl DefaultWorkflowFactory {
    pub fn new() -> Self {
        Self
    }
    
    /// Create workflow from string type name (convenience method)
    pub async fn create_from_type(
        &self,
        workflow_type: &str,
        name: String,
        config: WorkflowConfig,
        type_config: serde_json::Value,
    ) -> Result<Arc<dyn BaseAgent + Send + Sync>> {
        let workflow_type = match workflow_type {
            "sequential" => WorkflowType::Sequential,
            "parallel" => WorkflowType::Parallel,
            "conditional" => WorkflowType::Conditional,
            "loop" => WorkflowType::Loop,
            _ => {
                return Err(LLMSpellError::Validation {
                    message: format!("Unknown workflow type: {}", workflow_type),
                    field: Some("workflow_type".to_string()),
                })
            }
        };
        
        let params = WorkflowParams {
            name,
            workflow_type,
            config,
            type_config,
        };
        
        self.create_workflow(params).await
    }
}

impl Default for DefaultWorkflowFactory {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl WorkflowFactory for DefaultWorkflowFactory {
    async fn create_workflow(
        &self,
        params: WorkflowParams,
    ) -> Result<Arc<dyn BaseAgent + Send + Sync>> {
        match params.workflow_type {
            WorkflowType::Sequential => {
                let workflow = SequentialWorkflow::new(params.name, params.config);
                Ok(Arc::new(workflow))
            }
            WorkflowType::Parallel => {
                let config: ParallelConfig =
                    serde_json::from_value(params.type_config).map_err(|e| {
                        LLMSpellError::Validation {
                            message: format!("Invalid parallel config: {}", e),
                            field: Some("type_config".to_string()),
                        }
                    })?;
                let workflow = ParallelWorkflow::new(params.name, vec![], config, params.config);
                Ok(Arc::new(workflow))
            }
            WorkflowType::Conditional => {
                // Note: ConditionalWorkflowConfig would need to be set via builder pattern
                // For now, we create with default conditional config
                let workflow = ConditionalWorkflow::new(params.name, params.config);
                Ok(Arc::new(workflow))
            }
            WorkflowType::Loop => {
                let config: LoopConfig =
                    serde_json::from_value(params.type_config).map_err(|e| {
                        LLMSpellError::Validation {
                            message: format!("Invalid loop config: {}", e),
                            field: Some("type_config".to_string()),
                        }
                    })?;
                let workflow = LoopWorkflow::new(params.name, config, params.config);
                Ok(Arc::new(workflow))
            }
        }
    }

    fn available_types(&self) -> Vec<WorkflowType> {
        vec![
            WorkflowType::Sequential,
            WorkflowType::Parallel,
            WorkflowType::Conditional,
            WorkflowType::Loop,
        ]
    }

    fn default_config(&self, workflow_type: &WorkflowType) -> WorkflowConfig {
        match workflow_type {
            WorkflowType::Sequential => WorkflowConfig::default(),
            WorkflowType::Parallel => WorkflowConfig {
                continue_on_error: true,
                ..Default::default()
            },
            WorkflowType::Conditional => WorkflowConfig::default(),
            WorkflowType::Loop => WorkflowConfig {
                max_retry_attempts: 1, // Loops handle their own iteration
                ..Default::default()
            },
        }
    }
}

/// Workflow template for common configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTemplate {
    pub name: String,
    pub description: String,
    pub workflow_type: WorkflowType,
    pub config: WorkflowConfig,
    pub type_config: serde_json::Value,
    pub steps: Vec<WorkflowStep>,
}

/// Template-based workflow factory
pub struct TemplateWorkflowFactory {
    templates: std::collections::HashMap<String, WorkflowTemplate>,
    base_factory: DefaultWorkflowFactory,
}

impl TemplateWorkflowFactory {
    pub fn new() -> Self {
        let mut factory = Self {
            templates: std::collections::HashMap::new(),
            base_factory: DefaultWorkflowFactory::new(),
        };

        // Add default templates
        factory.add_default_templates();
        factory
    }

    /// Add a workflow template
    pub fn add_template(&mut self, template_name: String, template: WorkflowTemplate) {
        self.templates.insert(template_name, template);
    }

    /// Get a template by name
    pub fn get_template(&self, template_name: &str) -> Option<&WorkflowTemplate> {
        self.templates.get(template_name)
    }

    /// List available templates
    pub fn list_templates(&self) -> Vec<String> {
        self.templates.keys().cloned().collect()
    }

    /// Create workflow from template
    pub async fn create_from_template(
        &self,
        template_name: &str,
        workflow_name: String,
    ) -> Result<Arc<dyn BaseAgent + Send + Sync>> {
        let template =
            self.templates
                .get(template_name)
                .ok_or_else(|| LLMSpellError::Resource {
                    message: format!("Workflow template '{}' not found", template_name),
                    resource_type: Some("workflow_template".to_string()),
                    source: None,
                })?;

        let params = WorkflowParams {
            name: workflow_name,
            workflow_type: template.workflow_type.clone(),
            config: template.config.clone(),
            type_config: template.type_config.clone(),
        };

        let workflow = self.base_factory.create_workflow(params).await?;

        // Note: Adding template steps would require either:
        // 1. Making workflows mutable after creation
        // 2. Using a builder pattern with the steps
        // 3. Creating a workflow-specific factory method
        // For now, templates provide configuration but steps must be added separately

        Ok(workflow)
    }

    fn add_default_templates(&mut self) {
        use serde_json::json;

        // Data processing pipeline template
        self.add_template(
            "data_pipeline".to_string(),
            WorkflowTemplate {
                name: "Data Processing Pipeline".to_string(),
                description: "Sequential workflow for data extraction, transformation, and loading"
                    .to_string(),
                workflow_type: WorkflowType::Sequential,
                config: WorkflowConfig {
                    max_execution_time: Some(std::time::Duration::from_secs(300)),
                    continue_on_error: false,
                    ..Default::default()
                },
                type_config: json!({}),
                steps: vec![],
            },
        );

        // Parallel analysis template
        self.add_template(
            "parallel_analysis".to_string(),
            WorkflowTemplate {
                name: "Parallel Analysis".to_string(),
                description: "Analyze data using multiple approaches in parallel".to_string(),
                workflow_type: WorkflowType::Parallel,
                config: WorkflowConfig {
                    max_execution_time: Some(std::time::Duration::from_secs(600)),
                    continue_on_error: true,
                    ..Default::default()
                },
                type_config: json!({
                    "max_concurrency": 4,
                    "fail_fast": false,
                }),
                steps: vec![],
            },
        );

        // Retry with backoff template
        self.add_template(
            "retry_with_backoff".to_string(),
            WorkflowTemplate {
                name: "Retry with Backoff".to_string(),
                description: "Loop workflow with exponential backoff for retries".to_string(),
                workflow_type: WorkflowType::Loop,
                config: WorkflowConfig {
                    exponential_backoff: true,
                    retry_delay_ms: 1000,
                    ..Default::default()
                },
                type_config: json!({
                    "iterator": {
                        "type": "range",
                        "start": 1,
                        "end": 5,
                        "step": 1
                    },
                    "aggregation": "last_only",
                    "continue_on_error": false,
                }),
                steps: vec![],
            },
        );

        // Conditional routing template
        self.add_template(
            "conditional_router".to_string(),
            WorkflowTemplate {
                name: "Conditional Router".to_string(),
                description: "Route execution based on conditions".to_string(),
                workflow_type: WorkflowType::Conditional,
                config: WorkflowConfig::default(),
                type_config: json!({
                    "execute_all_matching": false,
                    "execute_default_on_no_match": true,
                    "short_circuit_evaluation": true,
                }),
                steps: vec![],
            },
        );
    }
}

impl Default for TemplateWorkflowFactory {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl WorkflowFactory for TemplateWorkflowFactory {
    async fn create_workflow(
        &self,
        params: WorkflowParams,
    ) -> Result<Arc<dyn BaseAgent + Send + Sync>> {
        self.base_factory.create_workflow(params).await
    }

    fn available_types(&self) -> Vec<WorkflowType> {
        self.base_factory.available_types()
    }

    fn default_config(&self, workflow_type: &WorkflowType) -> WorkflowConfig {
        self.base_factory.default_config(workflow_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_default_factory_creation() {
        let factory = DefaultWorkflowFactory::new();

        let params = WorkflowParams {
            name: "test_sequential".to_string(),
            workflow_type: WorkflowType::Sequential,
            config: WorkflowConfig::default(),
            type_config: serde_json::Value::Object(serde_json::Map::new()),
        };

        let workflow = factory.create_workflow(params).await.unwrap();
        assert_eq!(workflow.metadata().name, "test_sequential");
    }

    #[test]
    fn test_available_types() {
        let factory = DefaultWorkflowFactory::new();
        let types = factory.available_types();
        assert_eq!(types.len(), 4);
        assert!(types.contains(&WorkflowType::Sequential));
        assert!(types.contains(&WorkflowType::Parallel));
        assert!(types.contains(&WorkflowType::Conditional));
        assert!(types.contains(&WorkflowType::Loop));
    }

    #[tokio::test]
    async fn test_template_factory() {
        let factory = TemplateWorkflowFactory::new();

        // Check default templates
        let templates = factory.list_templates();
        assert!(templates.contains(&"data_pipeline".to_string()));
        assert!(templates.contains(&"parallel_analysis".to_string()));
        assert!(templates.contains(&"retry_with_backoff".to_string()));
        assert!(templates.contains(&"conditional_router".to_string()));

        // Create from template
        let workflow = factory
            .create_from_template("data_pipeline", "my_pipeline".to_string())
            .await
            .unwrap();
        assert_eq!(workflow.metadata().name, "my_pipeline");
    }

    #[test]
    fn test_default_configs() {
        let factory = DefaultWorkflowFactory::new();

        // Sequential should have default config
        let seq_config = factory.default_config(&WorkflowType::Sequential);
        assert!(!seq_config.continue_on_error);

        // Parallel should continue on error
        let par_config = factory.default_config(&WorkflowType::Parallel);
        assert!(par_config.continue_on_error);

        // Loop should have reduced retries
        let loop_config = factory.default_config(&WorkflowType::Loop);
        assert_eq!(loop_config.max_retry_attempts, 1);
    }
}
