//! Row-Level Security (RLS) style policies for vector operations

use anyhow::Result;
use async_trait::async_trait;
use llmspell_state_traits::StateScope;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;

use crate::audit::{AuditEvent, AuditLogger};

/// Security decision for an operation
#[derive(Debug, Clone, PartialEq)]
pub enum AccessDecision {
    /// Access allowed
    Allow,
    /// Access denied with reason
    Deny(String),
    /// Access allowed with filters applied
    AllowWithFilters(Vec<SecurityFilter>),
}

/// Security filter to apply to queries
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SecurityFilter {
    /// Field to filter on
    pub field: String,
    /// Allowed values
    pub allowed_values: HashSet<String>,
    /// Whether to exclude these values instead
    pub exclude: bool,
}

/// Operation context for security evaluation
#[derive(Debug, Clone)]
pub struct OperationContext {
    /// User or service principal
    pub principal: String,

    /// Operation type (insert, search, delete, etc.)
    pub operation: String,

    /// Target tenant ID
    pub tenant_id: Option<String>,

    /// Resource scope
    pub scope: StateScope,

    /// Additional metadata
    pub metadata: HashMap<String, String>,

    /// Source IP address
    pub source_ip: Option<String>,
}

/// Security policy trait
#[async_trait]
pub trait SecurityPolicy: Send + Sync {
    /// Evaluate access for an operation
    async fn evaluate(&self, context: &OperationContext) -> AccessDecision;

    /// Get policy name
    fn name(&self) -> &str;

    /// Get policy priority (higher = evaluated first)
    fn priority(&self) -> i32 {
        0
    }
}

/// Vector-specific access policy
#[derive(Debug, Clone)]
pub struct VectorAccessPolicy {
    /// Policy name
    name: String,

    /// Allowed operations per principal
    allowed_operations: HashMap<String, HashSet<String>>,

    /// Tenant access mapping
    tenant_access: HashMap<String, HashSet<String>>,

    /// Rate limits per principal
    rate_limits: HashMap<String, u32>,

    /// Global deny list
    deny_list: HashSet<String>,
}

impl VectorAccessPolicy {
    /// Create a new vector access policy
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            allowed_operations: HashMap::new(),
            tenant_access: HashMap::new(),
            rate_limits: HashMap::new(),
            deny_list: HashSet::new(),
        }
    }

    /// Grant operation access to a principal
    pub fn grant_operation(
        mut self,
        principal: impl Into<String>,
        operation: impl Into<String>,
    ) -> Self {
        self.allowed_operations
            .entry(principal.into())
            .or_default()
            .insert(operation.into());
        self
    }

    /// Grant tenant access to a principal
    pub fn grant_tenant_access(
        mut self,
        principal: impl Into<String>,
        tenant_id: impl Into<String>,
    ) -> Self {
        self.tenant_access
            .entry(principal.into())
            .or_default()
            .insert(tenant_id.into());
        self
    }

    /// Set rate limit for a principal
    pub fn set_rate_limit(mut self, principal: impl Into<String>, limit: u32) -> Self {
        self.rate_limits.insert(principal.into(), limit);
        self
    }

    /// Add principal to deny list
    pub fn deny_principal(mut self, principal: impl Into<String>) -> Self {
        self.deny_list.insert(principal.into());
        self
    }
}

#[async_trait]
impl SecurityPolicy for VectorAccessPolicy {
    async fn evaluate(&self, context: &OperationContext) -> AccessDecision {
        // Check deny list first
        if self.deny_list.contains(&context.principal) {
            return AccessDecision::Deny("Principal is denied access".to_string());
        }

        // Check operation permission
        if let Some(allowed_ops) = self.allowed_operations.get(&context.principal) {
            if !allowed_ops.contains(&context.operation) {
                return AccessDecision::Deny(format!(
                    "Operation '{}' not allowed for principal '{}'",
                    context.operation, context.principal
                ));
            }
        } else {
            // No explicit permissions
            return AccessDecision::Deny(format!(
                "No permissions configured for principal '{}'",
                context.principal
            ));
        }

        // Check tenant access if applicable
        if let Some(tenant_id) = &context.tenant_id {
            if let Some(allowed_tenants) = self.tenant_access.get(&context.principal) {
                if !allowed_tenants.contains(tenant_id) && !allowed_tenants.contains("*") {
                    return AccessDecision::Deny(format!(
                        "Access to tenant '{}' denied for principal '{}'",
                        tenant_id, context.principal
                    ));
                }
            } else {
                return AccessDecision::Deny(format!(
                    "No tenant access configured for principal '{}'",
                    context.principal
                ));
            }
        }

        // Apply filters if needed
        let mut filters = Vec::new();

        // Add tenant filter if not accessing all tenants
        if let Some(_tenant_id) = &context.tenant_id {
            if let Some(allowed_tenants) = self.tenant_access.get(&context.principal) {
                if !allowed_tenants.contains("*") {
                    filters.push(SecurityFilter {
                        field: "tenant_id".to_string(),
                        allowed_values: allowed_tenants.clone(),
                        exclude: false,
                    });
                }
            }
        }

        if filters.is_empty() {
            AccessDecision::Allow
        } else {
            AccessDecision::AllowWithFilters(filters)
        }
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn priority(&self) -> i32 {
        10 // Default priority
    }
}

/// Vector security manager
pub struct VectorSecurityManager {
    /// Security policies
    policies: Arc<RwLock<Vec<Arc<dyn SecurityPolicy>>>>,

    /// Audit logger
    audit_logger: Arc<AuditLogger>,

    /// Cache of recent decisions
    decision_cache: Arc<RwLock<HashMap<String, (AccessDecision, std::time::Instant)>>>,

    /// Cache TTL in seconds
    cache_ttl: u64,
}

impl VectorSecurityManager {
    /// Create a new security manager
    pub fn new(audit_logger: Arc<AuditLogger>) -> Self {
        Self {
            policies: Arc::new(RwLock::new(Vec::new())),
            audit_logger,
            decision_cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: 60, // 1 minute cache
        }
    }

    /// Add a security policy
    pub async fn add_policy(&self, policy: Arc<dyn SecurityPolicy>) {
        let mut policies = self.policies.write().await;
        policies.push(policy);

        // Sort by priority (descending)
        policies.sort_by_key(|p| -p.priority());
    }

    /// Validate operation
    pub async fn validate_operation(&self, context: &OperationContext) -> Result<AccessDecision> {
        // Create cache key
        let cache_key = format!(
            "{}:{}:{}:{:?}",
            context.principal,
            context.operation,
            context.tenant_id.as_ref().unwrap_or(&"".to_string()),
            context.scope
        );

        // Check cache
        {
            let cache = self.decision_cache.read().await;
            if let Some((decision, timestamp)) = cache.get(&cache_key) {
                if timestamp.elapsed().as_secs() < self.cache_ttl {
                    debug!("Using cached security decision for {}", cache_key);
                    return Ok(decision.clone());
                }
            }
        }

        // Evaluate policies
        let policies = self.policies.read().await;
        let mut final_decision = AccessDecision::Deny("No policies evaluated".to_string());
        let mut filters = Vec::new();

        for policy in policies.iter() {
            let decision = policy.evaluate(context).await;

            match decision {
                AccessDecision::Deny(reason) => {
                    // First deny wins
                    final_decision = AccessDecision::Deny(reason);
                    break;
                }
                AccessDecision::Allow => {
                    // Continue checking other policies
                    final_decision = AccessDecision::Allow;
                }
                AccessDecision::AllowWithFilters(mut policy_filters) => {
                    // Accumulate filters
                    filters.append(&mut policy_filters);
                    final_decision = AccessDecision::Allow;
                }
            }
        }

        // Apply accumulated filters
        if !filters.is_empty() && matches!(final_decision, AccessDecision::Allow) {
            final_decision = AccessDecision::AllowWithFilters(filters);
        }

        // Log audit event
        let event = match &final_decision {
            AccessDecision::Allow | AccessDecision::AllowWithFilters(_) => {
                AuditEvent::AccessGranted {
                    principal: context.principal.clone(),
                    operation: context.operation.clone(),
                    resource: context.tenant_id.clone().unwrap_or_default(),
                    metadata: context.metadata.clone(),
                }
            }
            AccessDecision::Deny(reason) => AuditEvent::AccessDenied {
                principal: context.principal.clone(),
                operation: context.operation.clone(),
                resource: context.tenant_id.clone().unwrap_or_default(),
                reason: reason.clone(),
                metadata: context.metadata.clone(),
            },
        };

        self.audit_logger.log(event).await?;

        // Cache decision
        {
            let mut cache = self.decision_cache.write().await;
            cache.insert(
                cache_key,
                (final_decision.clone(), std::time::Instant::now()),
            );

            // Clean old entries
            cache.retain(|_, (_, timestamp)| timestamp.elapsed().as_secs() < self.cache_ttl * 2);
        }

        Ok(final_decision)
    }

    /// Apply filters to metadata
    pub fn apply_filters(
        metadata: &HashMap<String, serde_json::Value>,
        filters: &[SecurityFilter],
    ) -> bool {
        for filter in filters {
            if let Some(value) = metadata.get(&filter.field) {
                if let Some(str_value) = value.as_str() {
                    let matches = filter.allowed_values.contains(str_value);

                    // If exclude mode and matches, deny
                    // If include mode and doesn't match, deny
                    if filter.exclude && matches || !filter.exclude && !matches {
                        return false;
                    }
                }
            } else if !filter.exclude {
                // Field doesn't exist and we're in include mode
                return false;
            }
        }

        true
    }

    /// Clear decision cache
    pub async fn clear_cache(&self) {
        self.decision_cache.write().await.clear();
    }
}

/// Default security policies
pub mod defaults {
    use super::*;

    /// Create admin policy with full access
    pub fn admin_policy() -> Arc<dyn SecurityPolicy> {
        Arc::new(
            VectorAccessPolicy::new("admin-policy")
                .grant_operation("admin", "insert")
                .grant_operation("admin", "search")
                .grant_operation("admin", "delete")
                .grant_operation("admin", "update")
                .grant_tenant_access("admin", "*"),
        )
    }

    /// Create read-only policy
    pub fn readonly_policy(principal: &str, tenant_id: &str) -> Arc<dyn SecurityPolicy> {
        Arc::new(
            VectorAccessPolicy::new(format!("readonly-{}", principal))
                .grant_operation(principal, "search")
                .grant_tenant_access(principal, tenant_id),
        )
    }

    /// Create tenant-scoped policy
    pub fn tenant_policy(principal: &str, tenant_id: &str) -> Arc<dyn SecurityPolicy> {
        Arc::new(
            VectorAccessPolicy::new(format!("tenant-{}", principal))
                .grant_operation(principal, "insert")
                .grant_operation(principal, "search")
                .grant_operation(principal, "update")
                .grant_operation(principal, "delete")
                .grant_tenant_access(principal, tenant_id),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_access_policy() {
        let policy = VectorAccessPolicy::new("test-policy")
            .grant_operation("user1", "search")
            .grant_tenant_access("user1", "tenant1");

        let context = OperationContext {
            principal: "user1".to_string(),
            operation: "search".to_string(),
            tenant_id: Some("tenant1".to_string()),
            scope: StateScope::Global,
            metadata: HashMap::new(),
            source_ip: None,
        };

        let decision = policy.evaluate(&context).await;
        assert!(matches!(decision, AccessDecision::Allow | AccessDecision::AllowWithFilters(_)));
    }

    #[tokio::test]
    async fn test_deny_unauthorized_operation() {
        let policy = VectorAccessPolicy::new("test-policy")
            .grant_operation("user1", "search")
            .grant_tenant_access("user1", "tenant1");

        let context = OperationContext {
            principal: "user1".to_string(),
            operation: "delete".to_string(), // Not granted
            tenant_id: Some("tenant1".to_string()),
            scope: StateScope::Global,
            metadata: HashMap::new(),
            source_ip: None,
        };

        let decision = policy.evaluate(&context).await;
        assert!(matches!(decision, AccessDecision::Deny(_)));
    }

    #[tokio::test]
    async fn test_cross_tenant_access_denied() {
        let policy = VectorAccessPolicy::new("test-policy")
            .grant_operation("user1", "search")
            .grant_tenant_access("user1", "tenant1");

        let context = OperationContext {
            principal: "user1".to_string(),
            operation: "search".to_string(),
            tenant_id: Some("tenant2".to_string()), // Different tenant
            scope: StateScope::Global,
            metadata: HashMap::new(),
            source_ip: None,
        };

        let decision = policy.evaluate(&context).await;
        assert!(matches!(decision, AccessDecision::Deny(_)));
    }
}
