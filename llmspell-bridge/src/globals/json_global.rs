//! ABOUTME: JSON global object providing JSON parsing and stringification
//! ABOUTME: Language-agnostic JSON utilities accessible from all script engines

#[cfg(any(feature = "lua", feature = "javascript"))]
use crate::globals::types::GlobalContext;
use crate::globals::types::{GlobalMetadata, GlobalObject};
#[cfg(any(feature = "lua", feature = "javascript"))]
use llmspell_core::error::LLMSpellError;

/// JSON global object providing JSON utilities
pub struct JsonGlobal {}

impl JsonGlobal {
    /// Create a new JSON global
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }
}

impl GlobalObject for JsonGlobal {
    fn metadata(&self) -> GlobalMetadata {
        GlobalMetadata {
            name: "JSON".to_string(),
            version: "1.0.0".to_string(),
            description: "JSON parsing and stringification utilities".to_string(),
            dependencies: vec![],
            required: true,
        }
    }

    #[cfg(feature = "lua")]
    fn inject_lua(&self, lua: &mlua::Lua, _context: &GlobalContext) -> Result<(), LLMSpellError> {
        crate::lua::globals::json::inject_json_global(lua)?;
        Ok(())
    }

    #[cfg(feature = "javascript")]
    fn inject_javascript(
        &self,
        _ctx: &mut boa_engine::Context,
        _context: &GlobalContext,
    ) -> Result<(), LLMSpellError> {
        // JavaScript has native JSON support, but we can enhance it if needed
        // For now, just return Ok since JSON is built-in
        Ok(())
    }
}

impl Default for JsonGlobal {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_json_global_metadata() {
        let global = JsonGlobal::new();
        let metadata = global.metadata();
        assert_eq!(metadata.name, "JSON");
        assert!(metadata.dependencies.is_empty());
    }
}
