# Extension Architecture

## Overview

LLMSpell's extension architecture enables building modular, pluggable systems using Rust's trait system. This pattern allows you to add functionality dynamically without modifying core code, making your tools and agents highly extensible.

This guide demonstrates how to build extension-based systems, including:
- Defining extension traits for pluggable functionality
- Implementing plugin registries for discovery
- Creating extensible components that use registered extensions
- Runtime extension loading and execution

## Core Concepts

### 1. Extension Trait

The extension trait defines the contract that all plugins must implement:

```rust
use async_trait::async_trait;
use std::collections::HashMap;

#[async_trait]
pub trait Extension: Send + Sync + std::fmt::Debug {
    /// Unique identifier for this extension
    fn id(&self) -> &str;

    /// Extension version
    fn version(&self) -> &str;

    /// Human-readable description
    fn description(&self) -> &str;

    /// Initialize the extension (called once during registration)
    async fn initialize(&self) -> Result<(), String>;

    /// Execute extension with parameters
    async fn execute(
        &self,
        params: &HashMap<String, serde_json::Value>,
    ) -> Result<serde_json::Value, String>;

    /// Get supported operations for this extension
    fn supported_operations(&self) -> Vec<String>;
}
```

### 2. Extension Registry

The registry manages extension lifecycle and discovery:

```rust
use std::sync::Arc;

#[derive(Debug, Default)]
pub struct ExtensionRegistry {
    extensions: HashMap<String, Arc<dyn Extension>>,
}

impl ExtensionRegistry {
    pub fn new() -> Self {
        Self {
            extensions: HashMap::new(),
        }
    }

    /// Register a new extension
    pub async fn register(&mut self, extension: Arc<dyn Extension>) -> Result<(), String> {
        let id = extension.id().to_string();

        // Initialize the extension
        extension.initialize().await?;

        // Check for conflicts
        if self.extensions.contains_key(&id) {
            return Err(format!("Extension '{}' already registered", id));
        }

        self.extensions.insert(id, extension);
        Ok(())
    }

    /// Get extension by ID
    pub fn get(&self, id: &str) -> Option<&Arc<dyn Extension>> {
        self.extensions.get(id)
    }

    /// List all registered extensions
    pub fn list(&self) -> Vec<&Arc<dyn Extension>> {
        self.extensions.values().collect()
    }

    /// Get extension IDs
    pub fn extension_ids(&self) -> Vec<String> {
        self.extensions.keys().cloned().collect()
    }
}
```

### 3. Extensible Component

Components use the registry to execute operations through registered extensions:

```rust
use llmspell_core::{
    ComponentMetadata, ExecutionContext, LLMSpellError,
    types::{AgentInput, AgentOutput},
    traits::base_agent::BaseAgent,
    Result
};

#[derive(Debug)]
pub struct ExtensibleTool {
    metadata: ComponentMetadata,
    registry: Arc<ExtensionRegistry>,
}

impl ExtensibleTool {
    pub fn new(name: String, registry: Arc<ExtensionRegistry>) -> Self {
        Self {
            metadata: ComponentMetadata::new(
                name,
                "Tool that executes operations through registered extensions".to_string(),
            ),
            registry,
        }
    }
}

#[async_trait]
impl BaseAgent for ExtensibleTool {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }

    async fn execute_impl(
        &self,
        input: AgentInput,
        _context: ExecutionContext,
    ) -> Result<AgentOutput> {
        // Extract extension ID from parameters
        let extension_id = input
            .parameters
            .get("extension")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LLMSpellError::Validation {
                message: "Missing 'extension' parameter".to_string(),
                field: Some("extension".to_string()),
            })?;

        // Get the extension
        let extension = self.registry
            .get(extension_id)
            .ok_or_else(|| LLMSpellError::Component {
                message: format!("Extension '{}' not found", extension_id),
                source: None,
            })?;

        // Prepare parameters for extension
        let mut ext_params = HashMap::new();
        for (key, value) in &input.parameters {
            if key != "extension" {
                ext_params.insert(key.clone(), value.clone());
            }
        }

        // Execute extension
        match extension.execute(&ext_params).await {
            Ok(result) => {
                let response = json!({
                    "extension_id": extension_id,
                    "extension_version": extension.version(),
                    "result": result,
                    "success": true
                });

                Ok(AgentOutput::text(response.to_string()))
            }
            Err(error) => Err(LLMSpellError::Component {
                message: format!("Extension execution failed: {}", error),
                source: None,
            }),
        }
    }

    async fn validate_input(&self, input: &AgentInput) -> Result<()> {
        if !input.parameters.contains_key("extension") {
            return Err(LLMSpellError::Validation {
                message: "Missing required 'extension' parameter".to_string(),
                field: Some("extension".to_string()),
            });
        }
        Ok(())
    }

    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
        let available_extensions: Vec<String> = self.registry.extension_ids();

        let error_response = json!({
            "error": error.to_string(),
            "available_extensions": available_extensions,
            "success": false
        });

        Ok(AgentOutput::text(error_response.to_string()))
    }
}
```

## Implementation Example

Here's a complete example implementing a text processing extension:

```rust
#[derive(Debug)]
pub struct TextProcessorExtension;

#[async_trait]
impl Extension for TextProcessorExtension {
    fn id(&self) -> &'static str {
        "text_processor"
    }

    fn version(&self) -> &'static str {
        "1.0.0"
    }

    fn description(&self) -> &'static str {
        "Text processing operations like uppercase, lowercase, reverse"
    }

    async fn initialize(&self) -> Result<(), String> {
        // Perform any setup needed (load resources, connect to services, etc.)
        Ok(())
    }

    async fn execute(
        &self,
        params: &HashMap<String, serde_json::Value>,
    ) -> Result<serde_json::Value, String> {
        let operation = params
            .get("operation")
            .and_then(|v| v.as_str())
            .unwrap_or("uppercase");

        let data = params
            .get("data")
            .and_then(|v| v.as_str())
            .ok_or_else(|| "Missing 'data' parameter".to_string())?;

        let result = match operation {
            "uppercase" => data.to_uppercase(),
            "lowercase" => data.to_lowercase(),
            "reverse" => data.chars().rev().collect::<String>(),
            _ => return Err(format!("Unknown operation: {}", operation)),
        };

        Ok(json!({
            "processed_text": result,
            "original_length": data.len(),
            "operation": operation
        }))
    }

    fn supported_operations(&self) -> Vec<String> {
        vec![
            "uppercase".to_string(),
            "lowercase".to_string(),
            "reverse".to_string(),
        ]
    }
}
```

## Usage Pattern

Here's how to set up and use the extension system:

```rust
use std::sync::Arc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Create extension registry
    let mut registry = ExtensionRegistry::new();

    // 2. Register extensions
    let text_extension = Arc::new(TextProcessorExtension);
    registry.register(text_extension).await?;

    // 3. Create extensible tool
    let registry_arc = Arc::new(registry);
    let tool = ExtensibleTool::new(
        "extensible_processor".to_string(),
        registry_arc.clone()
    );

    // 4. Use the tool
    let input = AgentInput::text("hello world")
        .with_parameter("extension", json!("text_processor"))
        .with_parameter("operation", json!("uppercase"));

    let context = ExecutionContext::new();
    let result = tool.execute(input, context).await?;

    println!("Result: {}", result.text);

    Ok(())
}
```

## Design Patterns

### 1. Extension Discovery

Allow extensions to advertise their capabilities:

```rust
pub trait Extension: Send + Sync + std::fmt::Debug {
    // ... existing methods ...

    /// Get extension capabilities
    fn capabilities(&self) -> Vec<String> {
        self.supported_operations()
    }

    /// Check if extension supports a specific capability
    fn supports(&self, capability: &str) -> bool {
        self.supported_operations()
            .iter()
            .any(|op| op == capability)
    }
}
```

### 2. Extension Versioning

Handle version compatibility:

```rust
pub struct ExtensionInfo {
    pub id: String,
    pub version: semver::Version,
    pub min_api_version: semver::Version,
}

impl ExtensionRegistry {
    pub fn is_compatible(&self, extension: &dyn Extension) -> bool {
        let version = semver::Version::parse(extension.version()).unwrap();
        // Check against API version requirements
        version >= MIN_SUPPORTED_VERSION
    }
}
```

### 3. Extension Configuration

Support extension-specific configuration:

```rust
#[async_trait]
pub trait Extension: Send + Sync + std::fmt::Debug {
    // ... existing methods ...

    /// Configure extension with settings
    async fn configure(
        &mut self,
        config: &HashMap<String, serde_json::Value>,
    ) -> Result<(), String> {
        // Default: no configuration needed
        Ok(())
    }
}
```

### 4. Extension Lifecycle Hooks

Provide lifecycle management:

```rust
#[async_trait]
pub trait Extension: Send + Sync + std::fmt::Debug {
    // ... existing methods ...

    /// Called when extension is being unloaded
    async fn shutdown(&self) -> Result<(), String> {
        Ok(())
    }

    /// Called periodically for health checks
    async fn health_check(&self) -> Result<bool, String> {
        Ok(true)
    }
}
```

## Best Practices

### 1. Error Handling

Extensions should provide meaningful error messages:

```rust
async fn execute(
    &self,
    params: &HashMap<String, serde_json::Value>,
) -> Result<serde_json::Value, String> {
    // Validate parameters with clear error messages
    let data = params
        .get("data")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            format!(
                "Missing required 'data' parameter for {} extension",
                self.id()
            )
        })?;

    // Provide operation-specific error context
    let operation = params.get("operation")
        .and_then(|v| v.as_str())
        .unwrap_or("default");

    if !self.supports(operation) {
        return Err(format!(
            "Unsupported operation '{}'. Supported: {:?}",
            operation,
            self.supported_operations()
        ));
    }

    // Execute with error handling
    match self.perform_operation(operation, data) {
        Ok(result) => Ok(result),
        Err(e) => Err(format!("Operation '{}' failed: {}", operation, e)),
    }
}
```

### 2. Resource Management

Extensions should clean up resources properly:

```rust
pub struct DatabaseExtension {
    connection: Option<DatabaseConnection>,
}

#[async_trait]
impl Extension for DatabaseExtension {
    async fn initialize(&self) -> Result<(), String> {
        // Acquire resources
        self.connection = Some(DatabaseConnection::new()?);
        Ok(())
    }

    async fn shutdown(&self) -> Result<(), String> {
        // Release resources
        if let Some(conn) = &self.connection {
            conn.close().await?;
        }
        Ok(())
    }
}
```

### 3. Thread Safety

Extensions must be `Send + Sync` for async execution:

```rust
use std::sync::{Arc, RwLock};

#[derive(Debug)]
pub struct StatefulExtension {
    state: Arc<RwLock<ExtensionState>>,
}

// Arc<RwLock<T>> is Send + Sync when T is Send + Sync
// This allows safe concurrent access from multiple tasks
```

### 4. Testing Extensions

Write comprehensive tests for extensions:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_extension_basic_operation() {
        let ext = TextProcessorExtension;

        // Test initialization
        assert!(ext.initialize().await.is_ok());

        // Test execution
        let mut params = HashMap::new();
        params.insert("data".to_string(), json!("hello"));
        params.insert("operation".to_string(), json!("uppercase"));

        let result = ext.execute(&params).await.unwrap();
        assert_eq!(result["processed_text"], "HELLO");
    }

    #[tokio::test]
    async fn test_extension_error_handling() {
        let ext = TextProcessorExtension;

        // Test missing parameter
        let params = HashMap::new();
        let result = ext.execute(&params).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_extension_capabilities() {
        let ext = TextProcessorExtension;
        let ops = ext.supported_operations();

        assert!(ops.contains(&"uppercase".to_string()));
        assert!(ops.contains(&"lowercase".to_string()));
    }
}
```

## Integration with Tools and Agents

Extensions integrate seamlessly with LLMSpell's Tool trait:

```rust
use llmspell_core::traits::tool::{Tool, ToolCategory, SecurityLevel, ToolSchema};

#[async_trait]
impl Tool for ExtensibleTool {
    fn category(&self) -> ToolCategory {
        ToolCategory::Utility
    }

    fn security_level(&self) -> SecurityLevel {
        SecurityLevel::Safe
    }

    fn schema(&self) -> ToolSchema {
        let available_extensions = self.registry.extension_ids();

        ToolSchema::new(
            self.metadata.name.clone(),
            format!(
                "Extensible tool with {} registered extensions",
                available_extensions.len()
            ),
        )
        .with_parameter(ParameterDef {
            name: "extension".to_string(),
            param_type: ParameterType::String,
            description: format!("Extension ID. Available: {:?}", available_extensions),
            required: true,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "operation".to_string(),
            param_type: ParameterType::String,
            description: "Operation to perform".to_string(),
            required: false,
            default: Some(json!("default")),
        })
    }
}
```

## Advanced Patterns

### 1. Dynamic Extension Loading

Load extensions from external modules (requires careful security consideration):

```rust
pub struct DynamicExtensionLoader {
    search_paths: Vec<PathBuf>,
}

impl DynamicExtensionLoader {
    pub async fn load_from_directory(
        &self,
        registry: &mut ExtensionRegistry,
    ) -> Result<usize, String> {
        let mut loaded = 0;

        for path in &self.search_paths {
            // Discover and load extension modules
            // This is a placeholder - actual implementation requires
            // dynamic library loading with libloading or similar
            loaded += self.scan_directory(path, registry).await?;
        }

        Ok(loaded)
    }
}
```

### 2. Extension Composition

Combine multiple extensions:

```rust
pub struct CompositeExtension {
    extensions: Vec<Arc<dyn Extension>>,
}

#[async_trait]
impl Extension for CompositeExtension {
    async fn execute(
        &self,
        params: &HashMap<String, serde_json::Value>,
    ) -> Result<serde_json::Value, String> {
        // Execute extensions in sequence, passing output to next
        let mut result = params.clone();

        for ext in &self.extensions {
            let output = ext.execute(&result).await?;
            result.insert("data".to_string(), output);
        }

        Ok(result.remove("data").unwrap())
    }
}
```

### 3. Extension Middleware

Add cross-cutting concerns to extensions:

```rust
pub struct LoggingExtensionWrapper<E: Extension> {
    inner: E,
}

#[async_trait]
impl<E: Extension> Extension for LoggingExtensionWrapper<E> {
    async fn execute(
        &self,
        params: &HashMap<String, serde_json::Value>,
    ) -> Result<serde_json::Value, String> {
        tracing::info!("Extension {} executing", self.inner.id());
        let start = Instant::now();

        let result = self.inner.execute(params).await;

        tracing::info!(
            "Extension {} completed in {:?}",
            self.inner.id(),
            start.elapsed()
        );

        result
    }
}
```

## Summary

The extension architecture pattern provides:

- **Modularity**: Add functionality without changing core code
- **Flexibility**: Support different operation types dynamically
- **Scalability**: Register unlimited extensions at runtime
- **Maintainability**: Separate concerns into focused extensions
- **Testability**: Test extensions independently

Use this pattern when building systems that need to support plugins, third-party integrations, or modular functionality that can be composed at runtime.

## See Also

- [Builder Pattern Documentation](llmspell-tools API docs)
- [Async Patterns Documentation](llmspell-core API docs)
- [Component Architecture](03-extending-components.md)
- [Tool Development](../user-guide/04-tools.md)
