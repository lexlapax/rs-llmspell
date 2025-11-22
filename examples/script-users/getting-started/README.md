# Getting Started with LLMSpell

Progressive examples to learn LLMSpell from scratch. Each example builds on the previous one.

**Total Time**: ~30 minutes | **Examples**: 5 | **Complexity**: BEGINNER → INTERMEDIATE

## 🚀 Quick Start

```bash
# 1. Verify installation (2 seconds)
./target/debug/llmspell run examples/script-users/getting-started/00-hello-world.lua

# 2. Try your first tool (5 seconds)
./target/debug/llmspell run examples/script-users/getting-started/01-first-tool.lua

# 3. Create an agent (10 seconds, requires API key)
./target/debug/llmspell -p providers run examples/script-users/getting-started/02-first-agent.lua

# 4. Build a workflow (20 milliseconds)
./target/debug/llmspell run examples/script-users/getting-started/03-first-workflow.lua

# 5. Handle errors properly (5 seconds)
./target/debug/llmspell -p state run examples/script-users/getting-started/04-handle-errors.lua

# 6. Memory, RAG & Context Assembly (10 minutes, requires API key)
./target/debug/llmspell -p memory-development run examples/script-users/getting-started/05-memory-rag-advanced.lua
```

## 🎯 Learning Path

### Step 1: Hello World (2 seconds)
**File**: `00-hello-world.lua`  
**Learn**: Verify installation, explore available globals, basic script structure  
**Prerequisites**: None  
**Key Concepts**: Script execution, return values, environment information

```bash
./target/debug/llmspell run 00-hello-world.lua
# Output: Hello from LLMSpell! Plus version info and available globals
```

### Step 2: Your First Tool (5 seconds)
**File**: `01-first-tool.lua`  
**Learn**: Tool invocation, parameter passing, result handling  
**Prerequisites**: None  
**Key Concepts**: Tool.execute(), file operations, error checking

```bash
./target/debug/llmspell run 01-first-tool.lua
# Creates, reads, and checks a file in /tmp
```

### Step 3: Your First Agent (10 seconds)
**File**: `02-first-agent.lua`
**Learn**: Agent creation, provider selection, basic conversation
**Prerequisites**: OpenAI or Anthropic API key (environment variable)
**Key Concepts**: Agent.builder(), system prompts, response handling

```bash
./target/debug/llmspell -p providers run 02-first-agent.lua
# Creates an agent and asks a simple math question

# Or with debug logging:
./target/debug/llmspell -p development run 02-first-agent.lua
```

### Step 4: Your First Workflow (20 milliseconds)
**File**: `03-first-workflow.lua`  
**Learn**: Workflow builder, sequential execution, multi-tool orchestration  
**Prerequisites**: None  
**Key Concepts**: Workflow.builder(), step chaining, result aggregation

```bash
./target/debug/llmspell run 03-first-workflow.lua
# Chains UUID generation, timestamp, hash, and file creation
```

### Step 5: Error Handling (5 seconds)
**File**: `04-handle-errors.lua`
**Learn**: Production-ready error handling patterns
**Prerequisites**: None (state profile recommended for full demo)
**Key Concepts**: pcall(), graceful degradation, user-friendly errors

```bash
# Basic run (no state):
./target/debug/llmspell run 04-handle-errors.lua

# With state enabled (recommended):
./target/debug/llmspell -p state run 04-handle-errors.lua
```

### Step 6: Memory, RAG & Context Assembly (10 minutes)
**File**: `05-memory-rag-advanced.lua`
**Complexity**: INTERMEDIATE
**Learn**: Complete memory & retrieval system integration
**Prerequisites**: OpenAI API key (for embeddings), understanding of examples 00-04
**Key Concepts**:
- **RAG**: Document ingestion, vector embeddings, semantic search
- **Memory**: Episodic conversation tracking with session isolation
- **Context**: Token-budgeted context assembly with reranking
- **Integration**: End-to-end workflow combining all three systems

```bash
./target/debug/llmspell -p memory-development run 05-memory-rag-advanced.lua
# Comprehensive demo of RAG, Memory, and Context working together

# What you'll see:
# Section 1: RAG - Ingest 5 programming docs, perform semantic searches
# Section 2: Memory - Track conversation exchanges across sessions
# Section 3: Context - Assemble context with token budgets
# Section 4: Integration - Combine RAG + Memory + Agent workflow
# Section 5: Statistics - View system metrics and performance
```

**This example demonstrates**:
- How to build AI that learns from documents (RAG)
- How to build AI that remembers conversations (Memory)
- How to build AI that selects relevant context (Context Assembly)
- How to integrate all three for production-ready systems

## 💡 Common Patterns

### Using Tools
```lua
local result = Tool.execute("tool_name", {
    operation = "operation_name",
    input = "your data"
})

if result.text then
    print("Success: " .. result.text)
else
    print("Error: " .. (result.error or "Unknown"))
end
```

### Creating Agents
```lua
local agent_result = Agent.builder()
    .provider("openai")  -- or detected from Provider.list()
    .system_prompt("You are helpful")
    .build()

if agent_result.success then
    local agent = agent_result.result
    local response = agent:invoke("Hello!")
end
```

### Building Workflows
```lua
local workflow = Workflow.builder()
    :name("my_workflow")
    :sequential()
    :add_step({
        name = "step1",
        type = "tool",
        tool = "tool_name",
        input = { ... }
    })
    :build()

local result = workflow:execute({})
```

### State Management (with Scopes)
```lua
-- State API requires scope parameter
State.save("global", "key", "value")
local value = State.load("global", "key")
State.delete("global", "key")

-- Available scopes: global, custom, workflow, agent, tool
local keys = State.list_keys("global")
```

### Error Handling Best Practices
```lua
local function safe_operation(...)
    local success, result = pcall(function()
        return Tool.execute(...)
    end)
    
    if success and result then
        return result
    else
        print("Error: " .. tostring(result))
        return nil
    end
end
```

## 📚 Key Concepts

### Tools (34+ available)
Built-in functions for file operations, web requests, data processing, etc.
- Synchronous execution in Lua
- Automatic error handling  
- Rich parameter validation

### Agents (Multi-provider)
LLM-powered assistants that can use tools and follow instructions.
- OpenAI, Anthropic, and more
- Tool integration capabilities
- Conversation state management

### Workflows (Orchestration)
Chain tools and agents in complex patterns.
- Sequential and parallel execution
- Conditional logic and loops
- Data flow between steps

### State (Persistence)
Scoped data storage across script executions.
- Multiple scopes for isolation
- Memory, file, and database backends
- JSON serialization support

### Error Handling (Production-ready)
Robust error management patterns.
- pcall for safe execution
- Result validation helpers
- Graceful degradation strategies

## 🔧 Troubleshooting

### No tools available
Ensure LLMSpell is properly built and initialized:
```bash
cargo build --release
./target/release/llmspell run 00-hello-world.lua
```

### Agent errors
Ensure you have API keys set in your environment:
```bash
# For OpenAI:
export OPENAI_API_KEY="your-key-here"

# For Anthropic:
export ANTHROPIC_API_KEY="your-key-here"

# Then run with providers profile:
./target/debug/llmspell -p providers run your_script.lua
```

### State not available
Use the state builtin profile:
```bash
./target/debug/llmspell -p state run your_script.lua
```

### Debug mode
For verbose output during troubleshooting:
```bash
RUST_LOG=debug ./target/debug/llmspell run your_script.lua
```

## 🎓 Next Steps

After completing these examples:
1. Explore [features](../features/) for specific capabilities like state persistence
2. Study [cookbook](../cookbook/) for production patterns (8 curated examples)
3. Review [applications](../applications/) for complete real-world systems
4. Build your own scripts combining these patterns!

## 🔗 Resources

- [LLMSpell API Reference](../../../docs/user-guide/api/lua/README.md)
- [Configuration Guide](../configs/README.md)
- [Tool Catalog](../../../docs/user-guide/tools-catalog.md)
- [Architecture Overview](../../../docs/technical/master-architecture-vision.md)