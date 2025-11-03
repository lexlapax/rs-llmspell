//! Core traits for multi-tenant infrastructure

use anyhow::Result;
use async_trait::async_trait;
use llmspell_core::state::StateScope;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

// Re-export TenantScoped from llmspell-core (Phase 13b.3.4 - moved to core to avoid cyclic deps)
pub use llmspell_core::TenantScoped;

/// Resource that can track usage metrics
#[async_trait]
pub trait UsageTracked: Send + Sync {
    /// Record resource usage
    async fn record_usage(&self, operation: &str, units: usize) -> Result<()>;

    /// Get current usage metrics
    async fn get_usage(&self) -> Result<HashMap<String, usize>>;

    /// Reset usage metrics
    async fn reset_usage(&self) -> Result<()>;
}

/// Manager for tenant-scoped resources
#[async_trait]
pub trait TenantResourceManager: Send + Sync {
    /// Type of resource being managed
    type Resource: TenantScoped + Send + Sync;

    /// Create a resource for a tenant
    async fn create_resource(
        &self,
        tenant_id: &str,
        config: serde_json::Value,
    ) -> Result<Arc<Self::Resource>>;

    /// Get resource for a tenant
    async fn get_resource(&self, tenant_id: &str) -> Result<Option<Arc<Self::Resource>>>;

    /// Delete resource for a tenant
    async fn delete_resource(&self, tenant_id: &str) -> Result<()>;

    /// List all resources
    async fn list_resources(&self) -> Result<Vec<String>>;
}

/// Registry for discovering and managing tenants
#[async_trait]
pub trait TenantRegistry: Send + Sync {
    /// Register a new tenant
    async fn register_tenant(&self, config: TenantConfig) -> Result<()>;

    /// Unregister a tenant
    async fn unregister_tenant(&self, tenant_id: &str) -> Result<()>;

    /// Get tenant configuration
    async fn get_tenant(&self, tenant_id: &str) -> Result<Option<TenantConfig>>;

    /// List all registered tenants
    async fn list_tenants(&self) -> Result<Vec<String>>;

    /// Update tenant metadata
    async fn update_metadata(
        &self,
        tenant_id: &str,
        metadata: HashMap<String, String>,
    ) -> Result<()>;

    /// Check if tenant exists
    async fn tenant_exists(&self, tenant_id: &str) -> Result<bool>;
}

/// Tenant lifecycle hooks
#[async_trait]
pub trait TenantLifecycleHook: Send + Sync {
    /// Called when a tenant is created
    async fn on_tenant_created(&self, config: &TenantConfig) -> Result<()>;

    /// Called before a tenant is deleted
    async fn on_tenant_deleting(&self, tenant_id: &str) -> Result<()>;

    /// Called after a tenant is deleted
    async fn on_tenant_deleted(&self, tenant_id: &str) -> Result<()>;

    /// Called when tenant is activated
    async fn on_tenant_activated(&self, tenant_id: &str) -> Result<()>;

    /// Called when tenant is deactivated
    async fn on_tenant_deactivated(&self, tenant_id: &str) -> Result<()>;
}

/// Extension point for custom tenant operations
#[async_trait]
pub trait TenantExtension: Send + Sync {
    /// Name of the extension
    fn name(&self) -> &str;

    /// Initialize extension for a tenant
    async fn initialize(&self, tenant_id: &str) -> Result<()>;

    /// Cleanup extension for a tenant
    async fn cleanup(&self, tenant_id: &str) -> Result<()>;

    /// Execute custom operation
    async fn execute(
        &self,
        tenant_id: &str,
        operation: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value>;
}

/// Tenant configuration with generic metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantConfig {
    /// Unique tenant identifier
    pub tenant_id: String,

    /// Display name
    pub name: String,

    /// Resource limits
    pub limits: TenantLimits,

    /// Whether tenant is active
    pub active: bool,

    /// Generic metadata
    pub metadata: HashMap<String, String>,

    /// Creation timestamp
    pub created_at: std::time::SystemTime,

    /// Last access timestamp
    pub last_accessed: std::time::SystemTime,

    /// Custom configuration
    pub custom_config: Option<serde_json::Value>,
}

/// Tenant resource limits
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TenantLimits {
    /// Maximum number of vectors
    pub max_vectors: Option<usize>,

    /// Maximum storage in bytes
    pub max_storage_bytes: Option<usize>,

    /// Maximum queries per second
    pub max_queries_per_second: Option<u32>,

    /// Maximum dimensions for vectors
    pub max_dimensions: Option<usize>,

    /// Allow temporary overflow
    #[serde(default)]
    pub allow_overflow: bool,

    /// Custom limits as key-value pairs
    #[serde(default)]
    pub custom_limits: HashMap<String, serde_json::Value>,
}

/// Result of a tenant operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TenantOperationResult {
    /// Whether operation succeeded
    pub success: bool,

    /// Operation message
    pub message: String,

    /// Optional result data
    pub data: Option<serde_json::Value>,

    /// Usage consumed by this operation
    pub usage: Option<HashMap<String, usize>>,
}

/// Tenant isolation mode
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum IsolationMode {
    /// Logical isolation (shared resources, separate data)
    Logical,

    /// Physical isolation (separate resources)
    Physical,

    /// Hybrid (some resources shared, some separate)
    Hybrid,
}

/// Multi-tenancy configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiTenancyConfig {
    /// Isolation mode
    pub isolation_mode: IsolationMode,

    /// Whether to enable usage tracking
    pub enable_usage_tracking: bool,

    /// Whether to enable audit logging
    pub enable_audit_logging: bool,

    /// Default resource limits for new tenants
    pub default_limits: TenantLimits,

    /// Maximum number of tenants
    pub max_tenants: Option<usize>,
}

impl Default for MultiTenancyConfig {
    fn default() -> Self {
        Self {
            isolation_mode: IsolationMode::Logical,
            enable_usage_tracking: true,
            enable_audit_logging: true,
            default_limits: TenantLimits::default(),
            max_tenants: None,
        }
    }
}
