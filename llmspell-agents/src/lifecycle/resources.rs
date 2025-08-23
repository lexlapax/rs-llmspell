//! ABOUTME: Resource management for agent lifecycle with allocation and deallocation hooks
//! ABOUTME: Provides resource tracking, limits enforcement, and cleanup during agent state transitions

#![allow(clippy::significant_drop_tightening)]

use super::events::{LifecycleEvent, LifecycleEventData, LifecycleEventSystem, LifecycleEventType};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::{Mutex, RwLock};
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Resource types that can be allocated to agents
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ResourceType {
    /// Memory in bytes
    Memory,
    /// CPU cores or percentage
    Cpu,
    /// Disk space in bytes
    Disk,
    /// Network bandwidth in bytes/sec
    Network,
    /// Tool access permissions
    ToolAccess,
    /// LLM provider connections
    LlmConnection,
    /// File handles
    FileHandles,
    /// Thread pool workers
    ThreadPool,
    /// Custom resource type
    Custom(String),
}

impl ResourceType {
    /// Get resource type name
    #[must_use]
    pub fn name(&self) -> String {
        match self {
            Self::Memory => "memory".to_string(),
            Self::Cpu => "cpu".to_string(),
            Self::Disk => "disk".to_string(),
            Self::Network => "network".to_string(),
            Self::ToolAccess => "tool_access".to_string(),
            Self::LlmConnection => "llm_connection".to_string(),
            Self::FileHandles => "file_handles".to_string(),
            Self::ThreadPool => "thread_pool".to_string(),
            Self::Custom(name) => name.clone(),
        }
    }
}

/// Resource allocation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequest {
    /// Unique request ID
    pub id: String,
    /// Agent requesting the resource
    pub agent_id: String,
    /// Type of resource
    pub resource_type: ResourceType,
    /// Amount requested
    pub amount: u64,
    /// Priority level (0-10, higher is more important)
    pub priority: u8,
    /// Maximum wait time for allocation
    pub timeout: Duration,
    /// Metadata for the request
    pub metadata: HashMap<String, String>,
}

impl ResourceRequest {
    #[must_use]
    pub fn new(agent_id: String, resource_type: ResourceType, amount: u64) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            agent_id,
            resource_type,
            amount,
            priority: 5,
            timeout: Duration::from_secs(30),
            metadata: HashMap::new(),
        }
    }

    #[must_use]
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = priority.min(10);
        self
    }

    #[must_use]
    pub const fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    #[must_use]
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

/// Resource allocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceAllocation {
    /// Unique allocation ID
    pub id: String,
    /// Agent that owns the allocation
    pub agent_id: String,
    /// Type of resource
    pub resource_type: ResourceType,
    /// Amount allocated
    pub amount: u64,
    /// When allocation was created
    pub allocated_at: SystemTime,
    /// Allocation metadata
    pub metadata: HashMap<String, String>,
}

impl ResourceAllocation {
    #[must_use]
    pub fn new(agent_id: String, resource_type: ResourceType, amount: u64) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            agent_id,
            resource_type,
            amount,
            allocated_at: SystemTime::now(),
            metadata: HashMap::new(),
        }
    }

    #[must_use]
    pub fn age(&self) -> Duration {
        self.allocated_at.elapsed().unwrap_or_default()
    }
}

/// Resource limits configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    /// Maximum memory per agent (bytes)
    pub max_memory_per_agent: u64,
    /// Maximum CPU per agent (percentage, 0-100)
    pub max_cpu_per_agent: u64,
    /// Maximum disk per agent (bytes)
    pub max_disk_per_agent: u64,
    /// Maximum network bandwidth per agent (bytes/sec)
    pub max_network_per_agent: u64,
    /// Maximum tools per agent
    pub max_tools_per_agent: u64,
    /// Maximum LLM connections per agent
    pub max_llm_connections_per_agent: u64,
    /// Maximum file handles per agent
    pub max_file_handles_per_agent: u64,
    /// Maximum thread pool workers per agent
    pub max_thread_pool_per_agent: u64,
    /// Global resource limits
    pub global_limits: HashMap<ResourceType, u64>,
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_memory_per_agent: 1024 * 1024 * 1024,    // 1GB
            max_cpu_per_agent: 50,                       // 50%
            max_disk_per_agent: 10 * 1024 * 1024 * 1024, // 10GB
            max_network_per_agent: 100 * 1024 * 1024,    // 100MB/s
            max_tools_per_agent: 50,
            max_llm_connections_per_agent: 5,
            max_file_handles_per_agent: 1000,
            max_thread_pool_per_agent: 10,
            global_limits: HashMap::new(),
        }
    }
}

impl ResourceLimits {
    /// Get limit for specific resource type per agent
    #[must_use]
    pub const fn get_per_agent_limit(&self, resource_type: &ResourceType) -> Option<u64> {
        match resource_type {
            ResourceType::Memory => Some(self.max_memory_per_agent),
            ResourceType::Cpu => Some(self.max_cpu_per_agent),
            ResourceType::Disk => Some(self.max_disk_per_agent),
            ResourceType::Network => Some(self.max_network_per_agent),
            ResourceType::ToolAccess => Some(self.max_tools_per_agent),
            ResourceType::LlmConnection => Some(self.max_llm_connections_per_agent),
            ResourceType::FileHandles => Some(self.max_file_handles_per_agent),
            ResourceType::ThreadPool => Some(self.max_thread_pool_per_agent),
            ResourceType::Custom(_) => None,
        }
    }

    /// Get global limit for resource type
    #[must_use]
    pub fn get_global_limit(&self, resource_type: &ResourceType) -> Option<u64> {
        self.global_limits.get(resource_type).copied()
    }
}

/// Resource allocation hook trait
#[async_trait]
pub trait ResourceAllocationHook: Send + Sync {
    /// Called before resource allocation
    async fn before_allocate(&self, request: &ResourceRequest) -> Result<()>;

    /// Called after successful resource allocation
    async fn after_allocate(&self, allocation: &ResourceAllocation) -> Result<()>;

    /// Called before resource deallocation
    async fn before_deallocate(&self, allocation: &ResourceAllocation) -> Result<()>;

    /// Called after resource deallocation
    async fn after_deallocate(&self, allocation: &ResourceAllocation) -> Result<()>;
}

/// Resource manager for agent lifecycle
pub struct ResourceManager {
    /// Current allocations by agent
    allocations: Arc<RwLock<HashMap<String, Vec<ResourceAllocation>>>>,
    /// Resource limits
    limits: ResourceLimits,
    /// Event system for notifications
    event_system: Arc<LifecycleEventSystem>,
    /// Resource allocation hooks
    hooks: Vec<Arc<dyn ResourceAllocationHook>>,
    /// Resource usage statistics
    usage_stats: Arc<Mutex<ResourceUsageStats>>,
}

/// Resource usage statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceUsageStats {
    pub total_allocations: u64,
    pub current_allocations: u64,
    pub total_deallocations: u64,
    pub failed_allocations: u64,
    pub allocations_by_type: HashMap<ResourceType, u64>,
    pub current_usage_by_type: HashMap<ResourceType, u64>,
    pub peak_usage_by_type: HashMap<ResourceType, u64>,
}

impl ResourceManager {
    /// Create new resource manager
    #[must_use]
    pub fn new(limits: ResourceLimits, event_system: Arc<LifecycleEventSystem>) -> Self {
        Self {
            allocations: Arc::new(RwLock::new(HashMap::new())),
            limits,
            event_system,
            hooks: Vec::new(),
            usage_stats: Arc::new(Mutex::new(ResourceUsageStats::default())),
        }
    }

    /// Add resource allocation hook
    pub fn add_hook(&mut self, hook: Arc<dyn ResourceAllocationHook>) {
        self.hooks.push(hook);
    }

    /// Allocate resources for agent
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Validation fails
    /// - Resource allocation fails
    /// - Insufficient resources available
    /// - Quota exceeded
    #[allow(clippy::cognitive_complexity)]
    pub async fn allocate(&self, request: ResourceRequest) -> Result<ResourceAllocation> {
        debug!(
            "Allocating {} {} for agent {}",
            request.amount,
            request.resource_type.name(),
            request.agent_id
        );

        // Execute before allocation hooks
        for hook in &self.hooks {
            hook.before_allocate(&request).await?;
        }

        // Check resource limits
        self.check_limits(&request).await?;

        // Create allocation
        let allocation = ResourceAllocation::new(
            request.agent_id.clone(),
            request.resource_type.clone(),
            request.amount,
        );

        // Record allocation
        {
            let mut allocations = self.allocations.write().await;
            allocations
                .entry(request.agent_id.clone())
                .or_insert_with(Vec::new)
                .push(allocation.clone());
        }

        // Update statistics
        {
            let mut stats = self.usage_stats.lock().await;
            stats.total_allocations += 1;
            stats.current_allocations += 1;
            *stats
                .allocations_by_type
                .entry(request.resource_type.clone())
                .or_insert(0) += 1;
            *stats
                .current_usage_by_type
                .entry(request.resource_type.clone())
                .or_insert(0) += request.amount;

            // Update peak usage
            let current_usage = stats.current_usage_by_type[&request.resource_type];
            let peak = stats
                .peak_usage_by_type
                .entry(request.resource_type.clone())
                .or_insert(0);
            if current_usage > *peak {
                *peak = current_usage;
            }
        }

        // Execute after allocation hooks
        for hook in &self.hooks {
            hook.after_allocate(&allocation).await?;
        }

        // Emit allocation event
        let event = LifecycleEvent::new(
            LifecycleEventType::ResourceAllocated,
            request.agent_id.clone(),
            LifecycleEventData::Resource {
                resource_type: request.resource_type.name(),
                resource_id: allocation.id.clone(),
                amount: Some(request.amount),
                status: "allocated".to_string(),
            },
            "resource_manager".to_string(),
        );

        if let Err(e) = self.event_system.emit(event).await {
            warn!("Failed to emit resource allocation event: {}", e);
        }

        info!(
            "Successfully allocated {} {} for agent {}",
            request.amount,
            request.resource_type.name(),
            request.agent_id
        );

        Ok(allocation)
    }

    /// Deallocate specific resource
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Allocation not found
    /// - Deallocation hooks fail
    #[allow(clippy::cognitive_complexity)]
    pub async fn deallocate(&self, allocation_id: &str) -> Result<()> {
        let allocation = self.find_allocation(allocation_id).await?;

        debug!(
            "Deallocating {} {} from agent {}",
            allocation.amount,
            allocation.resource_type.name(),
            allocation.agent_id
        );

        // Execute before deallocation hooks
        for hook in &self.hooks {
            hook.before_deallocate(&allocation).await?;
        }

        // Remove allocation
        {
            let mut allocations = self.allocations.write().await;
            if let Some(agent_allocations) = allocations.get_mut(&allocation.agent_id) {
                agent_allocations.retain(|a| a.id != allocation_id);
                if agent_allocations.is_empty() {
                    allocations.remove(&allocation.agent_id);
                }
            }
        }

        // Update statistics
        {
            let mut stats = self.usage_stats.lock().await;
            stats.total_deallocations += 1;
            stats.current_allocations = stats.current_allocations.saturating_sub(1);
            *stats
                .current_usage_by_type
                .entry(allocation.resource_type.clone())
                .or_insert(0) = stats.current_usage_by_type[&allocation.resource_type]
                .saturating_sub(allocation.amount);
        }

        // Execute after deallocation hooks
        for hook in &self.hooks {
            hook.after_deallocate(&allocation).await?;
        }

        // Emit deallocation event
        let event = LifecycleEvent::new(
            LifecycleEventType::ResourceDeallocated,
            allocation.agent_id.clone(),
            LifecycleEventData::Resource {
                resource_type: allocation.resource_type.name(),
                resource_id: allocation.id.clone(),
                amount: Some(allocation.amount),
                status: "deallocated".to_string(),
            },
            "resource_manager".to_string(),
        );

        if let Err(e) = self.event_system.emit(event).await {
            warn!("Failed to emit resource deallocation event: {}", e);
        }

        info!(
            "Successfully deallocated {} {} from agent {}",
            allocation.amount,
            allocation.resource_type.name(),
            allocation.agent_id
        );

        Ok(())
    }

    /// Deallocate all resources for agent
    ///
    /// # Errors
    ///
    /// Returns an error if any deallocation fails
    pub async fn deallocate_all(&self, agent_id: &str) -> Result<()> {
        debug!("Deallocating all resources for agent {}", agent_id);

        let allocations = {
            let allocations_guard = self.allocations.read().await;
            allocations_guard.get(agent_id).cloned().unwrap_or_default()
        };

        for allocation in allocations {
            self.deallocate(&allocation.id).await?;
        }

        info!(
            "Successfully deallocated all resources for agent {}",
            agent_id
        );
        Ok(())
    }

    /// Check resource limits before allocation
    async fn check_limits(&self, request: &ResourceRequest) -> Result<()> {
        // Check per-agent limits
        if let Some(per_agent_limit) = self.limits.get_per_agent_limit(&request.resource_type) {
            let current_usage = self
                .get_agent_usage(&request.agent_id, &request.resource_type)
                .await;
            if current_usage + request.amount > per_agent_limit {
                return Err(anyhow!(
                    "Agent {} exceeds per-agent limit for {}: {} + {} > {}",
                    request.agent_id,
                    request.resource_type.name(),
                    current_usage,
                    request.amount,
                    per_agent_limit
                ));
            }
        }

        // Check global limits
        if let Some(global_limit) = self.limits.get_global_limit(&request.resource_type) {
            let current_global_usage = self.get_global_usage(&request.resource_type).await;
            if current_global_usage + request.amount > global_limit {
                return Err(anyhow!(
                    "Global limit exceeded for {}: {} + {} > {}",
                    request.resource_type.name(),
                    current_global_usage,
                    request.amount,
                    global_limit
                ));
            }
        }

        Ok(())
    }

    /// Get current resource usage for agent
    async fn get_agent_usage(&self, agent_id: &str, resource_type: &ResourceType) -> u64 {
        let allocations = self.allocations.read().await;
        allocations.get(agent_id).map_or(0, |agent_allocations| {
            agent_allocations
                .iter()
                .filter(|a| a.resource_type == *resource_type)
                .map(|a| a.amount)
                .sum()
        })
    }

    /// Get global resource usage
    async fn get_global_usage(&self, resource_type: &ResourceType) -> u64 {
        let stats = self.usage_stats.lock().await;
        stats
            .current_usage_by_type
            .get(resource_type)
            .copied()
            .unwrap_or(0)
    }

    /// Find allocation by ID
    async fn find_allocation(&self, allocation_id: &str) -> Result<ResourceAllocation> {
        let allocations = self.allocations.read().await;
        for agent_allocations in allocations.values() {
            if let Some(allocation) = agent_allocations.iter().find(|a| a.id == allocation_id) {
                return Ok(allocation.clone());
            }
        }
        Err(anyhow!("Allocation not found: {}", allocation_id))
    }

    /// Get all allocations for agent
    pub async fn get_agent_allocations(&self, agent_id: &str) -> Vec<ResourceAllocation> {
        let allocations = self.allocations.read().await;
        allocations.get(agent_id).cloned().unwrap_or_default()
    }

    /// Get resource usage statistics
    pub async fn get_usage_stats(&self) -> ResourceUsageStats {
        let stats = self.usage_stats.lock().await;
        stats.clone()
    }

    /// Get total number of allocations
    pub async fn get_allocation_count(&self) -> usize {
        let allocations = self.allocations.read().await;
        allocations.values().map(std::vec::Vec::len).sum()
    }
}

/// Default logging resource hook
pub struct LoggingResourceHook;

#[async_trait]
impl ResourceAllocationHook for LoggingResourceHook {
    async fn before_allocate(&self, request: &ResourceRequest) -> Result<()> {
        debug!(
            "About to allocate {} {} for agent {}",
            request.amount,
            request.resource_type.name(),
            request.agent_id
        );
        Ok(())
    }

    async fn after_allocate(&self, allocation: &ResourceAllocation) -> Result<()> {
        info!(
            "Allocated {} {} for agent {} (ID: {})",
            allocation.amount,
            allocation.resource_type.name(),
            allocation.agent_id,
            allocation.id
        );
        Ok(())
    }

    async fn before_deallocate(&self, allocation: &ResourceAllocation) -> Result<()> {
        debug!(
            "About to deallocate {} {} from agent {} (ID: {})",
            allocation.amount,
            allocation.resource_type.name(),
            allocation.agent_id,
            allocation.id
        );
        Ok(())
    }

    async fn after_deallocate(&self, allocation: &ResourceAllocation) -> Result<()> {
        info!(
            "Deallocated {} {} from agent {} (ID: {})",
            allocation.amount,
            allocation.resource_type.name(),
            allocation.agent_id,
            allocation.id
        );
        Ok(())
    }
}

/// Security resource hook that enforces security policies
pub struct SecurityResourceHook {
    max_memory_per_untrusted_agent: u64,
    trusted_agents: Vec<String>,
}

impl Default for SecurityResourceHook {
    fn default() -> Self {
        Self::new()
    }
}

impl SecurityResourceHook {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            max_memory_per_untrusted_agent: 512 * 1024 * 1024, // 512MB
            trusted_agents: Vec::new(),
        }
    }

    #[must_use]
    pub fn with_trusted_agents(mut self, agents: Vec<String>) -> Self {
        self.trusted_agents = agents;
        self
    }
}

#[async_trait]
impl ResourceAllocationHook for SecurityResourceHook {
    async fn before_allocate(&self, request: &ResourceRequest) -> Result<()> {
        // Enforce stricter limits for untrusted agents
        if !self.trusted_agents.contains(&request.agent_id)
            && request.resource_type == ResourceType::Memory
            && request.amount > self.max_memory_per_untrusted_agent
        {
            return Err(anyhow!(
                "Untrusted agent {} requested {} bytes memory, maximum allowed is {}",
                request.agent_id,
                request.amount,
                self.max_memory_per_untrusted_agent
            ));
        }
        Ok(())
    }

    async fn after_allocate(&self, _allocation: &ResourceAllocation) -> Result<()> {
        Ok(())
    }

    async fn before_deallocate(&self, _allocation: &ResourceAllocation) -> Result<()> {
        Ok(())
    }

    async fn after_deallocate(&self, _allocation: &ResourceAllocation) -> Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lifecycle::events::EventSystemConfig;
    #[tokio::test]
    async fn test_resource_allocation_basic() {
        let event_system = Arc::new(LifecycleEventSystem::new(EventSystemConfig::default()));
        let mut manager = ResourceManager::new(ResourceLimits::default(), event_system);
        manager.add_hook(Arc::new(LoggingResourceHook));

        let request = ResourceRequest::new(
            "test-agent".to_string(),
            ResourceType::Memory,
            1024 * 1024, // 1MB
        );

        let allocation = manager.allocate(request).await.unwrap();
        assert_eq!(allocation.agent_id, "test-agent");
        assert_eq!(allocation.resource_type, ResourceType::Memory);
        assert_eq!(allocation.amount, 1024 * 1024);

        // Check allocation exists
        let allocations = manager.get_agent_allocations("test-agent").await;
        assert_eq!(allocations.len(), 1);

        // Deallocate
        manager.deallocate(&allocation.id).await.unwrap();

        // Check allocation removed
        let allocations = manager.get_agent_allocations("test-agent").await;
        assert_eq!(allocations.len(), 0);
    }
    #[tokio::test]
    async fn test_resource_limits() {
        let event_system = Arc::new(LifecycleEventSystem::new(EventSystemConfig::default()));
        let limits = ResourceLimits {
            max_memory_per_agent: 1024, // 1KB limit
            ..Default::default()
        };
        let manager = ResourceManager::new(limits, event_system);

        // First allocation should succeed
        let request1 = ResourceRequest::new(
            "test-agent".to_string(),
            ResourceType::Memory,
            512, // 512 bytes
        );
        manager.allocate(request1).await.unwrap();

        // Second allocation should also succeed (total 1024 bytes)
        let request2 = ResourceRequest::new(
            "test-agent".to_string(),
            ResourceType::Memory,
            512, // 512 bytes
        );
        manager.allocate(request2).await.unwrap();

        // Third allocation should fail (would exceed limit)
        let request3 = ResourceRequest::new(
            "test-agent".to_string(),
            ResourceType::Memory,
            1, // 1 byte
        );
        let result = manager.allocate(request3).await;
        assert!(result.is_err());
    }
    #[tokio::test]
    async fn test_deallocate_all() {
        let event_system = Arc::new(LifecycleEventSystem::new(EventSystemConfig::default()));
        let manager = ResourceManager::new(ResourceLimits::default(), event_system);

        // Allocate multiple resources
        for i in 0..3 {
            let request = ResourceRequest::new(
                "test-agent".to_string(),
                ResourceType::Memory,
                1024 * (i + 1),
            );
            manager.allocate(request).await.unwrap();
        }

        // Check allocations exist
        let allocations = manager.get_agent_allocations("test-agent").await;
        assert_eq!(allocations.len(), 3);

        // Deallocate all
        manager.deallocate_all("test-agent").await.unwrap();

        // Check all allocations removed
        let allocations = manager.get_agent_allocations("test-agent").await;
        assert_eq!(allocations.len(), 0);
    }
    #[tokio::test]
    async fn test_usage_statistics() {
        let event_system = Arc::new(LifecycleEventSystem::new(EventSystemConfig::default()));
        let manager = ResourceManager::new(ResourceLimits::default(), event_system);

        let request = ResourceRequest::new("test-agent".to_string(), ResourceType::Memory, 1024);

        let allocation = manager.allocate(request).await.unwrap();
        let stats = manager.get_usage_stats().await;

        assert_eq!(stats.total_allocations, 1);
        assert_eq!(stats.current_allocations, 1);
        assert_eq!(
            stats.allocations_by_type.get(&ResourceType::Memory),
            Some(&1)
        );
        assert_eq!(
            stats.current_usage_by_type.get(&ResourceType::Memory),
            Some(&1024)
        );

        manager.deallocate(&allocation.id).await.unwrap();
        let stats = manager.get_usage_stats().await;

        assert_eq!(stats.total_deallocations, 1);
        assert_eq!(stats.current_allocations, 0);
        assert_eq!(
            stats.current_usage_by_type.get(&ResourceType::Memory),
            Some(&0)
        );
    }
    #[tokio::test]
    async fn test_security_hook() {
        let event_system = Arc::new(LifecycleEventSystem::new(EventSystemConfig::default()));
        let mut manager = ResourceManager::new(ResourceLimits::default(), event_system);

        let security_hook =
            SecurityResourceHook::new().with_trusted_agents(vec!["trusted-agent".to_string()]);
        manager.add_hook(Arc::new(security_hook));

        // Trusted agent should be able to allocate large memory
        let trusted_request = ResourceRequest::new(
            "trusted-agent".to_string(),
            ResourceType::Memory,
            1024 * 1024 * 1024, // 1GB
        );
        let result = manager.allocate(trusted_request).await;
        assert!(result.is_ok());

        // Untrusted agent should be limited
        let untrusted_request = ResourceRequest::new(
            "untrusted-agent".to_string(),
            ResourceType::Memory,
            1024 * 1024 * 1024, // 1GB
        );
        let result = manager.allocate(untrusted_request).await;
        assert!(result.is_err());
    }
    #[tokio::test]
    async fn test_resource_types() {
        assert_eq!(ResourceType::Memory.name(), "memory");
        assert_eq!(ResourceType::Cpu.name(), "cpu");
        assert_eq!(ResourceType::Custom("test".to_string()).name(), "test");
    }
    #[tokio::test]
    async fn test_resource_request_builder() {
        let request = ResourceRequest::new("test-agent".to_string(), ResourceType::Memory, 1024)
            .with_priority(8)
            .with_timeout(Duration::from_secs(60))
            .with_metadata("purpose", "test");

        assert_eq!(request.priority, 8);
        assert_eq!(request.timeout, Duration::from_secs(60));
        assert_eq!(request.metadata.get("purpose"), Some(&"test".to_string()));
    }
}
