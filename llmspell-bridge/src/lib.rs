//! ABOUTME: llmspell-bridge - Language-agnostic script runtime with bridge pattern
//! ABOUTME: Supports multiple script engines (Lua, JavaScript, Python) through ScriptEngineBridge

// Core modules
pub mod engine;
pub mod registry;
pub mod providers;
pub mod runtime;

// Language-specific implementations (feature-gated)
#[cfg(feature = "lua")]
pub mod lua;

#[cfg(feature = "javascript")]
pub mod javascript;

// Re-exports for convenience
pub use engine::{
    ScriptEngineBridge, ScriptOutput, ScriptStream, ScriptMetadata,
    EngineFeatures, ExecutionContext, SecurityContext,
    EngineFactory, EngineInfo, ScriptEnginePlugin,
    register_engine_plugin, unregister_engine_plugin,
};

pub use registry::ComponentRegistry;
pub use providers::{ProviderManager, ProviderManagerConfig};
pub use runtime::{ScriptRuntime, RuntimeConfig};
