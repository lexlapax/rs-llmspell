//! Audit logging for security events

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tracing::{error, info};

/// Audit event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditEvent {
    /// Access granted
    AccessGranted {
        principal: String,
        operation: String,
        resource: String,
        metadata: HashMap<String, String>,
    },

    /// Access denied
    AccessDenied {
        principal: String,
        operation: String,
        resource: String,
        reason: String,
        metadata: HashMap<String, String>,
    },

    /// Rate limit exceeded
    RateLimitExceeded {
        principal: String,
        limit: u32,
        current: u32,
    },

    /// Tenant created
    TenantCreated {
        tenant_id: String,
        created_by: String,
    },

    /// Tenant deleted
    TenantDeleted {
        tenant_id: String,
        deleted_by: String,
        vectors_removed: usize,
    },

    /// Configuration changed
    ConfigurationChanged {
        tenant_id: String,
        changed_by: String,
        changes: HashMap<String, String>,
    },

    /// Suspicious activity detected
    SuspiciousActivity {
        principal: String,
        activity: String,
        details: String,
    },
}

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    /// Unique entry ID
    pub id: uuid::Uuid,

    /// Timestamp
    pub timestamp: DateTime<Utc>,

    /// Event
    pub event: AuditEvent,

    /// Source IP if available
    pub source_ip: Option<String>,

    /// Session ID if available
    pub session_id: Option<String>,

    /// Correlation ID for related events
    pub correlation_id: Option<String>,
}

impl AuditEntry {
    /// Create a new audit entry
    pub fn new(event: AuditEvent) -> Self {
        Self {
            id: uuid::Uuid::new_v4(),
            timestamp: Utc::now(),
            event,
            source_ip: None,
            session_id: None,
            correlation_id: None,
        }
    }

    /// Set source IP
    pub fn with_source_ip(mut self, ip: impl Into<String>) -> Self {
        self.source_ip = Some(ip.into());
        self
    }

    /// Set session ID
    pub fn with_session_id(mut self, id: impl Into<String>) -> Self {
        self.session_id = Some(id.into());
        self
    }

    /// Set correlation ID
    pub fn with_correlation_id(mut self, id: impl Into<String>) -> Self {
        self.correlation_id = Some(id.into());
        self
    }
}

/// Audit logger
pub struct AuditLogger {
    /// Channel for async logging
    sender: mpsc::UnboundedSender<AuditEntry>,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new() -> Self {
        let (sender, mut receiver) = mpsc::unbounded_channel::<AuditEntry>();

        // Spawn background task for processing audit logs
        tokio::spawn(async move {
            while let Some(entry) = receiver.recv().await {
                // Log to tracing
                match &entry.event {
                    AuditEvent::AccessGranted {
                        principal,
                        operation,
                        resource,
                        ..
                    } => {
                        info!(
                            audit = true,
                            event_type = "access_granted",
                            principal = %principal,
                            operation = %operation,
                            resource = %resource,
                            "Access granted"
                        );
                    }
                    AuditEvent::AccessDenied {
                        principal,
                        operation,
                        resource,
                        reason,
                        ..
                    } => {
                        info!(
                            audit = true,
                            event_type = "access_denied",
                            principal = %principal,
                            operation = %operation,
                            resource = %resource,
                            reason = %reason,
                            "Access denied"
                        );
                    }
                    AuditEvent::RateLimitExceeded {
                        principal,
                        limit,
                        current,
                    } => {
                        info!(
                            audit = true,
                            event_type = "rate_limit_exceeded",
                            principal = %principal,
                            limit = limit,
                            current = current,
                            "Rate limit exceeded"
                        );
                    }
                    AuditEvent::TenantCreated {
                        tenant_id,
                        created_by,
                    } => {
                        info!(
                            audit = true,
                            event_type = "tenant_created",
                            tenant_id = %tenant_id,
                            created_by = %created_by,
                            "Tenant created"
                        );
                    }
                    AuditEvent::TenantDeleted {
                        tenant_id,
                        deleted_by,
                        vectors_removed,
                    } => {
                        info!(
                            audit = true,
                            event_type = "tenant_deleted",
                            tenant_id = %tenant_id,
                            deleted_by = %deleted_by,
                            vectors_removed = vectors_removed,
                            "Tenant deleted"
                        );
                    }
                    AuditEvent::ConfigurationChanged {
                        tenant_id,
                        changed_by,
                        ..
                    } => {
                        info!(
                            audit = true,
                            event_type = "configuration_changed",
                            tenant_id = %tenant_id,
                            changed_by = %changed_by,
                            "Configuration changed"
                        );
                    }
                    AuditEvent::SuspiciousActivity {
                        principal,
                        activity,
                        details,
                    } => {
                        info!(
                            audit = true,
                            event_type = "suspicious_activity",
                            principal = %principal,
                            activity = %activity,
                            details = %details,
                            "Suspicious activity detected"
                        );
                    }
                }

                // Here you could also:
                // - Write to a database
                // - Send to an external audit service
                // - Write to a file
                // - Send alerts for critical events
            }
        });

        Self { sender }
    }

    /// Log an audit event
    pub async fn log(&self, event: AuditEvent) -> Result<()> {
        let entry = AuditEntry::new(event);

        self.sender.send(entry).map_err(|e| {
            error!("Failed to send audit entry: {}", e);
            anyhow::anyhow!("Audit logging failed")
        })?;

        Ok(())
    }

    /// Log an audit event with context
    pub async fn log_with_context(
        &self,
        event: AuditEvent,
        source_ip: Option<String>,
        session_id: Option<String>,
        correlation_id: Option<String>,
    ) -> Result<()> {
        let mut entry = AuditEntry::new(event);

        if let Some(ip) = source_ip {
            entry = entry.with_source_ip(ip);
        }
        if let Some(id) = session_id {
            entry = entry.with_session_id(id);
        }
        if let Some(id) = correlation_id {
            entry = entry.with_correlation_id(id);
        }

        self.sender.send(entry).map_err(|e| {
            error!("Failed to send audit entry: {}", e);
            anyhow::anyhow!("Audit logging failed")
        })?;

        Ok(())
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_audit_logging() {
        let logger = AuditLogger::new();

        // Test access granted
        logger
            .log(AuditEvent::AccessGranted {
                principal: "user1".to_string(),
                operation: "search".to_string(),
                resource: "tenant1".to_string(),
                metadata: HashMap::new(),
            })
            .await
            .unwrap();

        // Test access denied
        logger
            .log(AuditEvent::AccessDenied {
                principal: "user2".to_string(),
                operation: "delete".to_string(),
                resource: "tenant1".to_string(),
                reason: "Insufficient permissions".to_string(),
                metadata: HashMap::new(),
            })
            .await
            .unwrap();

        // Give time for async processing
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }

    #[tokio::test]
    async fn test_audit_with_context() {
        let logger = AuditLogger::new();

        logger
            .log_with_context(
                AuditEvent::TenantCreated {
                    tenant_id: "test-tenant".to_string(),
                    created_by: "admin".to_string(),
                },
                Some("192.168.1.1".to_string()),
                Some("session-123".to_string()),
                Some("correlation-456".to_string()),
            )
            .await
            .unwrap();

        // Give time for async processing
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
    }
}
