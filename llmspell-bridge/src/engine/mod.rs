//! ABOUTME: Language abstraction layer for script engines
//! ABOUTME: Provides ScriptEngineBridge trait and factory pattern for multi-language support

pub mod bridge;
pub mod factory;
pub mod types;

pub use bridge::{
    ScriptEngineBridge, ScriptOutput, ScriptStream, ScriptMetadata,
    EngineFeatures, ExecutionContext, SecurityContext,
};

pub use factory::{
    EngineFactory, EngineInfo, LuaConfig, JSConfig, StdlibLevel, ModuleResolution,
    ScriptEnginePlugin, register_engine_plugin, unregister_engine_plugin,
};

pub use types::{
    ApiSurface, AgentApiDefinition, ToolApiDefinition, WorkflowApiDefinition,
    StreamingApiDefinition, StreamType, ScriptEngineError,
};