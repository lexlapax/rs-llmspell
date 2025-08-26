//! ABOUTME: Hierarchical context support for parent-child relationships
//! ABOUTME: Enables context trees with traversal and management capabilities

#![allow(clippy::significant_drop_tightening)]

use llmspell_core::execution_context::{ContextScope, ExecutionContext, InheritancePolicy};
use llmspell_core::{LLMSpellError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

/// Node in the context hierarchy tree
#[derive(Debug, Clone)]
pub struct ContextNode {
    /// The execution context at this node
    pub context: ExecutionContext,
    /// Child nodes
    pub children: Vec<Arc<RwLock<ContextNode>>>,
    /// Node metadata
    pub metadata: NodeMetadata,
}

/// Metadata for a context node
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NodeMetadata {
    /// Node creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last access time
    pub last_accessed: chrono::DateTime<chrono::Utc>,
    /// Access count
    pub access_count: u64,
    /// Node tags
    pub tags: Vec<String>,
}

impl NodeMetadata {
    #[must_use]
    pub fn new() -> Self {
        let now = chrono::Utc::now();
        Self {
            created_at: now,
            last_accessed: now,
            access_count: 0,
            tags: Vec::new(),
        }
    }
}

impl ContextNode {
    /// Create a new context node
    #[must_use]
    pub fn new(context: ExecutionContext) -> Self {
        Self {
            context,
            children: Vec::new(),
            metadata: NodeMetadata::new(),
        }
    }

    /// Add a child node
    pub fn add_child(&mut self, child: Self) -> Arc<RwLock<Self>> {
        let child_arc = Arc::new(RwLock::new(child));
        self.children.push(child_arc.clone());
        child_arc
    }

    /// Find a node by context ID
    ///
    /// # Panics
    ///
    /// Panics if a `RwLock` is poisoned
    #[must_use]
    pub fn find_by_id(&self, id: &str) -> Option<Arc<RwLock<Self>>> {
        // Check self
        if self.context.id == id {
            return None; // Would need to return self somehow
        }

        // Search children
        for child in &self.children {
            let child_node = child.read().unwrap();
            if child_node.context.id == id {
                return Some(child.clone());
            }
            // Recursive search
            if let Some(found) = child_node.find_by_id(id) {
                return Some(found);
            }
        }

        None
    }

    /// Get all descendant nodes
    ///
    /// # Panics
    ///
    /// Panics if a `RwLock` is poisoned
    #[must_use]
    pub fn descendants(&self) -> Vec<Arc<RwLock<Self>>> {
        let mut result = Vec::new();

        for child in &self.children {
            result.push(child.clone());
            let child_node = child.read().unwrap();
            result.extend(child_node.descendants());
        }

        result
    }

    /// Update access metadata
    pub fn touch(&mut self) {
        self.metadata.last_accessed = chrono::Utc::now();
        self.metadata.access_count += 1;
    }
}

/// Manages hierarchical context trees
#[derive(Debug)]
pub struct HierarchicalContext {
    /// Root nodes of context trees
    roots: HashMap<String, Arc<RwLock<ContextNode>>>,
    /// Quick lookup by context ID
    index: Arc<RwLock<HashMap<String, Arc<RwLock<ContextNode>>>>>,
}

impl HierarchicalContext {
    /// Create a new hierarchical context manager
    #[must_use]
    pub fn new() -> Self {
        Self {
            roots: HashMap::new(),
            index: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create a new root context
    ///
    /// # Errors
    ///
    /// Returns an error if context creation fails
    ///
    /// # Panics
    ///
    /// Panics if a `RwLock` is poisoned
    pub fn create_root(&mut self, name: String, context: ExecutionContext) -> Result<String> {
        let node = ContextNode::new(context.clone());
        let node_arc = Arc::new(RwLock::new(node));

        self.roots.insert(name, node_arc.clone());
        self.index
            .write()
            .unwrap()
            .insert(context.id.clone(), node_arc);

        Ok(context.id)
    }

    /// Create a child context
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Parent context not found
    /// - Child context creation fails
    ///
    /// # Panics
    ///
    /// Panics if a `RwLock` is poisoned
    pub fn create_child(
        &self,
        parent_id: &str,
        scope: ContextScope,
        inheritance: InheritancePolicy,
    ) -> Result<ExecutionContext> {
        // Get parent node and drop index lock
        let parent_node = {
            let index = self.index.read().unwrap();
            index
                .get(parent_id)
                .ok_or_else(|| LLMSpellError::Component {
                    message: format!("Context not found: {parent_id}"),
                    source: None,
                })?
                .clone()
        };

        // Clone parent context to avoid holding lock during child creation
        let parent_context = parent_node.read().unwrap().context.clone();
        let child_context = parent_context.create_child(scope, inheritance);

        let child_node = ContextNode::new(child_context.clone());
        let child_arc = parent_node.write().unwrap().add_child(child_node);

        self.index
            .write()
            .unwrap()
            .insert(child_context.id.clone(), child_arc);

        Ok(child_context)
    }

    /// Get a context by ID
    ///
    /// # Panics
    ///
    /// Panics if a `RwLock` is poisoned
    #[must_use]
    pub fn get(&self, id: &str) -> Option<ExecutionContext> {
        let index = self.index.read().unwrap();
        index.get(id).map(|node| {
            let mut node_guard = node.write().unwrap();
            node_guard.touch();
            node_guard.context.clone()
        })
    }

    /// Remove a context and all its descendants
    ///
    /// # Errors
    ///
    /// Returns an error if context not found
    ///
    /// # Panics
    ///
    /// Panics if a `RwLock` is poisoned
    pub fn remove(&mut self, id: &str) -> Result<()> {
        let mut index = self.index.write().unwrap();

        if let Some(node) = index.remove(id) {
            let node_guard = node.read().unwrap();

            // Remove all descendants from index
            for descendant in node_guard.descendants() {
                let desc_guard = descendant.read().unwrap();
                index.remove(&desc_guard.context.id);
            }

            // Remove from roots if it's a root
            self.roots.retain(|_, root| {
                let root_guard = root.read().unwrap();
                root_guard.context.id != id
            });

            Ok(())
        } else {
            Err(LLMSpellError::Component {
                message: format!("Context not found: {id}"),
                source: None,
            })
        }
    }

    /// Get all root contexts
    ///
    /// # Panics
    ///
    /// Panics if a `RwLock` is poisoned
    #[must_use]
    pub fn roots(&self) -> HashMap<String, ExecutionContext> {
        self.roots
            .iter()
            .map(|(name, node)| {
                let mut node_guard = node.write().unwrap();
                node_guard.touch();
                (name.clone(), node_guard.context.clone())
            })
            .collect()
    }

    /// Get context statistics
    ///
    /// # Panics
    ///
    /// Panics if a `RwLock` is poisoned
    #[must_use]
    pub fn stats(&self) -> ContextStats {
        let index = self.index.read().unwrap();
        let total_contexts = index.len();

        let mut max_depth = 0;
        let mut total_depth = 0;

        for root in self.roots.values() {
            let depth = self.calculate_depth(root);
            max_depth = max_depth.max(depth);
            total_depth += depth;
        }

        ContextStats {
            total_contexts,
            root_count: self.roots.len(),
            max_depth,
            average_depth: if self.roots.is_empty() {
                0.0
            } else {
                #[allow(clippy::cast_precision_loss)]
                let total_depth_f64 = total_depth as f64;
                #[allow(clippy::cast_precision_loss)]
                let roots_len_f64 = self.roots.len() as f64;
                total_depth_f64 / roots_len_f64
            },
        }
    }

    #[allow(clippy::only_used_in_recursion)]
    fn calculate_depth(&self, node: &Arc<RwLock<ContextNode>>) -> usize {
        let node_guard = node.read().unwrap();
        if node_guard.children.is_empty() {
            1
        } else {
            1 + node_guard
                .children
                .iter()
                .map(|child| self.calculate_depth(child))
                .max()
                .unwrap_or(0)
        }
    }
}

impl Default for HierarchicalContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about the context hierarchy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextStats {
    pub total_contexts: usize,
    pub root_count: usize,
    pub max_depth: usize,
    pub average_depth: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_core::ComponentId;
    use serde_json::json;
    #[test]
    fn test_hierarchical_context_creation() {
        let mut hierarchy = HierarchicalContext::new();

        let root_ctx =
            ExecutionContext::new().with_data("root_key".to_string(), json!("root_value"));

        let root_id = hierarchy.create_root("main".to_string(), root_ctx).unwrap();

        // Create child
        let child_ctx = hierarchy
            .create_child(
                &root_id,
                ContextScope::Session("session-1".to_string()),
                InheritancePolicy::Inherit,
            )
            .unwrap();

        assert!(child_ctx.get("root_key").is_some());
        assert_eq!(child_ctx.depth(), 1);
    }
    #[test]
    fn test_context_removal() {
        let mut hierarchy = HierarchicalContext::new();

        let root_ctx = ExecutionContext::new();
        let root_id = hierarchy.create_root("main".to_string(), root_ctx).unwrap();

        // Create children
        let child1 = hierarchy
            .create_child(
                &root_id,
                ContextScope::Agent(ComponentId::from_name("agent-1")),
                InheritancePolicy::Inherit,
            )
            .unwrap();

        let _grandchild = hierarchy
            .create_child(
                &child1.id,
                ContextScope::Agent(ComponentId::from_name("agent-2")),
                InheritancePolicy::Inherit,
            )
            .unwrap();

        // Remove root should remove all descendants
        hierarchy.remove(&root_id).unwrap();

        assert!(hierarchy.get(&root_id).is_none());
        assert!(hierarchy.get(&child1.id).is_none());
    }
    #[test]
    fn test_context_stats() {
        let mut hierarchy = HierarchicalContext::new();

        let root1 = ExecutionContext::new();
        let root1_id = hierarchy.create_root("root1".to_string(), root1).unwrap();

        let root2 = ExecutionContext::new();
        let root2_id = hierarchy.create_root("root2".to_string(), root2).unwrap();

        // Create some children
        let child1 = hierarchy
            .create_child(
                &root1_id,
                ContextScope::Session("s1".to_string()),
                InheritancePolicy::Inherit,
            )
            .unwrap();

        hierarchy
            .create_child(
                &child1.id,
                ContextScope::Agent(ComponentId::from_name("a1")),
                InheritancePolicy::Inherit,
            )
            .unwrap();

        hierarchy
            .create_child(
                &root2_id,
                ContextScope::Session("s2".to_string()),
                InheritancePolicy::Inherit,
            )
            .unwrap();

        let stats = hierarchy.stats();
        assert_eq!(stats.total_contexts, 5); // 2 roots + 3 children
        assert_eq!(stats.root_count, 2);
        assert_eq!(stats.max_depth, 3); // root1 -> child1 -> grandchild
    }
    #[test]
    fn test_node_metadata() {
        let mut node = ContextNode::new(ExecutionContext::new());

        let initial_count = node.metadata.access_count;
        let initial_time = node.metadata.last_accessed;

        // Sleep briefly to ensure time difference
        std::thread::sleep(std::time::Duration::from_millis(10));

        node.touch();

        assert_eq!(node.metadata.access_count, initial_count + 1);
        assert!(node.metadata.last_accessed > initial_time);
    }
}
