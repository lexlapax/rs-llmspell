//! ABOUTME: State management types and structures for llmspell-core
//! ABOUTME: Provides artifact correlation, state traits, and core state interfaces

pub mod artifact_correlation;
pub mod error;
pub mod scope;
pub mod traits;

pub use artifact_correlation::{
    ArtifactCorrelation, ArtifactCorrelationManager, ArtifactId, ArtifactMetadata,
    ArtifactRelationship, StateOperation,
};

// Re-export state traits (consolidated from llmspell-state-traits)
pub use error::{StateError, StateResult};
pub use scope::StateScope;
pub use traits::{
    ComponentStateBackup, ComponentStateMetadata, ComponentStatePersistence, StateManager,
    StateMigration, StateObserver, StateObserverRegistry, StatePersistence, StateTransaction,
    TransactionId, TypedStatePersistence,
};
