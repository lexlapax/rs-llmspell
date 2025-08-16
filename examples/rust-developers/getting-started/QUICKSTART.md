# Quick Start Guide - Rust Developers

Embed LLMSpell in your Rust application in 5 minutes!

## Prerequisites

Add LLMSpell to your `Cargo.toml`:

```toml
[dependencies]
llmspell-core = "0.7"
llmspell-bridge = "0.7"
llmspell-tools = "0.7"
llmspell-agents = "0.7"
tokio = { version = "1", features = ["full"] }
anyhow = "1.0"
```

## Minimal Example

```rust
use anyhow::Result;
use llmspell_bridge::{lua::LuaEngine, ScriptEngine};
use llmspell_core::ToolRegistry;
use llmspell_tools::register_all_tools;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Set up tools
    let registry = ToolRegistry::new();
    register_all_tools(&registry).await?;
    
    // 2. Create script engine
    let mut engine = LuaEngine::new()?;
    engine.initialize().await?;
    
    // 3. Execute scripts
    let result = engine.execute(r#"
        local uuid = Tool.invoke("uuid_generator", {
            operation = "generate",
            version = "v4"
        })
        return uuid.text
    "#).await?;
    
    println!("Result: {:?}", result);
    Ok(())
}
```

## Learning Path

Follow these examples in order:

1. **01-embed-llmspell** - Basic embedding and script execution
2. **02-custom-tool** - Create custom tools
3. **03-custom-agent** - Build custom agents
4. **04-testing-components** - Test your components
5. **05-async-patterns** - Advanced async patterns

## Common Patterns

### Custom Tool

```rust
use async_trait::async_trait;
use llmspell_core::{Tool, ToolInput, ToolOutput};

#[derive(Debug, Clone)]
struct MyTool;

#[async_trait]
impl Tool for MyTool {
    fn name(&self) -> &str {
        "my_tool"
    }
    
    fn description(&self) -> &str {
        "My custom tool"
    }
    
    fn parameters(&self) -> Vec<(&str, &str)> {
        vec![("input", "Input data")]
    }
    
    async fn invoke(&self, input: ToolInput) -> Result<ToolOutput> {
        // Your logic here
        Ok(ToolOutput::from_json(json!({
            "result": "processed",
            "success": true
        })))
    }
}
```

### Custom Agent

```rust
use llmspell_agents::{BaseAgent, AgentCapabilities};
use llmspell_core::ComponentId;

struct MyAgent {
    id: ComponentId,
    name: String,
}

#[async_trait]
impl BaseAgent for MyAgent {
    fn id(&self) -> &ComponentId {
        &self.id
    }
    
    fn name(&self) -> &str {
        &self.name
    }
    
    fn capabilities(&self) -> AgentCapabilities {
        AgentCapabilities {
            can_use_tools: true,
            supports_streaming: false,
            max_context_length: Some(1000),
            supports_function_calling: false,
        }
    }
    
    async fn invoke(&self, input: ToolInput) -> Result<ToolOutput> {
        // Your agent logic
        Ok(ToolOutput::from_json(json!({
            "text": "Response",
            "success": true
        })))
    }
}
```

### Async Patterns

```rust
use tokio::join;

// Concurrent execution
let (r1, r2, r3) = join!(
    tool1.invoke(input1),
    tool2.invoke(input2),
    tool3.invoke(input3)
);

// Timeout
use tokio::time::{timeout, Duration};

let result = timeout(
    Duration::from_secs(5),
    slow_operation()
).await;
```

### Testing

```rust
use llmspell_testing::{
    tool_helpers::create_test_tool,
    agent_helpers::AgentTestBuilder,
};

#[tokio::test]
async fn test_my_component() {
    let tool = create_test_tool("test", "desc", vec![]);
    let result = tool.invoke(input).await.unwrap();
    assert!(result.to_json().get("success").is_some());
}
```

## Best Practices

1. **Error Handling**: Always use `Result<T>` and handle errors gracefully
2. **Resource Management**: Use `Arc` for shared ownership
3. **Async/Await**: Leverage tokio for concurrent operations
4. **Testing**: Use llmspell-testing helpers for unit tests
5. **Logging**: Use tracing for debugging

## Integration Tips

### With Existing Applications

```rust
// Add to your existing tokio runtime
let llmspell_handle = tokio::spawn(async move {
    let mut engine = LuaEngine::new()?;
    engine.initialize().await?;
    // Your LLMSpell logic
});
```

### With Web Frameworks

```rust
// Example with Axum
use axum::{Router, Json};

async fn llmspell_endpoint(
    Json(input): Json<Value>
) -> Json<Value> {
    let engine = get_engine(); // Your engine instance
    let result = engine.execute(&input["script"]).await?;
    Json(json!({ "result": result }))
}
```

## Next Steps

- Explore the [examples](../)
- Read the [Developer Guide](../../../docs/developer-guide/)
- Check the [API docs](https://docs.rs/llmspell)
- Contribute on [GitHub](https://github.com/yourusername/llmspell)

## Troubleshooting

### Compilation Errors
Ensure all dependencies are at compatible versions.

### Runtime Panics
Enable debug logging:
```rust
tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .init();
```

### Performance Issues
- Use concurrent patterns
- Profile with `cargo flamegraph`
- Check the [performance guide](../../../docs/developer-guide/performance.md)