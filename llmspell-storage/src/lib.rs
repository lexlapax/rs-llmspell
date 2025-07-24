//! ABOUTME: Storage backends and persistence for rs-llmspell
//! ABOUTME: Provides unified storage interface with multiple backend implementations

pub mod backends;
pub mod traits;

// Re-export commonly used types
pub use backends::{MemoryBackend, SledBackend};
pub use traits::{StorageBackend, StorageBackendType, StorageCharacteristics, StorageSerialize};
