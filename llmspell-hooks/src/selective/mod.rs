// ABOUTME: Selective hook registry for library mode support (Phase 18 prep)
// ABOUTME: Provides lazy loading, feature flags, and minimal memory footprint for embedded use

//! # Selective Hook Registry
//!
//! This module provides a selective hook registry implementation designed for
//! library mode usage where minimal memory footprint and lazy loading are critical.
//! It's designed as preparation for Phase 18 library mode support.
//!
//! ## Features
//!
//! - **Feature Flag Support**: Enable/disable hooks based on compile-time or runtime flags
//! - **Lazy Loading**: Hooks are only loaded when first accessed
//! - **Minimal Memory**: Optimized memory usage with deferred instantiation
//! - **Dynamic Control**: Enable/disable hooks at runtime
//! - **Registry Filtering**: Filter hooks by various criteria
//! - **Performance Optimized**: Zero-cost abstractions when hooks are disabled
//!
//! ## Example
//!
//! ```rust,no_run
//! use llmspell_hooks::selective::{SelectiveHookRegistry, HookFeatures};
//! use llmspell_hooks::{HookPoint, Hook, HookContext, HookResult};
//! use async_trait::async_trait;
//! use anyhow::Result;
//!
//! // Define a hook
//! struct MyHook;
//!
//! #[async_trait]
//! impl Hook for MyHook {
//!     async fn execute(&self, context: &mut HookContext) -> Result<HookResult> {
//!         Ok(HookResult::Continue)
//!     }
//! }
//!
//! // Create selective registry with features
//! let mut features = HookFeatures::default();
//! features.enable_feature("logging");
//! features.enable_feature("metrics");
//!
//! let registry = SelectiveHookRegistry::new(features);
//!
//! // Register hook with feature flag
//! registry.register_with_features(
//!     HookPoint::BeforeAgentExecution,
//!     || Box::new(MyHook),
//!     &["logging"]
//! );
//! ```

mod registry;

pub use registry::{
    HookFactory, HookFeatures, LazyHookEntry, SelectiveHookRegistry, SelectiveRegistryConfig,
    SelectiveRegistryStats,
};

// Version information for selective registry
pub const SELECTIVE_REGISTRY_VERSION: &str = "0.1.0";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports() {
        // Verify exports are accessible
        let _features = HookFeatures::default();
        let _config = SelectiveRegistryConfig::default();
        assert_eq!(SELECTIVE_REGISTRY_VERSION, "0.1.0");
    }
}
