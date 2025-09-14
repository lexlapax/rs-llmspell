//! Tests for the comprehensive tracing infrastructure

use llmspell_kernel::runtime::tracing::{
    detect_application_type, init_tracing_with_filter, ApplicationDetection,
    TracingInstrumentation, TracingLevel,
};
use test_log::test;

#[test]
fn test_tracing_session_creation() {
    let tracing = TracingInstrumentation::new_kernel_session(None);
    assert!(!tracing.session_id().is_empty());

    let metadata = tracing.metadata();
    assert_eq!(metadata.kernel_type, "integrated");
    assert!(metadata.script_path.is_none());
    assert!(metadata.agent_count.is_none());
}

#[test]
fn test_tracing_with_custom_session_id() {
    let custom_id = "test-session-123";
    let tracing = TracingInstrumentation::new_kernel_session(Some(custom_id.to_string()));
    assert_eq!(tracing.session_id(), custom_id);
}

#[test]
fn test_execution_tracing() {
    let tracing = TracingInstrumentation::new_kernel_session(None);

    tracing.start_execution("scripts/test.lua", 5);

    let metadata = tracing.metadata();
    assert_eq!(metadata.script_path, Some("scripts/test.lua".to_string()));
    assert_eq!(metadata.agent_count, Some(5));
}

#[test]
fn test_application_tracing() {
    let tracing = TracingInstrumentation::new_kernel_session(None);

    tracing.start_application("content-creator", 2, 4, 30);

    let metadata = tracing.metadata();
    assert_eq!(
        metadata.application_type,
        Some("content-creator".to_string())
    );
    assert_eq!(metadata.complexity_layer, Some(2));
    assert_eq!(metadata.expected_runtime_seconds, Some(30));
}

#[test]
fn test_debug_operations() {
    let tracing = TracingInstrumentation::new_kernel_session(None);

    // Start execution first
    tracing.start_execution("debug_test.lua", 1);

    // Debug operations shouldn't panic
    tracing.debug_operation("set_breakpoint", 10);
    tracing.debug_operation("step", 11);
    tracing.debug_operation("inspect_variable", 12);
}

#[test]
fn test_performance_metrics() {
    let tracing = TracingInstrumentation::new_kernel_session(None);

    tracing.record_metric("tool_init_time", 8.5, "ms");
    tracing.record_metric("agent_creation_time", 45.0, "ms");
    tracing.record_metric("message_handling_time", 3.2, "ms");
}

#[test]
fn test_completion_tracking() {
    let tracing = TracingInstrumentation::new_kernel_session(None);

    tracing.start_application("test-app", 1, 2, 10);

    // Complete within expected time
    tracing.complete_execution(true, 8000);

    // Start another execution
    tracing.start_application("slow-app", 1, 2, 5);

    // Complete with much longer time (should log warning)
    tracing.complete_execution(true, 15000);
}

#[test]
fn test_tracing_level_conversion() {
    assert_eq!(
        TracingLevel::Error.to_level(),
        tracing::Level::ERROR
    );
    assert_eq!(
        TracingLevel::Warn.to_level(),
        tracing::Level::WARN
    );
    assert_eq!(
        TracingLevel::Info.to_level(),
        tracing::Level::INFO
    );
    assert_eq!(
        TracingLevel::Debug.to_level(),
        tracing::Level::DEBUG
    );
    assert_eq!(
        TracingLevel::Trace.to_level(),
        tracing::Level::TRACE
    );
}

#[test]
fn test_tracing_level_parsing() {
    assert_eq!(TracingLevel::from_str("error"), Some(TracingLevel::Error));
    assert_eq!(TracingLevel::from_str("WARN"), Some(TracingLevel::Warn));
    assert_eq!(TracingLevel::from_str("Info"), Some(TracingLevel::Info));
    assert_eq!(TracingLevel::from_str("debug"), Some(TracingLevel::Debug));
    assert_eq!(TracingLevel::from_str("TRACE"), Some(TracingLevel::Trace));
    assert_eq!(TracingLevel::from_str("invalid"), None);
}

#[test]
fn test_application_detection() {
    // Test known applications
    let tests = vec![
        (
            "examples/file-organizer/main.lua",
            ApplicationDetection {
                app_type: "file-organizer".to_string(),
                agent_count: 3,
                estimated_seconds: 10,
                complexity_layer: 1,
            },
        ),
        (
            "examples/content-creator/main.lua",
            ApplicationDetection {
                app_type: "content-creator".to_string(),
                agent_count: 4,
                estimated_seconds: 30,
                complexity_layer: 2,
            },
        ),
        (
            "examples/code-review-assistant/main.lua",
            ApplicationDetection {
                app_type: "code-review-assistant".to_string(),
                agent_count: 7,
                estimated_seconds: 60,
                complexity_layer: 3,
            },
        ),
        (
            "examples/webapp-creator/main.lua",
            ApplicationDetection {
                app_type: "webapp-creator".to_string(),
                agent_count: 21,
                estimated_seconds: 180,
                complexity_layer: 5,
            },
        ),
        (
            "random/unknown/script.lua",
            ApplicationDetection {
                app_type: "unknown".to_string(),
                agent_count: 1,
                estimated_seconds: 30,
                complexity_layer: 1,
            },
        ),
    ];

    for (path, expected) in tests {
        let detection = detect_application_type(path);
        assert_eq!(detection.app_type, expected.app_type);
        assert_eq!(detection.agent_count, expected.agent_count);
        assert_eq!(detection.estimated_seconds, expected.estimated_seconds);
        assert_eq!(detection.complexity_layer, expected.complexity_layer);
    }
}

#[test]
fn test_warning_context() {
    let tracing = TracingInstrumentation::new_kernel_session(Some("warn-test".to_string()));

    tracing.warn_with_context("This is a warning with session context");

    // The warning should include the session ID in the log output
}

#[test]
fn test_metadata_updates() {
    let tracing = TracingInstrumentation::new_kernel_session(None);

    // Initial metadata
    let metadata1 = tracing.metadata();
    assert!(metadata1.script_path.is_none());
    assert!(metadata1.agent_count.is_none());

    // Update via start_execution
    tracing.start_execution("test.lua", 3);
    let metadata2 = tracing.metadata();
    assert_eq!(metadata2.script_path, Some("test.lua".to_string()));
    assert_eq!(metadata2.agent_count, Some(3));

    // Update via start_application
    tracing.start_application("test-app", 2, 5, 60);
    let metadata3 = tracing.metadata();
    assert_eq!(metadata3.application_type, Some("test-app".to_string()));
    assert_eq!(metadata3.complexity_layer, Some(2));
    assert_eq!(metadata3.expected_runtime_seconds, Some(60));
}

#[test]
fn test_span_entering() {
    let tracing = TracingInstrumentation::new_kernel_session(None);

    // Enter the kernel span context
    let _guard = tracing.enter();

    // Operations within this context will be traced under the kernel span
    tracing.start_execution("scoped_test.lua", 2);
}