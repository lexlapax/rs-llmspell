//! ABOUTME: Session analytics module using existing MetricsHook infrastructure
//! ABOUTME: Provides session-specific metrics collection and aggregation

use anyhow::Result;
use llmspell_hooks::{HookPoint, HookRegistry};
use std::sync::Arc;

pub mod session_metrics;

pub use session_metrics::{
    SessionAnalytics, SessionAnalyticsConfig, SessionMetricType, SessionMetricsCollector,
    SessionMetricsSummary,
};

/// Create default session analytics
///
/// # Errors
/// Returns error if hook registration fails
pub fn create_session_analytics(
    hook_registry: &Arc<HookRegistry>,
) -> Result<Arc<SessionAnalytics>> {
    let config = SessionAnalyticsConfig::default();
    let analytics = SessionAnalytics::new(config);

    // Register the analytics hook for all session-related hook points
    let hook_points = vec![
        HookPoint::SessionStart,
        HookPoint::SessionEnd,
        HookPoint::SessionCheckpoint,
        HookPoint::SessionRestore,
        HookPoint::SessionSave,
    ];

    for hook_point in hook_points {
        hook_registry
            .register_arc(hook_point, analytics.as_hook())
            .map_err(|e| anyhow::anyhow!("Failed to register analytics hook: {:?}", e))?;
    }

    Ok(Arc::new(analytics))
}
