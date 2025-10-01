//! Tenant registry implementation for discovery and management

use anyhow::{anyhow, Result};
use async_trait::async_trait;
use dashmap::DashMap;
use llmspell_events::{EventBus, Language, UniversalEvent};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::traits::{TenantConfig, TenantLifecycleHook, TenantRegistry};

/// Default tenant registry implementation
pub struct DefaultTenantRegistry {
    /// Tenant configurations
    tenants: DashMap<String, TenantConfig>,

    /// Lifecycle hooks
    hooks: Arc<RwLock<Vec<Arc<dyn TenantLifecycleHook>>>>,

    /// Event bus for notifications
    event_bus: Option<Arc<EventBus>>,
}

impl DefaultTenantRegistry {
    /// Create a new tenant registry
    pub fn new() -> Self {
        Self {
            tenants: DashMap::new(),
            hooks: Arc::new(RwLock::new(Vec::new())),
            event_bus: None,
        }
    }

    /// Create with event bus for notifications
    pub fn with_event_bus(event_bus: Arc<EventBus>) -> Self {
        Self {
            tenants: DashMap::new(),
            hooks: Arc::new(RwLock::new(Vec::new())),
            event_bus: Some(event_bus),
        }
    }

    /// Add a lifecycle hook
    pub async fn add_hook(&self, hook: Arc<dyn TenantLifecycleHook>) {
        let mut hooks = self.hooks.write().await;
        hooks.push(hook);
    }

    /// Emit event to event bus
    async fn emit_event(&self, event_type: &str, tenant_id: &str, metadata: serde_json::Value) {
        if let Some(bus) = &self.event_bus {
            let event = UniversalEvent::new(
                format!("tenant.{}.{}", tenant_id, event_type),
                json!({
                    "type": event_type,
                    "tenant_id": tenant_id,
                    "timestamp": SystemTime::now(),
                    "metadata": metadata,
                }),
                Language::Rust,
            );

            if let Err(e) = bus.publish(event).await {
                warn!("Failed to emit tenant event: {}", e);
            }
        }
    }

    /// Call lifecycle hooks
    async fn call_hooks<F>(&self, f: F) -> Result<()>
    where
        F: Fn(
            &Arc<dyn TenantLifecycleHook>,
        )
            -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + '_>>,
    {
        let hooks = self.hooks.read().await;
        for hook in hooks.iter() {
            if let Err(e) = f(hook).await {
                warn!("Lifecycle hook error: {}", e);
                // Continue with other hooks even if one fails
            }
        }
        Ok(())
    }
}

#[async_trait]
impl TenantRegistry for DefaultTenantRegistry {
    async fn register_tenant(&self, config: TenantConfig) -> Result<()> {
        let tenant_id = config.tenant_id.clone();

        // Check if already exists
        if self.tenants.contains_key(&tenant_id) {
            return Err(anyhow!("Tenant {} already exists", tenant_id));
        }

        // Call pre-creation hooks
        let config_clone = config.clone();
        self.call_hooks(move |hook| {
            let config = config_clone.clone();
            Box::pin(async move { hook.on_tenant_created(&config).await })
        })
        .await?;

        // Store configuration
        self.tenants.insert(tenant_id.clone(), config.clone());

        // Emit event
        self.emit_event(
            "registered",
            &tenant_id,
            json!({
                "name": config.name,
                "active": config.active,
                "limits": config.limits,
            }),
        )
        .await;

        info!("Registered tenant: {}", tenant_id);
        Ok(())
    }

    async fn unregister_tenant(&self, tenant_id: &str) -> Result<()> {
        // Check if exists
        if !self.tenants.contains_key(tenant_id) {
            return Err(anyhow!("Tenant {} not found", tenant_id));
        }

        // Call pre-deletion hooks
        self.call_hooks(|hook| {
            let tenant_id = tenant_id.to_string();
            Box::pin(async move { hook.on_tenant_deleting(&tenant_id).await })
        })
        .await?;

        // Remove configuration
        let config = self.tenants.remove(tenant_id);

        // Call post-deletion hooks
        self.call_hooks(|hook| {
            let tenant_id = tenant_id.to_string();
            Box::pin(async move { hook.on_tenant_deleted(&tenant_id).await })
        })
        .await?;

        // Emit event
        if let Some((_, config)) = config {
            self.emit_event(
                "unregistered",
                tenant_id,
                json!({
                    "name": config.name,
                }),
            )
            .await;
        }

        info!("Unregistered tenant: {}", tenant_id);
        Ok(())
    }

    async fn get_tenant(&self, tenant_id: &str) -> Result<Option<TenantConfig>> {
        Ok(self.tenants.get(tenant_id).map(|entry| entry.clone()))
    }

    async fn list_tenants(&self) -> Result<Vec<String>> {
        Ok(self
            .tenants
            .iter()
            .map(|entry| entry.key().clone())
            .collect())
    }

    async fn update_metadata(
        &self,
        tenant_id: &str,
        metadata: HashMap<String, String>,
    ) -> Result<()> {
        let mut entry = self
            .tenants
            .get_mut(tenant_id)
            .ok_or_else(|| anyhow!("Tenant {} not found", tenant_id))?;

        // Update metadata
        entry.metadata = metadata.clone();
        entry.last_accessed = SystemTime::now();

        // Emit event
        self.emit_event(
            "metadata_updated",
            tenant_id,
            json!({
                "metadata": metadata,
            }),
        )
        .await;

        debug!("Updated metadata for tenant: {}", tenant_id);
        Ok(())
    }

    async fn tenant_exists(&self, tenant_id: &str) -> Result<bool> {
        Ok(self.tenants.contains_key(tenant_id))
    }
}

impl Default for DefaultTenantRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Filtered tenant registry that applies access control
pub struct FilteredTenantRegistry {
    /// Underlying registry
    inner: Arc<dyn TenantRegistry>,

    /// Filter function
    filter: Arc<dyn Fn(&str) -> bool + Send + Sync>,
}

impl FilteredTenantRegistry {
    /// Create a filtered registry
    pub fn new(
        inner: Arc<dyn TenantRegistry>,
        filter: impl Fn(&str) -> bool + Send + Sync + 'static,
    ) -> Self {
        Self {
            inner,
            filter: Arc::new(filter),
        }
    }
}

#[async_trait]
impl TenantRegistry for FilteredTenantRegistry {
    async fn register_tenant(&self, config: TenantConfig) -> Result<()> {
        if !(self.filter)(&config.tenant_id) {
            return Err(anyhow!("Access denied for tenant {}", config.tenant_id));
        }
        self.inner.register_tenant(config).await
    }

    async fn unregister_tenant(&self, tenant_id: &str) -> Result<()> {
        if !(self.filter)(tenant_id) {
            return Err(anyhow!("Access denied for tenant {}", tenant_id));
        }
        self.inner.unregister_tenant(tenant_id).await
    }

    async fn get_tenant(&self, tenant_id: &str) -> Result<Option<TenantConfig>> {
        if !(self.filter)(tenant_id) {
            return Ok(None);
        }
        self.inner.get_tenant(tenant_id).await
    }

    async fn list_tenants(&self) -> Result<Vec<String>> {
        let all_tenants = self.inner.list_tenants().await?;
        Ok(all_tenants
            .into_iter()
            .filter(|id| (self.filter)(id))
            .collect())
    }

    async fn update_metadata(
        &self,
        tenant_id: &str,
        metadata: HashMap<String, String>,
    ) -> Result<()> {
        if !(self.filter)(tenant_id) {
            return Err(anyhow!("Access denied for tenant {}", tenant_id));
        }
        self.inner.update_metadata(tenant_id, metadata).await
    }

    async fn tenant_exists(&self, tenant_id: &str) -> Result<bool> {
        if !(self.filter)(tenant_id) {
            return Ok(false);
        }
        self.inner.tenant_exists(tenant_id).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_tenant_registration() {
        let registry = DefaultTenantRegistry::new();

        let config = TenantConfig {
            tenant_id: "test-tenant".to_string(),
            name: "Test Tenant".to_string(),
            limits: Default::default(),
            active: true,
            metadata: HashMap::new(),
            created_at: SystemTime::now(),
            last_accessed: SystemTime::now(),
            custom_config: None,
        };

        // Register tenant
        registry.register_tenant(config.clone()).await.unwrap();

        // Check it exists
        assert!(registry.tenant_exists("test-tenant").await.unwrap());

        // Get tenant
        let retrieved = registry.get_tenant("test-tenant").await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Test Tenant");

        // List tenants
        let tenants = registry.list_tenants().await.unwrap();
        assert_eq!(tenants.len(), 1);
        assert_eq!(tenants[0], "test-tenant");

        // Unregister
        registry.unregister_tenant("test-tenant").await.unwrap();
        assert!(!registry.tenant_exists("test-tenant").await.unwrap());
    }

    #[tokio::test]
    async fn test_duplicate_registration() {
        let registry = DefaultTenantRegistry::new();

        let config = TenantConfig {
            tenant_id: "test-tenant".to_string(),
            name: "Test Tenant".to_string(),
            limits: Default::default(),
            active: true,
            metadata: HashMap::new(),
            created_at: SystemTime::now(),
            last_accessed: SystemTime::now(),
            custom_config: None,
        };

        // First registration should succeed
        registry.register_tenant(config.clone()).await.unwrap();

        // Second should fail
        assert!(registry.register_tenant(config).await.is_err());
    }

    #[tokio::test]
    async fn test_filtered_registry() {
        let base_registry = Arc::new(DefaultTenantRegistry::new());

        // Create configs for multiple tenants
        for i in 1..=5 {
            let config = TenantConfig {
                tenant_id: format!("tenant-{}", i),
                name: format!("Tenant {}", i),
                limits: Default::default(),
                active: true,
                metadata: HashMap::new(),
                created_at: SystemTime::now(),
                last_accessed: SystemTime::now(),
                custom_config: None,
            };
            base_registry.register_tenant(config).await.unwrap();
        }

        // Create filtered registry that only shows even-numbered tenants
        let filtered = FilteredTenantRegistry::new(base_registry.clone(), |id| {
            id.split('-')
                .next_back()
                .and_then(|n| n.parse::<u32>().ok())
                .map(|n| n.is_multiple_of(2))
                .unwrap_or(false)
        });

        // List should only show even tenants
        let tenants = filtered.list_tenants().await.unwrap();
        assert_eq!(tenants.len(), 2);
        assert!(tenants.contains(&"tenant-2".to_string()));
        assert!(tenants.contains(&"tenant-4".to_string()));

        // Can't access odd tenants
        assert!(filtered.get_tenant("tenant-1").await.unwrap().is_none());
        assert!(filtered.get_tenant("tenant-2").await.unwrap().is_some());
    }
}
