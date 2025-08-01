// ABOUTME: SecurityHook implementation for comprehensive security monitoring and audit logging
// ABOUTME: Provides security violation detection, audit logging, and access control for all hook points

use crate::context::HookContext;
use crate::result::HookResult;
use crate::traits::{Hook, MetricHook, ReplayableHook};
use crate::types::{HookMetadata, HookPoint, Language, Priority};
use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, RwLock};

/// Security event severity levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum SecuritySeverity {
    Info,
    Low,
    Medium,
    High,
    Critical,
}

impl std::fmt::Display for SecuritySeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecuritySeverity::Info => write!(f, "INFO"),
            SecuritySeverity::Low => write!(f, "LOW"),
            SecuritySeverity::Medium => write!(f, "MEDIUM"),
            SecuritySeverity::High => write!(f, "HIGH"),
            SecuritySeverity::Critical => write!(f, "CRITICAL"),
        }
    }
}

/// Security event types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum SecurityEventType {
    AccessAttempt,
    PermissionDenied,
    SuspiciousActivity,
    DataAccess,
    ParameterInjection,
    RateLimitExceeded,
    UnauthorizedOperation,
    SecurityViolation,
    AuditLog,
    Custom(String),
}

impl std::fmt::Display for SecurityEventType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SecurityEventType::Custom(name) => write!(f, "CUSTOM:{}", name),
            _ => write!(f, "{:?}", self),
        }
    }
}

/// Security audit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: SecurityEventType,
    pub severity: SecuritySeverity,
    pub hook_point: HookPoint,
    pub component_name: String,
    pub component_type: String,
    pub language: String,
    pub correlation_id: String,
    pub user_id: Option<String>,
    pub session_id: Option<String>,
    pub source_ip: Option<String>,
    pub description: String,
    pub details: HashMap<String, serde_json::Value>,
    pub blocked: bool,
}

/// Security rule for validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityRule {
    pub name: String,
    pub description: String,
    pub hook_points: HashSet<HookPoint>,
    pub enabled: bool,
    pub severity: SecuritySeverity,
    pub action: SecurityAction,
}

/// Actions to take when a security rule is triggered
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SecurityAction {
    Log,
    Block,
    Alert,
    LogAndBlock,
    LogAndAlert,
    Custom(String),
}

/// Configuration for the security hook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Whether to enable audit logging
    pub enable_audit_logging: bool,
    /// Whether to enable parameter validation
    pub enable_parameter_validation: bool,
    /// Whether to enable rate limiting checks
    pub enable_rate_limiting: bool,
    /// Maximum number of events to keep in memory
    pub max_events: usize,
    /// Minimum severity level to log
    pub min_severity: SecuritySeverity,
    /// Whether to block on security violations
    pub block_on_violations: bool,
    /// Sensitive parameter names to mask
    pub sensitive_parameters: HashSet<String>,
    /// Maximum parameter value length
    pub max_parameter_length: usize,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        let mut sensitive_parameters = HashSet::new();
        sensitive_parameters.insert("password".to_string());
        sensitive_parameters.insert("token".to_string());
        sensitive_parameters.insert("key".to_string());
        sensitive_parameters.insert("secret".to_string());
        sensitive_parameters.insert("api_key".to_string());
        sensitive_parameters.insert("auth".to_string());
        sensitive_parameters.insert("authorization".to_string());

        Self {
            enable_audit_logging: true,
            enable_parameter_validation: true,
            enable_rate_limiting: true,
            max_events: 10000,
            min_severity: SecuritySeverity::Info,
            block_on_violations: false,
            sensitive_parameters,
            max_parameter_length: 10000, // 10KB max parameter size
        }
    }
}

/// Security event storage
#[derive(Debug, Default)]
pub struct SecurityStorage {
    events: Arc<RwLock<Vec<SecurityEvent>>>,
    rules: Arc<RwLock<Vec<SecurityRule>>>,
    config: SecurityConfig,
}

impl SecurityStorage {
    pub fn new(config: SecurityConfig) -> Self {
        let mut storage = Self {
            events: Arc::new(RwLock::new(Vec::new())),
            rules: Arc::new(RwLock::new(Vec::new())),
            config,
        };

        // Add default security rules
        storage.add_default_rules();
        storage
    }

    /// Add default security rules
    fn add_default_rules(&mut self) {
        let default_rules = vec![
            SecurityRule {
                name: "Parameter Size Validation".to_string(),
                description: "Validates that parameters don't exceed maximum size".to_string(),
                hook_points: [
                    HookPoint::BeforeToolExecution,
                    HookPoint::BeforeAgentExecution,
                ]
                .iter()
                .cloned()
                .collect(),
                enabled: true,
                severity: SecuritySeverity::Medium,
                action: SecurityAction::LogAndBlock,
            },
            SecurityRule {
                name: "Sensitive Parameter Detection".to_string(),
                description: "Detects and masks sensitive parameters".to_string(),
                hook_points: [
                    HookPoint::BeforeToolExecution,
                    HookPoint::BeforeAgentExecution,
                    HookPoint::BeforeStateWrite,
                ]
                .iter()
                .cloned()
                .collect(),
                enabled: true,
                severity: SecuritySeverity::Low,
                action: SecurityAction::Log,
            },
            SecurityRule {
                name: "Security Violation Monitor".to_string(),
                description: "Monitors explicit security violations".to_string(),
                hook_points: [HookPoint::SecurityViolation].iter().cloned().collect(),
                enabled: true,
                severity: SecuritySeverity::Critical,
                action: SecurityAction::LogAndAlert,
            },
        ];

        let mut rules = self.rules.write().unwrap();
        rules.extend(default_rules);
    }

    /// Add a security event
    pub fn add_event(&self, event: SecurityEvent) -> bool {
        // Check severity filter
        if event.severity < self.config.min_severity {
            return false;
        }

        let mut events = self.events.write().unwrap();
        events.push(event.clone());

        // Maintain max events limit
        if events.len() > self.config.max_events {
            events.remove(0);
        }

        // Log the security event
        if self.config.enable_audit_logging {
            log::warn!(
                "SECURITY EVENT [{}]: {} - {} ({}:{})",
                event.severity,
                event.event_type,
                event.description,
                event.component_type,
                event.component_name
            );
        }

        event.blocked
    }

    /// Add a security rule
    pub fn add_rule(&self, rule: SecurityRule) {
        let mut rules = self.rules.write().unwrap();
        rules.push(rule);
    }

    /// Get all security events
    pub fn get_events(&self) -> Vec<SecurityEvent> {
        self.events.read().unwrap().clone()
    }

    /// Get events by severity
    pub fn get_events_by_severity(&self, min_severity: SecuritySeverity) -> Vec<SecurityEvent> {
        self.events
            .read()
            .unwrap()
            .iter()
            .filter(|event| event.severity >= min_severity)
            .cloned()
            .collect()
    }

    /// Get events by type
    pub fn get_events_by_type(&self, event_type: &SecurityEventType) -> Vec<SecurityEvent> {
        self.events
            .read()
            .unwrap()
            .iter()
            .filter(|event| &event.event_type == event_type)
            .cloned()
            .collect()
    }

    /// Get security rules for a hook point
    pub fn get_rules_for_hook_point(&self, hook_point: &HookPoint) -> Vec<SecurityRule> {
        self.rules
            .read()
            .unwrap()
            .iter()
            .filter(|rule| rule.enabled && rule.hook_points.contains(hook_point))
            .cloned()
            .collect()
    }

    /// Get security statistics
    pub fn get_statistics(&self) -> HashMap<String, serde_json::Value> {
        let events = self.events.read().unwrap();
        let mut stats = HashMap::new();

        // Total events
        stats.insert(
            "total_events".to_string(),
            serde_json::Value::Number(events.len().into()),
        );

        // Events by severity
        let mut severity_counts = HashMap::new();
        for event in events.iter() {
            let severity_str = event.severity.to_string();
            *severity_counts.entry(severity_str).or_insert(0u64) += 1;
        }

        let severity_json: serde_json::Map<String, serde_json::Value> = severity_counts
            .into_iter()
            .map(|(k, v)| (k, serde_json::Value::Number(v.into())))
            .collect();
        stats.insert(
            "by_severity".to_string(),
            serde_json::Value::Object(severity_json),
        );

        // Events by type
        let mut type_counts = HashMap::new();
        for event in events.iter() {
            let type_str = event.event_type.to_string();
            *type_counts.entry(type_str).or_insert(0u64) += 1;
        }

        let type_json: serde_json::Map<String, serde_json::Value> = type_counts
            .into_iter()
            .map(|(k, v)| (k, serde_json::Value::Number(v.into())))
            .collect();
        stats.insert("by_type".to_string(), serde_json::Value::Object(type_json));

        // Blocked events count
        let blocked_count = events.iter().filter(|event| event.blocked).count();
        stats.insert(
            "blocked_events".to_string(),
            serde_json::Value::Number(blocked_count.into()),
        );

        stats
    }

    /// Clear all events
    pub fn clear_events(&self) {
        self.events.write().unwrap().clear();
    }
}

/// Built-in security hook for comprehensive security monitoring
pub struct SecurityHook {
    storage: Arc<SecurityStorage>,
    metadata: HookMetadata,
}

impl SecurityHook {
    /// Create a new security hook with default configuration
    pub fn new() -> Self {
        Self {
            storage: Arc::new(SecurityStorage::new(SecurityConfig::default())),
            metadata: HookMetadata {
                name: "SecurityHook".to_string(),
                description: Some(
                    "Built-in hook for security monitoring and audit logging".to_string(),
                ),
                priority: Priority::HIGHEST, // Run first for security checks
                language: Language::Native,
                tags: vec!["builtin".to_string(), "security".to_string()],
                version: "1.0.0".to_string(),
            },
        }
    }

    /// Create a new security hook with custom configuration
    pub fn with_config(config: SecurityConfig) -> Self {
        Self {
            storage: Arc::new(SecurityStorage::new(config)),
            metadata: HookMetadata {
                name: "SecurityHook".to_string(),
                description: Some(
                    "Built-in hook for security monitoring and audit logging".to_string(),
                ),
                priority: Priority::HIGHEST,
                language: Language::Native,
                tags: vec!["builtin".to_string(), "security".to_string()],
                version: "1.0.0".to_string(),
            },
        }
    }

    /// Enable or disable audit logging (only works on new instance)
    pub fn with_audit_logging(self, enable: bool) -> Self {
        let mut config = self.storage.config.clone();
        config.enable_audit_logging = enable;
        Self {
            storage: Arc::new(SecurityStorage::new(config)),
            metadata: self.metadata,
        }
    }

    /// Set minimum severity level (only works on new instance)
    pub fn with_min_severity(self, severity: SecuritySeverity) -> Self {
        let mut config = self.storage.config.clone();
        config.min_severity = severity;
        Self {
            storage: Arc::new(SecurityStorage::new(config)),
            metadata: self.metadata,
        }
    }

    /// Enable or disable blocking on violations (only works on new instance)
    pub fn with_blocking(self, enable: bool) -> Self {
        let mut config = self.storage.config.clone();
        config.block_on_violations = enable;
        Self {
            storage: Arc::new(SecurityStorage::new(config)),
            metadata: self.metadata,
        }
    }

    /// Get the security storage
    pub fn storage(&self) -> Arc<SecurityStorage> {
        self.storage.clone()
    }

    /// Get all security events
    pub fn get_events(&self) -> Vec<SecurityEvent> {
        self.storage.get_events()
    }

    /// Get security statistics
    pub fn get_statistics(&self) -> HashMap<String, serde_json::Value> {
        self.storage.get_statistics()
    }

    /// Add a custom security rule
    pub fn add_rule(&self, rule: SecurityRule) {
        self.storage.add_rule(rule);
    }

    /// Validate parameters against security rules
    fn validate_parameters(&self, context: &HookContext) -> Vec<SecurityEvent> {
        let mut events = Vec::new();

        if !self.storage.config.enable_parameter_validation {
            return events;
        }

        // Check parameter sizes
        for (key, value) in &context.data {
            let value_str = serde_json::to_string(value).unwrap_or_default();
            if value_str.len() > self.storage.config.max_parameter_length {
                events.push(SecurityEvent {
                    timestamp: Utc::now(),
                    event_type: SecurityEventType::ParameterInjection,
                    severity: SecuritySeverity::Medium,
                    hook_point: context.point.clone(),
                    component_name: context.component_id.name.clone(),
                    component_type: format!("{:?}", context.component_id.component_type),
                    language: format!("{:?}", context.language),
                    correlation_id: context.correlation_id.to_string(),
                    user_id: context.get_metadata("user_id").map(|s| s.to_string()),
                    session_id: context.get_metadata("session_id").map(|s| s.to_string()),
                    source_ip: context.get_metadata("source_ip").map(|s| s.to_string()),
                    description: format!(
                        "Parameter '{}' exceeds maximum size ({} > {})",
                        key,
                        value_str.len(),
                        self.storage.config.max_parameter_length
                    ),
                    details: {
                        let mut details = HashMap::new();
                        details.insert(
                            "parameter_name".to_string(),
                            serde_json::Value::String(key.clone()),
                        );
                        details.insert(
                            "parameter_size".to_string(),
                            serde_json::Value::Number(value_str.len().into()),
                        );
                        details.insert(
                            "max_size".to_string(),
                            serde_json::Value::Number(
                                self.storage.config.max_parameter_length.into(),
                            ),
                        );
                        details
                    },
                    blocked: self.storage.config.block_on_violations,
                });
            }

            // Check for sensitive parameters
            if self.storage.config.sensitive_parameters.contains(key) {
                events.push(SecurityEvent {
                    timestamp: Utc::now(),
                    event_type: SecurityEventType::DataAccess,
                    severity: SecuritySeverity::Low,
                    hook_point: context.point.clone(),
                    component_name: context.component_id.name.clone(),
                    component_type: format!("{:?}", context.component_id.component_type),
                    language: format!("{:?}", context.language),
                    correlation_id: context.correlation_id.to_string(),
                    user_id: context.get_metadata("user_id").map(|s| s.to_string()),
                    session_id: context.get_metadata("session_id").map(|s| s.to_string()),
                    source_ip: context.get_metadata("source_ip").map(|s| s.to_string()),
                    description: format!("Access to sensitive parameter '{}'", key),
                    details: {
                        let mut details = HashMap::new();
                        details.insert(
                            "parameter_name".to_string(),
                            serde_json::Value::String(key.clone()),
                        );
                        details.insert(
                            "parameter_value".to_string(),
                            serde_json::Value::String("[MASKED]".to_string()),
                        );
                        details
                    },
                    blocked: false, // Don't block sensitive parameter access, just log
                });
            }
        }

        events
    }

    /// Create a generic audit log event
    fn create_audit_event(&self, context: &HookContext) -> SecurityEvent {
        SecurityEvent {
            timestamp: Utc::now(),
            event_type: SecurityEventType::AuditLog,
            severity: SecuritySeverity::Info,
            hook_point: context.point.clone(),
            component_name: context.component_id.name.clone(),
            component_type: format!("{:?}", context.component_id.component_type),
            language: format!("{:?}", context.language),
            correlation_id: context.correlation_id.to_string(),
            user_id: context.get_metadata("user_id").map(|s| s.to_string()),
            session_id: context.get_metadata("session_id").map(|s| s.to_string()),
            source_ip: context.get_metadata("source_ip").map(|s| s.to_string()),
            description: format!("Hook execution at {:?}", context.point),
            details: HashMap::new(),
            blocked: false,
        }
    }
}

impl Default for SecurityHook {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Hook for SecurityHook {
    async fn execute(&self, context: &mut HookContext) -> Result<HookResult> {
        // Create audit log event if enabled
        if self.storage.config.enable_audit_logging {
            let audit_event = self.create_audit_event(context);
            self.storage.add_event(audit_event);
        }

        // Validate parameters
        let security_events = self.validate_parameters(context);
        let mut should_block = false;

        for event in security_events {
            if event.blocked {
                should_block = true;
            }
            self.storage.add_event(event);
        }

        // Add security metadata to context
        context.insert_metadata("security_checked_at".to_string(), Utc::now().to_rfc3339());
        context.insert_metadata(
            "security_hook_version".to_string(),
            self.metadata.version.clone(),
        );

        // Block execution if security violation detected
        if should_block {
            context.insert_metadata("security_blocked".to_string(), "true".to_string());
            return Ok(HookResult::Cancel(
                "Security violation detected".to_string(),
            ));
        }

        Ok(HookResult::Continue)
    }

    fn metadata(&self) -> HookMetadata {
        self.metadata.clone()
    }

    fn should_execute(&self, _context: &HookContext) -> bool {
        // Always execute security hook
        true
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

#[async_trait]
impl MetricHook for SecurityHook {
    async fn record_pre_execution(&self, context: &HookContext) -> Result<()> {
        // Record access attempt
        let access_event = SecurityEvent {
            timestamp: Utc::now(),
            event_type: SecurityEventType::AccessAttempt,
            severity: SecuritySeverity::Info,
            hook_point: context.point.clone(),
            component_name: context.component_id.name.clone(),
            component_type: format!("{:?}", context.component_id.component_type),
            language: format!("{:?}", context.language),
            correlation_id: context.correlation_id.to_string(),
            user_id: context.get_metadata("user_id").map(|s| s.to_string()),
            session_id: context.get_metadata("session_id").map(|s| s.to_string()),
            source_ip: context.get_metadata("source_ip").map(|s| s.to_string()),
            description: "Pre-execution security check".to_string(),
            details: HashMap::new(),
            blocked: false,
        };

        self.storage.add_event(access_event);
        Ok(())
    }

    async fn record_post_execution(
        &self,
        context: &HookContext,
        result: &HookResult,
        duration: std::time::Duration,
    ) -> Result<()> {
        // Record execution completion with security context
        let mut details = HashMap::new();
        details.insert(
            "execution_duration_ms".to_string(),
            serde_json::Value::Number((duration.as_millis() as u64).into()),
        );
        details.insert(
            "result_type".to_string(),
            serde_json::Value::String(format!("{:?}", std::mem::discriminant(result))),
        );
        details.insert(
            "execution_successful".to_string(),
            serde_json::Value::Bool(result.should_continue()),
        );

        let completion_event = SecurityEvent {
            timestamp: Utc::now(),
            event_type: SecurityEventType::AuditLog,
            severity: SecuritySeverity::Info,
            hook_point: context.point.clone(),
            component_name: context.component_id.name.clone(),
            component_type: format!("{:?}", context.component_id.component_type),
            language: format!("{:?}", context.language),
            correlation_id: context.correlation_id.to_string(),
            user_id: context.get_metadata("user_id").map(|s| s.to_string()),
            session_id: context.get_metadata("session_id").map(|s| s.to_string()),
            source_ip: context.get_metadata("source_ip").map(|s| s.to_string()),
            description: "Post-execution security audit".to_string(),
            details,
            blocked: false,
        };

        self.storage.add_event(completion_event);
        Ok(())
    }
}

#[cfg(test)]
#[cfg_attr(test_category = "hook")]
mod tests {
    use super::*;
    use crate::types::{ComponentId, ComponentType, HookPoint};
    use serde_json::json;

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_security_hook_basic() {
        let hook = SecurityHook::new();
        let component_id = ComponentId::new(ComponentType::System, "test".to_string());
        let mut context = HookContext::new(HookPoint::SystemStartup, component_id);

        let result = hook.execute(&mut context).await.unwrap();
        assert!(matches!(result, HookResult::Continue));

        // Check that security metadata was added
        assert!(context.get_metadata("security_checked_at").is_some());
        assert!(context.get_metadata("security_hook_version").is_some());

        // Check that audit event was recorded
        let events = hook.get_events();
        assert!(!events.is_empty());
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_security_hook_parameter_validation() {
        let hook = SecurityHook::new().with_blocking(true);
        let component_id = ComponentId::new(ComponentType::Tool, "test-tool".to_string());
        let mut context = HookContext::new(HookPoint::BeforeToolExecution, component_id);

        // Add a large parameter that should trigger validation
        let large_data = "x".repeat(20000);
        context.insert_data("large_param".to_string(), json!(large_data));

        let result = hook.execute(&mut context).await.unwrap();
        assert!(matches!(result, HookResult::Cancel(_)));

        // Check that security violation was recorded
        let events = hook.get_events();
        let violations: Vec<_> = events
            .iter()
            .filter(|e| e.event_type == SecurityEventType::ParameterInjection)
            .collect();
        assert!(!violations.is_empty());
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_security_hook_sensitive_parameters() {
        let hook = SecurityHook::new();
        let component_id = ComponentId::new(ComponentType::Agent, "test-agent".to_string());
        let mut context = HookContext::new(HookPoint::BeforeAgentExecution, component_id);

        // Add sensitive parameters
        context.insert_data("password".to_string(), json!("secret123"));
        context.insert_data("api_key".to_string(), json!("sk_test_123"));

        let result = hook.execute(&mut context).await.unwrap();
        assert!(matches!(result, HookResult::Continue)); // Should not block, just log

        // Check that sensitive parameter access was logged
        let events = hook.get_events();
        let data_access_events: Vec<_> = events
            .iter()
            .filter(|e| e.event_type == SecurityEventType::DataAccess)
            .collect();
        assert_eq!(data_access_events.len(), 2); // password and api_key
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_security_config_defaults() {
        let config = SecurityConfig::default();
        assert!(config.enable_audit_logging);
        assert!(config.enable_parameter_validation);
        assert_eq!(config.min_severity, SecuritySeverity::Info);
        assert!(!config.block_on_violations);
        assert!(config.sensitive_parameters.contains("password"));
        assert!(config.sensitive_parameters.contains("api_key"));
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_security_event_serialization() {
        let event = SecurityEvent {
            timestamp: Utc::now(),
            event_type: SecurityEventType::AccessAttempt,
            severity: SecuritySeverity::Medium,
            hook_point: HookPoint::SystemStartup,
            component_name: "test".to_string(),
            component_type: "System".to_string(),
            language: "Native".to_string(),
            correlation_id: "test-id".to_string(),
            user_id: Some("user123".to_string()),
            session_id: Some("session456".to_string()),
            source_ip: Some("192.168.1.1".to_string()),
            description: "Test event".to_string(),
            details: HashMap::new(),
            blocked: false,
        };

        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: SecurityEvent = serde_json::from_str(&serialized).unwrap();

        assert_eq!(deserialized.event_type, SecurityEventType::AccessAttempt);
        assert_eq!(deserialized.severity, SecuritySeverity::Medium);
        assert_eq!(deserialized.user_id, Some("user123".to_string()));
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_security_storage_filtering() {
        let storage = SecurityStorage::new(SecurityConfig::default());

        // Add test events with different severities
        let events = vec![
            SecurityEvent {
                timestamp: Utc::now(),
                event_type: SecurityEventType::AccessAttempt,
                severity: SecuritySeverity::Info,
                hook_point: HookPoint::SystemStartup,
                component_name: "test1".to_string(),
                component_type: "System".to_string(),
                language: "Native".to_string(),
                correlation_id: "test-id-1".to_string(),
                user_id: None,
                session_id: None,
                source_ip: None,
                description: "Info event".to_string(),
                details: HashMap::new(),
                blocked: false,
            },
            SecurityEvent {
                timestamp: Utc::now(),
                event_type: SecurityEventType::SecurityViolation,
                severity: SecuritySeverity::Critical,
                hook_point: HookPoint::SecurityViolation,
                component_name: "test2".to_string(),
                component_type: "System".to_string(),
                language: "Native".to_string(),
                correlation_id: "test-id-2".to_string(),
                user_id: None,
                session_id: None,
                source_ip: None,
                description: "Critical event".to_string(),
                details: HashMap::new(),
                blocked: true,
            },
        ];

        for event in events {
            storage.add_event(event);
        }

        // Test filtering by severity
        let critical_events = storage.get_events_by_severity(SecuritySeverity::Critical);
        assert_eq!(critical_events.len(), 1);
        assert_eq!(critical_events[0].severity, SecuritySeverity::Critical);

        // Test filtering by type
        let violation_events = storage.get_events_by_type(&SecurityEventType::SecurityViolation);
        assert_eq!(violation_events.len(), 1);
        assert_eq!(
            violation_events[0].event_type,
            SecurityEventType::SecurityViolation
        );
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_security_statistics() {
        let storage = SecurityStorage::new(SecurityConfig::default());

        // Add multiple events
        for i in 0..5 {
            let event = SecurityEvent {
                timestamp: Utc::now(),
                event_type: if i % 2 == 0 {
                    SecurityEventType::AccessAttempt
                } else {
                    SecurityEventType::AuditLog
                },
                severity: if i < 2 {
                    SecuritySeverity::Info
                } else {
                    SecuritySeverity::High
                },
                hook_point: HookPoint::SystemStartup,
                component_name: format!("test{}", i),
                component_type: "System".to_string(),
                language: "Native".to_string(),
                correlation_id: format!("test-id-{}", i),
                user_id: None,
                session_id: None,
                source_ip: None,
                description: format!("Test event {}", i),
                details: HashMap::new(),
                blocked: i == 0, // Only first event is blocked
            };
            storage.add_event(event);
        }

        let stats = storage.get_statistics();

        assert_eq!(stats.get("total_events").unwrap().as_u64().unwrap(), 5);
        assert!(stats.contains_key("by_severity"));
        assert!(stats.contains_key("by_type"));
        assert_eq!(stats.get("blocked_events").unwrap().as_u64().unwrap(), 1);
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_metric_hook_trait() {
        let hook = SecurityHook::new();
        let component_id = ComponentId::new(ComponentType::Tool, "test-tool".to_string());
        let context = HookContext::new(HookPoint::BeforeToolExecution, component_id);

        // Test MetricHook implementation
        hook.record_pre_execution(&context).await.unwrap();

        let result = HookResult::Continue;
        hook.record_post_execution(&context, &result, std::time::Duration::from_millis(15))
            .await
            .unwrap();

        // Verify security events were recorded
        let events = hook.get_events();
        let access_attempts: Vec<_> = events
            .iter()
            .filter(|e| e.event_type == SecurityEventType::AccessAttempt)
            .collect();
        assert!(!access_attempts.is_empty());
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_hook_metadata() {
        let hook = SecurityHook::new();
        let metadata = hook.metadata();

        assert_eq!(metadata.name, "SecurityHook");
        assert!(metadata.description.is_some());
        assert_eq!(metadata.priority, Priority::HIGHEST);
        assert_eq!(metadata.language, Language::Native);
        assert!(metadata.tags.contains(&"builtin".to_string()));
        assert!(metadata.tags.contains(&"security".to_string()));
    }
}

#[async_trait]
impl ReplayableHook for SecurityHook {
    fn is_replayable(&self) -> bool {
        true
    }

    fn serialize_context(&self, ctx: &HookContext) -> Result<Vec<u8>> {
        // Create a serializable version of the context with security config
        let mut context_data = ctx.data.clone();

        // Add security configuration for replay (excluding sensitive data)
        context_data.insert(
            "_security_config".to_string(),
            serde_json::json!({
                "enable_audit_logging": self.storage.config.enable_audit_logging,
                "enable_parameter_validation": self.storage.config.enable_parameter_validation,
                "enable_rate_limiting": self.storage.config.enable_rate_limiting,
                "min_severity": serde_json::to_value(self.storage.config.min_severity)?,
                "block_on_violations": self.storage.config.block_on_violations,
                "max_parameter_length": self.storage.config.max_parameter_length,
                // Note: We don't serialize sensitive_parameters for security reasons
                "sensitive_parameters_count": self.storage.config.sensitive_parameters.len(),
            }),
        );

        // Add security event summary (not full events for privacy)
        let events = self.storage.events.read().unwrap();
        let event_summary = serde_json::json!({
            "total_events": events.len(),
            "events_by_severity": {
                "info": events.iter().filter(|e| e.severity == SecuritySeverity::Info).count(),
                "low": events.iter().filter(|e| e.severity == SecuritySeverity::Low).count(),
                "medium": events.iter().filter(|e| e.severity == SecuritySeverity::Medium).count(),
                "high": events.iter().filter(|e| e.severity == SecuritySeverity::High).count(),
                "critical": events.iter().filter(|e| e.severity == SecuritySeverity::Critical).count(),
            },
            "blocked_events": events.iter().filter(|e| e.blocked).count(),
        });
        context_data.insert("_security_event_summary".to_string(), event_summary);

        let mut replay_context = ctx.clone();
        replay_context.data = context_data;

        // Mask any sensitive data in the context before serialization
        for (key, value) in &mut replay_context.data {
            if self.storage.config.sensitive_parameters.contains(key) {
                *value = serde_json::Value::String("[REDACTED]".to_string());
            }
        }

        Ok(serde_json::to_vec(&replay_context)?)
    }

    fn deserialize_context(&self, data: &[u8]) -> Result<HookContext> {
        let mut context: HookContext = serde_json::from_slice(data)?;

        // Remove the security-specific data from context
        context.data.remove("_security_config");
        context.data.remove("_security_event_summary");

        Ok(context)
    }

    fn replay_id(&self) -> String {
        format!("{}:{}", self.metadata.name, self.metadata.version)
    }
}
