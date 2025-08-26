# Custom Agent Example

**Complexity Level:** BEGINNER-INTERMEDIATE  
**Time to Complete:** ~3 minutes compilation + execution  

## Overview

This example demonstrates how to create custom agents with different personalities and specialized behaviors using the LLMSpell BaseAgent trait. You'll learn advanced patterns for agent customization and structured response handling.

## Key Concepts

- **Personality Agents** - Agents with distinct response styles and behaviors
- **Specialized Agents** - Task-specific agents (e.g., MathAgent)
- **Input Validation** - Parameter validation patterns for agents
- **Error Handling** - Graceful error handling and recovery
- **Metadata Enrichment** - Adding context and metadata to responses

## What You'll Learn

- Creating personality-based agents (pirate, robot, wizard)
- Implementing specialized agents for specific tasks
- Advanced parameter validation techniques
- Error handling patterns with graceful degradation
- Structured agent communication with AgentInput/AgentOutput

## Agents Demonstrated

### PersonalityAgent
- **Personalities:** Pirate, Robot, Wizard
- **Features:** Unique response styles, consistent character behavior
- **Example Usage:** Conversational interfaces with character

### MathAgent
- **Specialization:** Mathematical operations
- **Features:** Precision control, comprehensive math operations
- **Operations:** add, subtract, multiply, divide, sqrt, square, abs
- **Validation:** Parameter type checking, operation validation

## How to Run

```bash
cd custom-agent-example
cargo run
```

## Expected Output

The example demonstrates:
- Three personality agents with distinct response styles
- Mathematical agent with precision-controlled calculations
- Parameter validation (success and failure scenarios)
- Error handling with graceful degradation
- Structured agent metadata and responses

## Architecture Patterns

- **Personality Pattern** - Agents with consistent character behaviors
- **Specialization Pattern** - Task-specific agents with domain expertise
- **Validation Pipeline** - Multi-layer input validation
- **Error Recovery** - Graceful handling of invalid inputs
- **Response Enrichment** - Metadata and context in agent outputs

## Advanced Features

- **Dynamic Personalities** - Runtime personality switching
- **Precision Control** - Configurable mathematical precision
- **Comprehensive Validation** - Multi-parameter validation patterns
- **Structured Responses** - Rich metadata in agent outputs

## Next Steps

After completing this example:
- Explore `async-patterns-example` for concurrent agent execution
- Study `builder-pattern-example` for configurable agent creation
- Learn integration testing in `integration-test-example`