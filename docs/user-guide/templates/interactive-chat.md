# Interactive Chat Template

**Version:** 0.1.0
**Category:** Chat
**Status:** Placeholder Implementation (Phase 12.4.1)

## Overview

The Interactive Chat template provides session-based conversation capabilities with memory persistence, tool integration, and context management. It enables building chatbots, virtual assistants, and interactive help systems.

### What It Does

- **Session Management**: Persistent conversation sessions across interactions
- **Context Awareness**: Maintains conversation context and history
- **Tool Integration**: Can invoke tools during conversation (web search, calculations, etc.)
- **Memory Persistence**: Remembers previous interactions within and across sessions

### Use Cases

- Customer support chatbots
- Interactive documentation assistants
- Virtual teaching assistants
- Technical support agents
- Conversational interfaces for complex systems

---

## Quick Start

### CLI - Basic Usage

```bash
llmspell template exec interactive-chat \
  --param message="Your message here"
```

### Lua - Basic Usage

```lua
local result = Template.execute("interactive-chat", {
    message = "Hello, how can I help?"
})

if result.result_type == "text" then
    print(result.result)
end
```

---

## Parameters

### Required Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `message` | String | User message or query |

### Optional Parameters

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `session_id` | String | Auto-generated | Session identifier for conversation persistence |
| `model` | String | `"ollama/llama3.2:3b"` | LLM model for responses |
| `enable_memory` | Boolean | `true` | Enable conversation memory |
| `tools` | Array | `[]` | List of tool names to make available |
| `max_history` | Integer | `10` | Maximum conversation history length |

**Inspect Full Schema:**
```bash
llmspell template schema interactive-chat
```

---

## Implementation Status

⚠️ **Note**: This template is a **placeholder implementation** as of Phase 12.4.1.

**Implemented:**
- ✅ Template metadata and parameter schema
- ✅ Parameter validation
- ✅ Cost estimation
- ✅ CLI and Lua integration

**Placeholder/Pending:**
- ⏳ Session management integration
- ⏳ Memory persistence
- ⏳ Tool invocation during chat
- ⏳ Context window management
- ⏳ Multi-turn conversation logic

**Expected**: Full implementation in Phase 14 (Advanced Templates)

---

## Output Format

### Programmatic Mode

When `message` parameter is provided, returns single response:

```json
{
  "result_type": "text",
  "result": "AI response to your message",
  "metrics": {
    "duration_ms": 1234,
    "tokens_used": 256,
    "agents_invoked": 1
  }
}
```

### Interactive Mode

When `message` parameter is omitted, enters interactive REPL:

```
> Hello, how are you?
AI: I'm doing well! How can I help you today?

> What can you do?
AI: I can help with various tasks including answering questions,
    providing information, and assisting with problem-solving.

> exit
Goodbye!
```

---

## Examples

### CLI Examples

#### Basic Chat
```bash
llmspell template exec interactive-chat \
  --param message="What is Rust programming?"
```

#### Chat with Tools
```bash
llmspell template exec interactive-chat \
  --param message="Search for Rust async patterns" \
  --param tools='["web-search","file-read"]'
```

#### Session-Based Chat
```bash
# First interaction
llmspell template exec interactive-chat \
  --param session_id="user123" \
  --param message="My name is Alice"

# Later interaction (remembers context)
llmspell template exec interactive-chat \
  --param session_id="user123" \
  --param message="What's my name?"
# AI: Your name is Alice!
```

### Lua Examples

#### Basic Chat
```lua
local result = Template.execute("interactive-chat", {
    message = "Explain async/await in Rust"
})

print(result.result)
```

#### Session Management
```lua
-- Start session
local session_id = "session-" .. os.time()

local response1 = Template.execute("interactive-chat", {
    session_id = session_id,
    message = "I'm working on a web server"
})

-- Continue session (AI remembers context)
local response2 = Template.execute("interactive-chat", {
    session_id = session_id,
    message = "What framework should I use?"
})
```

---

## Troubleshooting

### Common Issues

#### Error: "Required parameter missing: message"

**Solution**: Provide the `message` parameter:
```bash
--param message="Your question here"
```

#### Using Placeholder Implementation

**Current Behavior**: The template generates basic responses but doesn't yet implement full session management or tool integration.

**Workaround**: For production chat needs, consider:
1. Using Agent system directly with session management
2. Waiting for Phase 14 full implementation
3. Implementing custom chat template

---

## Related Documentation

- [Template System Overview](../templates/README.md)
- [Research Assistant Template](./research-assistant.md) (production example)
- [Session Management Guide](../../sessions/README.md)
- [Tool Integration](../../tools/README.md)

---

## Roadmap

### Phase 12.4.1 (Current)
- ✅ Basic template structure
- ✅ Parameter validation
- ⏳ Placeholder responses

### Phase 14 (Planned)
- Multi-turn conversation logic
- Session persistence
- Context window management
- Tool invocation during chat
- Conversation summarization

### Phase 15 (Future)
- Multi-agent collaboration in chat
- Advanced memory integration
- Personality customization
- Voice interface support

---

**Last Updated**: Phase 12.4.1 (Placeholder Implementation)
**Next Review**: Phase 14 (Advanced Templates)
