# llmspell-core

## Purpose

The foundational crate of LLMSpell providing core traits (`BaseAgent`, `Agent`, `Tool`, `Workflow`), type system, error handling, and execution context. This crate defines the fundamental abstractions that all components in the system must implement, establishing a trait-based architecture where every component is ultimately a `BaseAgent`.

## Core Concepts

- **BaseAgent Trait**: Foundation trait that all components (agents, tools, workflows) must implement
- **Trait Hierarchy**: `BaseAgent` -> specialized traits (`Agent`, `Tool`, `Workflow`)
- **Execution Context**: Carries session info, state access, event emitters, and security context
- **Component Metadata**: Immutable identification for all components (ID, name, version, description)
- **Error System**: Comprehensive error types with severity levels and retry logic
- **Event Integration**: Built-in event emission for component lifecycle
- **Type Safety**: Strong typing for inputs/outputs with `AgentInput`/`AgentOutput`

## Primary Traits/Structs

### BaseAgent Trait

**Purpose**: Foundation trait for all executable components in LLMSpell, providing core execution interface with event emission and error handling.

**When to implement**: Every component that performs work must implement this trait - agents, tools, workflows, or any custom executable component.

**Required methods**:
- `metadata()` - Returns component identification
- `execute_impl()` - Core execution logic (called by framework)
- `validate_input()` - Input validation before execution
- `handle_error()` - Error recovery strategy

**Optional methods**:
- `stream_execute()` - Streaming execution support
- `supports_streaming()` - Indicates streaming capability
- `supports_multimodal()` - Indicates multimodal support
- `supported_media_types()` - Lists supported media types

```rust
use async_trait::async_trait;
use llmspell_core::{
    ComponentMetadata, Result, ExecutionContext, LLMSpellError,
    types::{AgentInput, AgentOutput, AgentStream, MediaType},
    traits::base_agent::BaseAgent
};

#[async_trait]
pub trait BaseAgent: Send + Sync {
    /// Get component metadata (ID, name, version, description)
    fn metadata(&self) -> &ComponentMetadata;
    
    /// Main execution method - DO NOT OVERRIDE
    /// Handles event emission and calls execute_impl()
    async fn execute(&self, input: AgentInput, context: ExecutionContext) -> Result<AgentOutput> {
        // Framework handles:
        // 1. Event emission (start/complete/failed)
        // 2. Timing and metrics
        // 3. Error handling
        // 4. Calls execute_impl() for actual work
    }
    
    /// Implementation-specific logic - MUST IMPLEMENT
    async fn execute_impl(
        &self,
        input: AgentInput,
        context: ExecutionContext,
    ) -> Result<AgentOutput>;
    
    /// Validate input before execution
    async fn validate_input(&self, input: &AgentInput) -> Result<()>;
    
    /// Handle execution errors for recovery
    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput>;
    
    /// Optional: Streaming execution
    async fn stream_execute(
        &self,
        input: AgentInput,
        context: ExecutionContext,
    ) -> Result<AgentStream> {
        Err(LLMSpellError::Component {
            message: "Streaming not supported".to_string(),
            source: None,
        })
    }
    
    /// Optional: Streaming support indicator
    fn supports_streaming(&self) -> bool { false }
    
    /// Optional: Multimodal support indicator
    fn supports_multimodal(&self) -> bool { false }
    
    /// Optional: Supported media types
    fn supported_media_types(&self) -> Vec<MediaType> {
        vec![MediaType::Text]
    }
}
```

**Implementation Example**:
```rust
use llmspell_core::{
    ComponentMetadata, Result, ExecutionContext, LLMSpellError,
    types::{AgentInput, AgentOutput},
    traits::base_agent::BaseAgent
};
use async_trait::async_trait;

pub struct DataProcessor {
    metadata: ComponentMetadata,
    max_size: usize,
}

impl DataProcessor {
    pub fn new(name: String) -> Self {
        Self {
            metadata: ComponentMetadata::new(name, "Processes data inputs".to_string()),
            max_size: 10_000,
        }
    }
}

#[async_trait]
impl BaseAgent for DataProcessor {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }
    
    async fn execute_impl(
        &self,
        input: AgentInput,
        context: ExecutionContext,
    ) -> Result<AgentOutput> {
        // Access state if available
        if let Some(state) = &context.state {
            if let Some(config) = state.get("processor_config").await? {
                // Use configuration from state
            }
        }
        
        // Process the input
        let processed = input.text.to_uppercase();
        
        // Emit custom events if needed
        if let Some(events) = &context.events {
            events.emit("data.processed", serde_json::json!({
                "size": input.text.len(),
                "session": context.session_id
            })).await?;
        }
        
        Ok(AgentOutput::text(processed))
    }
    
    async fn validate_input(&self, input: &AgentInput) -> Result<()> {
        if input.text.is_empty() {
            return Err(LLMSpellError::Validation {
                message: "Input text cannot be empty".to_string(),
                field: Some("text".to_string()),
            });
        }
        
        if input.text.len() > self.max_size {
            return Err(LLMSpellError::Validation {
                message: format!("Input exceeds max size of {}", self.max_size),
                field: Some("text".to_string()),
            });
        }
        
        Ok(())
    }
    
    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
        // Attempt recovery based on error type
        match error {
            LLMSpellError::Validation { .. } => {
                // Return helpful error message
                Ok(AgentOutput::text("Invalid input provided. Please check requirements."))
            }
            _ => Err(error), // Propagate other errors
        }
    }
}
```

### ExecutionContext

**Purpose**: Carries runtime context through component execution including session info, state access, event emitters, and security context.

```rust
use llmspell_core::{ExecutionContext, ContextScope, InheritancePolicy};
use std::sync::Arc;

pub struct ExecutionContext {
    /// Optional session identifier
    pub session_id: Option<String>,
    
    /// Optional conversation/thread identifier
    pub conversation_id: Option<String>,
    
    /// Security context with permissions
    pub security: SecurityContext,
    
    /// Component execution metadata
    pub metadata: ComponentMetadata,
    
    /// Optional state access
    pub state: Option<Arc<dyn StateAccess>>,
    
    /// Optional event emitter
    pub events: Option<Arc<dyn EventEmitter>>,
    
    /// Execution scope (global, tenant, session, user)
    pub scope: ContextScope,
    
    /// How context inherits from parent
    pub inheritance: InheritancePolicy,
}

impl ExecutionContext {
    /// Create new default context
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Create with session
    pub fn with_session(session_id: String) -> Self {
        Self {
            session_id: Some(session_id.clone()),
            scope: ContextScope::Session(session_id),
            ..Default::default()
        }
    }
    
    /// Create child context
    pub fn child(&self) -> Self {
        let mut child = self.clone();
        child.metadata = ComponentMetadata::new_child(&self.metadata);
        child
    }
    
    /// Add state access
    pub fn with_state(mut self, state: Arc<dyn StateAccess>) -> Self {
        self.state = Some(state);
        self
    }
    
    /// Add event emitter
    pub fn with_events(mut self, events: Arc<dyn EventEmitter>) -> Self {
        self.events = Some(events);
        self
    }
}
```

### LLMSpellError

**Purpose**: Central error type for all operations with categorization, severity levels, and retry logic support.

```rust
use thiserror::Error;
use std::time::Duration;

#[derive(Error, Debug)]
pub enum LLMSpellError {
    /// Component execution errors
    #[error("Component error: {message}")]
    Component {
        message: String,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    
    /// Validation errors
    #[error("Validation failed: {message}")]
    Validation {
        message: String,
        field: Option<String>,
    },
    
    /// Storage/persistence errors
    #[error("Storage error in {operation:?}: {message}")]
    Storage {
        message: String,
        operation: Option<String>,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    
    /// Network/provider errors
    #[error("Network error: {message}")]
    Network {
        message: String,
        retryable: bool,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
    
    /// Security/permission errors
    #[error("Security error: {message}")]
    Security {
        message: String,
        permission_required: Option<String>,
    },
    
    /// Timeout errors
    #[error("Operation timed out after {duration:?}")]
    Timeout {
        duration: Duration,
        operation: Option<String>,
    },
    
    /// Internal system errors
    #[error("Internal error: {message}")]
    Internal {
        message: String,
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

impl LLMSpellError {
    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Network { retryable, .. } => *retryable,
            Self::Timeout { .. } => true,
            Self::Storage { .. } => true, // Often transient
            _ => false,
        }
    }
    
    /// Get error severity
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            Self::Security { .. } => ErrorSeverity::Critical,
            Self::Internal { .. } => ErrorSeverity::Critical,
            Self::Component { .. } => ErrorSeverity::Error,
            Self::Validation { .. } => ErrorSeverity::Warning,
            _ => ErrorSeverity::Error,
        }
    }
    
    /// Get error category
    pub fn category(&self) -> ErrorCategory {
        match self {
            Self::Component { .. } => ErrorCategory::Logic,
            Self::Validation { .. } => ErrorCategory::Logic,
            Self::Storage { .. } => ErrorCategory::Resource,
            Self::Network { .. } => ErrorCategory::Network,
            Self::Security { .. } => ErrorCategory::Security,
            Self::Timeout { .. } => ErrorCategory::Network,
            Self::Internal { .. } => ErrorCategory::Internal,
        }
    }
}
```

### ComponentMetadata

**Purpose**: Immutable identification for all components including ID, name, version, and description.

```rust
use uuid::Uuid;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentMetadata {
    /// Unique component identifier
    pub id: ComponentId,
    
    /// Human-readable name
    pub name: String,
    
    /// Component version
    pub version: Version,
    
    /// Description of functionality
    pub description: String,
    
    /// Component creation timestamp
    pub created_at: SystemTime,
    
    /// Optional parent component
    pub parent_id: Option<ComponentId>,
    
    /// Component tags for discovery
    pub tags: Vec<String>,
}

impl ComponentMetadata {
    /// Create new metadata
    pub fn new(name: String, description: String) -> Self {
        Self {
            id: ComponentId::new(),
            name,
            version: Version::new(0, 1, 0),
            description,
            created_at: SystemTime::now(),
            parent_id: None,
            tags: Vec::new(),
        }
    }
    
    /// Get component type (agent, tool, workflow)
    pub fn component_type(&self) -> &str {
        // Inferred from name convention or explicit tag
        if self.tags.contains(&"agent".to_string()) {
            "agent"
        } else if self.tags.contains(&"tool".to_string()) {
            "tool"
        } else if self.tags.contains(&"workflow".to_string()) {
            "workflow"
        } else {
            "component"
        }
    }
    
    /// Create child component metadata
    pub fn new_child(&self) -> Self {
        Self {
            id: ComponentId::new(),
            parent_id: Some(self.id),
            ..self.clone()
        }
    }
}
```

### AgentInput and AgentOutput Types

**Purpose**: Strongly-typed input and output structures for component execution.

```rust
use serde::{Serialize, Deserialize};
use serde_json::Value;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInput {
    /// Primary text input
    pub text: String,
    
    /// Optional structured context
    pub context: Option<Value>,
    
    /// Named parameters
    pub parameters: HashMap<String, Value>,
    
    /// Media attachments (for multimodal)
    pub media: Vec<MediaAttachment>,
    
    /// Input metadata
    pub metadata: InputMetadata,
}

impl AgentInput {
    /// Create text-only input
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            context: None,
            parameters: HashMap::new(),
            media: Vec::new(),
            metadata: InputMetadata::default(),
        }
    }
    
    /// Add context data
    pub fn with_context(mut self, context: Value) -> Self {
        self.context = Some(context);
        self
    }
    
    /// Add parameter
    pub fn with_param(mut self, key: String, value: Value) -> Self {
        self.parameters.insert(key, value);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentOutput {
    /// Primary text output
    pub text: String,
    
    /// Tool calls made (for agents)
    pub tool_calls: Vec<ToolCall>,
    
    /// Structured data output
    pub data: Option<Value>,
    
    /// Media outputs (for multimodal)
    pub media: Vec<MediaOutput>,
    
    /// Output metadata
    pub metadata: OutputMetadata,
}

impl AgentOutput {
    /// Create text-only output
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            tool_calls: Vec::new(),
            data: None,
            media: Vec::new(),
            metadata: OutputMetadata::default(),
        }
    }
    
    /// Add structured data
    pub fn with_data(mut self, data: Value) -> Self {
        self.data = Some(data);
        self
    }
    
    /// Add tool call
    pub fn with_tool_call(mut self, tool_call: ToolCall) -> Self {
        self.tool_calls.push(tool_call);
        self
    }
}
```

## Usage Patterns

### Creating a Custom Component

**When to use**: When you need to implement custom business logic as a reusable component.

**Benefits**: Full integration with LLMSpell's event system, state management, and error handling.

**Example**:
```rust
use llmspell_core::{
    ComponentMetadata, Result, ExecutionContext, LLMSpellError,
    types::{AgentInput, AgentOutput},
    traits::base_agent::BaseAgent,
};
use async_trait::async_trait;
use serde_json::json;

pub struct DataValidator {
    metadata: ComponentMetadata,
    rules: Vec<ValidationRule>,
}

impl DataValidator {
    pub fn new(name: String) -> Self {
        Self {
            metadata: ComponentMetadata::new(
                name,
                "Validates data against configured rules".to_string()
            ),
            rules: vec![
                ValidationRule::MinLength(10),
                ValidationRule::MaxLength(1000),
                ValidationRule::Pattern(r"^\w+".to_string()),
            ],
        }
    }
}

#[async_trait]
impl BaseAgent for DataValidator {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }
    
    async fn execute_impl(
        &self,
        input: AgentInput,
        context: ExecutionContext,
    ) -> Result<AgentOutput> {
        // Validate against all rules
        for rule in &self.rules {
            if !rule.validate(&input.text) {
                return Ok(AgentOutput::text(format!("Validation failed: {:?}", rule))
                    .with_data(json!({
                        "valid": false,
                        "rule": format!("{:?}", rule),
                    })));
            }
        }
        
        // All validations passed
        Ok(AgentOutput::text("Validation successful")
            .with_data(json!({
                "valid": true,
                "rules_checked": self.rules.len(),
            })))
    }
    
    async fn validate_input(&self, input: &AgentInput) -> Result<()> {
        if input.text.is_empty() {
            return Err(LLMSpellError::Validation {
                message: "Cannot validate empty input".to_string(),
                field: Some("text".to_string()),
            });
        }
        Ok(())
    }
    
    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
        Ok(AgentOutput::text(format!("Validation error: {}", error)))
    }
}

#[derive(Debug)]
enum ValidationRule {
    MinLength(usize),
    MaxLength(usize),
    Pattern(String),
}

impl ValidationRule {
    fn validate(&self, text: &str) -> bool {
        match self {
            Self::MinLength(min) => text.len() >= *min,
            Self::MaxLength(max) => text.len() <= *max,
            Self::Pattern(pattern) => {
                // Simplified - real implementation would use regex
                text.contains(pattern)
            }
        }
    }
}
```

### Error Handling with Retry Logic

**When to use**: When dealing with potentially transient failures (network, storage).

**Benefits**: Automatic retry for recoverable errors with exponential backoff.

**Example**:
```rust
use llmspell_core::{LLMSpellError, Result};
use std::time::Duration;
use tokio::time::sleep;

pub async fn execute_with_retry<F, T>(
    operation: F,
    max_retries: u32,
) -> Result<T>
where
    F: Fn() -> std::future::Future<Output = Result<T>>,
{
    let mut attempt = 0;
    let mut last_error = None;
    
    while attempt < max_retries {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(error) => {
                if !error.is_retryable() {
                    return Err(error);
                }
                
                last_error = Some(error);
                attempt += 1;
                
                if attempt < max_retries {
                    // Exponential backoff
                    let delay = Duration::from_millis(100 * 2u64.pow(attempt));
                    sleep(delay).await;
                }
            }
        }
    }
    
    Err(last_error.unwrap_or_else(|| {
        LLMSpellError::Internal {
            message: "Max retries exceeded".to_string(),
            source: None,
        }
    }))
}

// Usage
async fn unreliable_operation() -> Result<String> {
    // Simulated network operation
    Err(LLMSpellError::Network {
        message: "Connection timeout".to_string(),
        retryable: true,
        source: None,
    })
}

let result = execute_with_retry(|| unreliable_operation(), 3).await;
```

## Integration Examples

### With State Management

```rust
use llmspell_core::{ExecutionContext, traits::StateAccess};
use std::sync::Arc;

pub async fn component_with_state(
    context: ExecutionContext,
) -> Result<()> {
    // Check if state is available
    if let Some(state) = &context.state {
        // Store component configuration
        state.set("component_config", serde_json::json!({
            "enabled": true,
            "threshold": 0.8,
        })).await?;
        
        // Retrieve configuration later
        if let Some(config) = state.get("component_config").await? {
            // Use configuration
            let threshold = config["threshold"].as_f64().unwrap_or(0.5);
        }
        
        // Scoped state operations
        let user_scope = format!("user:{}", context.session_id.as_ref().unwrap());
        state.set_scoped(&user_scope, "preferences", serde_json::json!({
            "theme": "dark",
        })).await?;
    }
    
    Ok(())
}
```

### With Event System

```rust
use llmspell_core::{ExecutionContext, traits::EventEmitter, EventData};
use serde_json::json;

pub async fn component_with_events(
    context: ExecutionContext,
) -> Result<()> {
    if let Some(events) = &context.events {
        // Emit structured event
        let event_data = EventData::new("component.custom_event")
            .with_component(context.metadata.id)
            .with_data(json!({
                "action": "processing",
                "items_count": 42,
                "session": context.session_id,
            }))
            .with_correlation(context.metadata.correlation_id().unwrap_or(""));
        
        events.emit_structured(event_data).await?;
        
        // Simple event emission
        events.emit("component.milestone", json!({
            "milestone": "halfway",
            "progress": 0.5,
        })).await?;
    }
    
    Ok(())
}
```

## Configuration

```toml
# Component configuration in LLMSpell config
[components]
# Component discovery paths
discovery_paths = ["./components", "./custom"]

# Component registry settings
[components.registry]
cache_enabled = true
cache_ttl_seconds = 300
lazy_loading = true

# Component execution defaults
[components.execution]
default_timeout_ms = 30000
max_retries = 3
retry_backoff_ms = 100

# Component validation
[components.validation]
strict_mode = true
validate_on_load = true
```

## Performance Considerations

- **Component Caching**: Components are cached after first instantiation - design them to be stateless or manage state carefully
- **Event Emission**: Event emission is fire-and-forget to not block execution - critical events should be handled differently
- **Validation**: Input validation runs before every execution - keep it lightweight
- **Error Handling**: `handle_error()` is called on every error - avoid expensive operations
- **Metadata**: Component metadata is immutable after creation - plan versioning strategy accordingly
- **Streaming**: Only implement streaming if truly needed - adds complexity
- **Context Cloning**: Execution context is cloned for child components - minimize data in context

## Security Considerations

- **Input Validation**: Always validate inputs in `validate_input()` - never trust external data
- **Error Messages**: Don't expose sensitive information in error messages
- **Security Context**: Check `context.security` for permissions before sensitive operations
- **State Access**: State access may be restricted by security context - handle permission errors
- **Event Data**: Be careful what data is included in events - may be logged/stored
- **Component Discovery**: Only load components from trusted paths
- **Resource Limits**: Implement timeouts and resource limits to prevent DoS

## Migration Guide

### From v0.5.x to v0.6.x

Breaking changes:
- `BaseAgent::execute()` now handles event emission internally - don't emit start/complete events in `execute_impl()`
- `ComponentMetadata` now requires `ComponentId` instead of string ID
- Error types reorganized - update match patterns

Migration steps:
1. Remove manual event emission from `execute_impl()`
2. Update metadata creation to use `ComponentMetadata::new()`
3. Update error handling to use new error variants
4. Add `supports_streaming()` if implementing streaming

### From v0.6.x to v0.8.x (Phase 8)

New features:
- Execution context now includes scope and inheritance policy
- Security context is always present (not optional)
- State access supports scoped operations
- Event system supports structured events with correlation

Migration steps:
1. Update context creation to specify scope
2. Check security context for permissions
3. Use scoped state operations for multi-tenancy
4. Add correlation IDs to events for tracing