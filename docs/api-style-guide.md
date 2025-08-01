# rs-llmspell API Style Guide

**Version**: 1.0
**Date**: August 1, 2025
**Status**: Official Guide

## Introduction

This guide defines the API design standards for rs-llmspell. All public APIs must follow these conventions to ensure consistency and usability.

## Naming Conventions

### Type Names

```rust
// ✅ GOOD
pub struct SessionManager { }
pub trait StorageBackend { }
pub enum SessionStatus { }

// ❌ BAD
pub struct session_manager { }      // Should be UpperCamelCase
pub struct SessionManagerService { } // Avoid Service suffix
```

### Function and Method Names

```rust
// ✅ GOOD
pub fn create_session() -> Result<Session>
pub fn get_artifact() -> Result<Artifact>
pub async fn execute_workflow() -> Result<()>

// ❌ BAD
pub fn CreateSession() -> Result<Session>    // Should be snake_case
pub fn retrieve_artifact() -> Result<Artifact> // Use get_ not retrieve_
```

### Constants

```rust
// ✅ GOOD
pub const MAX_SESSIONS: usize = 1000;
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

// ❌ BAD
pub const MaxSessions: usize = 1000;  // Should be SCREAMING_SNAKE_CASE
```

## Constructor Patterns

### Simple Construction

Use `new()` for simple, infallible construction:

```rust
// ✅ GOOD
impl Tool {
    pub fn new() -> Self {
        Self { 
            id: Uuid::new_v4(),
            created_at: Utc::now(),
        }
    }
}

// ❌ BAD
impl Tool {
    pub fn create() -> Self { }      // Use new() for simple construction
    pub fn make() -> Self { }        // Avoid non-standard names
}
```

### Fallible Construction

Return `Result` when construction can fail:

```rust
// ✅ GOOD
impl SessionManager {
    pub fn new(config: Config) -> Result<Self> {
        config.validate()?;
        Ok(Self { config })
    }
}
```

### Parameterized Construction

Use descriptive names for specialized constructors:

```rust
// ✅ GOOD
impl Buffer {
    pub fn new() -> Self { }
    pub fn with_capacity(cap: usize) -> Self { }
    pub fn from_string(s: String) -> Self { }
}

// ❌ BAD
impl Buffer {
    pub fn new_with_cap(cap: usize) -> Self { } // Use with_capacity
    pub fn new_from_str(s: &str) -> Self { }    // Use from_str
}
```

### Builder Pattern

Use builders for types with 3+ configuration options:

```rust
// ✅ GOOD
pub struct SessionConfig {
    max_sessions: usize,
    retention_days: u32,
    auto_save: bool,
    compression: bool,
}

impl SessionConfig {
    pub fn builder() -> SessionConfigBuilder {
        SessionConfigBuilder::default()
    }
}

pub struct SessionConfigBuilder {
    max_sessions: Option<usize>,
    retention_days: Option<u32>,
    auto_save: Option<bool>,
    compression: Option<bool>,
}

impl SessionConfigBuilder {
    pub fn max_sessions(mut self, max: usize) -> Self {
        self.max_sessions = Some(max);
        self
    }
    
    pub fn build(self) -> Result<SessionConfig> {
        Ok(SessionConfig {
            max_sessions: self.max_sessions.unwrap_or(100),
            retention_days: self.retention_days.unwrap_or(30),
            auto_save: self.auto_save.unwrap_or(true),
            compression: self.compression.unwrap_or(false),
        })
    }
}

// Usage
let config = SessionConfig::builder()
    .max_sessions(500)
    .retention_days(7)
    .build()?;
```

## Getter and Setter Patterns

### Simple Field Access

Omit `get_` prefix for simple field access:

```rust
// ✅ GOOD
impl Session {
    pub fn id(&self) -> &SessionId { &self.id }
    pub fn name(&self) -> &str { &self.name }
    pub fn status(&self) -> SessionStatus { self.status }
}

// ❌ BAD
impl Session {
    pub fn get_id(&self) -> &SessionId { }    // Don't use get_ for fields
    pub fn get_name(&self) -> &str { }        // Don't use get_ for fields
}
```

### Computed or Lookup Access

Use `get_` prefix for operations that compute or look up values:

```rust
// ✅ GOOD
impl SessionManager {
    pub async fn get_session(&self, id: &SessionId) -> Result<Session> {
        // Database lookup
        self.storage.find_by_id(id).await
    }
    
    pub fn get_active_count(&self) -> usize {
        // Computation required
        self.sessions.iter().filter(|s| s.is_active()).count()
    }
}
```

### Setters

Always use `set_` prefix for setters:

```rust
// ✅ GOOD
impl Session {
    pub fn set_name(&mut self, name: String) {
        self.name = name;
        self.updated_at = Utc::now();
    }
}
```

## Method Naming Patterns

### Action Methods

Use clear verb-first naming:

```rust
// ✅ GOOD
session.suspend()
workflow.execute()
agent.query(prompt)
tool.validate_params(params)

// ❌ BAD
session.suspension()      // Should be a verb
workflow.execution()      // Should be a verb
agent.prompt_query(prompt) // Verb should come first
```

### Conversion Methods

Follow Rust conventions for conversions:

```rust
// ✅ GOOD
impl MyType {
    pub fn as_str(&self) -> &str { }        // Cheap reference conversion
    pub fn to_string(&self) -> String { }    // Allocating conversion
    pub fn into_inner(self) -> InnerType { } // Consuming conversion
}

// From trait for type conversions
impl From<String> for MyType { }
impl TryFrom<&str> for MyType { }
```

### Collection Methods

Standard patterns for collections:

```rust
// ✅ GOOD
impl Registry {
    pub fn add(&mut self, item: Item)
    pub fn remove(&mut self, id: &Id) -> Option<Item>
    pub fn contains(&self, id: &Id) -> bool
    pub fn get(&self, id: &Id) -> Option<&Item>
    pub fn get_mut(&mut self, id: &Id) -> Option<&mut Item>
    pub fn iter(&self) -> impl Iterator<Item = &Item>
    pub fn clear(&mut self)
}
```

## Async Patterns

### Async Methods

Mark async trait bounds properly:

```rust
// ✅ GOOD
#[async_trait]
pub trait Workflow: Send + Sync {
    async fn execute(&self, context: Context) -> Result<Output>;
}

// For script bridge sync wrappers
impl ScriptBridge {
    pub fn query_sync(&self, prompt: &str) -> Result<String> {
        tokio::runtime::Runtime::new()?
            .block_on(self.query(prompt))
    }
}
```

## Error Handling

### Result Types

All fallible operations return `Result`:

```rust
// ✅ GOOD
pub fn parse_config(input: &str) -> Result<Config, ConfigError>
pub async fn connect(&self) -> Result<Connection>

// ❌ BAD
pub fn parse_config(input: &str) -> Config  // May panic
pub fn connect(&self) -> Option<Connection>  // Loses error information
```

### Error Types

Use descriptive error types with context:

```rust
// ✅ GOOD
#[derive(Debug, thiserror::Error)]
pub enum SessionError {
    #[error("Session not found: {id}")]
    NotFound { id: SessionId },
    
    #[error("Invalid state transition from {from:?} to {to:?}")]
    InvalidTransition { 
        from: SessionStatus, 
        to: SessionStatus 
    },
    
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),
}
```

## Type Suffixes

### Standard Suffixes

Use these standard suffixes consistently:

| Suffix | Usage | Example |
|--------|-------|---------|
| `Manager` | Service that manages resources | `SessionManager` |
| `Tool` | Implements Tool trait | `FileReadTool` |
| `Config` | Configuration struct | `AgentConfig` |
| `Builder` | Builder pattern | `WorkflowBuilder` |
| `Registry` | Component registry | `ToolRegistry` |
| `Factory` | Creates instances | `AgentFactory` |
| `Provider` | External service provider | `OpenAIProvider` |
| `Handler` | Handles events/requests | `EventHandler` |
| `Context` | Execution context | `HookContext` |
| `Error` | Error types | `SessionError` |

### Avoid These Suffixes

- ❌ `Service` - Use `Manager` or no suffix
- ❌ `Impl` - Implementation details shouldn't be in public names
- ❌ `Base` - Use traits for abstraction instead
- ❌ `Helper` - Too vague, be specific

## Lifecycle Methods

### Standard Lifecycle Patterns

```rust
// ✅ GOOD - Service lifecycle
impl Manager {
    pub async fn start(&mut self) -> Result<()>
    pub async fn stop(&mut self) -> Result<()>
    pub fn is_running(&self) -> bool
}

// ✅ GOOD - Pausable lifecycle
impl Session {
    pub async fn suspend(&mut self) -> Result<()>
    pub async fn resume(&mut self) -> Result<()>
    pub fn is_suspended(&self) -> bool
}

// ✅ GOOD - Resource lifecycle
impl Resource {
    pub fn acquire() -> Result<Self>
    pub fn release(self) -> Result<()>
}
```

## Documentation Standards

Every public API must have documentation:

```rust
/// Brief one-line summary ending with a period.
///
/// More detailed explanation if needed. Explain what the function
/// does, not how it does it.
///
/// # Arguments
///
/// * `param1` - Description of first parameter
/// * `param2` - Description of second parameter
///
/// # Returns
///
/// Description of the return value.
///
/// # Errors
///
/// Returns `ErrorType` when:
/// - Condition 1 that causes error
/// - Condition 2 that causes error
///
/// # Examples
///
/// ```rust
/// use llmspell_core::*;
/// 
/// let result = function(param1, param2)?;
/// assert_eq!(result, expected);
/// ```
///
/// # Panics
///
/// Panics if invariant X is violated (only if applicable).
pub fn function(param1: Type1, param2: Type2) -> Result<ReturnType> {
    // Implementation
}
```

## Deprecation Policy

When changing APIs, use deprecation warnings:

```rust
// Mark old API as deprecated
#[deprecated(since = "0.6.0", note = "Use `get_session` instead")]
pub async fn retrieve_session(&self, id: &SessionId) -> Result<Session> {
    self.get_session(id).await
}

// For types, provide type alias
#[deprecated(since = "0.6.0", note = "Use `HookExecutor` instead")]
pub type HookExecutorService = HookExecutor;
```

## Examples

### Complete API Example

```rust
/// Manages user sessions with persistence and lifecycle control.
pub struct SessionManager {
    storage: Arc<dyn StorageBackend>,
    config: SessionManagerConfig,
}

impl SessionManager {
    /// Creates a new session manager with the given configuration.
    ///
    /// # Errors
    ///
    /// Returns `SessionError::InvalidConfig` if configuration is invalid.
    pub fn new(config: SessionManagerConfig) -> Result<Self> {
        config.validate()?;
        Ok(Self {
            storage: config.storage_backend.clone(),
            config,
        })
    }
    
    /// Creates a new session with the specified options.
    ///
    /// # Examples
    ///
    /// ```rust
    /// let session_id = manager.create_session(
    ///     CreateSessionOptions::builder()
    ///         .name("My Session")
    ///         .build()
    /// ).await?;
    /// ```
    pub async fn create_session(&self, options: CreateSessionOptions) -> Result<SessionId> {
        // Implementation
    }
    
    /// Retrieves a session by its ID.
    ///
    /// # Errors
    ///
    /// Returns `SessionError::NotFound` if session doesn't exist.
    pub async fn get_session(&self, id: &SessionId) -> Result<Session> {
        // Implementation
    }
}
```

## Enforcement

1. All PRs must follow this style guide
2. CI will check for deprecated API usage
3. Breaking changes require migration guide
4. New public APIs require examples

## Version History

- 1.0 (2025-08-01): Initial version based on R.1.1 analysis