# Version 0.3.0 Breaking Changes

**Release Date**: TBD  
**Phase**: 3.0 - Tool Enhancement & Workflow Orchestration  
**Breaking Change Policy**: Pre-1.0 Clean Break (No Migration Tools)

## Overview

Version 0.3.0 introduces comprehensive standardization across all 26 tools to achieve consistency in parameter naming, response formats, and security hardening. As a pre-1.0 project, we're making clean breaking changes to establish the best possible architecture before our 1.0 stability commitment.

## Major Changes

### 1. Universal Parameter Standardization

All tools now use consistent parameter naming:
- **Primary data parameter**: `input` (was: text, content, data, expression, query, template, etc.)
- **File path parameters**: `path: PathBuf` for single files, `source_path`/`target_path` for transforms
- **Operation parameter**: Required for all multi-function tools

### 2. ResponseBuilder Pattern

All tools now return standardized responses:
```json
{
  "operation": "operation_name",
  "success": true,
  "result": {...},
  "error": null,
  "metadata": {...}
}
```

### 3. Security Hardening

- Calculator DoS protection with complexity limits
- Path traversal prevention in all file tools
- Symlink attack prevention
- Resource limit enforcement

### 4. Agent-Provider Integration (Phase 3.3)

Major changes to agent and provider architecture:

#### Provider Configuration
- Added `provider_type` field to ProviderConfig for clean separation
- Provider naming now follows hierarchical scheme (e.g., `rig/openai/gpt-4`)
- Bridge layer correctly preserves provider type information

#### Agent Factory Changes
- **BREAKING**: DefaultAgentFactory no longer implements `Default` trait - requires ProviderManager
- **BREAKING**: Default agent type is now "llm" not "basic"
- **BREAKING**: Factory returns error for unknown agent types instead of defaulting to basic agent
- Agent creation requires provider manager injection

#### LLM Agent Implementation
- New LLM agent type that uses actual language model providers
- Supports conversation management and system prompts
- Parses "provider/model" syntax from Lua (e.g., "openai/gpt-4")
- Basic agent relegated to test-only usage

#### Lua API Changes
```lua
-- OLD: Basic echo agent by default
local agent = Agent.create("my-agent", {})

-- NEW: LLM agent by default, requires model specification
local agent = Agent.create("my-agent", {
    model = "openai/gpt-4",  -- or separate provider/model fields
    temperature = 0.7,
    max_tokens = 1000
})
```

## Tool-by-Tool Breaking Changes

### File Operations Tools

**Detailed Migration Guide**: See [phase-3-file-tools-migration.md](docs/in-progress/phase-3-file-tools-migration.md)

#### FileOperationsTool
```rust
// OLD - Write operation
{"operation": "write", "path": "/tmp/file.txt", "content": "data"}
// OLD - Copy operation
{"operation": "copy", "from_path": "/src.txt", "to_path": "/dst.txt"}

// NEW - Write operation
{"operation": "write", "path": "/tmp/file.txt", "input": "data"}
// NEW - Copy operation  
{"operation": "copy", "source_path": "/src.txt", "target_path": "/dst.txt"}
```
- `content` → `input` for write/append operations
- `from_path`/`to_path` → `source_path`/`target_path` for copy/move operations
- All responses use ResponseBuilder pattern

#### ArchiveHandlerTool
```rust
// OLD
{"operation": "create", "archive_path": "/tmp/archive.zip", "files": ["file1.txt"]}
{"operation": "extract", "archive_path": "/tmp/archive.zip", "output_dir": "/tmp/out"}

// NEW
{"operation": "create", "path": "/tmp/archive.zip", "input": ["file1.txt"]}
{"operation": "extract", "path": "/tmp/archive.zip", "target_path": "/tmp/out"}
```
- `archive_path` → `path`
- `files` → `input` for create operation
- `output_dir` → `target_path` for extract operation

#### FileWatcherTool
```rust
// OLD
{"operation": "watch", "paths": ["/tmp/dir1", "/tmp/dir2"]}

// NEW
{"operation": "watch", "input": ["/tmp/dir1", "/tmp/dir2"]}
```
- `paths` → `input`

#### FileConverterTool
```rust
// OLD
{"operation": "encoding", "input_path": "/doc.txt", "output_path": "/doc-utf8.txt"}

// NEW
{"operation": "encoding", "path": "/doc.txt", "target_path": "/doc-utf8.txt"}
```
- `input_path` → `path`
- `output_path` → `target_path`

#### FileSearchTool
- No parameter changes (already used standard `path` parameter)
- Updated to use ResponseBuilder pattern

### Utility Tools

#### CalculatorTool
```rust
// OLD
{"expression": "2 + 2", "precision": 2}

// NEW
{"operation": "calculate", "input": "2 + 2", "precision": 2}
```
- Added required `operation` parameter
- `expression` → `input`
- Added DoS protection (max 1000 chars, 100ms timeout)

#### TextManipulatorTool
```rust
// OLD
{"text": "hello", "operation": "uppercase"}

// NEW
{"operation": "uppercase", "input": "hello"}
```
- `text` → `input`
- Parameter order changed (operation first)

#### HashCalculatorTool
```rust
// OLD
{"algorithm": "sha256", "data": "hello", "file_path": "/tmp/file"}

// NEW
{"operation": "hash", "input": "hello", "path": "/tmp/file", "algorithm": "sha256"}
```
- Added required `operation` parameter
- `data` → `input`
- `file_path` → `path`

### Data Processing Tools

#### TemplateEngineTool
```rust
// OLD
{"template": "Hello {{name}}", "context": {"name": "World"}, "engine": "tera"}

// NEW
{"operation": "render", "input": "Hello {{name}}", "context": {"name": "World"}, "engine": "tera"}
```
- Added required `operation` parameter
- `template` → `input`

#### DataValidationTool
```rust
// OLD
{"data": {"age": 25}, "schema": {...}, "format": "json"}

// NEW
{"operation": "validate", "input": {"age": 25}, "schema": {...}, "format": "json"}
```
- Added required `operation` parameter
- `data` → `input`

### API/Web Tools

#### WebSearchTool
```rust
// OLD
{"query": "rust programming", "max_results": 10}

// NEW
{"operation": "search", "input": "rust programming", "max_results": 10}
```
- Added required `operation` parameter
- `query` → `input`

#### HttpRequestTool
```rust
// OLD
{"method": "GET", "url": "https://api.example.com", "headers": {...}}

// NEW
{"operation": "request", "input": "https://api.example.com", "method": "GET", "headers": {...}}
```
- Added required `operation` parameter
- `url` → `input`

#### GraphQLQueryTool
```rust
// OLD
{"query": "{ users { id name } }", "endpoint": "https://api.example.com/graphql"}

// NEW
{"operation": "query", "input": "{ users { id name } }", "endpoint": "https://api.example.com/graphql"}
```
- Added required `operation` parameter
- `query` → `input`

### Data Processing Tools (Additional)

#### JsonProcessorTool
```rust
// OLD - Stream operation
{"operation": "stream", "content": "[{\"id\":1}]"}

// NEW - Stream operation
{"operation": "stream", "input": "[{\"id\":1}]"}
```
- `content` → `input` for stream operation
- Query and validate operations already used appropriate parameters

#### CsvAnalyzerTool
```rust
// OLD
{"operation": "analyze", "content": "id,name\n1,Alice"}

// NEW
{"operation": "analyze", "input": "id,name\n1,Alice"}
```
- `content` → `input` for all operations

### Media Processing Tools

#### ImageProcessorTool
```rust
// OLD
{"operation": "resize", "input_path": "/image.jpg", "output_path": "/resized.jpg"}

// NEW
{"operation": "resize", "source_path": "/image.jpg", "target_path": "/resized.jpg"}
```
- `input_path` → `source_path`
- `output_path` → `target_path`

#### AudioProcessorTool
```rust
// OLD
{"operation": "convert", "input_path": "/audio.wav", "output_path": "/audio.mp3"}

// NEW
{"operation": "convert", "source_path": "/audio.wav", "target_path": "/audio.mp3"}
```
- `input_path` → `source_path`
- `output_path` → `target_path`

#### VideoProcessorTool
```rust
// OLD
{"operation": "extract_frame", "path": "/video.mp4", "output_path": "/frame.jpg"}

// NEW
{"operation": "extract_frame", "path": "/video.mp4", "target_path": "/frame.jpg"}
```
- `output_path` → `target_path`
- `path` parameter unchanged

### System Integration Tools

#### EnvironmentReaderTool
- No parameter changes (already uses domain-appropriate `operation`, `variable_name`, `pattern`)
- Updated to use ResponseBuilder pattern

#### ProcessExecutorTool
- No parameter changes (already uses domain-appropriate `executable`, `arguments`, `timeout_ms`)
- Updated to use ResponseBuilder pattern

#### ServiceCheckerTool
- No parameter changes (already uses domain-appropriate `check_type`, `target`, `timeout_ms`)
- Updated to use ResponseBuilder pattern

#### SystemMonitorTool
- No parameter changes (already uses domain-appropriate `operation`)
- Updated to use ResponseBuilder pattern

### Additional Utility Tools

#### UuidGeneratorTool
- No primary data parameter (uses operation-specific parameters)
- Already had `operation` parameter
- Updated to use ResponseBuilder pattern

#### DiffCalculatorTool
- No parameter changes (uses domain-appropriate `old_text`, `new_text` for clarity)
- Updated to use ResponseBuilder pattern

#### Base64EncoderTool
- Already uses `input` parameter
- Minor adjustments for ResponseBuilder pattern

#### DateTimeHandlerTool
- Already uses `input` parameter where applicable
- Updated to use ResponseBuilder pattern

## Response Format Changes

### Old Formats (Varied by Tool)
```json
// FileOperationsTool
{"result": "file content"}

// CalculatorTool
{"result": 42.0, "expression": "40 + 2"}

// Error responses
{"error": "File not found"}
```

### New Standardized Format
```json
// Success
{
  "operation": "read",
  "success": true,
  "result": {
    "content": "file content",
    "size": 1024
  },
  "error": null
}

// Error
{
  "operation": "read",
  "success": false,
  "result": null,
  "error": {
    "code": "FILE_NOT_FOUND",
    "message": "File not found: /tmp/missing.txt",
    "details": {...}
  }
}
```

## Migration Guide

### Step 1: Update Parameter Names
1. Replace primary data parameters with `input`
2. Add `operation` parameter to tools that lack it
3. Update path parameters to use PathBuf types

### Step 2: Update Response Parsing
1. Access results via `response.result` instead of top-level
2. Check `response.success` for operation status
3. Error details now in `response.error` object

### Step 3: Update Error Handling
```rust
// Old
match tool.execute(params) {
    Ok(output) => {
        let result = output["result"].as_str()?;
    }
    Err(e) => println!("Error: {}", e),
}

// New
match tool.execute(params) {
    Ok(output) => {
        let response: StandardResponse = serde_json::from_str(&output)?;
        if response.success {
            let result = response.result;
        } else {
            let error = response.error.unwrap();
        }
    }
    Err(e) => println!("Tool execution error: {}", e),
}
```

## Validation Improvements

### Path Validation
All file operations now include:
- Path traversal prevention
- Symlink resolution blocking
- Sandbox enforcement
- Permission checking

### Input Validation
- Length limits enforced
- Format validation
- Injection prevention
- Resource consumption limits

## Performance Impact

Despite added security measures:
- Tool initialization: <10ms (maintained)
- Operation overhead: <1ms added for validation
- Memory usage: Negligible increase
- 52,600x performance advantage maintained

## Example Script Conversions

### Lua Script Migration Example

**Old Script (v0.2.x)**
```lua
-- Calculate expression
local calc = Tool.get("calculator")
local result = calc.execute({expression = "2 + 2"})
print("Result:", result.result)

-- Hash data
local hash = Tool.get("hash_calculator")
local hash_result = hash.execute({data = "hello world", algorithm = "sha256"})
print("Hash:", hash_result.hash)

-- Template rendering
local template = Tool.get("template_engine")
local output = template.execute({
    template = "Hello {{name}}",
    context = {name = "World"},
    engine = "handlebars"
})
print("Rendered:", output.result)
```

**New Script (v0.3.0)**
```lua
-- Calculate expression
local result = Tool.executeAsync("calculator", {
    operation = "evaluate",
    input = "2 + 2"
})
local response = JSON.parse(result.output)
print("Result:", response.result.result)

-- Hash data
local hash_result = Tool.executeAsync("hash_calculator", {
    operation = "hash",
    input = "hello world",
    algorithm = "sha256"
})
local hash_response = JSON.parse(hash_result.output)
print("Hash:", hash_response.result.hash)

-- Template rendering
local output = Tool.executeAsync("template_engine", {
    input = "Hello {{name}}",
    context = {name = "World"},
    engine = "handlebars"
})
local template_response = JSON.parse(output.output)
print("Rendered:", template_response.result.output)
```

### JavaScript/Python Script Updates

The same pattern applies:
1. Use `Tool.executeAsync` instead of direct execution
2. Add `operation` parameter where required
3. Replace old parameter names with standardized ones
4. Parse JSON response and access via `result` field

## Troubleshooting Guide

### Common Migration Issues

#### 1. "Missing required parameter 'operation'"
**Problem**: Tool now requires an explicit operation parameter  
**Solution**: Add the appropriate operation (e.g., `"evaluate"`, `"hash"`, `"render"`)

#### 2. "Unknown parameter 'data'"
**Problem**: Parameter has been renamed to `input`  
**Solution**: Replace `data`, `text`, `expression`, etc. with `input`

#### 3. "Invalid parameter 'from_path'"
**Problem**: Path parameters have been standardized  
**Solution**: Use `source_path` and `target_path` for transform operations

#### 4. Response parsing errors
**Problem**: Direct field access no longer works  
**Solution**: Parse JSON response and access fields via `response.result`

#### 5. Tool.get() returns nil
**Problem**: Direct tool access pattern has changed  
**Solution**: Use `Tool.executeAsync(tool_name, params)` instead

### Validation Steps

1. **Parameter Validation**
   ```bash
   # List all tools with their parameters
   cargo run --bin llmspell -- tool list --verbose
   ```

2. **Test Individual Tools**
   ```bash
   # Test a specific tool with new parameters
   cargo run --bin llmspell -- tool test calculator --params '{"operation":"evaluate","input":"2+2"}'
   ```

3. **Run Example Scripts**
   ```bash
   # Run all updated examples
   cd examples && ./run-all-tools-examples.sh
   ```

### Migration Checklist

- [ ] Update all parameter names according to the mapping table
- [ ] Add `operation` parameter to tools that now require it
- [ ] Update response parsing to use new structure
- [ ] Replace `Tool.get()` with `Tool.executeAsync()`
- [ ] Test each tool individually
- [ ] Run full integration tests

## Parameter Mapping Table

### Complete Parameter Changes by Tool

| Tool | Old Parameter | New Parameter | Category |
|------|---------------|---------------|----------|
| **CalculatorTool** | `expression` | `input` | Utility |
| **TextManipulatorTool** | `text` | `input` | Utility |
| **HashCalculatorTool** | `data` | `input` | Utility |
| | `file_path` | `path` | Utility |
| **TemplateEngineTool** | `template` | `input` | Utility |
| **DataValidationTool** | `data` | `input` | Utility |
| **FileOperationsTool** | `content` | `input` | File Ops |
| | `from_path` | `source_path` | File Ops |
| | `to_path` | `target_path` | File Ops |
| **ArchiveHandlerTool** | `archive_path` | `path` | File Ops |
| | `files` | `input` | File Ops |
| | `output_dir` | `target_path` | File Ops |
| **FileWatcherTool** | `paths` | `input` | File Ops |
| **FileConverterTool** | `input_path` | `path` | File Ops |
| | `output_path` | `target_path` | File Ops |
| **JsonProcessorTool** | `content` | `input` | Data Processing |
| **CsvAnalyzerTool** | `content` | `input` | Data Processing |
| **ImageProcessorTool** | `input_path` | `source_path` | Media |
| | `output_path` | `target_path` | Media |
| **AudioProcessorTool** | `input_path` | `source_path` | Media |
| | `output_path` | `target_path` | Media |
| **VideoProcessorTool** | `output_path` | `target_path` | Media |
| **HttpRequestTool** | `url` | `input` | API/Web |
| **GraphQLQueryTool** | `query` | `input` | API/Web |
| **WebSearchTool** | `query` | `input` | API/Web |

### Tools Requiring Operation Parameter

The following tools now require an explicit `operation` parameter:

| Tool | Required Operations |
|------|-------------------|
| **CalculatorTool** | `evaluate` |
| **HashCalculatorTool** | `hash`, `verify`, `hash_file` |
| **TemplateEngineTool** | `render` |
| **DataValidationTool** | `validate` |
| **WebSearchTool** | `search` |
| **HttpRequestTool** | `request` |
| **GraphQLQueryTool** | `query` |

## Tools Requiring No Changes

The following tools already comply with standards:
- Base64EncoderTool (minor adjustments only)
- DateTimeHandlerTool (already compliant)
- System integration tools (use domain-appropriate parameters)

## Support

For migration assistance:
- Review updated examples in `/examples/`
- Check tool-specific documentation in `/docs/tools/`
- Consult the comprehensive migration guide: `/docs/in-progress/phase-03-tools-migration.md`

## Future Compatibility

Version 0.3.0 establishes the parameter and response standards that will be maintained through 1.0 and beyond. No further breaking changes to these interfaces are planned.