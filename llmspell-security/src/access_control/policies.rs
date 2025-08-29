//! Row-Level Security (RLS) style policies for vector operations

use anyhow::Result;
use async_trait::async_trait;
use llmspell_state_traits::StateScope;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::debug;

use super::context::SecurityContext;
use crate::audit::{AuditEvent, AuditLogger};

/// Represents a security decision made by access control policies.
///
/// Used by security policies to communicate whether an operation should be
/// allowed, denied, or allowed with additional filtering constraints.
#[derive(Debug, Clone, PartialEq)]
pub enum AccessDecision {
    /// Access is allowed without restrictions.
    ///
    /// The operation can proceed normally with no additional constraints
    /// or filtering requirements.
    Allow,

    /// Access is denied with an explanatory reason.
    ///
    /// The operation should be blocked and the provided reason should be
    /// logged for audit purposes and potentially returned to the caller
    /// (depending on security requirements).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use llmspell_security::AccessDecision;
    ///
    /// let decision = AccessDecision::Deny("Insufficient permissions".to_string());
    /// ```
    Deny(String),

    /// Access is allowed but additional security filters must be applied.
    ///
    /// The operation can proceed but the provided filters must be applied
    /// to restrict the data that can be accessed. This enables row-level
    /// security where users can only see data they are authorized for.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use llmspell_security::{AccessDecision, SecurityFilter};
    /// use std::collections::HashSet;
    ///
    /// let filter = SecurityFilter {
    ///     field: "tenant_id".to_string(),
    ///     allowed_values: HashSet::from(["tenant-123".to_string()]),
    ///     exclude: false,
    /// };
    /// let decision = AccessDecision::AllowWithFilters(vec![filter]);
    /// ```
    AllowWithFilters(Vec<SecurityFilter>),
}

/// A security filter that restricts data access based on field values.
///
/// Security filters implement row-level security by constraining queries
/// to only return data matching specified field values. This enables
/// multi-tenant isolation and fine-grained access control.
///
/// # Examples
///
/// ```rust
/// use llmspell_security::SecurityFilter;
/// use std::collections::HashSet;
///
/// // Allow access only to data belonging to specific tenants
/// let tenant_filter = SecurityFilter {
///     field: "tenant_id".to_string(),
///     allowed_values: HashSet::from([
///         "tenant-123".to_string(),
///         "tenant-456".to_string(),
///     ]),
///     exclude: false,
/// };
///
/// // Exclude data with specific status values
/// let status_filter = SecurityFilter {
///     field: "status".to_string(),
///     allowed_values: HashSet::from(["deleted".to_string(), "archived".to_string()]),
///     exclude: true,
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SecurityFilter {
    /// The field name to filter on.
    ///
    /// This should correspond to a field in the data being queried,
    /// such as "tenant_id", "user_id", or "status".
    pub field: String,

    /// The set of values that are allowed (or disallowed if `exclude` is true).
    ///
    /// When `exclude` is false, only records with field values in this set
    /// will be returned. When `exclude` is true, records with field values
    /// in this set will be filtered out.
    pub allowed_values: HashSet<String>,

    /// Whether this is an inclusion filter (false) or exclusion filter (true).
    ///
    /// - `false`: Only include records where the field value is in `allowed_values`
    /// - `true`: Exclude records where the field value is in `allowed_values`
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

/// Core trait for implementing security policies in the access control system.
///
/// Security policies evaluate operation requests and determine whether they should
/// be allowed, denied, or allowed with additional filtering constraints. Policies
/// can be chained together with different priorities to create complex authorization
/// rules.
///
/// # Examples
///
/// ```rust
/// use llmspell_security::{SecurityPolicy, AccessDecision, OperationContext};
/// use async_trait::async_trait;
///
/// pub struct TenantIsolationPolicy;
///
/// #[async_trait]
/// impl SecurityPolicy for TenantIsolationPolicy {
///     async fn evaluate(&self, context: &OperationContext) -> AccessDecision {
///         if let Some(tenant_id) = &context.tenant_id {
///             if tenant_id.starts_with("authorized-") {
///                 AccessDecision::Allow
///             } else {
///                 AccessDecision::Deny("Unauthorized tenant".to_string())
///             }
///         } else {
///             AccessDecision::Deny("No tenant context provided".to_string())
///         }
///     }
///
///     fn name(&self) -> &str {
///         "tenant_isolation"
///     }
///
///     fn priority(&self) -> i32 {
///         100  // High priority for security-critical policies
///     }
/// }
/// ```
#[async_trait]
pub trait SecurityPolicy: Send + Sync {
    /// Evaluate whether an operation should be allowed.
    ///
    /// This is the core method that implements the security logic. It receives
    /// an operation context containing information about the requester, operation
    /// type, and target resources, then returns a decision about whether to allow
    /// or deny the operation.
    ///
    /// # Arguments
    ///
    /// * `context` - The operation context containing security-relevant information
    ///
    /// # Returns
    ///
    /// An `AccessDecision` indicating whether to allow, deny, or allow with filters.
    async fn evaluate(&self, context: &OperationContext) -> AccessDecision;

    /// Get the human-readable name of this policy.
    ///
    /// Used for logging, debugging, and policy management. Should be unique
    /// within a security manager to avoid confusion.
    fn name(&self) -> &str;

    /// Get the evaluation priority of this policy.
    ///
    /// Policies with higher priority values are evaluated first. This allows
    /// security-critical policies (like authentication) to run before less
    /// critical ones (like rate limiting).
    ///
    /// # Returns
    ///
    /// Priority value where higher numbers mean higher priority. Default is 0.
    fn priority(&self) -> i32 {
        0
    }
}

/// Enhanced access control policy trait with rich SecurityContext support.
///
/// This is the next-generation security policy interface that provides richer
/// context information and more sophisticated policy matching. It extends the
/// basic `SecurityPolicy` trait with additional metadata and version support
/// for more complex deployment scenarios.
///
/// # Examples
///
/// ```rust
/// use llmspell_security::{AccessControlPolicy, AccessDecision, SecurityContext};
/// use async_trait::async_trait;
/// use anyhow::Result;
///
/// pub struct ResourceBasedPolicy {
///     allowed_resources: Vec<String>,
/// }
///
/// #[async_trait]
/// impl AccessControlPolicy for ResourceBasedPolicy {
///     async fn evaluate_access(
///         &self,
///         security_context: &SecurityContext,
///         operation: &str,
///         resource: &str,
///     ) -> Result<AccessDecision> {
///         if self.allowed_resources.contains(&resource.to_string()) {
///             Ok(AccessDecision::Allow)
///         } else {
///             Ok(AccessDecision::Deny(
///                 format!("Access denied to resource: {}", resource)
///             ))
///         }
///     }
///
///     fn applies_to(&self, _context: &SecurityContext, operation: &str) -> bool {
///         operation.starts_with("resource_")
///     }
///
///     fn policy_id(&self) -> &str {
///         "resource_based_v1"
///     }
/// }
/// ```
#[async_trait]
pub trait AccessControlPolicy: Send + Sync {
    /// Evaluate access using enhanced security context and resource information.
    ///
    /// This method provides richer context than the basic `SecurityPolicy::evaluate`
    /// method, including structured security context and explicit resource identifiers.
    ///
    /// # Arguments
    ///
    /// * `security_context` - Rich security context with user, tenant, and metadata
    /// * `operation` - The operation being attempted (e.g., "read", "write", "delete")
    /// * `resource` - The resource identifier being accessed
    ///
    /// # Returns
    ///
    /// A `Result<AccessDecision>` indicating the policy's decision. The `Result`
    /// allows for error handling during policy evaluation.
    async fn evaluate_access(
        &self,
        security_context: &SecurityContext,
        operation: &str,
        resource: &str,
    ) -> Result<AccessDecision>;

    /// Check if this policy should be evaluated for the given context and operation.
    ///
    /// This method allows policies to opt out of evaluation for operations they
    /// don't care about, improving performance by reducing unnecessary evaluations.
    ///
    /// # Arguments
    ///
    /// * `security_context` - The security context for the operation
    /// * `operation` - The operation being attempted
    ///
    /// # Returns
    ///
    /// `true` if this policy should be evaluated, `false` to skip evaluation.
    fn applies_to(&self, security_context: &SecurityContext, operation: &str) -> bool;

    /// Get the unique identifier for this policy.
    ///
    /// Used for policy management, caching, and debugging. Should be unique
    /// across the entire policy set to avoid conflicts.
    fn policy_id(&self) -> &str;

    /// Get the version of this policy for cache invalidation.
    ///
    /// When policy logic changes, increment this version to invalidate any
    /// cached policy decisions. Useful in distributed deployments where
    /// policy evaluation results might be cached.
    ///
    /// # Returns
    ///
    /// Version number, defaults to 1.
    fn version(&self) -> u32 {
        1
    }

    /// Get the evaluation priority of this policy.
    ///
    /// Same semantics as `SecurityPolicy::priority()` - higher values are
    /// evaluated first.
    ///
    /// # Returns
    ///
    /// Priority value where higher numbers mean higher priority. Default is 0.
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

/// Tenant-aware access control policy using SecurityContext
#[derive(Debug, Clone)]
pub struct TenantAccessControlPolicy {
    /// Policy identifier
    policy_id: String,
    /// Default tenant for operations when none specified
    default_tenant: Option<String>,
    /// Admin roles that get global access
    admin_roles: HashSet<String>,
}

impl TenantAccessControlPolicy {
    /// Create a new tenant access control policy
    pub fn new(policy_id: impl Into<String>) -> Self {
        Self {
            policy_id: policy_id.into(),
            default_tenant: None,
            admin_roles: HashSet::from_iter(["admin".to_string(), "super_admin".to_string()]),
        }
    }

    /// Set default tenant for operations
    pub fn with_default_tenant(mut self, tenant_id: impl Into<String>) -> Self {
        self.default_tenant = Some(tenant_id.into());
        self
    }

    /// Add admin role
    pub fn with_admin_role(mut self, role: impl Into<String>) -> Self {
        self.admin_roles.insert(role.into());
        self
    }

    /// Check if user has admin access
    fn has_admin_access(&self, security_context: &SecurityContext) -> bool {
        security_context
            .roles
            .iter()
            .any(|role| self.admin_roles.contains(role))
    }

    /// Get effective tenant ID for the operation
    fn get_effective_tenant(&self, security_context: &SecurityContext) -> Option<String> {
        security_context
            .tenant_id
            .clone()
            .or_else(|| self.default_tenant.clone())
    }
}

#[async_trait]
impl AccessControlPolicy for TenantAccessControlPolicy {
    async fn evaluate_access(
        &self,
        security_context: &SecurityContext,
        _operation: &str,
        resource: &str,
    ) -> Result<AccessDecision> {
        // Validate context first
        security_context.validate()?;

        // Admin users get full access
        if self.has_admin_access(security_context) {
            return Ok(AccessDecision::Allow);
        }

        // Determine effective tenant
        let effective_tenant = self.get_effective_tenant(security_context);

        // For tenant-specific operations, enforce tenant isolation
        if resource.starts_with("tenant:") {
            let resource_tenant = resource.strip_prefix("tenant:").unwrap_or("");

            match &effective_tenant {
                Some(user_tenant) if user_tenant == resource_tenant => {
                    // User can access their own tenant's resources
                    Ok(AccessDecision::Allow)
                }
                Some(_) => {
                    // User trying to access different tenant's resources
                    Ok(AccessDecision::Deny(format!(
                        "Cross-tenant access denied: user tenant '{}' cannot access '{}'",
                        effective_tenant.unwrap(),
                        resource
                    )))
                }
                None => {
                    // No tenant context for tenant-specific resource
                    Ok(AccessDecision::Deny(
                        "Tenant context required for tenant-specific resources".to_string(),
                    ))
                }
            }
        } else {
            // Non-tenant resources - apply filters based on user's tenant
            if let Some(tenant_id) = effective_tenant {
                let filter = SecurityFilter {
                    field: "tenant_id".to_string(),
                    allowed_values: HashSet::from_iter([tenant_id]),
                    exclude: false,
                };
                Ok(AccessDecision::AllowWithFilters(vec![filter]))
            } else {
                // Global access for non-tenant operations
                Ok(AccessDecision::Allow)
            }
        }
    }

    fn applies_to(&self, security_context: &SecurityContext, _operation: &str) -> bool {
        // Policy applies to all requests with valid principals
        !security_context.principal.is_empty()
    }

    fn policy_id(&self) -> &str {
        &self.policy_id
    }

    fn priority(&self) -> i32 {
        100 // High priority for tenant isolation
    }
}

/// Enhanced security manager supporting both old and new policy types
pub struct EnhancedSecurityManager {
    /// Traditional security policies (for future compatibility)
    #[allow(dead_code)]
    policies: Arc<RwLock<Vec<Arc<dyn SecurityPolicy>>>>,
    /// Enhanced access control policies
    access_policies: Arc<RwLock<Vec<Arc<dyn AccessControlPolicy>>>>,
    /// Audit logger
    audit_logger: Arc<AuditLogger>,
    /// Decision cache
    decision_cache: Arc<RwLock<HashMap<String, (AccessDecision, std::time::Instant)>>>,
    /// Cache TTL
    cache_ttl: u64,
}

impl EnhancedSecurityManager {
    /// Create a new enhanced security manager
    pub fn new(audit_logger: Arc<AuditLogger>) -> Self {
        Self {
            policies: Arc::new(RwLock::new(Vec::new())),
            access_policies: Arc::new(RwLock::new(Vec::new())),
            audit_logger,
            decision_cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl: 60,
        }
    }

    /// Add an access control policy
    pub async fn add_access_policy(&self, policy: Arc<dyn AccessControlPolicy>) {
        let mut policies = self.access_policies.write().await;
        policies.push(policy);
        policies.sort_by_key(|p| -p.priority());
    }

    /// Evaluate access using SecurityContext
    pub async fn evaluate_access(
        &self,
        security_context: &SecurityContext,
        operation: &str,
        resource: &str,
    ) -> Result<AccessDecision> {
        let cache_key = format!(
            "{}:{}:{}:{}:{}",
            security_context.principal,
            security_context.tenant_id.as_deref().unwrap_or(""),
            operation,
            resource,
            security_context.roles.join(",")
        );

        // Check cache
        {
            let cache = self.decision_cache.read().await;
            if let Some((decision, timestamp)) = cache.get(&cache_key) {
                if timestamp.elapsed().as_secs() < self.cache_ttl {
                    return Ok(decision.clone());
                }
            }
        }

        let policies = self.access_policies.read().await;
        let mut final_decision = AccessDecision::Deny("No applicable policies".to_string());
        let mut filters = Vec::new();

        for policy in policies.iter() {
            if policy.applies_to(security_context, operation) {
                match policy
                    .evaluate_access(security_context, operation, resource)
                    .await?
                {
                    AccessDecision::Deny(reason) => {
                        final_decision = AccessDecision::Deny(reason);
                        break;
                    }
                    AccessDecision::Allow => {
                        final_decision = AccessDecision::Allow;
                    }
                    AccessDecision::AllowWithFilters(mut policy_filters) => {
                        filters.append(&mut policy_filters);
                        final_decision = AccessDecision::Allow;
                    }
                }
            }
        }

        if !filters.is_empty() && matches!(final_decision, AccessDecision::Allow) {
            final_decision = AccessDecision::AllowWithFilters(filters);
        }

        // Log audit event
        let event = match &final_decision {
            AccessDecision::Allow | AccessDecision::AllowWithFilters(_) => {
                AuditEvent::AccessGranted {
                    principal: security_context.principal.clone(),
                    operation: operation.to_string(),
                    resource: resource.to_string(),
                    metadata: security_context.attributes.clone(),
                }
            }
            AccessDecision::Deny(reason) => AuditEvent::AccessDenied {
                principal: security_context.principal.clone(),
                operation: operation.to_string(),
                resource: resource.to_string(),
                reason: reason.clone(),
                metadata: security_context.attributes.clone(),
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
            cache.retain(|_, (_, timestamp)| timestamp.elapsed().as_secs() < self.cache_ttl * 2);
        }

        Ok(final_decision)
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

    /// Create tenant-aware access control policy
    pub fn tenant_access_control_policy() -> Arc<dyn AccessControlPolicy> {
        Arc::new(TenantAccessControlPolicy::new("default-tenant-policy"))
    }

    /// Create admin access control policy
    pub fn admin_access_control_policy() -> Arc<dyn AccessControlPolicy> {
        Arc::new(
            TenantAccessControlPolicy::new("admin-access-policy")
                .with_admin_role("admin")
                .with_admin_role("super_admin"),
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
        assert!(matches!(
            decision,
            AccessDecision::Allow | AccessDecision::AllowWithFilters(_)
        ));
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

    #[tokio::test]
    async fn test_tenant_access_control_policy() {
        let policy = TenantAccessControlPolicy::new("test-tenant-policy");

        let context = SecurityContext::new("user1")
            .with_tenant_id("tenant1")
            .with_roles(vec!["user".to_string()]);

        // Test access to own tenant resource
        let decision = policy
            .evaluate_access(&context, "search", "tenant:tenant1")
            .await
            .unwrap();
        assert!(matches!(decision, AccessDecision::Allow));

        // Test access to different tenant resource
        let decision = policy
            .evaluate_access(&context, "search", "tenant:tenant2")
            .await
            .unwrap();
        assert!(matches!(decision, AccessDecision::Deny(_)));

        // Test admin access
        let admin_context = SecurityContext::new("admin")
            .with_tenant_id("tenant1")
            .with_roles(vec!["admin".to_string()]);

        let decision = policy
            .evaluate_access(&admin_context, "delete", "tenant:tenant2")
            .await
            .unwrap();
        assert!(matches!(decision, AccessDecision::Allow));
    }

    #[tokio::test]
    async fn test_rls_style_filtering() {
        let policy = TenantAccessControlPolicy::new("filtering-policy");

        let context = SecurityContext::new("user1")
            .with_tenant_id("tenant1")
            .with_roles(vec!["user".to_string()]);

        // Test non-tenant resource gets filtered
        let decision = policy
            .evaluate_access(&context, "search", "global-resource")
            .await
            .unwrap();

        if let AccessDecision::AllowWithFilters(filters) = decision {
            assert_eq!(filters.len(), 1);
            assert_eq!(filters[0].field, "tenant_id");
            assert!(filters[0].allowed_values.contains("tenant1"));
            assert!(!filters[0].exclude);
        } else {
            panic!("Expected AllowWithFilters, got {:?}", decision);
        }
    }

    #[tokio::test]
    async fn test_enhanced_security_manager() {
        let audit_logger = Arc::new(AuditLogger::new());
        let manager = EnhancedSecurityManager::new(audit_logger);

        // Add a tenant policy
        let policy = Arc::new(TenantAccessControlPolicy::new("test-policy"));
        manager.add_access_policy(policy).await;

        let context = SecurityContext::new("user1")
            .with_tenant_id("tenant1")
            .with_roles(vec!["user".to_string()]);

        // Test evaluation
        let decision = manager
            .evaluate_access(&context, "search", "tenant:tenant1")
            .await
            .unwrap();
        assert!(matches!(decision, AccessDecision::Allow));

        // Test caching - second call should be faster
        let decision2 = manager
            .evaluate_access(&context, "search", "tenant:tenant1")
            .await
            .unwrap();
        assert!(matches!(decision2, AccessDecision::Allow));
    }

    #[tokio::test]
    async fn test_apply_filters_functionality() {
        let mut metadata = HashMap::new();
        metadata.insert(
            "tenant_id".to_string(),
            serde_json::Value::String("tenant1".to_string()),
        );
        metadata.insert(
            "user_id".to_string(),
            serde_json::Value::String("user123".to_string()),
        );

        // Test include filter
        let filter = SecurityFilter {
            field: "tenant_id".to_string(),
            allowed_values: HashSet::from_iter(["tenant1".to_string()]),
            exclude: false,
        };

        assert!(VectorSecurityManager::apply_filters(
            &metadata,
            std::slice::from_ref(&filter)
        ));

        // Test exclude filter
        let exclude_filter = SecurityFilter {
            field: "tenant_id".to_string(),
            allowed_values: HashSet::from_iter(["tenant2".to_string()]),
            exclude: true,
        };

        assert!(VectorSecurityManager::apply_filters(
            &metadata,
            &[exclude_filter]
        ));

        // Test failed include
        let failed_filter = SecurityFilter {
            field: "tenant_id".to_string(),
            allowed_values: HashSet::from_iter(["tenant2".to_string()]),
            exclude: false,
        };

        assert!(!VectorSecurityManager::apply_filters(
            &metadata,
            &[failed_filter]
        ));
    }

    /// High priority policy for testing
    struct HighPriorityPolicy {
        policy_id: String,
    }

    #[async_trait]
    impl AccessControlPolicy for HighPriorityPolicy {
        async fn evaluate_access(
            &self,
            _security_context: &SecurityContext,
            _operation: &str,
            _resource: &str,
        ) -> Result<AccessDecision> {
            Ok(AccessDecision::Allow)
        }

        fn applies_to(&self, _security_context: &SecurityContext, _operation: &str) -> bool {
            true
        }

        fn policy_id(&self) -> &str {
            &self.policy_id
        }

        fn priority(&self) -> i32 {
            200 // Higher than TenantAccessControlPolicy
        }
    }

    #[tokio::test]
    async fn test_policy_priority_ordering() {
        let audit_logger = Arc::new(AuditLogger::new());
        let manager = EnhancedSecurityManager::new(audit_logger);

        // Add policies with different priorities
        let low_priority = Arc::new(TenantAccessControlPolicy::new("low-priority"));
        let high_priority = Arc::new(HighPriorityPolicy {
            policy_id: "high-priority".to_string(),
        });

        manager.add_access_policy(low_priority).await;
        manager.add_access_policy(high_priority).await;

        // Verify policies are ordered by priority
        let policies = manager.access_policies.read().await;
        assert_eq!(policies.len(), 2);
        assert_eq!(policies[0].policy_id(), "high-priority"); // Higher priority first
        assert_eq!(policies[1].policy_id(), "low-priority");
    }
}
