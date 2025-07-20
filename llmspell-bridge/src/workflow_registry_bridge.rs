//! ABOUTME: Workflow registry integration for script bridge
//! ABOUTME: Manages workflow instances and provides registry access to scripts

use crate::workflow_bridge::WorkflowBridge;
use crate::workflows::WorkflowExecutor;
use llmspell_core::{LLMSpellError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

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
    total_registered: std::sync::atomic::AtomicU64,
    /// Total templates registered
    total_templates: std::sync::atomic::AtomicU64,
    /// Total workflow executions through registry
    total_executions: std::sync::atomic::AtomicU64,
}

impl WorkflowRegistry {
    /// Create a new workflow registry
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
                message: format!("Workflow '{}' already registered", id),
                source: None,
            });
        }

        workflows.insert(id, registration);
        self.metrics
            .total_registered
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        Ok(())
    }

    /// Unregister a workflow
    pub async fn unregister_workflow(&self, id: &str) -> Result<()> {
        let mut workflows = self.workflows.write().await;
        workflows
            .remove(id)
            .ok_or_else(|| LLMSpellError::Component {
                message: format!("No workflow registered with ID: {}", id),
                source: None,
            })?;

        Ok(())
    }

    /// Get a workflow by ID
    pub async fn get_workflow(&self, id: &str) -> Result<Arc<Box<dyn WorkflowExecutor>>> {
        let workflows = self.workflows.read().await;
        workflows
            .get(id)
            .map(|reg| reg.workflow.clone())
            .ok_or_else(|| LLMSpellError::Component {
                message: format!("No workflow found with ID: {}", id),
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
                message: format!("No workflow registration found with ID: {}", id),
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

        self.metrics
            .total_executions
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        Ok(())
    }

    /// Get workflow usage statistics
    pub async fn get_usage_stats(&self, id: &str) -> Result<WorkflowUsageStats> {
        let workflows = self.workflows.read().await;
        workflows
            .get(id)
            .map(|reg| reg.usage_stats.clone())
            .ok_or_else(|| LLMSpellError::Component {
                message: format!("No workflow found with ID: {}", id),
                source: None,
            })
    }

    /// Register a workflow template
    pub async fn register_template(&self, template: WorkflowTemplate) -> Result<()> {
        let mut templates = self.templates.write().await;
        templates.insert(template.id.clone(), template);
        self.metrics
            .total_templates
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }

    /// Get a workflow template
    pub async fn get_template(&self, template_id: &str) -> Result<WorkflowTemplate> {
        let templates = self.templates.read().await;
        templates
            .get(template_id)
            .cloned()
            .ok_or_else(|| LLMSpellError::Component {
                message: format!("No workflow template found with ID: {}", template_id),
                source: None,
            })
    }

    /// List all templates
    pub async fn list_templates(&self) -> Vec<WorkflowTemplate> {
        let templates = self.templates.read().await;
        templates.values().cloned().collect()
    }

    /// Create workflow from template
    pub async fn create_from_template(
        &self,
        template_id: &str,
        params: serde_json::Value,
        bridge: &WorkflowBridge,
    ) -> Result<String> {
        let template = self.get_template(template_id).await?;

        // Merge template defaults with provided params
        let mut config = template.default_config.clone();
        if let (Some(config_obj), Some(params_obj)) = (config.as_object_mut(), params.as_object()) {
            for (key, value) in params_obj {
                config_obj.insert(key.clone(), value.clone());
            }
        }

        // Create workflow through bridge
        bridge
            .create_workflow(&template.workflow_type, config)
            .await
    }

    /// Register default workflow templates
    fn register_default_templates(&mut self) {
        use std::sync::atomic::Ordering;

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
            self.metrics.total_templates.fetch_add(1, Ordering::Relaxed);
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

#[cfg(test)]
mod tests {
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
