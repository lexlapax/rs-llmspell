use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowNode {
    pub id: String,
    pub label: String,
    #[serde(rename = "type")]
    pub type_: String, // "agent", "tool", "decision", etc.
    pub status: NodeStatus,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub duration_ms: Option<u64>,
    pub output: Option<serde_json::Value>,
    pub error: Option<String>,
    pub position: NodePosition, // For UI layout
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodePosition {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NodeStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Skipped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowLink {
    pub source: String,        // Node ID
    pub target: String,        // Node ID
    pub label: Option<String>, // e.g., "on_success", "on_failure"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowExecution {
    pub nodes: Vec<WorkflowNode>,
    pub links: Vec<WorkflowLink>,
    pub status: String, // "running", "completed", "failed"
    pub started_at: chrono::DateTime<chrono::Utc>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Clone)]
pub struct WorkflowGraphBuilder {
    nodes: Arc<RwLock<Vec<WorkflowNode>>>,
    links: Arc<RwLock<Vec<WorkflowLink>>>,
    started_at: chrono::DateTime<chrono::Utc>,
}

impl Default for WorkflowGraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl WorkflowGraphBuilder {
    pub fn new() -> Self {
        Self {
            nodes: Arc::new(RwLock::new(Vec::new())),
            links: Arc::new(RwLock::new(Vec::new())),
            started_at: chrono::Utc::now(),
        }
    }

    pub async fn add_node(&self, node: WorkflowNode) {
        self.nodes.write().await.push(node);
    }

    pub async fn add_link(&self, link: WorkflowLink) {
        self.links.write().await.push(link);
    }

    pub async fn update_node_status(
        &self,
        node_id: &str,
        status: NodeStatus,
        output: Option<serde_json::Value>,
        error: Option<String>,
    ) {
        let mut nodes = self.nodes.write().await;
        if let Some(node) = nodes.iter_mut().find(|n| n.id == node_id) {
            node.status = status.clone();
            if output.is_some() {
                node.output = output;
            }
            if error.is_some() {
                node.error = error;
            }
            if node.started_at.is_none() {
                node.started_at = Some(chrono::Utc::now());
            }
            if matches!(status, NodeStatus::Completed | NodeStatus::Failed) {
                node.completed_at = Some(chrono::Utc::now());
                if let Some(started) = node.started_at {
                    node.duration_ms =
                        Some((chrono::Utc::now() - started).num_milliseconds() as u64);
                }
            }
        }
    }

    pub async fn build(&self) -> WorkflowExecution {
        WorkflowExecution {
            nodes: self.nodes.read().await.clone(),
            links: self.links.read().await.clone(),
            status: self.compute_status().await,
            started_at: self.started_at,
            completed_at: Some(chrono::Utc::now()),
        }
    }

    async fn compute_status(&self) -> String {
        let nodes = self.nodes.read().await;
        if nodes.iter().any(|n| matches!(n.status, NodeStatus::Failed)) {
            "failed".to_string()
        } else if nodes
            .iter()
            .all(|n| matches!(n.status, NodeStatus::Completed))
        {
            "completed".to_string()
        } else {
            "running".to_string()
        }
    }

    pub async fn upsert_node_from_event(&self, event: llmspell_templates::core::StepEvent) {
        let mut new_link = None;

        {
            let mut nodes = self.nodes.write().await;

            // Find existing node
            if let Some(node) = nodes.iter_mut().find(|n| n.id == event.step_id) {
                // Update
                node.status = match event.status.as_str() {
                    "running" => NodeStatus::Running,
                    "completed" => NodeStatus::Completed,
                    "failed" => NodeStatus::Failed,
                    _ => NodeStatus::Pending,
                };
                if event.output.is_some() {
                    node.output = event.output;
                }
                if event.error.is_some() {
                    node.error = event.error;
                }

                if matches!(node.status, NodeStatus::Completed | NodeStatus::Failed) {
                    node.completed_at = Some(event.timestamp);
                    if let Some(started) = node.started_at {
                        node.duration_ms =
                            Some((event.timestamp - started).num_milliseconds() as u64);
                    }
                }
            } else {
                // New node
                let status = match event.status.as_str() {
                    "running" => NodeStatus::Running,
                    "completed" => NodeStatus::Completed,
                    "failed" => NodeStatus::Failed,
                    _ => NodeStatus::Pending,
                };

                // Simple vertical layout
                let count = nodes.len() as f64;
                let position = NodePosition {
                    x: 400.0,
                    y: 100.0 + (count * 120.0),
                };

                let new_node = WorkflowNode {
                    id: event.step_id.clone(),
                    label: event.label.clone(),
                    type_: event.step_type.clone(),
                    status,
                    started_at: Some(event.timestamp),
                    completed_at: if event.status == "completed" || event.status == "failed" {
                        Some(event.timestamp)
                    } else {
                        None
                    },
                    duration_ms: None,
                    output: event.output,
                    error: event.error,
                    position,
                };

                // Determine if we should link to previous node
                // Only if this is NOT the root template node (we can infer or just link sequential)
                if !nodes.is_empty() {
                    let prev_id = nodes.last().unwrap().id.clone();
                    new_link = Some(WorkflowLink {
                        source: prev_id,
                        target: event.step_id.clone(),
                        label: None,
                    });
                }

                nodes.push(new_node);
            }
        } // Drop nodes lock

        // Add link if needed
        if let Some(link) = new_link {
            self.add_link(link).await;
        }
    }
}
