// ABOUTME: Main entry point for the llmspell-hooks crate providing hook and event system
// ABOUTME: Exports all core types, traits, and implementations for cross-language hook support

//! # LLMSpell Hooks
//!
//! A comprehensive hook and event system for rs-llmspell with cross-language support,
//! automatic performance protection, and production-ready patterns.
//!
//! ## Features
//!
//! - **Cross-Language Support**: Hooks work across Lua, JavaScript, and Python
//! - **Performance Protection**: Built-in CircuitBreaker prevents slow hooks
//! - **Production Patterns**: Caching, rate limiting, retry, and cost tracking
//! - **Event System**: High-performance event bus with backpressure handling
//! - **Future Ready**: Prepared for persistence, distribution, and library mode
//!
//! ## Example
//!
//! ```rust,no_run
//! use llmspell_hooks::{Hook, HookContext, HookResult, HookPoint, ComponentId, ComponentType};
//! use async_trait::async_trait;
//! use anyhow::Result;
//!
//! struct MyHook;
//!
//! #[async_trait]
//! impl Hook for MyHook {
//!     async fn execute(&self, context: &mut HookContext) -> Result<HookResult> {
//!         println!("Hook executed at {:?}", context.point);
//!         Ok(HookResult::Continue)
//!     }
//! }
//! ```

// Re-export core types
pub mod circuit_breaker;
pub mod context;
pub mod executor;
pub mod performance;
pub mod priority;
pub mod registry;
pub mod result;
pub mod traits;
pub mod types;

// Re-export commonly used items at crate root
pub use circuit_breaker::{BreakerState, CircuitBreaker};
pub use context::{HookContext, HookContextBuilder, OperationContext};
pub use executor::{HookExecutor, HookExecutorBuilder};
pub use performance::{PerformanceMetrics, PerformanceMonitor};
pub use priority::{PriorityBucket, PriorityComparator};
pub use registry::{HookRegistry, RegistryError};
pub use result::{ForkBuilder, HookResult, Operation, RetryBuilder};
pub use traits::{
    ArcHook, BoxedHook, ComposableHook, FnHook, Hook, HookAdapter, HookExt, MetricHook,
    ReplayableHook,
};
pub use types::{ComponentId, ComponentType, HookMetadata, HookPoint, Language, Priority};

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::{
        BreakerState, CircuitBreaker, ComponentId, ComponentType, FnHook, Hook, HookAdapter,
        HookContext, HookExecutor, HookExt, HookPoint, HookRegistry, HookResult, Language,
        Priority, ReplayableHook,
    };
    pub use anyhow::Result;
    pub use async_trait::async_trait;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_crate_exports() {
        // Verify all major types are accessible
        let _point = HookPoint::BeforeAgentInit;
        let _lang = Language::Lua;
        let _priority = Priority::HIGH;
        let _result = HookResult::Continue;

        // Verify we can create core types
        let component_id = ComponentId::new(ComponentType::System, "test".to_string());
        let _context = HookContext::new(HookPoint::SystemStartup, component_id);
    }

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }
}
