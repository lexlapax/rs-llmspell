//! ABOUTME: ComponentLookup trait for registry access without circular dependencies
//! ABOUTME: Allows workflow execution to lookup components without depending on bridge

use crate::{Agent, BaseAgent, Tool, Workflow};
use async_trait::async_trait;
use std::sync::Arc;

/// Trait for looking up components (agents, tools, workflows) by name
///
/// This trait is implemented by ComponentRegistry in llmspell-bridge,
/// but defined here to avoid circular dependencies. It allows the
/// StepExecutor in llmspell-workflows to lookup and execute components.
#[async_trait]
pub trait ComponentLookup: Send + Sync {
    /// Get an agent by name
    async fn get_agent(&self, name: &str) -> Option<Arc<dyn Agent>>;

    /// Get a tool by name
    async fn get_tool(&self, name: &str) -> Option<Arc<dyn Tool>>;

    /// Get a workflow by name  
    async fn get_workflow(&self, name: &str) -> Option<Arc<dyn Workflow>>;

    /// Get any component as a BaseAgent by type and name
    async fn get_component(&self, component_type: &str, name: &str) -> Option<Arc<dyn BaseAgent>> {
        match component_type {
            "agent" => self.get_agent(name).await.map(|a| a as Arc<dyn BaseAgent>),
            "tool" => self.get_tool(name).await.map(|t| t as Arc<dyn BaseAgent>),
            "workflow" => self
                .get_workflow(name)
                .await
                .map(|w| w as Arc<dyn BaseAgent>),
            _ => None,
        }
    }

    /// List all available agents
    async fn list_agents(&self) -> Vec<String>;

    /// List all available tools
    async fn list_tools(&self) -> Vec<String>;

    /// List all available workflows
    async fn list_workflows(&self) -> Vec<String>;
}
