# Tool Integration Architecture

**Version**: Phase 3.3 Implementation  
**Status**: ✅ **CURRENT** - 34 production tools with async bridge architecture  
**Last Updated**: July 2025

> **🔧 INTEGRATION PATTERNS**: Technical documentation on how tools integrate with the script bridge layer, including async execution, security patterns, and performance optimizations.

## Overview

The tool integration in rs-llmspell follows a multi-layer architecture:

```
┌─────────────────┐
│   Lua Script    │
├─────────────────┤
│   Tool API      │  ← Tool.list(), Tool.get(), tool:execute()
├─────────────────┤
│  Bridge Layer   │  ← LuaEngine, async function wrapping
├─────────────────┤
│ ComponentRegistry│  ← Tool discovery and management
├─────────────────┤
│   Tool Impls    │  ← BaseAgent trait implementations
└─────────────────┘
```

## Bridge Layer Integration

### Tool API Injection

The Tool API is injected into the Lua environment during engine initialization:

```rust
// llmspell-bridge/src/lua/api/tool.rs
pub fn inject_tool_api(
    lua: &Lua,
    api_def: &ToolApiDefinition,
    registry: Arc<ComponentRegistry>,
) -> Result<(), LLMSpellError> {
    let tool_table = lua.create_table()?;
    
    // Tool.list() - returns array of tool names
    tool_table.set("list", list_fn)?;
    
    // Tool.get(name) - returns tool instance
    tool_table.set("get", get_fn)?;
    
    lua.globals().set("Tool", tool_table)?;
    Ok(())
}
```

### Async Execution Handling

Tools execute asynchronously through `create_async_function`:

```rust
// Tool execute method
tool_table.set(
    "execute",
    lua.create_async_function(
        move |lua, (_self_table, args): (mlua::Table, mlua::Table)| {
            let tool_instance = tool_arc_for_execute.clone();
            async move {
                // Convert Lua table to AgentInput
                let input = lua_table_to_agent_input(&args)?;
                
                // Execute tool (async)
                let output = tool_instance
                    .execute(input, ExecutionContext::default())
                    .await?;
                
                // Return as Lua table
                Ok(lua_output_table(lua, &output)?)
            }
        },
    )?,
)?;
```

### Lua Method Syntax Handling

The `:` operator in Lua automatically passes `self` as the first argument:

```lua
-- This Lua code:
tool:execute({operation = "hash"})

-- Is equivalent to:
tool.execute(tool, {operation = "hash"})
```

The bridge handles this by accepting a tuple in the function signature:

```rust
move |lua, (_self_table, args): (mlua::Table, mlua::Table)| {
    // _self_table is the tool table (ignored)
    // args is the parameter table
}
```

## Tool Output Format

### Standard Response Format

Most tools return responses using the ResponseBuilder pattern:

```rust
let response = ResponseBuilder::success(operation)
    .with_result(json!({
        "hash": hash_value,
        "algorithm": "sha256"
    }))
    .build();

Ok(AgentOutput::text(serde_json::to_string(&response)?))
```

This produces:
```json
{
    "success": true,
    "operation": "hash",
    "result": {
        "hash": "abc123...",
        "algorithm": "sha256"
    }
}
```

### Direct Response Format

Some tools (like file_operations) return data directly:

```rust
Ok(AgentOutput::text(serde_json::to_string(&json!({
    "content": file_content,
    "operation": "read",
    "path": path,
    "size": content.len()
}))?))
```

## JSON API Integration

The JSON API enables parsing of tool outputs:

```rust
// llmspell-bridge/src/lua/api/json.rs
pub fn inject_json_api(lua: &Lua, api_def: &JsonApiDefinition) -> Result<(), LLMSpellError> {
    let json_table = lua.create_table()?;
    
    // JSON.parse(string) -> Lua value
    json_table.set("parse", parse_fn)?;
    
    // JSON.stringify(value) -> string
    json_table.set("stringify", stringify_fn)?;
    
    lua.globals().set("JSON", json_table)?;
    Ok(())
}
```

## Component Registry

Tools are discovered and managed through the ComponentRegistry:

```rust
// Tool registration during initialization
pub fn register_builtin_tools(registry: &mut ComponentRegistry) -> Result<()> {
    // Data Processing Tools
    registry.register_tool("json_processor", 
        Arc::new(JsonProcessorTool::new(Default::default())));
    
    // File System Tools (with sandbox)
    let sandbox = create_file_sandbox()?;
    registry.register_tool("file_operations",
        Arc::new(FileOperationsTool::new(Default::default(), sandbox.clone())));
    
    // ... register all 26 Phase 2 tools
}
```

## Error Handling Patterns

### Parameter Validation Errors

Tools use `extract_required_string` which throws Lua errors:

```rust
let operation = extract_required_string(params, "operation")?;
// If missing, throws: "Missing required parameter: operation"
```

In Lua, this becomes a catchable error:
```lua
local success, err = pcall(function()
    return tool:execute({}) -- Missing operation
end)
-- success = false
-- err = "Missing required parameter: operation"
```

### Operational Errors

Tools return error responses for operational failures:

```rust
match operation {
    "hash" => { /* ... */ }
    _ => Err(validation_error(
        format!("Invalid operation: {}", operation),
        Some("operation".to_string()),
    ))
}
```

## Performance Characteristics

### Tool Initialization

All tools initialize in <10ms due to:
- Lazy loading of heavy dependencies
- Minimal allocation during construction
- Configuration structs using Default trait

### Async Execution Benefits

The async bridge provides:
- Non-blocking I/O operations
- Concurrent request handling
- Efficient resource utilization
- No thread-per-request overhead

### Benchmarking Results

Current performance characteristics (Phase 3.3):
```
Average tool initialization: 0.8ms (target: <10ms) ✅
Simple operation (hash): 0.5ms 
File operation (read 1KB): 2.1ms
Complex operation (JSON query): 1.3ms
Total tool count: 34 production tools
```

## Security Integration

### Sandboxing

File system tools integrate with FileSandbox:

```rust
impl FileOperationsTool {
    pub fn new(config: Config, sandbox: Arc<FileSandbox>) -> Self {
        Self {
            sandbox,
            // Sandbox validates all paths
        }
    }
}
```

### Security Levels

Tools declare their security requirements:

```rust
fn security_requirements(&self) -> SecurityRequirements {
    SecurityRequirements {
        level: SecurityLevel::Restricted,
        file_access: vec!["/tmp".to_string()],
        network_access: false,
        process_execution: false,
    }
}
```

## Testing Integration

### Integration Test Pattern

```rust
#[tokio::test]
async fn test_tool_from_lua() {
    let runtime = ScriptRuntime::new_with_lua(RuntimeConfig::default())
        .await
        .unwrap();
    
    let script = r#"
        local tool = Tool.get("hash_calculator")
        local result = tool:execute({
            operation = "hash",
            data = "test"
        })
        return JSON.parse(result.output)
    "#;
    
    let output = runtime.execute_script(script).await.unwrap();
    assert_eq!(output.output["success"].as_bool(), Some(true));
}
```

## Provider Enhancement Integration

The bridge supports provider/model syntax:

```lua
local agent = Agent.create({
    model = "openai/gpt-4",  -- Provider/model syntax
    base_url = "https://custom.openai.com/v1"  -- Optional override
})
```

This is parsed by ModelSpecifier:
```rust
// "openai/gpt-4" -> ModelSpecifier {
//     provider: Some("openai"),
//     model: "gpt-4",
//     base_url: None
// }
```

## Future Enhancements

### Streaming Support

Tools could support streaming responses:
```rust
tool:executeStream({operation = "large_file_read"}, function(chunk)
    print("Received chunk: " .. chunk)
end)
```

### Batch Operations

Native batch support for efficiency:
```rust
Tool.batch({
    {tool = "hash_calculator", params = {data = "test1"}},
    {tool = "hash_calculator", params = {data = "test2"}}
})
```

### Tool Composition

Tools as building blocks for agents:
```rust
local hashAgent = Agent.fromTool("hash_calculator", {
    defaultAlgorithm = "sha256"
})
```

## See Also

- [Tool Implementation Guide](/docs/technical/tool-implementation.md)
- [Bridge Architecture](/docs/technical/master-architecture-vision.md)
- [Security Model](/docs/technical/security-model.md)