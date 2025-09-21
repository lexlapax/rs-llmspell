//! Mock implementations for future-proofing traits
//!
//! Provides mock implementations of Phase 10-20 traits for testing
//! forward compatibility and ensuring the trait designs are sound.

pub mod debug;
pub mod memory;
pub mod observability;
pub mod service;

// Re-export commonly used mocks
pub use debug::MockMultiLanguageDebug;
pub use memory::MockMemoryIntegration;
pub use observability::MockObservabilityFramework;
pub use service::MockServiceInfrastructure;
