# Tool Signature Consistency Report

**Date**: 2025-01-27
**Task**: 2.10.1.4 - Verify consistency in tool signatures
**Status**: Analysis Complete - Multiple Inconsistencies Found

## Executive Summary

Analysis of all 26-27 Phase 2 tools reveals significant inconsistencies in both input parameter naming and output format patterns. These inconsistencies make it difficult for users to work with tools predictably and increase the learning curve.

## Input Parameter Inconsistencies

### 1. Primary Data Parameter Names

Different tools use different parameter names for their primary input data:

| Parameter Name | Tools Using It | Purpose |
|---------------|----------------|---------|
| `text` | text_manipulator | Text to process |
| `content` | csv_analyzer, file_operations (write), json_processor | Data content |
| `input` | base64_encoder, date_time_handler, json_processor | Input data |
| `data` | data_validation, hash_calculator | Data to process |
| `query` | graphql_query, web_search | Query string |
| `expression` | calculator | Math expression |
| `template` | template_engine | Template string |

**Issue**: Similar operations use different parameter names, making it hard to remember which to use.

### 2. File Path Parameters

File-related parameters are extremely inconsistent:

| Parameter Name | Tools Using It | Purpose |
|---------------|----------------|---------|
| `file_path` | file_converter, media processors | Single file path |
| `input_path`/`output_path` | media processors | Source/destination files |
| `path` | file_operations, file_search, file_watcher | Directory or file path |
| `paths` | file_watcher | Multiple paths to watch |
| `archive_path` | archive_handler | Archive file path |
| `file` | hash_calculator | File to hash |
| `url` | http_request, service_checker | URL/URI |

**Issue**: No standard for file path parameters across tools.

### 3. Operation Parameter

- **18 tools** use an `operation` parameter
- **7 tools** don't use it:
  - calculator (single purpose)
  - web_search (single purpose) 
  - graphql_query (uses `method` instead)
  - diff_calculator (single purpose)
  - template_engine (single purpose)
  - csv_analyzer (uses `operation` inconsistently)
  - data_validation (single purpose)

### 4. Most Problematic Tools

1. **json_processor**: Uses BOTH `input` (for objects) AND `content` (for strings)
2. **Media processors**: Have both `file_path` AND `input_path`/`output_path`
3. **file_operations**: Uses different parameters for each operation type

## Output Format Inconsistencies

### 1. Response Structure Patterns

Tools use three different response patterns:

#### Pattern A: ResponseBuilder (Standardized)
```json
{
  "success": true,
  "operation": "operation_name",
  "message": "Human readable message",
  "result": { /* operation specific data */ }
}
```
Used by: calculator, base64_encoder, uuid_generator, text_manipulator, hash_calculator, date_time_handler, diff_calculator, data_validation, template_engine

#### Pattern B: Direct JSON
```json
{
  "success": true,
  "custom_field": "value",
  // No standard structure
}
```
Used by: file_operations, archive_handler, environment_reader, process_executor, service_checker, system_monitor

#### Pattern C: Mixed/Custom
- **json_processor**: Returns raw JSON results without wrapper
- **web_search**: Returns plain text with JSON in metadata
- **Media processors**: Return placeholder responses

### 2. Field Naming Inconsistencies

| Field Purpose | Different Names Used | Tools |
|--------------|---------------------|--------|
| Main result | `result`, `output`, `data`, custom fields | Various |
| Success flag | `success`, (missing), `valid` | Various |
| Operation name | `operation`, (in metadata), (missing) | Various |
| Errors | `error`, `errors`, `message` | Various |

### 3. Validation Response Formats

Different validation operations return different structures:
- **uuid_generator**: `{valid: bool, error?: string}`
- **calculator**: `{success: bool, result: {valid: bool}}`
- **json_processor**: `{is_valid: bool, errors: []}`
- **data_validation**: `{success: bool, result: {valid: bool, errors: []}}`

## Recommendations

### 1. Standardize Input Parameters

#### Primary Data Parameters
- Use `input` for all primary data inputs
- Reserve `data` for when specifically referring to data structures
- Use `text` only for text-specific operations
- Deprecate `content` in favor of `input`

#### File Parameters
- Use `path` for single file/directory operations
- Use `source_path`/`target_path` for transformations
- Use `paths` (plural) for multiple paths
- Deprecate `file_path`, `input_path`, `output_path`

#### Operation Parameters
- All multi-function tools should use `operation`
- Single-purpose tools can omit it

### 2. Standardize Output Format

All tools should use ResponseBuilder pattern:
```json
{
  "operation": "operation_name",
  "success": true/false,
  "message": "Optional human-readable message",
  "result": {
    // Operation-specific data goes here
  },
  "error": "Only present if success is false"
}
```

### 3. Validation Response Standard
```json
{
  "operation": "validate",
  "success": true,
  "result": {
    "valid": true/false,
    "errors": [] // Optional array of error details
  }
}
```

## Impact Assessment

### High Priority Fixes
1. **json_processor**: Remove dual input/content parameters
2. **Media processors**: Remove redundant path parameters
3. **All tools**: Standardize on ResponseBuilder pattern

### Medium Priority Fixes
1. Align file path parameter names
2. Standardize validation response formats
3. Ensure all tools include operation in response

### Low Priority Fixes
1. Rename parameters for consistency
2. Add operation parameter to single-purpose tools
3. Update documentation to reflect standards

## Conclusion

The current tool signatures show significant inconsistencies that impact usability. Implementing these recommendations would greatly improve the developer experience and make the tool library more intuitive to use. The ResponseBuilder pattern already used by ~35% of tools provides a good foundation for standardization.