# RAG Commands Analysis - TODO.md Phase 9.9.1

## Problem Statement
```
- ❌ RAG System: Commands not implemented (no `rag` subcommand exists)
```

## Deep Analysis (Ultrathink)

### 1. Current State

#### RAG Backend Implementation (COMPLETE ✅)
The RAG system is fully implemented in `llmspell-rag` crate with:
- **RAGPipeline** - Core orchestration (llmspell-rag/src/pipeline/rag_pipeline.rs)
- **Document Ingestion** - `ingest_document()` method for processing and storing documents
- **Search/Query** - `search()` method for retrieval-augmented generation
- **Vector Storage** - HNSW-based storage with <8ms search on 100K vectors
- **Multi-tenant Support** - StateScope-based isolation
- **Embeddings** - OpenAI integration with caching
- **Session Integration** - SessionAwareRAGPipeline

#### CLI Integration (PARTIAL ⚠️)
RAG is only accessible via:
- `--rag-profile` flag on `run`, `exec`, `repl`, `debug` commands
- Profile-based configuration in TOML files
- Programmatic access through Lua scripts

#### Missing CLI Commands (❌)
No direct CLI commands exist for:
- Document ingestion (`rag ingest`)
- Search/query (`rag search`)
- Index management (`rag index`)
- Statistics (`rag stats`)
- Clear/reset (`rag clear`)

### 2. Root Cause Analysis

#### Why Commands Don't Exist
1. **Design Decision**: CLI architecture document (docs/technical/cli-command-architecture.md) shows RAG was designed as a **runtime feature** via `--rag-profile`, not as standalone commands
2. **Architecture Mismatch**: RAG pipeline requires kernel context for state/session integration, but CLI commands were designed for direct execution
3. **Implementation Gap**: Phase 8 focused on RAG backend, Phase 9 focused on kernel/debug, RAG CLI fell through the cracks

### 3. Expected Commands (from TODO.md test scenarios)

```bash
# Test 3: RAG System (line 13707)
llmspell rag ingest /tmp/test.txt --metadata '{"source": "test"}'
llmspell rag search "Lua programming" --limit 5
llmspell rag stats
```

### 4. Proposed RAG Command Structure

```bash
llmspell rag <SUBCOMMAND> [OPTIONS]

SUBCOMMANDS:
    ingest    Ingest documents into RAG system
    search    Search/query the RAG system
    index     Manage vector indices
    stats     Show RAG system statistics
    clear     Clear RAG data
    export    Export embeddings/index
    import    Import embeddings/index

EXAMPLES:
    # Ingest a document
    llmspell rag ingest document.txt --metadata '{"source": "docs"}'
    llmspell rag ingest /path/to/dir --recursive --chunk-size 512
    
    # Search
    llmspell rag search "How to use workflows?" --limit 10
    llmspell rag search "error handling" --threshold 0.8 --scope tenant:123
    
    # Index management
    llmspell rag index list
    llmspell rag index optimize
    llmspell rag index rebuild
    
    # Statistics
    llmspell rag stats
    llmspell rag stats --detailed
    
    # Clear data
    llmspell rag clear --scope session
    llmspell rag clear --all --confirm
```

### 5. Implementation Requirements

#### 5.1 CLI Definition (llmspell-cli/src/cli.rs)
```rust
/// RAG system management
Rag {
    #[command(subcommand)]
    command: RagCommands,
    
    /// Connect to external kernel
    #[arg(long)]
    connect: Option<String>,
}

#[derive(Subcommand, Debug)]
pub enum RagCommands {
    /// Ingest documents
    Ingest {
        /// File or directory to ingest
        path: PathBuf,
        
        /// Metadata as JSON
        #[arg(long)]
        metadata: Option<String>,
        
        /// Process directories recursively
        #[arg(long)]
        recursive: bool,
        
        /// Chunk size for splitting
        #[arg(long, default_value = "512")]
        chunk_size: usize,
        
        /// Scope for multi-tenancy
        #[arg(long)]
        scope: Option<String>,
    },
    
    /// Search the RAG system
    Search {
        /// Query string
        query: String,
        
        /// Maximum results
        #[arg(long, default_value = "10")]
        limit: usize,
        
        /// Similarity threshold
        #[arg(long)]
        threshold: Option<f32>,
        
        /// Scope for multi-tenancy
        #[arg(long)]
        scope: Option<String>,
    },
    
    /// Show statistics
    Stats {
        /// Show detailed statistics
        #[arg(long)]
        detailed: bool,
    },
    
    /// Clear RAG data
    Clear {
        /// Scope to clear
        #[arg(long)]
        scope: Option<String>,
        
        /// Clear all data
        #[arg(long)]
        all: bool,
        
        /// Confirm destructive operation
        #[arg(long)]
        confirm: bool,
    },
}
```

#### 5.2 Command Handler (llmspell-cli/src/commands/rag.rs)
```rust
pub async fn handle_rag_command(
    command: RagCommands,
    config: LLMSpellConfig,
    output_format: OutputFormat,
    connect: Option<String>,
) -> Result<()> {
    // Create kernel connection (RAG needs kernel for state/session)
    let kernel = create_kernel_connection(config.clone(), connect).await?;
    
    // Get RAG pipeline from kernel
    let rag_pipeline = kernel.get_rag_pipeline().await?;
    
    match command {
        RagCommands::Ingest { path, metadata, recursive, chunk_size, scope } => {
            handle_ingest(rag_pipeline, path, metadata, recursive, chunk_size, scope, output_format).await
        }
        RagCommands::Search { query, limit, threshold, scope } => {
            handle_search(rag_pipeline, query, limit, threshold, scope, output_format).await
        }
        RagCommands::Stats { detailed } => {
            handle_stats(rag_pipeline, detailed, output_format).await
        }
        RagCommands::Clear { scope, all, confirm } => {
            handle_clear(rag_pipeline, scope, all, confirm, output_format).await
        }
    }
}
```

#### 5.3 Kernel Integration (CRITICAL - Missing from Original Analysis)

The RAG commands must route through the kernel using custom Jupyter protocol messages, similar to StateRequest/SessionRequest pattern.

##### 5.3.1 Protocol Message Types (llmspell-kernel/src/jupyter/wire.rs)

```rust
// Add to MessageContent enum
pub enum MessageContent {
    // ... existing variants ...
    
    /// RAG system request
    RagRequest {
        operation: RagOperation,
        scope: Option<String>,
    },
    
    /// RAG system reply
    RagReply {
        status: String,
        data: serde_json::Value,
        error: Option<String>,
    },
}

// RAG operation types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum RagOperation {
    /// Ingest document(s)
    Ingest {
        path: String,
        content: Option<String>,  // For direct content ingestion
        metadata: Option<serde_json::Value>,
        chunk_size: usize,
        recursive: bool,
    },
    
    /// Search/query
    Search {
        query: String,
        limit: usize,
        threshold: Option<f32>,
        metadata_filter: Option<serde_json::Value>,
    },
    
    /// Get statistics
    Stats {
        detailed: bool,
    },
    
    /// Clear data
    Clear {
        scope: Option<String>,
        confirm: bool,
    },
    
    /// Index operations
    Index {
        action: IndexAction,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IndexAction {
    List,
    Optimize,
    Rebuild,
}
```

##### 5.3.2 Kernel RAG Pipeline Management (llmspell-kernel/src/kernel.rs)

```rust
pub struct GenericKernel {
    // ... existing fields ...
    
    /// RAG pipeline instance
    pub rag_pipeline: Option<Arc<RAGPipeline>>,
}

impl GenericKernel {
    /// Create RAG pipeline during kernel initialization
    async fn create_rag_pipeline(
        config: &Arc<LLMSpellConfig>,
        state_manager: &Option<Arc<StateManager>>,
    ) -> Result<Option<Arc<RAGPipeline>>> {
        if !config.rag.enabled {
            return Ok(None);
        }
        
        // Create vector storage based on config
        let storage: Arc<dyn VectorStorage> = match config.rag.backend.as_str() {
            "hnsw" => {
                let hnsw_config = HNSWConfig::from(&config.rag);
                Arc::new(HNSWVectorStorage::new(hnsw_config)?)
            }
            "memory" => Arc::new(InMemoryVectorStorage::new()),
            _ => return Err(anyhow!("Unknown RAG backend: {}", config.rag.backend))
        };
        
        // Create embedding factory
        let embedding_factory = Arc::new(EmbeddingFactory::new(config.clone())?);
        
        // Create embedding cache
        let cache_size = config.rag.cache_size.unwrap_or(1000);
        let embedding_cache = Arc::new(EmbeddingCache::new(cache_size));
        
        // Create RAG pipeline
        let pipeline = RAGPipeline::new(
            config.rag.clone(),
            storage,
            embedding_factory,
            embedding_cache,
        )?;
        
        Ok(Some(Arc::new(pipeline)))
    }
    
    /// Handle RAG requests (similar to handle_state_request)
    async fn handle_rag_request(&self, content: serde_json::Value) -> Result<serde_json::Value> {
        // Check if RAG pipeline is available
        let rag_pipeline = self.rag_pipeline.as_ref().ok_or_else(|| {
            anyhow::anyhow!("RAG system not available - no RAGPipeline configured")
        })?;
        
        // Parse the operation from the request
        let operation: RagOperation = serde_json::from_value(
            content.get("operation")
                .ok_or_else(|| anyhow::anyhow!("Missing operation field"))?
                .clone(),
        )?;
        
        // Parse optional scope
        let scope = content.get("scope")
            .and_then(|s| s.as_str())
            .map(|s| StateScope::from_str(s));
        
        // Execute RAG operation
        match operation {
            RagOperation::Ingest { path, content, metadata, chunk_size, recursive } => {
                // Read file or use provided content
                let doc_content = if let Some(content) = content {
                    content
                } else {
                    tokio::fs::read_to_string(&path).await?
                };
                
                let result = rag_pipeline.ingest_document(
                    path.clone(),
                    doc_content,
                    metadata,
                    scope,
                ).await?;
                
                Ok(serde_json::json!({
                    "status": "ok",
                    "ingested_chunks": result.chunks_created,
                    "document_id": result.document_id,
                }))
            }
            
            RagOperation::Search { query, limit, threshold, metadata_filter } => {
                let query_config = QueryConfig {
                    top_k: limit,
                    threshold,
                    metadata_filter,
                    ..Default::default()
                };
                
                let result = rag_pipeline.search(
                    query,
                    scope,
                    Some(query_config),
                ).await?;
                
                Ok(serde_json::json!({
                    "status": "ok",
                    "results": result.entries,
                    "total_found": result.entries.len(),
                }))
            }
            
            RagOperation::Stats { detailed } => {
                let stats = rag_pipeline.get_stats().await?;
                
                Ok(serde_json::json!({
                    "status": "ok",
                    "stats": stats,
                }))
            }
            
            RagOperation::Clear { scope: clear_scope, confirm } => {
                if !confirm {
                    return Ok(serde_json::json!({
                        "status": "error",
                        "message": "Clear operation requires confirmation"
                    }));
                }
                
                rag_pipeline.clear(clear_scope.or(scope)).await?;
                
                Ok(serde_json::json!({
                    "status": "ok",
                    "message": "RAG data cleared"
                }))
            }
            
            RagOperation::Index { action } => {
                match action {
                    IndexAction::List => {
                        let indices = rag_pipeline.list_indices().await?;
                        Ok(serde_json::json!({
                            "status": "ok",
                            "indices": indices,
                        }))
                    }
                    IndexAction::Optimize => {
                        rag_pipeline.optimize_index().await?;
                        Ok(serde_json::json!({
                            "status": "ok",
                            "message": "Index optimized"
                        }))
                    }
                    IndexAction::Rebuild => {
                        rag_pipeline.rebuild_index().await?;
                        Ok(serde_json::json!({
                            "status": "ok",
                            "message": "Index rebuilt"
                        }))
                    }
                }
            }
        }
    }
}
```

##### 5.3.3 CLI to Kernel Communication (llmspell-cli/src/kernel_client.rs)

```rust
impl KernelConnectionTrait for UnifiedKernelClient {
    /// Send RAG request to kernel
    async fn rag_request(
        &mut self,
        operation: Value,
        scope: Option<String>,
    ) -> Result<Value> {
        // Create RagRequest message
        let request = serde_json::json!({
            "operation": operation,
            "scope": scope,
        });
        
        // Send to kernel and wait for reply
        let reply = self.send_request("rag_request", request).await?;
        
        // Parse RagReply
        if reply["status"] == "error" {
            return Err(anyhow::anyhow!(
                "RAG operation failed: {}",
                reply["error"].as_str().unwrap_or("Unknown error")
            ));
        }
        
        Ok(reply["data"].clone())
    }
}
```

##### 5.3.4 Configuration Flow

```
1. CLI parses RAG command
2. CLI loads LLMSpellConfig (includes RAG settings)
3. CLI creates/connects to kernel with config
4. Kernel initializes RAGPipeline based on config.rag
5. CLI sends RagRequest via Jupyter protocol
6. Kernel processes request using RAGPipeline
7. Kernel sends RagReply back to CLI
8. CLI formats and displays results
```

##### 5.3.5 Message Flow Example

```
CLI: llmspell rag search "How to use workflows?" --limit 5

1. CLI creates RagRequest:
   {
     "operation": {
       "type": "Search",
       "query": "How to use workflows?",
       "limit": 5,
       "threshold": null,
       "metadata_filter": null
     },
     "scope": null
   }

2. Kernel receives on shell channel, routes to handle_rag_request()

3. Kernel executes RAGPipeline::search()

4. Kernel creates RagReply:
   {
     "status": "ok",
     "data": {
       "results": [...],
       "total_found": 5
     },
     "error": null
   }

5. CLI receives reply, formats as text/json based on --output flag
```

### 6. Architecture Considerations

#### Option A: Direct RAG Commands (Recommended)
- **Pros**: Clear UX, follows subcommand pattern, testable
- **Cons**: Requires kernel connection, adds complexity

#### Option B: Keep Profile-Only Design
- **Pros**: Simpler, already works
- **Cons**: Poor UX for ingestion, no direct search

#### Option C: Hybrid Approach
- Basic commands for common operations (ingest, search)
- Advanced features via scripts with --rag-profile

### 7. Testing Requirements

```bash
# Integration tests needed
cargo test --test rag_integration_test

# Test scenarios:
1. Ingest single file
2. Ingest directory recursively  
3. Search with various parameters
4. Multi-tenant isolation
5. Clear and rebuild index
6. Stats accuracy
```

## Recommendation

**Implement Option A (Direct RAG Commands)** because:
1. Matches user expectations from TODO.md tests
2. Provides better UX for common RAG operations
3. Aligns with existing subcommand patterns (state, session, kernel)
4. Can leverage existing RAGPipeline implementation
5. Testable and maintainable

## Implementation Summary

### Key Architectural Requirements

1. **Custom Jupyter Protocol Messages**: 
   - Add `RagRequest` and `RagReply` to `MessageContent` enum
   - Define `RagOperation` enum for all RAG operations
   - Follow existing StateRequest/SessionRequest pattern

2. **Kernel RAG Pipeline Management**:
   - Kernel creates and owns `RAGPipeline` instance
   - Initialized during kernel startup based on config.rag
   - Handles all RAG operations through `handle_rag_request()`

3. **CLI Command Structure**:
   - New `Rag` command with subcommands (ingest, search, stats, clear, index)
   - Routes through kernel connection (embedded or external)
   - Uses `rag_request()` method on `KernelConnectionTrait`

4. **Configuration Flow**:
   - RAG config passed from CLI → Kernel via `LLMSpellConfig`
   - Kernel creates appropriate vector storage backend (HNSW/memory)
   - Embedding provider configured based on config.providers

5. **Message Routing**:
   - CLI sends RagRequest on shell channel
   - Kernel processes with RAGPipeline
   - Kernel sends RagReply back to CLI
   - CLI formats output based on --output flag

### Architecture Diagram

```
┌─────────────────┐
│   CLI (User)    │
│ llmspell rag    │
│    search       │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  CLI Handler    │
│ commands/rag.rs │
└────────┬────────┘
         │
         ▼
┌─────────────────┐       ┌─────────────────┐
│ KernelClient    │──────▶│ Jupyter Protocol│
│ rag_request()   │       │   RagRequest    │
└─────────────────┘       └────────┬────────┘
                                   │
                          ▼────────▼────────▼
                    ┌───────────────────────────┐
                    │      GenericKernel        │
                    │  handle_rag_request()     │
                    ├───────────────────────────┤
                    │ rag_pipeline: RAGPipeline │
                    │ state_manager: StateManager│
                    │ session_manager: SessionMgr│
                    └───────────┬───────────────┘
                                │
                    ┌───────────▼───────────┐
                    │     RAGPipeline       │
                    │  - ingest_document()  │
                    │  - search()           │
                    │  - get_stats()        │
                    └───────────┬───────────┘
                                │
                    ┌───────────▼───────────┐
                    │   Vector Storage      │
                    │   (HNSW/Memory)       │
                    └───────────────────────┘
```

### Why Kernel Integration is Required

1. **State/Session Integration**: RAG needs access to StateManager for multi-tenant scoping
2. **Configuration Consistency**: Kernel manages all stateful components
3. **Resource Management**: Single RAGPipeline instance shared across operations
4. **Protocol Compliance**: Follows Jupyter messaging patterns for all operations
5. **Unified Architecture**: All stateful operations (state, session, RAG) go through kernel

## Next Steps

1. ✅ Analysis complete - RAG commands missing due to design gap
2. ✅ Kernel integration architecture designed
3. ⏳ Implement protocol messages in llmspell-kernel/src/jupyter/wire.rs
4. ⏳ Add handle_rag_request to llmspell-kernel/src/kernel.rs
5. ⏳ Implement CLI commands in llmspell-cli/src/commands/rag.rs
6. ⏳ Add rag_request to KernelConnectionTrait
7. ⏳ Write integration tests
8. ⏳ Update TODO.md when complete

## Files to Modify

1. `llmspell-kernel/src/jupyter/wire.rs` - Add RagRequest/RagReply messages
2. `llmspell-kernel/src/kernel.rs` - Add RAGPipeline creation and handler
3. `llmspell-cli/src/cli.rs` - Add Rag command definition
4. `llmspell-cli/src/commands/mod.rs` - Add rag module
5. `llmspell-cli/src/commands/rag.rs` - NEW - Implement command handlers
6. `llmspell-cli/src/kernel_client.rs` - Add rag_request method
7. `tests/rag_integration_test.rs` - NEW - Integration tests