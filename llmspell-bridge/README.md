# llmspell-bridge

**Script engine integration bridge for Lua and JavaScript**

**🔗 Navigation**: [← Project Root](../) | [Documentation](../docs/) | [Lua Examples](../examples/script-users/)

---

## Overview

This crate provides the bridge layer between script languages and rs-llmspell's Rust implementation, enabling:

- **Zero-Configuration Access**: Pre-injected global objects (Agent, Tool, Workflow, State, Hook, Event, RAG)
- **Shared State Architecture**: ScriptRuntime shares StateManager with kernel for unified state
- **RAG Integration (Phase 8)**: Complete RAG pipeline access with multi-tenant isolation
- **Synchronous API**: Transparent async-to-sync conversion for script compatibility
- **Cross-Language Support**: Consistent API across Lua and JavaScript
- **Performance Optimized**: <5ms injection overhead, <10ms execution overhead
- **State Management**: Full Phase 5 persistent state access from scripts with shared backend
- **Hook & Event Integration**: Complete Phase 4 hook/event system access
- **Multi-Tenant RAG**: Tenant-scoped document ingestion and retrieval operations

## Features

### Global Objects

All scripts automatically have access to these globals:

```lua
-- Agent operations
local agent = Agent.create({
    model = "openai/gpt-4",
    system_prompt = "You are helpful"
})
local response = agent:execute({prompt = "Hello"})

-- Tool usage
local tool = Tool.get("web_search")
local results = tool:execute({query = "rs-llmspell"})

-- Workflow orchestration
local workflow = Workflow.sequential({
    name = "research_flow",
    steps = {
        {tool = "web_search", input = {query = "$input"}},
        {agent = agent, prompt = "Summarize: $step1.output"}
    }
})

-- State persistence (Phase 5)
State.save("agent:gpt-4", "history", messages)
local data = State.load("global", "config")
State.migrate({from_version = 1, to_version = 2})

-- Hook registration (Phase 4)
Hook.register("agent:before_execution", function(ctx)
    Logger.info("Starting", {id = ctx.agent_id})
    return {continue_execution = true}
end)

-- Event handling (Phase 4)
Event.subscribe("*.error", function(event)
    Alert.send("Error", event.payload)
end)

-- RAG operations (Phase 8)
local rag = RAG.create({
    chunking = {strategy = "semantic", chunk_size = 512},
    embedding_provider = "openai/text-embedding-ada-002"
})

-- Multi-tenant document ingestion
RAG.ingest_documents("tenant:company-123", {
    {content = document_text, metadata = {type = "manual"}},
    {file_path = "docs/guide.md", metadata = {type = "documentation"}}
})

-- Tenant-scoped retrieval
local results = RAG.retrieve("tenant:company-123", {
    query = "How do I configure authentication?",
    top_k = 5,
    similarity_threshold = 0.8
})

-- Session-aware RAG with conversation memory
local session_id = Session.current_id()
local context_results = RAG.retrieve_with_context(
    "tenant:company-123",
    session_id,
    "What were the performance metrics we discussed?"
)
```

### Synchronous Wrappers

The bridge transparently converts Rust's async operations to synchronous for scripts:

```rust
// Rust async operation
pub async fn execute(&self, prompt: &str) -> Result<String>

// Becomes synchronous in Lua
local result = agent:execute({prompt = "Hello"})
```

### State Management API (Phase 5)

```lua
-- Save state with automatic persistence
State.save(scope, key, value)

-- Load state with optional default
local value = State.load(scope, key) or default_value

-- Delete state
State.delete(scope, key)

-- List keys in scope
local keys = State.list_keys(scope)

-- Perform migration
State.migrate({
    from_version = 1,
    to_version = 2,
    transformations = {
        {field = "old_field", transform = "copy", to = "new_field"},
        {field = "version", transform = "default", value = 2}
    }
})

-- Backup operations
local backup_id = State.backup({description = "Pre-update"})
State.restore(backup_id)
```

### Hook System Access (Phase 4)

```lua
-- Register hooks with priority
Hook.register("tool:*:before_execution", handler, "high")

-- Unregister hooks
Hook.unregister(hook_id)

-- List hooks with filtering
local hooks = Hook.list({
    hook_point = "agent:*",
    language = "lua",
    priority = "high"
})

-- Pause/resume hooks
Hook.pause(hook_id)
Hook.resume(hook_id)
```

### Event System Access (Phase 4)

```lua
-- Subscribe to events with patterns
local sub_id = Event.subscribe("workflow.*.completed", handler)

-- Emit custom events
Event.emit({
    event_type = "custom:milestone",
    payload = {progress = 0.75}
})

-- Unsubscribe
Event.unsubscribe(sub_id)

-- Get event statistics
local stats = Event.stats()
```

### RAG System Access (Phase 8)

```lua
-- Create RAG pipeline with tenant isolation
local rag = RAG.create({
    tenant_id = "company-123",
    chunking = {
        strategy = "semantic",
        chunk_size = 512,
        overlap = 50
    },
    embedding_provider = "openai/text-embedding-ada-002",
    vector_storage = {
        backend = "hnsw",
        distance_metric = "cosine",
        ef_construction = 200
    }
})

-- Batch document ingestion
local ingestion_result = RAG.ingest_documents("tenant:company-123", {
    {
        content = file_content,
        metadata = {
            document_id = "user-guide-v1",
            document_type = "documentation",
            version = "1.0"
        }
    },
    {
        file_path = "docs/api-reference.md",
        metadata = {
            document_type = "api_docs",
            last_updated = "2024-08-28"
        }
    }
})

Logger.info("Ingestion complete", {
    chunks_created = ingestion_result.chunks_created,
    vectors_indexed = ingestion_result.vectors_indexed
})

-- Advanced retrieval with filtering
local results = RAG.retrieve("tenant:company-123", {
    query = "How do I configure multi-tenant authentication?",
    top_k = 10,
    similarity_threshold = 0.8,
    metadata_filters = {
        document_type = "documentation",
        version = "1.0"
    },
    rerank = true
})

-- Process retrieval results
for i, result in ipairs(results.chunks) do
    Logger.info("Result", {
        rank = i,
        similarity = result.similarity,
        document_id = result.metadata.document_id,
        content_preview = string.sub(result.content, 1, 100)
    })
end

-- Session-aware retrieval with conversation context
local session_rag_results = RAG.retrieve_with_context(
    "tenant:company-123",
    Session.current_id(),
    {
        query = "What were the security considerations mentioned earlier?",
        conversation_memory_turns = 5,
        context_boost = 1.2
    }
)

-- RAG pipeline statistics and monitoring
local rag_stats = RAG.get_stats("tenant:company-123")
Logger.info("RAG Statistics", {
    total_documents = rag_stats.document_count,
    total_chunks = rag_stats.chunk_count,
    vector_storage_mb = rag_stats.storage_size_mb,
    avg_retrieval_time_ms = rag_stats.avg_retrieval_time
})

-- Tenant data management
RAG.delete_documents("tenant:company-123", {
    document_ids = {"obsolete-doc-1", "obsolete-doc-2"}
})

-- Bulk tenant cleanup
RAG.cleanup_tenant("tenant:old-company")
```

## Shared State Architecture (Phase 9)

The bridge now supports shared state management between kernel and ScriptRuntime:

### Creating ScriptRuntime with Shared StateManager

```rust
use llmspell_bridge::ScriptRuntime;
use llmspell_state_persistence::factory::StateFactory;
use llmspell_config::LLMSpellConfig;
use std::sync::Arc;

// Create configuration with state persistence enabled
let config = Arc::new(LLMSpellConfig::builder()
    .default_engine("lua")
    .runtime(GlobalRuntimeConfig::builder()
        .state_persistence(StatePersistenceConfig {
            enabled: true,
            backend_type: "sled".to_string(),
            ..Default::default()
        })
        .build())
    .build());

// Create shared StateManager
let state_manager = StateFactory::create_from_config(&config).await?
    .expect("State persistence enabled");

// Create ScriptRuntime with the shared StateManager
let runtime = ScriptRuntime::new_with_engine_and_state_manager(
    "lua",
    (*config).clone(),
    state_manager.clone(),  // Pass the shared StateManager
).await?;

// Now State global in scripts uses the shared backend
let result = runtime.execute_script(r#"
    State.save("global", "shared_key", {value = 42})
    return State.load("global", "shared_key")
"#).await?;
```

### Benefits of Shared State

1. **No File Lock Conflicts**: Single StateManager prevents concurrent access issues
2. **Data Consistency**: Kernel and scripts see the same state immediately
3. **Memory Efficiency**: One state backend instead of multiple instances
4. **Simplified Testing**: Single state manager simplifies test setup

## Architecture

The bridge consists of several layers:

### 1. Language Bindings (`src/lua/`, `src/javascript/`)
- mlua for Lua 5.4 integration
- boa/quickjs for JavaScript support
- Language-specific type conversions

### 2. Global Injection (`src/globals/`)
- `agent_global.rs` - Agent global with 23+ methods
- `tool_global.rs` - Tool discovery and execution
- `workflow_global.rs` - Workflow patterns
- `state_global.rs` - State persistence operations
- `hook_global.rs` - Hook registration and management
- `event_global.rs` - Event pub/sub system
- `rag_global.rs` - RAG pipeline operations (Phase 8)
- `rag_infrastructure.rs` - RAG system setup and configuration (Phase 8)
- `session_global.rs` - Session management
- `artifact_global.rs` - Artifact storage and retrieval
- `config_global.rs` - Configuration access
- `json_global.rs` - JSON utilities

### 3. Synchronous Utilities (`src/lua/sync_utils.rs`)
- `block_on_async()` - Efficient async-to-sync conversion
- Coroutine detection for proper yielding
- Minimal overhead execution

### 4. Type Conversion (`src/lua/conversions.rs`)
- Bidirectional Lua ↔ Rust conversions
- StateValue serialization
- Error propagation

## Performance

Achieved performance metrics (Phase 8):

| Operation | Target | Actual |
|-----------|--------|--------|
| Global Injection | <5ms | 2-4ms |
| Method Call Overhead | <10ms | <5ms |
| State Operation | <5ms | <5ms |
| Hook Registration | <1ms | <0.5ms |
| Event Emission | <1ms | <0.8ms |
| RAG Document Ingestion | <100ms/doc | <50ms/doc |
| RAG Vector Search | <10ms | <5ms |
| RAG Context Assembly | <5ms | <3ms |
| Multi-tenant Isolation Overhead | <5% | <3% |
| Memory per Context | <5MB | 1.8MB |
| Memory per RAG Pipeline | <10MB | 6-8MB |

## Usage Examples

### Complete Script Example

```lua
-- Initialize agent with state restoration
local agent_id = "research-bot"
local agent = Agent.create({
    name = agent_id,
    model = "anthropic/claude-3",
    system_prompt = "You are a research assistant"
})

-- Restore previous conversation
local history = State.load("agent:" .. agent_id, "history") or {}
if #history > 0 then
    Logger.info("Restored conversation", {messages = #history})
end

-- Set up monitoring
Hook.register("agent:after_execution", function(ctx)
    -- Update metrics
    local metrics = State.load("global", "metrics") or {}
    metrics.total_calls = (metrics.total_calls or 0) + 1
    State.save("global", "metrics", metrics)
    
    -- Emit event
    Event.emit({
        event_type = "agent:executed",
        payload = {
            agent_id = agent_id,
            duration_ms = ctx.duration_ms
        }
    })
end)

-- Main workflow
local workflow = Workflow.sequential({
    name = "research_workflow",
    steps = {
        {
            name = "search",
            tool = "web_search",
            input = {query = "$input"}
        },
        {
            name = "analyze",
            agent = agent,
            prompt = "Analyze these results: $search.output"
        },
        {
            name = "save",
            handler = function(ctx)
                -- Save results
                State.save("workflow:research", "last_result", ctx.analyze)
                return {success = true}
            end
        }
    }
})

-- Execute with error handling
local ok, result = pcall(function()
    return workflow:execute({input = "quantum computing advances"})
end)

if not ok then
    Logger.error("Workflow failed", {error = result})
    -- Restore from backup if available
    local backups = State.list_backups()
    if #backups > 0 then
        State.restore(backups[1].id)
    end
end
```

## Testing

```bash
# Run bridge tests
cargo test -p llmspell-bridge

# Test Lua integration
cargo test lua_integration

# Test JavaScript integration  
cargo test js_integration

# Benchmark performance
cargo bench -p llmspell-bridge
```

## Dependencies

- `llmspell-core` - Core traits and types
- `llmspell-agents` - Agent functionality
- `llmspell-tools` - Tool execution
- `llmspell-workflows` - Workflow patterns
- `llmspell-state-persistence` - State management
- `llmspell-hooks` - Hook system
- `llmspell-events` - Event system
- `llmspell-security` - Sandboxing and access control
- `llmspell-rag` - RAG pipeline functionality (Phase 8)
- `llmspell-storage` - Vector storage and HNSW (Phase 8)
- `llmspell-tenancy` - Multi-tenant isolation (Phase 8)
- `llmspell-sessions` - Session management
- `llmspell-config` - Configuration management
- `mlua` - Lua 5.4 bindings
- `boa_engine` - JavaScript engine

## License

This project is licensed under  Apache-2.0.