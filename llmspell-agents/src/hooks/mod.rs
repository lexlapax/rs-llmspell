//! ABOUTME: Agent lifecycle hooks for extending agent behavior
//! ABOUTME: Provides hooks that run at various stages of agent lifecycle with state persistence

pub mod state_persistence_hook;

pub use state_persistence_hook::StatePersistenceHook;
