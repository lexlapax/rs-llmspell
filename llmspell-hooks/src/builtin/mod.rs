// ABOUTME: Core built-in hooks module providing production-ready hooks
// ABOUTME: Includes logging, metrics, debugging, security, caching, rate limiting, retry, and cost tracking

pub mod caching;
pub mod cost_tracking;
pub mod debugging;
pub mod logging;
pub mod metrics;
pub mod rate_limit;
pub mod retry;
pub mod security;

// Re-exports for easy access
pub use caching::CachingHook;
pub use cost_tracking::CostTrackingHook;
pub use debugging::DebuggingHook;
pub use logging::LoggingHook;
pub use metrics::MetricsHook;
pub use rate_limit::RateLimitHook;
pub use retry::RetryHook;
pub use security::SecurityHook;

#[cfg(test)]
#[cfg_attr(test_category = "hook")]
mod tests {
    use super::*;
    use crate::context::HookContext;
    use crate::traits::Hook;
    use crate::types::{ComponentId, ComponentType, HookPoint};

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_all_builtin_hooks_basic_execution() {
        let component_id = ComponentId::new(ComponentType::System, "test".to_string());
        let mut context = HookContext::new(HookPoint::SystemStartup, component_id);

        // Test logging hook
        let logging_hook = LoggingHook::new();
        let result = logging_hook.execute(&mut context).await.unwrap();
        assert!(result.should_continue());

        // Test metrics hook
        let metrics_hook = MetricsHook::new();
        let result = metrics_hook.execute(&mut context).await.unwrap();
        assert!(result.should_continue());

        // Test debugging hook
        let debugging_hook = DebuggingHook::new();
        let result = debugging_hook.execute(&mut context).await.unwrap();
        assert!(result.should_continue());

        // Test security hook
        let security_hook = SecurityHook::new();
        let result = security_hook.execute(&mut context).await.unwrap();
        assert!(result.should_continue());

        // Test caching hook
        let caching_hook = CachingHook::new();
        let result = caching_hook.execute(&mut context).await.unwrap();
        assert!(result.should_continue());

        // Test rate limit hook
        let rate_limit_hook = RateLimitHook::new();
        let result = rate_limit_hook.execute(&mut context).await.unwrap();
        assert!(result.should_continue());

        // Test retry hook
        let retry_hook = RetryHook::new();
        let result = retry_hook.execute(&mut context).await.unwrap();
        assert!(result.should_continue());

        // Test cost tracking hook
        let cost_tracking_hook = CostTrackingHook::new();
        let result = cost_tracking_hook.execute(&mut context).await.unwrap();
        assert!(result.should_continue());
    }
}
