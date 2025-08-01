// ABOUTME: DistributedHookContext implementation for network-aware hook execution
// ABOUTME: Extends HookContext with remote agent tracking, propagation control, and security

use crate::context::HookContext;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fmt;
use std::hash::{Hash, Hasher};
use std::net::IpAddr;
use std::time::{Duration, SystemTime};
use uuid::Uuid;

/// Identifies a remote agent in the distributed system
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteAgentId {
    /// Node identifier (e.g., hostname, cluster node ID)
    pub node_id: String,

    /// Agent identifier within the node
    pub agent_id: String,

    /// Optional network address for direct communication
    pub network_address: Option<String>,

    /// Agent capabilities/version for compatibility
    pub capabilities: HashMap<String, String>,
}

impl RemoteAgentId {
    /// Create a new remote agent identifier
    pub fn new(node_id: impl Into<String>, agent_id: impl Into<String>) -> Self {
        Self {
            node_id: node_id.into(),
            agent_id: agent_id.into(),
            network_address: None,
            capabilities: HashMap::new(),
        }
    }

    /// Add a network address
    pub fn with_address(mut self, address: impl Into<String>) -> Self {
        self.network_address = Some(address.into());
        self
    }

    /// Add a capability
    pub fn with_capability(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.capabilities.insert(key.into(), value.into());
        self
    }

    /// Get a unique identifier string
    pub fn unique_id(&self) -> String {
        format!("{}/{}", self.node_id, self.agent_id)
    }
}

impl fmt::Display for RemoteAgentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.unique_id())
    }
}

impl Hash for RemoteAgentId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.node_id.hash(state);
        self.agent_id.hash(state);
        // Only hash the key identifiers, not mutable fields like capabilities
    }
}

/// Flags controlling hook propagation across the network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropagationFlags {
    /// Whether this hook should be broadcast to all agents
    pub broadcast: bool,

    /// Specific agents to propagate to (if not broadcasting)
    pub target_agents: HashSet<RemoteAgentId>,

    /// Maximum number of hops this hook can propagate
    pub max_hops: u32,

    /// Current hop count
    pub current_hops: u32,

    /// Whether to wait for remote execution results
    pub await_remote: bool,

    /// Timeout for remote execution
    pub remote_timeout: Duration,

    /// Whether to continue on remote failure
    pub continue_on_failure: bool,

    /// Priority for network transmission
    pub network_priority: NetworkPriority,
}

/// Network transmission priority
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkPriority {
    Low,
    Normal,
    High,
    Critical,
}

impl Default for PropagationFlags {
    fn default() -> Self {
        Self {
            broadcast: false,
            target_agents: HashSet::new(),
            max_hops: 3,
            current_hops: 0,
            await_remote: false,
            remote_timeout: Duration::from_secs(30),
            continue_on_failure: true,
            network_priority: NetworkPriority::Normal,
        }
    }
}

impl PropagationFlags {
    /// Enable broadcasting
    pub fn with_broadcast(mut self, broadcast: bool) -> Self {
        self.broadcast = broadcast;
        self
    }

    /// Add a target agent
    pub fn with_target(mut self, agent: RemoteAgentId) -> Self {
        self.target_agents.insert(agent);
        self
    }

    /// Set maximum hops
    pub fn with_max_hops(mut self, hops: u32) -> Self {
        self.max_hops = hops;
        self
    }

    /// Set whether to await remote execution
    pub fn with_await_remote(mut self, await_remote: bool) -> Self {
        self.await_remote = await_remote;
        self
    }

    /// Set remote timeout
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.remote_timeout = timeout;
        self
    }

    /// Check if propagation is allowed
    pub fn can_propagate(&self) -> bool {
        self.current_hops < self.max_hops
    }

    /// Increment hop count for propagation
    pub fn increment_hops(&mut self) {
        self.current_hops += 1;
    }
}

/// Security context for distributed hook execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    /// Authentication token or credentials
    pub auth_token: Option<String>,

    /// Signature for message integrity
    pub signature: Option<Vec<u8>>,

    /// Trusted agent whitelist
    pub trusted_agents: HashSet<RemoteAgentId>,

    /// Allowed hook points for remote execution
    pub allowed_hook_points: HashSet<String>,

    /// IP address restrictions
    pub allowed_ips: Vec<IpAddr>,

    /// Timestamp for replay attack prevention
    pub timestamp: SystemTime,

    /// Nonce for uniqueness
    pub nonce: Uuid,
}

impl Default for SecurityContext {
    fn default() -> Self {
        Self {
            auth_token: None,
            signature: None,
            trusted_agents: HashSet::new(),
            allowed_hook_points: HashSet::new(),
            allowed_ips: Vec::new(),
            timestamp: SystemTime::now(),
            nonce: Uuid::new_v4(),
        }
    }
}

impl SecurityContext {
    /// Create a new security context with a nonce
    pub fn new() -> Self {
        Self::default()
    }

    /// Set authentication token
    pub fn with_auth_token(mut self, token: impl Into<String>) -> Self {
        self.auth_token = Some(token.into());
        self
    }

    /// Add a trusted agent
    pub fn with_trusted_agent(mut self, agent: RemoteAgentId) -> Self {
        self.trusted_agents.insert(agent);
        self
    }

    /// Add an allowed hook point
    pub fn with_allowed_hook_point(mut self, hook_point: impl Into<String>) -> Self {
        self.allowed_hook_points.insert(hook_point.into());
        self
    }

    /// Check if an agent is trusted
    pub fn is_agent_trusted(&self, agent: &RemoteAgentId) -> bool {
        self.trusted_agents.is_empty() || self.trusted_agents.contains(agent)
    }

    /// Check if a hook point is allowed
    pub fn is_hook_point_allowed(&self, hook_point: &str) -> bool {
        self.allowed_hook_points.is_empty() || self.allowed_hook_points.contains(hook_point)
    }

    /// Validate timestamp to prevent replay attacks
    pub fn is_timestamp_valid(&self, max_age: Duration) -> bool {
        if let Ok(elapsed) = self.timestamp.elapsed() {
            elapsed <= max_age
        } else {
            false
        }
    }
}

/// Extended hook context for distributed execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributedHookContext {
    /// Base hook context
    pub base_context: HookContext,

    /// Source agent that initiated the hook
    pub source_agent: Option<RemoteAgentId>,

    /// Target agents for propagation
    pub target_agents: Vec<RemoteAgentId>,

    /// Propagation control flags
    pub propagation_flags: PropagationFlags,

    /// Security context
    pub security_context: SecurityContext,

    /// Network correlation ID for tracing across agents
    pub network_correlation_id: Uuid,

    /// Execution results from remote agents
    pub remote_results: HashMap<RemoteAgentId, RemoteExecutionResult>,

    /// Additional distributed metadata
    pub distributed_metadata: HashMap<String, String>,
}

/// Result from remote hook execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteExecutionResult {
    /// Agent that executed the hook
    pub agent_id: RemoteAgentId,

    /// Whether execution succeeded
    pub success: bool,

    /// Execution result data
    pub result_data: Option<serde_json::Value>,

    /// Error message if failed
    pub error: Option<String>,

    /// Execution duration
    pub duration: Duration,

    /// Timestamp of execution
    pub timestamp: SystemTime,
}

impl DistributedHookContext {
    /// Create from a local hook context
    pub fn from_local(base_context: HookContext) -> Self {
        Self {
            base_context,
            source_agent: None,
            target_agents: Vec::new(),
            propagation_flags: PropagationFlags::default(),
            security_context: SecurityContext::default(),
            network_correlation_id: Uuid::new_v4(),
            remote_results: HashMap::new(),
            distributed_metadata: HashMap::new(),
        }
    }

    /// Set the source agent
    pub fn with_source_agent(mut self, agent: RemoteAgentId) -> Self {
        self.source_agent = Some(agent);
        self
    }

    /// Add a target agent
    pub fn with_target_agent(mut self, agent: RemoteAgentId) -> Self {
        self.target_agents.push(agent);
        self
    }

    /// Set propagation flags
    pub fn with_propagation_flags(mut self, flags: PropagationFlags) -> Self {
        self.propagation_flags = flags;
        self
    }

    /// Set security context
    pub fn with_security_context(mut self, context: SecurityContext) -> Self {
        self.security_context = context;
        self
    }

    /// Add distributed metadata
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.distributed_metadata.insert(key.into(), value.into());
        self
    }

    /// Check if this is a remote execution
    pub fn is_remote_execution(&self) -> bool {
        self.source_agent.is_some()
    }

    /// Check if propagation is enabled
    pub fn should_propagate(&self) -> bool {
        self.propagation_flags.can_propagate()
            && (self.propagation_flags.broadcast || !self.target_agents.is_empty())
    }

    /// Add a remote execution result
    pub fn add_remote_result(&mut self, result: RemoteExecutionResult) {
        self.remote_results.insert(result.agent_id.clone(), result);
    }

    /// Get all successful remote results
    pub fn successful_remote_results(&self) -> Vec<&RemoteExecutionResult> {
        self.remote_results.values().filter(|r| r.success).collect()
    }

    /// Check if all remote executions succeeded
    pub fn all_remote_succeeded(&self) -> bool {
        !self.remote_results.is_empty() && self.remote_results.values().all(|r| r.success)
    }

    /// Create a propagated context for a target agent
    pub fn create_propagated_context(&self) -> Self {
        let mut propagated = self.clone();
        propagated.propagation_flags.increment_hops();
        propagated.remote_results.clear();
        propagated
    }
}

/// Builder for DistributedHookContext
pub struct DistributedHookContextBuilder {
    context: DistributedHookContext,
}

impl DistributedHookContextBuilder {
    /// Create a new builder from a base context
    pub fn new(base_context: HookContext) -> Self {
        Self {
            context: DistributedHookContext::from_local(base_context),
        }
    }

    /// Set source agent
    pub fn source_agent(mut self, agent: RemoteAgentId) -> Self {
        self.context.source_agent = Some(agent);
        self
    }

    /// Add target agent
    pub fn target_agent(mut self, agent: RemoteAgentId) -> Self {
        self.context.target_agents.push(agent);
        self
    }

    /// Set propagation flags
    pub fn propagation_flags(mut self, flags: PropagationFlags) -> Self {
        self.context.propagation_flags = flags;
        self
    }

    /// Set security context
    pub fn security_context(mut self, context: SecurityContext) -> Self {
        self.context.security_context = context;
        self
    }

    /// Add metadata
    pub fn metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context
            .distributed_metadata
            .insert(key.into(), value.into());
        self
    }

    /// Set network correlation ID
    pub fn network_correlation_id(mut self, id: Uuid) -> Self {
        self.context.network_correlation_id = id;
        self
    }

    /// Build the context
    pub fn build(self) -> DistributedHookContext {
        self.context
    }
}

#[cfg(test)]
#[cfg_attr(test_category = "hook")]
mod tests {
    use super::*;
    use crate::types::{ComponentId, ComponentType, HookPoint};

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_remote_agent_id() {
        let agent = RemoteAgentId::new("node-1", "agent-1")
            .with_address("192.168.1.10:8080")
            .with_capability("version", "1.0.0");

        assert_eq!(agent.unique_id(), "node-1/agent-1");
        assert_eq!(agent.network_address, Some("192.168.1.10:8080".to_string()));
        assert_eq!(
            agent.capabilities.get("version"),
            Some(&"1.0.0".to_string())
        );
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_propagation_flags() {
        let mut flags = PropagationFlags::default()
            .with_broadcast(true)
            .with_max_hops(5)
            .with_await_remote(true);

        assert!(flags.broadcast);
        assert_eq!(flags.max_hops, 5);
        assert!(flags.await_remote);
        assert!(flags.can_propagate());

        // Test hop counting
        for _ in 0..5 {
            assert!(flags.can_propagate());
            flags.increment_hops();
        }
        assert!(!flags.can_propagate());
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_security_context() {
        let agent1 = RemoteAgentId::new("trusted-node", "trusted-agent");
        let agent2 = RemoteAgentId::new("untrusted-node", "untrusted-agent");

        let security = SecurityContext::new()
            .with_auth_token("secret-token")
            .with_trusted_agent(agent1.clone())
            .with_allowed_hook_point("BeforeAgentExecution");

        assert!(security.is_agent_trusted(&agent1));
        assert!(!security.is_agent_trusted(&agent2));
        assert!(security.is_hook_point_allowed("BeforeAgentExecution"));
        assert!(!security.is_hook_point_allowed("SystemShutdown"));

        // Test timestamp validation
        assert!(security.is_timestamp_valid(Duration::from_secs(300)));
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_distributed_context_creation() {
        let base_context = HookContext::new(
            HookPoint::BeforeAgentExecution,
            ComponentId::new(ComponentType::Agent, "test-agent".to_string()),
        );

        let source = RemoteAgentId::new("source-node", "source-agent");
        let target = RemoteAgentId::new("target-node", "target-agent");

        let distributed = DistributedHookContext::from_local(base_context)
            .with_source_agent(source.clone())
            .with_target_agent(target)
            .with_metadata("test", "value");

        assert!(distributed.is_remote_execution());
        assert_eq!(distributed.source_agent, Some(source));
        assert_eq!(distributed.target_agents.len(), 1);
        assert_eq!(
            distributed.distributed_metadata.get("test"),
            Some(&"value".to_string())
        );
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_remote_results() {
        let base_context = HookContext::new(
            HookPoint::AfterToolExecution,
            ComponentId::new(ComponentType::Tool, "calculator".to_string()),
        );

        let mut distributed = DistributedHookContext::from_local(base_context);

        let agent1 = RemoteAgentId::new("node-1", "agent-1");
        let agent2 = RemoteAgentId::new("node-2", "agent-2");

        distributed.add_remote_result(RemoteExecutionResult {
            agent_id: agent1.clone(),
            success: true,
            result_data: Some(serde_json::json!({"value": 42})),
            error: None,
            duration: Duration::from_millis(100),
            timestamp: SystemTime::now(),
        });

        distributed.add_remote_result(RemoteExecutionResult {
            agent_id: agent2.clone(),
            success: false,
            result_data: None,
            error: Some("Connection timeout".to_string()),
            duration: Duration::from_secs(30),
            timestamp: SystemTime::now(),
        });

        assert_eq!(distributed.remote_results.len(), 2);
        assert_eq!(distributed.successful_remote_results().len(), 1);
        assert!(!distributed.all_remote_succeeded());
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_propagation() {
        let base_context = HookContext::new(
            HookPoint::BeforeWorkflowStart,
            ComponentId::new(ComponentType::Workflow, "pipeline".to_string()),
        );

        let distributed = DistributedHookContext::from_local(base_context).with_propagation_flags(
            PropagationFlags::default()
                .with_broadcast(true)
                .with_max_hops(2),
        );

        assert!(distributed.should_propagate());

        // Create propagated context
        let propagated = distributed.create_propagated_context();
        assert_eq!(propagated.propagation_flags.current_hops, 1);
        assert!(propagated.should_propagate());

        // Propagate again
        let propagated2 = propagated.create_propagated_context();
        assert_eq!(propagated2.propagation_flags.current_hops, 2);
        assert!(!propagated2.should_propagate()); // Max hops reached
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_context_builder() {
        let base_context = HookContext::new(
            HookPoint::SystemStartup,
            ComponentId::new(ComponentType::System, "main".to_string()),
        );

        let source = RemoteAgentId::new("local", "main");
        let target = RemoteAgentId::new("remote", "worker");
        let correlation_id = Uuid::new_v4();

        let distributed = DistributedHookContextBuilder::new(base_context)
            .source_agent(source.clone())
            .target_agent(target.clone())
            .network_correlation_id(correlation_id)
            .metadata("cluster", "production")
            .propagation_flags(PropagationFlags::default().with_await_remote(true))
            .security_context(SecurityContext::new().with_auth_token("token"))
            .build();

        assert_eq!(distributed.source_agent, Some(source));
        assert_eq!(distributed.target_agents.len(), 1);
        assert_eq!(distributed.network_correlation_id, correlation_id);
        assert!(distributed.propagation_flags.await_remote);
        assert!(distributed.security_context.auth_token.is_some());
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_serialization() {
        let base_context = HookContext::new(
            HookPoint::BeforeAgentInit,
            ComponentId::new(ComponentType::Agent, "test".to_string()),
        );

        let distributed = DistributedHookContext::from_local(base_context)
            .with_source_agent(RemoteAgentId::new("node", "agent"))
            .with_propagation_flags(PropagationFlags::default().with_broadcast(true))
            .with_security_context(SecurityContext::new().with_auth_token("test"));

        // Test serialization round-trip
        let serialized = serde_json::to_string(&distributed).unwrap();
        let deserialized: DistributedHookContext = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.source_agent, distributed.source_agent);
        assert!(deserialized.propagation_flags.broadcast);
        assert_eq!(
            deserialized.security_context.auth_token,
            Some("test".to_string())
        );
    }
}
