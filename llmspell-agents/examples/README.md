# LLMSpell Agent Examples

This directory contains comprehensive examples demonstrating various agent patterns and use cases in the LLMSpell framework.

## Overview

These examples showcase:
- Different agent architectures and patterns
- Real-world use cases for agents
- Integration with the 33+ standardized tools
- Multi-agent coordination patterns
- Performance optimization techniques
- Error handling and recovery strategies

## Examples

### 1. Tool Orchestrator Agent (`tool_orchestrator.rs`)
Demonstrates an agent that coordinates multiple tools to solve complex tasks.
- **Use Case**: Data analysis pipeline that uses multiple tools
- **Pattern**: Sequential tool orchestration with error handling
- **Tools Used**: FileOperations, JSONProcessor, CSVAnalyzer, TextManipulator

### 2. Multi-Agent Coordinator (`multi_agent_coordinator.rs`)
Shows how to coordinate multiple agents working together on a shared task.
- **Use Case**: Research project requiring specialized agents
- **Pattern**: Hierarchical agent coordination
- **Features**: Task delegation, result aggregation, consensus building

### 3. Monitoring Agent (`monitoring_agent.rs`)
Implements an agent that monitors system health and other agents.
- **Use Case**: System observability and alerting
- **Pattern**: Observer pattern with event-driven responses
- **Features**: Metric collection, health checks, alert generation

### 4. Data Pipeline Agent (`data_pipeline_agent.rs`)
Creates an agent specialized in data transformation workflows.
- **Use Case**: ETL operations with intelligent decision making
- **Pattern**: Pipeline pattern with conditional branching
- **Tools Used**: CSVAnalyzer, JSONProcessor, DataValidation, FileOperations

### 5. Research Agent (`research_agent.rs`)
Builds an agent that conducts research using web and file resources.
- **Use Case**: Automated research and report generation
- **Pattern**: Information gathering and synthesis
- **Tools Used**: WebSearch, WebScraper, FileOperations, TextManipulator

### 6. Code Generation Agent (`code_gen_agent.rs`)
Demonstrates an agent that generates and validates code.
- **Use Case**: Automated code generation with testing
- **Pattern**: Generate-test-refine loop
- **Tools Used**: FileOperations, ProcessExecutor, TextManipulator

### 7. Decision-Making Agent (`decision_agent.rs`)
Shows an agent that makes complex decisions based on multiple criteria.
- **Use Case**: Business logic automation
- **Pattern**: Rule-based decision tree with ML integration
- **Features**: Multi-criteria evaluation, confidence scoring

### 8. Agent Library Catalog (`agent_library.rs`)
A meta-example showing how to create reusable agent components.
- **Use Case**: Building a library of agent templates
- **Pattern**: Factory pattern with configuration
- **Features**: Agent templates, configuration management

### 9. Context-Aware Agent (`context_aware_agent.rs`)
Demonstrates advanced context management in agents.
- **Use Case**: Conversational agents with memory
- **Pattern**: Context inheritance and sharing
- **Features**: Hierarchical contexts, shared memory regions

### 10. Performance Optimized Agent (`performance_agent.rs`)
Shows techniques for building high-performance agents.
- **Use Case**: High-throughput data processing
- **Pattern**: Async execution with resource pooling
- **Features**: Parallel tool execution, caching, resource limits

## Running the Examples

Each example can be run using:

```bash
cargo run --example <example_name>
```

For example:
```bash
cargo run --example tool_orchestrator
```

## Common Patterns

### 1. Error Handling
All examples demonstrate proper error handling:
- Graceful degradation
- Retry mechanisms
- Error context preservation
- Recovery strategies

### 2. Resource Management
Examples show how to:
- Set and respect resource limits
- Manage concurrent operations
- Clean up resources properly
- Monitor resource usage

### 3. Tool Integration
Each example that uses tools demonstrates:
- Tool discovery
- Parameter validation
- Result processing
- Error handling from tools

### 4. State Management
Examples showcase different state management approaches:
- In-memory state
- Context-based state
- Shared state between agents
- State persistence patterns

## Performance Considerations

The examples are designed to showcase performance best practices:
- Efficient tool usage
- Parallel execution where appropriate
- Resource pooling
- Caching strategies
- Minimal overhead patterns

## Testing

Each example includes:
- Unit tests for core logic
- Integration tests with mock tools
- Performance benchmarks
- Error scenario tests

Run tests with:
```bash
cargo test --examples
```

## Contributing

When adding new examples:
1. Follow the established patterns
2. Include comprehensive documentation
3. Add appropriate tests
4. Update this README
5. Ensure the example demonstrates a unique use case or pattern

## Further Reading

- See `GUIDE.md` for detailed documentation on each example
- Check the main documentation for agent architecture details
- Review the tool documentation for available tools
- Explore the test files for additional usage patterns