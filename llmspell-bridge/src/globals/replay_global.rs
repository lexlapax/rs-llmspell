// ABOUTME: Global object for hook replay functionality across all script languages
// ABOUTME: Provides unified access to replay capabilities for debugging and analysis

#[cfg(any(feature = "lua", feature = "javascript"))]
use super::GlobalContext;
use super::{GlobalMetadata, GlobalObject};
#[cfg(any(feature = "lua", feature = "javascript"))]
use llmspell_core::Result;

/// Global object that provides replay functionality
pub struct ReplayGlobal {
    // Currently replay is stateless, but we keep this for future extensions
}

impl ReplayGlobal {
    #[must_use]
    pub const fn new() -> Self {
        Self {}
    }
}

impl Default for ReplayGlobal {
    fn default() -> Self {
        Self::new()
    }
}

impl GlobalObject for ReplayGlobal {
    fn metadata(&self) -> GlobalMetadata {
        GlobalMetadata {
            name: "Replay".to_string(),
            description: "Hook replay functionality for debugging and what-if analysis".to_string(),
            dependencies: vec![],
            required: false,
            version: "1.0.0".to_string(),
        }
    }

    #[cfg(feature = "lua")]
    fn inject_lua(&self, lua: &mlua::Lua, _context: &GlobalContext) -> Result<()> {
        // Use the existing inject_replay_global function
        crate::lua::globals::replay::inject_replay_global(lua)
    }

    #[cfg(feature = "javascript")]
    fn inject_javascript(
        &self,
        _ctx: &mut boa_engine::Context,
        _context: &GlobalContext,
    ) -> Result<()> {
        // JavaScript implementation placeholder
        // Full implementation will be done when JavaScript replay is needed
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_replay_global_metadata() {
        let global = ReplayGlobal::new();
        let metadata = global.metadata();
        assert_eq!(metadata.name, "Replay");
        assert_eq!(metadata.dependencies.len(), 0);
        assert!(!metadata.required);
        assert_eq!(metadata.version, "1.0.0");
    }
}
