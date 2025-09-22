//! ABOUTME: Enhanced `ExecutionContext` with hierarchical support and service bundle architecture
//! ABOUTME: Provides comprehensive runtime services for agents, tools, and workflows

use crate::traits::event::EventEmitter;
use crate::traits::state::StateAccess;
use crate::types::{ComponentId, EventMetadata};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, RwLock};
use tracing::{debug, info, trace};

/// Context inheritance policy
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum InheritancePolicy {
    /// Inherit all data from parent context
    Inherit,
    /// Isolate from parent, start fresh
    Isolate,
    /// Copy specific fields only
    Copy,
    /// Share read-only access to parent
    Share,
}

/// Hierarchical scope for context data
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContextScope {
    /// Global application-wide scope
    Global,
    /// Session-specific scope
    Session(String),
    /// Workflow execution scope
    Workflow(String),
    /// Agent-specific scope
    Agent(ComponentId),
    /// User-specific scope
    User(String),
}

impl fmt::Display for ContextScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ContextScope::Global => write!(f, "global"),
            ContextScope::Session(id) => write!(f, "session:{id}"),
            ContextScope::Workflow(id) => write!(f, "workflow:{id}"),
            ContextScope::Agent(id) => write!(f, "agent:{id}"),
            ContextScope::User(id) => write!(f, "user:{id}"),
        }
    }
}

/// Shared memory region for inter-agent communication
#[derive(Debug, Clone)]
pub struct SharedMemory {
    /// Memory regions by scope
    regions: Arc<RwLock<HashMap<ContextScope, HashMap<String, Value>>>>,
}

impl SharedMemory {
    /// Create new shared memory
    #[must_use]
    pub fn new() -> Self {
        Self {
            regions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get value from a memory region
    #[must_use]
    pub fn get(&self, scope: &ContextScope, key: &str) -> Option<Value> {
        trace!(
            scope = %scope,
            key = %key,
            "SharedMemory::get"
        );
        self.regions
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(scope)
            .and_then(|region| region.get(key).cloned())
    }

    /// Set value in a memory region
    pub fn set(&self, scope: ContextScope, key: String, value: Value) {
        debug!(
            scope = %scope,
            key = %key,
            "SharedMemory::set"
        );
        let mut regions = self
            .regions
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        regions.entry(scope).or_default().insert(key, value);
    }

    /// Remove value from a memory region
    #[must_use]
    pub fn remove(&self, scope: &ContextScope, key: &str) -> Option<Value> {
        let mut regions = self
            .regions
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        regions.get_mut(scope).and_then(|region| region.remove(key))
    }

    /// Clear all data in a scope
    pub fn clear_scope(&self, scope: &ContextScope) {
        let mut regions = self
            .regions
            .write()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        regions.remove(scope);
    }

    /// Get all keys in a scope
    #[must_use]
    pub fn keys(&self, scope: &ContextScope) -> Vec<String> {
        self.regions
            .read()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(scope)
            .map(|region| region.keys().cloned().collect())
            .unwrap_or_default()
    }
}

impl Default for SharedMemory {
    fn default() -> Self {
        Self::new()
    }
}

/// Enhanced execution context with hierarchical support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    /// Unique context ID
    pub id: String,

    /// Parent context ID for hierarchical reference (avoids circular references)
    pub parent_id: Option<String>,

    /// Context scope
    pub scope: ContextScope,

    /// Inheritance policy
    pub inheritance: InheritancePolicy,

    /// Current conversation ID
    pub conversation_id: Option<String>,

    /// User ID if applicable
    pub user_id: Option<String>,

    /// Session ID for tracking
    pub session_id: Option<String>,

    /// Local context data
    pub data: HashMap<String, Value>,

    /// Shared memory for inter-agent communication (transient data)
    #[serde(skip)]
    pub shared_memory: SharedMemory,

    /// State access for persistent data (first-class citizen for component communication)
    /// Uses StateAccess trait to avoid direct dependency on llmspell-state-persistence
    #[serde(skip)]
    pub state: Option<Arc<dyn StateAccess>>,

    /// Event emitter for component lifecycle events (observability and coordination)
    /// Uses EventEmitter trait to avoid direct dependency on llmspell-events
    #[serde(skip)]
    pub events: Option<Arc<dyn EventEmitter>>,

    /// Event metadata for correlation
    pub metadata: EventMetadata,

    /// Security context (placeholder for Phase 3.2 integration)
    pub security_context: Option<SecurityContext>,
}

/// Security context placeholder
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    /// User permissions
    pub permissions: Vec<String>,
    /// Security level
    pub level: String,
}

impl ExecutionContext {
    /// Create a new root execution context
    #[must_use]
    pub fn new() -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            parent_id: None,
            scope: ContextScope::Global,
            inheritance: InheritancePolicy::Inherit,
            conversation_id: None,
            user_id: None,
            session_id: None,
            data: HashMap::new(),
            shared_memory: SharedMemory::new(),
            state: None,
            events: None,
            metadata: EventMetadata::default(),
            security_context: None,
        }
    }

    /// Create with conversation ID
    #[must_use]
    pub fn with_conversation(conversation_id: String) -> Self {
        let mut ctx = Self::new();
        ctx.conversation_id = Some(conversation_id);
        ctx
    }

    /// Create a child context with inheritance
    #[must_use]
    pub fn create_child(&self, scope: ContextScope, inheritance: InheritancePolicy) -> Self {
        info!(
            parent_id = %self.id,
            child_scope = %scope,
            inheritance_policy = ?inheritance,
            "Creating child context"
        );

        let mut child = Self {
            id: uuid::Uuid::new_v4().to_string(),
            parent_id: Some(self.id.clone()),
            scope,
            inheritance,
            conversation_id: self.conversation_id.clone(),
            user_id: self.user_id.clone(),
            session_id: self.session_id.clone(),
            data: HashMap::new(),
            shared_memory: self.shared_memory.clone(), // Shared across hierarchy
            state: self.state.clone(),                 // State is shared across hierarchy
            events: self.events.clone(),               // Events are shared across hierarchy
            metadata: self.metadata.clone(),
            security_context: self.security_context.clone(),
        };

        // Apply inheritance policy
        match inheritance {
            InheritancePolicy::Inherit => {
                // Copy all parent data
                child.data = self.data.clone();
            }
            InheritancePolicy::Copy => {
                // Copy specific fields (configured elsewhere)
                // For now, copy conversation-related fields
                if let Some(conv) = self.data.get("conversation_context") {
                    child
                        .data
                        .insert("conversation_context".to_string(), conv.clone());
                }
            }
            InheritancePolicy::Isolate | InheritancePolicy::Share => {
                // Isolate: Start fresh, no data copied
                // Share: Parent data accessible via parent reference
            }
        }

        child
    }

    /// Get data from context hierarchy
    pub fn get(&self, key: &str) -> Option<Value> {
        trace!(
            context_id = %self.id,
            key = %key,
            scope = %self.scope,
            "Getting value from context"
        );

        // First check local data
        if let Some(value) = self.data.get(key) {
            trace!(
                context_id = %self.id,
                key = %key,
                found_in = "local",
                "Value found in local data"
            );
            return Some(value.clone());
        }

        // Then check shared memory at current scope
        if let Some(value) = self.shared_memory.get(&self.scope, key) {
            trace!(
                context_id = %self.id,
                key = %key,
                found_in = "shared_memory",
                "Value found in shared memory"
            );
            return Some(value);
        }

        // Finally, check parent if inheritance allows
        // Note: Without parent reference, we can't check parent data
        // This would need to be handled by the HierarchicalContext
        trace!(
            context_id = %self.id,
            key = %key,
            "Value not found in context"
        );
        None
    }

    /// Set data in context
    pub fn set(&mut self, key: String, value: Value) {
        let value_size = serde_json::to_string(&value).map(|s| s.len()).unwrap_or(0);
        debug!(
            context_id = %self.id,
            key = %key,
            value_size,
            scope = %self.scope,
            "Setting value in context"
        );
        self.data.insert(key, value);
    }

    /// Set data in shared memory
    pub fn set_shared(&self, key: String, value: Value) {
        debug!(
            context_id = %self.id,
            key = %key,
            scope = %self.scope,
            "Setting value in shared memory"
        );
        self.shared_memory.set(self.scope.clone(), key, value);
    }

    /// Get data from shared memory at specific scope
    pub fn get_shared(&self, scope: &ContextScope, key: &str) -> Option<Value> {
        trace!(
            context_id = %self.id,
            key = %key,
            scope = %scope,
            "Getting value from shared memory"
        );
        self.shared_memory.get(scope, key)
    }

    /// Add a data field (builder pattern)
    pub fn with_data(mut self, key: String, value: Value) -> Self {
        self.data.insert(key, value);
        self
    }

    /// Set the scope
    pub fn with_scope(mut self, scope: ContextScope) -> Self {
        self.scope = scope;
        self
    }

    /// Set the inheritance policy
    pub fn with_inheritance(mut self, inheritance: InheritancePolicy) -> Self {
        self.inheritance = inheritance;
        self
    }

    /// Set the state access provider
    pub fn with_state(mut self, state: Arc<dyn StateAccess>) -> Self {
        self.state = Some(state);
        self
    }

    /// Check if context has a specific capability
    pub fn has_capability(&self, capability: &str) -> bool {
        self.get("capabilities")
            .as_ref()
            .and_then(|v| v.as_array())
            .map(|caps| caps.iter().any(|c| c.as_str() == Some(capability)))
            .unwrap_or(false)
    }

    /// Get the root context
    pub fn root(&self) -> &ExecutionContext {
        // Without parent reference, we return self
        // HierarchicalContext should handle finding the actual root
        self
    }

    /// Get context depth in hierarchy
    pub fn depth(&self) -> usize {
        // Without parent reference, we can only return 0 or 1
        // HierarchicalContext should track the actual depth
        if self.parent_id.is_some() {
            1
        } else {
            0
        }
    }

    /// Merge data from another context
    pub fn merge(&mut self, other: &ExecutionContext) {
        let key_count = other.data.len();
        debug!(
            context_id = %self.id,
            other_context_id = %other.id,
            key_count,
            "Merging context data"
        );
        for (key, value) in &other.data {
            self.data.insert(key.clone(), value.clone());
        }
    }
}

impl Default for ExecutionContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for `ExecutionContext`
pub struct ExecutionContextBuilder {
    context: ExecutionContext,
}

impl ExecutionContextBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            context: ExecutionContext::new(),
        }
    }

    /// Set conversation ID
    pub fn conversation_id(mut self, id: String) -> Self {
        self.context.conversation_id = Some(id);
        self
    }

    /// Set user ID
    pub fn user_id(mut self, id: String) -> Self {
        self.context.user_id = Some(id);
        self
    }

    /// Set session ID
    pub fn session_id(mut self, id: String) -> Self {
        self.context.session_id = Some(id);
        self
    }

    /// Set scope
    pub fn scope(mut self, scope: ContextScope) -> Self {
        self.context.scope = scope;
        self
    }

    /// Set parent context ID
    pub fn parent_id(mut self, parent_id: String) -> Self {
        self.context.parent_id = Some(parent_id);
        self
    }

    /// Set inheritance policy
    pub fn inheritance(mut self, policy: InheritancePolicy) -> Self {
        self.context.inheritance = policy;
        self
    }

    /// Add data
    pub fn data(mut self, key: String, value: Value) -> Self {
        self.context.data.insert(key, value);
        self
    }

    /// Set security context
    pub fn security(mut self, security: SecurityContext) -> Self {
        self.context.security_context = Some(security);
        self
    }

    /// Set state access provider
    pub fn state(mut self, state: Arc<dyn StateAccess>) -> Self {
        self.context.state = Some(state);
        self
    }

    /// Set event emitter provider
    pub fn events(mut self, events: Arc<dyn EventEmitter>) -> Self {
        self.context.events = Some(events);
        self
    }

    /// Build the context
    pub fn build(self) -> ExecutionContext {
        self.context
    }
}

impl Default for ExecutionContextBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    #[test]
    fn test_context_creation() {
        let ctx = ExecutionContext::with_conversation("conv-123".to_string());
        assert_eq!(ctx.conversation_id, Some("conv-123".to_string()));
        assert!(ctx.parent_id.is_none());
        assert_eq!(ctx.scope, ContextScope::Global);
    }
    #[test]
    fn test_context_hierarchy() {
        let root = ExecutionContext::new().with_data("root_data".to_string(), json!("root_value"));

        let child = root.create_child(
            ContextScope::Agent(ComponentId::from_name("agent-1")),
            InheritancePolicy::Inherit,
        );

        // Child has parent_id set
        assert_eq!(child.parent_id, Some(root.id.clone()));
        assert_eq!(child.depth(), 1);
        // Note: get() no longer checks parent data directly
        // HierarchicalContext should handle inheritance
    }
    #[test]
    fn test_inheritance_policies() {
        let parent = ExecutionContext::new()
            .with_data("parent_key".to_string(), json!("parent_value"))
            .with_data("conversation_context".to_string(), json!({"topic": "test"}));

        // Test Inherit policy
        let inherit_child = parent.create_child(
            ContextScope::Session("session-1".to_string()),
            InheritancePolicy::Inherit,
        );
        // With Inherit policy, data is copied to child
        assert_eq!(inherit_child.get("parent_key"), Some(json!("parent_value")));

        // Test Isolate policy
        let isolate_child = parent.create_child(
            ContextScope::Session("session-2".to_string()),
            InheritancePolicy::Isolate,
        );
        assert_eq!(isolate_child.get("parent_key"), None);

        // Test Copy policy
        let copy_child = parent.create_child(
            ContextScope::Session("session-3".to_string()),
            InheritancePolicy::Copy,
        );
        assert_eq!(
            copy_child.get("conversation_context"),
            Some(json!({"topic": "test"}))
        );
        assert_eq!(copy_child.get("parent_key"), None);
    }
    #[test]
    fn test_shared_memory() {
        let ctx = ExecutionContext::new()
            .with_scope(ContextScope::Agent(ComponentId::from_name("agent-1")));

        // Set shared memory
        ctx.shared_memory.set(
            ContextScope::Workflow("workflow-1".to_string()),
            "shared_data".to_string(),
            json!("shared_value"),
        );

        // Can access shared data
        let value = ctx.get_shared(
            &ContextScope::Workflow("workflow-1".to_string()),
            "shared_data",
        );
        assert_eq!(value, Some(json!("shared_value")));

        // Different scope doesn't have the data
        let value2 = ctx.get_shared(
            &ContextScope::Session("session-1".to_string()),
            "shared_data",
        );
        assert_eq!(value2, None);
    }
    #[test]
    fn test_context_builder() {
        let ctx = ExecutionContextBuilder::new()
            .conversation_id("conv-456".to_string())
            .user_id("user-789".to_string())
            .session_id("session-123".to_string())
            .scope(ContextScope::User("user-789".to_string()))
            .data("preference".to_string(), json!("dark_mode"))
            .build();

        assert_eq!(ctx.conversation_id, Some("conv-456".to_string()));
        assert_eq!(ctx.user_id, Some("user-789".to_string()));
        assert_eq!(ctx.session_id, Some("session-123".to_string()));
        assert_eq!(ctx.get("preference"), Some(json!("dark_mode")));
    }
    #[test]
    fn test_capability_checking() {
        let mut ctx = ExecutionContext::new();
        ctx.set(
            "capabilities".to_string(),
            json!(["read", "write", "execute"]),
        );

        assert!(ctx.has_capability("read"));
        assert!(ctx.has_capability("write"));
        assert!(!ctx.has_capability("delete"));
    }
    #[test]
    fn test_context_merging() {
        let mut ctx1 = ExecutionContext::new().with_data("key1".to_string(), json!("value1"));

        let ctx2 = ExecutionContext::new()
            .with_data("key2".to_string(), json!("value2"))
            .with_data("key3".to_string(), json!("value3"));

        ctx1.merge(&ctx2);

        assert_eq!(ctx1.get("key1"), Some(json!("value1")));
        assert_eq!(ctx1.get("key2"), Some(json!("value2")));
        assert_eq!(ctx1.get("key3"), Some(json!("value3")));
    }
    #[test]
    fn test_scope_display() {
        assert_eq!(ContextScope::Global.to_string(), "global");
        assert_eq!(
            ContextScope::Session("s1".to_string()).to_string(),
            "session:s1"
        );
        assert_eq!(
            ContextScope::Workflow("w1".to_string()).to_string(),
            "workflow:w1"
        );
    }
}
