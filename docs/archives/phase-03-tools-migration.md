# Phase 3: Complete Tools Migration Guide

**Version**: 0.2.0 → 0.3.0  
**Date**: 2025-07-11  
**Impact**: Breaking Changes (Parameter Standardization & Response Format)  
**Scope**: All 26 tools across 6 categories

## Executive Summary

Phase 3.0 represents a comprehensive standardization effort affecting all 26 tools in the rs-llmspell project. This migration introduces consistent parameter naming conventions and a unified response format using the ResponseBuilder pattern. As a pre-1.0 project, we've adopted a clean break approach - no automated migration tools are provided, only comprehensive documentation.

## Migration Overview

### What Changed
1. **Parameter Standardization**: Primary data parameters unified to `input`
2. **Path Parameter Conventions**: Consistent `path`, `source_path`, `target_path` naming
3. **Response Format**: All tools now use ResponseBuilder for consistent JSON responses
4. **DRY Compliance**: Shared utilities extracted to llmspell-utils crate

### What Stayed the Same
- Core functionality of all tools
- Security levels and resource limits
- Rate limiting configurations
- Domain-specific parameters where appropriate

## Parameter Standardization Rules

### Primary Data Parameter: `input`
Most tools now use `input` as their primary data parameter:

| Tool Category | Old Parameter | New Parameter | Examples |
|---------------|---------------|---------------|----------|
| Calculations | `expression` | `input` | CalculatorTool |
| Text Processing | `text` | `input` | TextManipulatorTool |
| Data Processing | `content` | `input` | JsonProcessorTool, CsvAnalyzerTool |
| Hash Operations | `data` | `input` | HashCalculatorTool |
| Templates | `template` | `input` | TemplateEngineTool |
| Validation | `data` | `input` | DataValidationTool |
| Web Requests | `url` | `input` | HttpRequestTool |
| Web Search | `query` | `input` | WebSearchTool |
| GraphQL | `query` | `input` | GraphQLQueryTool |

### Path Parameter Conventions

#### Single File Operations
Use `path` for operations on a single file:
- `archive_path` → `path` (ArchiveHandlerTool)
- File read/write operations maintain `path`

#### Source-to-Target Operations
Use `source_path` and `target_path` for transformations:
- `from_path`/`to_path` → `source_path`/`target_path` (FileOperationsTool copy/move)
- `input_path`/`output_path` → `source_path`/`target_path` (Media tools)

#### Domain-Specific Parameters Preserved
Some tools retain domain-appropriate names:
- `video_path` for video source in VideoProcessorTool
- `old_text`/`new_text` for diff operations
- Environment variables keep `variable_name`
- System monitoring keeps operation-specific parameters

## Response Format Standardization

### Universal ResponseBuilder Pattern

All tools now return consistent JSON responses:

```json
{
    "operation": "operation_name",
    "success": true,
    "message": "Human-readable status message",
    "result": {
        // Tool-specific result data
    },
    "metadata": {
        // Optional additional metadata
    }
}
```

### Response Parsing Migration

**Old (varied by tool)**:
```rust
// Different tools had different formats
let data = output.metadata.extra["data"];
let result = serde_json::from_str(&output.text)?;
```

**New (consistent across all tools)**:
```rust
let output = tool.execute(input, context).await?;
let response: Value = serde_json::from_str(&output.text)?;
let success = response["success"].as_bool().unwrap_or(false);
let message = response["message"].as_str().unwrap_or("");
let result = &response["result"];
```

## Category-Specific Migration Details

### File Operation Tools (5 tools)
**Affected**: FileOperationsTool, ArchiveHandlerTool, FileWatcherTool, FileConverterTool, FileSearchTool

**Key Changes**:
- `content` → `input` for write operations
- `from_path`/`to_path` → `source_path`/`target_path` for copy/move
- `archive_path` → `path` for archive operations
- `files` → `input` for archive creation
- `paths` → `input` for file watching
- `input_path`/`output_path` → `path`/`target_path` for conversion

**Migration Example**:
```rust
// Old
let params = json!({
    "operation": "copy",
    "from_path": "/tmp/source.txt",
    "to_path": "/tmp/dest.txt"
});

// New
let params = json!({
    "operation": "copy",
    "source_path": "/tmp/source.txt",
    "target_path": "/tmp/dest.txt"
});
```

### Data Processing Tools (2 tools)
**Affected**: JsonProcessorTool, CsvAnalyzerTool

**Key Changes**:
- `content` → `input` for all operations
- Response maintains actual data in `text` field for backward compatibility
- ResponseBuilder metadata added to `metadata.extra`

**Migration Example**:
```rust
// Old
let params = json!({
    "operation": "query",
    "content": json_data,
    "query": ".users"
});

// New
let params = json!({
    "operation": "query",
    "input": json_data,
    "query": ".users"
});
```

### Utility Tools (9 tools)
**Affected**: CalculatorTool, TextManipulatorTool, HashCalculatorTool, TemplateEngineTool, DataValidationTool, DateTimeHandlerTool, UuidGeneratorTool, Base64EncoderTool, DiffCalculatorTool

**Key Changes**:
- Various parameters → `input` (expression, text, data, template)
- DateTimeHandlerTool, UuidGeneratorTool, Base64EncoderTool, DiffCalculatorTool: No parameter changes
- All tools now use ResponseBuilder format

**Migration Example**:
```rust
// Old
let params = json!({
    "operation": "hash",
    "algorithm": "sha256",
    "data": "hello world"
});

// New
let params = json!({
    "operation": "hash",
    "algorithm": "sha256",
    "input": "hello world"
});
```

### System Integration Tools (4 tools)
**Affected**: EnvironmentReaderTool, ProcessExecutorTool, ServiceCheckerTool, SystemMonitorTool

**Key Changes**:
- **NO parameter changes** - all tools already used appropriate domain-specific names
- Only response format updated to ResponseBuilder pattern

**No Migration Required for Parameters** - Only update response parsing.

### Media Processing Tools (3 tools)
**Affected**: ImageProcessorTool, AudioProcessorTool, VideoProcessorTool

**Key Changes**:
- `input_path`/`output_path` → `source_path`/`target_path` for processing operations
- Read-only operations keep `file_path`
- VideoProcessorTool keeps `video_path` for source, uses `target_path` for output

**Migration Example**:
```rust
// Old
let params = json!({
    "operation": "resize",
    "input_path": "/path/to/input.png",
    "output_path": "/path/to/output.png",
    "width": 800,
    "height": 600
});

// New
let params = json!({
    "operation": "resize",
    "source_path": "/path/to/input.png",
    "target_path": "/path/to/output.png",
    "width": 800,
    "height": 600
});
```

### API/Web Tools (3 tools)
**Affected**: HttpRequestTool, GraphQLQueryTool, WebSearchTool

**Key Changes**:
- `url` → `input` (HttpRequestTool)
- `query` → `input` (GraphQLQueryTool, WebSearchTool)
- All other parameters remain unchanged
- ResponseBuilder format with structured result data

**Migration Example**:
```rust
// Old
let params = json!({
    "url": "https://api.example.com/data",
    "method": "GET"
});

// New
let params = json!({
    "input": "https://api.example.com/data",
    "method": "GET"
});
```

## Complete Migration Checklist

### Phase 1: Parameter Updates
- [ ] **File Operations**: Update all path parameters to new naming conventions
- [ ] **Data Processing**: Change `content` to `input`
- [ ] **Utilities**: Update tool-specific parameters to `input` where applicable
- [ ] **Media Tools**: Change path parameters to `source_path`/`target_path`
- [ ] **API/Web Tools**: Update primary parameters to `input`
- [ ] **System Tools**: No changes needed

### Phase 2: Response Handling
- [ ] **Stop parsing text as JSON** for file operations (now contains only messages)
- [ ] **Update to ResponseBuilder format** for all tools
- [ ] **Access result data** from `response["result"]` instead of metadata
- [ ] **Check success status** from `response["success"]` field
- [ ] **Update error handling** to use consistent error format

### Phase 3: Testing
- [ ] **Run all tests** with updated parameters
- [ ] **Verify response parsing** works correctly
- [ ] **Check tool chaining** still functions properly
- [ ] **Test error scenarios** with new format
- [ ] **Validate security levels** remain unchanged

## Common Migration Patterns

### Parameter Name Changes
```rust
// Pattern 1: Primary data parameter
"expression" | "text" | "data" | "content" | "template" | "url" | "query" → "input"

// Pattern 2: Path parameters
"from_path" → "source_path"
"to_path" → "target_path"
"input_path" → "source_path" OR "path"
"output_path" → "target_path"
"archive_path" → "path"

// Pattern 3: Collection parameters
"files" | "paths" → "input"
```

### Response Parsing Updates
```rust
// Old: Various formats
let result = match tool_type {
    FileOps => serde_json::from_str(&output.text)?,
    DataOps => output.text, // raw data
    Others => output.metadata.extra,
};

// New: Unified format
let response: Value = serde_json::from_str(&output.text)?;
let success = response["success"].as_bool().unwrap_or(false);
let result = &response["result"];
```

## Error Patterns and Solutions

### Common Errors During Migration

#### Missing Required Parameter 'input'
**Cause**: Using old parameter name  
**Solution**: Update to standardized parameter name

```rust
// Error: "Missing required parameter 'input'"
let params = json!({"expression": "2 + 2"});  // ❌

// Fix
let params = json!({"input": "2 + 2"});       // ✅
```

#### JSON Parsing Fails on Response
**Cause**: Expecting old response format  
**Solution**: Update to ResponseBuilder format

```rust
// Error: JSON parsing fails
let data = serde_json::from_str(&output.text)?;  // ❌

// Fix
let response: Value = serde_json::from_str(&output.text)?;
let data = &response["result"];                   // ✅
```

#### Missing Path Parameters
**Cause**: Using old path parameter names  
**Solution**: Update to new conventions

```rust
// Error: "Missing required parameter 'source_path'"
let params = json!({"from_path": "/a", "to_path": "/b"});  // ❌

// Fix
let params = json!({"source_path": "/a", "target_path": "/b"}); // ✅
```

## Testing Strategy

### Unit Tests
All tools maintain comprehensive test coverage:
- Parameter validation tests updated
- Response format tests added
- Error handling tests verified

### Integration Tests
- Tool chaining workflows tested
- Cross-tool data flow validated
- Performance benchmarks maintained

### Migration Validation
```bash
# Run full test suite
cargo test --workspace

# Run specific tool tests
cargo test -p llmspell-tools

# Run quality checks
./scripts/quality-check.sh
```

## Performance Impact

### Positive Changes
- **Shared Utilities**: Common functionality moved to llmspell-utils
- **Consistent Patterns**: Reduced cognitive overhead
- **Better Caching**: ResponseBuilder enables better response caching

### Neutral Changes
- **Parameter Validation**: Same validation, different parameter names
- **Response Generation**: JSON serialization overhead similar
- **Memory Usage**: ResponseBuilder adds minimal overhead

## Security Considerations

### Maintained Security Levels
- All tool security levels unchanged
- Resource limits preserved
- Sandboxing configurations intact

### Enhanced Security
- Parameter validation standardized
- Response sanitization improved
- Error message consistency prevents information leakage

## Breaking Changes Summary

This is a comprehensive breaking change affecting:

### Parameters Changed (17 tools)
1. **File Operations (5)**: Path parameter restructuring
2. **Data Processing (2)**: Content → input
3. **Utilities (5 of 9)**: Various → input
4. **Media Tools (3)**: Path parameter restructuring  
5. **API/Web Tools (3)**: Primary parameter → input

### Parameters Unchanged (9 tools)
1. **System Tools (4)**: Domain-appropriate names preserved
2. **Utilities (4 of 9)**: Already used appropriate conventions

### Response Format Changed (All 26 tools)
- All tools now use ResponseBuilder pattern
- Consistent JSON structure across all tools
- Success/failure, message, and result clearly separated

## Post-Migration Benefits

### Developer Experience
1. **Predictable APIs**: Consistent parameter naming across tools
2. **Easy Tool Discovery**: Standard patterns reduce learning curve
3. **Better Error Messages**: ResponseBuilder provides clear error context
4. **Improved Debugging**: Consistent response format aids troubleshooting

### System Benefits
1. **DRY Compliance**: Shared utilities reduce code duplication
2. **Maintainability**: Consistent patterns easier to maintain
3. **Extensibility**: New tools follow established conventions
4. **Testing**: Uniform testing patterns across all tools

### Future-Proofing
1. **Phase 3+ Readiness**: Foundation for external integration tools
2. **Workflow Orchestration**: Consistent interfaces enable complex workflows
3. **Performance Optimization**: Shared utilities enable system-wide optimizations
4. **Documentation**: Standardized patterns simplify documentation

## Migration Timeline Recommendations

### Immediate (Week 1)
1. **Update Core Integrations**: Fix most-used tools first
2. **Update Test Suite**: Ensure validation catches regressions
3. **Document Internal Usage**: Update team documentation

### Short Term (Weeks 2-3)
1. **Update Bridge Code**: Fix llmspell-bridge integrations
2. **Update Script APIs**: Fix Lua/JavaScript API calls
3. **Update Examples**: Fix documentation examples

### Medium Term (Month 1)
1. **Update Documentation**: Complete user-facing documentation
2. **Performance Testing**: Validate performance characteristics
3. **Integration Testing**: Test with external systems

## Lua Examples Migration (Task 3.0.13)

### Overview
All Lua example files have been updated to use the new standardized parameters for tools that have been migrated to the v0.3.0 parameter format. This includes changing from direct tool execution to the `Tool.executeAsync` pattern and updating all parameter names.

### Key Changes Applied

#### Parameter Standardization
All utility tool examples have been updated to use the standardized `input` parameter:

1. **HashCalculatorTool**: `data` → `input`
2. **TextManipulatorTool**: `text` → `input`
3. **CalculatorTool**: `expression` → `input`
4. **TemplateEngineTool**: `template` → `input`
5. **DataValidationTool**: `data` → `input`

#### Execution Pattern Changes
Changed from direct tool execution to async pattern:
```lua
-- Old pattern
local tool = Tool.get("calculator")
local result = tool.execute({expression = "2 + 2"})

-- New pattern
local result = Tool.executeAsync("calculator", {
    operation = "evaluate",
    input = "2 + 2"
})
```

### Files Updated

#### tools-utility.lua
- 22 parameter updates across all utility tools
- Hash calculations, text manipulations, calculations, template rendering, and validation examples

#### tools-workflow.lua
- 25 updates total (6 parameter updates + 19 execution method changes)
- Data processing pipeline, file analysis, system monitoring, validation, and error handling workflows

#### tools-performance.lua
- 10 updates (execution method changes + operation parameters)
- Performance benchmarks for all tool categories

#### Domain-Appropriate Parameters
System tools retain their domain-specific parameter names as they don't deal with generic data input:
- **EnvironmentReaderTool**: `operation`, `variable_name`, `pattern`
- **ProcessExecutorTool**: `executable`, `arguments`, `timeout_ms`
- **ServiceCheckerTool**: `check_type`, `target`, `timeout_ms`
- **SystemMonitorTool**: `operation`

### Testing Results
- All 10 example files now pass successfully (100% success rate)
- Performance benchmarks show all tools meeting their target metrics:
  - Lightweight tools: <10ms initialization, <50ms operations
  - Medium weight tools: <50ms initialization, <100ms operations
  - Heavy tools: <100ms initialization, <500ms operations

### Additional Updates During Testing

#### Tool Execution Method
The Lua bridge uses async execution for tool calls, requiring the pattern:
```lua
local result = Tool.executeAsync(tool_name, params)
```

#### Operation Parameters
Added missing `operation` parameters where required:
- UUID Generator: `operation = "generate"` or `operation = "component_id"`
- Calculator: `operation = "evaluate"`
- Hash Calculator: `operation = "hash"`, `operation = "verify"`, `operation = "hash_file"`
- Template Engine: `operation = "render"`
- Data Validation: `operation = "validate"`

## Support and Resources

### Documentation
- This comprehensive migration guide
- Individual tool documentation updated
- CHANGELOG_v0.3.0.md with detailed changes
- Test files as working examples

### Getting Help
1. **Review Migration Examples**: Check tool-specific examples above
2. **Run Test Suite**: Use tests as reference implementations
3. **Check Error Messages**: ResponseBuilder provides detailed error context
4. **Submit Issues**: Report problems to project repository

### Tool-Specific Resources
- Integration tests in `llmspell-tools/tests/`
- Bridge tests in `llmspell-bridge/tests/`
- Individual tool documentation in source files
- Benchmark tests for performance validation

---

## Conclusion

Phase 3.0 represents a foundational standardization that positions rs-llmspell for scalable growth. While this migration requires updating parameter names and response parsing across all tool usage, the result is a more consistent, maintainable, and extensible tool ecosystem.

The investment in this migration pays dividends in:
- **Reduced cognitive load** for developers
- **Faster onboarding** for new team members  
- **Easier tool composition** for complex workflows
- **Simplified testing and debugging**
- **Foundation for Phase 3+ features**

This migration is mandatory for continued development and is the foundation for the upcoming external integration tools and workflow orchestration features planned for the remainder of Phase 3.

---

*This migration guide represents the complete standardization of 26 tools across 6 categories. No automated migration tools are provided as part of the pre-1.0 clean break approach.*