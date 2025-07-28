// ABOUTME: Persistent state management for rs-llmspell with hook integration
// ABOUTME: Provides StateManager with Phase 4 hooks and Phase 3.3 storage backends

//! # State Persistence Module
//!
//! This module provides persistent state management with the following features:
//! - Multiple storage backends (Memory, Sled, RocksDB)
//! - Hook integration for state change events
//! - State scoping and isolation
//! - Schema versioning and migration support
//! - Backup and recovery capabilities
//!
//! ## Example Usage
//!
//! ```rust,ignore
//! use llmspell_state_persistence::{StateManager, StateScope, PersistenceConfig};
//! use serde_json::json;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create state manager with default in-memory backend
//!     let state_manager = StateManager::new().await?;
//!
//!     // Set state in global scope
//!     state_manager.set(StateScope::Global, "key", json!("value")).await?;
//!
//!     // Get state
//!     let value = state_manager.get(StateScope::Global, "key").await?;
//!     assert_eq!(value, Some(json!("value")));
//!
//!     // Use agent-scoped state
//!     let agent_scope = StateScope::Agent("agent123".to_string());
//!     state_manager.set(agent_scope.clone(), "config", json!({"model": "gpt-4"})).await?;
//!
//!     Ok(())
//! }
//! ```

pub mod agent_state;
pub mod backend_adapter;
pub mod backup;
pub mod circular_ref;
pub mod config;
pub mod hooks;
pub mod key_manager;
pub mod manager;
pub mod migration;
pub mod performance;
pub mod schema;
pub mod sensitive_data;

// Re-export main types
pub use agent_state::{
    AgentMetadata, AgentStateData, ConversationMessage, ExecutionState, MessageRole,
    PersistentAgent, PersistentAgentState, ToolPerformance, ToolUsageStats,
};
pub use circular_ref::{CircularReferenceCheck, CircularReferenceDetector, CircularReferenceError};
pub use config::{
    CompatibilityLevel, EncryptionAlgorithm, EncryptionConfig, FieldSchema, KeyDerivationConfig,
    MigrationStep, PerformanceConfig, PersistenceConfig, RocksDBConfig, SledConfig, StateSchema,
    StorageBackendType,
};
// Re-export from llmspell-state-traits
pub use key_manager::{KeyManager, StateAccessControl, StatePermission};
pub use llmspell_state_traits::{
    StateError, StateManager as StateManagerTrait, StatePersistence, StateResult, StateScope,
};
pub use manager::{HookReplayManager, SerializableState, SerializedHookExecution, StateManager};
pub use migration::{
    DataTransformer, MigrationConfig, MigrationEngine, MigrationResult, MigrationStatus,
    ValidationLevel, ValidationResult,
};
pub use performance::{FastPathConfig, FastPathManager, StateClass};
pub use schema::{
    CompatibilityChecker, CompatibilityResult, EnhancedStateSchema, MigrationPlan,
    MigrationPlanner, SchemaRegistry, SchemaVersion, SemanticVersion,
};
// StateScope now comes from llmspell-state-traits
pub use sensitive_data::{RedactSensitiveData, SensitiveDataConfig, SensitiveDataProtector};

pub use serde_json::{json, Value};

/// Prelude module for common imports
pub mod prelude {
    pub use crate::{
        config::{PersistenceConfig, StorageBackendType},
        StateError, StateManager, StateManagerTrait, StatePersistence, StateResult, StateScope,
    };
}
