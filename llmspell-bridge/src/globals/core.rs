//! ABOUTME: Core global objects that are injected into script engines
//! ABOUTME: Provides fundamental globals like Agent, Tool, Workflow, Hook, Event, State, Logger, Config, Security, Utils

use super::types::{GlobalContext, GlobalMetadata, GlobalObject};
use llmspell_core::Result;

#[cfg(feature = "lua")]
use mlua::Lua;

// Re-export the actual global implementations
pub use super::agent_global::AgentGlobal;
pub use super::tool_global::ToolGlobal;
pub use super::workflow_global::WorkflowGlobal;

/// Logger global object for script engines
pub struct LoggerGlobal;

impl LoggerGlobal {
    pub fn new() -> Self {
        Self
    }
}

impl Default for LoggerGlobal {
    fn default() -> Self {
        Self::new()
    }
}

impl GlobalObject for LoggerGlobal {
    fn metadata(&self) -> GlobalMetadata {
        GlobalMetadata {
            name: "Logger".to_string(),
            description: "Logging utilities".to_string(),
            dependencies: vec![],
            required: false,
            version: "1.0.0".to_string(),
        }
    }

    #[cfg(feature = "lua")]
    fn inject_lua(&self, lua: &Lua, _context: &GlobalContext) -> Result<()> {
        // TODO: Implement Lua injection
        let table = lua
            .create_table()
            .map_err(|e| llmspell_core::LLMSpellError::Component {
                message: format!("Failed to create Logger table: {}", e),
                source: None,
            })?;
        lua.globals().set("Logger", table).map_err(|e| {
            llmspell_core::LLMSpellError::Component {
                message: format!("Failed to set Logger global: {}", e),
                source: None,
            }
        })?;
        Ok(())
    }

    #[cfg(feature = "javascript")]
    fn inject_javascript(
        &self,
        _ctx: &mut boa_engine::Context,
        _context: &GlobalContext,
    ) -> Result<()> {
        // TODO: Implement JavaScript injection
        Ok(())
    }
}

/// Config global object for script engines
pub struct ConfigGlobal {
    #[allow(dead_code)]
    config: serde_json::Value,
}

impl ConfigGlobal {
    pub fn new(config: serde_json::Value) -> Self {
        Self { config }
    }
}

impl GlobalObject for ConfigGlobal {
    fn metadata(&self) -> GlobalMetadata {
        GlobalMetadata {
            name: "Config".to_string(),
            description: "Configuration access".to_string(),
            dependencies: vec![],
            required: false,
            version: "1.0.0".to_string(),
        }
    }

    #[cfg(feature = "lua")]
    fn inject_lua(&self, lua: &Lua, _context: &GlobalContext) -> Result<()> {
        // TODO: Convert config to Lua table
        let table = lua
            .create_table()
            .map_err(|e| llmspell_core::LLMSpellError::Component {
                message: format!("Failed to create Config table: {}", e),
                source: None,
            })?;
        lua.globals().set("Config", table).map_err(|e| {
            llmspell_core::LLMSpellError::Component {
                message: format!("Failed to set Config global: {}", e),
                source: None,
            }
        })?;
        Ok(())
    }

    #[cfg(feature = "javascript")]
    fn inject_javascript(
        &self,
        _ctx: &mut boa_engine::Context,
        _context: &GlobalContext,
    ) -> Result<()> {
        // TODO: Implement JavaScript injection
        Ok(())
    }
}

/// Utils global object for script engines
pub struct UtilsGlobal;

impl UtilsGlobal {
    pub fn new() -> Self {
        Self
    }
}

impl Default for UtilsGlobal {
    fn default() -> Self {
        Self::new()
    }
}

impl GlobalObject for UtilsGlobal {
    fn metadata(&self) -> GlobalMetadata {
        GlobalMetadata {
            name: "Utils".to_string(),
            description: "Utility functions".to_string(),
            dependencies: vec![],
            required: false,
            version: "1.0.0".to_string(),
        }
    }

    #[cfg(feature = "lua")]
    fn inject_lua(&self, lua: &Lua, _context: &GlobalContext) -> Result<()> {
        // TODO: Implement Lua injection
        let table = lua
            .create_table()
            .map_err(|e| llmspell_core::LLMSpellError::Component {
                message: format!("Failed to create Utils table: {}", e),
                source: None,
            })?;
        lua.globals()
            .set("Utils", table)
            .map_err(|e| llmspell_core::LLMSpellError::Component {
                message: format!("Failed to set Utils global: {}", e),
                source: None,
            })?;
        Ok(())
    }

    #[cfg(feature = "javascript")]
    fn inject_javascript(
        &self,
        _ctx: &mut boa_engine::Context,
        _context: &GlobalContext,
    ) -> Result<()> {
        // TODO: Implement JavaScript injection
        Ok(())
    }
}
