//! Tests for the comprehensive tracing infrastructure

use llmspell_kernel::runtime::tracing::{SessionType, TracingInstrumentation, TracingLevel};
use test_log::test;

#[test]
fn test_tracing_session_creation() {
    let tracing = TracingInstrumentation::new_kernel_session(None, "integrated");
    assert!(!tracing.session_id().is_empty());

    let metadata = tracing.metadata();
    assert_eq!(metadata.kernel_type, "integrated");
    assert!(metadata.script_path.is_none());
    assert!(metadata.session_type.is_none());
}

#[test]
fn test_tracing_with_custom_session_id() {
    let custom_id = "test-session-123";
    let tracing =
        TracingInstrumentation::new_kernel_session(Some(custom_id.to_string()), "integrated");
    assert_eq!(tracing.session_id(), custom_id);
}

#[test]
fn test_execution_tracing() {
    let tracing = TracingInstrumentation::new_kernel_session(None, "integrated");

    tracing.start_session(SessionType::Script, Some("scripts/test.lua"));

    let metadata = tracing.metadata();
    assert_eq!(metadata.script_path, Some("scripts/test.lua".to_string()));
    assert_eq!(metadata.session_type, Some(SessionType::Script));
}

#[test]
fn test_debug_operations() {
    let tracing = TracingInstrumentation::new_kernel_session(None, "integrated");

    // Start session first
    tracing.start_session(SessionType::Debug, Some("debug_test.lua"));

    // Debug operations shouldn't panic
    tracing.trace_debug_operation("set_breakpoint", Some("line 10"));
    tracing.trace_debug_operation("step", Some("line 11"));
    tracing.trace_debug_operation("inspect_variable", Some("line 12"));
}

#[test]
fn test_performance_metrics() {
    let tracing = TracingInstrumentation::new_kernel_session(None, "integrated");

    tracing.record_metric("tool_init_time", 8.5, "ms");
    tracing.record_metric("agent_creation_time", 45.0, "ms");
    tracing.record_metric("message_handling_time", 3.2, "ms");
}

#[test]
fn test_completion_tracking() {
    let tracing = TracingInstrumentation::new_kernel_session(None, "integrated");

    tracing.start_session(SessionType::Script, Some("test-app"));

    // Complete within expected time
    tracing.complete_session(true, 8000);

    // Start another session
    tracing.start_session(SessionType::Script, Some("slow-app"));

    // Complete with much longer time (should log warning)
    tracing.complete_session(true, 15000);
}

#[test]
fn test_tracing_level_conversion() {
    assert_eq!(TracingLevel::Error.to_level(), tracing::Level::ERROR);
    assert_eq!(TracingLevel::Warn.to_level(), tracing::Level::WARN);
    assert_eq!(TracingLevel::Info.to_level(), tracing::Level::INFO);
    assert_eq!(TracingLevel::Debug.to_level(), tracing::Level::DEBUG);
    assert_eq!(TracingLevel::Trace.to_level(), tracing::Level::TRACE);
}

#[test]
fn test_tracing_level_parsing() {
    assert_eq!(TracingLevel::parse("error"), Some(TracingLevel::Error));
    assert_eq!(TracingLevel::parse("WARN"), Some(TracingLevel::Warn));
    assert_eq!(TracingLevel::parse("Info"), Some(TracingLevel::Info));
    assert_eq!(TracingLevel::parse("debug"), Some(TracingLevel::Debug));
    assert_eq!(TracingLevel::parse("TRACE"), Some(TracingLevel::Trace));
    assert_eq!(TracingLevel::parse("invalid"), None);
}

#[test]
fn test_warning_context() {
    let tracing =
        TracingInstrumentation::new_kernel_session(Some("warn-test".to_string()), "integrated");

    tracing.warn_with_context("This is a warning with session context");

    // The warning should include the session ID in the log output
}

#[test]
fn test_metadata_updates() {
    let tracing = TracingInstrumentation::new_kernel_session(None, "integrated");

    // Initial metadata
    let metadata1 = tracing.metadata();
    assert!(metadata1.script_path.is_none());
    assert!(metadata1.session_type.is_none());

    // Update via start_session
    tracing.start_session(SessionType::Script, Some("test.lua"));
    let metadata2 = tracing.metadata();
    assert_eq!(metadata2.script_path, Some("test.lua".to_string()));
    assert_eq!(metadata2.session_type, Some(SessionType::Script));

    // Update via another start_session
    tracing.start_session(SessionType::Debug, Some("test-app"));
    let metadata3 = tracing.metadata();
    assert_eq!(metadata3.script_path, Some("test-app".to_string()));
    assert_eq!(metadata3.session_type, Some(SessionType::Debug));
}

#[test]
fn test_span_entering() {
    let tracing = TracingInstrumentation::new_kernel_session(None, "integrated");

    // Enter the kernel span context
    let _guard = tracing.enter();

    // Operations within this context will be traced under the kernel span
    tracing.start_session(SessionType::Script, Some("scoped_test.lua"));
}
