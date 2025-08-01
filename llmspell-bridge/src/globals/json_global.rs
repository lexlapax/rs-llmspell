//! ABOUTME: JSON global object providing JSON parsing and stringification
//! ABOUTME: Language-agnostic JSON utilities accessible from all script engines

use crate::globals::types::{GlobalContext, GlobalMetadata, GlobalObject};
use llmspell_core::error::LLMSpellError;

/// JSON global object providing JSON utilities
pub struct JsonGlobal {}

impl JsonGlobal {
    /// Create a new JSON global
    pub fn new() -> Self {
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
#[cfg_attr(test_category = "bridge")]
mod tests {
    use super::*;

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_json_global_metadata() {
        let global = JsonGlobal::new();
        let metadata = global.metadata();
        assert_eq!(metadata.name, "JSON");
        assert!(metadata.dependencies.is_empty());
    }
}
