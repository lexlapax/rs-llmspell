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
//!     fn as_any(&self) -> &dyn std::any::Any {
//!         self
//!     }
//!     
//!     async fn execute(&self, context: &mut HookContext) -> Result<HookResult> {
//!         println!("Hook executed at {:?}", context.point);
//!         Ok(HookResult::Continue)
//!     }
//! }
//! ```
//!
//! ## Performance Characteristics
//!
//! - **Hook Execution**: <1ms overhead with CircuitBreaker protection
//! - **Event Bus Throughput**: >90K events/sec
//! - **Circuit Breaker**: Opens after 50ms latency (configurable)
//! - **Cache Hit Rate**: >95% for repeated patterns
//! - **Rate Limiting**: Token bucket with configurable rates
//!
//! ## Integration Example
//!
//! ```rust,no_run
//! use llmspell_hooks::{
//!     HookRegistry, HookPoint, CachingHook, CachingConfig,
//!     RateLimitHook, LoggingHook, MetricsHook
//! };
//! use std::time::Duration;
//!
//! # fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create registry with production-ready hooks
//! let registry = HookRegistry::new();
//!
//! // Add caching for performance
//! let mut cache_config = CachingConfig::default();
//! cache_config.default_ttl = Duration::from_secs(60);
//! cache_config.max_entries = 1000;
//! let cache = CachingHook::with_config(cache_config);
//!
//! // Add rate limiting for cost control  
//! let limiter = RateLimitHook::new()
//!     .with_rate_per_second(100.0) // 100 requests per second
//!     .with_burst(10);
//!
//! // Add logging hook
//! let logger = LoggingHook::new();
//!
//! // Add metrics collection
//! let metrics = MetricsHook::new();
//!
//! // Register hooks for specific hook points
//! registry.register(HookPoint::BeforeAgentExecution, cache)?;
//! registry.register(HookPoint::BeforeAgentExecution, limiter)?;
//! registry.register(HookPoint::BeforeAgentExecution, logger)?;
//! registry.register(HookPoint::AfterAgentExecution, metrics)?;
//! # Ok(())
//! # }
//! ```

// Re-export core types
pub mod artifact_hooks;
pub mod builtin;
pub mod cache;
pub mod circuit_breaker;
pub mod collectors;
pub mod context;
pub mod coordination;
pub mod distributed;
pub mod executor;
pub mod performance;
pub mod persistence;
pub mod priority;
pub mod rate_limiter;
pub mod registry;
pub mod replay;
pub mod result;
pub mod selective;
pub mod traits;
pub mod types;

// Re-export commonly used items at crate root
pub use artifact_hooks::{event_to_hook_point, is_artifact_hook_point, ArtifactHookPoints};
pub use circuit_breaker::{BreakerConfig, BreakerState, CircuitBreaker};
pub use collectors::{
    AgentOutputCollector, ArtifactCollector, ArtifactData, CollectionConfig, ToolResultCollector,
};
pub use context::{HookContext, HookContextBuilder, OperationContext};
pub use coordination::{
    CorrelationId, CrossComponentContext, CrossComponentCoordinator, DependencyGraph,
    DependencyNode, EventCorrelator, EventTrace, ExecutionChain, ExecutionOrder,
};
pub use distributed::{
    DistributedHookContext, DistributedHookContextBuilder, PropagationFlags, RemoteAgentId,
    SecurityContext,
};
pub use executor::{HookExecutor, HookExecutorBuilder};
pub use performance::{PerformanceMetrics, PerformanceMonitor};
pub use persistence::{
    HookMetadata as PersistenceHookMetadata, HookPersistenceManager, RetentionManager,
    RetentionPolicy,
};
pub use priority::{PriorityBucket, PriorityComparator};
pub use registry::{HookRegistry, RegistryError};
pub use result::{ForkBuilder, HookResult, Operation, RetryBuilder};
pub use selective::{HookFeatures, SelectiveHookRegistry, SelectiveRegistryConfig};
pub use traits::{
    ArcHook, BoxedHook, ComposableHook, FnHook, Hook, HookAdapter, HookExt, MetricHook,
    ReplayableHook,
};
pub use types::{ComponentId, ComponentType, HookMetadata, HookPoint, Language, Priority};

// Re-export built-in hooks for easy access
pub use builtin::{
    caching::CachingConfig, CachingHook, CostTrackingHook, DebuggingHook, LoggingHook, MetricsHook,
    RateLimitHook, RetryHook, SecurityHook,
};

// Re-export cache types for easy access
pub use cache::{Cache, CacheKey, CacheStats};

// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::{
        BreakerState, Cache, CacheKey, CachingHook, CircuitBreaker, ComponentId, ComponentType,
        CorrelationId, CostTrackingHook, CrossComponentCoordinator, DebuggingHook, DependencyGraph,
        EventCorrelator, ExecutionChain, FnHook, Hook, HookAdapter, HookContext, HookExecutor,
        HookExt, HookPoint, HookRegistry, HookResult, Language, LoggingHook, MetricsHook, Priority,
        RateLimitHook, ReplayableHook, RetryHook, SecurityHook,
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
    #[allow(clippy::len_zero)]
    fn test_version() {
        assert!(VERSION.len() > 0);
    }
}
