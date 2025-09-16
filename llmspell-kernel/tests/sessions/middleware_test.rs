//! ABOUTME: Integration tests for session middleware using pattern hooks
//! ABOUTME: Validates Sequential, Parallel, and Voting patterns in middleware execution

use anyhow::Result;
use llmspell_hooks::{
    types::{ComponentId, ComponentType},
    HookContext, HookExecutor, HookPoint, HookRegistry,
};
use llmspell_kernel::sessions::middleware::{
    create_caching_middleware, create_default_middleware, create_operation_middleware,
    create_security_middleware, create_session_middleware, session_middleware::MiddlewarePattern,
    MiddlewareConfig, MiddlewareType, SessionMiddleware,
};
use std::sync::Arc;

/// Test basic middleware creation
#[tokio::test]
async fn test_middleware_creation() -> Result<()> {
    let _hook_registry = Arc::new(HookRegistry::new());
    let _hook_executor = Arc::new(HookExecutor::new());

    // Test default middleware creation
    let default_middleware = create_default_middleware()?;
    assert_eq!(
        default_middleware.metadata().name,
        "DefaultSessionMiddleware"
    );

    // Test caching middleware creation
    let caching_middleware = create_caching_middleware()?;
    assert_eq!(caching_middleware.metadata().name, "CachingMiddleware");

    // Test security middleware creation
    let security_middleware = create_security_middleware()?;
    assert_eq!(security_middleware.metadata().name, "SecurityMiddleware");

    Ok(())
}

/// Test sequential middleware execution
#[tokio::test]
async fn test_sequential_middleware() -> Result<()> {
    let hook_registry = Arc::new(HookRegistry::new());
    let hook_executor = Arc::new(HookExecutor::new());

    let config = MiddlewareConfig {
        enable_logging: true,
        enable_metrics: true,
        enable_security: true,
        enable_caching: false,
        enable_rate_limiting: false,
        custom_hooks: vec![],
        pattern: MiddlewarePattern::Sequential,
    };

    let middleware =
        create_session_middleware(config, hook_registry.clone(), hook_executor.clone())?;

    // Create test context
    let mut context = HookContext::new(
        HookPoint::SessionStart,
        ComponentId::new(ComponentType::Agent, "test-session".to_string()),
    );

    // Execute middleware
    let result = middleware
        .execute_middleware(MiddlewareType::SessionCreate, &mut context)
        .await?;

    assert!(result.should_continue());

    // Verify hooks were registered
    let hooks = hook_registry.get_hooks(&HookPoint::SessionStart);
    assert!(!hooks.is_empty());

    Ok(())
}

/// Test parallel middleware execution
#[tokio::test]
async fn test_parallel_middleware() -> Result<()> {
    let hook_registry = Arc::new(HookRegistry::new());
    let hook_executor = Arc::new(HookExecutor::new());

    let config = MiddlewareConfig {
        enable_logging: true,
        enable_metrics: true,
        enable_security: false,
        enable_caching: true,
        enable_rate_limiting: false,
        custom_hooks: vec![],
        pattern: MiddlewarePattern::Parallel,
    };

    let middleware =
        create_session_middleware(config, hook_registry.clone(), hook_executor.clone())?;

    // Create test context for read operation
    let mut context = HookContext::new(
        HookPoint::SessionCheckpoint,
        ComponentId::new(ComponentType::Agent, "test-session".to_string()),
    );

    // Add some data for caching
    context.data.insert(
        "cache_key".to_string(),
        serde_json::json!("test_session_123"),
    );

    // Execute middleware
    let result = middleware
        .execute_middleware(MiddlewareType::SessionRead, &mut context)
        .await?;

    assert!(result.should_continue());

    Ok(())
}

/// Test voting middleware pattern
#[tokio::test]
async fn test_voting_middleware() -> Result<()> {
    let hook_registry = Arc::new(HookRegistry::new());
    let hook_executor = Arc::new(HookExecutor::new());

    let config = MiddlewareConfig {
        enable_logging: true,
        enable_metrics: true,
        enable_security: true,
        enable_caching: false,
        enable_rate_limiting: false,
        custom_hooks: vec![],
        pattern: MiddlewarePattern::Voting { threshold: 0.6 },
    };

    let middleware =
        create_session_middleware(config, hook_registry.clone(), hook_executor.clone())?;

    // Create test context
    let mut context = HookContext::new(
        HookPoint::SessionEnd,
        ComponentId::new(ComponentType::Agent, "test-session".to_string()),
    );

    // Execute middleware
    let result = middleware
        .execute_middleware(MiddlewareType::SessionDelete, &mut context)
        .await?;

    assert!(result.should_continue());

    Ok(())
}

/// Test operation-specific middleware
#[tokio::test]
async fn test_operation_specific_middleware() -> Result<()> {
    let hook_registry = Arc::new(HookRegistry::new());
    let hook_executor = Arc::new(HookExecutor::new());

    // Create read operation middleware (should include caching)
    let read_middleware = create_operation_middleware(
        MiddlewareType::SessionRead,
        hook_registry.clone(),
        hook_executor.clone(),
    )?;

    assert_eq!(read_middleware.metadata().name, "SessionReadMiddleware");

    // Create write operation middleware (should be sequential)
    let write_middleware = create_operation_middleware(
        MiddlewareType::SessionCreate,
        hook_registry.clone(),
        hook_executor.clone(),
    )?;

    assert_eq!(write_middleware.metadata().name, "SessionCreateMiddleware");

    Ok(())
}

/// Test middleware configuration update
#[tokio::test]
async fn test_middleware_config_update() -> Result<()> {
    let hook_registry = Arc::new(HookRegistry::new());
    let hook_executor = Arc::new(HookExecutor::new());

    let initial_config = MiddlewareConfig {
        enable_logging: true,
        enable_metrics: false,
        enable_security: false,
        enable_caching: false,
        enable_rate_limiting: false,
        custom_hooks: vec![],
        pattern: MiddlewarePattern::Sequential,
    };

    let mut middleware =
        SessionMiddleware::new(initial_config, hook_registry.clone(), hook_executor.clone());
    middleware.initialize()?;

    // Update configuration
    let new_config = MiddlewareConfig {
        enable_logging: true,
        enable_metrics: true,
        enable_security: true,
        enable_caching: true,
        enable_rate_limiting: true,
        custom_hooks: vec![],
        pattern: MiddlewarePattern::Parallel,
    };

    middleware.update_config(new_config)?;

    // Verify configuration was updated
    assert!(middleware.config().enable_metrics);
    assert!(middleware.config().enable_security);
    assert_eq!(middleware.config().pattern, MiddlewarePattern::Parallel);

    Ok(())
}

/// Test middleware with rate limiting
#[tokio::test]
async fn test_middleware_rate_limiting() -> Result<()> {
    let hook_registry = Arc::new(HookRegistry::new());
    let hook_executor = Arc::new(HookExecutor::new());

    let config = MiddlewareConfig {
        enable_logging: false,
        enable_metrics: false,
        enable_security: false,
        enable_caching: false,
        enable_rate_limiting: true,
        custom_hooks: vec![],
        pattern: MiddlewarePattern::Sequential,
    };

    let middleware =
        create_session_middleware(config, hook_registry.clone(), hook_executor.clone())?;

    // Create test context
    let mut context = HookContext::new(
        HookPoint::SessionStart,
        ComponentId::new(ComponentType::Agent, "test-session".to_string()),
    );

    // Execute within rate limit
    let result = middleware
        .execute_middleware(MiddlewareType::SessionCreate, &mut context)
        .await?;

    assert!(result.should_continue());

    // Rapid executions might trigger rate limit (depending on configuration)
    for _ in 0..5 {
        let _ = middleware
            .execute_middleware(MiddlewareType::SessionCreate, &mut context)
            .await;
    }

    Ok(())
}

/// Test middleware registration and execution
#[tokio::test]
async fn test_middleware_registration() -> Result<()> {
    let hook_registry = Arc::new(HookRegistry::new());
    let hook_executor = Arc::new(HookExecutor::new());

    // Create middleware
    let config = MiddlewareConfig {
        enable_logging: true,
        enable_metrics: true,
        enable_security: false,
        enable_caching: false,
        enable_rate_limiting: false,
        custom_hooks: vec![],
        pattern: MiddlewarePattern::Sequential,
    };

    let middleware =
        create_session_middleware(config, hook_registry.clone(), hook_executor.clone())?;

    // Create test context
    let mut context = HookContext::new(
        HookPoint::SessionStart,
        ComponentId::new(ComponentType::Agent, "test-session".to_string()),
    );

    // Verify hooks were registered for SessionStart
    let hooks = hook_registry.get_hooks(&HookPoint::SessionStart);
    assert!(!hooks.is_empty(), "No hooks registered for SessionStart");

    // Execute middleware
    let result = middleware
        .execute_middleware(MiddlewareType::SessionCreate, &mut context)
        .await?;

    assert!(result.should_continue());

    Ok(())
}

/// Test error propagation in middleware
#[tokio::test]
async fn test_middleware_error_handling() -> Result<()> {
    let hook_registry = Arc::new(HookRegistry::new());
    let hook_executor = Arc::new(HookExecutor::new());

    // Create a custom hook that fails
    use async_trait::async_trait;
    use llmspell_hooks::traits::Hook;

    struct FailingHook;

    #[async_trait]
    impl Hook for FailingHook {
        async fn execute(&self, _context: &mut HookContext) -> Result<llmspell_hooks::HookResult> {
            Err(anyhow::anyhow!("Simulated hook failure"))
        }

        fn metadata(&self) -> llmspell_hooks::types::HookMetadata {
            llmspell_hooks::types::HookMetadata {
                name: "FailingHook".to_string(),
                ..Default::default()
            }
        }

        fn should_execute(&self, _context: &HookContext) -> bool {
            true
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    // Register the failing hook
    hook_registry
        .register_arc(HookPoint::SessionStart, Arc::new(FailingHook))
        .map_err(|e| anyhow::anyhow!("Failed to register hook: {:?}", e))?;

    let config = MiddlewareConfig::default();
    let middleware =
        create_session_middleware(config, hook_registry.clone(), hook_executor.clone())?;

    let mut context = HookContext::new(
        HookPoint::SessionStart,
        ComponentId::new(ComponentType::Agent, "test-session".to_string()),
    );

    // Execute middleware - should propagate the error
    let result = middleware
        .execute_middleware(MiddlewareType::SessionCreate, &mut context)
        .await;

    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Simulated hook failure"));

    Ok(())
}
