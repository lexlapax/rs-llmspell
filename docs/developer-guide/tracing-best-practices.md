# Tracing Best Practices Guide

**Comprehensive guide for implementing consistent tracing across the LLMSpell codebase**

**🔗 Navigation**: [← Developer Guide](README.md) | [Architecture](../technical/master-architecture-vision.md) | [Contributing](../../CONTRIBUTING.md)

---

## Overview

This guide establishes best practices for implementing tracing throughout the LLMSpell system to ensure consistent, performant, and useful instrumentation across all components.

## Core Principles

1. **Structured Over Unstructured**: Use tracing spans and structured fields instead of plain log messages
2. **Context Propagation**: Ensure correlation IDs and session information flow through the entire request lifecycle
3. **Performance First**: Minimize overhead in hot paths (<2% at INFO level, <5% at DEBUG level)
4. **Error Context**: Capture sufficient context when errors occur for effective debugging
5. **Consistency**: Follow standard patterns across all crates

## Instrumentation Patterns

### Basic Function Instrumentation

```rust
use tracing::{instrument, info, debug};

#[instrument(skip(self), fields(
    component_id = %self.id(),
    operation = "execute"
))]
pub async fn execute(&self, input: AgentInput) -> Result<AgentOutput> {
    info!("Starting execution");
    let start = std::time::Instant::now();

    let result = self.process(input).await?;

    info!(
        duration_ms = start.elapsed().as_millis() as u64,
        output_size = result.text.len(),
        "Execution completed"
    );

    Ok(result)
}
```

### Error Handling with Context

```rust
#[instrument(err, skip(self, data), fields(data_size = data.len()))]
async fn process_data(&self, data: Vec<u8>) -> Result<ProcessedData> {
    debug!("Processing data");

    match self.validate(&data) {
        Err(e) => {
            error!(
                error = %e,
                data_size = data.len(),
                "Validation failed"
            );
            return Err(e);
        }
        Ok(_) => debug!("Validation successful"),
    }

    self.transform(data).await
}
```

### Performance-Sensitive Code

```rust
// Use conditional compilation for hot paths
#[cfg_attr(feature = "trace-hot-paths", instrument(skip_all))]
pub fn fast_operation(&self, value: u64) -> u64 {
    // Only trace if explicitly enabled
    #[cfg(feature = "trace-hot-paths")]
    trace!("Fast operation: {}", value);

    value * 2
}
```

## Component-Specific Guidelines

### Tools

```rust
#[instrument(skip(self, context), fields(
    tool_name = %self.name(),
    tool_version = %self.version()
))]
pub async fn execute(&self, input: ToolInput, context: ExecutionContext) -> Result<ToolOutput> {
    info!(input_type = ?input.input_type(), "Tool execution started");

    // Tool logic...

    Ok(output)
}
```

### Agents

```rust
#[instrument(skip(self, input, context), fields(
    agent_id = %self.id(),
    agent_type = %self.agent_type(),
    input_size = input.text.len()
))]
pub async fn process(&self, input: AgentInput, context: ExecutionContext) -> Result<AgentOutput> {
    let span = tracing::Span::current();
    span.record("session_id", &context.session_id());

    info!("Agent processing started");

    // Agent logic...

    Ok(output)
}
```

### Workflows

```rust
#[instrument(skip(self, input), fields(
    workflow_name = %self.name(),
    step_count = self.steps.len()
))]
pub async fn execute(&self, input: WorkflowInput) -> Result<WorkflowOutput> {
    for (idx, step) in self.steps.iter().enumerate() {
        let step_span = info_span!("workflow_step",
            step_index = idx,
            step_name = %step.name()
        );

        let _enter = step_span.enter();
        info!("Executing step");

        // Step execution...
    }

    Ok(output)
}
```

## Session and Correlation

### Session Management

```rust
use llmspell_kernel::runtime::tracing::{TracingInstrumentation, SessionType};

// Initialize tracing for a session
let tracing = TracingInstrumentation::new_kernel_session(
    Some(session_id.to_string()),
    "integrated"
);

// Start session with type
tracing.start_session(SessionType::Script, Some("script.lua"));

// Track operations within session
tracing.trace_tool_operation("file_read", Some("config.toml"));
tracing.trace_agent_operation("text_processor", Some("analyzing"));
```

### Correlation ID Propagation

```rust
#[instrument(fields(correlation_id = %correlation_id))]
async fn handle_request(&self, request: Request, correlation_id: Uuid) -> Result<Response> {
    // Correlation ID is automatically included in all spans within this function

    let result = self.process(request).await?;
    self.store(result, correlation_id).await?;

    Ok(Response::new(result))
}
```

## Performance Optimization

### Level-Based Overhead Management

```rust
// Use appropriate levels to control overhead
pub async fn operation(&self) -> Result<()> {
    // Always log critical operations
    info!("Operation started");

    // Detailed logging only at DEBUG level
    debug!(config = ?self.config, "Using configuration");

    // Very detailed logging only at TRACE level
    trace!(internal_state = ?self.state, "Internal state");

    Ok(())
}
```

### Sampling High-Frequency Operations

```rust
use std::sync::atomic::{AtomicU64, Ordering};

static COUNTER: AtomicU64 = AtomicU64::new(0);

pub fn high_frequency_operation(&self) {
    let count = COUNTER.fetch_add(1, Ordering::Relaxed);

    // Sample 1% of operations
    if count % 100 == 0 {
        debug!(sample_count = count, "Operation sample");
    }
}
```

### Batching Related Operations

```rust
#[instrument(skip(self, items))]
pub async fn batch_process(&self, items: Vec<Item>) -> Result<Vec<Output>> {
    let batch_size = items.len();
    info!(batch_size, "Starting batch processing");

    let mut results = Vec::with_capacity(batch_size);
    let mut errors = 0;

    for item in items {
        match self.process_item(item).await {
            Ok(output) => results.push(output),
            Err(_) => errors += 1,
        }
    }

    // Log summary instead of individual items
    info!(
        batch_size,
        successful = results.len(),
        errors,
        "Batch processing completed"
    );

    Ok(results)
}
```

## Testing with Tracing

### Using tracing-test

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tracing_test::traced_test;

    #[traced_test]
    #[tokio::test]
    async fn test_with_tracing() {
        let component = MyComponent::new();

        // Test will capture all tracing output
        let result = component.execute(input).await;

        // Verify tracing output
        assert!(logs_contain("Starting execution"));
        assert!(logs_contain("Execution completed"));
    }
}
```

### Performance Testing

```rust
#[bench]
fn bench_with_tracing(b: &mut Bencher) {
    // Initialize tracing for benchmarks
    let _guard = init_test_tracing();

    b.iter(|| {
        // Benchmark with tracing enabled
        black_box(operation());
    });
}
```

## Environment Configuration

### Development

```bash
# Verbose logging for development
RUST_LOG=debug cargo run

# Specific crate debugging
RUST_LOG=llmspell_kernel=trace,llmspell_agents=debug cargo run
```

### Testing

```bash
# Enable tracing during tests
RUST_LOG=info cargo test

# JSON output for CI/CD
RUST_LOG=info RUST_LOG_FORMAT=json cargo test
```

### Production

```bash
# Conservative production logging
RUST_LOG=warn,llmspell_core=info

# With sampling for high-traffic services
RUST_LOG=info RUST_LOG_SAMPLE_RATE=0.1
```

## Common Pitfalls to Avoid

### ❌ Don't Use Direct Prefixes

```rust
// Wrong
use tracing::info;
tracing::info!("Message");  // ❌ Direct prefix

// Correct
info!("Message");  // ✅ Imported
```

### ❌ Don't Log Sensitive Data

```rust
// Wrong
debug!(password = user.password, "Login attempt");  // ❌

// Correct
debug!(user_id = user.id, "Login attempt");  // ✅
```

### ❌ Don't Over-Instrument

```rust
// Wrong - Too much detail for a simple getter
#[instrument]
pub fn get_id(&self) -> &str {  // ❌
    &self.id
}

// Correct - Instrument significant operations
#[instrument]
pub async fn process(&self) -> Result<()> {  // ✅
    // Complex logic...
}
```

### ❌ Don't Forget to Skip Large Parameters

```rust
// Wrong
#[instrument]  // ❌ Will try to serialize large_data
async fn process(&self, large_data: Vec<u8>) -> Result<()>

// Correct
#[instrument(skip(large_data), fields(data_size = large_data.len()))]  // ✅
async fn process(&self, large_data: Vec<u8>) -> Result<()>
```

## Enforcement and Validation

### Pre-commit Checks

```bash
#!/bin/bash
# Check for tracing anti-patterns

# No direct tracing:: prefixes
! grep -r "tracing::" --include="*.rs" .

# No log:: usage
! grep -r "log::" --include="*.rs" .

# Verify instrumentation on public async functions
# (automated tooling can help here)
```

### CI Pipeline

```yaml
- name: Tracing Validation
  run: |
    cargo clippy -- -W clippy::missing_instrument
    ./scripts/validate-tracing.sh
```

## Migration Guide

For existing code that needs tracing:

1. **Identify Key Functions**: Focus on public APIs and significant operations
2. **Add Basic Instrumentation**: Start with `#[instrument]` on async functions
3. **Add Context Fields**: Include relevant identifiers and metrics
4. **Handle Errors**: Ensure error paths have sufficient context
5. **Test**: Verify tracing output during tests
6. **Optimize**: Adjust levels and sampling based on performance impact

## Resources

- [Tracing Documentation](https://docs.rs/tracing)
- [Tracing Subscriber Configuration](https://docs.rs/tracing-subscriber)
- [OpenTelemetry Integration](https://docs.rs/tracing-opentelemetry)
- [Performance Best Practices](https://tokio.rs/tokio/topics/tracing)

---

**Remember**: Good tracing is like good documentation - it should help future developers (including yourself) understand what the system is doing and why.