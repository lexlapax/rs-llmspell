//! ABOUTME: Language abstraction layer for script engines
//! ABOUTME: Provides `ScriptEngineBridge` trait and factory pattern for multi-language support

pub mod bridge;
pub mod factory;
pub mod types;

pub use bridge::{
    EngineFeatures, ExecutionContext, ScriptEngineBridge, ScriptMetadata, ScriptOutput,
    ScriptStream, SecurityContext,
};

pub use factory::{
    register_engine_plugin, unregister_engine_plugin, EngineFactory, EngineInfo, JSConfig,
    LuaConfig, ModuleResolution, ScriptEnginePlugin, StdlibLevel,
};

pub use types::ScriptEngineError;
