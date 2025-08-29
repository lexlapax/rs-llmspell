# Getting Started

**Version**: 0.8.10  
**Time Required**: 10 minutes

> **🚀 Quick Start**: Get LLMSpell running in under 5 minutes with this focused guide.

**🔗 Navigation**: [← User Guide](README.md) | [Core Concepts →](concepts.md) | [Examples →](../../examples/EXAMPLE-INDEX.md)

---

## Prerequisites

- Rust toolchain (`curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`)
- At least one API key (OpenAI recommended for RAG features)
- Basic command line familiarity
- 10+ minutes for full RAG walkthrough

## Installation

```bash
# Clone and build
git clone https://github.com/yourusername/rs-llmspell.git
cd rs-llmspell
cargo build --release

# Set API key (required for agents and RAG)
export OPENAI_API_KEY="sk-..."

# Verify installation
./target/release/llmspell exec 'print("LLMSpell " .. tostring(_G._VERSION or "0.8.10") .. " ready!")'
```

## First Script

### Option 1: One-liner (Basic)
```bash
./target/release/llmspell exec 'print("Hello from LLMSpell Phase 8.10.6!")'
```

### Option 2: Agent One-liner (Requires API key)
```bash
./target/release/llmspell exec '
  local agent = Agent.builder():model("openai/gpt-4o-mini"):build()
  local result = agent:execute({prompt = "Hello, LLMSpell!"})
  print(result.content or result.response or result)
'
```

### Option 3: Script File (Recommended)
Create `hello.lua`:
```lua
-- hello.lua
-- Create agent with error handling
local agent_result = Agent.builder()
    .provider("openai")  -- or use Provider.list()[1]
    .system_prompt("You are a helpful assistant")
    .build()

if not agent_result.success then
    print("❌ Error: " .. (agent_result.error or "Failed to create agent"))
    return
end

local agent = agent_result.result
local input = ARGS[1] or "Tell me an interesting fact"
local response = agent:invoke(input)

if response.success then
    print("🤖 Assistant:", response.result.content)
else
    print("❌ Error:", response.error or "Unknown error")
end
```

Run it:
```bash
# Needs provider configuration
./target/release/llmspell -c examples/script-users/configs/example-providers.toml run hello.lua
```

## Core Patterns

### 1. Creating Agents (Phase 8.10.6)
```lua
-- New pattern with explicit error handling
local agent_result = Agent.builder()
    .provider("openai")           -- Use Provider.list() to see available
    .system_prompt("You are helpful")
    .build()

if agent_result.success then
    local agent = agent_result.result
    local response = agent:invoke("Hello!")
    print(response.result.content)
end
```

### 2. Using Tools (40+ Available)
```lua
-- List available tools  
for i, tool in ipairs(Tool.list()) do
    print(i, tool.name or tool)
end

-- File operations
local result = Tool.invoke("file-operations", {
    operation = "read",
    path = "document.txt"
})

-- Web search
local search = Tool.invoke("web-search", {
    query = "LLMSpell Phase 8",
    provider = "duckduckgo"
})
```

### 3. RAG (Retrieval-Augmented Generation) ⭐ **NEW**
```lua
-- Ingest documents into knowledge base
RAG.ingest({
    content = "LLMSpell is a scriptable AI framework...",
    metadata = {
        source = "docs.md",
        category = "documentation"
    }
})

-- Search for relevant context
local results = RAG.search("How does LLMSpell work?", {
    limit = 3,
    threshold = 0.7
})

-- Use with agent for augmented responses
for _, result in ipairs(results.results) do
    print("Score:", result.score, "Content:", result.content)
end
```

### 4. Building Workflows
```lua
local workflow = Workflow.builder()
    :name("analysis_workflow")
    :sequential()
    :add_step({
        name = "fetch",
        type = "tool",
        tool = "web-scraper",
        input = {url = "https://example.com"}
    })
    :add_step({
        name = "analyze",
        type = "agent", 
        agent = agent,
        input = "Summarize: $fetch"
    })
    :build()

local result = workflow:execute({})
```

### 5. Managing State (Scoped)
```lua
-- State requires scope parameter in Phase 8.10.6
State.save("global", "counter", 0)
State.save("user", "preferences", {theme = "dark"})

-- Retrieve with scope
local count = State.load("global", "counter") 
local prefs = State.load("user", "preferences")

-- List available scopes
local scopes = State.list_scopes()
```

## Command Line Usage

```bash
# Execute inline code
./target/release/llmspell exec 'print("Hello from LLMSpell!")'

# Run a script  
./target/release/llmspell run script.lua

# Pass arguments
./target/release/llmspell run script.lua arg1 arg2

# Use configuration file (required for agents and RAG)
./target/release/llmspell -c config.toml run script.lua

# RAG-specific configuration
./target/release/llmspell -c examples/script-users/configs/rag-basic.toml run rag-script.lua

# Enable debug output
RUST_LOG=debug ./target/release/llmspell run script.lua

# Validate configuration
./target/release/llmspell -c config.toml validate
```

## Progressive Learning Path

Follow the getting-started examples in order for best results:

```bash
# 1. Hello World (2 seconds)
./target/release/llmspell run examples/script-users/getting-started/00-hello-world.lua

# 2. First Tool (5 seconds) 
./target/release/llmspell run examples/script-users/getting-started/01-first-tool.lua

# 3. First Agent (10 seconds, needs config)
./target/release/llmspell -c examples/script-users/configs/example-providers.toml \
  run examples/script-users/getting-started/02-first-agent.lua

# 4. First Workflow (20 seconds)
./target/release/llmspell run examples/script-users/getting-started/03-first-workflow.lua

# 5. Error Handling (5 seconds)
./target/release/llmspell run examples/script-users/getting-started/04-handle-errors.lua

# 6. First RAG System (15 seconds, needs RAG config)
./target/release/llmspell -c examples/script-users/configs/rag-basic.toml \
  run examples/script-users/getting-started/05-first-rag.lua
```

## Quick Examples

### RAG Knowledge Base ⭐ **NEW**
```lua
-- rag-demo.lua (requires rag-basic.toml config)
-- Ingest documents
RAG.ingest({
    content = "Rust is a systems programming language focused on safety and performance.",
    metadata = {source = "rust-guide.md"}
})

-- Search and get augmented response
local results = RAG.search("What is Rust good for?")
for _, result in ipairs(results.results) do
    print("📄 [" .. result.score .. "] " .. result.content)
end
```

### Chat Interface with Error Handling
```lua
-- chat.lua (requires example-providers.toml config)
local agent_result = Agent.builder().provider("openai").build()
if not agent_result.success then
    print("❌ Setup required: configure API key in config file")
    return
end

local agent = agent_result.result
while true do
    io.write("🤖 > ")
    local input = io.read()
    if input == "exit" then break end
    
    local response = agent:invoke(input)
    if response.success then
        print("AI: " .. response.result.content)
    else
        print("❌ Error: " .. response.error)
    end
end
```

### File Processor with Validation
```lua
-- process.lua
local file_path = ARGS[1] or "input.txt"

-- Read file with error handling
local content = Tool.invoke("file-operations", {
    operation = "read",
    path = file_path
})

if not content or not content.content then
    print("❌ Could not read file: " .. file_path)
    return
end

print("✅ Read " .. #content.content .. " characters from " .. file_path)

-- Process with agent (if available)
local providers = Provider.list()
if #providers > 0 then
    local agent_result = Agent.builder().provider(providers[1]).build()
    if agent_result.success then
        local response = agent_result.result:invoke("Summarize: " .. content.content)
        if response.success then
            -- Save summary
            Tool.invoke("file-operations", {
                operation = "write",
                path = "summary.txt",
                content = response.result.content
            })
            print("✅ Summary saved to summary.txt")
        end
    end
else
    print("⚠️ No providers configured - skipping AI processing")
end
```

## What's Next?

### Immediate Next Steps
1. **Try the 6 progressive examples** → `examples/script-users/getting-started/`
2. **Configure your providers** → [Configuration Guide](configuration.md)
3. **Set up RAG (Phase 8.10.6)** → Use `configs/rag-basic.toml`

### Deeper Learning 
4. **Understand the architecture** → [Core Concepts](concepts.md) (RAG, HNSW, Multi-tenancy)
5. **Production patterns** → [Cookbook](../../examples/script-users/cookbook/) (11 examples)
6. **Real applications** → [Applications](../../examples/script-users/applications/) (9 complete apps)
7. **Complete API reference** → [Lua API](api/lua/README.md) (17+ globals)

### Advanced Topics
8. **Multi-tenant RAG** → `configs/rag-multi-tenant.toml` 
9. **Vector storage optimization** → [RAG Configuration](configuration.md#rag-configuration)
10. **Debug and troubleshoot** → [Troubleshooting](troubleshooting.md)

## Quick Tips

- **17+ globals pre-injected** (no `require()` needed): `Agent`, `Tool`, `RAG`, `State`, etc.
- **Configuration required** for agents and RAG (use `-c config.toml`)
- **Provider setup**: Add API keys to config files, not environment variables
- **RAG setup**: Use `configs/rag-basic.toml` for document search and ingestion
- **Tool discovery**: `Tool.list()` shows all 37+ available tools
- **Error handling**: Always check `.success` field on agent/tool results
- **Debug mode**: `RUST_LOG=debug` for verbose output
- **Timeouts**: Scripts timeout after 5 minutes by default
- **State scoping**: State operations require scope parameter (`"global"`, `"user"`, etc.)

### Phase 8.10.6 Specific
- **RAG requires OpenAI**: Set up OpenAI provider for embedding generation
- **Vector dimensions**: Default 384, supports 768, 1536, 3072
- **HNSW performance**: <10ms search for 1M vectors with proper configuration
- **Multi-tenancy**: Use `multi_tenant = true` in config for isolation

---

## Quick Start Checklist

- [ ] ✅ Install Rust and build LLMSpell (`cargo build --release`)
- [ ] 🔑 Set up OpenAI API key (`export OPENAI_API_KEY="sk-..."`)  
- [ ] 📂 Try hello world (`./target/release/llmspell exec 'print("Hello!")'`)
- [ ] 🤖 Create first agent with config file
- [ ] 📚 Try RAG example with `rag-basic.toml`
- [ ] 🛠️ Explore 37+ tools with `Tool.list()`
- [ ] 🔄 Build your first workflow
- [ ] 📖 Read [Core Concepts](concepts.md) to understand RAG and HNSW

**Phase 8.10.6 Ready!** RAG, vector search, and multi-tenancy at your fingertips.

---

**Need help?** Check [Troubleshooting](troubleshooting.md) or [Configuration Guide](configuration.md) for RAG setup. Report issues on [GitHub](https://github.com/yourusername/rs-llmspell/issues)