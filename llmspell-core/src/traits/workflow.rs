//! ABOUTME: Workflow trait for orchestration components
//! ABOUTME: Extends BaseAgent with step management and execution planning

use super::base_agent::BaseAgent;
use crate::{ComponentId, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Workflow step definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStep {
    pub id: ComponentId,
    pub name: String,
    pub component_id: ComponentId,
    pub dependencies: Vec<ComponentId>,
}

/// Step execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepResult {
    pub step_id: ComponentId,
    pub output: super::base_agent::AgentOutput,
    pub duration: std::time::Duration,
    pub success: bool,
}

/// Workflow execution status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkflowStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Cancelled,
}

/// Workflow trait for orchestration components
#[async_trait]
pub trait Workflow: BaseAgent {
    /// Add step to workflow
    async fn add_step(&mut self, step: WorkflowStep) -> Result<()>;
    
    /// Remove step from workflow
    async fn remove_step(&mut self, step_id: ComponentId) -> Result<()>;
    
    /// Get execution plan
    async fn plan_execution(&self) -> Result<Vec<WorkflowStep>>;
    
    /// Get workflow status
    async fn status(&self) -> Result<WorkflowStatus>;
    
    /// Get step results
    async fn get_results(&self) -> Result<Vec<StepResult>>;
}