# Core Traits & Foundation

**Thematic guide to llmspell's foundational architecture**

🔗 **Quick Links**: `cargo doc --open -p llmspell-core` | `cargo doc --open -p llmspell-utils` | `cargo doc --open -p llmspell-testing` | [Crate Index](crate-index.md)

---

## Overview

This guide covers the foundational traits, types, and utilities that form the backbone of llmspell's architecture. All components in llmspell implement the `BaseAgent` trait, creating a uniform interface for agents, tools, workflows, and custom components.

**Key Crates**:
- **llmspell-core**: Foundation traits (`BaseAgent`, `Agent`, `Tool`), execution context, error handling
- **llmspell-utils**: Shared utilities (async ops, security, rate limiting, API keys)
- **llmspell-testing**: Testing framework (macros, mocks, property-based testing)

---

## Core Architecture Principles

### Trait-Based Design

Every executable component in llmspell implements `BaseAgent`:

```
BaseAgent (foundation trait)
    ↓
    ├─→ Agent (LLM-backed agents)
    ├─→ Tool (executable tools)
    ├─→ Workflow (orchestration)
    └─→ Custom Components
```

**Benefits**:
- Uniform execution interface across all components
- Built-in event emission and error handling
- Composability through trait objects
- Type-safe execution with `AgentInput`/`AgentOutput`

### Execution Context Pattern

All execution flows through `ExecutionContext`, providing:
- Session management and correlation IDs
- State access (read/write)
- Event emission
- Security context and permissions
- Tracing and metrics

---

## BaseAgent Trait

**Purpose**: Foundation trait for all executable components

**Required Methods**:
```rust
fn metadata(&self) -> &ComponentMetadata;           // Component identity
async fn execute_impl(...) -> Result<AgentOutput>;  // Core logic
async fn validate_input(...) -> Result<()>;         // Input validation
async fn handle_error(...) -> Result<AgentOutput>;  // Error recovery
```

**Optional Methods**:
```rust
async fn stream_execute(...) -> Result<AgentStream>;  // Streaming support
fn supports_streaming(&self) -> bool;                 // Capability flag
fn supports_multimodal(&self) -> bool;                // Multimodal flag
fn supported_media_types(&self) -> Vec<MediaType>;    // Media types
```

**Framework-Provided**:
The `execute()` method is implemented by the framework and handles:
1. Event emission (agent.execution.started/completed/failed)
2. Timing and metrics collection
3. Error propagation through `handle_error()`
4. Calls your `execute_impl()` for actual work

**Implementation Pattern**:
```rust
use llmspell_core::{BaseAgent, ExecutionContext, ComponentMetadata};
use async_trait::async_trait;

pub struct MyComponent {
    metadata: ComponentMetadata,
}

#[async_trait]
impl BaseAgent for MyComponent {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute_impl(
        &self,
        input: AgentInput,
        context: ExecutionContext,
    ) -> Result<AgentOutput> {
        // Your logic here
        Ok(AgentOutput::text("result"))
    }

    async fn validate_input(&self, input: &AgentInput) -> Result<()> {
        // Validation logic
        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
        // Error recovery or propagation
        Err(error)
    }
}
```

📚 **Full Details**: [llmspell-core.md](llmspell-core.md#baseagent-trait)

---

## Execution Context

**Purpose**: Runtime context carrying session state, events, and security

**Key Components**:
- `session_id`: Correlation ID for request tracing
- `state`: Read/write access to shared state
- `event_emitter`: Publish lifecycle events
- `security_context`: Permissions and sandbox settings
- `correlation_id`: Distributed tracing support

**Usage Pattern**:
```rust
async fn execute_impl(
    &self,
    input: AgentInput,
    context: ExecutionContext,
) -> Result<AgentOutput> {
    // Access session state
    let value = context.state.read("key").await?;

    // Write to state
    context.state.write("result", output).await?;

    // Emit events
    context.event_emitter.emit("custom.event", data).await?;

    // Check permissions
    if context.security_context.has_permission("tool.execute") {
        // ...
    }

    Ok(AgentOutput::text("done"))
}
```

📚 **Full Details**: [llmspell-core.md](llmspell-core.md#executioncontext)

---

## Error Handling

**Error Types**:
- `LLMSpellError::Component` - Component-level errors
- `LLMSpellError::Validation` - Input validation failures
- `LLMSpellError::Security` - Security violations
- `LLMSpellError::Provider` - LLM provider errors
- `LLMSpellError::State` - State access errors

**Error Attributes**:
- `severity`: Error, Warning, Info
- `retry_strategy`: Retry eligibility
- `context`: Additional error context

**Best Practices**:
1. Return specific error types with context
2. Implement `handle_error()` for component-specific recovery
3. Use severity levels appropriately
4. Provide actionable error messages

```rust
async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
    match error {
        LLMSpellError::Provider { .. } if error.is_retryable() => {
            // Retry with backoff
            self.retry_with_backoff().await
        }
        LLMSpellError::Security { .. } => {
            // Never retry security errors
            Err(error)
        }
        _ => {
            // Log and propagate
            tracing::error!("Execution failed: {}", error);
            Err(error)
        }
    }
}
```

📚 **Full Details**: [llmspell-core.md](llmspell-core.md#error-handling)

---

## Component Metadata

**Purpose**: Immutable component identification

**Fields**:
```rust
pub struct ComponentMetadata {
    pub id: String,           // Unique identifier
    pub name: String,         // Human-readable name
    pub version: String,      // Semantic version
    pub description: String,  // Component purpose
}
```

**Builder Pattern**:
```rust
let metadata = ComponentMetadata::builder()
    .id("my-component-v1")
    .name("My Component")
    .version("1.0.0")
    .description("Does something useful")
    .build();
```

---

## Shared Utilities (llmspell-utils)

### Async Operations
- `TimeoutExt` - Timeout wrapper for async operations
- `RetryPolicy` - Configurable retry with exponential backoff
- `CircuitBreaker` - Circuit breaker pattern for external calls

```rust
use llmspell_utils::async_utils::{TimeoutExt, RetryPolicy};

let result = some_async_op()
    .with_timeout(Duration::from_secs(30))
    .with_retry(RetryPolicy::exponential(3))
    .await?;
```

### Security Utilities
- `validate_path()` - Path traversal prevention
- `validate_url()` - SSRF protection
- `sanitize_input()` - Input sanitization

```rust
use llmspell_utils::security::{validate_path, validate_url};

let safe_path = validate_path(&user_path)?;
let safe_url = validate_url(&user_url)?;
```

### Rate Limiting
- Token bucket algorithm
- Per-key rate limits
- Sliding window support

```rust
use llmspell_utils::rate_limiter::RateLimiter;

let limiter = RateLimiter::new(100, Duration::from_secs(60));
limiter.check_rate_limit("api-key").await?;
```

### API Key Management
- Secure key storage
- Key rotation support
- Environment variable integration

📚 **Full Details**: [llmspell-utils.md](llmspell-utils.md)

---

## Testing Framework (llmspell-testing)

### Test Categories
Categorize tests with feature flags for selective execution:

```rust
use llmspell_testing::test_categories::*;

#[tokio::test]
#[cfg_attr(not(feature = "integration-tests"), ignore)]
async fn test_database_integration() {
    // Integration test requiring external resources
}

#[tokio::test]
#[cfg_attr(not(feature = "model-tests"), ignore)]
async fn test_llm_provider() {
    // Test requiring LLM API access
}
```

**Available Categories**:
- `unit-tests` (default)
- `integration-tests`
- `model-tests`
- `performance-tests`

### Mock Implementations
Pre-built mocks for testing:

```rust
use llmspell_testing::mocks::{MockProvider, MockStateStore};

let mock_provider = MockProvider::new()
    .with_response("Mocked LLM response")
    .with_latency(Duration::from_millis(100));
```

### Property-Based Testing
Generators for fuzzing and property testing:

```rust
use llmspell_testing::generators::*;

proptest! {
    #[test]
    fn test_agent_input_handling(input in any_agent_input()) {
        // Test with generated inputs
        assert!(my_component.validate_input(&input).is_ok());
    }
}
```

### Test Fixtures
```rust
use llmspell_testing::fixtures::*;

#[tokio::test]
async fn test_with_runtime() {
    let runtime = test_runtime().await;
    // Use pre-configured runtime
}
```

📚 **Full Details**: [llmspell-testing.md](llmspell-testing.md)

---

## Quick Reference: Common Patterns

### Implementing a Custom Agent

```rust
use llmspell_core::*;
use async_trait::async_trait;

pub struct CustomAgent {
    metadata: ComponentMetadata,
    config: CustomConfig,
}

impl CustomAgent {
    pub fn new(config: CustomConfig) -> Self {
        Self {
            metadata: ComponentMetadata::builder()
                .id("custom-agent-v1")
                .name("Custom Agent")
                .version("1.0.0")
                .description("My custom agent")
                .build(),
            config,
        }
    }
}

#[async_trait]
impl BaseAgent for CustomAgent {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute_impl(
        &self,
        input: AgentInput,
        context: ExecutionContext,
    ) -> Result<AgentOutput> {
        // 1. Validate input
        self.validate_input(&input).await?;

        // 2. Access state if needed
        let state_value = context.state.read("key").await.ok();

        // 3. Perform work
        let result = self.do_work(&input, state_value).await?;

        // 4. Emit events
        context.event_emitter.emit("agent.completed", &result).await?;

        // 5. Return output
        Ok(AgentOutput::text(result))
    }

    async fn validate_input(&self, input: &AgentInput) -> Result<()> {
        if input.is_empty() {
            return Err(LLMSpellError::validation("Input cannot be empty"));
        }
        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
        tracing::error!("Agent failed: {}", error);
        Err(error)
    }
}
```

### State Access Pattern

```rust
// Read from state
let value: Option<String> = context.state.read("key").await.ok();

// Write to state
context.state.write("key", value).await?;

// Atomic operations
context.state.update("counter", |old: i32| old + 1).await?;

// Delete from state
context.state.delete("key").await?;
```

### Event Emission Pattern

```rust
// Emit lifecycle events
context.event_emitter.emit("component.started", metadata).await?;
context.event_emitter.emit("component.progress", progress_data).await?;
context.event_emitter.emit("component.completed", result).await?;

// Emit custom events
context.event_emitter.emit_with_options(
    "custom.event",
    data,
    EventOptions {
        priority: EventPriority::High,
        correlation_id: Some(context.correlation_id.clone()),
        ..Default::default()
    }
).await?;
```

---

## Architecture Decision Records

### Why Trait-Based Architecture?

**Decision**: Use `BaseAgent` trait as foundation for all components

**Rationale**:
- Uniform execution interface simplifies composition
- Type-safe without sacrificing flexibility
- Built-in event emission and error handling
- Enables plugin architecture for future extensions
- Aligns with Rust's zero-cost abstractions

**Trade-offs**:
- Async trait requires `async-trait` dependency
- Trait objects have minor runtime cost vs monomorphization
- Learning curve for trait system

### Why ExecutionContext vs Dependency Injection?

**Decision**: Pass `ExecutionContext` to every execution vs DI container

**Rationale**:
- Explicit dependencies visible in method signature
- No hidden global state
- Easier to test (mock context)
- Supports multi-tenancy (context carries tenant ID)
- Aligns with functional programming principles

---

## Performance Considerations

### Component Initialization
- **Target**: <10ms per component
- Use lazy initialization for expensive resources
- Cache metadata objects
- Avoid network calls in constructors

### Execution Overhead
- **Target**: <1% framework overhead
- Event emission is async and non-blocking
- State access uses `Arc<DashMap>` (concurrent)
- Error handling has zero-cost when no error

### Memory Usage
- Components are `Arc<dyn BaseAgent>` - shared ownership
- Metadata is immutable and cloneable
- Context is passed by value (cheap clone via `Arc`)

---

## Related Documentation

- **Individual Crate Docs**:
  - [llmspell-core.md](llmspell-core.md) - Complete trait definitions
  - [llmspell-utils.md](llmspell-utils.md) - Utility functions
  - [llmspell-testing.md](llmspell-testing.md) - Testing framework

- **Other Thematic Guides**:
  - [storage-backends.md](storage-backends.md) - Storage abstraction
  - [rag-pipeline.md](rag-pipeline.md) - RAG architecture
  - [memory-backends.md](memory-backends.md) - Memory system
  - [security-integration.md](security-integration.md) - Security layers

- **General Documentation**:
  - [../developer-guide.md](../developer-guide.md) - Developer overview
  - [../extending-llmspell.md](../extending-llmspell.md) - Extension guide
  - [../../technical/master-architecture-vision.md](../../technical/master-architecture-vision.md) - System architecture

---

**Version**: 0.13.0 | **Phase**: 13b.18.1 | **Last Updated**: 2025-11-08
