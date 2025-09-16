//! Security context for access control decisions

use anyhow::Result;
use llmspell_core::state::StateScope;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::SystemTime;

/// Security context containing all information needed for access control decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityContext {
    /// User or service principal making the request
    pub principal: String,

    /// Tenant ID for multi-tenant operations
    pub tenant_id: Option<String>,

    /// User roles and permissions
    pub roles: Vec<String>,

    /// Additional attributes for policy evaluation
    pub attributes: HashMap<String, String>,

    /// Request metadata
    pub metadata: RequestMetadata,
}

/// Request metadata for audit and policy evaluation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMetadata {
    /// Request timestamp
    pub timestamp: SystemTime,

    /// Source IP address
    pub source_ip: Option<String>,

    /// User agent or client identifier
    pub user_agent: Option<String>,

    /// Session ID
    pub session_id: Option<String>,

    /// Correlation ID for tracking across services
    pub correlation_id: Option<String>,
}

impl SecurityContext {
    /// Create a new security context
    pub fn new(principal: impl Into<String>) -> Self {
        Self {
            principal: principal.into(),
            tenant_id: None,
            roles: Vec::new(),
            attributes: HashMap::new(),
            metadata: RequestMetadata::default(),
        }
    }

    /// Set the tenant ID for multi-tenant operations
    pub fn with_tenant_id(mut self, tenant_id: impl Into<String>) -> Self {
        self.tenant_id = Some(tenant_id.into());
        self
    }

    /// Add roles to the security context
    pub fn with_roles(mut self, roles: Vec<String>) -> Self {
        self.roles = roles;
        self
    }

    /// Add an attribute for policy evaluation
    pub fn with_attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }

    /// Set request metadata
    pub fn with_metadata(mut self, metadata: RequestMetadata) -> Self {
        self.metadata = metadata;
        self
    }

    /// Check if the context has a specific role
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }

    /// Get an attribute value
    pub fn get_attribute(&self, key: &str) -> Option<&String> {
        self.attributes.get(key)
    }

    /// Convert to StateScope for tenant-aware operations
    pub fn to_state_scope(&self) -> Result<StateScope> {
        if let Some(tenant_id) = &self.tenant_id {
            Ok(StateScope::Custom(format!("tenant:{}", tenant_id)))
        } else {
            Ok(StateScope::Global)
        }
    }

    /// Check if the context is valid for the operation
    pub fn validate(&self) -> Result<()> {
        if self.principal.is_empty() {
            return Err(anyhow::anyhow!("Principal cannot be empty"));
        }

        // Additional validation can be added here
        Ok(())
    }
}

impl Default for RequestMetadata {
    fn default() -> Self {
        Self {
            timestamp: SystemTime::now(),
            source_ip: None,
            user_agent: None,
            session_id: None,
            correlation_id: None,
        }
    }
}

/// Builder for creating SecurityContext instances
pub struct SecurityContextBuilder {
    context: SecurityContext,
}

impl SecurityContextBuilder {
    /// Start building a security context
    pub fn new(principal: impl Into<String>) -> Self {
        Self {
            context: SecurityContext::new(principal),
        }
    }

    /// Set tenant ID
    pub fn tenant_id(mut self, tenant_id: impl Into<String>) -> Self {
        self.context.tenant_id = Some(tenant_id.into());
        self
    }

    /// Add roles
    pub fn roles(mut self, roles: Vec<String>) -> Self {
        self.context.roles = roles;
        self
    }

    /// Add attribute
    pub fn attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.context.attributes.insert(key.into(), value.into());
        self
    }

    /// Set source IP
    pub fn source_ip(mut self, ip: impl Into<String>) -> Self {
        self.context.metadata.source_ip = Some(ip.into());
        self
    }

    /// Set session ID
    pub fn session_id(mut self, session_id: impl Into<String>) -> Self {
        self.context.metadata.session_id = Some(session_id.into());
        self
    }

    /// Build the security context
    pub fn build(self) -> SecurityContext {
        self.context
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_context_creation() {
        let context = SecurityContext::new("user123")
            .with_tenant_id("tenant-1")
            .with_roles(vec!["admin".to_string(), "user".to_string()])
            .with_attribute("department", "engineering");

        assert_eq!(context.principal, "user123");
        assert_eq!(context.tenant_id, Some("tenant-1".to_string()));
        assert!(context.has_role("admin"));
        assert!(context.has_role("user"));
        assert!(!context.has_role("guest"));
        assert_eq!(
            context.get_attribute("department"),
            Some(&"engineering".to_string())
        );
    }

    #[test]
    fn test_security_context_builder() {
        let context = SecurityContextBuilder::new("service-account")
            .tenant_id("prod-tenant")
            .roles(vec!["service".to_string()])
            .attribute("environment", "production")
            .source_ip("192.168.1.100")
            .session_id("session-123")
            .build();

        assert_eq!(context.principal, "service-account");
        assert_eq!(context.tenant_id, Some("prod-tenant".to_string()));
        assert_eq!(
            context.metadata.source_ip,
            Some("192.168.1.100".to_string())
        );
        assert_eq!(context.metadata.session_id, Some("session-123".to_string()));
    }

    #[test]
    fn test_state_scope_conversion() {
        let context = SecurityContext::new("user").with_tenant_id("test-tenant");

        let scope = context.to_state_scope().unwrap();
        assert_eq!(scope, StateScope::Custom("tenant:test-tenant".to_string()));

        let global_context = SecurityContext::new("admin");
        let global_scope = global_context.to_state_scope().unwrap();
        assert_eq!(global_scope, StateScope::Global);
    }

    #[test]
    fn test_validation() {
        let valid_context = SecurityContext::new("user");
        assert!(valid_context.validate().is_ok());

        let invalid_context = SecurityContext::new("");
        assert!(invalid_context.validate().is_err());
    }
}
