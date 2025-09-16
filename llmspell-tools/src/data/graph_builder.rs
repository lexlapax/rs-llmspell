// ABOUTME: Graph data structure builder tool for network analysis and visualization
// ABOUTME: Provides graph creation, manipulation, and JSON serialization with multiple graph types

//! Graph Builder tool
//!
//! This tool provides graph data structure manipulation including:
//! - Directed and undirected graph creation
//! - Node and edge management with custom data
//! - JSON import/export with serialization
//! - Graph analysis (connectivity, metrics)
//! - Multiple graph backends (adjacency list, stable indices, label-based)

use crate::resource_limited::ResourceLimited;
use async_trait::async_trait;
use llmspell_core::{
    traits::{
        base_agent::BaseAgent,
        tool::{
            ParameterDef, ParameterType, ResourceLimits as ToolResourceLimits, SecurityLevel,
            SecurityRequirements, Tool, ToolCategory, ToolSchema,
        },
    },
    types::{AgentInput, AgentOutput},
    ComponentMetadata, ExecutionContext, LLMSpellError, Result,
};
use llmspell_utils::{
    error_builders::llmspell::{tool_error, validation_error},
    params::{
        extract_optional_object, extract_optional_string, extract_parameters,
        extract_required_string, extract_string_with_default,
    },
    resource_limits::{ResourceLimits, ResourceTracker},
    response::ResponseBuilder,
    timeout::with_timeout,
};
// Phase 7 implementation - petgraph integration for Phase 8
// use petgraph::Graph;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value as JsonValue};
use std::collections::HashMap;
use std::time::Instant;
use tracing::{debug, error, info, warn};

/// Graph types supported by the tool
#[derive(Debug, Clone)]
pub enum GraphType {
    Directed,
    Undirected,
}

/// Node data structure for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeData {
    pub id: String,
    pub label: Option<String>,
    pub data: Option<JsonValue>,
}

/// Edge data structure for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeData {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
    pub weight: Option<f64>,
    pub data: Option<JsonValue>,
}

/// Serializable graph structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableGraph {
    pub graph_type: String,
    pub nodes: Vec<NodeData>,
    pub edges: Vec<EdgeData>,
    pub metadata: Option<JsonValue>,
}

/// Graph Builder tool for data structure manipulation
#[derive(Debug, Clone)]
pub struct GraphBuilderTool {
    /// Tool metadata
    metadata: ComponentMetadata,
    /// Maximum number of nodes
    max_nodes: usize,
    /// Maximum number of edges
    max_edges: usize,
    /// Maximum JSON size for import/export
    max_json_size: usize,
}

impl GraphBuilderTool {
    /// Create a new graph builder tool
    #[must_use]
    pub fn new() -> Self {
        info!(
            max_nodes = 10_000,
            max_edges = 50_000,
            max_json_size_mb = 10,
            supported_operations = 6, // create_graph, add_node, add_edge, analyze, export_json, import_json
            supported_graph_types = 2, // directed, undirected
            phase = "Phase 7 (basic graph operations)",
            "Creating GraphBuilderTool"
        );
        Self {
            metadata: ComponentMetadata::new(
                "graph-builder".to_string(),
                "Build and analyze graph data structures with JSON serialization".to_string(),
            ),
            max_nodes: 10_000,               // Maximum 10K nodes
            max_edges: 50_000,               // Maximum 50K edges
            max_json_size: 10 * 1024 * 1024, // 10MB JSON
        }
    }

    /// Create a new empty graph
    #[allow(clippy::unused_self)]
    fn create_empty_graph(&self, graph_type: &GraphType) -> SerializableGraph {
        let type_str = match graph_type {
            GraphType::Directed => "directed".to_string(),
            GraphType::Undirected => "undirected".to_string(),
        };

        SerializableGraph {
            graph_type: type_str,
            nodes: Vec::new(),
            edges: Vec::new(),
            metadata: Some(json!({
                "created_at": chrono::Utc::now().to_rfc3339(),
                "node_count": 0,
                "edge_count": 0
            })),
        }
    }

    /// Add node to graph
    #[allow(clippy::cognitive_complexity)]
    fn add_node_to_graph(
        &self,
        mut graph: SerializableGraph,
        node_id: &str,
        label: Option<String>,
        data: Option<JsonValue>,
    ) -> Result<SerializableGraph> {
        let add_node_start = Instant::now();
        debug!(
            operation = "add_node",
            node_id = %node_id,
            has_label = label.is_some(),
            has_data = data.is_some(),
            current_nodes = graph.nodes.len(),
            max_nodes = self.max_nodes,
            "Starting add node operation"
        );

        // Check if node already exists
        if graph.nodes.iter().any(|n| n.id == node_id) {
            warn!(
                operation = "add_node",
                node_id = %node_id,
                "Node with specified ID already exists"
            );
            return Err(validation_error(
                format!("Node with id '{node_id}' already exists"),
                Some("node_id".to_string()),
            ));
        }

        // Check node limit
        if graph.nodes.len() >= self.max_nodes {
            error!(
                operation = "add_node",
                current_nodes = graph.nodes.len(),
                max_nodes = self.max_nodes,
                "Maximum number of nodes reached"
            );
            return Err(validation_error(
                format!("Maximum number of nodes reached: {}", self.max_nodes),
                Some("max_nodes".to_string()),
            ));
        }

        // Add the node
        graph.nodes.push(NodeData {
            id: node_id.to_string(),
            label,
            data,
        });

        // Update metadata
        if let Some(ref mut meta) = graph.metadata {
            if let Some(meta_obj) = meta.as_object_mut() {
                meta_obj.insert("node_count".to_string(), json!(graph.nodes.len()));
                meta_obj.insert(
                    "last_modified".to_string(),
                    json!(chrono::Utc::now().to_rfc3339()),
                );
            }
        }

        let add_node_duration_ms = add_node_start.elapsed().as_millis();
        debug!(
            operation = "add_node",
            node_id = %node_id,
            total_nodes = graph.nodes.len(),
            success = true,
            duration_ms = add_node_duration_ms,
            "Add node operation completed successfully"
        );

        Ok(graph)
    }

    /// Add edge to graph
    fn add_edge_to_graph(
        &self,
        mut graph: SerializableGraph,
        from: &str,
        to: &str,
        label: Option<String>,
        weight: Option<f64>,
        data: Option<JsonValue>,
    ) -> Result<SerializableGraph> {
        let add_edge_start = Instant::now();
        debug!(
            operation = "add_edge",
            from = %from,
            to = %to,
            has_label = label.is_some(),
            has_weight = weight.is_some(),
            has_data = data.is_some(),
            current_edges = graph.edges.len(),
            max_edges = self.max_edges,
            "Starting add edge operation"
        );

        Self::validate_edge_nodes(&graph, from, to)?;
        self.validate_edge_limit(&graph)?;

        // Add the edge
        graph.edges.push(EdgeData {
            from: from.to_string(),
            to: to.to_string(),
            label,
            weight,
            data,
        });

        Self::update_graph_metadata(&mut graph);

        let add_edge_duration_ms = add_edge_start.elapsed().as_millis();
        debug!(
            operation = "add_edge",
            from = %from,
            to = %to,
            total_edges = graph.edges.len(),
            success = true,
            duration_ms = add_edge_duration_ms,
            "Add edge operation completed successfully"
        );

        Ok(graph)
    }

    fn validate_edge_nodes(graph: &SerializableGraph, from: &str, to: &str) -> Result<()> {
        if !graph.nodes.iter().any(|n| n.id == from) {
            warn!(
                operation = "add_edge",
                from = %from,
                "Source node does not exist"
            );
            return Err(validation_error(
                format!("Source node '{from}' does not exist"),
                Some("from".to_string()),
            ));
        }
        if !graph.nodes.iter().any(|n| n.id == to) {
            warn!(
                operation = "add_edge",
                to = %to,
                "Target node does not exist"
            );
            return Err(validation_error(
                format!("Target node '{to}' does not exist"),
                Some("to".to_string()),
            ));
        }
        Ok(())
    }

    fn validate_edge_limit(&self, graph: &SerializableGraph) -> Result<()> {
        if graph.edges.len() >= self.max_edges {
            error!(
                operation = "add_edge",
                current_edges = graph.edges.len(),
                max_edges = self.max_edges,
                "Maximum number of edges reached"
            );
            return Err(validation_error(
                format!("Maximum number of edges reached: {}", self.max_edges),
                Some("max_edges".to_string()),
            ));
        }
        Ok(())
    }

    fn update_graph_metadata(graph: &mut SerializableGraph) {
        if let Some(ref mut meta) = graph.metadata {
            if let Some(meta_obj) = meta.as_object_mut() {
                meta_obj.insert("edge_count".to_string(), json!(graph.edges.len()));
                meta_obj.insert(
                    "last_modified".to_string(),
                    json!(chrono::Utc::now().to_rfc3339()),
                );
            }
        }
    }

    /// Analyze graph structure
    #[allow(clippy::unused_self)]
    fn analyze_graph(&self, graph: &SerializableGraph) -> JsonValue {
        let node_count = graph.nodes.len();
        let edge_count = graph.edges.len();

        // Calculate degree statistics
        let mut degree_map: HashMap<String, usize> = HashMap::new();
        for node in &graph.nodes {
            degree_map.insert(node.id.clone(), 0);
        }

        // Count connections
        for edge in &graph.edges {
            *degree_map.entry(edge.from.clone()).or_insert(0) += 1;
            if graph.graph_type == "undirected" {
                *degree_map.entry(edge.to.clone()).or_insert(0) += 1;
            } else {
                // For directed graphs, count in-degree separately if needed
            }
        }

        let degrees: Vec<usize> = degree_map.values().copied().collect();
        let max_degree = degrees.iter().max().copied().unwrap_or(0);
        let min_degree = degrees.iter().min().copied().unwrap_or(0);
        let avg_degree = if degrees.is_empty() {
            0.0
        } else {
            #[allow(clippy::cast_precision_loss)]
            {
                degrees.iter().sum::<usize>() as f64 / degrees.len() as f64
            }
        };

        // Basic connectivity analysis
        let is_empty = node_count == 0;
        let is_complete = if node_count > 1 {
            edge_count == (node_count * (node_count - 1)) / 2
        } else {
            false
        };

        json!({
            "node_count": node_count,
            "edge_count": edge_count,
            "graph_type": graph.graph_type,
            "density": if node_count > 1 {
                #[allow(clippy::cast_precision_loss)]
                {
                    edge_count as f64 / ((node_count * (node_count - 1)) as f64 / 2.0)
                }
            } else {
                0.0
            },
            "degree_statistics": {
                "max_degree": max_degree,
                "min_degree": min_degree,
                "average_degree": avg_degree
            },
            "properties": {
                "is_empty": is_empty,
                "is_complete": is_complete,
                "has_self_loops": graph.edges.iter().any(|e| e.from == e.to)
            },
            "node_ids": graph.nodes.iter().map(|n| &n.id).collect::<Vec<_>>(),
            "note": "Advanced analysis (components, paths, centrality) available with graph algorithms library"
        })
    }

    fn validate_json_size(&self, json_str: &str) -> Result<()> {
        if json_str.len() > self.max_json_size {
            error!(
                operation = "import_graph",
                json_size = json_str.len(),
                max_json_size = self.max_json_size,
                "JSON size exceeds maximum limit"
            );
            return Err(validation_error(
                format!(
                    "JSON too large: {} bytes (max: {} bytes)",
                    json_str.len(),
                    self.max_json_size
                ),
                Some("input".to_string()),
            ));
        }
        Ok(())
    }

    fn validate_graph_constraints(&self, graph: &SerializableGraph) -> Result<()> {
        if graph.nodes.len() > self.max_nodes {
            return Err(validation_error(
                format!(
                    "Too many nodes: {} (max: {})",
                    graph.nodes.len(),
                    self.max_nodes
                ),
                Some("nodes".to_string()),
            ));
        }

        if graph.edges.len() > self.max_edges {
            return Err(validation_error(
                format!(
                    "Too many edges: {} (max: {})",
                    graph.edges.len(),
                    self.max_edges
                ),
                Some("edges".to_string()),
            ));
        }

        Ok(())
    }

    fn validate_edge_references(graph: &SerializableGraph) -> Result<()> {
        let node_ids: std::collections::HashSet<String> =
            graph.nodes.iter().map(|n| n.id.clone()).collect();

        for edge in &graph.edges {
            if !node_ids.contains(&edge.from) {
                error!(
                    operation = "import_graph",
                    edge_from = %edge.from,
                    "Edge references non-existent source node"
                );
                return Err(validation_error(
                    format!("Edge references non-existent node: {}", edge.from),
                    Some("edges".to_string()),
                ));
            }
            if !node_ids.contains(&edge.to) {
                error!(
                    operation = "import_graph",
                    edge_to = %edge.to,
                    "Edge references non-existent target node"
                );
                return Err(validation_error(
                    format!("Edge references non-existent node: {}", edge.to),
                    Some("edges".to_string()),
                ));
            }
        }

        Ok(())
    }

    /// Import graph from JSON
    async fn import_graph_from_json(&self, json_str: &str) -> Result<SerializableGraph> {
        let import_start = Instant::now();
        debug!(
            operation = "import_graph",
            json_size = json_str.len(),
            max_json_size = self.max_json_size,
            "Starting graph JSON import"
        );

        // Validate JSON size
        self.validate_json_size(json_str)?;

        // Parse JSON with timeout
        let graph: SerializableGraph =
            with_timeout(std::time::Duration::from_secs(10), async move {
                serde_json::from_str(json_str).map_err(|e| {
                    tool_error(
                        format!("JSON parsing failed: {e}"),
                        Some("json_parse".to_string()),
                    )
                })
            })
            .await
            .map_err(|_| {
                tool_error(
                    "JSON parsing timed out after 10 seconds".to_string(),
                    Some("timeout".to_string()),
                )
            })??;

        // Validate graph constraints
        self.validate_graph_constraints(&graph)?;

        // Validate edge references
        let validation_start = Instant::now();
        Self::validate_edge_references(&graph)?;

        let import_duration_ms = import_start.elapsed().as_millis();
        let validation_duration_ms = validation_start.elapsed().as_millis();
        info!(
            operation = "import_graph",
            nodes_imported = graph.nodes.len(),
            edges_imported = graph.edges.len(),
            graph_type = %graph.graph_type,
            validation_duration_ms,
            total_duration_ms = import_duration_ms,
            "Graph JSON import completed successfully"
        );

        Ok(graph)
    }

    /// Export graph to JSON
    #[allow(clippy::unused_self)]
    fn export_graph_to_json(&self, graph: &SerializableGraph) -> Result<String> {
        serde_json::to_string_pretty(graph).map_err(|e| {
            tool_error(
                format!("JSON serialization failed: {e}"),
                Some("json_serialize".to_string()),
            )
        })
    }
}

impl Default for GraphBuilderTool {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl BaseAgent for GraphBuilderTool {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    #[allow(clippy::too_many_lines)]
    async fn execute_impl(
        &self,
        input: AgentInput,
        _context: ExecutionContext,
    ) -> Result<AgentOutput> {
        let start = Instant::now();
        info!(
            input_size = input.text.len(),
            has_params = !input.parameters.is_empty(),
            "Executing graph builder tool"
        );

        // Create resource tracker for this execution
        let limits = ResourceLimits {
            max_memory_bytes: Some(100 * 1024 * 1024), // 100MB for graph processing
            max_cpu_time_ms: Some(30_000),             // 30 seconds
            max_operations: Some(10_000),              // 10K operations
            operation_timeout_ms: Some(30_000),        // 30 seconds
            ..Default::default()
        };
        let tracker = ResourceTracker::new(limits);

        // Track the operation
        tracker.track_operation()?;

        // Get parameters using shared utility
        let params = extract_parameters(&input)?;

        // Extract operation type
        let operation = extract_string_with_default(params, "operation", "create_graph");

        debug!(
            operation = %operation,
            "Processing graph builder operation"
        );

        // Execute based on operation
        let result = match operation {
            "create_graph" => {
                let graph_type_str = extract_string_with_default(params, "graph_type", "directed");
                let graph_type = match graph_type_str {
                    "directed" => GraphType::Directed,
                    "undirected" => GraphType::Undirected,
                    _ => {
                        return Err(validation_error(
                            format!("Invalid graph_type: {graph_type_str}. Supported: directed, undirected"),
                            Some("graph_type".to_string()),
                        ));
                    }
                };

                let graph = self.create_empty_graph(&graph_type);

                json!({
                    "operation": "create_graph",
                    "success": true,
                    "result": graph,
                    "metadata": {
                        "tool": "graph-builder",
                        "operation": "create_graph"
                    }
                })
            }
            "add_node" => {
                let graph_json = extract_required_string(params, "graph")?;
                let graph = self.import_graph_from_json(graph_json).await?;

                let node_id = extract_required_string(params, "node_id")?;
                let label =
                    extract_optional_string(params, "label").map(std::string::ToString::to_string);
                let data = extract_optional_object(params, "data")
                    .map(|obj| JsonValue::Object(obj.clone()));

                let updated_graph = self.add_node_to_graph(graph, node_id, label, data)?;

                json!({
                    "operation": "add_node",
                    "success": true,
                    "result": updated_graph,
                    "metadata": {
                        "tool": "graph-builder",
                        "operation": "add_node"
                    }
                })
            }
            "add_edge" => {
                let graph_json = extract_required_string(params, "graph")?;
                let graph = self.import_graph_from_json(graph_json).await?;

                let from = extract_required_string(params, "from")?;
                let to = extract_required_string(params, "to")?;
                let label =
                    extract_optional_string(params, "label").map(std::string::ToString::to_string);
                let weight = params.get("weight").and_then(serde_json::Value::as_f64);
                let data = extract_optional_object(params, "data")
                    .map(|obj| JsonValue::Object(obj.clone()));

                let updated_graph = self.add_edge_to_graph(graph, from, to, label, weight, data)?;

                json!({
                    "operation": "add_edge",
                    "success": true,
                    "result": updated_graph,
                    "metadata": {
                        "tool": "graph-builder",
                        "operation": "add_edge"
                    }
                })
            }
            "analyze" => {
                let graph_json = extract_required_string(params, "input")?;
                let graph = self.import_graph_from_json(graph_json).await?;

                let analysis = self.analyze_graph(&graph);

                json!({
                    "operation": "analyze",
                    "success": true,
                    "result": analysis,
                    "metadata": {
                        "tool": "graph-builder",
                        "operation": "analyze"
                    }
                })
            }
            "export_json" => {
                let graph_json = extract_required_string(params, "input")?;
                let graph = self.import_graph_from_json(graph_json).await?;

                let exported = self.export_graph_to_json(&graph)?;

                json!({
                    "operation": "export_json",
                    "success": true,
                    "result": {
                        "json": exported,
                        "format": "json",
                        "node_count": graph.nodes.len(),
                        "edge_count": graph.edges.len()
                    },
                    "metadata": {
                        "tool": "graph-builder",
                        "operation": "export_json"
                    }
                })
            }
            "import_json" => {
                let graph_json = extract_required_string(params, "input")?;
                let graph = self.import_graph_from_json(graph_json).await?;

                json!({
                    "operation": "import_json",
                    "success": true,
                    "result": graph,
                    "metadata": {
                        "tool": "graph-builder",
                        "operation": "import_json"
                    }
                })
            }
            _ => {
                error!(
                    operation = %operation,
                    "Unsupported graph builder operation requested"
                );
                return Err(validation_error(
                    format!("Unsupported operation: {operation}. Supported: create_graph, add_node, add_edge, analyze, export_json, import_json"),
                    Some("operation".to_string()),
                ));
            }
        };

        let response = ResponseBuilder::success("graph_builder_execute")
            .with_result(result)
            .build();

        let elapsed_ms = start.elapsed().as_millis();
        info!(
            operation = %operation,
            success = true,
            duration_ms = elapsed_ms,
            "Graph builder execution completed successfully"
        );

        Ok(AgentOutput::text(serde_json::to_string_pretty(&response)?))
    }

    async fn validate_input(&self, input: &AgentInput) -> Result<()> {
        let params = extract_parameters(input)?;

        // Validate operation
        let operation = extract_string_with_default(params, "operation", "create_graph");
        match operation {
            "create_graph" => {
                // Validate graph_type if provided
                let graph_type = extract_string_with_default(params, "graph_type", "directed");
                match graph_type {
                    "directed" | "undirected" => {}
                    _ => {
                        return Err(validation_error(
                            format!("Invalid graph_type: {graph_type}"),
                            Some("graph_type".to_string()),
                        ));
                    }
                }
            }
            "add_node" => {
                extract_required_string(params, "graph")?;
                extract_required_string(params, "node_id")?;
            }
            "add_edge" => {
                extract_required_string(params, "graph")?;
                extract_required_string(params, "from")?;
                extract_required_string(params, "to")?;
            }
            "analyze" | "export_json" | "import_json" => {
                extract_required_string(params, "input")?;
            }
            _ => {
                return Err(validation_error(
                    format!("Invalid operation: {operation}"),
                    Some("operation".to_string()),
                ));
            }
        }

        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
        let error_response = json!({
            "operation": "error",
            "success": false,
            "error": error.to_string(),
            "metadata": {
                "tool": "graph-builder"
            }
        });

        let response = ResponseBuilder::error("graph_builder_error", error.to_string())
            .with_result(error_response)
            .build();
        Ok(AgentOutput::text(serde_json::to_string_pretty(&response)?))
    }
}

#[async_trait]
impl Tool for GraphBuilderTool {
    fn category(&self) -> ToolCategory {
        ToolCategory::Data
    }

    fn security_level(&self) -> SecurityLevel {
        SecurityLevel::Safe // No file system or network access
    }

    fn schema(&self) -> ToolSchema {
        ToolSchema::new(
            "graph-builder".to_string(),
            "Build and analyze graph data structures with JSON serialization".to_string(),
        )
        .with_parameter(ParameterDef {
            name: "operation".to_string(),
            param_type: ParameterType::String,
            description:
                "Operation: create_graph, add_node, add_edge, analyze, export_json, import_json"
                    .to_string(),
            required: false,
            default: Some(json!("create_graph")),
        })
        .with_parameter(ParameterDef {
            name: "input".to_string(),
            param_type: ParameterType::String,
            description: "Graph JSON for analysis/import operations".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "graph_type".to_string(),
            param_type: ParameterType::String,
            description: "Graph type for creation: directed or undirected".to_string(),
            required: false,
            default: Some(json!("directed")),
        })
        .with_parameter(ParameterDef {
            name: "graph".to_string(),
            param_type: ParameterType::String,
            description: "Existing graph JSON for modification operations".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "node_id".to_string(),
            param_type: ParameterType::String,
            description: "Node identifier for add_node operation".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "from".to_string(),
            param_type: ParameterType::String,
            description: "Source node ID for add_edge operation".to_string(),
            required: false,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "to".to_string(),
            param_type: ParameterType::String,
            description: "Target node ID for add_edge operation".to_string(),
            required: false,
            default: None,
        })
        .with_returns(ParameterType::Object)
    }

    fn security_requirements(&self) -> SecurityRequirements {
        SecurityRequirements {
            level: SecurityLevel::Safe,
            file_permissions: vec![],
            network_permissions: vec![],
            env_permissions: vec![],
            custom_requirements: std::collections::HashMap::new(),
        }
    }

    fn resource_limits(&self) -> ToolResourceLimits {
        ToolResourceLimits {
            max_memory_bytes: Some(100 * 1024 * 1024), // 100MB for graph processing
            max_cpu_time_ms: Some(30_000),             // 30 seconds
            max_network_bps: Some(0),                  // No network needed
            max_file_ops_per_sec: Some(0),             // No file operations
            custom_limits: std::collections::HashMap::new(),
        }
    }
}

impl ResourceLimited for GraphBuilderTool {}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_testing::tool_helpers::create_test_tool_input;

    #[tokio::test]
    async fn test_graph_builder_creation() {
        let tool = GraphBuilderTool::new();
        assert_eq!(tool.metadata().name, "graph-builder");
        assert_eq!(tool.category(), ToolCategory::Data);
        assert_eq!(tool.security_level(), SecurityLevel::Safe);
    }

    #[tokio::test]
    async fn test_create_empty_graph() {
        let tool = GraphBuilderTool::new();

        let input = create_test_tool_input(vec![
            ("operation", "create_graph"),
            ("graph_type", "directed"),
        ]);

        let context = ExecutionContext::default();
        let result = tool.execute(input, context).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.text.contains("directed"));
        assert!(output.text.contains("node_count"));
    }

    #[tokio::test]
    async fn test_add_node_to_graph() {
        let tool = GraphBuilderTool::new();

        // First create a graph
        let empty_graph = tool.create_empty_graph(&GraphType::Directed);
        let graph_json = serde_json::to_string(&empty_graph).unwrap();

        // Then add a node
        let input = create_test_tool_input(vec![
            ("operation", "add_node"),
            ("graph", &graph_json),
            ("node_id", "node1"),
            ("label", "Test Node"),
        ]);

        let context = ExecutionContext::default();
        let result = tool.execute(input, context).await;

        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.text.contains("node1"));
        assert!(output.text.contains("Test Node"));
    }

    #[tokio::test]
    async fn test_graph_builder_validation() {
        let tool = GraphBuilderTool::new();

        // Test missing required parameters for add_node
        let invalid_input = create_test_tool_input(vec![("operation", "add_node")]);
        let result = tool.validate_input(&invalid_input).await;
        assert!(result.is_err());

        // Test invalid operation
        let invalid_op_input = create_test_tool_input(vec![("operation", "invalid_op")]);
        let result = tool.validate_input(&invalid_op_input).await;
        assert!(result.is_err());

        // Test valid input for create_graph
        let valid_input = create_test_tool_input(vec![("operation", "create_graph")]);
        let result = tool.validate_input(&valid_input).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_graph_analysis() {
        let tool = GraphBuilderTool::new();

        // Create a test graph with nodes and edges
        let test_graph = SerializableGraph {
            graph_type: "directed".to_string(),
            nodes: vec![
                NodeData {
                    id: "a".to_string(),
                    label: Some("Node A".to_string()),
                    data: None,
                },
                NodeData {
                    id: "b".to_string(),
                    label: Some("Node B".to_string()),
                    data: None,
                },
            ],
            edges: vec![EdgeData {
                from: "a".to_string(),
                to: "b".to_string(),
                label: None,
                weight: Some(1.0),
                data: None,
            }],
            metadata: None,
        };

        let analysis = tool.analyze_graph(&test_graph);

        assert_eq!(analysis["node_count"], 2);
        assert_eq!(analysis["edge_count"], 1);
        assert_eq!(analysis["graph_type"], "directed");
    }

    #[tokio::test]
    async fn test_graph_builder_schema() {
        let tool = GraphBuilderTool::new();
        let schema = tool.schema();

        assert_eq!(schema.name, "graph-builder");
        assert!(schema.description.contains("graph"));
        assert!(schema.parameters.len() >= 6); // Should have multiple parameters

        // Check for operation parameter
        let op_param = schema.parameters.iter().find(|p| p.name == "operation");
        assert!(op_param.is_some());
    }

    #[tokio::test]
    async fn test_json_import_export() {
        let tool = GraphBuilderTool::new();

        let test_json = r#"{
            "graph_type": "undirected",
            "nodes": [
                {"id": "1", "label": "First", "data": null},
                {"id": "2", "label": "Second", "data": null}
            ],
            "edges": [
                {"from": "1", "to": "2", "label": "connection", "weight": 2.5, "data": null}
            ],
            "metadata": null
        }"#;

        let graph = tool.import_graph_from_json(test_json).await.unwrap();
        assert_eq!(graph.nodes.len(), 2);
        assert_eq!(graph.edges.len(), 1);
        assert_eq!(graph.graph_type, "undirected");

        let exported = tool.export_graph_to_json(&graph).unwrap();
        assert!(exported.contains("undirected"));
        assert!(exported.contains("First"));
        assert!(exported.contains("Second"));
    }
}
