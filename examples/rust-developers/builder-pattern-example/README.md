# Builder Pattern Example

**Complexity Level:** INTERMEDIATE  
**Time to Complete:** ~3 minutes compilation + execution  

## Overview

This example demonstrates the builder pattern for constructing complex LLMSpell tools with fluent APIs, validation, and flexible configuration. You'll learn how to create user-friendly APIs for complex object construction.

## Key Concepts

- **Builder Pattern** - Fluent API for object construction
- **Method Chaining** - Readable configuration through chained method calls
- **Validation** - Build-time validation of configuration
- **Type Safety** - Compile-time guarantees for required fields
- **Immutability** - Builders consume self to prevent misuse

## What You'll Learn

- Creating fluent builder APIs with method chaining
- Implementing validation during the build process
- Handling optional parameters with sensible defaults
- Building complex configuration objects step-by-step
- Error handling in builder patterns (build vs try_build)

## Architecture Components

### ConfigurableTool
- **Main Tool** - The complex tool being constructed
- **Configuration** - Rich configuration object with validation
- **Builder Integration** - Tool that uses builder for construction

### ConfigurableToolBuilder
- **Fluent API** - Method chaining for readable configuration
- **Validation** - Build-time checks for configuration correctness
- **Defaults** - Sensible default values for optional parameters
- **Error Handling** - Validation errors during construction

### ToolConfig
- **Configuration Object** - Complex configuration with many options
- **Validation Rules** - Business logic for valid configurations
- **Serialization** - JSON serialization for debugging and storage

## Builder Patterns Demonstrated

### 1. Basic Builder
- **Simple Construction** - Basic tool creation with minimal configuration
- **Default Values** - Automatic defaults for unspecified options

### 2. Fluent Builder
- **Method Chaining** - Readable configuration through chaining
- **Complex Configuration** - Multiple processors, custom settings
- **Rich Metadata** - Comprehensive tool description and settings

### 3. Builder Validation
- **Input Validation** - Rejecting invalid configurations
- **Business Rules** - Enforcing domain-specific constraints
- **Error Messages** - Clear validation failure descriptions

### 4. Method Chaining
- **Immutable Builders** - Self-consuming methods prevent reuse
- **Type Safety** - Compile-time guarantees for method order
- **Fluent APIs** - Natural language-like configuration

## How to Run

```bash
cd builder-pattern-example
cargo run
```

## Expected Output

The example demonstrates:
- Basic tool construction with simple configuration
- Advanced tool with complex multi-step configuration
- Validation errors for invalid configurations (empty name, zero timeout, excessive retries)
- Method chaining for readable tool configuration
- Tool execution with built configuration

## Builder Pattern Benefits

- **Fluent API** - Readable method chaining for configuration
- **Validation** - Build-time checks for configuration correctness
- **Flexibility** - Optional parameters with sensible defaults
- **Immutability** - Builders consume self, preventing misuse
- **Type Safety** - Compile-time guarantees for required fields

## Common Use Cases

- **Complex Object Construction** - Objects with many optional parameters
- **Configuration Objects** - Settings that need validation
- **API Design** - User-friendly interfaces for complex operations
- **Domain-Specific Languages** - Readable configuration syntax

## Design Patterns

- **Builder Pattern** - Step-by-step object construction
- **Fluent Interface** - Method chaining for readability
- **Validation Pattern** - Early validation of configuration
- **Factory Pattern** - Centralized object creation logic

## Next Steps

After completing this example:
- Study `integration-test-example` for comprehensive testing strategies
- Explore tool composition and advanced configuration patterns
- Learn about persistent configuration and tool templates