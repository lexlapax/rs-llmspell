# Bridge Architecture Pattern

## Overview

The llmspell bridge architecture follows a clean separation of concerns pattern that ensures:
- Single source of truth for data formats
- No logic duplication across language bridges  
- Consistent behavior across Lua, JavaScript, and Python integrations
- Maintainable and extensible codebase

## Architecture Layers

### 1. Core Layer (`llmspell-workflows`)
- **Purpose**: Contains all business logic for workflow execution
- **Responsibilities**: 
  - Workflow execution algorithms
  - Condition evaluation
  - Step orchestration
  - State management
- **Key Principle**: Language-agnostic, pure Rust implementation

### 2. Native Bridge Layer (`llmspell-bridge/src/workflows.rs`)
- **Purpose**: Single source of truth for data serialization/deserialization
- **Responsibilities**:
  - JSON ↔ Rust struct conversion
  - Workflow factory methods
  - Common validation logic
  - Format specifications
- **Key Functions**:
  - `workflow_step_to_json()` - Converts WorkflowStep to flat JSON format
  - `parse_workflow_step()` - Parses JSON to WorkflowStep
  - `create_*_workflow()` - Factory methods for workflow types

### 3. Language Bridge Layers
#### Lua Bridge (`llmspell-bridge/src/lua/globals/workflow.rs`)
- **Purpose**: Lua-specific integration only
- **Responsibilities**:
  - Convert Lua tables → Rust structs
  - Call native bridge functions
  - Manage Lua userdata lifecycle
- **Key Pattern**: 
  ```rust
  // DON'T: Create JSON directly in Lua bridge
  // let json = serde_json::json!({...});
  
  // DO: Use native bridge for JSON conversion
  let steps_json = steps.iter()
      .map(|step| crate::workflows::workflow_step_to_json(step))
      .collect();
  ```

#### JavaScript Bridge (`llmspell-bridge/src/javascript/globals/workflow.rs`)
- **Purpose**: JavaScript-specific integration (Phase 12)
- **Responsibilities**: Same as Lua but for JS objects
- **Implementation Note**: Will follow same pattern - convert JS objects to Rust structs, delegate to native bridge

#### Python Bridge (Future)
- **Purpose**: Python-specific integration
- **Responsibilities**: Same pattern for Python dicts

## Data Flow

```
User Script (Lua/JS/Python)
    ↓
Language-specific types (tables/objects/dicts)
    ↓
Language Bridge (converts to Rust structs)
    ↓
Native Bridge (single JSON conversion point)
    ↓
Core Workflows (business logic execution)
```

## JSON Format Specification

### Workflow Step Format (Flat Structure)
The native bridge expects and produces flat JSON for workflow steps:

```json
// Tool step
{
  "name": "step_name",
  "tool": "calculator",
  "parameters": {"expression": "2 + 2"}
}

// Agent step
{
  "name": "step_name", 
  "agent": "agent_id",
  "input": {"prompt": "..."}
}

// Workflow step (nested)
{
  "name": "step_name",
  "workflow": "workflow_id",
  "input": {}
}
```

### Why Flat Format?
- Simpler parsing logic
- Clear field presence checking
- Consistent across all step types
- No nested type discrimination needed

## Migration Guide

### For Existing Code
If you have code that creates JSON directly in language bridges:

1. **Identify JSON creation**: Look for `serde_json::json!` in language bridge files
2. **Extract to native bridge**: Move JSON format logic to `workflows.rs`
3. **Create conversion function**: Add a public function like `workflow_step_to_json`
4. **Update language bridge**: Call the new function instead of creating JSON

### For New Language Bridges
When implementing a new language bridge (e.g., Python):

1. **Parse language types**: Convert Python dicts/lists to Rust structs
2. **Use native bridge functions**: Call existing conversion functions
3. **Never create JSON directly**: Always delegate to native bridge
4. **Test with existing tests**: Reuse integration tests to verify consistency

## Benefits of This Architecture

1. **Single Source of Truth**: JSON format defined in one place
2. **Consistency**: All languages produce identical JSON
3. **Maintainability**: Format changes require single update
4. **Testability**: One set of tests covers all languages
5. **Performance**: Minimal conversion overhead
6. **Type Safety**: Rust's type system ensures correctness

## Common Pitfalls to Avoid

1. **DON'T** create JSON in language bridges
2. **DON'T** duplicate parsing logic across bridges
3. **DON'T** add language-specific format variations
4. **DO** keep language bridges thin
5. **DO** centralize format specifications
6. **DO** use Rust structs as intermediate representation

## Testing Strategy

### Unit Tests
- Test native bridge JSON conversion functions
- Test language bridge struct creation
- Keep tests focused on single layer

### Integration Tests  
- Test full flow from language → JSON → execution
- Use consistent test cases across all languages
- Verify format compatibility

### Example Test Pattern
```rust
#[test]
fn test_workflow_step_json_format() {
    let step = WorkflowStep::new(
        "test".to_string(),
        StepType::Tool {
            tool_name: "calculator".to_string(),
            parameters: json!({"input": "2+2"})
        }
    );
    
    let json = workflow_step_to_json(&step);
    assert_eq!(json["name"], "test");
    assert_eq!(json["tool"], "calculator");
    assert!(json["step_type"].is_null()); // Flat format, no nesting
}
```

## Future Considerations

- **GraphQL Bridge**: Follow same pattern for GraphQL schema
- **REST API**: Use native bridge for request/response formatting
- **Binary Protocols**: Add binary serialization in native bridge
- **Schema Evolution**: Version formats in native bridge

## Summary

The bridge architecture pattern ensures that:
- Business logic stays in `llmspell-workflows`
- Format logic stays in `llmspell-bridge/src/workflows.rs`  
- Language bridges only handle language-specific conversions

This separation enables us to support multiple scripting languages without duplicating logic or formats, making the system maintainable and extensible.