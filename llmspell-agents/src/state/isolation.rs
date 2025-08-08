#![allow(clippy::significant_drop_tightening)]
// ABOUTME: Agent state isolation enforcement for multi-agent security
// ABOUTME: Prevents unauthorized cross-agent state access with permission controls

use anyhow::Result;
use llmspell_core::traits::agent::Agent;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tracing::{debug, instrument, warn};

/// State scope for isolation
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StateScope {
    Global,
    Agent(String),
    Workflow(String),
    Step {
        workflow_id: String,
        step_name: String,
    },
    Session(String),
    Custom(String),
}

impl StateScope {
    #[must_use]
    pub fn parent(&self) -> Option<Self> {
        match self {
            Self::Step { workflow_id, .. } => Some(Self::Workflow(workflow_id.clone())),
            Self::Workflow(_) | Self::Agent(_) | Self::Session(_) => Some(Self::Global),
            Self::Global | Self::Custom(_) => None,
        }
    }

    #[must_use]
    pub fn prefix(&self) -> String {
        match self {
            Self::Global => String::new(),
            Self::Agent(id) => format!("agent:{id}:"),
            Self::Workflow(id) => format!("workflow:{id}:"),
            Self::Step {
                workflow_id,
                step_name,
            } => format!("workflow:{workflow_id}:step:{step_name}:"),
            Self::Session(id) => format!("session:{id}:"),
            Self::Custom(prefix) => format!("{prefix}:"),
        }
    }
}

/// State permission types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StatePermission {
    Read,
    Write,
    Delete,
    List,
}

/// Isolation boundary types
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum IsolationBoundary {
    /// Complete isolation - no cross-agent access
    Strict,
    /// Allow read-only access to shared scopes
    ReadOnlyShared,
    /// Allow full access to shared scopes
    SharedAccess,
    /// Custom boundary with specific permissions
    Custom(String),
}

/// State access audit entry
#[derive(Debug, Clone)]
pub struct StateAccessAudit {
    pub timestamp: SystemTime,
    pub agent_id: String,
    pub target_scope: StateScope,
    pub operation: StateOperation,
    pub key: String,
    pub allowed: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Clone)]
pub enum StateOperation {
    Read,
    Write,
    Delete,
    List,
}

/// Access control system for state operations
#[derive(Default)]
pub struct StateAccessControl {
    permissions: HashMap<(StateScope, String), Vec<StatePermission>>,
}

impl StateAccessControl {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn grant_permission(
        &mut self,
        agent_id: &str,
        scope: StateScope,
        permission: StatePermission,
    ) {
        let key = (scope, agent_id.to_string());
        let perms = self.permissions.entry(key).or_default();
        if !perms.contains(&permission) {
            perms.push(permission);
        }
    }

    #[must_use]
    pub fn has_permission(
        &self,
        agent_id: &str,
        scope: &StateScope,
        permission: &StatePermission,
    ) -> bool {
        if let Some(perms) = self.permissions.get(&(scope.clone(), agent_id.to_string())) {
            if perms.contains(permission) {
                return true;
            }
        }

        // Check parent scope permissions
        scope.parent().map_or(false, |parent| {
            self.has_permission(agent_id, &parent, permission)
        })
    }

    pub fn revoke_permissions(&mut self, agent_id: &str, scope: StateScope) {
        self.permissions.remove(&(scope, agent_id.to_string()));
    }
}

/// Agent state isolation manager
pub struct StateIsolationManager {
    /// State manager instance (using trait object to avoid circular dependency)
    #[allow(dead_code)]
    state_manager: Arc<dyn std::any::Any + Send + Sync>,
    /// Access control system
    access_control: Arc<RwLock<StateAccessControl>>,
    /// Isolation boundaries per agent
    boundaries: Arc<RwLock<HashMap<String, IsolationBoundary>>>,
    /// Audit log of state access attempts
    audit_log: Arc<RwLock<Vec<StateAccessAudit>>>,
    /// Shared scope registry
    shared_scopes: Arc<RwLock<HashMap<String, SharedScopeConfig>>>,
}

/// Configuration for shared state scopes
#[derive(Debug, Clone)]
pub struct SharedScopeConfig {
    pub scope_id: String,
    pub owner_agent_id: Option<String>,
    pub allowed_agents: Vec<String>,
    pub permissions: HashMap<String, Vec<StatePermission>>,
    pub created_at: SystemTime,
    pub expires_at: Option<SystemTime>,
}

impl StateIsolationManager {
    pub fn new<T: std::any::Any + Send + Sync + 'static>(state_manager: Arc<T>) -> Self {
        Self {
            state_manager,
            access_control: Arc::new(RwLock::new(StateAccessControl::new())),
            boundaries: Arc::new(RwLock::new(HashMap::new())),
            audit_log: Arc::new(RwLock::new(Vec::new())),
            shared_scopes: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Set isolation boundary for an agent
    #[instrument(skip(self))]
    pub fn set_agent_boundary(&self, agent_id: &str, boundary: IsolationBoundary) {
        let mut boundaries = self.boundaries.write();
        boundaries.insert(agent_id.to_string(), boundary);
        debug!(
            "Set isolation boundary for agent {}: {:?}",
            agent_id,
            boundaries.get(agent_id)
        );
    }

    /// Check if an agent can access a specific state scope
    ///
    /// # Errors
    ///
    /// Returns an error if access check fails
    #[instrument(skip(self))]
    pub fn check_access(
        &self,
        agent_id: &str,
        target_scope: &StateScope,
        operation: StateOperation,
    ) -> Result<bool> {
        let start_time = std::time::Instant::now();

        // Check if agent owns the scope
        if Self::agent_owns_scope(agent_id, target_scope) {
            self.audit_access(
                agent_id,
                target_scope,
                operation,
                true,
                Some("Owner access"),
            );
            return Ok(true);
        }

        // Get agent's isolation boundary
        let boundaries = self.boundaries.read();
        let boundary = boundaries
            .get(agent_id)
            .unwrap_or(&IsolationBoundary::Strict);

        let allowed = match boundary {
            IsolationBoundary::Strict => {
                // Only allow access to agent's own scope
                false
            }
            IsolationBoundary::ReadOnlyShared => {
                // Allow read-only access to shared scopes
                matches!(operation, StateOperation::Read) && self.is_shared_scope(target_scope)
            }
            IsolationBoundary::SharedAccess => {
                // Allow full access to shared scopes
                self.is_shared_scope(target_scope)
            }
            IsolationBoundary::Custom(policy) => {
                // Check custom policy
                self.check_custom_policy(agent_id, target_scope, &operation, policy)
            }
        };

        // Check explicit permissions
        let permission = match operation {
            StateOperation::Read => StatePermission::Read,
            StateOperation::Write => StatePermission::Write,
            StateOperation::Delete => StatePermission::Delete,
            StateOperation::List => StatePermission::List,
        };

        let access_control = self.access_control.read();
        let has_permission = access_control.has_permission(agent_id, target_scope, &permission);

        let final_allowed = allowed || has_permission;

        // Audit the access attempt
        self.audit_access(
            agent_id,
            target_scope,
            operation,
            final_allowed,
            if final_allowed {
                None
            } else {
                Some("Access denied by isolation policy")
            },
        );

        // Check performance
        let duration = start_time.elapsed();
        if duration > Duration::from_millis(1) {
            warn!("State isolation check took {:?} (>1ms threshold)", duration);
        }

        Ok(final_allowed)
    }

    /// Create a shared scope for multiple agents
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The shared scope already exists
    /// - Creation fails
    #[instrument(skip(self))]
    pub fn create_shared_scope(
        &self,
        scope_id: &str,
        owner_agent_id: Option<&str>,
        config: &SharedScopeConfig,
    ) -> Result<()> {
        let mut shared_scopes = self.shared_scopes.write();

        if shared_scopes.contains_key(scope_id) {
            return Err(anyhow::anyhow!("Shared scope {} already exists", scope_id));
        }

        shared_scopes.insert(scope_id.to_string(), config.clone());

        // Grant permissions to allowed agents
        let mut access_control = self.access_control.write();
        for agent_id in &config.allowed_agents {
            if let Some(permissions) = config.permissions.get(agent_id) {
                for permission in permissions {
                    access_control.grant_permission(
                        agent_id,
                        StateScope::Custom(format!("shared:{scope_id}")),
                        permission.clone(),
                    );
                }
            }
        }

        debug!(
            "Created shared scope {} with {} allowed agents",
            scope_id,
            config.allowed_agents.len()
        );
        Ok(())
    }

    /// Remove a shared scope
    ///
    /// # Errors
    ///
    /// Returns an error if the shared scope is not found
    #[instrument(skip(self))]
    pub fn remove_shared_scope(&self, scope_id: &str) -> Result<()> {
        let mut shared_scopes = self.shared_scopes.write();

        shared_scopes.remove(scope_id).map_or_else(
            || Err(anyhow::anyhow!("Shared scope {} not found", scope_id)),
            |config| {
                // Revoke permissions from all agents
                let mut access_control = self.access_control.write();
                for agent_id in &config.allowed_agents {
                    access_control.revoke_permissions(
                        agent_id,
                        StateScope::Custom(format!("shared:{scope_id}")),
                    );
                }
                debug!("Removed shared scope {}", scope_id);
                Ok(())
            },
        )
    }

    /// Grant specific permission to an agent for a scope
    #[instrument(skip(self))]
    pub fn grant_permission(&self, agent_id: &str, scope: &StateScope, permission: StatePermission) {
        let mut access_control = self.access_control.write();
        access_control.grant_permission(agent_id, scope.clone(), permission.clone());
        debug!(
            "Granted {:?} permission to agent {} for scope {:?}",
            permission, agent_id, scope
        );
    }

    /// Revoke all permissions for an agent in a scope
    #[instrument(skip(self))]
    pub fn revoke_permissions(&self, agent_id: &str, scope: &StateScope) {
        let mut access_control = self.access_control.write();
        access_control.revoke_permissions(agent_id, scope.clone());
        debug!(
            "Revoked all permissions for agent {} in scope {:?}",
            agent_id, scope
        );
    }

    /// Get audit log entries
    #[must_use]
    pub fn get_audit_log(&self, limit: Option<usize>) -> Vec<StateAccessAudit> {
        let audit_log = self.audit_log.read();
        limit.map_or_else(
            || audit_log.clone(),
            |n| audit_log.iter().rev().take(n).cloned().collect(),
        )
    }

    /// Clear old audit log entries
    pub fn cleanup_audit_log(&self, older_than: Duration) {
        let cutoff_time = SystemTime::now() - older_than;
        let mut audit_log = self.audit_log.write();
        audit_log.retain(|entry| entry.timestamp > cutoff_time);
    }

    // Private helper methods

    fn agent_owns_scope(agent_id: &str, scope: &StateScope) -> bool {
        match scope {
            StateScope::Agent(id) => id == agent_id,
            StateScope::Custom(s) if s.starts_with(&format!("agent:{agent_id}:")) => true,
            _ => false,
        }
    }

    fn is_shared_scope(&self, scope: &StateScope) -> bool {
        match scope {
            StateScope::Global => true,
            StateScope::Custom(s) if s.starts_with("shared:") => {
                let shared_scopes = self.shared_scopes.read();
                let scope_id = s.strip_prefix("shared:").unwrap_or("");
                shared_scopes.contains_key(scope_id)
            }
            _ => false,
        }
    }

    fn check_custom_policy(
        &self,
        _agent_id: &str,
        target_scope: &StateScope,
        operation: &StateOperation,
        policy: &str,
    ) -> bool {
        // Implement custom policy evaluation
        // For now, just check if policy allows the operation
        match policy {
            "read-all" => matches!(operation, StateOperation::Read),
            "write-shared" => {
                matches!(operation, StateOperation::Write) && self.is_shared_scope(target_scope)
            }
            _ => false,
        }
    }

    fn audit_access(
        &self,
        agent_id: &str,
        target_scope: &StateScope,
        operation: StateOperation,
        allowed: bool,
        reason: Option<&str>,
    ) {
        let entry = StateAccessAudit {
            timestamp: SystemTime::now(),
            agent_id: agent_id.to_string(),
            target_scope: target_scope.clone(),
            operation,
            key: String::new(), // Will be filled by actual access
            allowed,
            reason: reason.map(String::from),
        };

        let mut audit_log = self.audit_log.write();
        audit_log.push(entry);

        // Warn on denied access attempts
        if !allowed {
            warn!(
                "State access denied: agent {} attempted {:?} on scope {:?}",
                agent_id,
                audit_log.last().unwrap().operation,
                target_scope
            );
        }
    }
}

/// Extension trait for agents to use isolated state
#[async_trait::async_trait]
pub trait IsolatedStateAgent: Agent {
    /// Get isolated state accessor
    fn isolated_state(&self) -> IsolatedStateAccessor
    where
        Self: Sized,
    {
        IsolatedStateAccessor::new(
            self.metadata().id.to_string(),
            self.state_manager(),
            self.isolation_manager(),
        )
    }

    /// Get state manager (to be implemented by agent)
    fn state_manager(&self) -> Arc<dyn std::any::Any + Send + Sync>;

    /// Get isolation manager (to be implemented by agent)
    fn isolation_manager(&self) -> Arc<StateIsolationManager>;
}

/// Isolated state accessor that enforces permissions
pub struct IsolatedStateAccessor {
    agent_id: String,
    #[allow(dead_code)]
    state_manager: Arc<dyn std::any::Any + Send + Sync>,
    isolation_manager: Arc<StateIsolationManager>,
}

impl IsolatedStateAccessor {
    pub fn new(
        agent_id: String,
        state_manager: Arc<dyn std::any::Any + Send + Sync>,
        isolation_manager: Arc<StateIsolationManager>,
    ) -> Self {
        Self {
            agent_id,
            state_manager,
            isolation_manager,
        }
    }

    /// Get state with isolation check
    ///
    /// # Errors
    ///
    /// Returns an error if access is denied to the scope
    pub fn get(&self, scope: &StateScope, _key: &str) -> Result<Option<serde_json::Value>> {
        // Check access permission
        if !self
            .isolation_manager
            .check_access(&self.agent_id, scope, StateOperation::Read)?
        {
            return Err(anyhow::anyhow!("Access denied to scope {:?}", scope));
        }

        // Perform actual state read (would need proper implementation with StateManager)
        // For now, return a placeholder
        Ok(None)
    }

    /// Set state with isolation check
    ///
    /// # Errors
    ///
    /// Returns an error if access is denied to the scope
    pub fn set(&self, scope: &StateScope, _key: &str, _value: &serde_json::Value) -> Result<()> {
        // Check access permission
        if !self
            .isolation_manager
            .check_access(&self.agent_id, scope, StateOperation::Write)?
        {
            return Err(anyhow::anyhow!("Access denied to scope {:?}", scope));
        }

        // Perform actual state write (would need proper implementation with StateManager)
        Ok(())
    }

    /// Delete state with isolation check
    ///
    /// # Errors
    ///
    /// Returns an error if access is denied to the scope
    pub fn delete(&self, scope: &StateScope, _key: &str) -> Result<()> {
        // Check access permission
        if !self
            .isolation_manager
            .check_access(&self.agent_id, scope, StateOperation::Delete)?
        {
            return Err(anyhow::anyhow!("Access denied to scope {:?}", scope));
        }

        // Perform actual state deletion (would need proper implementation with StateManager)
        Ok(())
    }

    /// List keys with isolation check
    ///
    /// # Errors
    ///
    /// Returns an error if access is denied to the scope
    pub fn list_keys(&self, scope: &StateScope) -> Result<Vec<String>> {
        // Check access permission
        if !self
            .isolation_manager
            .check_access(&self.agent_id, scope, StateOperation::List)?
        {
            return Err(anyhow::anyhow!("Access denied to scope {:?}", scope));
        }

        // Perform actual key listing (would need proper implementation with StateManager)
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Mock state manager for testing
    struct MockStateManager;
    #[test]
    fn test_strict_isolation() {
        let state_manager = Arc::new(MockStateManager);
        let isolation_manager = StateIsolationManager::new(state_manager);

        // Set strict isolation for agent1
        isolation_manager.set_agent_boundary("agent1", IsolationBoundary::Strict);

        // Agent1 can access its own scope
        assert!(isolation_manager
            .check_access(
                "agent1",
                &StateScope::Agent("agent1".to_string()),
                StateOperation::Read
            )
            .unwrap());

        // Agent1 cannot access agent2's scope
        assert!(!isolation_manager
            .check_access(
                "agent1",
                &StateScope::Agent("agent2".to_string()),
                StateOperation::Read
            )
            .unwrap());

        // Agent1 cannot access global scope under strict isolation
        assert!(!isolation_manager
            .check_access("agent1", &StateScope::Global, StateOperation::Read)
            .unwrap());
    }
    #[test]
    fn test_shared_scope_access() {
        let state_manager = Arc::new(MockStateManager);
        let isolation_manager = StateIsolationManager::new(state_manager);

        // Create shared scope
        let config = SharedScopeConfig {
            scope_id: "team-data".to_string(),
            owner_agent_id: Some("agent1".to_string()),
            allowed_agents: vec!["agent1".to_string(), "agent2".to_string()],
            permissions: {
                let mut perms = HashMap::new();
                perms.insert(
                    "agent1".to_string(),
                    vec![StatePermission::Read, StatePermission::Write],
                );
                perms.insert("agent2".to_string(), vec![StatePermission::Read]);
                perms
            },
            created_at: SystemTime::now(),
            expires_at: None,
        };

        isolation_manager
            .create_shared_scope("team-data", Some("agent1"), &config)
            .unwrap();

        // Set boundaries
        isolation_manager.set_agent_boundary("agent1", IsolationBoundary::SharedAccess);
        isolation_manager.set_agent_boundary("agent2", IsolationBoundary::ReadOnlyShared);

        let shared_scope = StateScope::Custom("shared:team-data".to_string());

        // Agent1 can read and write
        assert!(isolation_manager
            .check_access("agent1", &shared_scope, StateOperation::Read)
            .unwrap());
        assert!(isolation_manager
            .check_access("agent1", &shared_scope, StateOperation::Write)
            .unwrap());

        // Agent2 can only read
        assert!(isolation_manager
            .check_access("agent2", &shared_scope, StateOperation::Read)
            .unwrap());
        assert!(!isolation_manager
            .check_access("agent2", &shared_scope, StateOperation::Write)
            .unwrap());

        // Agent3 has no access
        assert!(!isolation_manager
            .check_access("agent3", &shared_scope, StateOperation::Read)
            .unwrap());
    }
    #[test]
    fn test_audit_logging() {
        let state_manager = Arc::new(MockStateManager);
        let isolation_manager = StateIsolationManager::new(state_manager);

        // Perform some access checks
        let _ = isolation_manager.check_access(
            "agent1",
            &StateScope::Agent("agent2".to_string()),
            StateOperation::Read,
        );

        let _ = isolation_manager.check_access(
            "agent1",
            &StateScope::Agent("agent1".to_string()),
            StateOperation::Write,
        );

        // Check audit log
        let audit_log = isolation_manager.get_audit_log(None);
        assert_eq!(audit_log.len(), 2);
        assert!(!audit_log[0].allowed); // First access denied
        assert!(audit_log[1].allowed); // Second access allowed
    }
    #[test]
    fn test_permission_grant_revoke() {
        let state_manager = Arc::new(MockStateManager);
        let isolation_manager = StateIsolationManager::new(state_manager);

        let scope = StateScope::Custom("project:data".to_string());

        // Initially no access
        assert!(!isolation_manager
            .check_access("agent1", &scope, StateOperation::Read)
            .unwrap());

        // Grant read permission
        isolation_manager.grant_permission("agent1", &scope, StatePermission::Read);
        assert!(isolation_manager
            .check_access("agent1", &scope, StateOperation::Read)
            .unwrap());
        assert!(!isolation_manager
            .check_access("agent1", &scope, StateOperation::Write)
            .unwrap());

        // Grant write permission
        isolation_manager.grant_permission("agent1", &scope, StatePermission::Write);
        assert!(isolation_manager
            .check_access("agent1", &scope, StateOperation::Write)
            .unwrap());

        // Revoke all permissions
        isolation_manager.revoke_permissions("agent1", &scope);
        assert!(!isolation_manager
            .check_access("agent1", &scope, StateOperation::Read)
            .unwrap());
        assert!(!isolation_manager
            .check_access("agent1", &scope, StateOperation::Write)
            .unwrap());
    }
}
