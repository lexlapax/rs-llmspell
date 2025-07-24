// ABOUTME: Core built-in hooks module providing production-ready hooks
// ABOUTME: Includes logging, metrics, debugging, security, caching, rate limiting, retry, and cost tracking

pub mod debugging;
pub mod logging;
pub mod metrics;
pub mod security;

// Re-exports for easy access
pub use debugging::DebuggingHook;
pub use logging::LoggingHook;
pub use metrics::MetricsHook;
pub use security::SecurityHook;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::HookContext;
    use crate::traits::Hook;
    use crate::types::{ComponentId, ComponentType, HookPoint};

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
    }
}
