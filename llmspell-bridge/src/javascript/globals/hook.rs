//! ABOUTME: JavaScript hook global stub for Phase 15 implementation
//! ABOUTME: Placeholder implementation for future JavaScript hook support

use crate::globals::types::GlobalContext;
use llmspell_core::Result;

/// Inject the Hook global into JavaScript (stub for Phase 15)
#[cfg(feature = "javascript")]
pub fn inject_hook_global(_ctx: &mut boa_engine::Context, _context: &GlobalContext) -> Result<()> {
    // TODO (Phase 15): Implement JavaScript hook global
    // This will include:
    // - Hook.register(hook_point, callback, priority)
    // - Hook.unregister(handle)
    // - Hook.list(hook_point)
    Ok(())
}

#[cfg(not(feature = "javascript"))]
pub fn inject_hook_global(_ctx: &mut (), _context: &GlobalContext) -> Result<()> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_hook_global_stub() {
        // Basic compilation test
        let context = GlobalContext::new(
            std::sync::Arc::new(crate::ComponentRegistry::new()),
            std::sync::Arc::new(
                crate::ProviderManager::new(Default::default())
                    .await
                    .unwrap(),
            ),
        );

        #[cfg(feature = "javascript")]
        {
            let mut ctx = boa_engine::Context::default();
            assert!(inject_hook_global(&mut ctx, &context).is_ok());
        }

        #[cfg(not(feature = "javascript"))]
        {
            let mut ctx = ();
            assert!(inject_hook_global(&mut ctx, &context).is_ok());
        }
    }
}
