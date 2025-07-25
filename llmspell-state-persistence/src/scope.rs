// ABOUTME: State scope definitions and key management for isolation
// ABOUTME: Provides hierarchical state scoping and key validation

use serde::{Deserialize, Serialize};

/// State scope for isolation and namespacing
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum StateScope {
    Global,
    Agent(String),
    Workflow(String),
    Step { workflow_id: String, step_name: String },
    Session(String), // Preparation for Phase 6
    Custom(String),
}

impl StateScope {
    /// Get the prefix for this scope
    pub fn prefix(&self) -> String {
        match self {
            StateScope::Global => String::new(),
            StateScope::Agent(id) => format!("agent:{}:", id),
            StateScope::Workflow(id) => format!("workflow:{}:", id),
            StateScope::Step { workflow_id, step_name } => {
                format!("workflow:{}:step:{}:", workflow_id, step_name)
            }
            StateScope::Session(id) => format!("session:{}:", id),
            StateScope::Custom(namespace) => format!("custom:{}:", namespace),
        }
    }

    /// Check if this scope is a parent of another scope
    pub fn is_parent_of(&self, other: &StateScope) -> bool {
        match (self, other) {
            (StateScope::Global, _) => true,
            (StateScope::Workflow(id1), StateScope::Step { workflow_id, .. }) => id1 == workflow_id,
            _ => false,
        }
    }

    /// Get the parent scope if one exists
    pub fn parent(&self) -> Option<StateScope> {
        match self {
            StateScope::Global => None,
            StateScope::Agent(_) => Some(StateScope::Global),
            StateScope::Workflow(_) => Some(StateScope::Global),
            StateScope::Session(_) => Some(StateScope::Global),
            StateScope::Custom(_) => Some(StateScope::Global),
            StateScope::Step { workflow_id, .. } => Some(StateScope::Workflow(workflow_id.clone())),
        }
    }

    /// Get the scope type as a string
    pub fn scope_type(&self) -> &'static str {
        match self {
            StateScope::Global => "global",
            StateScope::Agent(_) => "agent",
            StateScope::Workflow(_) => "workflow",
            StateScope::Step { .. } => "step",
            StateScope::Session(_) => "session",
            StateScope::Custom(_) => "custom",
        }
    }
}

impl std::fmt::Display for StateScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StateScope::Global => write!(f, "Global"),
            StateScope::Agent(id) => write!(f, "Agent({})", id),
            StateScope::Workflow(id) => write!(f, "Workflow({})", id),
            StateScope::Step { workflow_id, step_name } => {
                write!(f, "Step({}/{})", workflow_id, step_name)
            }
            StateScope::Session(id) => write!(f, "Session({})", id),
            StateScope::Custom(ns) => write!(f, "Custom({})", ns),
        }
    }
}