# Lua Scripting Essentials

**Quick guide to writing Lua scripts with llmspell**

🔗 **Navigation**: [← User Guide](README.md) | [Core Concepts](02-core-concepts.md) | [Complete API Reference](appendix/lua-api-reference.md)

---

## Overview

llmspell provides 18 global objects in Lua for rapid AI experimentation. This guide covers the essentials to get you started quickly.

**What You'll Learn:**
- Core globals and their purposes
- Common scripting patterns
- Quick examples for each major feature
- Links to complete API documentation

**For Complete API Documentation:**
See [Lua API Reference](appendix/lua-api-reference.md) for all 18 globals with 200+ methods.

---

## The 18 Lua Globals

**Core Infrastructure (6 globals):**
1. **Agent** - Create and execute LLM-backed agents
2. **Tool** - Access 40+ built-in tools
3. **Workflow** - Orchestrate multi-step processes
4. **State** - Persistent key-value storage
5. **Event** - Publish/subscribe event system
6. **Hook** - Intercept and modify execution

**AI & RAG (6 globals):**
7. **RAG** - Retrieval-Augmented Generation pipeline
8. **Memory** - Episodic, semantic, procedural memory (Phase 13)
9. **Context** - Context engineering and retrieval (Phase 13)
10. **Embedding** - Generate vector embeddings
11. **Provider** - LLM provider management (OpenAI, Anthropic, Ollama, Candle)
12. **Template** - Execute AI workflow templates (Phase 12)

**Advanced Features (6 globals):**
13. **Session** - Session management with artifacts
14. **Security** - Access control and permissions
15. **Debug** - Debugging and introspection
16. **Config** - Runtime configuration access
17. **Model** - Model management (pull, list, info)
18. **Kernel** - Kernel control (start, stop, status)

---

## Quick Start Examples

### 1. Simple Agent Execution

```lua
-- Create an agent
local agent = Agent.new({
    name = "assistant",
    provider = "openai",
    model = "gpt-4o-mini"
})

-- Execute
local result = agent:execute("What is the capital of France?")
print(result.content)
-- Output: "The capital of France is Paris."
```

### 2. Using Tools

```lua
-- Get a tool
local calc = Tool.get("calculator")

-- Invoke it
local result = calc:invoke({ expression = "2 + 2 * 3" })
print(result.value)  -- Output: 8

-- List all available tools
local tools = Tool.list()
for _, tool_name in ipairs(tools) do
    print(tool_name)
end
```

### 3. RAG (Retrieval-Augmented Generation)

```lua
-- Create RAG pipeline
local rag = RAG.new({
    collection = "documentation",
    embedding_model = "text-embedding-3-small"
})

-- Ingest documents
rag:ingest("Getting started with llmspell...", {
    source = "docs/getting-started.md",
    category = "documentation"
})

-- Query
local results = rag:search("How do I get started?", { k = 5 })
for _, result in ipairs(results) do
    print(string.format("Score: %.3f - %s", result.score, result.content))
end
```

### 4. Memory System (Phase 13)

```lua
-- Episodic memory (conversation history)
Memory.add_episodic("session_123", "User asked about PostgreSQL", {
    role = "user",
    timestamp = os.time()
})

Memory.add_episodic("session_123", "I provided setup instructions", {
    role = "assistant",
    timestamp = os.time()
})

-- Query episodic memory
local memories = Memory.query_episodic("PostgreSQL setup", {
    session_id = "session_123",
    k = 3
})

-- Semantic memory (knowledge graph)
Memory.add_semantic("PostgreSQL is a relational database", {
    relations = {
        {"PostgreSQL", "is_a", "database"},
        {"PostgreSQL", "supports", "ACID"}
    }
})

-- Query semantic memory
local facts = Memory.query_semantic("What is PostgreSQL?")
```

### 5. State Management

```lua
-- Write to state
State.write("user_name", "Alice")
State.write("count", 42)

-- Read from state
local name = State.read("user_name")
print(name)  -- Output: "Alice"

-- Check if exists
if State.exists("count") then
    local count = State.read("count")
    State.write("count", count + 1)
end

-- Delete
State.delete("old_key")

-- List all keys
local keys = State.list_keys()
```

### 6. Workflows

```lua
-- Create sequential workflow
local workflow = Workflow.new({
    type = "sequential",
    steps = {
        {
            name = "research",
            agent = {
                provider = "openai",
                model = "gpt-4o-mini"
            },
            prompt = "Research topic: {{topic}}"
        },
        {
            name = "summarize",
            agent = {
                provider = "openai",
                model = "gpt-4o-mini"
            },
            prompt = "Summarize: {{research.content}}"
        }
    }
})

-- Execute
local result = workflow:execute({ topic = "Rust async programming" })
print(result.steps.summarize.content)
```

### 7. Templates (Phase 12)

```lua
-- List available templates
local templates = Template.list()
for _, t in ipairs(templates) do
    print(t.name, "-", t.description)
end

-- Execute template
local result = Template.execute("code-generator", {
    description = "A function to validate email addresses",
    language = "rust",
    model = "openai/gpt-4o-mini"
})

print(result.content)  -- Generated code
```

### 8. Events & Hooks

```lua
-- Subscribe to event
Event.subscribe("agent.completed", function(data)
    print("Agent completed:", data.agent_name)
end)

-- Publish event
Event.publish("custom.event", { message = "Hello" })

-- Add hook
Hook.add("before_tool_execution", function(context)
    print("Executing tool:", context.tool_name)
    return context  -- Continue execution
end)
```

### 9. Context Engineering (Phase 13)

```lua
-- Assemble context from multiple sources
local context = Context.assemble({
    query = "How do I set up PostgreSQL?",
    strategies = {"episodic", "rag"},
    token_budget = 2000,
    rerank = true
})

-- Use context in agent
local agent = Agent.new({
    provider = "openai",
    model = "gpt-4o-mini"
})

local prompt = string.format([[
Context:
%s

Question: %s

Answer:
]], context.text, "How do I set up PostgreSQL?")

local result = agent:execute(prompt)
```

---

## Common Patterns

### Pattern 1: Agent with RAG

```lua
-- Setup RAG
local rag = RAG.new({ collection = "docs" })
rag:ingest_file("docs/README.md")

-- Create agent
local agent = Agent.new({
    provider = "openai",
    model = "gpt-4o-mini"
})

-- Query with context
local query = "How do I configure the system?"
local context = rag:search(query, { k = 3 })

local prompt = "Context:\n"
for _, result in ipairs(context) do
    prompt = prompt .. result.content .. "\n"
end
prompt = prompt .. "\nQuestion: " .. query

local answer = agent:execute(prompt)
print(answer.content)
```

### Pattern 2: Multi-Agent Collaboration

```lua
-- Researcher agent
local researcher = Agent.new({
    name = "researcher",
    provider = "openai",
    model = "gpt-4o-mini",
    system_prompt = "You are a research assistant. Provide detailed information."
})

-- Summarizer agent
local summarizer = Agent.new({
    name = "summarizer",
    provider = "openai",
    model = "gpt-4o-mini",
    system_prompt = "You summarize information concisely."
})

-- Collaborate
local research = researcher:execute("Research Rust async programming")
local summary = summarizer:execute("Summarize: " .. research.content)

print(summary.content)
```

### Pattern 3: Stateful Conversation

```lua
local session_id = "conversation_123"

-- Add to episodic memory
local function add_message(role, content)
    Memory.add_episodic(session_id, content, {
        role = role,
        timestamp = os.time()
    })
end

-- Get conversation context
local function get_context()
    local memories = Memory.query_episodic("", {
        session_id = session_id,
        k = 10
    })

    local context = ""
    for _, mem in ipairs(memories) do
        context = context .. mem.metadata.role .. ": " .. mem.content .. "\n"
    end
    return context
end

-- Chat loop
add_message("user", "What is Rust?")
local context = get_context()
local agent = Agent.new({ provider = "openai", model = "gpt-4o-mini" })
local response = agent:execute(context .. "\nassistant:")
add_message("assistant", response.content)
```

### Pattern 4: Tool Chaining

```lua
-- Chain multiple tools
local web = Tool.get("web_search")
local calc = Tool.get("calculator")
local file = Tool.get("file_write")

-- 1. Search for data
local search_result = web:invoke({ query = "Rust adoption statistics 2024" })

-- 2. Process data
local calc_result = calc:invoke({ expression = "42 * 1.5" })

-- 3. Save results
file:invoke({
    path = "results.txt",
    content = search_result.content .. "\nCalculation: " .. calc_result.value
})
```

---

## Error Handling

```lua
-- Use pcall for error handling
local success, result = pcall(function()
    local agent = Agent.new({
        provider = "openai",
        model = "gpt-4o-mini"
    })
    return agent:execute("Hello")
end)

if success then
    print("Result:", result.content)
else
    print("Error:", result)
end
```

---

## Configuration

```lua
-- Access configuration
local config = Config.get("provider.openai.api_key")
print(config)

-- Get all config
local all_config = Config.get_all()
```

---

## Next Steps

### Learn More
- **Complete API**: [Lua API Reference](appendix/lua-api-reference.md) - All 18 globals with 200+ methods
- **Examples**: See `examples/script-users/` for 60+ working examples
- **Core Concepts**: [Core Concepts Guide](02-core-concepts.md) - Architecture details

### Try Examples
```bash
# Run examples
llmspell run examples/script-users/getting-started/01-hello-world.lua
llmspell run examples/script-users/getting-started/02-first-agent.lua
llmspell run examples/script-users/getting-started/05-first-rag.lua
```

### Advanced Topics
- **Templates**: [Templates & Workflows](06-templates-and-workflows.md) - Pre-built AI workflows
- **Memory**: See Memory sections in [Core Concepts](02-core-concepts.md)
- **RAG**: See RAG sections in [Core Concepts](02-core-concepts.md)
- **Deployment**: [Deployment Guide](08-deployment.md) - Production deployment

---

**Version**: 0.13.0 | **Phase**: 13b.18.3 | **Last Updated**: 2025-11-08
