# Interactive Chat Template

**Version:** 0.2.0
**Category:** Chat
**Status:** Production Ready (Phase 12.9 - Full REPL Integration)

## Overview

Production-ready conversational AI template with two execution modes: **interactive REPL** for human conversation and **programmatic single-message** mode for scripting. Features full session management, conversation history persistence, optional tool integration, and context-aware multi-turn dialog.

### What It Does

- **Dual Execution Modes**: Interactive stdin loop (REPL) OR programmatic single-message
- **Session-Based History**: Automatic conversation persistence across turns in `Session.state["conversation_history"]`
- **Tool Integration**: Optional tools validated via ToolRegistry and passed to LLM agent
- **Multi-Model Support**: Local (Ollama) and remote (Anthropic, OpenAI) LLMs via `provider/model-id` format
- **Context Management**: Full conversation history with turn tracking and timestamps

### Use Cases

- Interactive CLI chatbots and REPL interfaces
- Scripted single-message Q&A automation
- Customer support agents with tool access (calculator, web-searcher, etc.)
- Technical documentation assistants
- Multi-turn conversational workflows with session persistence

---

## Quick Start

### CLI - Interactive Mode (Full REPL)

```bash
# Omit 'message' parameter to enter production-grade REPL
./target/debug/llmspell template exec interactive-chat

# With custom model and tools
./target/debug/llmspell template exec interactive-chat \
  --param model=ollama/mistral:7b \
  --param tools='["calculator", "web-searcher"]'
```

**REPL Features** (Phase 12.9):
- ✅ **Readline**: Arrow keys (↑↓), Ctrl-A/E, history navigation
- ✅ **Multi-line**: Smart detection with `...` continuation prompts
- ✅ **Ctrl-C**: Graceful interrupt (doesn't exit REPL)
- ✅ **Commands**: `.system`, `.model`, `.tools`, `.context`, `.clearchat`, `.info`
- ✅ **History**: Persistent across sessions (~/.cache/llmspell_chat_history_*)

### CLI - Programmatic Mode (Single Message)

```bash
# Provide 'message' parameter for single-shot response
./target/debug/llmspell template exec interactive-chat \
  --param message="Explain Rust lifetimes in 3 sentences"

# With remote LLM
./target/debug/llmspell template exec interactive-chat \
  --param model=anthropic/claude-3-7-sonnet-latest \
  --param message="What is dependency injection?"
```

### CLI - With Memory and Provider

Enable memory-enhanced execution with custom provider:

```bash
llmspell template exec interactive-chat \
  --param message="Explain Rust lifetimes in 3 sentences" \
  --param session-id="user-session-123" \
  --param memory-enabled=true \
  --param context-budget=3000 \
  --param provider-name="ollama"
```

### Lua - Programmatic Usage

```lua
local result = Template.execute("interactive-chat", {
    message = "What is dependency injection?",
    model = "ollama/llama3.2:3b"
})

print(result.result)
```

### Lua - With Memory and Provider

Enable memory-enhanced execution:

```lua
local result = Template.execute("interactive-chat", {
    message = "Explain Rust lifetimes in 3 sentences",
    session_id = "user-session-123",
    memory_enabled = true,
    context_budget = 3000,
    provider_name = "ollama"
})
```

---

## Parameters

**All parameters are optional.** Execution mode determined by `message` parameter presence.

| Parameter | Type | Default | Description |
|-----------|------|---------|-------------|
| `message` | String | (none) | **MODE SELECTOR**: If present → programmatic mode (single response); if absent → interactive mode (REPL) |
| `model` | String | `"ollama/llama3.2:3b"` | LLM model specification. Format: `provider/model-id`<br>Examples: `ollama/llama3.2:3b`, `anthropic/claude-3-7-sonnet-latest`, `openai/gpt-5-mini` |
| `system_prompt` | String | `"You are a helpful AI assistant..."` | System instructions defining AI behavior and personality |
| `max_turns` | Integer | `10` | Maximum conversation turns (range: 1-100). Enforced in interactive mode only |
| `tools` | Array | `[]` | Tool names to enable (e.g., `["calculator", "web-searcher"]`). Tools validated via ToolRegistry before agent creation |
| `session_id` | String | (none) | Optional session UUID for reusing existing conversation. Enables multi-turn context across separate CLI invocations. If not provided, creates new session. See Example 4 for session reuse workflow |
| `enable_memory` | Boolean | `false` | Long-term memory integration (Phase 13 placeholder - not yet active) |

### Memory Parameters

All templates support optional memory integration for context-aware execution:

| Parameter | Type | Default | Range/Values | Description |
|-----------|------|---------|--------------|-------------|
| `session_id` | String | `null` | Any string | Session identifier for conversation memory filtering |
| `memory_enabled` | Boolean | `true` | `true`, `false` | Enable memory-enhanced execution (uses episodic + semantic memory) |
| `context_budget` | Integer | `2000` | `100-8000` | Token budget for context assembly (higher = more context) |

**Memory Integration**: When `session_id` is provided and `memory_enabled` is `true`, the template will:
- Retrieve relevant episodic memory from conversation history
- Query semantic memory for related concepts
- Assemble context within the `context_budget` token limit
- Provide memory-enhanced context to LLM for better results

### Provider Parameters

Templates support dual-path provider resolution:

| Parameter | Type | Default | Range/Values | Description |
|-----------|------|---------|--------------|-------------|
| `provider_name` | String | `null` | `"ollama"`, `"openai"`, etc. | Provider name (mutually exclusive with `model`) |

**Provider Resolution**:
- Use `provider_name` to select a provider with its default model (e.g., `provider_name: "ollama"`)
- Use `model` for explicit model selection (e.g., `model: "gpt-4"`)
- If both provided, `model` takes precedence
- `provider_name` and `model` are mutually exclusive

**Inspect Full Schema:**
```bash
./target/debug/llmspell template schema interactive-chat
```

---

## Implementation Status

✅ **Fully Implemented** (Phase 12.9 - Full REPL Integration)

**Production Features:**
- ✅ **Dual execution modes** (production REPL + programmatic single-message)
- ✅ **Full REPL Integration** (Phase 12.9):
  - Readline support (arrow keys, Ctrl-A/E, history navigation)
  - Multi-line input with smart continuation detection
  - Ctrl-C graceful interrupt (doesn't exit REPL)
  - Persistent command history across sessions
  - Tab completion for chat commands
- ✅ **Chat Commands** (Phase 12.9):
  - `.system` - Runtime system prompt updates
  - `.model` - Dynamic model switching with agent auto-creation
  - `.tools` - Runtime tool configuration
  - `.context` - Conversation history display
  - `.clearchat` - Clear conversation without losing session
  - `.info` - Comprehensive session diagnostics (4 info sections)
- ✅ Session management with UUID-based identifiers (auto-created)
- ✅ Conversation history persistence in `Session.state["conversation_history"]`
- ✅ Tool validation and integration via ToolRegistry
- ✅ Multi-turn context management with history serialization
- ✅ Timeout enforcement (120s per chat response - Phase 12.8.2.7)
- ✅ Conversation artifacts saved to output directory
- ✅ Cost estimation and metrics tracking (tokens, duration, turns)
- ✅ Model specification parsing (`provider/model-id` format)
- ✅ Agent auto-creation callback pattern (dependency inversion)

**Placeholder (Future):**
- ⏳ Long-term memory (`enable_memory` flag - Phase 13 A-TKG integration)
- ⏳ Code execution in REPL (currently chat-only via NoOpScriptExecutor)

---

## Execution Modes

### Interactive Mode (Production-Grade REPL)

**Activation**: Omit the `message` parameter

**REPL Features** (Phase 12.9):
- **Readline Integration**: Full terminal editing support
  - Arrow keys (↑↓) for history navigation
  - Ctrl-A (home), Ctrl-E (end) for line navigation
  - Ctrl-K (kill to end), Ctrl-U (kill to start)
  - Tab completion for commands
- **Multi-line Input**: Smart detection with continuation prompts
- **Ctrl-C Handling**: Graceful interrupt without exiting REPL
- **Persistent History**: Saved to `~/.cache/llmspell_chat_history_{session_id}`
- **Chat Commands**: Meta commands for runtime configuration

**REPL Commands**:
- **Chat Control**:
  - `.system "prompt"` - Update system prompt
  - `.model provider/model` - Switch LLM model (auto-creates agent)
  - `.tools tool1,tool2` - Configure available tools
  - `.context` - Show conversation history
  - `.clearchat` - Clear conversation history
  - `.info` - Display session information (config, infrastructure, chat state)
- **Session Control**:
  - `.exit` or `.quit` - End conversation gracefully
  - `.help` - Show all available commands

**Example Session**:
```
╔══════════════════════════════════════════════╗
║   Interactive REPL Chat Session Started     ║
╚══════════════════════════════════════════════╝

Model: ollama/llama3.2:3b
Session: 550e8400-e29b-41d4-a716-446655440000

📝 Chat Commands:
  • Type your message to chat with the AI
  • .exit or .quit - end the conversation
  • .system "prompt" - change system prompt
  • .model provider/model - change LLM model
  • .tools tool1,tool2 - configure available tools
  • .context - show conversation history
  • .clearchat - clear conversation history
  • .info - display session information

💬 Chat Mode:
  • Type naturally - What is the capital of France?
  • Multi-turn conversations with context retention
  • Tool integration (if enabled)

✨ REPL Features: Arrow keys, history (↑↓), multi-line, Ctrl-C interrupt

You> What is Rust?
Assistant> Rust is a systems programming language focused on safety, concurrency, and performance...

You> .info
📊 Session Information:
────────────────────────────────────────────────────────────
📅 Session:
  Uptime: 45.3s
  Executions: 1
  History entries: 2
  Variables: 0
  Breakpoints: 0
  Debug mode: false

⚙️ Configuration:
  Execution timeout: 300s
  Performance monitoring: enabled
  Debug commands: disabled
  Session persistence: true
  History file: /Users/user/.cache/llmspell_chat_history_550e8400...

🔧 Script Executor:
  Language: none

🏗️ Infrastructure:
  Session manager: enabled
  Hooks: enabled
  Provider manager: enabled
  Agent registry: enabled
  RAG system: disabled

💬 Chat Mode:
  Model: ollama/llama3.2:3b
  System prompt: You are a helpful AI assistant. Provide cle...
  Agent: initialized
  Tools: none
  Conversation turns: 1
  Total tokens: 428

You> .exit
[Ending conversation]

╔══════════════════════════════════════════════╗
║    Interactive REPL Chat Session Ended      ║
╚══════════════════════════════════════════════╝
Total turns: 1
Total tokens (estimated): 428
```

### Programmatic Mode (Single Message)

**Activation**: Provide the `message` parameter

**Behavior**:
- Loads conversation history from session (if exists)
- Adds user message to history
- Calls LLM agent with system prompt + conversation context
- Adds assistant response to history
- Saves updated history to session
- Returns single response and exits

**Use Case**: Scripting, automation, single Q&A interactions

---

## Output Format

### Programmatic Mode Output

```json
{
  "result_type": "text",
  "result": "# Chat Conversation\n\nUser: What is Rust?\n\nAssistant: Rust is a systems programming language...",
  "metrics": {
    "duration_ms": 1234,
    "tokens_used": 428,
    "agents_invoked": 1,
    "tools_invoked": 0,
    "turn_count": 1,
    "total_tokens": 428,
    "session_id": "550e8400-e29b-41d4-a716-446655440000"
  },
  "artifacts": [
    {
      "path": "/output/conversation-550e8400-e29b-41d4-a716-446655440000.txt",
      "content": "# Chat Conversation\n...",
      "mime_type": "text/plain"
    }
  ]
}
```

### Interactive Mode Output

Returns after conversation ends with full transcript:

```json
{
  "result_type": "text",
  "result": "# Interactive Chat Session\n\n**Turn 1:**\n\nUser: What is Rust?\n\nAssistant: Rust is...\n\n**Turn 2:**\n...",
  "metrics": {
    "duration_ms": 45230,
    "turn_count": 5,
    "total_tokens": 2140,
    "session_id": "550e8400-e29b-41d4-a716-446655440000"
  }
}
```

---

## Examples

### CLI Examples

#### 1. Local LLM - Interactive REPL
```bash
# Default: Ollama with llama3.2:3b, interactive mode
./target/debug/llmspell template exec interactive-chat

# Custom local model with personality
./target/debug/llmspell template exec interactive-chat \
  --param model=ollama/mistral:7b \
  --param system_prompt="You are a technical documentation expert specializing in Rust." \
  --param max_turns=20
```

#### 2. Remote LLM - Programmatic Mode
```bash
# Anthropic Claude - single message
./target/debug/llmspell template exec interactive-chat \
  --param model=anthropic/claude-3-7-sonnet-latest \
  --param message="Explain Rust lifetimes in 3 sentences"

# OpenAI GPT-5-mini - programmatic with custom prompt
./target/debug/llmspell template exec interactive-chat \
  --param model=openai/gpt-5-mini \
  --param message="Write a haiku about Rust compilation times" \
  --param system_prompt="You are a poetic systems programmer"
```

#### 3. With Tools Integration
```bash
# Interactive chat with calculator and web-searcher
./target/debug/llmspell template exec interactive-chat \
  --param model=ollama/llama3.2:3b \
  --param tools='["calculator", "web-searcher"]'

# Programmatic with tools
./target/debug/llmspell template exec interactive-chat \
  --param model=anthropic/claude-3-7-sonnet-latest \
  --param message="What is 15 * 847 + 23? Use calculator tool." \
  --param tools='["calculator"]'
```

#### 4. Session Reuse - Multi-Turn Context Across CLI Invocations
```bash
# First call: Introduce yourself and capture session_id from JSON output
./target/debug/llmspell template exec interactive-chat \
  --param model=ollama/llama3.2:3b \
  --param message="My name is Alice" \
  --output json > /tmp/chat1.json

# Extract session_id from output
SESSION_ID=$(jq -r '.metrics.custom_metrics.session_id' /tmp/chat1.json)
echo "Session ID: $SESSION_ID"

# Second call: Reuse the session to test conversation history
./target/debug/llmspell template exec interactive-chat \
  --param model=ollama/llama3.2:3b \
  --param message="What is my name?" \
  --param session_id="$SESSION_ID" \
  --output json

# Output: "Nice to see you again, Alice!"
# ✓ Session persistence working - conversation history maintained across CLI calls
```

**How it works:**
- Sessions are persisted to `./sessions/` using SledBackend (embedded database)
- Session state includes full conversation history
- `session_id` parameter enables context continuity across separate executions
- Useful for stateful CLI workflows and automation scripts

### Lua Examples

#### Basic Chat
```lua
local result = Template.execute("interactive-chat", {
    message = "Explain async/await in Rust"
})

print(result.result)
```

#### Multi-Turn Conversation
```lua
-- Interactive-chat automatically manages session across calls
local turn1 = Template.execute("interactive-chat", {
    message = "I'm building a web server in Rust"
})

local session_id = turn1.metrics.session_id

-- Reuse session for context continuity
local turn2 = Template.execute("interactive-chat", {
    message = "What framework should I use?",
    -- Session ID extracted from turn1, conversation history maintained
})
```

#### With Tools
```lua
local result = Template.execute("interactive-chat", {
    message = "Calculate 25 * 17 and search for Rust async tutorials",
    tools = {"calculator", "web-searcher"}
})

print(result.result)
```

---

## Session Management

### Session Creation & Persistence

**Default Behavior**: Sessions are **automatically created** with UUID identifiers.

**Session Reuse** (v0.11.2+): Provide `session_id` parameter to reuse existing conversation:
- Sessions persisted to `./sessions/` directory (SledBackend embedded database)
- Conversation history and state maintained across separate CLI invocations
- Enables stateful workflows and automation scripts
- See **Example 4** for session reuse workflow

**Session State Storage**:
- `conversation_history`: Array of `ConversationTurn` objects
- `conversation_metrics`: Turn count, token usage, last updated timestamp

**ConversationTurn Schema**:
```json
{
  "role": "user" | "assistant",
  "content": "Message text",
  "timestamp": "2024-01-15T10:30:00Z",
  "turn_number": 1,
  "token_count": null
}
```

### History Persistence

**Storage Location**:
- In-session: `Session.state["conversation_history"]`
- On-disk: `./sessions/` (MessagePack format with optional LZ4 compression)

**Behavior**:
- **Programmatic mode**: Loads history → adds user message → calls LLM → adds assistant response → saves history
- **Interactive mode**: Each turn updates history via programmatic mode logic
- **History command**: Displays all turns with colored output
- **Session reuse**: Auto-loads conversation history when `session_id` provided
- **Persistence**: Immediate save on session creation + periodic auto-persist (300s interval)

---

## Technical Details

### Architecture

1. **Phase 1**: Create/restore session (UUID-based, auto-generated)
2. **Phase 2**: Validate tools via ToolRegistry (fails if tool not found)
3. **Phase 3**: Check memory flag (placeholder, logs warning if enabled)
4. **Phase 4**: Execute conversation (interactive loop OR single programmatic call)
5. **Phase 5**: Save session state (history + metrics)

### Agent Configuration

**Chat Agent Config** (llmspell-agents/src/agents/llm.rs):
```rust
AgentConfig {
    name: "chat-agent-{session_id}",
    agent_type: "llm",
    model: ModelConfig {
        provider: "ollama" | "anthropic" | "openai",
        model_id: "llama3.2:3b" | "claude-3-7-sonnet-latest" | "gpt-5-mini",
        temperature: 0.7,
        max_tokens: 1000,
    },
    allowed_tools: tools,
    resource_limits: ResourceLimits {
        max_execution_time_secs: 120,  // 2 minutes (Phase 12.8.2.7)
        max_memory_mb: 256,
        max_tool_calls: 10,
        max_recursion_depth: 1,
    },
}
```

### Timeout Architecture (Phase 12.8.2.7)

**Reverse Pyramid Pattern**:
- Provider timeout: `None` (no default, opt-in per call)
- Agent timeout: `120s` (ResourceLimits enforcement via tokio::timeout)
- Kernel timeout: `900s` (integrated execution wrapper)

**Chat Response Timeout**: 120 seconds enforced at agent level

### Model Specification Format

**Syntax**: `provider/model-id`

**Supported Providers**:
- `ollama/llama3.2:3b`, `ollama/mistral:7b`, `ollama/codellama:13b`
- `anthropic/claude-3-7-sonnet-latest`, `anthropic/claude-3-5-haiku-latest`
- `openai/gpt-5-mini`, `openai/gpt-4`, `openai/gpt-4-turbo`

**Default Provider**: If no slash (`/`) in model string, defaults to `ollama`

---

## Troubleshooting

### Common Issues

#### "Tool not found: calculator"

**Cause**: Tool not registered in ToolRegistry

**Solution**: Ensure tool is available via:
```bash
./target/debug/llmspell tool list
```

#### Interactive mode not starting

**Cause**: `message` parameter provided

**Solution**: Omit `message` parameter entirely for interactive mode:
```bash
# Wrong (enters programmatic mode)
--param message=""

# Right (enters interactive mode)
# Just omit the parameter
```

#### Session history not persisting across separate CLI calls

**Cause**: Not providing `session_id` parameter to reuse existing session

**Solution**: Extract `session_id` from first call's JSON output and provide it in subsequent calls:

```bash
# First call - creates new session
SESSION_ID=$(./target/debug/llmspell template exec interactive-chat \
  --param message="My name is Alice" \
  --output json | jq -r '.metrics.custom_metrics.session_id')

# Second call - reuses session
./target/debug/llmspell template exec interactive-chat \
  --param message="What is my name?" \
  --param session_id="$SESSION_ID"
```

See **Example 4** for complete session reuse workflow. Sessions automatically persist to `./sessions/` directory using SledBackend.

### REPL-Specific Issues (Phase 12.9)

#### "Agent not available" error after `.model` command

**Cause**: Agent auto-creation failed (provider not configured or model not found)

**Solution**:
1. Check provider configuration:
   ```bash
   ./target/debug/llmspell provider list
   ```
2. Verify model exists:
   ```bash
   ollama list  # For Ollama models
   ```
3. Check `.info` output to see agent status

#### Arrow keys not working in REPL

**Cause**: Terminal doesn't support readline or TERM environment variable not set

**Solution**:
```bash
# Check TERM variable
echo $TERM

# Set if missing
export TERM=xterm-256color

# If still doesn't work, check terminal emulator supports readline
```

#### Multi-line input not detecting continuations

**Cause**: Chat-only mode uses NoOpScriptExecutor (no code execution)

**Note**: Multi-line input is primarily for code execution. In chat-only mode, each line is sent as a message. This is expected behavior.

#### Ctrl-C exits REPL instead of interrupting

**Cause**: Old version (pre-12.9) or terminal signal handling issue

**Solution**:
1. Verify version: `./target/debug/llmspell --version` should show ≥0.13.0
2. Check REPL is using production integration (Phase 12.9)
3. If issue persists, report as bug with terminal type

#### `.info` command shows "Agent: not available"

**Cause**: Agent hasn't been created yet (no messages sent)

**Solution**: Send at least one chat message to trigger agent creation, or use `.model` command to explicitly create agent

#### Conversation history lost after `.clearchat`

**Expected Behavior**: `.clearchat` clears in-memory conversation history. This is intentional.

**Note**: Session state is preserved. Only the conversation turns are cleared. Use this to start fresh topic without losing session configuration.

---

## Related Documentation

- [Template System Overview](../templates/README.md)
- [Research Assistant Template](./research-assistant.md) (production example)
- [Session Management Guide](../../sessions/README.md)
- [Tool Integration](../../tools/README.md)
- [Agent Configuration](../agents/README.md)

---

## Roadmap

### Phase 12.9 (Current - Complete ✅)
- ✅ Production-grade REPL with readline integration
- ✅ Full terminal editing (arrow keys, Ctrl-A/E, history)
- ✅ Multi-line input with smart continuation
- ✅ Ctrl-C graceful interrupt handling
- ✅ Chat commands (`.system`, `.model`, `.tools`, `.context`, `.clearchat`, `.info`)
- ✅ Agent auto-creation callback pattern
- ✅ Comprehensive session diagnostics via `.info`
- ✅ Backward compatibility (programmatic mode unchanged)

### Phase 12.10 (Future - Dual-Mode REPL)
- Code execution integration (Lua/JS in same REPL)
- Smart input detection (code vs chat)
- Shared variable context between code and chat
- `.chat` explicit command for forcing chat mode

### Phase 13 (Planned - A-TKG)
- Long-term memory via `enable_memory` flag
- Temporal knowledge graph integration
- Cross-session memory retrieval
- Conversation summarization for context window management

### Phase 14 (Future)
- Multi-agent collaboration in chat
- Advanced personality customization
- Voice input/output integration

---

**Last Updated**: Phase 12.9 (Full REPL Integration)
**Implementation**: `llmspell-templates/src/builtin/interactive_chat.rs:1-1419`
**Kernel Integration**: `llmspell-kernel/src/repl/session.rs` (chat commands + .info)
