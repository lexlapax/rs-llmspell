//! ABOUTME: State global object providing persistent state management
//! ABOUTME: Integrates with `StateAccess` trait for flexible state backend support

#![allow(clippy::significant_drop_tightening)]

use crate::globals::types::{GlobalContext, GlobalMetadata, GlobalObject};
use llmspell_core::{error::LLMSpellError, traits::state::StateAccess};
use llmspell_state_persistence::{
    migration::MigrationEngine, schema::SchemaRegistry, StateManager, StateScope,
};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::info;

/// State global object providing persistent state management
///
/// Integrates with `StateAccess` trait for flexible state backend support.
/// Falls back to in-memory storage when `StateAccess` is not available.
pub struct StateGlobal {
    /// `StateAccess` implementation for persistent storage (optional for backward compatibility)
    pub state_access: Option<Arc<dyn StateAccess>>,
    /// Original `StateManager` for migration/backup features (optional)
    pub state_manager: Option<Arc<StateManager>>,
    /// Fallback in-memory state storage (when `StateAccess` is not available)
    pub fallback_state: Arc<RwLock<HashMap<String, serde_json::Value>>>,
    /// Migration engine for schema transitions (optional)
    pub migration_engine: Option<Arc<MigrationEngine>>,
    /// Schema registry for migration planning (optional)
    pub schema_registry: Option<Arc<SchemaRegistry>>,
    /// Backup manager for state backup/restore operations (optional)
    pub backup_manager: Option<Arc<llmspell_state_persistence::backup::BackupManager>>,
}

impl StateGlobal {
    /// Create a new State global without state backend (fallback mode)
    #[must_use]
    pub fn new() -> Self {
        Self {
            state_access: None,
            state_manager: None,
            fallback_state: Arc::new(RwLock::new(HashMap::new())),
            migration_engine: None,
            schema_registry: None,
            backup_manager: None,
        }
    }

    /// Create a new State global with `StateAccess` integration
    pub fn with_state_access(state_access: Arc<dyn StateAccess>) -> Self {
        Self {
            state_access: Some(state_access),
            state_manager: None,
            fallback_state: Arc::new(RwLock::new(HashMap::new())),
            migration_engine: None,
            schema_registry: None,
            backup_manager: None,
        }
    }

    /// Create a new State global with `StateManager` integration (for migration/backup support)
    pub fn with_state_manager(state_manager: Arc<StateManager>) -> Self {
        // Create StateManagerAdapter to provide StateAccess interface
        let state_access: Arc<dyn StateAccess> =
            Arc::new(crate::state_adapter::StateManagerAdapter::new(
                state_manager.clone(),
                StateScope::Global,
            ));

        Self {
            state_access: Some(state_access),
            state_manager: Some(state_manager),
            fallback_state: Arc::new(RwLock::new(HashMap::new())),
            migration_engine: None,
            schema_registry: None,
            backup_manager: None,
        }
    }

    /// Create a new State global with full migration support
    pub fn with_migration_support(
        state_manager: Arc<StateManager>,
        migration_engine: Arc<MigrationEngine>,
        schema_registry: Arc<SchemaRegistry>,
    ) -> Self {
        // Create StateManagerAdapter to provide StateAccess interface
        let state_access: Arc<dyn StateAccess> =
            Arc::new(crate::state_adapter::StateManagerAdapter::new(
                state_manager.clone(),
                StateScope::Global,
            ));

        Self {
            state_access: Some(state_access),
            state_manager: Some(state_manager),
            fallback_state: Arc::new(RwLock::new(HashMap::new())),
            migration_engine: Some(migration_engine),
            schema_registry: Some(schema_registry),
            backup_manager: None,
        }
    }

    /// Create a new State global with full infrastructure support
    pub fn with_full_support(
        state_manager: Arc<StateManager>,
        migration_engine: Option<Arc<MigrationEngine>>,
        schema_registry: Option<Arc<SchemaRegistry>>,
        backup_manager: Option<Arc<llmspell_state_persistence::backup::BackupManager>>,
    ) -> Self {
        // Use StateManagerAdapter with Custom scope for StateGlobal
        // This allows reading keys that were written by NoScopeStateAdapter
        info!(
            "StateGlobal: Creating StateManagerAdapter with StateManager at {:p}",
            Arc::as_ptr(&state_manager)
        );
        let state_access: Arc<dyn StateAccess> =
            Arc::new(crate::state_adapter::StateManagerAdapter::new(
                state_manager.clone(),
                StateScope::Custom(String::new()),
            ));

        Self {
            state_access: Some(state_access),
            state_manager: Some(state_manager),
            fallback_state: Arc::new(RwLock::new(HashMap::new())),
            migration_engine,
            schema_registry,
            backup_manager,
        }
    }

    /// Helper method to parse scope string to `StateScope` enum
    #[must_use]
    pub fn parse_scope(scope_str: &str) -> StateScope {
        if scope_str.starts_with("agent:") {
            StateScope::Agent(
                scope_str
                    .strip_prefix("agent:")
                    .unwrap_or("unknown")
                    .to_string(),
            )
        } else if scope_str.starts_with("workflow:") {
            StateScope::Workflow(
                scope_str
                    .strip_prefix("workflow:")
                    .unwrap_or("unknown")
                    .to_string(),
            )
        } else if scope_str.starts_with("session:") {
            StateScope::Session(
                scope_str
                    .strip_prefix("session:")
                    .unwrap_or("unknown")
                    .to_string(),
            )
        } else if scope_str.starts_with("custom:") {
            StateScope::Custom(
                scope_str
                    .strip_prefix("custom:")
                    .unwrap_or("unknown")
                    .to_string(),
            )
        } else if scope_str == "global" {
            StateScope::Global
        } else {
            StateScope::Custom(scope_str.to_string())
        }
    }
}

impl GlobalObject for StateGlobal {
    fn metadata(&self) -> GlobalMetadata {
        let description = if self.state_access.is_some() {
            "State management system with persistent storage".to_string()
        } else {
            "State management system (in-memory fallback)".to_string()
        };

        GlobalMetadata {
            name: "State".to_string(),
            version: "2.0.0".to_string(), // Phase 7 version with StateAccess trait
            description,
            dependencies: vec![], // StateAccess is optional - gracefully falls back to in-memory
            required: false,
        }
    }

    #[cfg(feature = "lua")]
    fn inject_lua(&self, lua: &mlua::Lua, context: &GlobalContext) -> Result<(), LLMSpellError> {
        // Delegate to the inject_state_global function in lua/globals/state.rs
        crate::lua::globals::state::inject_state_global(lua, context, self).map_err(|e| {
            LLMSpellError::Component {
                message: format!("Failed to inject State global: {e}"),
                source: None,
            }
        })
    }
    #[cfg(feature = "javascript")]
    fn inject_javascript(
        &self,
        _ctx: &mut boa_engine::Context,
        _context: &GlobalContext,
    ) -> Result<(), LLMSpellError> {
        // TODO (Phase 5): JavaScript State implementation - stub for now
        Ok(())
    }
}

impl Default for StateGlobal {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_state_global_metadata() {
        let global = StateGlobal::new();
        let metadata = global.metadata();
        assert_eq!(metadata.name, "State");
        assert_eq!(metadata.version, "2.0.0"); // Phase 7 version with StateAccess trait
    }
    #[test]
    fn test_state_in_memory_storage() {
        let global = StateGlobal::new();

        // Test basic storage operations (fallback mode)
        {
            let mut state = global.fallback_state.write();
            state.insert("test_key".to_string(), serde_json::json!("test_value"));
        }

        {
            let state = global.fallback_state.read();
            assert_eq!(
                state.get("test_key"),
                Some(&serde_json::json!("test_value"))
            );
        }
    }
}
