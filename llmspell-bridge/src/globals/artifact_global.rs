//! ABOUTME: Artifact global object providing artifact management for scripts
//! ABOUTME: Integrates with `SessionManager` via `ArtifactBridge` for language-specific bindings

use crate::artifact_bridge::ArtifactBridge;
use crate::globals::types::{GlobalContext, GlobalMetadata, GlobalObject};
use llmspell_core::error::LLMSpellError;
use std::sync::Arc;

/// Artifact global object providing artifact management for scripts
///
/// This wraps `ArtifactBridge` and provides language-specific bindings,
/// converting between async Rust operations and synchronous script calls.
pub struct ArtifactGlobal {
    /// Artifact bridge for core operations
    pub artifact_bridge: Arc<ArtifactBridge>,
}

impl ArtifactGlobal {
    /// Create a new Artifact global
    #[must_use]
    pub const fn new(artifact_bridge: Arc<ArtifactBridge>) -> Self {
        Self { artifact_bridge }
    }
}

impl GlobalObject for ArtifactGlobal {
    fn metadata(&self) -> GlobalMetadata {
        GlobalMetadata {
            name: "Artifact".to_string(),
            version: "1.0.0".to_string(),
            description: "Artifact storage and management system".to_string(),
            dependencies: vec!["Session".to_string()], // Artifacts belong to sessions
            required: false,
        }
    }

    #[cfg(feature = "lua")]
    fn inject_lua(&self, lua: &mlua::Lua, context: &GlobalContext) -> Result<(), LLMSpellError> {
        crate::lua::globals::artifact::inject_artifact_global(
            lua,
            context,
            self.artifact_bridge.clone(),
        )
        .map_err(|e| LLMSpellError::Component {
            message: format!("Failed to inject Artifact global: {e}"),
            source: None,
        })
    }

    #[cfg(feature = "javascript")]
    fn inject_javascript(
        &self,
        _ctx: &mut boa_engine::Context,
        _context: &GlobalContext,
    ) -> Result<(), LLMSpellError> {
        // TODO: Implement JavaScript bindings for Artifact global
        Ok(())
    }
}
