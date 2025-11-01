# Getting Started

**Version**: 0.13.0
**Time Required**: 10 minutes (or <5 minutes with AI Agent Templates!)

> **🚀 Quick Start**: Get LLMSpell kernel running in under 5 minutes with this focused guide. **NEW in Phase 12**: Use experimental workflows for rapid exploration AI workflow templates for instant productivity!

**🔗 Navigation**: [← User Guide](README.md) | [Core Concepts →](concepts.md) | [Service Deployment →](service-deployment.md)

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

# Optional: Relax security for development/learning (NOT for production)
export LLMSPELL_ALLOW_FILE_ACCESS="true"
export LLMSPELL_ALLOW_NETWORK_ACCESS="true"
export LLMSPELL_TOOLS_ALLOWED_PATHS="/tmp,/workspace"
# See docs/user-guide/security-and-permissions.md for production security

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

### Option 4: Template One-liner ⭐ **Phase 12** (Fastest!)
```bash
# Execute research assistant template
./target/release/llmspell template exec research-assistant \
    --param topic="Rust async programming" \
    --param max_sources=5 \
    --param model="openai/gpt-4o-mini" \
    --output text

# Execute code generator template
./target/release/llmspell template exec code-generator \
    --param description="A function to calculate fibonacci numbers" \
    --param language="rust" \
    --param model="openai/gpt-4o-mini" \
    --output text

# List all available templates
./target/release/llmspell template list
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
local result = Tool.execute("file-operations", {
    operation = "read",
    path = "document.txt"
})

-- Web search
local search = Tool.execute("web-search", {
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

### 6. Using Templates ⭐ **Phase 12**
```lua
-- List all available templates
local templates = Template.list()
for _, tmpl in ipairs(templates) do
    print(tmpl.name .. " (" .. tmpl.category .. "): " .. tmpl.description)
end

-- Get template information
local info = Template.info("research-assistant")
print("Parameters:", JSON.encode(info.config_schema))

-- Execute template from Lua
local result = Template.execute("code-generator", {
    description = "A function to parse JSON",
    language = "lua",
    model = "openai/gpt-4o-mini"
})

if result.success then
    print("Generated code:", result.result.result)
else
    print("Error:", result.error)
end

-- Search templates by keyword
local found = Template.search("research", "research")
for _, tmpl in ipairs(found) do
    print("Found:", tmpl.name)
end
```

## Command Line Usage

### Basic Execution (Embedded Kernel)

```bash
# Execute inline code (kernel runs embedded)
./target/release/llmspell exec 'print("Hello from LLMSpell!")'

# Run a script
./target/release/llmspell run script.lua

# Pass arguments
./target/release/llmspell run script.lua arg1 arg2

# Use configuration file (required for agents and RAG)
./target/release/llmspell -c config.toml run script.lua

# Enable debug output with --trace flag (Phase 9)
./target/release/llmspell --trace debug run script.lua
./target/release/llmspell --trace info exec "print('test')"

# Validate configuration
./target/release/llmspell -c config.toml validate
```

### Kernel Service Mode (Phase 9-10)

```bash
# Start kernel as service (listens for connections)
./target/release/llmspell kernel start --port 9555

# Start as daemon (background service)
./target/release/llmspell kernel start --daemon --port 9555

# Connect to running kernel
./target/release/llmspell kernel connect --address tcp://localhost:9555

# List running kernels
./target/release/llmspell kernel list

# Stop kernel
./target/release/llmspell kernel stop --all
```

### Service Installation (Phase 10)

```bash
# Install as system service (auto-detect platform)
./target/release/llmspell kernel install-service

# Install with options
./target/release/llmspell kernel install-service --enable --start --port 9600

# Manage service
systemctl --user start llmspell-kernel  # Linux
launchctl start com.llmspell.kernel     # macOS
```

### Template Management (Phase 12)

```bash
# List all templates
./target/release/llmspell template list

# List by category
./target/release/llmspell template list --category research
./target/release/llmspell template list --category codegen

# Show template details
./target/release/llmspell template info research-assistant
./target/release/llmspell template info code-generator

# Show parameter schema
./target/release/llmspell template schema research-assistant

# Execute template with parameters
./target/release/llmspell template exec research-assistant \
    --param topic="Rust async programming" \
    --param max_sources=10 \
    --param model="openai/gpt-4o-mini" \
    --output json

# Execute code generator
./target/release/llmspell template exec code-generator \
    --param description="A function to validate email addresses" \
    --param language="rust" \
    --param model="openai/gpt-4o-mini" \
    --output text

# Execute data analysis
./target/release/llmspell template exec data-analysis \
    --param data_file="/path/to/data.csv" \
    --param analysis_type="descriptive" \
    --param chart_type="bar" \
    --param model="openai/gpt-4o-mini"

# Search templates
./target/release/llmspell template search "code" --category codegen
./target/release/llmspell template search "research"
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

# 7. First Template ⭐ **Phase 12** (5 seconds)
./target/release/llmspell template exec code-generator \
    --param description="A function to reverse a string" \
    --param language="lua" \
    --param model="openai/gpt-4o-mini" \
    --output text
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
local content = Tool.execute("file-operations", {
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
            Tool.execute("file-operations", {
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

### Research Assistant with Template ⭐ **Phase 12**
```lua
-- research.lua - Use template for instant research capability
local result = Template.execute("research-assistant", {
    topic = ARGS[1] or "Rust async programming",
    max_sources = 5,
    model = "openai/gpt-4o-mini"
})

if result.success then
    local output = result.result
    print("=== Research Summary ===")
    print(output.result)

    -- Access metadata
    if output.metadata then
        print("\n=== Sources ===")
        for i, source in ipairs(output.metadata.sources or {}) do
            print(i .. ". " .. source)
        end
    end

    -- Check metrics
    if output.metrics then
        print("\n=== Metrics ===")
        print("Duration:", output.metrics.duration_ms, "ms")
        print("Tokens used:", output.metrics.tokens_used or "N/A")
    end
else
    print("❌ Error:", result.error)
end
```

## What's Next?

### Immediate Next Steps
1. **Try the 7 progressive examples** → `examples/script-users/getting-started/` (including templates!)
2. **Explore templates** → Run `./target/release/llmspell template list` to see 10 built-in workflows
3. **Configure your providers** → [Configuration Guide](configuration.md)
4. **Set up RAG (Phase 8.10.6)** → Use `configs/rag-basic.toml`

### Deeper Learning
5. **Understand the architecture** → [Core Concepts](concepts.md) (RAG, HNSW, Multi-tenancy, Templates)
6. **Template documentation** → [Template User Guide](templates/) (10 experimental workflows for rapid exploration workflows)
7. **Production patterns** → [Cookbook](../../examples/script-users/cookbook/) (11 examples)
8. **Real applications** → [Applications](../../examples/script-users/applications/) (9 complete apps)
9. **Complete API reference** → [Lua API](api/lua/README.md) (18 globals including Template)

### Advanced Topics
10. **Multi-tenant RAG** → `configs/rag-multi-tenant.toml`
11. **Vector storage optimization** → [RAG Configuration](configuration.md#rag-configuration)
12. **Debug and troubleshoot** → [Troubleshooting](troubleshooting.md)

## Quick Tips

### Core Tips
- **18 globals pre-injected** (no `require()` needed): `Agent`, `Tool`, `Template`, `RAG`, `State`, `Debug`, etc.
- **Configuration required** for agents and RAG (use `-c config.toml`)
- **Provider setup**: Add API keys to config files or environment variables
- **Tool discovery**: `Tool.list()` shows all 40+ available tools
- **Template discovery**: `Template.list()` shows all 10 built-in workflow templates
- **Error handling**: Always check `.success` field on agent/tool/template results
- **Timeouts**: Scripts timeout after 5 minutes by default
- **State scoping**: State operations require scope parameter (`"global"`, `"user"`, etc.)

### Phase 9-10 Kernel Features
- **Use --trace flag**: Replace `--debug`/`--verbose` with `--trace debug` or `--trace info`
- **Kernel modes**: Embedded (default), Service (external connections), Daemon (background)
- **Service installation**: Use `kernel install-service` for production deployment
- **Debug with DAP**: Enable IDE debugging with `Debug.enableDAP()`
- **Multiple kernels**: Run fleet of kernels on different ports for scaling
- **Global IO runtime**: Fixes "dispatch task is gone" errors automatically
- **Signal handling**: SIGTERM (shutdown), SIGHUP (reload), SIGUSR1 (stats)

### Phase 8 RAG Features
- **RAG requires OpenAI**: Set up OpenAI provider for embedding generation
- **Vector dimensions**: Default 384, supports 768, 1536, 3072
- **HNSW performance**: <10ms search for 1M vectors with proper configuration
- **Multi-tenancy**: Use `multi_tenant = true` in config for isolation

### Phase 12 Template Features
- **10 built-in templates**: Instant productivity from day 0 with experimental workflows for rapid exploration workflows
- **6 template categories**: Research, Chat, Analysis, CodeGen, Document, Workflow
- **Zero configuration**: Templates work out-of-the-box with just API keys
- **CLI and Lua APIs**: Use templates from command line or scripts
- **Template performance**: <2ms initialization, <1ms registry lookup
- **Cost estimation**: Pre-execution cost estimation with `Template.estimate_cost()`
- **Extensible system**: Create custom templates using Template trait
- **Template discovery**: Search by name, category, or keywords with `Template.search()`
- **Parameter validation**: Automatic schema validation before execution
- **Rich metadata**: Every template includes description, parameters, examples, and usage

---

## Quick Start Checklist

- [ ] ✅ Install Rust and build LLMSpell (`cargo build --release`)
- [ ] 🔑 Set up OpenAI API key (`export OPENAI_API_KEY="sk-..."`)
- [ ] 📂 Try hello world (`./target/release/llmspell exec 'print("Hello!")'`)
- [ ] 🤖 Create first agent with config file
- [ ] 📋 Try first template (`./target/release/llmspell template exec code-generator --param...`)
- [ ] 📚 Try RAG example with `rag-basic.toml`
- [ ] 🛠️ Explore 40+ tools with `Tool.list()`
- [ ] 📝 Explore 10 templates with `Template.list()`
- [ ] 🔄 Build your first workflow
- [ ] 📖 Read [Core Concepts](concepts.md) to understand RAG, HNSW, and Templates

**Phase 12 Ready!** Production-ready AI workflow templates for instant productivity from day 0.

---

**Need help?** Check [Troubleshooting](troubleshooting.md) for kernel issues or [Service Deployment](service-deployment.md) for production setup. Report issues on [GitHub](https://github.com/yourusername/rs-llmspell/issues)