# Rs-LLMSpell Examples

**Current Version**: Phase 4.8.1 Complete  
**Available Tools**: 34 production tools  
**Features**: Agents, Workflows, Tools, State Management, Hooks, Events

**🔗 Navigation**: [← Project Home](../README.md) | [Documentation Hub](../docs/README.md) | [User Guide](../docs/user-guide/README.md)

---

## Overview

This directory contains working examples demonstrating rs-llmspell capabilities. All examples are tested and work with the current Phase 4.8.1 implementation, including comprehensive hook and event system integration.

## 📁 Directory Structure

```
examples/
├── hello.lua                    # Basic hello world script
├── llmspell-demo.lua           # Comprehensive demo
├── provider-info.lua           # Provider configuration info
├── streaming-demo.lua          # Streaming responses
├── multimodal-stub.lua         # Multimodal capabilities
├── llmspell.toml              # Example configuration
├── minimal.toml               # Minimal configuration
├── lua/
│   ├── agents/                 # Agent examples (10 files)
│   ├── tools/                  # Tool examples (13 files)
│   ├── workflows/              # Workflow examples (9 files)
│   ├── hooks/                  # Hook system examples (10 files)
│   ├── events/                 # Event system examples (10 files)
│   ├── integration/            # Real-world integration examples (3 files)
│   ├── run-all-examples.lua    # Master runner for all examples
│   ├── run-integration-demos.lua  # Integration demo runner
│   └── run-performance-benchmarks.lua  # Performance testing runner
└── *.sh                        # Legacy runner scripts
```

## 🚀 Quick Start

### Basic Examples

```bash
# Run hello world
llmspell run examples/hello.lua

# Run comprehensive demo
llmspell run examples/llmspell-demo.lua

# Check provider configuration
llmspell run examples/provider-info.lua
```

### Agent Examples (Requires API Keys)

```bash
# Run all agent examples
./examples/run-all-agent-examples.sh

# Run specific agent example
llmspell run examples/lua/agents/agent-simple.lua
llmspell run examples/lua/agents/agent-orchestrator.lua
```

### Tool Examples

```bash
# Run all tool examples
./examples/run-all-tools-examples.sh

# Run specific tool category
llmspell run examples/lua/tools/tools-filesystem.lua
llmspell run examples/lua/tools/tools-utility.lua
```

### Workflow Examples

```bash
# Run workflow examples
./examples/run-workflow-examples.sh

# Run specific workflow pattern
llmspell run examples/lua/workflows/workflow-sequential.lua
llmspell run examples/lua/workflows/workflow-parallel.lua
```

### Hook & Event Examples

```bash
# Run all hook and event examples
llmspell run examples/lua/run-all-examples.lua

# Run specific hook examples
llmspell run examples/lua/hooks/hook-basic.lua
llmspell run examples/lua/hooks/hook-advanced-patterns.lua

# Run specific event examples
llmspell run examples/lua/events/event-basic.lua
llmspell run examples/lua/events/event-cross-language.lua

# Run integration demos
llmspell run examples/lua/run-integration-demos.lua

# Run performance benchmarks
llmspell run examples/lua/run-performance-benchmarks.lua

# Run AI-powered research assistant (requires API keys)
export OPENAI_API_KEY="your-key-here"  # or ANTHROPIC_API_KEY
llmspell run examples/lua/integration/ai-powered-research-assistant.lua
```

## 📋 Example Categories

### Root Examples

- **`hello.lua`** - Simplest possible script
- **`llmspell-demo.lua`** - Shows core functionality
- **`provider-info.lua`** - Display configured providers
- **`streaming-demo.lua`** - Streaming LLM responses
- **`multimodal-stub.lua`** - Multimodal input examples

### Agent Examples (`lua/agents/`)

Demonstrates agent creation, configuration, and usage patterns:

- **`agent-simple.lua`** - Basic agent creation and execution
- **`agent-orchestrator.lua`** - Agent coordinating multiple tools
- **`agent-coordinator.lua`** - Multi-agent coordination
- **`agent-processor.lua`** - Data processing with agents
- **`agent-monitor.lua`** - System monitoring agent
- **`agent-composition.lua`** - Composing agents together
- **`agent-async-example.lua`** - Async patterns (internal use)
- **`agent-simple-demo.lua`** - Interactive demo
- **`agent-simple-benchmark.lua`** - Performance testing
- **`agent-api-comprehensive.lua`** - Full API demonstration

### Tool Examples (`lua/tools/`)

Shows all 34 available tools organized by category:

- **`tools-showcase.lua`** - Demonstrates all tools
- **`tools-filesystem.lua`** - File operations (5 tools)
- **`tools-data.lua`** - Data processing (4 tools)
- **`tools-web.lua`** - Web & network tools (7 tools)
- **`tools-system.lua`** - System integration (4 tools)
- **`tools-utility.lua`** - Utility tools (10 tools)
- **`tools-media.lua`** - Media processing (3 tools)
- **`tools-security.lua`** - Security features demo
- **`tools-integration.lua`** - External integrations
- **`tools-workflow.lua`** - Tools in workflows
- **`tools-performance.lua`** - Performance benchmarks
- **`tools-utility-reference.lua`** - API reference example

### Workflow Examples (`lua/workflows/`)

Demonstrates all 4 workflow patterns:

- **`workflow-sequential.lua`** - Step-by-step execution
- **`workflow-conditional.lua`** - Branching logic
- **`workflow-loop.lua`** - Iteration patterns
- **`workflow-parallel.lua`** - Concurrent execution
- **`workflow-basics-*.lua`** - Basic pattern examples
- **`workflow-agent-integration.lua`** - Agents in workflows

### Hook Examples (`lua/hooks/`)

Demonstrates the complete hook system from basic registration to advanced patterns:

- **`hook-basic.lua`** - Basic hook registration and unregistration
- **`hook-priorities.lua`** - All 5 priority levels (highest to lowest)
- **`hook-lifecycle.lua`** - Complete agent lifecycle hooks
- **`hook-tool-integration.lua`** - Tool execution hooks with validation
- **`hook-workflow-integration.lua`** - Workflow stage hooks and coordination
- **`hook-data-modification.lua`** - All hook result types (continue, modified, cancel, etc.)
- **`hook-error-handling.lua`** - Comprehensive error handling patterns
- **`hook-cross-language.lua`** - Cross-language hook coordination
- **`hook-filtering-listing.lua`** - Hook listing and filtering capabilities
- **`hook-advanced-patterns.lua`** - Complex patterns (retry, circuit breaker, etc.)

### Event Examples (`lua/events/`)

Shows event system capabilities from basic pub/sub to advanced coordination:

- **`event-basic.lua`** - Basic publish/subscribe patterns
- **`event-patterns.lua`** - Pattern matching with wildcards (`user.*`, `*.error`)
- **`event-cross-language.lua`** - Cross-language event communication
- **`event-data-structures.lua`** - Complex nested event data structures
- **`event-subscription-management.lua`** - Subscription lifecycle management
- **`event-performance.lua`** - High-throughput performance scenarios
- **`event-timeout-handling.lua`** - Timeout patterns and error recovery
- **`event-statistics.lua`** - System monitoring and metrics collection
- **`event-workflow-coordination.lua`** - Workflow orchestration patterns
- **`event-hook-integration.lua`** - Hook-event integration patterns

### Integration Examples (`lua/integration/`)

Real-world scenarios combining hooks, events, agents, tools, and workflows:

- **`realtime-data-pipeline.lua`** - Complete data pipeline with end-to-end processing
- **`user-workflow-automation.lua`** - Business workflow automation with intelligent routing
- **`intelligent-monitoring-system.lua`** - AI-driven monitoring with predictive analytics
- **`ai-powered-research-assistant.lua`** - Full LLM integration with agents, tools, workflows, hooks, and events (requires API keys)
- **`ai-research-assistant-simple.lua`** - Simplified LLM agent example with hooks and events (simulation mode works, real agents require API keys)

### Runner Scripts (`lua/`)

Comprehensive test and demonstration runners:

- **`run-all-examples.lua`** - Master runner executing all 23 examples in sequence
- **`run-integration-demos.lua`** - Focused runner for integration examples with analysis
- **`run-performance-benchmarks.lua`** - Performance testing with stress tests and metrics

## 🔧 Configuration

### Example Configuration Files

- **`llmspell.toml`** - Full configuration example
- **`minimal.toml`** - Minimal required configuration

### Setting API Keys

```bash
# OpenAI
export OPENAI_API_KEY="your-key-here"

# Anthropic
export ANTHROPIC_API_KEY="your-key-here"

# Other providers as needed
```

## 📚 Learning Path

### For Beginners

1. Start with `hello.lua`
2. Explore `tools-utility.lua` for basic tool usage
3. Try `agent-simple.lua` for agent basics
4. Move to `workflow-sequential.lua` for workflows

### For Tool Users

1. Browse `tools-showcase.lua` for overview
2. Deep dive into category-specific examples
3. Check `tools-security.lua` for security patterns
4. Study `tools-workflow.lua` for integration

### For Agent Developers

1. Start with `agent-simple.lua`
2. Progress to `agent-orchestrator.lua`
3. Explore `agent-coordinator.lua` for multi-agent
4. Review `agent-api-comprehensive.lua` for full API

### For Workflow Builders

1. Learn patterns in `workflow-basics-*.lua`
2. Study complete examples in main workflow files
3. Check `workflow-agent-integration.lua` for AI workflows

### For Hook & Event System Users

1. Start with `hook-basic.lua` and `event-basic.lua`
2. Explore priorities and patterns in dedicated examples
3. Study cross-language coordination examples
4. Review integration examples for real-world usage patterns
5. Run performance benchmarks to understand system limits

### For System Integrators

1. Review all integration examples in `lua/integration/`
2. Run `run-integration-demos.lua` for guided analysis
3. Study performance characteristics with `run-performance-benchmarks.lua`
4. Use `run-all-examples.lua` for comprehensive system validation

## 🏃 Running Examples

### Individual Scripts

```bash
# Change to llmspell directory
cd /path/to/rs-llmspell

# Run with llmspell
llmspell run examples/hello.lua

# Or use cargo run
cargo run --bin llmspell -- run examples/hello.lua
```

### Batch Execution

```bash
# Make scripts executable
chmod +x examples/*.sh

# Run all examples of a type (legacy scripts)
./examples/run-all-agent-examples.sh
./examples/run-all-tools-examples.sh
./examples/run-workflow-examples.sh

# Run new hook and event examples
llmspell run examples/lua/run-all-examples.lua
llmspell run examples/lua/run-integration-demos.lua
```

### With Custom Config

```bash
# Use specific configuration
llmspell run --config examples/minimal.toml examples/hello.lua
```

## 🐛 Troubleshooting

### Common Issues

1. **"API key not found"**
   - Set environment variables for your providers
   - Check `llmspell.toml` configuration

2. **"Tool not found"**
   - Ensure you're using correct tool names
   - Run `Tool.list()` to see available tools

3. **"Agent timeout"**
   - Check network connectivity
   - Verify API keys are valid
   - Increase timeout in agent configuration

4. **"Permission denied"**
   - Some system tools require elevated permissions
   - File operations are sandboxed by default

5. **"Hook registration failed"**
   - Check hook point name spelling and validity
   - Ensure hook function returns valid result type
   - Use `Hook.list()` to see registered hooks

6. **"Event not received"**
   - Verify subscription pattern matches event name
   - Check event was published before receive timeout
   - Use `Event.list_subscriptions()` to debug subscriptions

7. **"Integration example failed"**
   - Ensure system has sufficient resources for complex examples
   - Check that all dependencies are properly initialized
   - Review integration logs for specific component failures

### Debug Mode

Enable debug output:

```lua
-- In your script
Logger.set_level("debug")

-- Or via environment
RUST_LOG=debug llmspell run examples/hello.lua
```

### Hook & Event System Debugging

For hook and event system issues:

```lua
-- List all registered hooks
local hooks = Hook.list()
print("Registered hooks:", #hooks)

-- List all event subscriptions  
local subs = Event.list_subscriptions()
print("Active subscriptions:", #subs)

-- Test hook registration
local test_handle = Hook.register("BeforeAgentExecution", function(ctx)
    print("Hook called with:", ctx.component_id.name)
    return "continue"
end, "normal")

-- Test event publish/subscribe
local sub_id = Event.subscribe("test.*")
Event.publish("test.message", {data = "hello"})
local received = Event.receive(sub_id, 1000) -- 1 second timeout
```

## 📖 Documentation

For detailed documentation, see:

- [User Guide](../docs/user-guide/) - Complete usage documentation
- [Tutorial](../docs/user-guide/tutorial-agents-workflows.md) - Step-by-step tutorial
- [API Reference](../docs/user-guide/api-reference-agents-workflows.md) - Full API docs
- [Tool Reference](../docs/user-guide/tool-reference.md) - All 34 tools documented
- [Hook System Guide](../docs/user-guide/hooks-guide.md) - Complete hook system documentation
- [Event System Guide](../docs/user-guide/events-guide.md) - Event system architecture and patterns

## 🤝 Contributing Examples

To add new examples:

1. Follow existing naming patterns
2. Include clear comments explaining the example
3. Test thoroughly before submitting
4. Update this README if adding new categories
5. Ensure examples work with current implementation
6. For hook/event examples, include comprehensive error handling
7. Add performance considerations for resource-intensive examples
8. Update runner scripts if adding new example categories

## 🪝 Hook & Event System Overview

Phase 4.8.1 introduces a comprehensive hook and event system enabling advanced automation, monitoring, and coordination patterns.

### Hook System Architecture

The hook system provides 14 execution points across the component lifecycle:

**Agent Hooks:**
- `BeforeAgentInit`, `AfterAgentInit` - Agent initialization
- `BeforeAgentExecution`, `AfterAgentExecution` - Agent task execution
- `BeforeAgentShutdown`, `AfterAgentShutdown` - Agent cleanup

**Tool Hooks:**
- `BeforeToolExecution`, `AfterToolExecution` - Tool invocation

**Workflow Hooks:**
- `BeforeWorkflowStart`, `AfterWorkflowEnd` - Workflow lifecycle
- `BeforeWorkflowStep`, `AfterWorkflowStep` - Individual step execution

**General Hooks:**
- `OnError` - Error handling and recovery
- `OnPerformanceThreshold` - Performance monitoring

### Hook Priority System

5 priority levels control execution order:
- `highest` - Critical system hooks (security, validation)
- `high` - Important business logic hooks
- `normal` - Standard application hooks (default)
- `low` - Logging and monitoring hooks
- `lowest` - Debug and development hooks

### Hook Result Types

Hooks can control execution flow:
- `continue` - Normal execution continues
- `modified` - Data was modified, continue with changes
- `cancel` - Stop execution immediately
- `redirect` - Change execution target
- `replace` - Replace current operation with alternative
- `retry` - Retry the operation
- `skipped` - Mark operation as skipped

### Event System Architecture

Publish/subscribe event system with:
- **Pattern Matching**: Wildcard patterns (`user.*`, `*.error`, `system.*.warning`)
- **Cross-Language Communication**: Events cross Lua/Rust boundaries
- **Correlation**: Event correlation for distributed tracing
- **Timeout Handling**: Configurable receive timeouts
- **Subscription Management**: Dynamic subscription lifecycle

### Integration Patterns

The examples demonstrate these key integration patterns:

1. **Event-Driven Architecture**: Loose coupling via events
2. **Hook-Based Monitoring**: Comprehensive observability
3. **Automated Recovery**: Error handling and retry mechanisms
4. **Performance Optimization**: Real-time metrics and thresholds
5. **Cross-Component Coordination**: Unified state management
6. **Business Process Automation**: Intelligent routing and approval flows
7. **Predictive Analytics**: AI-driven monitoring and remediation

### Performance Characteristics

System performance targets (measured in benchmarks):
- Hook registration: <10ms per hook
- Hook execution overhead: <1% of operation time
- Event publishing: >1000 events/second
- Event receiving: <10ms latency
- Memory usage: Linear scaling with load
- Concurrent operations: 100+ simultaneous hook/event operations

### Use Cases

**Data Processing**: Real-time ETL pipelines with event coordination
**Business Workflows**: Approval chains with intelligent routing
**System Monitoring**: Predictive analytics with automated remediation
**DevOps Automation**: CI/CD pipeline coordination and monitoring
**Security**: Event-driven security monitoring and response
**Performance**: Real-time performance monitoring and optimization

---

**Happy experimenting with rs-llmspell!** 🚀