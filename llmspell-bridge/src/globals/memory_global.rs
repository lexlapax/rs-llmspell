//! ABOUTME: Memory global object providing memory management for scripts
//! ABOUTME: Integrates with `MemoryManager` via `MemoryBridge` for language-specific bindings

#[cfg(any(feature = "lua", feature = "javascript"))]
use crate::globals::types::GlobalContext;
use crate::globals::types::{GlobalMetadata, GlobalObject};
use crate::memory_bridge::MemoryBridge;
#[cfg(any(feature = "lua", feature = "javascript"))]
use llmspell_core::error::LLMSpellError;
use std::sync::Arc;

/// Memory global object providing memory management for scripts
///
/// This wraps `MemoryBridge` and provides language-specific bindings,
/// converting between async Rust operations and synchronous script calls.
pub struct MemoryGlobal {
    /// Memory bridge for core operations
    pub memory_bridge: Arc<MemoryBridge>,
}

impl MemoryGlobal {
    /// Create a new Memory global
    #[must_use]
    pub const fn new(memory_bridge: Arc<MemoryBridge>) -> Self {
        Self { memory_bridge }
    }
}

impl GlobalObject for MemoryGlobal {
    fn metadata(&self) -> GlobalMetadata {
        GlobalMetadata {
            name: "Memory".to_string(),
            version: "1.0.0".to_string(),
            description: "Adaptive memory system with episodic, semantic, and procedural storage"
                .to_string(),
            dependencies: vec![], // Self-contained
            required: false,
        }
    }

    #[cfg(feature = "lua")]
    fn inject_lua(&self, lua: &mlua::Lua, context: &GlobalContext) -> Result<(), LLMSpellError> {
        crate::lua::globals::memory::inject_memory_global(
            lua,
            context,
            self.memory_bridge.clone(),
        )
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to inject Memory global: {e}"),
            source: None,
        })
    }

    #[cfg(feature = "javascript")]
    fn inject_javascript(
        &self,
        _ctx: &mut boa_engine::Context,
        _context: &GlobalContext,
    ) -> Result<(), LLMSpellError> {
        // TODO: Implement JavaScript bindings for Memory global
        Ok(())
    }
}
