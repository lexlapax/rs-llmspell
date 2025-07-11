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

## Tools Requiring No Changes

The following tools already comply with standards:
- Base64EncoderTool (minor adjustments only)
- DateTimeHandlerTool (already compliant)

## Support

For migration assistance:
- Review examples in `/examples/v0.3.0/`
- Check tool-specific documentation
- Run validation script: `cargo run --bin validate-params`

## Future Compatibility

Version 0.3.0 establishes the parameter and response standards that will be maintained through 1.0 and beyond. No further breaking changes to these interfaces are planned.