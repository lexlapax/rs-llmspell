//! Lua-specific tracing implementation
//!
//! This module provides Lua-specific implementations of the tracing traits
//! defined in the bridge layer. It follows the three-layer architecture pattern.

use crate::execution_context::SharedExecutionContext;
use crate::tracing::{ScriptTracer, SpanHandle, TracingConfig};
use mlua::{Lua, Result as LuaResult, Table, UserData, UserDataMethods};
use opentelemetry::{
    global,
    trace::{SpanKind, Status, Tracer},
    Context, KeyValue,
};
use std::sync::Arc;

/// Lua-specific tracer implementation
pub struct LuaTracer {
    tracer: opentelemetry::global::BoxedTracer,
    #[allow(dead_code)]
    config: TracingConfig,
}

impl LuaTracer {
    /// Create a new Lua tracer
    ///
    /// # Errors
    ///
    /// Returns an error if tracer creation fails
    pub fn new(config: TracingConfig) -> Result<Self, String> {
        let tracer = global::tracer("llmspell-lua");

        Ok(Self { tracer, config })
    }

    /// Trace a Lua function execution
    #[must_use]
    pub fn trace_lua_function(&self, function_name: &str) -> Box<dyn SpanHandle> {
        let span = self
            .tracer
            .span_builder(format!("lua.function.{function_name}"))
            .with_kind(SpanKind::Internal)
            .with_attributes(vec![
                KeyValue::new("script.language", "lua"),
                KeyValue::new("function.name", function_name.to_string()),
            ])
            .start(&self.tracer);

        Box::new(LuaSpanHandle { span })
    }

    /// Trace a Lua script execution
    #[must_use]
    pub fn trace_script_execution(&self, script_name: &str) -> Box<dyn SpanHandle> {
        let span = self
            .tracer
            .span_builder(format!("lua.script.{script_name}"))
            .with_kind(SpanKind::Internal)
            .with_attributes(vec![
                KeyValue::new("script.language", "lua"),
                KeyValue::new("script.name", script_name.to_string()),
            ])
            .start(&self.tracer);

        Box::new(LuaSpanHandle { span })
    }
}

impl ScriptTracer for LuaTracer {
    fn start_span(&self, operation: &str, context: &SharedExecutionContext) -> Box<dyn SpanHandle> {
        use opentelemetry::trace::Span;

        let mut span = self
            .tracer
            .span_builder(operation.to_string())
            .with_kind(SpanKind::Internal)
            .start(&self.tracer);

        // Add context attributes
        if let Some(location) = &context.location {
            span.set_attribute(KeyValue::new("source.file", location.source.clone()));
            span.set_attribute(KeyValue::new("source.line", i64::from(location.line)));
        }

        if let Some(correlation_id) = context.correlation_id {
            span.set_attribute(KeyValue::new("correlation.id", correlation_id.to_string()));
        }

        Box::new(LuaSpanHandle { span })
    }

    fn record_event(&self, _name: &str, _attributes: Vec<KeyValue>) {
        // Note: get_active_span is not available in the current opentelemetry version
        // This would need to be implemented differently
    }

    fn set_status(&self, _status: Status) {
        // Note: get_active_span is not available in the current opentelemetry version
        // This would need to be implemented differently
    }

    fn add_attributes(&self, _attributes: Vec<KeyValue>) {
        // Note: get_active_span is not available in the current opentelemetry version
        // This would need to be implemented differently
    }

    fn current_context(&self) -> Context {
        Context::current()
    }

    fn extract_context(&self, context: &SharedExecutionContext) -> Option<Context> {
        // Extract trace context from correlation ID if it contains trace data
        // This would need actual implementation based on how correlation IDs encode trace context
        context.correlation_id.map(|_| Context::current())
    }

    fn inject_context(&self, context: &mut SharedExecutionContext, _trace_context: &Context) {
        // Inject trace context into correlation ID
        // This would need actual implementation based on how we want to encode trace context
        if context.correlation_id.is_none() {
            context.correlation_id = Some(uuid::Uuid::new_v4());
        }
    }
}

/// Lua span handle implementation
struct LuaSpanHandle {
    span: opentelemetry::global::BoxedSpan,
}

impl SpanHandle for LuaSpanHandle {
    fn end(self: Box<Self>) {
        drop(self.span); // Span ends when dropped
    }

    fn record_exception(&mut self, exception: &str, stacktrace: Option<&str>) {
        // Record exception as an event
        use opentelemetry::trace::Span;
        let mut attributes = vec![
            KeyValue::new("exception.message", exception.to_string()),
            KeyValue::new("exception.language", "lua"),
        ];
        if let Some(trace) = stacktrace {
            attributes.push(KeyValue::new("exception.stacktrace", trace.to_string()));
        }
        self.span.add_event("exception", attributes);
    }

    fn set_attribute(&mut self, key: &str, value: String) {
        use opentelemetry::trace::Span;
        self.span
            .set_attribute(KeyValue::new(key.to_string(), value));
    }

    fn context(&self) -> Context {
        // Return current context - span is already active
        Context::current()
    }
}

/// Lua bindings for tracing
pub struct LuaTracingGlobal {
    tracer: Arc<LuaTracer>,
}

impl LuaTracingGlobal {
    /// Create new Lua tracing global
    ///
    /// # Errors
    ///
    /// Returns an error if tracer creation fails
    pub fn new(config: TracingConfig) -> Result<Self, String> {
        Ok(Self {
            tracer: Arc::new(LuaTracer::new(config)?),
        })
    }
}

impl UserData for LuaTracingGlobal {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        // Start a new span
        methods.add_method("start_span", |_lua, this, name: String| {
            let context = SharedExecutionContext::new();
            let span = this.tracer.start_span(&name, &context);
            // Store span handle for later use
            Ok(LuaSpanWrapper {
                span: Some(span),
                name,
            })
        });

        // Record an event
        methods.add_method(
            "record_event",
            |_lua, this, (name, attrs): (String, Option<Table>)| {
                let attributes = if let Some(table) = attrs {
                    parse_attributes(table)?
                } else {
                    Vec::new()
                };
                this.tracer.record_event(&name, attributes);
                Ok(())
            },
        );

        // Set span status
        methods.add_method(
            "set_status",
            |_lua, this, (code, message): (String, Option<String>)| {
                let status = match code.as_str() {
                    "ok" => Status::Ok,
                    "error" => Status::error(message.unwrap_or_default()),
                    _ => Status::Unset,
                };
                this.tracer.set_status(status);
                Ok(())
            },
        );

        // Trace a function execution
        methods.add_method("trace_function", |_lua, this, name: String| {
            let span = this.tracer.trace_lua_function(&name);
            Ok(LuaSpanWrapper {
                span: Some(span),
                name,
            })
        });

        // Trace script execution
        methods.add_method("trace_script", |_lua, this, name: String| {
            let span = this.tracer.trace_script_execution(&name);
            Ok(LuaSpanWrapper {
                span: Some(span),
                name,
            })
        });
    }
}

/// Wrapper for span handles in Lua
struct LuaSpanWrapper {
    span: Option<Box<dyn SpanHandle>>,
    name: String,
}

impl UserData for LuaSpanWrapper {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        // End the span
        methods.add_method_mut("end_span", |_lua, this, ()| {
            if let Some(span) = this.span.take() {
                span.end();
            }
            Ok(())
        });

        // Set an attribute
        methods.add_method_mut(
            "set_attribute",
            |_lua, this, (key, value): (String, String)| {
                if let Some(ref mut span) = this.span {
                    span.set_attribute(&key, value);
                }
                Ok(())
            },
        );

        // Record an exception
        methods.add_method_mut(
            "record_exception",
            |_lua, this, (msg, trace): (String, Option<String>)| {
                if let Some(ref mut span) = this.span {
                    span.record_exception(&msg, trace.as_deref());
                }
                Ok(())
            },
        );

        // Get span name
        methods.add_method("name", |_lua, this, ()| Ok(this.name.clone()));
    }
}

/// Parse Lua table into OpenTelemetry attributes
fn parse_attributes(table: Table) -> LuaResult<Vec<KeyValue>> {
    let mut attributes = Vec::new();

    for pair in table.pairs::<String, mlua::Value>() {
        let (key, value) = pair?;
        let attr = match value {
            mlua::Value::String(s) => KeyValue::new(key, s.to_str()?.to_string()),
            mlua::Value::Integer(i) => KeyValue::new(key, i),
            mlua::Value::Number(n) => KeyValue::new(key, n),
            mlua::Value::Boolean(b) => KeyValue::new(key, b),
            _ => continue, // Skip unsupported types
        };
        attributes.push(attr);
    }

    Ok(attributes)
}

/// Inject tracing global into Lua environment
///
/// # Errors
///
/// Returns an error if global injection fails
pub fn inject_tracing_global(lua: &Lua, config: TracingConfig) -> Result<(), String> {
    let tracing_global = LuaTracingGlobal::new(config)?;

    lua.globals()
        .set("Tracing", tracing_global)
        .map_err(|e| format!("Failed to inject tracing global: {e}"))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lua_tracer_creation() {
        let config = TracingConfig::default();
        let result = LuaTracer::new(config);
        assert!(result.is_ok());
    }

    #[test]
    fn test_span_creation() {
        let config = TracingConfig::default();
        let tracer = LuaTracer::new(config).unwrap();
        let context = SharedExecutionContext::new();

        let span = tracer.start_span("test_operation", &context);
        // Span will be ended when dropped
        drop(span);
    }
}
