//! Integration tests for llmspell-tenancy crate

use anyhow::Result;
use async_trait::async_trait;
#[allow(unused_imports)] // Used in event type inference
use llmspell_events::{EventBus, UniversalEvent};
use llmspell_storage::backends::sqlite::{SqliteBackend, SqliteConfig, SqliteVectorStorage};
use llmspell_storage::{VectorEntry, VectorQuery};
use llmspell_tenancy::{
    DefaultTenantRegistry, MultiTenantVectorManager, TenantConfig, TenantLifecycleHook,
    TenantLimits, TenantRegistry,
};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;
use tokio::sync::mpsc;

/// Test lifecycle hook that records calls
struct TestLifecycleHook {
    calls: Arc<Mutex<Vec<String>>>,
}

impl TestLifecycleHook {
    fn new() -> (Self, Arc<Mutex<Vec<String>>>) {
        let calls = Arc::new(Mutex::new(Vec::new()));
        let hook = Self {
            calls: calls.clone(),
        };
        (hook, calls)
    }
}

#[async_trait]
impl TenantLifecycleHook for TestLifecycleHook {
    async fn on_tenant_created(&self, config: &TenantConfig) -> Result<()> {
        self.calls
            .lock()
            .unwrap()
            .push(format!("created:{}", config.tenant_id));
        Ok(())
    }

    async fn on_tenant_deleting(&self, tenant_id: &str) -> Result<()> {
        self.calls
            .lock()
            .unwrap()
            .push(format!("deleting:{tenant_id}"));
        Ok(())
    }

    async fn on_tenant_deleted(&self, tenant_id: &str) -> Result<()> {
        self.calls
            .lock()
            .unwrap()
            .push(format!("deleted:{tenant_id}"));
        Ok(())
    }

    async fn on_tenant_activated(&self, tenant_id: &str) -> Result<()> {
        self.calls
            .lock()
            .unwrap()
            .push(format!("activated:{tenant_id}"));
        Ok(())
    }

    async fn on_tenant_deactivated(&self, tenant_id: &str) -> Result<()> {
        self.calls
            .lock()
            .unwrap()
            .push(format!("deactivated:{tenant_id}"));
        Ok(())
    }
}

#[tokio::test]
async fn test_tenant_registry_with_lifecycle_hooks() -> Result<()> {
    let registry = DefaultTenantRegistry::new();
    let (hook, calls) = TestLifecycleHook::new();

    // Add lifecycle hook
    registry.add_hook(Arc::new(hook)).await;

    let config = TenantConfig {
        tenant_id: "test-tenant".to_string(),
        name: "Test Tenant".to_string(),
        limits: TenantLimits::default(),
        active: true,
        metadata: HashMap::new(),
        created_at: SystemTime::now(),
        last_accessed: SystemTime::now(),
        custom_config: None,
    };

    // Register tenant - should trigger hook
    registry.register_tenant(config).await?;

    // Unregister tenant - should trigger hooks
    registry.unregister_tenant("test-tenant").await?;

    // Check hook calls
    let hook_calls = calls.lock().unwrap().clone();
    assert_eq!(hook_calls.len(), 3);
    assert_eq!(hook_calls[0], "created:test-tenant");
    assert_eq!(hook_calls[1], "deleting:test-tenant");
    assert_eq!(hook_calls[2], "deleted:test-tenant");

    Ok(())
}

#[tokio::test]
async fn test_tenant_registry_with_event_bus() -> Result<()> {
    let event_bus = Arc::new(EventBus::new());
    let (tx, mut rx) = mpsc::channel(100);

    // Start event listener
    let event_bus_clone = event_bus.clone();
    let tx_clone = tx.clone();
    tokio::spawn(async move {
        let mut event_stream = event_bus_clone.subscribe("tenant.*").await.unwrap();
        while let Some(event) = event_stream.recv().await {
            let _ = tx_clone.send(event).await;
        }
    });

    // Give some time for subscription to be set up
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;

    let registry = DefaultTenantRegistry::with_event_bus(event_bus);

    let config = TenantConfig {
        tenant_id: "event-tenant".to_string(),
        name: "Event Tenant".to_string(),
        limits: TenantLimits::default(),
        active: true,
        metadata: HashMap::new(),
        created_at: SystemTime::now(),
        last_accessed: SystemTime::now(),
        custom_config: None,
    };

    // Register tenant - should emit event
    registry.register_tenant(config).await?;

    // Wait for event
    let event = tokio::time::timeout(
        std::time::Duration::from_millis(500), // Longer timeout
        rx.recv(),
    )
    .await
    .map_err(|_| anyhow::anyhow!("Timeout waiting for event"))?
    .ok_or_else(|| anyhow::anyhow!("No event received"))?;

    assert!(event.event_type.contains("tenant.event-tenant.registered"));

    Ok(())
}

#[tokio::test]
async fn test_multi_tenant_vector_manager_with_events() -> Result<()> {
    let config = SqliteConfig::new(":memory:");
    let backend = Arc::new(SqliteBackend::new(config).await?);
    let storage = Arc::new(SqliteVectorStorage::new(backend, 3).await?);
    let event_bus = Arc::new(EventBus::new());

    // Set up event subscription
    let (tx, mut rx) = mpsc::channel(100);

    let tx_clone = tx.clone();
    let event_bus_clone = event_bus.clone();
    tokio::spawn(async move {
        let mut event_stream = event_bus_clone.subscribe("tenant.*").await.unwrap();
        while let Some(event) = event_stream.recv().await {
            let _ = tx_clone.send(event).await;
        }
    });

    // Give some time for subscription to be set up
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;

    let manager = MultiTenantVectorManager::with_event_bus(storage, event_bus);

    let config = TenantConfig {
        tenant_id: "vector-tenant".to_string(),
        name: "Vector Tenant".to_string(),
        limits: TenantLimits {
            max_vectors: Some(100),
            max_storage_bytes: Some(1024 * 1024),
            max_queries_per_second: Some(10),
            max_dimensions: Some(10),
            allow_overflow: false,
            custom_limits: HashMap::new(),
        },
        active: true,
        metadata: HashMap::new(),
        created_at: SystemTime::now(),
        last_accessed: SystemTime::now(),
        custom_config: None,
    };

    // Create tenant - should emit creation event
    manager.create_tenant(config).await?;

    // Wait for creation event
    let creation_event = tokio::time::timeout(std::time::Duration::from_millis(500), rx.recv())
        .await
        .map_err(|_| anyhow::anyhow!("Timeout waiting for event"))?
        .ok_or_else(|| anyhow::anyhow!("No event received"))?;

    assert!(creation_event
        .event_type
        .contains("tenant.vector-tenant.created"));

    // Insert vectors - should emit insertion event
    let vectors = vec![VectorEntry::new(
        "test-vector-1".to_string(),
        vec![1.0, 2.0, 3.0],
    )];

    manager.insert_for_tenant("vector-tenant", vectors).await?;

    // Wait for insertion event
    let insert_event = tokio::time::timeout(std::time::Duration::from_millis(500), rx.recv())
        .await
        .map_err(|_| anyhow::anyhow!("Timeout waiting for event"))?
        .ok_or_else(|| anyhow::anyhow!("No event received"))?;

    assert!(insert_event
        .event_type
        .contains("tenant.vector-tenant.vectors_inserted"));

    // Search vectors - should emit search event
    let query = VectorQuery {
        vector: vec![1.0, 2.0, 3.0],
        k: 5,
        filter: None,
        threshold: None,
        include_metadata: true,
        scope: None,
        event_time_range: None,
        ingestion_time_range: None,
        exclude_expired: false,
    };

    manager.search_for_tenant("vector-tenant", query).await?;

    // Wait for search event
    let search_event = tokio::time::timeout(std::time::Duration::from_millis(500), rx.recv())
        .await
        .map_err(|_| anyhow::anyhow!("Timeout waiting for event"))?
        .ok_or_else(|| anyhow::anyhow!("No event received"))?;

    assert!(search_event
        .event_type
        .contains("tenant.vector-tenant.vectors_searched"));

    Ok(())
}

#[tokio::test]
async fn test_tenant_limits_enforcement() -> Result<()> {
    let config = SqliteConfig::new(":memory:");
    let backend = Arc::new(SqliteBackend::new(config).await?);
    let storage = Arc::new(SqliteVectorStorage::new(backend, 3).await?);
    let manager = MultiTenantVectorManager::new(storage);

    let config = TenantConfig {
        tenant_id: "limited-tenant".to_string(),
        name: "Limited Tenant".to_string(),
        limits: TenantLimits {
            max_vectors: Some(10), // Higher limit so we can test dimensions
            max_dimensions: Some(3),
            allow_overflow: false,
            ..Default::default()
        },
        active: true,
        metadata: HashMap::new(),
        created_at: SystemTime::now(),
        last_accessed: SystemTime::now(),
        custom_config: None,
    };

    manager.create_tenant(config).await?;

    // Try to insert vector with too many dimensions - should fail
    let oversized_vector = vec![VectorEntry::new(
        "oversized-vector".to_string(),
        vec![1.0, 2.0, 3.0, 4.0], // 4 dimensions > 3 limit
    )];
    let result = manager
        .insert_for_tenant("limited-tenant", oversized_vector)
        .await;
    assert!(result.is_err());
    let error_msg = result.unwrap_err().to_string();
    assert!(error_msg.contains("dimension") || error_msg.contains("exceeds"));

    // Insert first vector - should succeed
    let vectors1 = vec![VectorEntry::new(
        "vector-1".to_string(),
        vec![1.0, 2.0, 3.0],
    )];
    manager
        .insert_for_tenant("limited-tenant", vectors1)
        .await?;

    // Insert second vector - should succeed
    let vectors2 = vec![VectorEntry::new(
        "vector-2".to_string(),
        vec![4.0, 5.0, 6.0],
    )];
    manager
        .insert_for_tenant("limited-tenant", vectors2)
        .await?;

    Ok(())
}

#[tokio::test]
async fn test_tenant_isolation() -> Result<()> {
    let config = SqliteConfig::new(":memory:");
    let backend = Arc::new(SqliteBackend::new(config).await?);
    let storage = Arc::new(SqliteVectorStorage::new(backend, 3).await?);
    let manager = MultiTenantVectorManager::new(storage);

    // Create two tenants
    let config1 = TenantConfig {
        tenant_id: "tenant-1".to_string(),
        name: "Tenant 1".to_string(),
        limits: TenantLimits::default(),
        active: true,
        metadata: HashMap::new(),
        created_at: SystemTime::now(),
        last_accessed: SystemTime::now(),
        custom_config: None,
    };

    let config2 = TenantConfig {
        tenant_id: "tenant-2".to_string(),
        name: "Tenant 2".to_string(),
        limits: TenantLimits::default(),
        active: true,
        metadata: HashMap::new(),
        created_at: SystemTime::now(),
        last_accessed: SystemTime::now(),
        custom_config: None,
    };

    manager.create_tenant(config1).await?;
    manager.create_tenant(config2).await?;

    // Insert different vectors for each tenant
    let vectors1 = vec![VectorEntry::new(
        "tenant1-vector".to_string(),
        vec![1.0, 0.0, 0.0],
    )];
    let vectors2 = vec![VectorEntry::new(
        "tenant2-vector".to_string(),
        vec![0.0, 1.0, 0.0],
    )];

    manager.insert_for_tenant("tenant-1", vectors1).await?;
    manager.insert_for_tenant("tenant-2", vectors2).await?;

    // Search from each tenant - should only see their own vectors
    let query = VectorQuery {
        vector: vec![1.0, 0.0, 0.0],
        k: 10,
        filter: None,
        threshold: None,
        include_metadata: true,
        scope: None,
        event_time_range: None,
        ingestion_time_range: None,
        exclude_expired: false,
    };

    let results1 = manager.search_for_tenant("tenant-1", query.clone()).await?;
    let results2 = manager.search_for_tenant("tenant-2", query).await?;

    // Tenant 1 should find their vector (perfect match)
    assert_eq!(results1.len(), 1);

    // Tenant 2 should find their vector but with lower similarity
    assert_eq!(results2.len(), 1);

    // Results should be different (different vectors)
    assert_ne!(results1[0].id, results2[0].id);

    Ok(())
}

#[tokio::test]
async fn test_inactive_tenant_access() -> Result<()> {
    let config = SqliteConfig::new(":memory:");
    let backend = Arc::new(SqliteBackend::new(config).await?);
    let storage = Arc::new(SqliteVectorStorage::new(backend, 3).await?);
    let manager = MultiTenantVectorManager::new(storage);

    let config = TenantConfig {
        tenant_id: "inactive-tenant".to_string(),
        name: "Inactive Tenant".to_string(),
        limits: TenantLimits::default(),
        active: false, // Inactive tenant
        metadata: HashMap::new(),
        created_at: SystemTime::now(),
        last_accessed: SystemTime::now(),
        custom_config: None,
    };

    manager.create_tenant(config).await?;

    // Try to insert vectors - should fail
    let vectors = vec![VectorEntry::new(
        "inactive-vector".to_string(),
        vec![1.0, 2.0, 3.0],
    )];
    let result = manager.insert_for_tenant("inactive-tenant", vectors).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("inactive"));

    // Try to search - should also fail
    let query = VectorQuery {
        vector: vec![1.0, 2.0, 3.0],
        k: 5,
        filter: None,
        threshold: None,
        include_metadata: true,
        scope: None,
        event_time_range: None,
        ingestion_time_range: None,
        exclude_expired: false,
    };

    let result = manager.search_for_tenant("inactive-tenant", query).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("inactive"));

    Ok(())
}
