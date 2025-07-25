//! ABOUTME: JavaScript hook adapter stub for Phase 15 implementation
//! ABOUTME: Placeholder implementation for future JavaScript hook support

use llmspell_hooks::{HookAdapter, HookContext, HookResult};
use std::any::Any;

/// JavaScript-specific hook adapter (stub for Phase 15)
pub struct JavaScriptHookAdapter;

impl JavaScriptHookAdapter {
    /// Create a new JavaScript hook adapter
    pub fn new() -> Self {
        Self
    }
}

impl Default for JavaScriptHookAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "javascript")]
impl HookAdapter for JavaScriptHookAdapter {
    type Context = Box<dyn Any>;
    type Result = Box<dyn Any>;

    fn adapt_context(&self, _ctx: &HookContext) -> Self::Context {
        // TODO (Phase 15): Implement JavaScript context adaptation
        Box::new(())
    }

    fn adapt_result(&self, _result: Self::Result) -> HookResult {
        // TODO (Phase 15): Implement JavaScript result adaptation
        HookResult::Continue
    }

    fn extract_error(&self, _result: &Self::Result) -> Option<String> {
        // TODO (Phase 15): Implement JavaScript error extraction
        None
    }
}

#[cfg(not(feature = "javascript"))]
impl HookAdapter for JavaScriptHookAdapter {
    type Context = Box<dyn Any>;
    type Result = Box<dyn Any>;

    fn adapt_context(&self, _ctx: &HookContext) -> Self::Context {
        Box::new(())
    }

    fn adapt_result(&self, _result: Self::Result) -> HookResult {
        HookResult::Continue
    }

    fn extract_error(&self, _result: &Self::Result) -> Option<String> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_javascript_hook_adapter_stub() {
        let adapter = JavaScriptHookAdapter::new();
        
        // Test that stub methods don't panic
        let ctx = Box::new(()) as Box<dyn Any>;
        let _ = adapter.adapt_context(&HookContext::default());
        let _ = adapter.adapt_result(ctx.clone());
        let _ = adapter.extract_error(&ctx);
    }
}