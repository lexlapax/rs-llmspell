//! ABOUTME: llmspell-providers implementation crate
//! ABOUTME: Provider abstraction layer and LLM provider implementations

pub mod abstraction;
pub mod local;
pub mod model_specifier;
pub mod rig;

// Re-export main types
pub use abstraction::{
    ProviderCapabilities, ProviderConfig, ProviderInstance, ProviderManager, ProviderRegistry,
};
pub use model_specifier::ModelSpecifier;

// Re-export local provider types
pub use local::{
    DownloadStatus, HealthStatus, LocalModel, LocalProviderInstance, ModelInfo, ModelSpec,
    PullProgress,
};

// Re-export provider factories
pub use local::create_candle_provider;
pub use local::create_ollama_provider;
pub use rig::create_rig_provider;
