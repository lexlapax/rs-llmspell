# llmspell-security

Comprehensive security framework for Rs-LLMSpell with multi-tenant isolation, access control, and sandboxing.

## Features

### Multi-Tenant Access Control (Phase 8)
- **Policy-Based Authorization**: Fine-grained access control with tenant isolation
- **Row-Level Security**: Filter data at the record level based on tenant context
- **Resource-Based Permissions**: Control access to tools, workflows, and data by tenant
- **Security Context Propagation**: Automatic security context inheritance across operations

### Advanced Sandboxing
- **File System Isolation**: Chroot jails and path restrictions with configurable boundaries
- **Network Sandboxing**: Granular network access control with allowlist/denylist policies
- **Resource Monitoring**: CPU, memory, and I/O limits with real-time enforcement
- **Process Isolation**: Secure execution environments with capability dropping

## Usage

### Multi-Tenant Access Control
```rust
use llmspell_security::{
    AccessControlPolicy, SecurityContext, SecurityFilter, AccessDecision
};
use llmspell_state_traits::StateScope;

// Define tenant-specific access policy
struct TenantPolicy {
    tenant_id: String,
}

#[async_trait]
impl AccessControlPolicy for TenantPolicy {
    async fn authorize(&self, context: &SecurityContext) -> Result<AccessDecision, SecurityError> {
        // Check if user belongs to this tenant
        if context.tenant_id() == Some(&self.tenant_id) {
            Ok(AccessDecision::AllowWithFilters(vec![
                SecurityFilter::include("tenant_id", &self.tenant_id)
            ]))
        } else {
            Ok(AccessDecision::Deny("Cross-tenant access denied".into()))
        }
    }
}

// Apply policy to operations
let policy = TenantPolicy { tenant_id: "company-123".to_string() };
let context = SecurityContext::new()
    .with_tenant("company-123")
    .with_scope(StateScope::Custom("tenant:company-123".to_string()));

let decision = policy.authorize(&context).await?;
match decision {
    AccessDecision::Allow => { /* Proceed with full access */ },
    AccessDecision::AllowWithFilters(filters) => {
        // Apply row-level security filters
        for filter in filters {
            query = query.with_filter(filter);
        }
    },
    AccessDecision::Deny(reason) => return Err(SecurityError::AccessDenied(reason)),
}
```

### Advanced Sandboxing
```rust
use llmspell_security::{IntegratedSandbox, SandboxContext};
use std::time::Duration;

// Create comprehensive sandbox with multi-layered security
let sandbox_config = SandboxContext::builder()
    .with_file_permissions(vec![
        ("/tmp".to_string(), vec!["read".to_string(), "write".to_string()]),
        ("/data".to_string(), vec!["read".to_string()]),
    ])
    .with_network_policy(NetworkPolicy::AllowList(vec![
        "api.openai.com".to_string(),
        "api.anthropic.com".to_string(),
    ]))
    .with_resource_limits(ResourceLimits {
        max_memory: 512 * 1024 * 1024,  // 512MB
        max_cpu_time: Duration::from_secs(300),  // 5 minutes
        max_file_size: 100 * 1024 * 1024,  // 100MB
    })
    .with_process_isolation(true)
    .build();

let sandbox = IntegratedSandbox::new(sandbox_config);

// Execute with comprehensive security
sandbox.execute_with_monitoring(untrusted_code, |event| {
    match event {
        SecurityEvent::ResourceLimitApproached(limit) => {
            log::warn!("Resource limit {} at 80%", limit);
        },
        SecurityEvent::UnauthorizedAccess(attempt) => {
            log::error!("Blocked access attempt: {}", attempt);
        },
    }
}).await?;
```

### Security Context Propagation
```rust
use llmspell_security::{SecurityContext, with_security_context};

// Automatically propagate security context across async operations
let tenant_context = SecurityContext::new()
    .with_tenant("company-123")
    .with_user("user-456")
    .with_permissions(vec!["read:documents", "write:own_documents"]);

// Context is automatically available in nested operations
with_security_context(tenant_context, || async {
    // All operations within this block inherit the security context
    let documents = document_service.list_documents().await?;  // Automatically filtered
    let result = ai_service.process_document(document).await?;  // Security validated
    Ok(result)
}).await?;
```

## Dependencies
- `llmspell-core` - Security trait definitions and error types
- `llmspell-state-traits` - StateScope integration for tenant isolation
- `llmspell-hooks` - Security event monitoring and audit logging
- `tokio` - Async runtime and security context propagation
- `tracing` - Structured logging with security correlation
- Platform-specific sandboxing libraries (seccomp, AppArmor, etc.)

## Phase 8 Architecture
```
llmspell-security
├── access_control/
│   ├── policies.rs         # AccessControlPolicy trait and implementations
│   ├── context.rs          # SecurityContext with tenant/user information
│   └── mod.rs              # Access control module exports
├── sandbox/
│   ├── file_sandbox.rs     # File system isolation and chroot jails
│   ├── network_sandbox.rs  # Network access control policies
│   ├── resource_monitor.rs # CPU, memory, I/O limit enforcement
│   └── mod.rs              # IntegratedSandbox implementation
├── audit.rs                # Security audit logging and compliance
└── lib.rs                  # Security framework integration
```

## Security Guarantees
- **Tenant Isolation**: Complete separation of tenant data with zero cross-tenant access
- **Resource Boundaries**: Enforced limits prevent resource exhaustion attacks
- **Audit Compliance**: Complete audit trail of all security decisions and violations
- **Defense in Depth**: Multiple security layers (authentication, authorization, sandboxing)
- **Zero Trust**: All operations validated regardless of source or previous authorization