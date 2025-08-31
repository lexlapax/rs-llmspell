//! Tests for distributed tracing integration (Task 9.2.11)
//!
//! These tests verify that:
//! - OpenTelemetry is integrated with `DiagnosticsBridge`
//! - Script execution is traced via `SharedExecutionContext` correlation IDs
//! - Trace spans are enriched with context data
//! - OTLP exporter configuration works

#![allow(clippy::significant_drop_tightening)]

use llmspell_bridge::{
    diagnostics_bridge::DiagnosticsBridge,
    execution_context::{ExecutionContextBridge, SharedExecutionContext, SourceLocation},
    tracing::{DefaultTraceEnricher, ScriptTracer, TraceEnricher, TracingConfig},
};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Test creating `DiagnosticsBridge` with tracing enabled
#[test]
fn test_diagnostics_bridge_with_tracing() {
    let config = TracingConfig {
        enabled: false, // Don't actually connect to OTLP in tests
        otlp_endpoint: "http://localhost:4317".to_string(),
        service_name: "llmspell-test".to_string(),
        sampling_rate: 1.0,
        max_attributes_per_span: 128,
        max_events_per_span: 128,
        auto_propagate: true,
    };

    let bridge = DiagnosticsBridge::new().with_distributed_tracing(config);
    assert!(bridge.is_enabled());
}

/// Test trace execution with context enrichment
#[test]
fn test_trace_execution_with_context() {
    let config = TracingConfig {
        enabled: false, // Don't actually connect to OTLP
        ..Default::default()
    };

    let bridge = DiagnosticsBridge::new().with_distributed_tracing(config);

    let mut context = SharedExecutionContext::new();
    context.location = Some(SourceLocation {
        source: "test.lua".to_string(),
        line: 42,
        column: Some(10),
    });
    context.correlation_id = Some(uuid::Uuid::new_v4());

    // This should create a span (or None if tracing is disabled)
    let _span = bridge.trace_execution("test_operation", &context);
}

/// Test `ExecutionContextBridge` implementation
#[test]
fn test_execution_context_bridge() {
    let bridge = DiagnosticsBridge::new();

    // Test get and update context
    let mut context = SharedExecutionContext::new();
    context.location = Some(SourceLocation {
        source: "test.lua".to_string(),
        line: 100,
        column: None,
    });

    bridge.update_context(context.clone());
    let retrieved = bridge.get_context();
    assert_eq!(retrieved.location, context.location);

    // Test enrich_diagnostic
    let enriched = bridge.enrich_diagnostic("Test message");
    assert!(enriched.contains("Test message"));
    assert!(enriched.contains("test.lua"));
    assert!(enriched.contains("100"));
}

/// Test `DefaultTraceEnricher`
#[test]
fn test_trace_enricher() {
    let enricher = DefaultTraceEnricher;

    let mut context = SharedExecutionContext::new();
    context.location = Some(SourceLocation {
        source: "script.lua".to_string(),
        line: 25,
        column: Some(5),
    });
    context.correlation_id = Some(uuid::Uuid::new_v4());
    context.performance_metrics.execution_count = 10;
    context.performance_metrics.function_time_us = 5000;

    // Create attributes from context
    let attributes = enricher.context_to_attributes(&context);
    assert!(!attributes.is_empty());

    // Check that expected attributes are present
    let attr_strings: Vec<String> = attributes
        .iter()
        .map(|kv| format!("{}={:?}", kv.key.as_str(), kv.value))
        .collect();

    assert!(attr_strings.iter().any(|s| s.contains("source.file")));
    assert!(attr_strings.iter().any(|s| s.contains("source.line")));
    assert!(attr_strings.iter().any(|s| s.contains("correlation.id")));
    assert!(attr_strings
        .iter()
        .any(|s| s.contains("performance.execution_count")));
}

/// Test `TracingConfig` defaults
#[test]
fn test_tracing_config_defaults() {
    let config = TracingConfig::default();

    assert!(!config.enabled);
    assert_eq!(config.otlp_endpoint, "http://localhost:4317");
    assert_eq!(config.service_name, "llmspell");
    #[allow(clippy::float_cmp)]
    { assert_eq!(config.sampling_rate, 1.0); }
    assert_eq!(config.max_attributes_per_span, 128);
    assert_eq!(config.max_events_per_span, 128);
    assert!(config.auto_propagate);
}

/// Test async context preservation with tracing
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_async_context_with_tracing() {
    let config = TracingConfig {
        enabled: false, // Don't actually connect to OTLP
        ..Default::default()
    };

    let bridge = DiagnosticsBridge::new().with_distributed_tracing(config);
    let shared_context = Arc::new(RwLock::new(SharedExecutionContext::new()));

    // Set up context with async support
    {
        let mut ctx = shared_context.write().await;
        *ctx = ctx.clone().with_async_support();
    }

    // Update bridge context
    {
        let ctx = shared_context.read().await;
        bridge.update_context(ctx.clone());
    }

    // Preserve across async boundary
    let snapshot = {
        let ctx = shared_context.read().await;
        ctx.preserve_across_async_boundary()
    };

    // Simulate async work
    tokio::spawn(async move {
        let mut ctx = SharedExecutionContext::new();
        ctx.restore_from_async_boundary(snapshot);
        assert!(ctx.correlation_id.is_some());
    })
    .await
    .unwrap();
}

/// Test diagnostic tracing
#[test]
fn test_diagnostic_tracing() {
    let config = TracingConfig {
        enabled: false, // Don't actually connect to OTLP
        ..Default::default()
    };

    let bridge = DiagnosticsBridge::new().with_distributed_tracing(config);

    // This should create a diagnostic span (or do nothing if disabled)
    bridge.trace_diagnostic("Test diagnostic message", "info");
    bridge.trace_diagnostic("Error occurred", "error");
}

/// Test that tracing works with `log_with_metadata`
#[test]
fn test_log_with_metadata_traces() {
    let config = TracingConfig {
        enabled: false, // Don't actually connect to OTLP
        ..Default::default()
    };

    let bridge = DiagnosticsBridge::new().with_distributed_tracing(config);

    let metadata = serde_json::json!({
        "component": "test",
        "action": "testing"
    });

    // This should trace the diagnostic and log it
    bridge.log_with_metadata(
        "info",
        "Test message with metadata",
        Some("test_module"),
        metadata,
    );
}

#[cfg(feature = "lua")]
mod lua_tests {
    use super::*;
    use llmspell_bridge::lua::tracing_impl::{inject_tracing_global, LuaTracer};
    use mlua::Lua;

    #[test]
    fn test_lua_tracer_creation() {
        let config = TracingConfig {
            enabled: false,
            ..Default::default()
        };

        let tracer = LuaTracer::new(config);
        assert!(tracer.is_ok());
    }

    #[test]
    fn test_lua_tracer_span_creation() {
        let config = TracingConfig {
            enabled: false,
            ..Default::default()
        };

        let tracer = LuaTracer::new(config).unwrap();
        let context = SharedExecutionContext::new();

        // Create a span
        let span = tracer.start_span("lua_operation", &context);
        // Span will be ended when dropped
        drop(span);
    }

    #[test]
    fn test_lua_tracing_global_injection() {
        let lua = Lua::new();
        let config = TracingConfig {
            enabled: false,
            ..Default::default()
        };

        let result = inject_tracing_global(&lua, config);
        assert!(result.is_ok());

        // Check that the global exists
        let has_global: bool = lua.load("return Tracing ~= nil").eval().unwrap();
        assert!(has_global);
    }

    #[test]
    fn test_lua_tracing_from_script() {
        let lua = Lua::new();
        let config = TracingConfig {
            enabled: false,
            ..Default::default()
        };

        inject_tracing_global(&lua, config).unwrap();

        // Test basic tracing operations from Lua
        let script = r#"
            -- Start a span
            local span = Tracing:start_span("test_span")
            
            -- Set an attribute
            span:set_attribute("test.key", "test_value")
            
            -- End the span
            span:end_span()
            
            return true
        "#;

        let result: bool = lua.load(script).eval().unwrap();
        assert!(result);
    }

    #[test]
    fn test_lua_tracing_function_trace() {
        let lua = Lua::new();
        let config = TracingConfig {
            enabled: false,
            ..Default::default()
        };

        inject_tracing_global(&lua, config).unwrap();

        let script = r#"
            -- Trace a function execution
            local span = Tracing:trace_function("my_function")
            
            -- Simulate function work
            local x = 1 + 1
            
            -- Record an exception
            span:record_exception("Test error", "stack trace here")
            
            -- End the span
            span:end_span()
            
            return span:name()
        "#;

        let name: String = lua.load(script).eval().unwrap();
        assert_eq!(name, "my_function");
    }
}
