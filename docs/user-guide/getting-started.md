# Getting Started

**Version**: 0.6.0  
**Time Required**: 5 minutes

> **🚀 Quick Start**: Get LLMSpell running in under 5 minutes with this focused guide.

**🔗 Navigation**: [← User Guide](README.md) | [Core Concepts →](concepts.md) | [Examples →](../../examples/EXAMPLE-INDEX.md)

---

## Prerequisites

- Rust toolchain (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- At least one API key (OpenAI, Anthropic, etc.)
- Basic command line familiarity

## Installation

```bash
# Clone and build
git clone https://github.com/yourusername/rs-llmspell.git
cd rs-llmspell
cargo build --release

# Set API key
export OPENAI_API_KEY="sk-..."
```

## First Script

### Option 1: One-liner
```bash
./target/release/llmspell exec '
  local agent = Agent.builder():model("openai/gpt-4o-mini"):build()
  print(agent:execute({prompt = "Hello, LLMSpell!"}).response)
'
```

### Option 2: Script File
Create `hello.lua`:
```lua
-- hello.lua
local agent = Agent.builder()
    :name("assistant")
    :model("openai/gpt-4o-mini")
    :system_prompt("You are a helpful assistant")
    :temperature(0.7)
    :build()

local input = ARGS[1] or "Tell me an interesting fact"
local response = agent:execute({prompt = input})

print("Assistant:", response.response)
print("Tokens:", response.usage.total_tokens)
```

Run it:
```bash
./target/release/llmspell run hello.lua "What's the weather like?"
```

## Core Patterns

### 1. Creating Agents
```lua
local agent = Agent.builder()
    :model("openai/gpt-4o-mini")  -- or "anthropic/claude-3-haiku"
    :temperature(0.7)
    :max_tokens(500)
    :build()
```

### 2. Using Tools
```lua
-- List available tools
for i, tool in ipairs(Tool.list()) do
    print(i, tool)
end

-- Use a tool
local result = Tool.invoke("web-search", {
    query = "LLMSpell tutorial",
    max_results = 5
})
```

### 3. Building Workflows
```lua
local workflow = Workflow.sequential({
    name = "my_workflow",
    steps = {
        {name = "search", tool = "web-search", input = {query = "AI news"}},
        {name = "summarize", agent = agent, prompt = "Summarize: $search"}
    }
})

local result = workflow:execute()
print(result.summarize)
```

### 4. Managing State
```lua
-- Store data
State.set("counter", 0)
State.set("config", {theme = "dark", lang = "en"})

-- Retrieve data
local count = State.get("counter")
local config = State.get("config")
```

## Command Line Usage

```bash
# Execute inline code
./llmspell exec 'print("Hello")'

# Run a script
./llmspell run script.lua

# Pass arguments
./llmspell run script.lua arg1 arg2

# Use configuration file
./llmspell -c config.toml run script.lua

# Enable debug output
RUST_LOG=debug ./llmspell run script.lua
```

## Quick Examples

### Chat Interface
```lua
-- chat.lua
local agent = Agent.builder():model("openai/gpt-4o-mini"):build()
while true do
    io.write("> ")
    local input = io.read()
    if input == "exit" then break end
    local response = agent:execute({prompt = input})
    print("AI:", response.response)
end
```

### File Processor
```lua
-- process.lua
local content = Tool.invoke("file-operations", {
    operation = "read",
    path = ARGS[1] or "input.txt"
})

local agent = Agent.builder():model("openai/gpt-4o-mini"):build()
local result = agent:execute({
    prompt = "Summarize this text:\n" .. content.content
})

Tool.invoke("file-operations", {
    operation = "write",
    path = "summary.txt",
    content = result.response
})
```

## What's Next?

1. **Understand the architecture** → [Core Concepts](concepts.md)
2. **Explore examples** → [Example Index](../../examples/EXAMPLE-INDEX.md)
3. **Configure providers** → [Configuration](configuration.md)
4. **Learn the APIs** → [Lua API](api/lua/README.md)
5. **Debug issues** → [Troubleshooting](troubleshooting.md)

## Quick Tips

- All globals are pre-injected (no `require()` needed)
- Use `-c` flag for config files, not environment variables
- Check `Tool.list()` to discover available tools
- Use `Debug.setLevel("debug")` for verbose output
- Scripts timeout after 5 minutes by default

---

**Need help?** Check [Troubleshooting](troubleshooting.md) or report issues on [GitHub](https://github.com/yourusername/rs-llmspell/issues)