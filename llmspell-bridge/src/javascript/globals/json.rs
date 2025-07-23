//! ABOUTME: JavaScript JSON global implementation - native JSON support
//! ABOUTME: JavaScript has built-in JSON object, this ensures globals pattern consistency

use crate::globals::GlobalContext;
use llmspell_core::error::LLMSpellError;

/// Inject JSON global into JavaScript engine
///
/// Note: JavaScript already has native JSON.parse() and JSON.stringify()
/// This function ensures the global exists and matches our definition for consistency.
#[cfg(feature = "javascript")]
pub fn inject_json_global(
    _ctx: &mut boa_engine::Context,
    _context: &GlobalContext,
) -> Result<(), LLMSpellError> {
    // TODO (Phase 12): When implementing JavaScript engine:
    // 1. Verify native JSON object is accessible in the engine context
    // 2. Optionally wrap native JSON to ensure consistent error handling
    // 3. Add any LLMSpell-specific extensions if needed
    // 4. Ensure behavior matches Lua implementation for cross-engine compatibility
    Ok(())
}

/// Stub for when JavaScript feature is not enabled
#[cfg(not(feature = "javascript"))]
pub fn inject_json_global(_ctx: &(), _context: &GlobalContext) -> Result<(), LLMSpellError> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::globals::GlobalContext;

    #[test]
    fn test_json_global_metadata() {
        let json_global = JsonGlobal {};
        let metadata = json_global.metadata();
        assert_eq!(metadata.name, "JSON");
        assert!(!metadata.required);
    }

    #[test]
    fn test_json_global_stub() {
        // This test verifies the stub compiles and has correct signature
        let json_global = JsonGlobal {};
        let context = GlobalContext::new();

        // Test Lua injection (should be no-op)
        let lua = mlua::Lua::new();
        let result = json_global.inject_lua(&lua, &context);
        assert!(result.is_ok());
    }

    // TODO (Phase 12): Add comprehensive tests when JS engine is implemented:
    // - Test native JSON.parse() behavior matches Lua implementation
    // - Test native JSON.stringify() behavior matches Lua implementation
    // - Test error cases and edge conditions
    // - Test performance characteristics
    // - Test cross-engine compatibility scenarios
}
