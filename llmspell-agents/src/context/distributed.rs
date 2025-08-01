//! ABOUTME: Distributed context synchronization for multi-node deployments
//! ABOUTME: Provides context replication, consistency, and node discovery

use async_trait::async_trait;
use llmspell_core::execution_context::{ContextScope, ExecutionContext};
use llmspell_core::{LLMSpellError, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock};

/// Node information in distributed system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    /// Node ID
    pub id: String,
    /// Node address
    pub address: String,
    /// Node capabilities
    pub capabilities: Vec<String>,
    /// Node status
    pub status: NodeStatus,
    /// Last heartbeat
    pub last_heartbeat: chrono::DateTime<chrono::Utc>,
}

/// Node status
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum NodeStatus {
    /// Node is healthy and accepting requests
    Healthy,
    /// Node is overloaded
    Overloaded,
    /// Node is draining (preparing to shutdown)
    Draining,
    /// Node is unreachable
    Unreachable,
}

/// Distributed context for cross-node coordination
#[derive(Clone)]
pub struct DistributedContext {
    /// Local node info
    local_node: NodeInfo,
    /// Known nodes in the cluster
    nodes: Arc<RwLock<HashMap<String, NodeInfo>>>,
    /// Context replication manager
    replication: Arc<RwLock<ReplicationManager>>,
    /// Node discovery service
    discovery: Arc<dyn NodeDiscovery>,
    /// Consistency manager
    _consistency: ConsistencyManager,
}

/// Context replication strategy
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ReplicationStrategy {
    /// No replication
    None,
    /// Replicate to N nodes
    NReplicas(usize),
    /// Replicate to all nodes
    All,
    /// Replicate based on scope
    ScopeBased,
}

/// Replication manager
#[derive(Debug)]
struct ReplicationManager {
    /// Replication strategy
    strategy: ReplicationStrategy,
    /// Pending replications
    pending: HashMap<String, PendingReplication>,
    /// Replication metrics
    metrics: ReplicationMetrics,
}

/// Pending replication operation
#[derive(Debug, Clone)]
struct PendingReplication {
    /// Context to replicate
    _context_id: String,
    /// Target nodes
    target_nodes: Vec<String>,
    /// Creation time
    _created_at: Instant,
    /// Retry count
    _retry_count: u32,
}

/// Replication metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ReplicationMetrics {
    /// Total replications
    pub total_replications: u64,
    /// Successful replications
    pub successful_replications: u64,
    /// Failed replications
    pub failed_replications: u64,
    /// Average replication time
    pub avg_replication_time_ms: u64,
}

/// Node discovery trait
#[async_trait]
pub trait NodeDiscovery: Send + Sync {
    /// Discover nodes in the cluster
    async fn discover(&self) -> Result<Vec<NodeInfo>>;

    /// Register local node
    async fn register(&self, node: NodeInfo) -> Result<()>;

    /// Unregister local node
    async fn unregister(&self, node_id: &str) -> Result<()>;

    /// Send heartbeat
    async fn heartbeat(&self, node_id: &str) -> Result<()>;
}

/// Consistency level for distributed operations
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ConsistencyLevel {
    /// Eventually consistent
    Eventual,
    /// Strong consistency (linearizable)
    Strong,
    /// Read from quorum
    QuorumRead,
    /// Write to quorum
    QuorumWrite,
}

/// Consistency manager
#[derive(Debug, Clone)]
struct ConsistencyManager {
    /// Default consistency level
    _default_level: ConsistencyLevel,
    /// Quorum size
    _quorum_size: usize,
    /// Conflict resolution strategy
    _conflict_resolution: ConflictResolution,
}

/// Conflict resolution strategy
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ConflictResolution {
    /// Last write wins
    LastWriteWins,
    /// Highest version wins
    HighestVersion,
    /// Custom resolver
    Custom,
}

/// Context synchronization message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncMessage {
    /// Request to sync context
    SyncRequest {
        context_id: String,
        requester: String,
    },
    /// Context data for sync
    SyncData {
        context: SerializedContext,
        version: u64,
    },
    /// Acknowledgment of sync
    SyncAck {
        context_id: String,
        node_id: String,
        success: bool,
    },
    /// Node heartbeat
    Heartbeat { node: NodeInfo },
}

/// Serialized context for network transmission
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializedContext {
    pub id: String,
    pub scope: ContextScope,
    pub conversation_id: Option<String>,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub data: HashMap<String, Value>,
    pub version: u64,
    pub last_modified: chrono::DateTime<chrono::Utc>,
}

impl From<&ExecutionContext> for SerializedContext {
    fn from(ctx: &ExecutionContext) -> Self {
        Self {
            id: ctx.id.clone(),
            scope: ctx.scope.clone(),
            conversation_id: ctx.conversation_id.clone(),
            user_id: ctx.user_id.clone(),
            session_id: ctx.session_id.clone(),
            data: ctx.data.clone(),
            version: 1, // Would be tracked properly in real implementation
            last_modified: chrono::Utc::now(),
        }
    }
}

impl DistributedContext {
    /// Create new distributed context
    pub fn new(node_id: String, address: String, discovery: Arc<dyn NodeDiscovery>) -> Self {
        let local_node = NodeInfo {
            id: node_id,
            address,
            capabilities: vec!["context_sync".to_string()],
            status: NodeStatus::Healthy,
            last_heartbeat: chrono::Utc::now(),
        };

        Self {
            local_node,
            nodes: Arc::new(RwLock::new(HashMap::new())),
            replication: Arc::new(RwLock::new(ReplicationManager {
                strategy: ReplicationStrategy::NReplicas(3),
                pending: HashMap::new(),
                metrics: ReplicationMetrics::default(),
            })),
            discovery,
            _consistency: ConsistencyManager {
                _default_level: ConsistencyLevel::Eventual,
                _quorum_size: 2,
                _conflict_resolution: ConflictResolution::LastWriteWins,
            },
        }
    }

    /// Initialize distributed context
    pub async fn initialize(&self) -> Result<()> {
        // Register with discovery service
        self.discovery.register(self.local_node.clone()).await?;

        // Discover other nodes
        self.refresh_nodes().await?;

        // Start heartbeat task
        self.start_heartbeat().await;

        Ok(())
    }

    /// Refresh node list
    pub async fn refresh_nodes(&self) -> Result<()> {
        let discovered = self.discovery.discover().await?;
        let mut nodes = self.nodes.write().await;

        nodes.clear();
        for node in discovered {
            if node.id != self.local_node.id {
                nodes.insert(node.id.clone(), node);
            }
        }

        Ok(())
    }

    /// Replicate context to other nodes
    pub async fn replicate(&self, context: &ExecutionContext) -> Result<()> {
        let replication = self.replication.read().await;

        let target_nodes = match replication.strategy {
            ReplicationStrategy::None => return Ok(()),
            ReplicationStrategy::All => {
                let nodes = self.nodes.read().await;
                nodes.keys().cloned().collect()
            }
            ReplicationStrategy::NReplicas(n) => {
                let nodes = self.nodes.read().await;
                nodes.keys().take(n).cloned().collect()
            }
            ReplicationStrategy::ScopeBased => self.get_nodes_for_scope(&context.scope).await,
        };

        drop(replication);

        // Create pending replication
        let pending = PendingReplication {
            _context_id: context.id.clone(),
            target_nodes: target_nodes.clone(),
            _created_at: Instant::now(),
            _retry_count: 0,
        };

        let mut replication = self.replication.write().await;
        replication.pending.insert(context.id.clone(), pending);

        // Send to target nodes
        let serialized = SerializedContext::from(context);
        for node_id in target_nodes {
            self.send_to_node(
                &node_id,
                SyncMessage::SyncData {
                    context: serialized.clone(),
                    version: serialized.version,
                },
            )
            .await?;
        }

        Ok(())
    }

    /// Sync context from remote node
    pub async fn sync_from(&self, context_id: &str, node_id: &str) -> Result<ExecutionContext> {
        self.send_to_node(
            node_id,
            SyncMessage::SyncRequest {
                context_id: context_id.to_string(),
                requester: self.local_node.id.clone(),
            },
        )
        .await?;

        // In real implementation, would wait for response
        Err(LLMSpellError::Component {
            message: "Remote context sync not yet implemented".to_string(),
            source: None,
        })
    }

    /// Handle incoming sync message
    pub async fn handle_sync_message(&self, message: SyncMessage) -> Result<()> {
        match message {
            SyncMessage::SyncRequest {
                context_id,
                requester,
            } => {
                // Handle sync request
                tracing::info!(
                    context_id = %context_id,
                    requester = %requester,
                    "Handling sync request"
                );
            }
            SyncMessage::SyncData { context, version } => {
                // Handle incoming context data
                tracing::info!(
                    context_id = %context.id,
                    version = %version,
                    "Received context data"
                );
            }
            SyncMessage::SyncAck {
                context_id,
                node_id,
                success,
            } => {
                // Handle acknowledgment
                let mut replication = self.replication.write().await;
                let mut should_remove = false;
                let mut update_success = false;
                let mut update_failure = false;

                if let Some(pending) = replication.pending.get_mut(&context_id) {
                    pending.target_nodes.retain(|id| id != &node_id);

                    if success {
                        update_success = true;
                    } else {
                        update_failure = true;
                    }

                    if pending.target_nodes.is_empty() {
                        should_remove = true;
                    }
                }

                if update_success {
                    replication.metrics.successful_replications += 1;
                }
                if update_failure {
                    replication.metrics.failed_replications += 1;
                }

                if should_remove {
                    replication.pending.remove(&context_id);
                }
            }
            SyncMessage::Heartbeat { node } => {
                // Update node info
                let mut nodes = self.nodes.write().await;
                nodes.insert(node.id.clone(), node);
            }
        }

        Ok(())
    }

    /// Get nodes for a specific scope
    async fn get_nodes_for_scope(&self, scope: &ContextScope) -> Vec<String> {
        let nodes = self.nodes.read().await;

        // Simple implementation - in reality would use consistent hashing
        match scope {
            ContextScope::Global => nodes.keys().cloned().collect(),
            ContextScope::Session(_) => {
                // Hash session to select nodes
                nodes.keys().take(2).cloned().collect()
            }
            ContextScope::Agent(_) => {
                // Single node for agent scope
                nodes.keys().take(1).cloned().collect()
            }
            _ => Vec::new(),
        }
    }

    /// Send message to specific node
    async fn send_to_node(&self, node_id: &str, _message: SyncMessage) -> Result<()> {
        let nodes = self.nodes.read().await;

        if let Some(node) = nodes.get(node_id) {
            // In real implementation, would send over network
            tracing::info!(
                target_node = %node_id,
                address = %node.address,
                "Sending sync message"
            );
            Ok(())
        } else {
            Err(LLMSpellError::Component {
                message: format!("Node not found: {}", node_id),
                source: None,
            })
        }
    }

    /// Start heartbeat task
    async fn start_heartbeat(&self) {
        let discovery = self.discovery.clone();
        let node_id = self.local_node.id.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));

            loop {
                interval.tick().await;
                if let Err(e) = discovery.heartbeat(&node_id).await {
                    tracing::error!(error = ?e, "Heartbeat failed");
                }
            }
        });
    }

    /// Get cluster statistics
    pub async fn stats(&self) -> ClusterStats {
        let nodes = self.nodes.read().await;
        let replication = self.replication.read().await;

        let healthy_nodes = nodes
            .values()
            .filter(|n| n.status == NodeStatus::Healthy)
            .count();

        ClusterStats {
            total_nodes: nodes.len() + 1, // Include self
            healthy_nodes: healthy_nodes + 1,
            pending_replications: replication.pending.len(),
            replication_metrics: replication.metrics.clone(),
        }
    }
}

/// Cluster statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterStats {
    pub total_nodes: usize,
    pub healthy_nodes: usize,
    pub pending_replications: usize,
    pub replication_metrics: ReplicationMetrics,
}

/// Context synchronization service
pub struct ContextSync {
    /// Distributed context
    distributed: Arc<DistributedContext>,
    /// Sync message channel
    sync_rx: mpsc::Receiver<SyncMessage>,
}

impl ContextSync {
    /// Create new sync service
    pub fn new(distributed: Arc<DistributedContext>, sync_rx: mpsc::Receiver<SyncMessage>) -> Self {
        Self {
            distributed,
            sync_rx,
        }
    }

    /// Run sync service
    pub async fn run(&mut self) {
        while let Some(message) = self.sync_rx.recv().await {
            if let Err(e) = self.distributed.handle_sync_message(message).await {
                tracing::error!(error = ?e, "Failed to handle sync message");
            }
        }
    }
}

/// Mock node discovery for testing
pub struct MockNodeDiscovery {
    nodes: Arc<RwLock<Vec<NodeInfo>>>,
}

impl MockNodeDiscovery {
    pub fn new() -> Self {
        Self {
            nodes: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

impl Default for MockNodeDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl NodeDiscovery for MockNodeDiscovery {
    async fn discover(&self) -> Result<Vec<NodeInfo>> {
        let nodes = self.nodes.read().await;
        Ok(nodes.clone())
    }

    async fn register(&self, node: NodeInfo) -> Result<()> {
        let mut nodes = self.nodes.write().await;
        nodes.push(node);
        Ok(())
    }

    async fn unregister(&self, node_id: &str) -> Result<()> {
        let mut nodes = self.nodes.write().await;
        nodes.retain(|n| n.id != node_id);
        Ok(())
    }

    async fn heartbeat(&self, node_id: &str) -> Result<()> {
        let mut nodes = self.nodes.write().await;
        if let Some(node) = nodes.iter_mut().find(|n| n.id == node_id) {
            node.last_heartbeat = chrono::Utc::now();
        }
        Ok(())
    }
}

#[cfg(test)]
#[cfg_attr(test_category = "agent")]
mod tests {
    use super::*;

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_distributed_context_creation() {
        let discovery = Arc::new(MockNodeDiscovery::new());
        let dist_ctx =
            DistributedContext::new("node1".to_string(), "localhost:8080".to_string(), discovery);

        assert_eq!(dist_ctx.local_node.id, "node1");
        assert_eq!(dist_ctx.local_node.status, NodeStatus::Healthy);
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_node_discovery() {
        let discovery = Arc::new(MockNodeDiscovery::new());

        // Register some nodes
        let node1 = NodeInfo {
            id: "node1".to_string(),
            address: "localhost:8081".to_string(),
            capabilities: vec!["context_sync".to_string()],
            status: NodeStatus::Healthy,
            last_heartbeat: chrono::Utc::now(),
        };

        discovery.register(node1.clone()).await.unwrap();

        let dist_ctx = DistributedContext::new(
            "node2".to_string(),
            "localhost:8082".to_string(),
            discovery.clone(),
        );

        dist_ctx.initialize().await.unwrap();

        let nodes = dist_ctx.nodes.read().await;
        assert_eq!(nodes.len(), 1);
        assert!(nodes.contains_key("node1"));
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_replication_strategies() {
        let discovery = Arc::new(MockNodeDiscovery::new());
        let dist_ctx =
            DistributedContext::new("node1".to_string(), "localhost:8080".to_string(), discovery);

        // Test different strategies
        let mut replication = dist_ctx.replication.write().await;

        replication.strategy = ReplicationStrategy::None;
        drop(replication);

        let ctx = ExecutionContext::new();
        dist_ctx.replicate(&ctx).await.unwrap(); // Should do nothing

        let replication = dist_ctx.replication.read().await;
        assert_eq!(replication.pending.len(), 0);
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_cluster_stats() {
        let discovery = Arc::new(MockNodeDiscovery::new());

        // Register some nodes
        for i in 1..4 {
            let node = NodeInfo {
                id: format!("node{}", i),
                address: format!("localhost:808{}", i),
                capabilities: vec!["context_sync".to_string()],
                status: if i == 3 {
                    NodeStatus::Overloaded
                } else {
                    NodeStatus::Healthy
                },
                last_heartbeat: chrono::Utc::now(),
            };
            discovery.register(node).await.unwrap();
        }

        let dist_ctx = DistributedContext::new(
            "node0".to_string(),
            "localhost:8080".to_string(),
            discovery.clone(),
        );

        dist_ctx.refresh_nodes().await.unwrap();

        let stats = dist_ctx.stats().await;
        assert_eq!(stats.total_nodes, 4); // 3 registered + self
        assert_eq!(stats.healthy_nodes, 3); // 2 healthy registered + self
    }
}
