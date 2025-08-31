# Dependency Injection Patterns in llmspell-bridge

## Overview

The llmspell-bridge crate uses dependency injection (DI) patterns to ensure testability, modularity, and separation of concerns. This document outlines the patterns and best practices for DI in the codebase.

## Core Principles

### 1. Trait-Based Abstraction
All pluggable components are defined as traits, not concrete types:
- `Profiler` - CPU profiling abstraction
- `HookProfiler` - Hook execution profiling
- `CircuitBreaker` - Fault tolerance mechanism
- `SessionRecorder` - Session recording/replay

### 2. Constructor Injection via Builder Pattern
Components are injected through the `DiagnosticsBridgeBuilder`:

```rust
let bridge = DiagnosticsBridge::builder()
    .profiler(Box::new(CustomProfiler::new()))
    .hook_profiler(Box::new(CustomHookProfiler::new()))
    .circuit_breaker(Box::new(CustomCircuitBreaker::new()))
    .session_recorder(Box::new(CustomSessionRecorder::new()))
    .build();
```

### 3. Null Object Pattern for Testing
Every injectable component has a corresponding null implementation:
- `NullProfiler` - No-op profiler (avoids signal handlers)
- `NullHookProfiler` - No-op hook profiler
- `NullCircuitBreaker` - Always-open circuit breaker
- `NullSessionRecorder` - No file I/O session recorder

## Test Helpers

### Common Test Module
Tests should use the common test helpers located in `tests/common/mod.rs`:

```rust
mod common;
use common::create_test_bridge;

#[test]
fn test_something() {
    let bridge = create_test_bridge();
    // Test with null implementations - no side effects
}
```

### Custom Component Testing
For testing specific components:

```rust
use common::create_test_bridge_with_profiler;

#[test]
fn test_custom_profiler() {
    let profiler = Box::new(MyCustomProfiler::new());
    let bridge = create_test_bridge_with_profiler(profiler);
    // Test with custom profiler, other components are null
}
```

## Anti-Patterns to Avoid

### 1. Factory Functions
❌ **Don't** create factory functions in `src/`:
```rust
// BAD - Factory function in production code
pub fn create_test_bridge() -> DiagnosticsBridge {
    // ...
}
```

✅ **Do** use builders or constructors:
```rust
// GOOD - Builder pattern
DiagnosticsBridge::builder().build()
```

### 2. Conditional Compilation for Tests
❌ **Don't** use `#[cfg(test)]` for test-specific behavior in production code:
```rust
// BAD - Test-specific behavior in production
impl DiagnosticsBridge {
    #[cfg(test)]
    pub fn create_test_bridge() -> Self {
        // ...
    }
}
```

✅ **Do** use dependency injection with null implementations:
```rust
// GOOD - Test helpers in tests/ directory
// tests/common/mod.rs
pub fn create_test_bridge() -> DiagnosticsBridge {
    DiagnosticsBridge::builder()
        .profiler(Box::new(NullProfiler::new()))
        .build()
}
```

### 3. Hardcoded Dependencies
❌ **Don't** hardcode dependencies in constructors:
```rust
// BAD - Hardcoded dependency
impl DiagnosticsBridge {
    pub fn new() -> Self {
        Self {
            profiler: Box::new(PprofProfiler::new()), // Hardcoded!
            // ...
        }
    }
}
```

✅ **Do** allow injection with sensible defaults:
```rust
// GOOD - Injectable with defaults
impl DiagnosticsBridge {
    pub fn builder() -> DiagnosticsBridgeBuilder {
        DiagnosticsBridgeBuilder::new()
    }
}

impl DiagnosticsBridgeBuilder {
    pub fn build(self) -> DiagnosticsBridge {
        DiagnosticsBridge {
            profiler: self.profiler
                .unwrap_or_else(|| Box::new(PprofProfiler::new())),
            // ...
        }
    }
}
```

## Exceptions

### Architectural Factory Functions
The following factory functions are architectural necessities:
- `engine/factory.rs` - Multi-language engine creation
- `workflows.rs` - Workflow pattern constructors
- `lua/globals/` - Lua API table creation

These are part of the public API and not test-specific helpers.

### Unit Test Modules
Standard Rust `#[cfg(test)]` modules in source files are acceptable for unit tests. These are completely excluded from release builds and don't affect production code behavior.

## Benefits

1. **Testability**: Components can be easily mocked/stubbed
2. **Modularity**: Components can be swapped without changing consumer code
3. **Safety**: Test implementations avoid side effects (no file I/O, no signals)
4. **Flexibility**: Different configurations for different environments
5. **Maintainability**: Clear separation between production and test code