# llmspell-tools

**Comprehensive tool system with 100+ built-in tools**

**🔗 Navigation**: [← Rust API](README.md) | [Crate Docs](https://docs.rs/llmspell-tools) | [Source](../../../../llmspell-tools)

---

## Overview

`llmspell-tools` provides a comprehensive tool system with over 100 built-in tools organized by category, security levels, and composition capabilities.

**Key Features:**
- 🔧 100+ built-in tools
- 🏷️ Category organization
- 🔐 Security levels (Safe, Moderate, Dangerous)
- 📝 Schema validation
- 🎯 Tool discovery
- 🔄 Tool composition
- ⚡ Async execution
- 📊 Usage metrics

## Tool Trait

```rust
#[async_trait]
pub trait Tool: BaseAgent {
    /// Tool category
    fn category(&self) -> ToolCategory;
    
    /// Security level
    fn security_level(&self) -> SecurityLevel;
    
    /// Tool schema
    fn schema(&self) -> ToolSchema;
    
    /// Execute tool
    async fn invoke(&self, params: Value) -> Result<Value>;
}
```

## Tool Categories

- **File System**: Read, write, list, search files
- **Web**: HTTP requests, web scraping, API testing
- **Data Processing**: JSON, CSV, text manipulation
- **Communication**: Email, webhooks, notifications
- **Media**: Image/audio/video processing
- **Utility**: Hashing, encoding, calculations
- **Development**: Code execution, testing
- **AI/ML**: Embeddings, classification

## Tool Registry

```rust
use llmspell_tools::ToolRegistry;

let registry = ToolRegistry::new();

// Register all built-in tools
registry.register_builtin_tools()?;

// Invoke a tool
let result = registry.invoke("web-searcher", json!({
    "query": "rust programming",
    "max_results": 10
})).await?;

// List tools by category
let file_tools = registry.list_by_category(ToolCategory::FileSystem);

// Search tools
let tools = registry.search("json")?;
```

## Example Tools

### File Operations
```rust
registry.invoke("file-operations", json!({"operation": "read", "path": "data.txt"})).await?;
registry.invoke("file-operations", json!({"operation": "write", "path": "out.txt", "input": "..."})).await?;
registry.invoke("file-operations", json!({"operation": "list", "path": "./src"})).await?;
```

### Web Tools
```rust
registry.invoke("http-requester", json!({"method": "GET", "url": "https://api.example.com"})).await?;
registry.invoke("web-searcher", json!({"query": "LLMSpell", "engine": "duckduckgo"})).await?;
registry.invoke("http-requester", json!({"method": "POST", "url": "...", "body": {}})).await?;
```

### Data Processing
```rust
registry.invoke("json-processor", json!({"operation": "query", "data": data, "query": "$.users[?(@.age > 18)]"})).await?;
registry.invoke("csv-analyzer", json!({"operation": "parse", "input": csv_string})).await?;
registry.invoke("text-manipulator", json!({"operation": "uppercase", "input": "hello world"})).await?;
```

## Creating Custom Tools

```rust
pub struct MyCustomTool;

#[async_trait]
impl Tool for MyCustomTool {
    fn category(&self) -> ToolCategory {
        ToolCategory::Custom
    }
    
    fn security_level(&self) -> SecurityLevel {
        SecurityLevel::Safe
    }
    
    fn schema(&self) -> ToolSchema {
        ToolSchema::new("my-custom-tool", "Description")
            .with_parameter("input", ParameterType::String, true)
            .with_returns(ParameterType::String)
    }
    
    async fn invoke(&self, params: Value) -> Result<Value> {
        let input = params["input"].as_str().ok_or("Invalid input")?;
        // Tool logic here
        Ok(json!({"result": "processed"}))
    }
}
```

## Related Documentation

- [llmspell-agents](llmspell-agents.md) - Agents using tools
- [llmspell-security](llmspell-security.md) - Tool security levels