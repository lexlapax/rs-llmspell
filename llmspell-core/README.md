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

### Tracing

```rust
use tracing::{info, debug, instrument, warn};
use llmspell_core::traits::base_agent::BaseAgent;

// Instrument async functions for automatic span creation
#[instrument(skip(self), fields(component_id = %self.id()))]
async fn execute(&self, input: AgentInput) -> Result<AgentOutput> {
    info!("Starting component execution");
    debug!(?input, "Processing input");

    // Spans automatically track execution flow
    let result = self.process_internal(input).await?;

    info!(output_size = result.text.len(), "Execution complete");
    Ok(result)
}

// Error context is automatically captured
#[instrument(err)]
async fn validate_input(&self, input: &AgentInput) -> Result<()> {
    if input.text.is_empty() {
        warn!("Empty input received");
        return Err(validation_error!("Input cannot be empty"));
    }
    Ok(())
}

// Performance metrics are automatically collected
#[instrument(skip(self, data), fields(data_size = data.len()))]
async fn process_data(&self, data: Vec<u8>) -> Result<ProcessedData> {
    debug!("Processing {} bytes", data.len());
    // Processing logic...
}
```

### Tracing Configuration

Set the `RUST_LOG` environment variable to control tracing verbosity:

```bash
# Enable INFO level for all crates
RUST_LOG=info cargo run

# Enable DEBUG for specific crates
RUST_LOG=llmspell_core=debug,llmspell_agents=trace cargo run

# Enable structured JSON output for production
RUST_LOG=info RUST_LOG_FORMAT=json cargo run
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