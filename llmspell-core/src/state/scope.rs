// ABOUTME: State scope definitions for hierarchical state organization
// ABOUTME: Provides scoping mechanisms for different types of state data

use serde::{Deserialize, Serialize};
use std::fmt;

/// Hierarchical scope for state data organization
///
/// State scopes provide a way to organize state data into logical hierarchies,
/// allowing different components to maintain their state separately while
/// enabling controlled sharing and access patterns.
///
/// # Examples
///
/// ```
/// use llmspell_state_traits::StateScope;
///
/// // Global application state
/// let global_scope = StateScope::Global;
///
/// // User-specific state
/// let user_scope = StateScope::User("alice".to_string());
///
/// // Agent-specific state
/// let agent_scope = StateScope::Agent("assistant-1".to_string());
///
/// // Custom component state
/// let custom_scope = StateScope::Custom("tool_calculator".to_string());
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StateScope {
    /// Global application-wide scope
    ///
    /// Used for system-wide configuration, feature flags, and shared state
    /// that affects all components.
    Global,

    /// User-specific scope
    ///
    /// Used for user preferences, session data, and per-user configuration.
    /// The string identifier should be a unique user ID.
    User(String),

    /// Session-specific scope
    ///
    /// Used for temporary session data that should not persist beyond
    /// the current session. The string identifier should be a session ID.
    Session(String),

    /// Agent-specific scope
    ///
    /// Used for agent configuration, conversation history, and agent-specific
    /// state. The string identifier should be a unique agent ID.
    Agent(String),

    /// Tool-specific scope
    ///
    /// Used for tool configuration, cached results, and tool-specific state.
    /// The string identifier should be a unique tool ID.
    Tool(String),

    /// Workflow-specific scope
    ///
    /// Used for workflow execution state, checkpoints, and workflow-specific
    /// data. The string identifier should be a unique workflow ID.
    Workflow(String),

    /// Hook-specific scope
    ///
    /// Used for hook metadata, execution history, and hook-specific state.
    /// The string identifier should be a unique hook ID or type.
    Hook(String),

    /// Custom component scope
    ///
    /// Used for any other component that needs isolated state storage.
    /// The string identifier should uniquely identify the component instance.
    Custom(String),
}

impl StateScope {
    /// Check if this scope is global
    #[must_use]
    pub fn is_global(&self) -> bool {
        matches!(self, StateScope::Global)
    }

    /// Check if this scope is user-specific
    #[must_use]
    pub fn is_user_scope(&self) -> bool {
        matches!(self, StateScope::User(_))
    }

    /// Check if this scope is session-specific
    #[must_use]
    pub fn is_session_scope(&self) -> bool {
        matches!(self, StateScope::Session(_))
    }

    /// Check if this scope is agent-specific
    #[must_use]
    pub fn is_agent_scope(&self) -> bool {
        matches!(self, StateScope::Agent(_))
    }

    /// Check if this scope is tool-specific
    #[must_use]
    pub fn is_tool_scope(&self) -> bool {
        matches!(self, StateScope::Tool(_))
    }

    /// Check if this scope is workflow-specific
    #[must_use]
    pub fn is_workflow_scope(&self) -> bool {
        matches!(self, StateScope::Workflow(_))
    }

    /// Check if this scope is hook-specific
    #[must_use]
    pub fn is_hook_scope(&self) -> bool {
        matches!(self, StateScope::Hook(_))
    }

    /// Check if this scope is custom
    #[must_use]
    pub fn is_custom_scope(&self) -> bool {
        matches!(self, StateScope::Custom(_))
    }

    /// Get the scope identifier if it has one
    #[must_use]
    pub fn identifier(&self) -> Option<&str> {
        match self {
            StateScope::Global => None,
            StateScope::User(id)
            | StateScope::Session(id)
            | StateScope::Agent(id)
            | StateScope::Tool(id)
            | StateScope::Workflow(id)
            | StateScope::Hook(id)
            | StateScope::Custom(id) => Some(id),
        }
    }

    /// Get the scope type as a string
    #[must_use]
    pub fn scope_type(&self) -> &'static str {
        match self {
            StateScope::Global => "global",
            StateScope::User(_) => "user",
            StateScope::Session(_) => "session",
            StateScope::Agent(_) => "agent",
            StateScope::Tool(_) => "tool",
            StateScope::Workflow(_) => "workflow",
            StateScope::Hook(_) => "hook",
            StateScope::Custom(_) => "custom",
        }
    }

    /// Check if this scope can access data from another scope
    ///
    /// This implements a basic access control model:
    /// - Global scope can access everything
    /// - User scopes can access their own data and global data
    /// - Session scopes can access their own data, their user's data, and global data
    /// - Component scopes (agent, tool, workflow, hook, custom) can access their own data and global data
    #[must_use]
    pub fn can_access(&self, target_scope: &StateScope) -> bool {
        match (self, target_scope) {
            // Global can access everything
            (StateScope::Global, _) => true,

            // Same scope can always access itself
            (a, b) if a == b => true,

            // Any scope can access global
            (_, StateScope::Global) => true,

            // User can access their sessions
            (StateScope::User(user_id), StateScope::Session(session_id)) => {
                // In a real implementation, you'd check if the session belongs to the user
                // For now, we assume session IDs contain user IDs or have a mapping
                session_id.starts_with(user_id)
            }

            // Sessions can access their user's data
            (StateScope::Session(session_id), StateScope::User(user_id)) => {
                session_id.starts_with(user_id)
            }

            // Component scopes cannot access each other by default
            _ => false,
        }
    }

    /// Create a hierarchical key for storage
    ///
    /// This creates a storage key that includes the scope information,
    /// allowing for efficient querying and organization in storage backends.
    #[must_use]
    pub fn storage_key(&self, key: &str) -> String {
        match self {
            StateScope::Global => format!("global:{key}"),
            StateScope::User(id) => format!("user:{id}:{key}"),
            StateScope::Session(id) => format!("session:{id}:{key}"),
            StateScope::Agent(id) => format!("agent:{id}:{key}"),
            StateScope::Tool(id) => format!("tool:{id}:{key}"),
            StateScope::Workflow(id) => format!("workflow:{id}:{key}"),
            StateScope::Hook(id) => format!("hook:{id}:{key}"),
            StateScope::Custom(id) => format!("custom:{id}:{key}"),
        }
    }

    /// Parse a storage key back into scope and key components
    #[must_use]
    pub fn parse_storage_key(storage_key: &str) -> Option<(StateScope, String)> {
        let parts: Vec<&str> = storage_key.splitn(3, ':').collect();

        match parts.as_slice() {
            ["global", key] => Some((StateScope::Global, (*key).to_string())),
            ["user", id, key] => Some((StateScope::User((*id).to_string()), (*key).to_string())),
            ["session", id, key] => {
                Some((StateScope::Session((*id).to_string()), (*key).to_string()))
            }
            ["agent", id, key] => Some((StateScope::Agent((*id).to_string()), (*key).to_string())),
            ["tool", id, key] => Some((StateScope::Tool((*id).to_string()), (*key).to_string())),
            ["workflow", id, key] => {
                Some((StateScope::Workflow((*id).to_string()), (*key).to_string()))
            }
            ["hook", id, key] => Some((StateScope::Hook((*id).to_string()), (*key).to_string())),
            ["custom", id, key] => {
                Some((StateScope::Custom((*id).to_string()), (*key).to_string()))
            }
            _ => None,
        }
    }

    /// Get the prefix for this scope (used by `KeyManager`)
    #[must_use]
    pub fn prefix(&self) -> String {
        match self {
            StateScope::Global => "global:".to_string(),
            StateScope::User(id) => format!("user:{id}:"),
            StateScope::Session(id) => format!("session:{id}:"),
            StateScope::Agent(id) => format!("agent:{id}:"),
            StateScope::Tool(id) => format!("tool:{id}:"),
            StateScope::Workflow(id) => format!("workflow:{id}:"),
            StateScope::Hook(id) => format!("hook:{id}:"),
            StateScope::Custom(id) => format!("custom:{id}:"),
        }
    }

    /// Get the parent scope if this scope has one
    #[must_use]
    pub fn parent(&self) -> Option<StateScope> {
        match self {
            StateScope::Global => None,
            StateScope::Session(session_id) => {
                // Extract user ID from session ID if it follows the pattern "user_session_*"
                if let Some(user_id) = session_id.split('_').next() {
                    Some(StateScope::User(user_id.to_string()))
                } else {
                    Some(StateScope::Global)
                }
            }
            StateScope::User(_)
            | StateScope::Agent(_)
            | StateScope::Tool(_)
            | StateScope::Workflow(_)
            | StateScope::Hook(_)
            | StateScope::Custom(_) => Some(StateScope::Global),
        }
    }
}

impl fmt::Display for StateScope {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StateScope::Global => write!(f, "global"),
            StateScope::User(id) => write!(f, "user:{id}"),
            StateScope::Session(id) => write!(f, "session:{id}"),
            StateScope::Agent(id) => write!(f, "agent:{id}"),
            StateScope::Tool(id) => write!(f, "tool:{id}"),
            StateScope::Workflow(id) => write!(f, "workflow:{id}"),
            StateScope::Hook(id) => write!(f, "hook:{id}"),
            StateScope::Custom(id) => write!(f, "custom:{id}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_scope_types() {
        assert!(StateScope::Global.is_global());
        assert!(StateScope::User("alice".to_string()).is_user_scope());
        assert!(StateScope::Agent("agent-1".to_string()).is_agent_scope());
    }
    #[test]
    fn test_scope_identifiers() {
        assert_eq!(StateScope::Global.identifier(), None);
        assert_eq!(
            StateScope::User("alice".to_string()).identifier(),
            Some("alice")
        );
        assert_eq!(
            StateScope::Agent("agent-1".to_string()).identifier(),
            Some("agent-1")
        );
    }
    #[test]
    fn test_access_control() {
        let global = StateScope::Global;
        let user = StateScope::User("alice".to_string());
        let session = StateScope::Session("alice_session_123".to_string());
        let agent = StateScope::Agent("agent-1".to_string());

        // Global can access everything
        assert!(global.can_access(&user));
        assert!(global.can_access(&session));
        assert!(global.can_access(&agent));

        // Everyone can access global
        assert!(user.can_access(&global));
        assert!(session.can_access(&global));
        assert!(agent.can_access(&global));

        // User can access their sessions
        assert!(user.can_access(&session));
        assert!(session.can_access(&user));

        // Components cannot access each other
        assert!(!agent.can_access(&user));
        assert!(!user.can_access(&agent));
    }
    #[test]
    fn test_storage_keys() {
        let global = StateScope::Global;
        let user = StateScope::User("alice".to_string());
        let agent = StateScope::Agent("agent-1".to_string());

        assert_eq!(global.storage_key("config"), "global:config");
        assert_eq!(user.storage_key("preferences"), "user:alice:preferences");
        assert_eq!(agent.storage_key("history"), "agent:agent-1:history");
    }
    #[test]
    fn test_storage_key_parsing() {
        let test_cases = vec![
            ("global:config", StateScope::Global, "config"),
            (
                "user:alice:preferences",
                StateScope::User("alice".to_string()),
                "preferences",
            ),
            (
                "agent:agent-1:history",
                StateScope::Agent("agent-1".to_string()),
                "history",
            ),
        ];

        for (storage_key, expected_scope, expected_key) in test_cases {
            let (scope, key) = StateScope::parse_storage_key(storage_key)
                .expect("valid storage key should parse successfully");
            assert_eq!(scope, expected_scope);
            assert_eq!(key, expected_key);
        }

        // Test invalid keys
        assert!(StateScope::parse_storage_key("invalid").is_none());
        assert!(StateScope::parse_storage_key("unknown:type:key").is_none());
    }
    #[test]
    fn test_display() {
        assert_eq!(StateScope::Global.to_string(), "global");
        assert_eq!(
            StateScope::User("alice".to_string()).to_string(),
            "user:alice"
        );
        assert_eq!(
            StateScope::Agent("agent-1".to_string()).to_string(),
            "agent:agent-1"
        );
    }
}
