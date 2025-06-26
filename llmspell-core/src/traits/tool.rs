//! ABOUTME: Tool trait for functional components with schema validation
//! ABOUTME: Extends BaseAgent with parameter validation and tool categorization

use super::base_agent::BaseAgent;
use crate::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};

/// Tool category for organization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ToolCategory {
    Filesystem,
    Web,
    Analysis,
    Data,
    System,
    Utility,
}

/// Security level for tools
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SecurityLevel {
    Safe,
    Restricted,
    Privileged,
}

/// Tool schema for parameter validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolSchema {
    pub parameters: serde_json::Value,
    pub required: Vec<String>,
}

/// Tool trait for functional components
#[async_trait]
pub trait Tool: BaseAgent {
    /// Get tool category
    fn category(&self) -> ToolCategory;
    
    /// Get security level
    fn security_level(&self) -> SecurityLevel;
    
    /// Get parameter schema
    fn schema(&self) -> ToolSchema;
    
    /// Validate tool parameters
    async fn validate_parameters(&self, params: &serde_json::Value) -> Result<()>;
}