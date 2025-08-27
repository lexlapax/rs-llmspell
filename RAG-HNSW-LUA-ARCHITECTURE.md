# RAG HNSW Lua Architecture

## Overview

This document explains how HNSW vector storage becomes available to Lua through the RAG system in rs-llmspell.

## Architecture Flow: HNSW → Lua

The HNSW configuration is **NOT** exposed through Lua's `RAG.configure()` - it's set up during system initialization. Here's the complete flow:

### 1. System Initialization (Rust Layer)

```rust
// ScriptRuntime initialization creates the infrastructure
let runtime = ScriptRuntime::new_with_lua(config).await?;

// Inside new_with_engine():
// 1. Create GlobalContext
let global_context = Arc::new(GlobalContext::new(registry, providers));

// 2. Set up RAG infrastructure in context (if dependencies available)
if config.rag.enabled {
    // HNSW storage configured here with fixed parameters
    let hnsw_config = HNSWConfig {
        m: 16,                    // NOT configurable from Lua
        ef_construction: 200,     // NOT configurable from Lua
        ef_search: 50,           // NOT configurable from Lua
        max_elements: 1_000_000, // NOT configurable from Lua
        metric: DistanceMetric::Cosine,
        // ... other fixed parameters
    };
    let vector_storage = Arc::new(HNSWVectorStorage::new(384, hnsw_config));
    let tenant_manager = Arc::new(MultiTenantVectorManager::new(vector_storage));
    let multi_tenant_rag = Arc::new(MultiTenantRAG::new(tenant_manager));
    
    global_context.set_bridge("multi_tenant_rag", multi_tenant_rag);
}

// 3. Create standard registry (includes RAG global if available)
let global_registry = create_standard_registry(global_context.clone()).await?;

// 4. Inject into Lua engine
let injector = GlobalInjector::new(Arc::new(global_registry));
injector.inject_lua(&lua, &global_context)?;
```

### 2. RAG Global Creation

In `llmspell-bridge/src/globals/mod.rs:73-96`:

```rust
async fn register_rag_global(/* ... */) {
    if let (Some(state_manager), Some(session_manager), Some(multi_tenant_rag)) = (
        context.get_bridge::<StateManager>("state_manager"),
        session_manager_opt,
        context.get_bridge::<MultiTenantRAG>("multi_tenant_rag"), // Gets from context
    ) {
        // Create RAGGlobal using the pre-configured multi_tenant_rag
        match rag_global::RAGGlobal::with_managers(/* ... */).await {
            Ok(rag_global) => builder.register(Arc::new(rag_global)),
            // ...
        }
    }
}
```

### 3. Lua Binding Injection

In `llmspell-bridge/src/lua/globals/rag.rs:401-422`:

```rust
pub fn inject_rag_global(
    lua: &Lua,
    _context: &GlobalContext,
    bridge: Arc<RAGBridge>, // Contains the configured HNSW storage
) -> mlua::Result<()> {
    let rag_table = lua.create_table()?;
    
    // Register all RAG methods that use the pre-configured HNSW
    register_search_method(lua, &rag_table, &bridge)?;
    register_ingest_method(lua, &rag_table, &bridge)?;
    register_configure_method(lua, &rag_table, &bridge)?; // Only runtime settings!
    // ...
    
    lua.globals().set("RAG", rag_table)?;
    Ok(())
}
```

## Lua Usage Example

Here's how you use RAG from Lua. **Note**: HNSW parameters are pre-configured, not scriptable.

```lua
-- Configure runtime settings only (NOT HNSW parameters)
RAG.configure({
    session_ttl = 3600,        -- Session timeout
    default_provider = "openai", -- Embedding provider
    enable_cache = true,       -- Cache embeddings  
    cache_ttl = 1800          -- Cache timeout
})

-- List available embedding providers
local providers = RAG.list_providers()
for i, provider in ipairs(providers) do
    print(i .. ": " .. provider)
end

-- Ingest documents (uses pre-configured HNSW with 384 dimensions, m=16, etc.)
local ingest_result = RAG.ingest({
    {
        id = "doc1",
        text = "Vector databases enable semantic search",
        metadata = { 
            source = "tech_article.md",
            category = "database" 
        }
    },
    {
        id = "doc2", 
        text = "HNSW provides approximate nearest neighbor search",
        metadata = { 
            source = "research_paper.pdf",
            category = "algorithms"
        }
    }
}, {
    scope = "project",
    scope_id = "knowledge_base",
    provider = "openai",  -- Which embedding model to use
    chunking = {          -- Text chunking options
        chunk_size = 500,
        overlap = 50,
        strategy = "sliding_window"
    }
})

print("Processed:", ingest_result.documents_processed)
print("Vectors created:", ingest_result.vectors_created)

-- Search (uses pre-configured HNSW parameters)
local search_result = RAG.search("vector similarity search", {
    k = 5,                    -- Top 5 results
    scope = "project", 
    scope_id = "knowledge_base",
    filters = { 
        category = "database"  -- Metadata filtering
    },
    threshold = 0.7           -- Similarity threshold
})

print("Found:", search_result.total, "results")
for i, result in ipairs(search_result.results) do
    print(string.format("%.3f - %s", result.score, result.text))
end

-- Get statistics
local stats = RAG.get_stats("project", "knowledge_base")
print("Total vectors:", stats.total_vectors)
```

## Key Points

1. **HNSW Configuration**: Fixed at system initialization (dimensions=384, m=16, ef_construction=200, etc.)
2. **Lua RAGConfig**: Only controls runtime settings (TTL, providers, cache)
3. **Vector Storage**: Transparent to Lua - it just calls RAG methods
4. **Multi-tenancy**: Handled automatically through scopes and namespacing
5. **Performance**: Configured to meet <10ms search targets for 1M vectors

## Architecture Summary

The HNSW vector storage is a **system-level configuration** that provides the performance foundation, while Lua scripts focus on **semantic operations** like document ingestion, search, and metadata filtering.

### Flow Summary:
1. **System Init**: HNSW configured in Rust with fixed performance parameters
2. **Bridge Creation**: RAGBridge wraps the configured HNSW storage
3. **Global Registry**: RAGGlobal registered with the bridge reference
4. **Lua Injection**: RAG table methods injected into Lua runtime
5. **Script Access**: Lua scripts call `RAG.*` methods that use HNSW transparently

This design ensures optimal performance while providing a clean, semantic API to script authors.