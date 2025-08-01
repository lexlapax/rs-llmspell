//! ABOUTME: Dependency graph for managing hook execution ordering across components
//! ABOUTME: Ensures proper sequencing and dependency resolution in cross-component scenarios

//! # Dependency Graph for Hook Execution
//!
//! This module provides a dependency graph system for managing the order of hook execution
//! across different components. It ensures that hooks with dependencies are executed in the
//! correct order, preventing race conditions and ensuring data consistency.
//!
//! ## Features
//!
//! - **Topological Sorting**: Automatically determines execution order
//! - **Cycle Detection**: Prevents infinite dependency loops
//! - **Dependency Validation**: Ensures all dependencies are satisfiable
//! - **Performance Optimization**: Identifies parallel execution opportunities
//!
//! ## Example
//!
//! ```rust,no_run
//! use llmspell_hooks::coordination::{DependencyGraph, DependencyNode, ExecutionOrder};
//! use llmspell_hooks::{ComponentId, ComponentType, HookPoint};
//!
//! # fn example() -> anyhow::Result<()> {
//! let mut graph = DependencyGraph::new();
//!
//! // Define components
//! let agent_id = ComponentId::new(ComponentType::Agent, "gpt-4".to_string());
//! let tool_id = ComponentId::new(ComponentType::Tool, "calculator".to_string());
//! let workflow_id = ComponentId::new(ComponentType::Workflow, "analysis".to_string());
//!
//! // Add dependencies: workflow depends on tool, tool depends on agent
//! graph.add_dependency(&workflow_id, &tool_id, HookPoint::BeforeWorkflowStart)?;
//! graph.add_dependency(&tool_id, &agent_id, HookPoint::BeforeAgentExecution)?;
//!
//! // Get execution order
//! let order = graph.get_execution_order()?;
//! assert_eq!(order.sequence.len(), 3);
//! # Ok(())
//! # }
//! ```

use crate::{ComponentId, HookPoint};
use anyhow::Result;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;
use tracing::{debug, warn};

/// Dependency graph for managing hook execution order
#[derive(Debug, Clone)]
pub struct DependencyGraph {
    /// Nodes in the dependency graph
    nodes: HashMap<ComponentId, DependencyNode>,
    /// Adjacency list: component -> list of dependencies
    dependencies: HashMap<ComponentId, Vec<Dependency>>,
    /// Reverse adjacency list: component -> list of dependents
    dependents: HashMap<ComponentId, Vec<ComponentId>>,
}

/// A node in the dependency graph representing a component
#[derive(Debug, Clone)]
pub struct DependencyNode {
    /// Component identifier
    pub component_id: ComponentId,
    /// Hook points that this component participates in
    pub hook_points: HashSet<HookPoint>,
    /// Metadata associated with this node
    pub metadata: HashMap<String, String>,
    /// Priority for execution ordering
    pub priority: i32,
}

/// A dependency relationship between components
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Dependency {
    /// Component that this one depends on
    pub depends_on: ComponentId,
    /// Hook point where the dependency applies
    pub hook_point: HookPoint,
    /// Whether this is a hard dependency (must be satisfied) or soft (optional)
    pub is_hard: bool,
    /// Optional reason for the dependency
    pub reason: Option<String>,
}

/// Result of topological sorting with execution phases
#[derive(Debug, Clone)]
pub struct ExecutionOrder {
    /// Components in execution order
    pub sequence: Vec<ComponentId>,
    /// Phases of execution (components that can run in parallel)
    pub phases: Vec<Vec<ComponentId>>,
    /// Any warnings about the execution order
    pub warnings: Vec<String>,
}

/// Error types for dependency graph operations
#[derive(Debug, Clone)]
pub enum DependencyError {
    /// Circular dependency detected
    CircularDependency { cycle: Box<Vec<ComponentId>> },
    /// Missing dependency
    MissingDependency {
        component: Box<ComponentId>,
        missing_dependency: Box<ComponentId>,
    },
    /// Invalid dependency relationship
    InvalidDependency {
        from: Box<ComponentId>,
        to: Box<ComponentId>,
        reason: Box<String>,
    },
}

impl fmt::Display for DependencyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DependencyError::CircularDependency { cycle } => {
                write!(f, "Circular dependency detected: {:?}", cycle)
            }
            DependencyError::MissingDependency {
                component,
                missing_dependency,
            } => write!(
                f,
                "Component {:?} depends on missing component {:?}",
                component, missing_dependency
            ),
            DependencyError::InvalidDependency { from, to, reason } => {
                write!(
                    f,
                    "Invalid dependency from {:?} to {:?}: {}",
                    from, to, reason
                )
            }
        }
    }
}

impl std::error::Error for DependencyError {}

impl DependencyGraph {
    /// Creates a new empty dependency graph
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            dependencies: HashMap::new(),
            dependents: HashMap::new(),
        }
    }

    /// Adds a component node to the graph
    pub fn add_node(&mut self, node: DependencyNode) -> Result<()> {
        let component_id = node.component_id.clone();

        if self.nodes.contains_key(&component_id) {
            warn!(
                component_id = ?component_id,
                "Component already exists in dependency graph, updating"
            );
        }

        self.nodes.insert(component_id.clone(), node);
        self.dependencies.entry(component_id.clone()).or_default();
        self.dependents.entry(component_id.clone()).or_default();

        debug!(
            component_id = ?component_id,
            "Added component to dependency graph"
        );

        Ok(())
    }

    /// Adds a dependency relationship between components
    pub fn add_dependency(
        &mut self,
        component: &ComponentId,
        depends_on: &ComponentId,
        hook_point: HookPoint,
    ) -> Result<()> {
        self.add_dependency_with_options(
            component, depends_on, hook_point, true, // hard dependency by default
            None,
        )
    }

    /// Adds a dependency with additional options
    pub fn add_dependency_with_options(
        &mut self,
        component: &ComponentId,
        depends_on: &ComponentId,
        hook_point: HookPoint,
        is_hard: bool,
        reason: Option<String>,
    ) -> Result<()> {
        // Ensure both components exist as nodes
        if !self.nodes.contains_key(component) {
            let node = DependencyNode::new(component.clone());
            self.add_node(node)?;
        }
        if !self.nodes.contains_key(depends_on) {
            let node = DependencyNode::new(depends_on.clone());
            self.add_node(node)?;
        }

        let dependency = Dependency {
            depends_on: depends_on.clone(),
            hook_point: hook_point.clone(),
            is_hard,
            reason,
        };

        // Add to dependencies
        self.dependencies
            .entry(component.clone())
            .or_default()
            .push(dependency);

        // Add to dependents (reverse mapping)
        self.dependents
            .entry(depends_on.clone())
            .or_default()
            .push(component.clone());

        debug!(
            component = ?component,
            depends_on = ?depends_on,
            hook_point = ?hook_point,
            is_hard = is_hard,
            "Added dependency relationship"
        );

        // Check for circular dependencies after adding
        if let Err(e) = self.detect_cycles() {
            // Remove the dependency we just added
            if let Some(deps) = self.dependencies.get_mut(component) {
                deps.retain(|d| d.depends_on != *depends_on || d.hook_point != hook_point);
            }
            if let Some(deps) = self.dependents.get_mut(depends_on) {
                deps.retain(|d| d != component);
            }
            return Err(e.into());
        }

        Ok(())
    }

    /// Removes a dependency relationship
    pub fn remove_dependency(
        &mut self,
        component: &ComponentId,
        depends_on: &ComponentId,
        hook_point: HookPoint,
    ) -> Result<bool> {
        let mut removed = false;

        if let Some(deps) = self.dependencies.get_mut(component) {
            let initial_len = deps.len();
            deps.retain(|d| d.depends_on != *depends_on || d.hook_point != hook_point);
            removed = deps.len() < initial_len;
        }

        if removed {
            if let Some(deps) = self.dependents.get_mut(depends_on) {
                deps.retain(|d| d != component);
            }

            debug!(
                component = ?component,
                depends_on = ?depends_on,
                hook_point = ?hook_point,
                "Removed dependency relationship"
            );
        }

        Ok(removed)
    }

    /// Gets the execution order using topological sort
    pub fn get_execution_order(&self) -> Result<ExecutionOrder> {
        self.topological_sort()
    }

    /// Gets execution order for a specific hook point
    pub fn get_execution_order_for_hook(&self, hook_point: HookPoint) -> Result<ExecutionOrder> {
        // Filter graph to only include components that participate in this hook point
        let filtered_nodes: HashMap<ComponentId, DependencyNode> = self
            .nodes
            .iter()
            .filter(|(_, node)| node.hook_points.contains(&hook_point))
            .map(|(id, node)| (id.clone(), node.clone()))
            .collect();

        if filtered_nodes.is_empty() {
            return Ok(ExecutionOrder {
                sequence: Vec::new(),
                phases: Vec::new(),
                warnings: Vec::new(),
            });
        }

        // Create a temporary graph with filtered nodes
        let mut temp_graph = DependencyGraph::new();
        for (_id, node) in filtered_nodes {
            temp_graph.add_node(node)?;
        }

        // Add relevant dependencies
        for (component, deps) in &self.dependencies {
            if temp_graph.nodes.contains_key(component) {
                for dep in deps {
                    if dep.hook_point == hook_point
                        && temp_graph.nodes.contains_key(&dep.depends_on)
                    {
                        temp_graph.add_dependency_with_options(
                            component,
                            &dep.depends_on,
                            dep.hook_point.clone(),
                            dep.is_hard,
                            dep.reason.clone(),
                        )?;
                    }
                }
            }
        }

        temp_graph.topological_sort()
    }

    /// Detects circular dependencies in the graph
    pub fn detect_cycles(&self) -> Result<(), DependencyError> {
        let mut visited = HashSet::new();
        let mut rec_stack = HashSet::new();
        let mut path = Vec::new();

        for component_id in self.nodes.keys() {
            if !visited.contains(component_id) {
                if let Some(cycle) =
                    self.dfs_cycle_detection(component_id, &mut visited, &mut rec_stack, &mut path)
                {
                    return Err(DependencyError::CircularDependency {
                        cycle: Box::new(cycle),
                    });
                }
            }
        }

        Ok(())
    }

    /// Performs topological sort to determine execution order
    fn topological_sort(&self) -> Result<ExecutionOrder> {
        let mut in_degree: HashMap<ComponentId, usize> = HashMap::new();
        let mut warnings = Vec::new();

        // Initialize in-degree for all nodes
        for component_id in self.nodes.keys() {
            in_degree.insert(component_id.clone(), 0);
        }

        // Calculate in-degrees
        for (component, deps) in &self.dependencies {
            for dep in deps {
                if dep.is_hard {
                    *in_degree.entry(component.clone()).or_insert(0) += 1;
                } else {
                    warnings.push(format!(
                        "Soft dependency from {:?} to {:?} may affect execution order",
                        component, dep.depends_on
                    ));
                }
            }
        }

        let mut queue = VecDeque::new();
        let mut sequence = Vec::new();
        let mut phases = Vec::new();

        // Add nodes with no dependencies to queue
        for (component_id, &degree) in &in_degree {
            if degree == 0 {
                queue.push_back(component_id.clone());
            }
        }

        // Process queue level by level to create phases
        while !queue.is_empty() {
            let mut current_phase = Vec::new();
            let phase_size = queue.len();

            for _ in 0..phase_size {
                if let Some(component_id) = queue.pop_front() {
                    sequence.push(component_id.clone());
                    current_phase.push(component_id.clone());

                    // Reduce in-degree for components that depend on this one
                    for (other_component, deps) in &self.dependencies {
                        for dep in deps {
                            if dep.depends_on == component_id && dep.is_hard {
                                if let Some(degree) = in_degree.get_mut(other_component) {
                                    *degree -= 1;
                                    if *degree == 0 {
                                        queue.push_back(other_component.clone());
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if !current_phase.is_empty() {
                phases.push(current_phase);
            }
        }

        // Check if all nodes were processed (no cycles)
        if sequence.len() != self.nodes.len() {
            let unprocessed: Vec<ComponentId> = self
                .nodes
                .keys()
                .filter(|id| !sequence.contains(id))
                .cloned()
                .collect();

            return Err(DependencyError::CircularDependency {
                cycle: Box::new(unprocessed),
            }
            .into());
        }

        Ok(ExecutionOrder {
            sequence,
            phases,
            warnings,
        })
    }

    /// DFS-based cycle detection
    fn dfs_cycle_detection(
        &self,
        component_id: &ComponentId,
        visited: &mut HashSet<ComponentId>,
        rec_stack: &mut HashSet<ComponentId>,
        path: &mut Vec<ComponentId>,
    ) -> Option<Vec<ComponentId>> {
        visited.insert(component_id.clone());
        rec_stack.insert(component_id.clone());
        path.push(component_id.clone());

        if let Some(deps) = self.dependencies.get(component_id) {
            for dep in deps {
                if dep.is_hard {
                    if !visited.contains(&dep.depends_on) {
                        if let Some(cycle) =
                            self.dfs_cycle_detection(&dep.depends_on, visited, rec_stack, path)
                        {
                            return Some(cycle);
                        }
                    } else if rec_stack.contains(&dep.depends_on) {
                        // Found a cycle - return the cycle path
                        let cycle_start = path
                            .iter()
                            .position(|id| id == &dep.depends_on)
                            .unwrap_or(0);
                        return Some(path[cycle_start..].to_vec());
                    }
                }
            }
        }

        rec_stack.remove(component_id);
        path.pop();
        None
    }
}

impl DependencyNode {
    /// Creates a new dependency node
    pub fn new(component_id: ComponentId) -> Self {
        Self {
            component_id,
            hook_points: HashSet::new(),
            metadata: HashMap::new(),
            priority: 0,
        }
    }

    /// Adds a hook point to this node
    pub fn add_hook_point(mut self, hook_point: HookPoint) -> Self {
        self.hook_points.insert(hook_point);
        self
    }

    /// Sets the priority for this node
    pub fn with_priority(mut self, priority: i32) -> Self {
        self.priority = priority;
        self
    }

    /// Adds metadata to this node
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[cfg_attr(test_category = "hook")]
mod tests {
    use super::*;
    use crate::{ComponentType, HookPoint};

    fn create_test_component(name: &str, component_type: ComponentType) -> ComponentId {
        ComponentId::new(component_type, name.to_string())
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_dependency_graph_creation() {
        let graph = DependencyGraph::new();
        assert!(graph.nodes.is_empty());
        assert!(graph.dependencies.is_empty());
        assert!(graph.dependents.is_empty());
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_add_node() {
        let mut graph = DependencyGraph::new();
        let agent_id = create_test_component("agent", ComponentType::Agent);

        let node = DependencyNode::new(agent_id.clone())
            .add_hook_point(HookPoint::BeforeAgentInit)
            .with_priority(10);

        graph.add_node(node).expect("Should add node successfully");

        assert!(graph.nodes.contains_key(&agent_id));
        assert_eq!(graph.nodes[&agent_id].priority, 10);
        assert!(graph.nodes[&agent_id]
            .hook_points
            .contains(&HookPoint::BeforeAgentInit));
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_add_dependency() {
        let mut graph = DependencyGraph::new();
        let agent_id = create_test_component("agent", ComponentType::Agent);
        let tool_id = create_test_component("tool", ComponentType::Tool);

        graph
            .add_dependency(&tool_id, &agent_id, HookPoint::BeforeAgentExecution)
            .expect("Should add dependency successfully");

        assert!(graph.dependencies.contains_key(&tool_id));
        assert_eq!(graph.dependencies[&tool_id].len(), 1);
        assert_eq!(graph.dependencies[&tool_id][0].depends_on, agent_id);

        assert!(graph.dependents.contains_key(&agent_id));
        assert_eq!(graph.dependents[&agent_id].len(), 1);
        assert_eq!(graph.dependents[&agent_id][0], tool_id);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_circular_dependency_detection() {
        let mut graph = DependencyGraph::new();
        let agent_id = create_test_component("agent", ComponentType::Agent);
        let tool_id = create_test_component("tool", ComponentType::Tool);

        // Add A -> B
        graph
            .add_dependency(&agent_id, &tool_id, HookPoint::BeforeAgentExecution)
            .expect("Should add first dependency");

        // Try to add B -> A (creates cycle)
        let result = graph.add_dependency(&tool_id, &agent_id, HookPoint::BeforeAgentExecution);

        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("Circular dependency"));
        }
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_topological_sort() {
        let mut graph = DependencyGraph::new();
        let agent_id = create_test_component("agent", ComponentType::Agent);
        let tool_id = create_test_component("tool", ComponentType::Tool);
        let workflow_id = create_test_component("workflow", ComponentType::Workflow);

        // Create chain: agent -> tool -> workflow
        // tool depends on agent (agent executes first)
        graph
            .add_dependency(&tool_id, &agent_id, HookPoint::BeforeAgentExecution)
            .expect("Should add tool->agent dependency");
        // workflow depends on tool (tool executes first)
        graph
            .add_dependency(&workflow_id, &tool_id, HookPoint::BeforeWorkflowStart)
            .expect("Should add workflow->tool dependency");

        let order = graph
            .get_execution_order()
            .expect("Should get execution order");

        assert_eq!(order.sequence.len(), 3);
        assert!(
            order
                .sequence
                .iter()
                .position(|id| id == &agent_id)
                .unwrap()
                < order.sequence.iter().position(|id| id == &tool_id).unwrap()
        );
        assert!(
            order.sequence.iter().position(|id| id == &tool_id).unwrap()
                < order
                    .sequence
                    .iter()
                    .position(|id| id == &workflow_id)
                    .unwrap()
        );
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_parallel_execution_phases() {
        let mut graph = DependencyGraph::new();
        let agent_id = create_test_component("agent", ComponentType::Agent);
        let tool1_id = create_test_component("tool1", ComponentType::Tool);
        let tool2_id = create_test_component("tool2", ComponentType::Tool);
        let workflow_id = create_test_component("workflow", ComponentType::Workflow);

        // Create diamond pattern: agent -> (tool1, tool2) -> workflow
        graph
            .add_dependency(&tool1_id, &agent_id, HookPoint::BeforeAgentExecution)
            .expect("Should add agent->tool1 dependency");
        graph
            .add_dependency(&tool2_id, &agent_id, HookPoint::BeforeAgentExecution)
            .expect("Should add agent->tool2 dependency");
        graph
            .add_dependency(&workflow_id, &tool1_id, HookPoint::BeforeWorkflowStart)
            .expect("Should add tool1->workflow dependency");
        graph
            .add_dependency(&workflow_id, &tool2_id, HookPoint::BeforeWorkflowStart)
            .expect("Should add tool2->workflow dependency");

        let order = graph
            .get_execution_order()
            .expect("Should get execution order");

        assert_eq!(order.phases.len(), 3);
        assert_eq!(order.phases[0].len(), 1); // agent only
        assert_eq!(order.phases[1].len(), 2); // both tools in parallel
        assert_eq!(order.phases[2].len(), 1); // workflow only

        assert!(order.phases[0].contains(&agent_id));
        assert!(order.phases[1].contains(&tool1_id));
        assert!(order.phases[1].contains(&tool2_id));
        assert!(order.phases[2].contains(&workflow_id));
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_hook_point_filtering() {
        let mut graph = DependencyGraph::new();
        let agent_id = create_test_component("agent", ComponentType::Agent);
        let tool_id = create_test_component("tool", ComponentType::Tool);

        // Add nodes with specific hook points
        let agent_node = DependencyNode::new(agent_id.clone())
            .add_hook_point(HookPoint::BeforeAgentInit)
            .add_hook_point(HookPoint::BeforeAgentExecution);
        let tool_node =
            DependencyNode::new(tool_id.clone()).add_hook_point(HookPoint::BeforeAgentExecution);

        graph.add_node(agent_node).expect("Should add agent node");
        graph.add_node(tool_node).expect("Should add tool node");

        graph
            .add_dependency(&tool_id, &agent_id, HookPoint::BeforeAgentExecution)
            .expect("Should add dependency");

        // Get order for BeforeAgentExecution - should include both
        let order = graph
            .get_execution_order_for_hook(HookPoint::BeforeAgentExecution)
            .expect("Should get execution order");
        assert_eq!(order.sequence.len(), 2);

        // Get order for BeforeAgentInit - should include only agent
        let order = graph
            .get_execution_order_for_hook(HookPoint::BeforeAgentInit)
            .expect("Should get execution order");
        assert_eq!(order.sequence.len(), 1);
        assert_eq!(order.sequence[0], agent_id);
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_dependency_removal() {
        let mut graph = DependencyGraph::new();
        let agent_id = create_test_component("agent", ComponentType::Agent);
        let tool_id = create_test_component("tool", ComponentType::Tool);

        graph
            .add_dependency(&tool_id, &agent_id, HookPoint::BeforeAgentExecution)
            .expect("Should add dependency");

        let removed = graph
            .remove_dependency(&tool_id, &agent_id, HookPoint::BeforeAgentExecution)
            .expect("Should remove dependency");

        assert!(removed);
        assert!(graph.dependencies[&tool_id].is_empty());
        assert!(graph.dependents[&agent_id].is_empty());
    }
}
