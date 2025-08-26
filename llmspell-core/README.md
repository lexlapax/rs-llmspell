# llmspell-core

**Core traits, types, and infrastructure for the LLMSpell system**

**🔗 Navigation**: [← Project Root](../) | [Documentation](../docs/) | [Examples](../examples/)

## Overview

`llmspell-core` is the foundational crate that defines the core abstractions and infrastructure for the LLMSpell framework. It provides the essential building blocks for creating agents, tools, and workflows that can be composed and orchestrated.

## Features

- **Component System**: Trait-based architecture with `BaseAgent`, `Agent`, `Tool`, and `Workflow` traits
- **Type System**: Unique component identification with `ComponentId`, semantic versioning, and metadata
- **Error Handling**: Comprehensive error types with severity levels, categorization, and retry logic
- **Structured Logging**: JSON and human-readable logging formats with component lifecycle tracking
- **Async-First**: All component operations are async-enabled using Tokio

## Core Concepts

### BaseAgent

The foundational trait that all components implement:

```rust
use llmspell_core::{ComponentMetadata, Result};
use llmspell_core::traits::base_agent::{BaseAgent, AgentInput, AgentOutput, ExecutionContext};
use async_trait::async_trait;

#[async_trait]
impl BaseAgent for MyComponent {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }
    
    async fn execute(&self, input: AgentInput, context: ExecutionContext) -> Result<AgentOutput> {
        // Component logic here
        Ok(AgentOutput::new("Result".to_string()))
    }
    
    async fn validate_input(&self, input: &AgentInput) -> Result<()> {
        // Validation logic
        Ok(())
    }
    
    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
        // Error recovery
        Ok(AgentOutput::new(format!("Error: {}", error)))
    }
}
```

### Component Types

- **Agent**: LLM-powered components with conversation management
- **Tool**: Functional components with parameter validation and schemas
- **Workflow**: Orchestration components that manage execution of other components

### Error Handling

```rust
use llmspell_core::{LLMSpellError, Result, component_error, validation_error};

// Create errors with context
let err = LLMSpellError::Validation {
    message: "Invalid input".to_string(),
    field: Some("email".to_string()),
};

// Use convenience macros
let err = validation_error!("Invalid format", "username");
let err = component_error!("Initialization failed");

// Check error properties
if err.is_retryable() {
    let delay = err.retry_delay_ms().unwrap_or(1000);
    // Retry after delay
}
```

### Logging

```rust
use llmspell_core::logging::{init_from_env, info, debug};
use llmspell_core::{log_component_event, log_execution_start, log_execution_end};

// Initialize logging
init_from_env()?;

// Log component events
log_component_event!(component, "Processing started");
log_execution_start!(component, input);
log_execution_end!(component, duration, true);
```

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
llmspell-core = { path = "../llmspell-core" }
```

## Examples

See the documentation for comprehensive examples of implementing agents, tools, and workflows.

## License

This project is licensed under the MIT License - see the LICENSE file for details.