# Custom Tool Example

**Complexity Level:** BEGINNER  
**Time to Complete:** ~3 minutes compilation + execution  

## Overview

This example demonstrates how to create custom tools using the LLMSpell framework's BaseAgent and Tool traits. You'll learn the fundamental patterns for building tools that can be integrated into LLM workflows.

## Key Concepts

- **BaseAgent trait** - Core execution interface with `execute_impl`, `validate_input`, and `handle_error` methods
- **Tool trait** - Tool-specific metadata with `category`, `security_level`, and `schema` methods  
- **ComponentMetadata** - Tool identification and description
- **AgentInput/AgentOutput** - Structured data flow for tools
- **Parameter validation** - JSON parameter extraction and validation patterns

## What You'll Learn

- Creating custom tools with the BaseAgent + Tool trait pattern
- Implementing parameter validation and error handling
- Tool categorization and security levels (Safe, Restricted, Privileged)
- JSON parameter processing with serde_json
- Tool schema definition for parameter validation

## Tools Demonstrated

### TextAnalyzerTool
- **Category:** Analysis
- **Security Level:** Safe
- **Operations:** sentiment analysis, word counting
- **Features:** JSON parameter extraction, text processing

### MathCalculatorTool  
- **Category:** Utility
- **Security Level:** Safe
- **Operations:** add, subtract, multiply, divide, sqrt
- **Features:** Mathematical operations, division by zero handling

## How to Run

```bash
cd custom-tool-example
cargo run
```

## Expected Output

The example will demonstrate:
- Tool creation and metadata display
- Parameter validation (both success and failure cases)
- Text analysis operations with sentiment detection
- Mathematical calculations with error handling
- Tool schema generation

## Architecture Patterns

- **Trait-based design** - BaseAgent + Tool traits for modularity
- **JSON parameter processing** - Structured input validation
- **Error handling** - Graceful failure with informative messages
- **Metadata-driven** - Self-describing tools with categories and schemas

## Next Steps

After completing this example:
- Explore `custom-agent-example` for personality-based agents
- Study `async-patterns-example` for concurrent tool execution
- Learn extension patterns in `extension-pattern-example`