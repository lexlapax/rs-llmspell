# llmspell-tenancy

**Multi-tenant isolation and resource management (Phase 8)**

**🔗 Navigation**: [← Rust API](README.md) | [Crate Docs](https://docs.rs/llmspell-tenancy) | [Source](../../../../llmspell-tenancy)

---

## Overview

`llmspell-tenancy` provides comprehensive multi-tenant capabilities ensuring complete isolation between tenants including data separation, resource quotas, billing tracking, and cross-tenant operations. This is a Phase 8 core component enabling enterprise-grade multi-tenancy.

**Key Features:**
- 🏢 Complete tenant isolation
- 📊 Resource quota management
- 💰 Usage tracking and billing
- 🔐 Tenant-specific security contexts
- 🔄 Cross-tenant operations (with permissions)
- 📈 Tenant metrics and analytics
- 🛡️ Data isolation guarantees
- ⚡ Performance isolation

## Core Components

### TenantManager

Central tenant orchestration:

```rust
use async_trait::async_trait;
use uuid::Uuid;

pub struct TenantManager {
    registry: Arc<RwLock<TenantRegistry>>,
    storage: Arc<dyn TenantStorage>,
    quota_manager: Arc<QuotaManager>,
    isolation_manager: Arc<IsolationManager>,
    billing: Option<Arc<dyn BillingProvider>>,
    metrics: Arc<TenantMetrics>,
}

impl TenantManager {
    /// Create a new tenant manager
    pub async fn new(config: TenantConfig) -> Result<Self> {
        let storage = create_storage_backend(&config.storage)?;
        
        Ok(Self {
            registry: Arc::new(RwLock::new(TenantRegistry::new())),
            storage: Arc::new(storage),
            quota_manager: Arc::new(QuotaManager::new(config.quotas)),
            isolation_manager: Arc::new(IsolationManager::new(config.isolation)),
            billing: config.billing.map(|b| Arc::new(create_billing_provider(b))),
            metrics: Arc::new(TenantMetrics::default()),
        })
    }
    
    /// Create a new tenant
    pub async fn create_tenant(&self, request: CreateTenantRequest) -> Result<Tenant> {
        // Validate tenant creation
        self.validate_tenant_creation(&request)?;
        
        let tenant = Tenant {
            id: TenantId(Uuid::new_v4()),
            name: request.name,
            display_name: request.display_name,
            status: TenantStatus::Active,
            tier: request.tier.unwrap_or(TenantTier::Free),
            metadata: request.metadata,
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
            settings: TenantSettings::default(),
        };
        
        // Initialize tenant resources
        self.initialize_tenant_resources(&tenant).await?;
        
        // Register tenant
        self.registry.write().await.register(tenant.clone());
        self.storage.save_tenant(&tenant).await?;
        
        // Set up quotas
        self.quota_manager.initialize_tenant(&tenant.id, &tenant.tier).await?;
        
        // Set up isolation
        self.isolation_manager.create_isolation_context(&tenant.id).await?;
        
        self.metrics.tenants_created.fetch_add(1, Ordering::Relaxed);
        
        Ok(tenant)
    }
    
    /// Get tenant by ID
    pub async fn get_tenant(&self, id: &TenantId) -> Result<Option<Tenant>> {
        if let Some(tenant) = self.registry.read().await.get(id) {
            return Ok(Some(tenant.clone()));
        }
        
        self.storage.load_tenant(id).await
    }
    
    /// Update tenant
    pub async fn update_tenant(&self, id: &TenantId, update: UpdateTenantRequest) -> Result<Tenant> {
        let mut tenant = self.get_tenant(id).await?
            .ok_or_else(|| Error::TenantNotFound(id.clone()))?;
        
        // Apply updates
        if let Some(name) = update.display_name {
            tenant.display_name = name;
        }
        
        if let Some(tier) = update.tier {
            self.upgrade_tenant_tier(&tenant.id, &tenant.tier, &tier).await?;
            tenant.tier = tier;
        }
        
        if let Some(status) = update.status {
            tenant.status = status;
        }
        
        tenant.updated_at = SystemTime::now();
        
        // Save changes
        self.storage.save_tenant(&tenant).await?;
        self.registry.write().await.update(tenant.clone());
        
        Ok(tenant)
    }
    
    /// Delete tenant
    pub async fn delete_tenant(&self, id: &TenantId) -> Result<()> {
        // Check if tenant can be deleted
        self.validate_tenant_deletion(id).await?;
        
        // Clean up resources
        self.cleanup_tenant_resources(id).await?;
        
        // Remove from registry and storage
        self.registry.write().await.remove(id);
        self.storage.delete_tenant(id).await?;
        
        // Clean up isolation
        self.isolation_manager.remove_isolation_context(id).await?;
        
        self.metrics.tenants_deleted.fetch_add(1, Ordering::Relaxed);
        
        Ok(())
    }
}
```

### Tenant Isolation

Complete data and resource isolation:

```rust
pub struct IsolationManager {
    config: IsolationConfig,
    contexts: Arc<RwLock<HashMap<TenantId, IsolationContext>>>,
}

#[derive(Debug, Clone)]
pub struct IsolationContext {
    pub tenant_id: TenantId,
    pub namespace: String,
    pub data_boundary: DataBoundary,
    pub network_policy: NetworkPolicy,
    pub resource_limits: ResourceLimits,
    pub security_context: SecurityContext,
}

#[derive(Debug, Clone)]
pub struct DataBoundary {
    pub storage_path: PathBuf,
    pub database_schema: String,
    pub cache_namespace: String,
    pub encryption_key_id: Option<String>,
}

impl IsolationManager {
    /// Create isolation context for tenant
    pub async fn create_isolation_context(&self, tenant_id: &TenantId) -> Result<IsolationContext> {
        let namespace = format!("tenant_{}", tenant_id.0);
        
        let context = IsolationContext {
            tenant_id: tenant_id.clone(),
            namespace: namespace.clone(),
            data_boundary: DataBoundary {
                storage_path: self.config.base_path.join(&namespace),
                database_schema: namespace.clone(),
                cache_namespace: namespace.clone(),
                encryption_key_id: if self.config.encrypt_tenant_data {
                    Some(self.generate_encryption_key(tenant_id).await?)
                } else {
                    None
                },
            },
            network_policy: self.config.default_network_policy.clone(),
            resource_limits: self.config.default_resource_limits.clone(),
            security_context: self.create_security_context(tenant_id),
        };
        
        // Create isolated resources
        self.create_isolated_storage(&context).await?;
        self.create_isolated_database(&context).await?;
        self.create_isolated_cache(&context).await?;
        
        self.contexts.write().await.insert(tenant_id.clone(), context.clone());
        
        Ok(context)
    }
    
    /// Create isolated storage
    async fn create_isolated_storage(&self, context: &IsolationContext) -> Result<()> {
        // Create tenant-specific directory
        tokio::fs::create_dir_all(&context.data_boundary.storage_path).await?;
        
        // Set permissions (Unix-like systems)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = std::fs::Permissions::from_mode(0o700);
            tokio::fs::set_permissions(&context.data_boundary.storage_path, permissions).await?;
        }
        
        Ok(())
    }
    
    /// Create isolated database schema
    async fn create_isolated_database(&self, context: &IsolationContext) -> Result<()> {
        let conn = self.get_database_connection().await?;
        
        // Create schema
        conn.execute(&format!(
            "CREATE SCHEMA IF NOT EXISTS {}",
            context.data_boundary.database_schema
        )).await?;
        
        // Create tenant-specific tables
        conn.execute(&format!(
            "CREATE TABLE IF NOT EXISTS {}.state (
                key TEXT PRIMARY KEY,
                value JSONB,
                created_at TIMESTAMP,
                updated_at TIMESTAMP
            )",
            context.data_boundary.database_schema
        )).await?;
        
        Ok(())
    }
    
    /// Enforce isolation for operations
    pub fn enforce_isolation<T>(&self, tenant_id: &TenantId, operation: impl FnOnce() -> Result<T>) -> Result<T> {
        // Set thread-local tenant context
        CURRENT_TENANT.with(|t| t.set(Some(tenant_id.clone())));
        
        // Execute operation
        let result = operation();
        
        // Clear context
        CURRENT_TENANT.with(|t| t.set(None));
        
        result
    }
}
```

### Resource Quotas

Manage and enforce resource limits:

```rust
pub struct QuotaManager {
    quotas: Arc<RwLock<HashMap<TenantId, TenantQuota>>>,
    usage: Arc<RwLock<HashMap<TenantId, ResourceUsage>>>,
    enforcement: QuotaEnforcement,
}

#[derive(Debug, Clone)]
pub struct TenantQuota {
    pub tier: TenantTier,
    pub limits: QuotaLimits,
    pub custom_limits: HashMap<String, QuotaLimit>,
}

#[derive(Debug, Clone)]
pub struct QuotaLimits {
    pub max_storage_gb: f64,
    pub max_api_calls_per_month: u64,
    pub max_agents: usize,
    pub max_workflows: usize,
    pub max_concurrent_executions: usize,
    pub max_tokens_per_month: u64,
    pub max_artifacts: usize,
    pub max_sessions: usize,
    pub max_state_size_mb: f64,
}

#[derive(Debug, Clone)]
pub struct ResourceUsage {
    pub storage_bytes: AtomicU64,
    pub api_calls: AtomicU64,
    pub agent_count: AtomicUsize,
    pub workflow_count: AtomicUsize,
    pub concurrent_executions: AtomicUsize,
    pub tokens_used: AtomicU64,
    pub artifact_count: AtomicUsize,
    pub session_count: AtomicUsize,
    pub state_size_bytes: AtomicU64,
    pub period_start: SystemTime,
}

impl QuotaManager {
    /// Check if operation is within quota
    pub async fn check_quota(&self, tenant_id: &TenantId, resource: Resource, amount: u64) -> Result<bool> {
        let quota = self.get_quota(tenant_id).await?;
        let usage = self.get_usage(tenant_id).await?;
        
        match resource {
            Resource::Storage => {
                let current = usage.storage_bytes.load(Ordering::Relaxed);
                let limit = (quota.limits.max_storage_gb * 1024.0 * 1024.0 * 1024.0) as u64;
                Ok(current + amount <= limit)
            }
            Resource::ApiCalls => {
                let current = usage.api_calls.load(Ordering::Relaxed);
                Ok(current + amount <= quota.limits.max_api_calls_per_month)
            }
            Resource::Tokens => {
                let current = usage.tokens_used.load(Ordering::Relaxed);
                Ok(current + amount <= quota.limits.max_tokens_per_month)
            }
            _ => Ok(true)
        }
    }
    
    /// Consume quota
    pub async fn consume_quota(&self, tenant_id: &TenantId, resource: Resource, amount: u64) -> Result<()> {
        if !self.check_quota(tenant_id, resource.clone(), amount).await? {
            return Err(Error::QuotaExceeded(resource));
        }
        
        let usage = self.get_usage_mut(tenant_id).await?;
        
        match resource {
            Resource::Storage => {
                usage.storage_bytes.fetch_add(amount, Ordering::Relaxed);
            }
            Resource::ApiCalls => {
                usage.api_calls.fetch_add(amount, Ordering::Relaxed);
            }
            Resource::Tokens => {
                usage.tokens_used.fetch_add(amount, Ordering::Relaxed);
            }
            _ => {}
        }
        
        // Track for billing
        if let Some(billing) = &self.billing {
            billing.track_usage(tenant_id, resource, amount).await?;
        }
        
        Ok(())
    }
    
    /// Get quota summary
    pub async fn get_quota_summary(&self, tenant_id: &TenantId) -> Result<QuotaSummary> {
        let quota = self.get_quota(tenant_id).await?;
        let usage = self.get_usage(tenant_id).await?;
        
        Ok(QuotaSummary {
            tier: quota.tier.clone(),
            limits: quota.limits.clone(),
            usage: ResourceUsageSnapshot {
                storage_bytes: usage.storage_bytes.load(Ordering::Relaxed),
                api_calls: usage.api_calls.load(Ordering::Relaxed),
                tokens_used: usage.tokens_used.load(Ordering::Relaxed),
                // ... other fields
            },
            percentage_used: self.calculate_usage_percentages(&quota, &usage),
            reset_at: self.next_reset_time(&usage.period_start),
        })
    }
}
```

### Cross-Tenant Operations

Enable controlled cross-tenant access:

```rust
pub struct CrossTenantManager {
    permissions: Arc<RwLock<HashMap<(TenantId, TenantId), CrossTenantPermissions>>>,
    audit: Arc<dyn AuditLogger>,
}

#[derive(Debug, Clone)]
pub struct CrossTenantPermissions {
    pub source_tenant: TenantId,
    pub target_tenant: TenantId,
    pub permissions: HashSet<CrossTenantPermission>,
    pub expires_at: Option<SystemTime>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum CrossTenantPermission {
    ReadData,
    WriteData,
    ExecuteAgent,
    InvokeWorkflow,
    ShareArtifact,
    Custom(String),
}

impl CrossTenantManager {
    /// Grant cross-tenant permission
    pub async fn grant_permission(
        &self,
        source: &TenantId,
        target: &TenantId,
        permission: CrossTenantPermission,
        expires_at: Option<SystemTime>,
    ) -> Result<()> {
        let mut permissions = self.permissions.write().await;
        
        let key = (source.clone(), target.clone());
        let entry = permissions.entry(key).or_insert_with(|| CrossTenantPermissions {
            source_tenant: source.clone(),
            target_tenant: target.clone(),
            permissions: HashSet::new(),
            expires_at,
            metadata: HashMap::new(),
        });
        
        entry.permissions.insert(permission.clone());
        
        // Audit the grant
        self.audit.log(AuditEvent::CrossTenantGrant {
            source: source.clone(),
            target: target.clone(),
            permission,
            granted_at: SystemTime::now(),
            expires_at,
        }).await?;
        
        Ok(())
    }
    
    /// Check cross-tenant permission
    pub async fn check_permission(
        &self,
        source: &TenantId,
        target: &TenantId,
        permission: &CrossTenantPermission,
    ) -> Result<bool> {
        let permissions = self.permissions.read().await;
        
        if let Some(perms) = permissions.get(&(source.clone(), target.clone())) {
            // Check expiration
            if let Some(expires) = perms.expires_at {
                if SystemTime::now() > expires {
                    return Ok(false);
                }
            }
            
            return Ok(perms.permissions.contains(permission));
        }
        
        Ok(false)
    }
    
    /// Execute cross-tenant operation
    pub async fn execute_cross_tenant<T, F>(
        &self,
        source: &TenantId,
        target: &TenantId,
        permission: CrossTenantPermission,
        operation: F,
    ) -> Result<T>
    where
        F: FnOnce() -> Result<T>,
    {
        // Check permission
        if !self.check_permission(source, target, &permission).await? {
            return Err(Error::CrossTenantAccessDenied);
        }
        
        // Audit the operation
        self.audit.log(AuditEvent::CrossTenantOperation {
            source: source.clone(),
            target: target.clone(),
            permission,
            timestamp: SystemTime::now(),
        }).await?;
        
        // Execute with dual tenant context
        CROSS_TENANT_CONTEXT.with(|ctx| {
            ctx.set(Some(CrossTenantContext {
                source: source.clone(),
                target: target.clone(),
            }));
        });
        
        let result = operation();
        
        CROSS_TENANT_CONTEXT.with(|ctx| ctx.set(None));
        
        result
    }
}
```

### Tenant Metrics

Track and analyze tenant usage:

```rust
pub struct TenantMetrics {
    pub total_tenants: AtomicUsize,
    pub active_tenants: AtomicUsize,
    pub tenants_by_tier: Arc<RwLock<HashMap<TenantTier, usize>>>,
    pub total_api_calls: AtomicU64,
    pub total_storage_bytes: AtomicU64,
    pub total_tokens_used: AtomicU64,
    pub revenue_by_tenant: Arc<RwLock<HashMap<TenantId, f64>>>,
}

impl TenantMetrics {
    /// Get tenant analytics
    pub async fn get_analytics(&self, tenant_id: &TenantId) -> Result<TenantAnalytics> {
        let usage_history = self.get_usage_history(tenant_id, 30).await?;
        let cost_breakdown = self.calculate_cost_breakdown(tenant_id).await?;
        let growth_rate = self.calculate_growth_rate(&usage_history);
        
        Ok(TenantAnalytics {
            tenant_id: tenant_id.clone(),
            usage_history,
            cost_breakdown,
            growth_rate,
            health_score: self.calculate_health_score(tenant_id).await?,
            recommendations: self.generate_recommendations(tenant_id).await?,
        })
    }
    
    /// Export metrics for monitoring
    pub fn export_prometheus(&self) -> String {
        format!(
            "# HELP tenants_total Total number of tenants\n\
             # TYPE tenants_total gauge\n\
             tenants_total {}\n\
             # HELP tenants_active Active tenants\n\
             # TYPE tenants_active gauge\n\
             tenants_active {}\n\
             # HELP tenant_api_calls_total Total API calls across all tenants\n\
             # TYPE tenant_api_calls_total counter\n\
             tenant_api_calls_total {}\n\
             # HELP tenant_storage_bytes Total storage used by all tenants\n\
             # TYPE tenant_storage_bytes gauge\n\
             tenant_storage_bytes {}\n\
             # HELP tenant_tokens_used_total Total tokens used by all tenants\n\
             # TYPE tenant_tokens_used_total counter\n\
             tenant_tokens_used_total {}",
            self.total_tenants.load(Ordering::Relaxed),
            self.active_tenants.load(Ordering::Relaxed),
            self.total_api_calls.load(Ordering::Relaxed),
            self.total_storage_bytes.load(Ordering::Relaxed),
            self.total_tokens_used.load(Ordering::Relaxed),
        )
    }
}
```

### Billing Integration

Track usage for billing:

```rust
#[async_trait]
pub trait BillingProvider: Send + Sync {
    /// Track resource usage
    async fn track_usage(&self, tenant_id: &TenantId, resource: Resource, amount: u64) -> Result<()>;
    
    /// Get current billing period usage
    async fn get_billing_usage(&self, tenant_id: &TenantId) -> Result<BillingUsage>;
    
    /// Calculate costs
    async fn calculate_costs(&self, tenant_id: &TenantId) -> Result<CostBreakdown>;
    
    /// Generate invoice
    async fn generate_invoice(&self, tenant_id: &TenantId, period: BillingPeriod) -> Result<Invoice>;
}

pub struct StripeBillingProvider {
    client: StripeClient,
    price_map: HashMap<Resource, PriceConfig>,
}

#[async_trait]
impl BillingProvider for StripeBillingProvider {
    async fn track_usage(&self, tenant_id: &TenantId, resource: Resource, amount: u64) -> Result<()> {
        let subscription_id = self.get_subscription_id(tenant_id).await?;
        
        self.client.create_usage_record(UsageRecord {
            subscription_item: subscription_id,
            quantity: amount,
            timestamp: SystemTime::now(),
            action: UsageAction::Increment,
        }).await?;
        
        Ok(())
    }
    
    async fn calculate_costs(&self, tenant_id: &TenantId) -> Result<CostBreakdown> {
        let usage = self.get_billing_usage(tenant_id).await?;
        let mut costs = CostBreakdown::default();
        
        for (resource, amount) in usage.resources {
            if let Some(price_config) = self.price_map.get(&resource) {
                let cost = match price_config.pricing_model {
                    PricingModel::PerUnit => amount as f64 * price_config.unit_price,
                    PricingModel::Tiered(ref tiers) => self.calculate_tiered_price(amount, tiers),
                    PricingModel::Volume(ref volumes) => self.calculate_volume_price(amount, volumes),
                };
                
                costs.add_line_item(resource, amount, cost);
            }
        }
        
        Ok(costs)
    }
}
```

## Usage Examples

### Creating and Managing Tenants

```rust
use llmspell_tenancy::{TenantManager, CreateTenantRequest, TenantTier};

#[tokio::main]
async fn main() -> Result<()> {
    let config = TenantConfig {
        storage: StorageConfig::Postgres(postgres_config),
        isolation: IsolationConfig {
            encrypt_tenant_data: true,
            separate_databases: false,
            network_isolation: true,
        },
        quotas: QuotaConfig::default(),
        billing: Some(BillingConfig::Stripe(stripe_config)),
    };
    
    let manager = TenantManager::new(config).await?;
    
    // Create a new tenant
    let tenant = manager.create_tenant(CreateTenantRequest {
        name: "acme-corp".to_string(),
        display_name: "Acme Corporation".to_string(),
        tier: Some(TenantTier::Professional),
        metadata: hashmap!{
            "industry".to_string() => "technology".to_string(),
            "size".to_string() => "enterprise".to_string(),
        },
    }).await?;
    
    println!("Created tenant: {} ({})", tenant.display_name, tenant.id);
    
    Ok(())
}
```

### Enforcing Tenant Isolation

```rust
async fn isolated_operation(
    manager: &TenantManager,
    tenant_id: &TenantId,
) -> Result<()> {
    let isolation = manager.get_isolation_manager();
    
    // All operations within this block are isolated to the tenant
    isolation.enforce_isolation(tenant_id, || {
        // Database queries automatically use tenant schema
        let data = query_database("SELECT * FROM users")?;
        
        // File operations use tenant-specific paths
        let file_path = get_tenant_path("data.json");
        std::fs::write(file_path, data)?;
        
        // Cache operations use tenant namespace
        cache_set("key", "value")?;
        
        Ok(())
    })
}
```

### Managing Resource Quotas

```rust
async fn check_and_consume_quota(
    quota_manager: &QuotaManager,
    tenant_id: &TenantId,
) -> Result<()> {
    // Check if tenant has quota for API call
    if !quota_manager.check_quota(tenant_id, Resource::ApiCalls, 1).await? {
        return Err(Error::QuotaExceeded(Resource::ApiCalls));
    }
    
    // Execute operation
    let result = expensive_operation().await?;
    
    // Consume quota after successful operation
    quota_manager.consume_quota(tenant_id, Resource::ApiCalls, 1).await?;
    quota_manager.consume_quota(tenant_id, Resource::Tokens, result.tokens_used).await?;
    
    // Get quota summary
    let summary = quota_manager.get_quota_summary(tenant_id).await?;
    println!("API Calls: {}/{} ({:.1}%)",
        summary.usage.api_calls,
        summary.limits.max_api_calls_per_month,
        summary.percentage_used.api_calls
    );
    
    Ok(())
}
```

### Cross-Tenant Operations

```rust
async fn share_data_between_tenants(
    cross_tenant: &CrossTenantManager,
    source: &TenantId,
    target: &TenantId,
) -> Result<()> {
    // Grant permission
    cross_tenant.grant_permission(
        source,
        target,
        CrossTenantPermission::ReadData,
        Some(SystemTime::now() + Duration::from_days(7)),
    ).await?;
    
    // Execute cross-tenant operation
    let shared_data = cross_tenant.execute_cross_tenant(
        source,
        target,
        CrossTenantPermission::ReadData,
        || {
            // This runs with access to both tenant contexts
            let source_data = read_tenant_data(source)?;
            let filtered = filter_sensitive_data(source_data);
            write_tenant_data(target, filtered)?;
            Ok(filtered)
        },
    ).await?;
    
    println!("Shared {} records from {} to {}", 
        shared_data.len(), source, target);
    
    Ok(())
}
```

### Tenant Analytics

```rust
async fn analyze_tenant_usage(
    metrics: &TenantMetrics,
    tenant_id: &TenantId,
) -> Result<()> {
    let analytics = metrics.get_analytics(tenant_id).await?;
    
    println!("Tenant Analytics for {}", tenant_id);
    println!("==========================================");
    println!("Health Score: {}/100", analytics.health_score);
    println!("Growth Rate: {:.1}% per month", analytics.growth_rate);
    
    println!("\nResource Usage (Last 30 days):");
    for (day, usage) in analytics.usage_history {
        println!("  {}: {} API calls, {} tokens, {} GB storage",
            day, usage.api_calls, usage.tokens, usage.storage_gb);
    }
    
    println!("\nCost Breakdown:");
    for (resource, cost) in analytics.cost_breakdown {
        println!("  {}: ${:.2}", resource, cost);
    }
    
    println!("\nRecommendations:");
    for recommendation in analytics.recommendations {
        println!("  - {}", recommendation);
    }
    
    Ok(())
}
```

### Billing Integration

```rust
async fn process_tenant_billing(
    billing: &dyn BillingProvider,
    tenant_id: &TenantId,
) -> Result<()> {
    // Get current usage
    let usage = billing.get_billing_usage(tenant_id).await?;
    
    println!("Current Billing Period Usage:");
    for (resource, amount) in &usage.resources {
        println!("  {:?}: {}", resource, amount);
    }
    
    // Calculate costs
    let costs = billing.calculate_costs(tenant_id).await?;
    
    println!("\nEstimated Costs:");
    println!("  Subtotal: ${:.2}", costs.subtotal);
    println!("  Tax: ${:.2}", costs.tax);
    println!("  Total: ${:.2}", costs.total);
    
    // Generate invoice at end of period
    if is_end_of_billing_period() {
        let invoice = billing.generate_invoice(
            tenant_id,
            BillingPeriod::current(),
        ).await?;
        
        send_invoice_to_customer(invoice).await?;
    }
    
    Ok(())
}
```

## Testing

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_tenant_isolation() {
        let manager = TenantManager::new(test_config()).await.unwrap();
        
        // Create two tenants
        let tenant_a = manager.create_tenant(CreateTenantRequest {
            name: "tenant-a".to_string(),
            ..Default::default()
        }).await.unwrap();
        
        let tenant_b = manager.create_tenant(CreateTenantRequest {
            name: "tenant-b".to_string(),
            ..Default::default()
        }).await.unwrap();
        
        // Write data for tenant A
        let isolation = manager.get_isolation_manager();
        isolation.enforce_isolation(&tenant_a.id, || {
            write_data("key", "value-a")
        }).unwrap();
        
        // Try to read from tenant B context - should not see tenant A's data
        let result = isolation.enforce_isolation(&tenant_b.id, || {
            read_data("key")
        }).unwrap();
        
        assert_eq!(result, None);
    }
    
    #[tokio::test]
    async fn test_quota_enforcement() {
        let manager = QuotaManager::new(test_quota_config());
        let tenant_id = TenantId(Uuid::new_v4());
        
        // Set low quota for testing
        manager.set_quota(&tenant_id, TenantQuota {
            tier: TenantTier::Free,
            limits: QuotaLimits {
                max_api_calls_per_month: 10,
                ..Default::default()
            },
            custom_limits: HashMap::new(),
        }).await.unwrap();
        
        // Consume quota up to limit
        for _ in 0..10 {
            assert!(manager.consume_quota(&tenant_id, Resource::ApiCalls, 1)
                .await.is_ok());
        }
        
        // Next call should fail
        assert!(manager.consume_quota(&tenant_id, Resource::ApiCalls, 1)
            .await.is_err());
    }
}
```

## Performance Considerations

1. **Lazy Loading**: Tenant contexts loaded on demand
2. **Caching**: Frequently accessed tenant data cached
3. **Batch Operations**: Quota updates batched
4. **Connection Pooling**: Per-tenant connection pools
5. **Async I/O**: All operations non-blocking

## Security Guarantees

- **Complete Isolation**: No data leakage between tenants
- **Encrypted Storage**: Per-tenant encryption keys
- **Network Isolation**: Tenant-specific network policies
- **Audit Trail**: All cross-tenant operations logged
- **Resource Limits**: Prevent resource exhaustion

## Related Documentation

- [llmspell-security](llmspell-security.md) - Security framework
- [llmspell-storage](llmspell-storage.md) - Multi-tenant storage
- [llmspell-rag](llmspell-rag.md) - Multi-tenant RAG