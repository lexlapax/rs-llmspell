//! Service discovery abstraction for distributed deployment
//!
//! Following the three-layer architecture pattern from Phase 9.1-9.3

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;
use tokio::sync::RwLock;

/// Service discovery errors
#[derive(Error, Debug)]
pub enum DiscoveryError {
    #[error("Service not found: {0}")]
    ServiceNotFound(String),

    #[error("Registration failed: {0}")]
    RegistrationFailed(String),

    #[error("Health check failed: {0}")]
    HealthCheckFailed(String),

    #[error("Query failed: {0}")]
    QueryFailed(String),
}

/// Result type for discovery operations
pub type DiscoveryResult<T> = Result<T, DiscoveryError>;

/// Health status of a service
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Service is healthy and accepting requests
    Healthy,
    /// Service is degraded but operational
    Degraded,
    /// Service is unhealthy and not accepting requests
    Unhealthy,
    /// Health status is unknown
    Unknown,
}

/// Service information for registration and discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    /// Unique service identifier
    pub id: String,
    /// Service name
    pub name: String,
    /// Service version
    pub version: String,
    /// Network address (host:port)
    pub address: String,
    /// Service metadata
    pub metadata: HashMap<String, String>,
    /// Supported protocols
    pub protocols: Vec<String>,
    /// Current health status
    pub health: HealthStatus,
}

/// Query parameters for service discovery
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServiceQuery {
    /// Filter by service name
    pub name: Option<String>,
    /// Filter by protocol support
    pub protocol: Option<String>,
    /// Filter by health status
    pub health_status: Option<HealthStatus>,
    /// Filter by metadata key-value pairs
    pub metadata: HashMap<String, String>,
}

/// Service discovery trait - Layer 1: Trait abstraction
#[async_trait]
pub trait ServiceDiscovery: Send + Sync {
    /// Register a service with the discovery system
    async fn register(&self, service: ServiceInfo) -> DiscoveryResult<()>;

    /// Discover services matching the query
    async fn discover(&self, query: ServiceQuery) -> DiscoveryResult<Vec<ServiceInfo>>;

    /// Check health status of a specific service
    async fn health_check(&self, service_id: &str) -> DiscoveryResult<HealthStatus>;

    /// Unregister a service
    async fn unregister(&self, service_id: &str) -> DiscoveryResult<()>;

    /// Update service information
    async fn update(&self, service: ServiceInfo) -> DiscoveryResult<()>;
}

/// Local in-memory service discovery - Layer 3: Concrete implementation
pub struct LocalServiceDiscovery {
    services: Arc<RwLock<HashMap<String, ServiceInfo>>>,
}

impl LocalServiceDiscovery {
    /// Create a new local service discovery instance
    #[must_use]
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Check if a service matches the query
    fn matches_query(service: &ServiceInfo, query: &ServiceQuery) -> bool {
        // Check name filter
        if let Some(ref name) = query.name {
            if &service.name != name {
                return false;
            }
        }

        // Check protocol filter
        if let Some(ref protocol) = query.protocol {
            if !service.protocols.contains(protocol) {
                return false;
            }
        }

        // Check health status filter
        if let Some(health) = query.health_status {
            if service.health != health {
                return false;
            }
        }

        // Check metadata filters
        for (key, value) in &query.metadata {
            if service.metadata.get(key) != Some(value) {
                return false;
            }
        }

        true
    }
}

impl Default for LocalServiceDiscovery {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ServiceDiscovery for LocalServiceDiscovery {
    async fn register(&self, service: ServiceInfo) -> DiscoveryResult<()> {
        let mut services = self.services.write().await;
        services.insert(service.id.clone(), service);
        drop(services);
        Ok(())
    }

    async fn discover(&self, query: ServiceQuery) -> DiscoveryResult<Vec<ServiceInfo>> {
        let services = self.services.read().await;
        let matching: Vec<ServiceInfo> = services
            .values()
            .filter(|s| Self::matches_query(s, &query))
            .cloned()
            .collect();
        drop(services);
        Ok(matching)
    }

    async fn health_check(&self, service_id: &str) -> DiscoveryResult<HealthStatus> {
        let services = self.services.read().await;
        let result = services
            .get(service_id)
            .map(|s| s.health)
            .ok_or_else(|| DiscoveryError::ServiceNotFound(service_id.to_string()));
        drop(services);
        result
    }

    async fn unregister(&self, service_id: &str) -> DiscoveryResult<()> {
        let mut services = self.services.write().await;
        let result = services
            .remove(service_id)
            .map(|_| ())
            .ok_or_else(|| DiscoveryError::ServiceNotFound(service_id.to_string()));
        drop(services);
        result
    }

    async fn update(&self, service: ServiceInfo) -> DiscoveryResult<()> {
        let mut services = self.services.write().await;
        let result = if services.contains_key(&service.id) {
            services.insert(service.id.clone(), service);
            Ok(())
        } else {
            Err(DiscoveryError::ServiceNotFound(service.id.clone()))
        };
        drop(services);
        result
    }
}

/// Null service discovery for testing - Following Phase 9.3 pattern
pub struct NullServiceDiscovery;

#[async_trait]
impl ServiceDiscovery for NullServiceDiscovery {
    async fn register(&self, _service: ServiceInfo) -> DiscoveryResult<()> {
        Ok(())
    }

    async fn discover(&self, _query: ServiceQuery) -> DiscoveryResult<Vec<ServiceInfo>> {
        Ok(Vec::new())
    }

    async fn health_check(&self, _service_id: &str) -> DiscoveryResult<HealthStatus> {
        Ok(HealthStatus::Unknown)
    }

    async fn unregister(&self, _service_id: &str) -> DiscoveryResult<()> {
        Ok(())
    }

    async fn update(&self, _service: ServiceInfo) -> DiscoveryResult<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_local_discovery_registration() {
        let discovery = LocalServiceDiscovery::new();

        let service = ServiceInfo {
            id: "test-service".to_string(),
            name: "TestService".to_string(),
            version: "1.0.0".to_string(),
            address: "127.0.0.1:8080".to_string(),
            metadata: HashMap::new(),
            protocols: vec!["LRP".to_string()],
            health: HealthStatus::Healthy,
        };

        // Register service
        discovery.register(service.clone()).await.unwrap();

        // Verify it can be discovered
        let query = ServiceQuery {
            name: Some("TestService".to_string()),
            ..Default::default()
        };
        let found = discovery.discover(query).await.unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].id, "test-service");
    }

    #[tokio::test]
    async fn test_discovery_with_filters() {
        let discovery = LocalServiceDiscovery::new();

        // Register multiple services
        for i in 0..3 {
            let service = ServiceInfo {
                id: format!("service-{i}"),
                name: if i == 0 { "ServiceA" } else { "ServiceB" }.to_string(),
                version: "1.0.0".to_string(),
                address: format!("127.0.0.1:808{i}"),
                metadata: HashMap::new(),
                protocols: if i == 0 {
                    vec!["LRP".to_string()]
                } else {
                    vec!["LDP".to_string()]
                },
                health: if i == 2 {
                    HealthStatus::Unhealthy
                } else {
                    HealthStatus::Healthy
                },
            };
            discovery.register(service).await.unwrap();
        }

        // Query by protocol
        let query = ServiceQuery {
            protocol: Some("LRP".to_string()),
            ..Default::default()
        };
        let found = discovery.discover(query).await.unwrap();
        assert_eq!(found.len(), 1);

        // Query by health status
        let query = ServiceQuery {
            health_status: Some(HealthStatus::Healthy),
            ..Default::default()
        };
        let found = discovery.discover(query).await.unwrap();
        assert_eq!(found.len(), 2);
    }

    #[tokio::test]
    async fn test_null_discovery() {
        let discovery = NullServiceDiscovery;

        let service = ServiceInfo {
            id: "test".to_string(),
            name: "Test".to_string(),
            version: "1.0.0".to_string(),
            address: "127.0.0.1:8080".to_string(),
            metadata: HashMap::new(),
            protocols: vec![],
            health: HealthStatus::Healthy,
        };

        // All operations should succeed but do nothing
        discovery.register(service).await.unwrap();
        let found = discovery.discover(ServiceQuery::default()).await.unwrap();
        assert_eq!(found.len(), 0);
        let health = discovery.health_check("test").await.unwrap();
        assert_eq!(health, HealthStatus::Unknown);
    }
}
