//! ABOUTME: Enhanced metadata management for agents
//! ABOUTME: Provides rich metadata tracking, versioning, and relationships

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Extended agent metadata with versioning and relationships
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtendedAgentMetadata {
    /// Core metadata
    #[serde(flatten)]
    pub core: super::AgentMetadata,

    /// Version information
    pub version: VersionInfo,

    /// Dependencies on other agents
    pub dependencies: Vec<AgentDependency>,

    /// Capabilities and features
    pub capabilities: Vec<AgentCapability>,

    /// Resource requirements
    pub resource_requirements: ResourceRequirements,

    /// Deployment information
    pub deployment: DeploymentInfo,

    /// Health status
    pub health: HealthStatus,
}

/// Agent version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    /// Semantic version (e.g., "1.2.3")
    pub version: String,

    /// Git commit hash if available
    pub commit_hash: Option<String>,

    /// Build timestamp
    pub build_timestamp: Option<chrono::DateTime<chrono::Utc>>,

    /// Change log entry
    pub changelog: Option<String>,
}

/// Agent dependency specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentDependency {
    /// Agent ID or type
    pub agent_id: String,

    /// Required version range
    pub version_range: Option<String>,

    /// Dependency type
    pub dependency_type: DependencyType,

    /// Whether dependency is optional
    pub optional: bool,
}

/// Type of agent dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DependencyType {
    /// Hard requirement
    Required,

    /// Recommended but not required
    Recommended,

    /// Peer dependency
    Peer,

    /// Development/testing only
    Development,
}

/// Agent capability declaration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentCapability {
    /// Capability name
    pub name: String,

    /// Capability type
    pub capability_type: CapabilityType,

    /// Configuration for the capability
    pub config: HashMap<String, serde_json::Value>,

    /// Whether capability is enabled
    pub enabled: bool,
}

/// Type of agent capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CapabilityType {
    /// Tool usage capability
    ToolUsage,

    /// Workflow orchestration
    WorkflowOrchestration,

    /// Multi-agent communication
    MultiAgentCommunication,

    /// Learning/adaptation
    Learning,

    /// Custom capability
    Custom(String),
}

/// Resource requirements specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    /// Minimum memory in MB
    pub min_memory_mb: u64,

    /// Recommended memory in MB
    pub recommended_memory_mb: u64,

    /// CPU cores required
    pub cpu_cores: f32,

    /// GPU required
    pub gpu_required: bool,

    /// Network bandwidth in Mbps
    pub network_bandwidth_mbps: Option<f32>,

    /// Storage requirements in MB
    pub storage_mb: Option<u64>,
}

/// Deployment information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentInfo {
    /// Deployment environment
    pub environment: String,

    /// Host information
    pub host: Option<String>,

    /// Container/process ID
    pub process_id: Option<String>,

    /// Deployment timestamp
    pub deployed_at: chrono::DateTime<chrono::Utc>,

    /// Deployment configuration
    pub config: HashMap<String, serde_json::Value>,
}

/// Agent health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Overall health state
    pub state: HealthState,

    /// Last health check
    pub last_check: chrono::DateTime<chrono::Utc>,

    /// Health checks
    pub checks: Vec<HealthCheck>,

    /// Issues/warnings
    pub issues: Vec<HealthIssue>,
}

/// Health state enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HealthState {
    /// Healthy and operational
    Healthy,

    /// Degraded but operational
    Degraded,

    /// Unhealthy/failing
    Unhealthy,

    /// Unknown state
    Unknown,
}

/// Individual health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    /// Check name
    pub name: String,

    /// Check result
    pub passed: bool,

    /// Check message
    pub message: Option<String>,

    /// Check duration in ms
    pub duration_ms: u64,
}

/// Health issue/warning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthIssue {
    /// Issue severity
    pub severity: IssueSeverity,

    /// Issue code
    pub code: String,

    /// Issue message
    pub message: String,

    /// When issue was detected
    pub detected_at: chrono::DateTime<chrono::Utc>,

    /// Suggested remediation
    pub remediation: Option<String>,
}

/// Issue severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Ord, PartialOrd, Eq)]
pub enum IssueSeverity {
    /// Informational
    Info,

    /// Warning
    Warning,

    /// Error
    Error,

    /// Critical
    Critical,
}

/// Metadata manager for rich metadata operations
pub struct MetadataManager {
    storage: HashMap<String, ExtendedAgentMetadata>,
}

impl MetadataManager {
    /// Create new metadata manager
    pub fn new() -> Self {
        Self {
            storage: HashMap::new(),
        }
    }

    /// Store extended metadata
    pub fn store(&mut self, id: String, metadata: ExtendedAgentMetadata) -> Result<()> {
        self.storage.insert(id, metadata);
        Ok(())
    }

    /// Retrieve extended metadata
    pub fn get(&self, id: &str) -> Option<&ExtendedAgentMetadata> {
        self.storage.get(id)
    }

    /// Update version info
    pub fn update_version(&mut self, id: &str, version: VersionInfo) -> Result<()> {
        match self.storage.get_mut(id) {
            Some(metadata) => {
                metadata.version = version;
                metadata.core.updated_at = chrono::Utc::now();
                Ok(())
            }
            None => anyhow::bail!("Agent '{}' not found", id),
        }
    }

    /// Add capability
    pub fn add_capability(&mut self, id: &str, capability: AgentCapability) -> Result<()> {
        match self.storage.get_mut(id) {
            Some(metadata) => {
                metadata.capabilities.push(capability);
                metadata.core.updated_at = chrono::Utc::now();
                Ok(())
            }
            None => anyhow::bail!("Agent '{}' not found", id),
        }
    }

    /// Update health status
    pub fn update_health(&mut self, id: &str, health: HealthStatus) -> Result<()> {
        match self.storage.get_mut(id) {
            Some(metadata) => {
                metadata.health = health;
                metadata.core.updated_at = chrono::Utc::now();
                Ok(())
            }
            None => anyhow::bail!("Agent '{}' not found", id),
        }
    }

    /// Find agents by capability
    pub fn find_by_capability(
        &self,
        capability_type: &CapabilityType,
    ) -> Vec<&ExtendedAgentMetadata> {
        self.storage
            .values()
            .filter(|metadata| {
                metadata.capabilities.iter().any(|cap| {
                    match (&cap.capability_type, capability_type) {
                        (CapabilityType::Custom(a), CapabilityType::Custom(b)) => a == b,
                        (a, b) => std::mem::discriminant(a) == std::mem::discriminant(b),
                    }
                })
            })
            .collect()
    }

    /// Find dependent agents
    pub fn find_dependents(&self, agent_id: &str) -> Vec<&ExtendedAgentMetadata> {
        self.storage
            .values()
            .filter(|metadata| {
                metadata
                    .dependencies
                    .iter()
                    .any(|dep| dep.agent_id == agent_id)
            })
            .collect()
    }
}

impl Default for MetadataManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for extended metadata
pub struct ExtendedMetadataBuilder {
    core: super::AgentMetadata,
    version: VersionInfo,
    dependencies: Vec<AgentDependency>,
    capabilities: Vec<AgentCapability>,
    resource_requirements: ResourceRequirements,
    deployment: Option<DeploymentInfo>,
    health: HealthStatus,
}

impl ExtendedMetadataBuilder {
    /// Create new builder from core metadata
    pub fn from_core(core: super::AgentMetadata) -> Self {
        Self {
            core,
            version: VersionInfo {
                version: "0.1.0".to_string(),
                commit_hash: None,
                build_timestamp: Some(chrono::Utc::now()),
                changelog: None,
            },
            dependencies: Vec::new(),
            capabilities: Vec::new(),
            resource_requirements: ResourceRequirements {
                min_memory_mb: 256,
                recommended_memory_mb: 512,
                cpu_cores: 1.0,
                gpu_required: false,
                network_bandwidth_mbps: None,
                storage_mb: None,
            },
            deployment: None,
            health: HealthStatus {
                state: HealthState::Unknown,
                last_check: chrono::Utc::now(),
                checks: Vec::new(),
                issues: Vec::new(),
            },
        }
    }

    /// Set version info
    pub fn version(mut self, version: VersionInfo) -> Self {
        self.version = version;
        self
    }

    /// Add dependency
    pub fn add_dependency(mut self, dependency: AgentDependency) -> Self {
        self.dependencies.push(dependency);
        self
    }

    /// Add capability
    pub fn add_capability(mut self, capability: AgentCapability) -> Self {
        self.capabilities.push(capability);
        self
    }

    /// Set resource requirements
    pub fn resource_requirements(mut self, requirements: ResourceRequirements) -> Self {
        self.resource_requirements = requirements;
        self
    }

    /// Set deployment info
    pub fn deployment(mut self, deployment: DeploymentInfo) -> Self {
        self.deployment = Some(deployment);
        self
    }

    /// Build extended metadata
    pub fn build(self) -> ExtendedAgentMetadata {
        ExtendedAgentMetadata {
            core: self.core,
            version: self.version,
            dependencies: self.dependencies,
            capabilities: self.capabilities,
            resource_requirements: self.resource_requirements,
            deployment: self.deployment.unwrap_or_else(|| DeploymentInfo {
                environment: "default".to_string(),
                host: None,
                process_id: None,
                deployed_at: chrono::Utc::now(),
                config: HashMap::new(),
            }),
            health: self.health,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metadata_manager() {
        let mut manager = MetadataManager::new();

        let core = crate::registry::AgentMetadata {
            id: "test-agent".to_string(),
            name: "Test Agent".to_string(),
            agent_type: "test".to_string(),
            description: "Test agent".to_string(),
            categories: vec![],
            custom_metadata: HashMap::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            status: crate::registry::AgentStatus::Active,
            metrics: crate::registry::AgentMetrics::default(),
        };

        let extended = ExtendedMetadataBuilder::from_core(core)
            .add_capability(AgentCapability {
                name: "tool-usage".to_string(),
                capability_type: CapabilityType::ToolUsage,
                config: HashMap::new(),
                enabled: true,
            })
            .build();

        manager.store("test-agent".to_string(), extended).unwrap();

        assert!(manager.get("test-agent").is_some());
        assert_eq!(
            manager.find_by_capability(&CapabilityType::ToolUsage).len(),
            1
        );
    }
}
