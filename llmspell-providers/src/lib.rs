//! ABOUTME: llmspell-providers implementation crate
//! ABOUTME: Provider abstraction layer and LLM provider implementations

pub mod abstraction;
pub mod model_specifier;
pub mod rig;

// Re-export main types
pub use abstraction::{
    ProviderCapabilities, ProviderConfig, ProviderInstance, ProviderManager, ProviderRegistry,
};
pub use model_specifier::ModelSpecifier;

// Re-export provider factories
pub use rig::create_rig_provider;
