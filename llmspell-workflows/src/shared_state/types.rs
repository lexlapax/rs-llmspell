//! ABOUTME: State type definitions for workflow state management
//! ABOUTME: Defines state scoping and access patterns for workflows

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// State scope for isolation between workflows
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StateScope {
    /// Global scope - accessible by all workflows
    Global,

    /// Workflow-specific scope
    Workflow(String),

    /// Step-specific scope within a workflow
    Step {
        workflow_id: String,
        step_name: String,
    },

    /// Custom namespace
    Custom(String),
}

impl StateScope {
    /// Create a scoped key combining scope and key
    pub fn scoped_key(&self, key: &str) -> String {
        match self {
            StateScope::Global => key.to_string(),
            StateScope::Workflow(id) => format!("workflow:{}:{}", id, key),
            StateScope::Step {
                workflow_id,
                step_name,
            } => {
                format!("step:{}:{}:{}", workflow_id, step_name, key)
            }
            StateScope::Custom(namespace) => format!("{}:{}", namespace, key),
        }
    }
}

/// State entry with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateEntry {
    /// The actual value
    pub value: serde_json::Value,

    /// When this entry was created
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// When this entry was last updated
    pub updated_at: chrono::DateTime<chrono::Utc>,

    /// Optional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl StateEntry {
    /// Create a new state entry
    pub fn new(value: serde_json::Value) -> Self {
        let now = chrono::Utc::now();
        Self {
            value,
            created_at: now,
            updated_at: now,
            metadata: HashMap::new(),
        }
    }

    /// Update the value and timestamp
    pub fn update(&mut self, value: serde_json::Value) {
        self.value = value;
        self.updated_at = chrono::Utc::now();
    }
}

/// Trait for types that can access workflow state
pub trait StateAccess {
    /// Get a value from state
    fn get_state(&self, scope: &StateScope, key: &str) -> Option<serde_json::Value>;

    /// Set a value in state
    fn set_state(&self, scope: &StateScope, key: &str, value: serde_json::Value);

    /// Delete a value from state
    fn delete_state(&self, scope: &StateScope, key: &str);

    /// List all keys in a scope
    fn list_keys(&self, scope: &StateScope) -> Vec<String>;

    /// Get all values in a scope
    fn get_scope(&self, scope: &StateScope) -> HashMap<String, serde_json::Value>;

    /// Clear all values in a scope
    fn clear_scope(&self, scope: &StateScope);
}
