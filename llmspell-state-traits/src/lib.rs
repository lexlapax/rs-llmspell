//! ABOUTME: State management traits and interfaces for rs-llmspell
//! ABOUTME: Provides common state management abstractions without implementation dependencies

pub mod error;
pub mod scope;
pub mod traits;

pub use error::{StateError, StateResult};
pub use scope::StateScope;
pub use traits::{
    ComponentStateBackup, ComponentStateMetadata, ComponentStatePersistence, StateManager,
    StateMigration, StateObserver, StateObserverRegistry, StatePersistence, StateTransaction,
    TransactionId, TypedStatePersistence,
};
