# Security & Multi-Tenancy Integration

**Thematic guide to llmspell's security architecture and tenant isolation**

🔗 **Quick Links**: [llmspell-security](llmspell-security.md) | [llmspell-tenancy](llmspell-tenancy.md) | [Crate Index](crate-index.md)

---

## Overview

llmspell provides layered security with sandboxing, access control, and multi-tenant isolation. Phase 8 introduced comprehensive multi-tenancy support, enabling safe deployment in SaaS environments.

**Core Components**:
- **llmspell-security**: Access control, input validation, sandboxing, audit logging
- **llmspell-tenancy**: Tenant management, resource quotas, data isolation

---

## Security Architecture

```
Request
    ↓
[llmspell-security]
    ├─→ Input Validation (SSRF, path traversal, injection)
    ├─→ Authentication (API keys, OAuth)
    ├─→ Authorization (RBAC policies)
    ├─→ Sandboxing (script execution limits)
    └─→ Audit Logging (security events)
    ↓
[llmspell-tenancy]
    ├─→ Tenant Identification
    ├─→ Resource Quota Enforcement
    ├─→ Data Isolation (storage, memory, state)
    └─→ Cross-Tenant Prevention
    ↓
Execution
```

---

## Security Framework (llmspell-security)

### Input Validation

**SSRF Protection**:
```rust
use llmspell_security::validation::validate_url;

// Prevent Server-Side Request Forgery
let url = validate_url(&user_provided_url)?;
// Blocks: localhost, 127.0.0.1, 169.254.0.0/16, 10.0.0.0/8, 192.168.0.0/16

// Whitelist approach
let url = validate_url_with_whitelist(
    &user_provided_url,
    whitelist: vec!["https://api.example.com"]
)?;
```

**Path Traversal Prevention**:
```rust
use llmspell_security::validation::validate_path;

// Prevent directory traversal
let safe_path = validate_path(&user_provided_path, base_dir: "/var/lib/llmspell")?;
// Blocks: ../, /etc/passwd, symlink attacks

// Example
validate_path("../../etc/passwd", "/var/lib/llmspell")?;  // Error
validate_path("data/file.txt", "/var/lib/llmspell")?;     // OK: /var/lib/llmspell/data/file.txt
```

**Input Sanitization**:
```rust
use llmspell_security::validation::sanitize_input;

// Remove dangerous characters/patterns
let safe_input = sanitize_input(
    &user_input,
    options: SanitizeOptions {
        allow_html: false,
        allow_scripts: false,
        max_length: Some(10000),
    }
)?;
```

### Access Control (RBAC)

**Policy Definition**:
```rust
use llmspell_security::{AccessPolicy, Permission, Resource, Action};

let policy = AccessPolicy::builder()
    .allow(Permission {
        resource: Resource::Tool("web_search".into()),
        action: Action::Execute,
        conditions: vec![
            Condition::RateLimit { max_per_hour: 100 },
            Condition::TimeWindow { start: "09:00", end: "17:00" },
        ],
    })
    .deny(Permission {
        resource: Resource::Tool("file_delete".into()),
        action: Action::Execute,
        conditions: vec![],
    })
    .build();
```

**Permission Checking**:
```rust
use llmspell_security::SecurityContext;

let security_ctx = SecurityContext::new(
    user_id: "user_123",
    policies: vec![policy],
);

// Check permission before execution
if security_ctx.has_permission(Resource::Tool("web_search".into()), Action::Execute) {
    // Execute tool
} else {
    return Err(SecurityError::PermissionDenied);
}
```

**Role-Based Policies**:
```rust
let admin_policy = AccessPolicy::admin();     // Full access
let user_policy = AccessPolicy::user();       // Standard user
let readonly_policy = AccessPolicy::readonly(); // Read-only

// Assign to security context
let ctx = SecurityContext::with_role(user_id, Role::User);
```

### Sandboxing

**Script Execution Limits**:
```rust
use llmspell_security::sandbox::{Sandbox, SandboxConfig};

let sandbox = Sandbox::new(SandboxConfig {
    max_execution_time: Duration::from_secs(30),
    max_memory_mb: 256,
    max_cpu_percent: 50,
    network_access: false,
    file_system_access: FileSystemAccess::ReadOnly("/var/lib/llmspell/scripts"),
})?;

// Execute script in sandbox
let result = sandbox.execute(script_code).await?;
```

**Resource Limits**:
```rust
pub struct ResourceLimits {
    pub max_execution_time: Duration,
    pub max_memory_mb: usize,
    pub max_cpu_percent: u8,
    pub max_file_size_mb: usize,
    pub max_open_files: usize,
    pub max_network_requests: usize,
}

// Apply limits
let limits = ResourceLimits::default();
sandbox.set_limits(limits)?;
```

**Capability-Based Security**:
```rust
pub enum Capability {
    FileRead,
    FileWrite,
    NetworkAccess,
    ToolExecution,
    StateAccess,
}

let sandbox = Sandbox::with_capabilities(vec![
    Capability::FileRead,
    Capability::ToolExecution,
]);

// Script can only read files and execute tools
```

### Audit Logging

**Security Events**:
```rust
use llmspell_security::audit::{AuditLogger, SecurityEvent};

let logger = AuditLogger::new()?;

// Log authentication
logger.log(SecurityEvent::Authentication {
    user_id: "user_123",
    method: "api_key",
    success: true,
    ip_address: "192.168.1.100",
    timestamp: Utc::now(),
}).await?;

// Log authorization check
logger.log(SecurityEvent::Authorization {
    user_id: "user_123",
    resource: "tool:web_search",
    action: "execute",
    granted: true,
    policy_id: Some("policy_456"),
}).await?;

// Log security violation
logger.log(SecurityEvent::Violation {
    user_id: "user_123",
    violation_type: "path_traversal_attempt",
    details: "Attempted to access ../../etc/passwd",
    severity: Severity::High,
}).await?;
```

**Audit Queries**:
```rust
// Query audit log
let events = logger.query(
    user_id: Some("user_123"),
    event_type: Some(EventType::Violation),
    start_time: Utc::now() - Duration::days(7),
    end_time: Utc::now(),
).await?;

// Generate security report
let report = logger.generate_report(
    time_range: Duration::days(30)
).await?;

println!("Total events: {}", report.total_events);
println!("Violations: {}", report.violations);
println!("Failed auths: {}", report.failed_authentications);
```

📚 **Full Details**: [llmspell-security.md](llmspell-security.md)

---

## Multi-Tenancy (llmspell-tenancy)

**Purpose**: Isolate data and resources across multiple tenants in SaaS deployments

### Tenant Management

**Tenant Creation**:
```rust
use llmspell_tenancy::{TenantManager, Tenant, TenantConfig};

let manager = TenantManager::new(storage)?;

let tenant = manager.create_tenant(TenantConfig {
    name: "Acme Corp",
    tier: TenantTier::Professional,
    quotas: ResourceQuotas {
        max_users: 100,
        max_storage_gb: 50,
        max_api_calls_per_day: 10_000,
        max_concurrent_sessions: 50,
    },
    features: vec![
        Feature::AdvancedRAG,
        Feature::CustomModels,
        Feature::APIAccess,
    ],
}).await?;

println!("Tenant ID: {}", tenant.id);
```

**Tenant Identification**:
```rust
// From API key
let tenant_id = manager.identify_tenant_by_api_key(&api_key).await?;

// From subdomain
let tenant_id = manager.identify_tenant_by_subdomain("acme.llmspell.com").await?;

// From JWT
let tenant_id = manager.identify_tenant_from_jwt(&jwt_token).await?;
```

### Data Isolation

**Storage Isolation**:
```rust
use llmspell_storage::TenantAwareStorage;

// All operations automatically scoped to tenant
let storage = TenantAwareStorage::new(
    base_storage,
    tenant_id: "tenant_123"
);

// Keys automatically prefixed: tenant_123:key
storage.set("key", value).await?;

// List only tenant's keys
let keys = storage.list_keys(None).await?;
```

**Memory Isolation**:
```rust
use llmspell_memory::TenantMemory;

let memory = TenantMemory::new(
    base_memory,
    tenant_id: "tenant_123"
);

// All memories scoped to tenant
memory.add_episodic("session_1", "Memory", metadata).await?;

// No cross-tenant access possible
let results = memory.query_episodic("query", None, 5).await?;
// Only returns memories from tenant_123
```

**State Isolation**:
```rust
use llmspell_state_persistence::TenantState;

let state = TenantState::new(
    base_state,
    tenant_id: "tenant_123"
);

// State operations scoped to tenant
state.write("config", config_value).await?;
```

### Resource Quotas

**Quota Definition**:
```rust
pub struct ResourceQuotas {
    pub max_users: usize,
    pub max_storage_gb: usize,
    pub max_api_calls_per_day: usize,
    pub max_concurrent_sessions: usize,
    pub max_vector_dimensions: usize,
    pub max_rag_documents: usize,
}
```

**Quota Enforcement**:
```rust
// Check quota before operation
manager.check_quota(tenant_id, QuotaType::ApiCalls).await?;

// Record usage
manager.record_usage(
    tenant_id,
    usage: Usage {
        api_calls: 1,
        storage_mb: 10,
        vector_operations: 5,
    }
).await?;

// Get current usage
let usage = manager.get_usage(tenant_id).await?;
let quotas = manager.get_quotas(tenant_id).await?;

if usage.api_calls >= quotas.max_api_calls_per_day {
    return Err(TenancyError::QuotaExceeded("api_calls"));
}
```

**Quota Alerts**:
```rust
// Set up alerts
manager.configure_alerts(
    tenant_id,
    alerts: vec![
        QuotaAlert {
            quota_type: QuotaType::ApiCalls,
            threshold_percent: 80,
            action: AlertAction::EmailAdmin,
        },
        QuotaAlert {
            quota_type: QuotaType::Storage,
            threshold_percent: 90,
            action: AlertAction::BlockWrites,
        },
    ]
).await?;
```

### Cross-Tenant Prevention

**Validation Pattern**:
```rust
async fn execute_tenant_operation(
    tenant_id: &str,
    resource_id: &str,
    operation: Operation,
) -> Result<()> {
    // 1. Validate resource belongs to tenant
    let resource = storage.get(resource_id).await?;

    if resource.tenant_id != tenant_id {
        return Err(TenancyError::CrossTenantAccessDenied {
            requested_by: tenant_id.to_string(),
            resource_owner: resource.tenant_id.clone(),
        });
    }

    // 2. Perform operation
    operation.execute(resource).await?;

    Ok(())
}
```

**Double-Check Pattern**:
```rust
// Primary check: Tenant-scoped collection
let collection = format!("tenant_{}:docs", tenant_id);

// Secondary check: Metadata filter
let filter = MetadataFilter::builder()
    .eq("tenant_id", tenant_id)
    .build();

// Query with both checks
let results = storage.search_with_filter(&collection, query, k, Some(filter)).await?;

// Tertiary check: Post-filter validation
for result in results {
    assert_eq!(result.metadata.get("tenant_id"), Some(&tenant_id));
}
```

📚 **Full Details**: [llmspell-tenancy.md](llmspell-tenancy.md)

---

## Integration Patterns

### Secure Request Flow

```rust
use llmspell_security::SecurityContext;
use llmspell_tenancy::TenantContext;

pub struct RequestContext {
    pub security: SecurityContext,
    pub tenant: TenantContext,
}

async fn handle_request(
    request: Request,
    context: RequestContext,
) -> Result<Response> {
    // 1. Authenticate
    let user = authenticate(&request)?;

    // 2. Identify tenant
    let tenant_id = context.tenant.identify(&request).await?;

    // 3. Check quotas
    context.tenant.check_quotas(tenant_id).await?;

    // 4. Authorize
    if !context.security.is_authorized(&user, &request.resource, &request.action) {
        return Err(SecurityError::Unauthorized);
    }

    // 5. Validate input
    validate_input(&request.data)?;

    // 6. Execute with isolation
    let result = execute_isolated(tenant_id, request.data).await?;

    // 7. Audit log
    context.security.log_access(&user, &request, true).await?;

    // 8. Record usage
    context.tenant.record_usage(tenant_id, result.usage).await?;

    Ok(result.response)
}
```

### Execution Context Integration

```rust
use llmspell_core::ExecutionContext;

// Attach security and tenant context to execution
let exec_context = ExecutionContext::builder()
    .with_session_id("session_123")
    .with_security_context(security_ctx)
    .with_tenant_id(tenant_id)
    .build();

// All components receive security context
agent.execute(input, exec_context).await?;
```

---

## Testing Patterns

### Security Testing

```rust
#[tokio::test]
async fn test_ssrf_prevention() {
    let result = validate_url("http://localhost/admin");
    assert!(result.is_err());

    let result = validate_url("http://169.254.169.254/metadata");
    assert!(result.is_err());

    let result = validate_url("https://api.example.com/data");
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_path_traversal_prevention() {
    let base = "/var/lib/llmspell";

    assert!(validate_path("../../etc/passwd", base).is_err());
    assert!(validate_path("/etc/passwd", base).is_err());
    assert!(validate_path("data/file.txt", base).is_ok());
}
```

### Multi-Tenancy Testing

```rust
#[tokio::test]
async fn test_tenant_isolation() {
    let storage = TenantAwareStorage::new(base, "tenant_a");
    storage.set("key", b"value_a".to_vec()).await.unwrap();

    let storage_b = TenantAwareStorage::new(base, "tenant_b");
    let value = storage_b.get("key").await.unwrap();

    // tenant_b cannot see tenant_a's data
    assert!(value.is_none());
}

#[tokio::test]
async fn test_quota_enforcement() {
    let manager = TenantManager::new_with_quotas(ResourceQuotas {
        max_api_calls_per_day: 10,
        ..Default::default()
    });

    // First 10 calls succeed
    for _ in 0..10 {
        manager.check_quota(tenant_id, QuotaType::ApiCalls).await.unwrap();
        manager.record_usage(tenant_id, Usage { api_calls: 1, ..Default::default() }).await.unwrap();
    }

    // 11th call fails
    let result = manager.check_quota(tenant_id, QuotaType::ApiCalls).await;
    assert!(matches!(result, Err(TenancyError::QuotaExceeded(_))));
}
```

---

## Best Practices

### Security Layering

1. **Input Validation** (first line of defense)
   - Validate all user inputs
   - Sanitize before processing
   - Use whitelists over blacklists

2. **Authentication** (identity verification)
   - Use strong API key generation
   - Support OAuth/JWT
   - Implement rate limiting

3. **Authorization** (permission checks)
   - Check permissions before every operation
   - Use principle of least privilege
   - Audit all access attempts

4. **Sandboxing** (execution limits)
   - Limit resource usage
   - Restrict file system access
   - Control network access

5. **Audit Logging** (monitoring)
   - Log all security events
   - Monitor for anomalies
   - Retain logs per compliance requirements

### Multi-Tenancy Best Practices

1. **Always Use Tenant Context**
   - Never trust client-provided tenant IDs
   - Derive tenant from authenticated credentials
   - Validate tenant access on every operation

2. **Double-Check Isolation**
   - Use tenant-prefixed collections/namespaces
   - Add tenant_id to all metadata
   - Post-filter results to verify isolation

3. **Enforce Quotas Strictly**
   - Check quotas before operations
   - Use atomic operations for usage tracking
   - Provide clear error messages on quota exceeded

4. **Test Cross-Tenant Access**
   - Write tests attempting cross-tenant access
   - Verify all isolation mechanisms
   - Test quota boundaries

---

## Related Documentation

- **Detailed API**:
  - [llmspell-security.md](llmspell-security.md) - Security framework
  - [llmspell-tenancy.md](llmspell-tenancy.md) - Multi-tenancy

- **Other Guides**:
  - [core-traits.md](core-traits.md) - ExecutionContext integration
  - [storage-backends.md](storage-backends.md) - Tenant-aware storage
  - [memory-backends.md](memory-backends.md) - Tenant memory isolation

- **User Guides**:
  - [../../user-guide/security.md](../../user-guide/security.md) - Security concepts
  - [../../user-guide/deployment.md](../../user-guide/deployment.md) - Secure deployment

---

**Version**: 0.13.0 | **Phase**: 13b.18.1 | **Last Updated**: 2025-11-08
