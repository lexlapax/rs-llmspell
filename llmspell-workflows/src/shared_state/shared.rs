//! ABOUTME: Shared state implementation for workflows
//! ABOUTME: Thread-safe state management with scoping and isolation

use super::types::{StateAccess, StateEntry, StateScope};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, trace};

/// Thread-safe workflow state manager
#[derive(Clone, Debug)]
pub struct WorkflowStateManager {
    /// Internal state storage
    state: Arc<RwLock<HashMap<String, StateEntry>>>,
}

impl WorkflowStateManager {
    /// Create a new state manager
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get the size of the state store
    pub fn size(&self) -> usize {
        self.state.read().len()
    }

    /// Create a workflow-specific state accessor
    pub fn workflow_state(&self, workflow_id: String) -> WorkflowStateAccessor {
        WorkflowStateAccessor {
            manager: self.clone(),
            default_scope: StateScope::Workflow(workflow_id),
        }
    }
}

impl Default for WorkflowStateManager {
    fn default() -> Self {
        Self::new()
    }
}

impl StateAccess for WorkflowStateManager {
    fn get_state(&self, scope: &StateScope, key: &str) -> Option<serde_json::Value> {
        let scoped_key = scope.scoped_key(key);
        let state = self.state.read();

        state.get(&scoped_key).map(|entry| {
            trace!("State get: {} = {:?}", scoped_key, entry.value);
            entry.value.clone()
        })
    }

    fn set_state(&self, scope: &StateScope, key: &str, value: serde_json::Value) {
        let scoped_key = scope.scoped_key(key);
        let mut state = self.state.write();

        debug!("State set: {} = {:?}", scoped_key, value);

        match state.get_mut(&scoped_key) {
            Some(entry) => entry.update(value),
            None => {
                state.insert(scoped_key, StateEntry::new(value));
            }
        }
    }

    fn delete_state(&self, scope: &StateScope, key: &str) {
        let scoped_key = scope.scoped_key(key);
        let mut state = self.state.write();

        if state.remove(&scoped_key).is_some() {
            debug!("State delete: {}", scoped_key);
        }
    }

    fn list_keys(&self, scope: &StateScope) -> Vec<String> {
        let state = self.state.read();
        let prefix = match scope {
            StateScope::Global => String::new(),
            StateScope::Workflow(id) => format!("workflow:{}:", id),
            StateScope::Step {
                workflow_id,
                step_name,
            } => {
                format!("step:{}:{}:", workflow_id, step_name)
            }
            StateScope::Custom(namespace) => format!("{}:", namespace),
        };

        state
            .keys()
            .filter(|k| {
                if prefix.is_empty() {
                    // For global scope, only return keys without ':'
                    !k.contains(':')
                } else {
                    // For other scopes, return keys with the prefix
                    k.starts_with(&prefix)
                }
            })
            .map(|k| {
                if prefix.is_empty() {
                    k.clone()
                } else {
                    k.strip_prefix(&prefix).unwrap_or(k).to_string()
                }
            })
            .collect()
    }

    fn get_scope(&self, scope: &StateScope) -> HashMap<String, serde_json::Value> {
        let keys = self.list_keys(scope);
        let mut result = HashMap::new();

        for key in keys {
            if let Some(value) = self.get_state(scope, &key) {
                result.insert(key, value);
            }
        }

        result
    }

    fn clear_scope(&self, scope: &StateScope) {
        let mut state = self.state.write();
        let prefix = match scope {
            StateScope::Global => String::new(),
            StateScope::Workflow(id) => format!("workflow:{}:", id),
            StateScope::Step {
                workflow_id,
                step_name,
            } => {
                format!("step:{}:{}:", workflow_id, step_name)
            }
            StateScope::Custom(namespace) => format!("{}:", namespace),
        };

        if prefix.is_empty() {
            // Clear only global entries (those without ':')
            state.retain(|k, _| k.contains(':'));
        } else {
            // Clear entries with the prefix
            state.retain(|k, _| !k.starts_with(&prefix));
        }

        debug!("State cleared for scope: {:?}", scope);
    }
}

/// Workflow-specific state accessor
#[derive(Clone, Debug)]
pub struct WorkflowStateAccessor {
    /// Reference to the state manager
    manager: WorkflowStateManager,

    /// Default scope for this workflow
    default_scope: StateScope,
}

impl WorkflowStateAccessor {
    /// Get a value using the default workflow scope
    pub fn get(&self, key: &str) -> Option<serde_json::Value> {
        self.manager.get_state(&self.default_scope, key)
    }

    /// Set a value using the default workflow scope
    pub fn set(&self, key: &str, value: serde_json::Value) {
        self.manager.set_state(&self.default_scope, key, value)
    }

    /// Delete a value using the default workflow scope
    pub fn delete(&self, key: &str) {
        self.manager.delete_state(&self.default_scope, key)
    }

    /// Access global state
    pub fn global(&self) -> GlobalStateAccess {
        GlobalStateAccess {
            manager: self.manager.clone(),
        }
    }

    /// Access step-specific state
    pub fn step(&self, step_name: String) -> StepStateAccess {
        let workflow_id = match &self.default_scope {
            StateScope::Workflow(id) => id.clone(),
            _ => panic!("WorkflowStateAccessor must have Workflow scope"),
        };

        StepStateAccess {
            manager: self.manager.clone(),
            scope: StateScope::Step {
                workflow_id,
                step_name,
            },
        }
    }
}

/// Global state accessor
#[derive(Clone, Debug)]
pub struct GlobalStateAccess {
    manager: WorkflowStateManager,
}

impl GlobalStateAccess {
    /// Get a value from global state by key
    pub fn get(&self, key: &str) -> Option<serde_json::Value> {
        self.manager.get_state(&StateScope::Global, key)
    }

    /// Set a value in global state
    pub fn set(&self, key: &str, value: serde_json::Value) {
        self.manager.set_state(&StateScope::Global, key, value)
    }

    /// Delete a value from global state
    pub fn delete(&self, key: &str) {
        self.manager.delete_state(&StateScope::Global, key)
    }
}

/// Step-specific state accessor
#[derive(Clone, Debug)]
pub struct StepStateAccess {
    manager: WorkflowStateManager,
    scope: StateScope,
}

impl StepStateAccess {
    /// Get a value from step-specific state by key
    pub fn get(&self, key: &str) -> Option<serde_json::Value> {
        self.manager.get_state(&self.scope, key)
    }

    /// Set a value in step-specific state
    pub fn set(&self, key: &str, value: serde_json::Value) {
        self.manager.set_state(&self.scope, key, value)
    }

    /// Delete a value from step-specific state
    pub fn delete(&self, key: &str) {
        self.manager.delete_state(&self.scope, key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_state_scoping() {
        let manager = WorkflowStateManager::new();

        // Test global state
        manager.set_state(&StateScope::Global, "key1", serde_json::json!("global"));

        // Test workflow state
        let workflow_scope = StateScope::Workflow("wf1".to_string());
        manager.set_state(&workflow_scope, "key1", serde_json::json!("workflow"));

        // Verify isolation
        assert_eq!(
            manager.get_state(&StateScope::Global, "key1"),
            Some(serde_json::json!("global"))
        );
        assert_eq!(
            manager.get_state(&workflow_scope, "key1"),
            Some(serde_json::json!("workflow"))
        );
    }
    #[test]
    fn test_workflow_state_accessor() {
        let manager = WorkflowStateManager::new();
        let workflow_state = manager.workflow_state("test_workflow".to_string());

        // Test workflow-scoped access
        workflow_state.set("config", serde_json::json!({"retry": 3}));
        assert_eq!(
            workflow_state.get("config"),
            Some(serde_json::json!({"retry": 3}))
        );

        // Test global access
        workflow_state
            .global()
            .set("shared", serde_json::json!("data"));
        assert_eq!(
            workflow_state.global().get("shared"),
            Some(serde_json::json!("data"))
        );

        // Test step access
        let step_state = workflow_state.step("step1".to_string());
        step_state.set("output", serde_json::json!({"result": "success"}));
        assert_eq!(
            step_state.get("output"),
            Some(serde_json::json!({"result": "success"}))
        );
    }
    #[test]
    fn test_list_and_clear() {
        let manager = WorkflowStateManager::new();

        // Add some state
        manager.set_state(&StateScope::Global, "g1", serde_json::json!(1));
        manager.set_state(&StateScope::Global, "g2", serde_json::json!(2));
        manager.set_state(
            &StateScope::Workflow("wf1".to_string()),
            "w1",
            serde_json::json!(3),
        );

        // Test list
        let global_keys = manager.list_keys(&StateScope::Global);
        assert_eq!(global_keys.len(), 2);
        assert!(global_keys.contains(&"g1".to_string()));
        assert!(global_keys.contains(&"g2".to_string()));

        // Test clear
        manager.clear_scope(&StateScope::Global);
        assert_eq!(manager.list_keys(&StateScope::Global).len(), 0);

        // Workflow state should still exist
        assert_eq!(
            manager.get_state(&StateScope::Workflow("wf1".to_string()), "w1"),
            Some(serde_json::json!(3))
        );
    }
}
