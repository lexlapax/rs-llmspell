//! ABOUTME: llmspell-providers implementation crate
//! ABOUTME: Provider abstraction layer and LLM provider implementations

pub mod abstraction;
pub mod rig;

// Re-export main types
pub use abstraction::{
    ProviderCapabilities,
    ProviderConfig,
    ProviderInstance,
    ProviderRegistry,
    ProviderManager,
};

// Re-export provider factories
pub use rig::create_rig_provider;
