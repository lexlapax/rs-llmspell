//! ABOUTME: Session middleware implementation using existing pattern hooks
//! ABOUTME: Provides middleware chains for session operations using Sequential, Parallel, and Voting patterns

use anyhow::Result;
use llmspell_hooks::{
    builtin::{CachingHook, LoggingHook, MetricsHook, SecurityHook},
    traits::Hook,
    types::{HookMetadata, Priority},
};
use std::sync::Arc;

pub mod session_middleware;

pub use session_middleware::{
    create_operation_middleware, create_session_middleware, MiddlewareConfig, MiddlewareType,
    ParallelMiddleware, SequentialMiddleware, SessionMiddleware, VotingMiddleware,
};

/// Create default middleware chain for session operations
pub fn create_default_middleware() -> Result<Arc<dyn Hook>> {
    // Create a custom hook that executes logging, metrics, and security in sequence
    let middleware = SequentialMiddleware::new("default_session_middleware")
        .add_hook(Arc::new(LoggingHook::new()))
        .add_hook(Arc::new(MetricsHook::new()))
        .add_hook(Arc::new(SecurityHook::new()))
        .with_metadata(HookMetadata {
            name: "DefaultSessionMiddleware".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Default middleware chain for session operations".to_string()),
            priority: Priority(50),
            tags: vec!["middleware".to_string(), "session".to_string()],
            language: llmspell_hooks::Language::Native,
        });

    Ok(Arc::new(middleware))
}

/// Create caching middleware for read operations
pub fn create_caching_middleware() -> Result<Arc<dyn Hook>> {
    // Parallel execution of caching and metrics
    let middleware = ParallelMiddleware::new("caching_middleware")
        .add_hook(Arc::new(CachingHook::new()))
        .add_hook(Arc::new(MetricsHook::new()))
        .with_metadata(HookMetadata {
            name: "CachingMiddleware".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Caching middleware for read operations".to_string()),
            priority: Priority(60),
            tags: vec!["middleware".to_string(), "caching".to_string()],
            language: llmspell_hooks::Language::Native,
        });

    Ok(Arc::new(middleware))
}

/// Create security middleware with voting pattern
pub fn create_security_middleware() -> Result<Arc<dyn Hook>> {
    // Multiple security checks with voting (at least 2 must pass)
    let middleware = VotingMiddleware::new("security_middleware", 0.66)
        .add_hook(Arc::new(SecurityHook::new()))
        .add_hook(Arc::new(SecurityHook::new())) // Different security policies
        .add_hook(Arc::new(SecurityHook::new())) // Additional validation
        .with_metadata(HookMetadata {
            name: "SecurityMiddleware".to_string(),
            version: "1.0.0".to_string(),
            description: Some("Multi-layer security middleware with voting".to_string()),
            priority: Priority(90),
            tags: vec!["middleware".to_string(), "security".to_string()],
            language: llmspell_hooks::Language::Native,
        });

    Ok(Arc::new(middleware))
}
