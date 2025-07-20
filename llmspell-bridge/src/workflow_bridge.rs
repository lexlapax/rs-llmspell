//! ABOUTME: Workflow bridge for script-to-workflow communication
//! ABOUTME: Provides unified interface for scripts to interact with workflows

use crate::workflow_performance::{ExecutionCache, OptimizedConverter, PerformanceMetrics};
use crate::workflows::{WorkflowDiscovery, WorkflowExecutor, WorkflowFactory, WorkflowInfo};
use crate::ComponentRegistry;
use llmspell_core::{LLMSpellError, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Type alias for active workflow storage
type ActiveWorkflowMap = HashMap<String, Arc<Box<dyn WorkflowExecutor>>>;

/// Bridge between scripts and workflows
pub struct WorkflowBridge {
    /// Workflow discovery service
    discovery: Arc<WorkflowDiscovery>,
    /// Component registry for script access
    _registry: Arc<ComponentRegistry>,
    /// Active workflow instances
    active_workflows: Arc<RwLock<ActiveWorkflowMap>>,
    /// Workflow execution history
    execution_history: Arc<RwLock<Vec<WorkflowExecutionRecord>>>,
    /// Bridge metrics
    metrics: Arc<BridgeMetrics>,
    /// Performance optimizations
    converter: Arc<OptimizedConverter>,
    execution_cache: Arc<ExecutionCache>,
    perf_metrics: Arc<PerformanceMetrics>,
}

/// Record of workflow execution
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

/// Bridge performance metrics
#[derive(Debug, Default)]
struct BridgeMetrics {
    /// Total workflows created
    workflows_created: std::sync::atomic::AtomicU64,
    /// Total workflow executions
    workflow_executions: std::sync::atomic::AtomicU64,
    /// Total successful executions
    successful_executions: std::sync::atomic::AtomicU64,
    /// Total failed executions
    failed_executions: std::sync::atomic::AtomicU64,
    /// Average execution time in milliseconds
    avg_execution_time_ms: std::sync::atomic::AtomicU64,
}

impl WorkflowBridge {
    /// Create a new workflow bridge
    pub fn new(registry: Arc<ComponentRegistry>) -> Self {
        Self {
            discovery: Arc::new(WorkflowDiscovery::new()),
            _registry: registry,
            active_workflows: Arc::new(RwLock::new(HashMap::new())),
            execution_history: Arc::new(RwLock::new(Vec::new())),
            metrics: Arc::new(BridgeMetrics::default()),
            converter: Arc::new(OptimizedConverter::new()),
            execution_cache: Arc::new(ExecutionCache::new(100)),
            perf_metrics: Arc::new(PerformanceMetrics::new()),
        }
    }

    /// List available workflow types
    pub async fn list_workflow_types(&self) -> Vec<String> {
        self.discovery.list_workflow_types()
    }

    /// Get information about all workflow types
    pub async fn get_all_workflow_info(&self) -> Vec<WorkflowInfo> {
        self.discovery.get_all_workflow_info()
    }

    /// Get information about a specific workflow type
    pub async fn get_workflow_info(&self, workflow_type: &str) -> Option<WorkflowInfo> {
        self.discovery.get_workflow_info(workflow_type).cloned()
    }

    /// Create a new workflow instance
    pub async fn create_workflow(
        &self,
        workflow_type: &str,
        params: serde_json::Value,
    ) -> Result<String> {
        let start = std::time::Instant::now();

        debug!(
            "Creating workflow of type: {} with params: {:?}",
            workflow_type, params
        );

        // Fast parameter validation
        if !self.converter.validate_params(workflow_type, &params) {
            return Err(LLMSpellError::Component {
                message: format!("Invalid parameters for workflow type: {}", workflow_type),
                source: None,
            });
        }

        // Create the workflow using factory
        let workflow = WorkflowFactory::create_workflow(workflow_type, params).await?;

        // Generate unique ID
        let workflow_id = format!("workflow_{}", uuid::Uuid::new_v4());

        // Store the workflow
        let mut workflows = self.active_workflows.write().await;
        workflows.insert(workflow_id.clone(), Arc::new(workflow));

        // Update metrics
        self.metrics
            .workflows_created
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed);

        // Record performance
        let duration_ms = start.elapsed().as_millis() as u64;
        self.perf_metrics.record_operation(duration_ms);

        info!(
            "Created workflow '{}' of type '{}' in {}ms",
            workflow_id, workflow_type, duration_ms
        );
        Ok(workflow_id)
    }

    /// Execute a workflow by ID
    pub async fn execute_workflow(
        &self,
        workflow_id: &str,
        input: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let start_time = chrono::Utc::now();
        let start_instant = std::time::Instant::now();

        debug!(
            "Executing workflow '{}' with input: {:?}",
            workflow_id, input
        );

        // Check cache first
        if let Some(cached_result) = self.execution_cache.get(workflow_id) {
            debug!("Using cached result for workflow '{}'", workflow_id);
            self.perf_metrics.record_operation(1); // Cache hit is very fast
            return Ok(cached_result);
        }

        // Get the workflow
        let workflows = self.active_workflows.read().await;
        let workflow = workflows
            .get(workflow_id)
            .ok_or_else(|| LLMSpellError::Component {
                message: format!("No active workflow with ID: {}", workflow_id),
                source: None,
            })?
            .clone();
        drop(workflows);

        // Execute the workflow
        let workflow_type = workflow.workflow_type().to_string();

        match workflow.execute(input).await {
            Ok(output) => {
                let duration_ms = start_instant.elapsed().as_millis() as u64;

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
                self.update_metrics(true, duration_ms).await;

                // Cache successful result
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
                let duration_ms = start_instant.elapsed().as_millis() as u64;

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
                self.update_metrics(false, duration_ms).await;

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

    /// Execute a workflow and immediately return (one-shot execution)
    pub async fn execute_workflow_oneshot(
        &self,
        workflow_type: &str,
        params: serde_json::Value,
        input: serde_json::Value,
    ) -> Result<serde_json::Value> {
        // Create workflow
        let workflow_id = self.create_workflow(workflow_type, params).await?;

        // Execute workflow
        let result = self.execute_workflow(&workflow_id, input).await;

        // Clean up workflow
        self.remove_workflow(&workflow_id).await?;

        result
    }

    /// Remove a workflow instance
    pub async fn remove_workflow(&self, workflow_id: &str) -> Result<()> {
        let mut workflows = self.active_workflows.write().await;
        workflows
            .remove(workflow_id)
            .ok_or_else(|| LLMSpellError::Component {
                message: format!("No active workflow with ID: {}", workflow_id),
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

    /// List all active workflows with detailed info
    pub async fn list_workflows(&self) -> Vec<(String, crate::workflows::WorkflowInfo)> {
        let workflows = self.active_workflows.read().await;
        workflows
            .iter()
            .map(|(id, workflow)| {
                let info = crate::workflows::WorkflowInfo {
                    workflow_type: workflow.workflow_type().to_string(),
                    description: format!("Active workflow: {}", workflow.name()),
                    features: vec![],
                    required_params: vec![],
                    optional_params: vec![],
                };
                (id.clone(), info)
            })
            .collect()
    }

    /// Discover available workflow types
    pub async fn discover_workflow_types(&self) -> Vec<(String, crate::workflows::WorkflowInfo)> {
        self.discovery.get_workflow_types()
    }

    /// Get bridge metrics
    pub async fn get_bridge_metrics(&self) -> serde_json::Value {
        use std::sync::atomic::Ordering;

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

    async fn update_metrics(&self, success: bool, duration_ms: u64) {
        use std::sync::atomic::Ordering;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_workflow_bridge_creation() {
        let registry = Arc::new(ComponentRegistry::new());
        let bridge = WorkflowBridge::new(registry);

        // Test listing workflow types
        let types = bridge.list_workflow_types().await;
        assert_eq!(types.len(), 4);
        assert!(types.contains(&"sequential".to_string()));
    }

    #[tokio::test]
    async fn test_workflow_info() {
        let registry = Arc::new(ComponentRegistry::new());
        let bridge = WorkflowBridge::new(registry);

        // Test getting workflow info
        let info = bridge.get_workflow_info("sequential").await.unwrap();
        assert_eq!(info.workflow_type, "sequential");
        assert!(info.required_params.contains(&"steps".to_string()));

        // Test getting all workflow info
        let all_info = bridge.get_all_workflow_info().await;
        assert_eq!(all_info.len(), 4);
    }

    #[tokio::test]
    async fn test_bridge_metrics() {
        let registry = Arc::new(ComponentRegistry::new());
        let bridge = WorkflowBridge::new(registry);

        // Get initial metrics
        let metrics = bridge.get_bridge_metrics().await;
        assert_eq!(metrics["workflows_created"], 0);
        assert_eq!(metrics["workflow_executions"], 0);
        assert_eq!(metrics["active_workflows"], 0);
    }
}
