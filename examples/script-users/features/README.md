# Feature Demonstrations

Comprehensive examples demonstrating LLMSpell features.

## 📚 Available Features

### Agents
- `agents-comprehensive.lua` - All agent capabilities
- `agents-async.lua` - Async agent operations
- `agents-streaming.lua` - Streaming responses
- `agents-multi-provider.lua` - Using multiple providers

### Tools (34+ built-in)
- `tools-filesystem.lua` - File operations
- `tools-web.lua` - Web requests and scraping
- `tools-data.lua` - Data processing
- `tools-communication.lua` - Email, webhooks
- `tools-system.lua` - System information
- `tools-utility.lua` - Utilities and helpers
- `tools-media.lua` - Image and media processing
- `tools-search.lua` - Search capabilities

### Workflows
- `workflow-sequential.lua` - Step-by-step execution
- `workflow-parallel.lua` - Concurrent operations
- `workflow-conditional.lua` - Branching logic
- `workflow-loop.lua` - Iteration patterns
- `workflow-nested.lua` - Complex compositions

### State Management
- `state-persistence.lua` - Saving and loading
- `state-scoped.lua` - Isolated contexts
- `state-migration.lua` - Schema evolution
- `state-backends.lua` - Different storage options

### Events
- `events-pubsub.lua` - Publish/subscribe
- `events-patterns.lua` - Pattern matching
- `events-correlation.lua` - Event correlation
- `events-cross-language.lua` - Lua/Rust events

### Hooks
- `hooks-lifecycle.lua` - Component lifecycle
- `hooks-priority.lua` - Execution order
- `hooks-modification.lua` - Data transformation
- `hooks-metrics.lua` - Performance monitoring

### Sessions
- `sessions-artifacts.lua` - Artifact management
- `sessions-replay.lua` - Session replay
- `sessions-access.lua` - Access control
- `sessions-multi-user.lua` - Multi-user support

## 🎯 How to Use These Examples

1. **Explore a Feature**: Pick the feature you need
2. **Run the Example**: Execute with appropriate configuration
3. **Modify and Learn**: Change parameters to understand behavior
4. **Combine Features**: Mix patterns for your use case

## 🔧 Configuration

Most examples need API keys or configuration:

```bash
# Set API keys
export OPENAI_API_KEY="your-key"
export ANTHROPIC_API_KEY="your-key"

# Run with custom config
llmspell --config custom.toml run workflow-parallel.lua
```

## 📖 Feature Documentation

Detailed documentation for each feature:
- [Agent API](../../../docs/user-guide/agent-api.md)
- [Tool Reference](../../../docs/user-guide/tool-reference.md)
- [Workflow Guide](../../../docs/user-guide/workflow-api.md)
- [State Management](../../../docs/user-guide/state-management.md)
- [Events Guide](../../../docs/user-guide/events-guide.md)
- [Hooks Guide](../../../docs/user-guide/hooks-guide.md)
- [Session Management](../../../docs/user-guide/session-management.md)