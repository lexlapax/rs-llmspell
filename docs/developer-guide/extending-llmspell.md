# Extending rs-llmspell: The Complete Guide

✅ **CURRENT**: Phase 8 + 13b.16 - Comprehensive extension guide for tools, agents, hooks, workflows, RAG, and storage backends
**Version**: 0.14.0 | **Focus**: Building production extensions

**Quick Navigation**: [Tools](#part-1-tool-development) | [Agents](#part-2-agent-development) | [Hooks](#part-3-hook-development) | [Workflows](#part-4-workflow-development) | [RAG](#part-5-rag-system-extension) | [Storage](#part-6-storage-backend-extension-new---phase-13b16)

---

## Overview

This guide covers **ALL** extension patterns for rs-llmspell, from simple tools to complex RAG pipelines and storage backends. Each part builds on the previous, showing how components integrate.

**What You'll Learn**:
- Tool development with llmspell-utils patterns
- Agent creation with BaseAgent trait
- Hook system integration for cross-cutting concerns
- Workflow orchestration patterns
- RAG pipeline extension (NEW in Phase 8)
- Storage backend implementation (NEW in Phase 13b.16)

---

## PART 1: Tool Development

### Tool Architecture

Tools are the foundation - reusable components that perform specific tasks. Currently **37 tools** across 10 categories.

```rust
use llmspell_core::traits::{BaseAgent, Tool, ToolCategory, SecurityLevel};
use llmspell_utils::params::{extract_parameters, extract_required_string};
use llmspell_utils::response::ResponseBuilder;
use llmspell_utils::error_builders::llmspell::{component_error, validation_error};
```

### Complete Tool Example

**Study**: `llmspell-tools/src/web/web_scraper.rs` for production patterns

```rust
#[derive(Debug, Clone)]
pub struct CustomTool {
    metadata: ComponentMetadata,
    config: CustomToolConfig,
}

impl CustomTool {
    pub fn new(config: CustomToolConfig) -> Self {
        Self {
            metadata: ComponentMetadata::new(
                "custom-tool".to_string(),
                "Does something useful".to_string(),
            ),
            config,
        }
    }
}

// BaseAgent implementation (foundation for all components)
impl BaseAgent for CustomTool {
    fn metadata(&self) -> &ComponentMetadata { 
        &self.metadata 
    }
    
    async fn execute(&self, input: AgentInput, _ctx: ExecutionContext) 
        -> Result<AgentOutput> {
        // 1. Extract parameters using llmspell-utils
        let params = extract_parameters(&input)?;
        let operation = extract_required_string(params, "operation")?;
        let path = extract_required_string(params, "path")?;
        let timeout = extract_optional_u64(params, "timeout_ms")
            .unwrap_or(30000);
        
        // 2. Validate inputs
        if path.is_empty() {
            return Err(validation_error(
                "Path cannot be empty",
                Some("path".to_string())
            ));
        }
        
        // 3. Perform operation
        let result = match operation {
            "read" => self.read_file(path).await?,
            "write" => self.write_file(path, params).await?,
            _ => return Err(validation_error(
                format!("Unknown operation: {}", operation),
                Some("operation".to_string())
            ))
        };
        
        // 4. Build response using ResponseBuilder
        let response = ResponseBuilder::success("custom-tool")
            .with_result(json!(result))
            .with_metadata("operation", json!(operation))
            .with_duration_ms(start.elapsed().as_millis() as u64)
            .build();
            
        Ok(AgentOutput::tool_result(response))
    }
}

// Tool trait implementation
impl Tool for CustomTool {
    fn category(&self) -> ToolCategory { 
        ToolCategory::Utility 
    }
    
    fn security_level(&self) -> SecurityLevel { 
        SecurityLevel::Restricted  // Safe, Restricted, or Privileged
    }
    
    fn security_requirements(&self) -> SecurityRequirements {
        SecurityRequirements::restricted()
            .with_file_access("/tmp")
            .with_network_access("api.example.com")
    }
    
    fn resource_limits(&self) -> ResourceLimits {
        ResourceLimits::default()
            .with_memory_limit(50 * 1024 * 1024)  // 50MB
            .with_cpu_limit(10000)  // 10 seconds
    }
}
```

### File System Tools (Special Pattern)

**CRITICAL**: File tools MUST use bridge-provided sandbox

```rust
use llmspell_security::sandbox::FileSandbox;
use std::sync::Arc;

#[derive(Clone)]
pub struct FileSystemTool {
    metadata: ComponentMetadata,
    sandbox: Arc<FileSandbox>,  // ✅ Bridge-provided, never create your own
}

impl FileSystemTool {
    // Constructor MUST accept sandbox
    pub fn new(sandbox: Arc<FileSandbox>) -> Self {
        Self {
            metadata: ComponentMetadata::new(
                "file-operations".to_string(),
                "Safe file operations".to_string(),
            ),
            sandbox,
        }
    }
    
    async fn safe_read(&self, path: &str) -> Result<String> {
        // Use bridge sandbox for validation
        let validated_path = self.sandbox.validate_path(Path::new(path))?;
        tokio::fs::read_to_string(validated_path).await
            .map_err(|e| component_error(format!("Read failed: {}", e)))
    }
}
```

### Tool Registration

In `llmspell-bridge/src/tools.rs`:

```rust
pub fn register_all_tools(
    registry: &Arc<ComponentRegistry>,
    sandbox: Arc<FileSandbox>,
) -> Result<()> {
    // Standard tools (use kebab-case for tool names)
    registry.register_tool("custom-tool", Arc::new(CustomTool::new(config)));

    // File system tools with sandbox
    registry.register_tool("file-operations", Arc::new(FileSystemTool::new(sandbox.clone())));

    Ok(())
}
```

### Tool Testing Pattern

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_testing::tool_helpers::{create_test_tool_input};
    
    #[tokio::test]
    #[cfg_attr(test_category = "unit")]
    #[cfg_attr(test_category = "tool")]
    async fn test_custom_tool_operation() {
        let tool = CustomTool::new(Default::default());
        
        let input = create_test_tool_input(vec![
            ("operation", "read"),
            ("path", "/tmp/test.txt"),
        ]);
        
        let result = tool.execute(input, Default::default()).await;
        assert!(result.is_ok());
        
        let output = result.unwrap();
        assert!(output.tool_calls[0].result.is_some());
    }
}
```

---

## PART 2: Agent Development

### Agent Architecture

Agents build on BaseAgent to provide LLM-powered reasoning. Study `examples/rust-developers/custom-agent-example/`.

```rust
use llmspell_core::traits::BaseAgent;
use llmspell_providers::Provider;

pub struct CustomAgent {
    metadata: ComponentMetadata,
    provider: Arc<dyn Provider>,
    tools: Vec<Arc<dyn Tool>>,
    temperature: f32,
}

impl BaseAgent for CustomAgent {
    fn metadata(&self) -> &ComponentMetadata { 
        &self.metadata 
    }
    
    async fn execute(&self, input: AgentInput, ctx: ExecutionContext) 
        -> Result<AgentOutput> {
        // 1. Prepare prompt with context
        let prompt = self.prepare_prompt(&input)?;
        
        // 2. Call LLM provider
        let response = self.provider
            .complete(prompt)
            .with_temperature(self.temperature)
            .with_tools(&self.tools)
            .execute()
            .await?;
        
        // 3. Process response
        let output = self.process_response(response)?;
        
        Ok(AgentOutput::text(output))
    }
    
    async fn validate_input(&self, input: &AgentInput) -> Result<()> {
        if input.text.is_empty() && input.parameters.is_empty() {
            return Err(validation_error(
                "Either text or parameters required",
                None
            ));
        }
        Ok(())
    }
    
    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
        // Graceful error handling
        Ok(AgentOutput::text(format!(
            "I encountered an issue: {}. Let me try a different approach.",
            error
        )))
    }
}
```

### Agent Builder Pattern

```rust
pub struct AgentBuilder {
    name: Option<String>,
    provider: Option<Arc<dyn Provider>>,
    tools: Vec<Arc<dyn Tool>>,
    temperature: f32,
    system_prompt: Option<String>,
}

impl AgentBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            provider: None,
            tools: Vec::new(),
            temperature: 0.7,
            system_prompt: None,
        }
    }
    
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
    
    pub fn provider(mut self, provider: Arc<dyn Provider>) -> Self {
        self.provider = Some(provider);
        self
    }
    
    pub fn add_tool(mut self, tool: Arc<dyn Tool>) -> Self {
        self.tools.push(tool);
        self
    }
    
    pub async fn build(self) -> Result<CustomAgent> {
        let name = self.name.ok_or_else(|| 
            validation_error("Agent name required", None))?;
        let provider = self.provider.ok_or_else(|| 
            validation_error("Provider required", None))?;
            
        Ok(CustomAgent {
            metadata: ComponentMetadata::new(name, "Custom agent"),
            provider,
            tools: self.tools,
            temperature: self.temperature,
        })
    }
}
```

### Multi-Agent Coordination

```rust
pub struct CoordinatorAgent {
    metadata: ComponentMetadata,
    agents: HashMap<String, Arc<dyn BaseAgent>>,
}

impl CoordinatorAgent {
    pub async fn delegate_task(&self, task: &str, input: AgentInput) 
        -> Result<AgentOutput> {
        // Select appropriate agent
        let agent = self.select_agent_for_task(task)?;
        
        // Execute with monitoring
        let start = Instant::now();
        let result = agent.execute(input, ExecutionContext::default()).await?;
        
        // Log coordination metrics
        tracing::info!(
            agent = agent.metadata().name,
            duration_ms = start.elapsed().as_millis(),
            "Task delegated and completed"
        );
        
        Ok(result)
    }
}
```

---

## PART 3: Hook Development

### Hook Architecture

Hooks provide cross-cutting concerns like logging, security, caching. They intercept execution at defined points.

```rust
use async_trait::async_trait;
use llmspell_core::hooks::{Hook, HookContext, HookResult};

#[async_trait]
pub trait Hook: Send + Sync {
    fn id(&self) -> &str;
    
    async fn execute(&self, context: &mut HookContext) -> Result<HookResult>;
    
    async fn on_register(&self) -> Result<()> { Ok(()) }
    async fn on_unregister(&self) -> Result<()> { Ok(()) }
}
```

### Security Validation Hook

```rust
use regex::Regex;
use once_cell::sync::Lazy;

static SQL_INJECTION_PATTERNS: Lazy<Vec<Regex>> = Lazy::new(|| vec![
    Regex::new(r"(?i)\b(union|select|insert|update|delete)\b").unwrap(),
]);

pub struct SecurityHook {
    id: String,
    strict_mode: bool,
}

#[async_trait]
impl Hook for SecurityHook {
    fn id(&self) -> &str { &self.id }
    
    async fn execute(&self, context: &mut HookContext) -> Result<HookResult> {
        // Extract input
        let input = context.data
            .get("input")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow!("No input found"))?;
        
        // Check for SQL injection
        for pattern in &*SQL_INJECTION_PATTERNS {
            if pattern.is_match(input) {
                tracing::warn!(
                    component = %context.component_id.name,
                    "SQL injection detected"
                );
                
                if self.strict_mode {
                    return Ok(HookResult::Cancel {
                        reason: "Security violation detected".to_string()
                    });
                } else {
                    // Sanitize and continue
                    let sanitized = self.sanitize_input(input);
                    return Ok(HookResult::Modified {
                        data: json!({ "input": sanitized })
                    });
                }
            }
        }
        
        Ok(HookResult::Continue)
    }
}
```

### Caching Hook with Redis

```rust
use redis::aio::ConnectionManager;
use sha2::{Sha256, Digest};

#[derive(Clone)]
pub struct CachingHook {
    id: String,
    redis: ConnectionManager,
    ttl: Duration,
}

impl CachingHook {
    fn generate_key(&self, context: &HookContext) -> String {
        let mut hasher = Sha256::new();
        hasher.update(context.component_id.name.as_bytes());
        
        if let Some(input) = context.data.get("input") {
            hasher.update(serde_json::to_vec(input).unwrap_or_default());
        }
        
        format!("llmspell:cache:{:x}", hasher.finalize())
    }
}

#[async_trait]
impl Hook for CachingHook {
    fn id(&self) -> &str { &self.id }
    
    async fn execute(&self, context: &mut HookContext) -> Result<HookResult> {
        let key = self.generate_key(context);
        
        // Try cache hit
        if let Ok(cached) = redis::cmd("GET")
            .arg(&key)
            .query_async::<_, String>(&mut self.redis.clone())
            .await 
        {
            if let Ok(data) = serde_json::from_str(&cached) {
                return Ok(HookResult::Modified { data });
            }
        }
        
        // Store result in post-execution hook
        context.metadata.insert("cache_key".to_string(), json!(key));
        
        Ok(HookResult::Continue)
    }
}
```

### Hook Registration

```rust
use llmspell_hooks::HookManager;

pub async fn register_hooks(manager: &HookManager) -> Result<()> {
    // Security hook with high priority
    manager.register(
        Arc::new(SecurityHook::new()),
        HookPriority::High
    ).await?;
    
    // Caching hook with normal priority
    manager.register(
        Arc::new(CachingHook::new().await?),
        HookPriority::Normal
    ).await?;
    
    Ok(())
}
```

---

## PART 4: Workflow Development

### Workflow Types

Four core workflow patterns with multi-agent support:

```rust
use llmspell_workflows::{Workflow, WorkflowStep, WorkflowResult};
```

### Sequential Workflow

```rust
let workflow = Workflow::sequential()
    .name("data_pipeline")
    .add_step(WorkflowStep::Tool {
        name: "file_reader".to_string(),
        params: json!({ "path": "input.txt" })
    })
    .add_step(WorkflowStep::Agent {
        id: "processor".to_string(),
        params: json!({ "task": "analyze" })
    })
    .add_step(WorkflowStep::Tool {
        name: "file_writer".to_string(),
        params: json!({ "path": "output.txt" })
    })
    .build()?;

let result = workflow.execute(input).await?;
```

### Parallel Workflow

```rust
let workflow = Workflow::parallel()
    .name("multi_analysis")
    .add_branch("sentiment", WorkflowStep::Agent {
        id: "sentiment_analyzer".to_string(),
        params: json!({})
    })
    .add_branch("facts", WorkflowStep::Agent {
        id: "fact_checker".to_string(),
        params: json!({})
    })
    .join_strategy(JoinStrategy::Merge)  // or First, All
    .build()?;
```

### Conditional Workflow

```rust
let workflow = Workflow::conditional()
    .name("smart_router")
    .condition(|input| input.get("priority").unwrap_or(&json!(0)).as_i64().unwrap() > 5)
    .then_branch(WorkflowStep::Agent {
        id: "urgent_handler".to_string(),
        params: json!({})
    })
    .else_branch(WorkflowStep::Agent {
        id: "normal_handler".to_string(),
        params: json!({})
    })
    .build()?;
```

### Multi-Agent Patterns

```rust
// Pipeline Pattern - Sequential processing
let pipeline = Workflow::multi_agent_pipeline()
    .name("research_pipeline")
    .add_agent("researcher", json!({ "depth": "comprehensive" }))
    .add_agent("analyst", json!({ "focus": "insights" }))
    .add_agent("writer", json!({ "style": "technical" }))
    .build()?;

// Fork-Join Pattern - Parallel execution
let parallel = Workflow::multi_agent_fork_join()
    .name("document_analysis")
    .add_task("sentiment", "sentiment_agent")
    .add_task("facts", "fact_checker")
    .add_task("style", "style_agent")
    .coordinator("result_aggregator")
    .build()?;

// Consensus Pattern - Multiple evaluators
let consensus = Workflow::multi_agent_consensus()
    .name("investment_decision")
    .add_evaluator("financial_expert")
    .add_evaluator("market_expert")
    .add_evaluator("risk_expert")
    .consensus_threshold(0.7)  // 70% agreement
    .build()?;
```

---

## PART 5: RAG System Extension (NEW - Phase 8)

### RAG Architecture Overview

The RAG system provides vector storage, embeddings, and retrieval for augmented generation.

```rust
use llmspell_rag::prelude::*;
use llmspell_storage::{VectorStorage, HNSWConfig};
```

### RAG Pipeline Builder

**Pattern from**: `llmspell-rag/src/pipeline/builder.rs`

```rust
use llmspell_rag::pipeline::{RAGPipelineBuilder, RAGConfig};

// Build a complete RAG pipeline
let pipeline = RAGPipelineBuilder::new()
    .with_config(RAGConfig {
        collection: "knowledge_base".to_string(),
        chunk_size: 512,
        chunk_overlap: 64,
        embedding_model: "text-embedding-ada-002".to_string(),
        search_limit: 10,
    })
    .with_storage(Arc::new(hnsw_storage))
    .with_embedding_factory(Arc::new(embedding_factory))
    .with_embedding_cache(Arc::new(cache))
    .build()
    .await?;
```

### Custom Embedding Provider

```rust
use llmspell_rag::embeddings::{EmbeddingProvider, EmbeddingModel};

pub struct CustomEmbeddingProvider {
    api_client: CustomApiClient,
    model: String,
}

#[async_trait]
impl EmbeddingProvider for CustomEmbeddingProvider {
    async fn embed_text(&self, text: &str) -> Result<Vec<f32>> {
        let response = self.api_client
            .embed(text, &self.model)
            .await?;
        Ok(response.embedding)
    }
    
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        let response = self.api_client
            .embed_batch(texts, &self.model)
            .await?;
        Ok(response.embeddings)
    }
    
    fn dimensions(&self) -> usize {
        match self.model.as_str() {
            "small-model" => 384,
            "large-model" => 1536,
            _ => 768,
        }
    }
}
```

### Custom Vector Storage Backend

```rust
use llmspell_storage::{VectorStorage, VectorEntry, VectorQuery, VectorResult};

pub struct CustomVectorStorage {
    backend: CustomBackend,
    config: StorageConfig,
}

#[async_trait]
impl VectorStorage for CustomVectorStorage {
    async fn insert(&self, entry: VectorEntry) -> Result<String> {
        // Validate dimensions
        if entry.vector.len() != self.config.dimensions {
            return Err(anyhow!("Dimension mismatch"));
        }
        
        // Store in backend
        let id = self.backend.store(
            &entry.id,
            &entry.vector,
            &entry.metadata,
            entry.scope.as_ref()
        ).await?;
        
        Ok(id)
    }
    
    async fn search(&self, query: VectorQuery) -> Result<Vec<VectorResult>> {
        // Perform similarity search
        let results = self.backend.search(
            &query.vector,
            query.limit,
            query.threshold,
            query.scope.as_ref()
        ).await?;
        
        // Convert to VectorResult
        Ok(results.into_iter().map(|r| VectorResult {
            id: r.id,
            score: r.similarity,
            vector: r.vector,
            metadata: r.metadata,
        }).collect())
    }
    
    async fn get_stats(&self) -> Result<StorageStats> {
        Ok(StorageStats {
            total_vectors: self.backend.count().await?,
            dimensions: self.config.dimensions,
            memory_bytes: self.backend.memory_usage().await?,
        })
    }
}
```

### HNSW Configuration and Tuning

```rust
use llmspell_storage::HNSWConfig;

// Balanced configuration (default)
let config = HNSWConfig::balanced();

// Performance-optimized (faster search, more memory)
let config = HNSWConfig::performance()
    .with_m(32)                    // More connections (default: 16)
    .with_ef_construction(400)     // Better index quality (default: 200)
    .with_ef_search(200);          // More accurate search (default: 100)

// Accuracy-optimized (better recall, slower)
let config = HNSWConfig::accuracy()
    .with_m(48)                    // Maximum connections
    .with_ef_construction(500)     // Best index quality
    .with_ef_search(300);          // Most accurate search

// Memory-optimized (less memory, reasonable performance)
let config = HNSWConfig::memory_optimized()
    .with_m(8)                     // Fewer connections
    .with_ef_construction(100)     // Faster build
    .with_ef_search(50);           // Faster search
```

### Custom Chunking Strategy

```rust
use llmspell_rag::chunking::{ChunkingStrategy, Chunk};

pub struct SemanticChunker {
    max_chunk_size: usize,
    overlap: usize,
    sentence_detector: SentenceDetector,
}

impl ChunkingStrategy for SemanticChunker {
    fn chunk(&self, text: &str) -> Vec<Chunk> {
        let sentences = self.sentence_detector.detect(text);
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let mut current_sentences = Vec::new();
        
        for sentence in sentences {
            if current_chunk.len() + sentence.len() > self.max_chunk_size {
                // Save current chunk
                if !current_chunk.is_empty() {
                    chunks.push(Chunk {
                        text: current_chunk.clone(),
                        metadata: json!({
                            "sentences": current_sentences.len(),
                            "start_pos": chunks.len() * self.max_chunk_size,
                        }),
                    });
                    
                    // Keep overlap
                    let overlap_sentences = current_sentences
                        .iter()
                        .rev()
                        .take_while(|s| s.len() < self.overlap)
                        .collect::<Vec<_>>();
                    
                    current_chunk = overlap_sentences.iter()
                        .rev()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>()
                        .join(" ");
                    current_sentences = overlap_sentences.into_iter()
                        .rev()
                        .cloned()
                        .collect();
                }
            }
            
            current_chunk.push_str(&sentence);
            current_chunk.push(' ');
            current_sentences.push(sentence);
        }
        
        // Add final chunk
        if !current_chunk.is_empty() {
            chunks.push(Chunk {
                text: current_chunk,
                metadata: json!({
                    "sentences": current_sentences.len(),
                    "is_final": true,
                }),
            });
        }
        
        chunks
    }
}
```

### Multi-Tenant RAG Pattern

```rust
use llmspell_state_traits::StateScope;
use llmspell_tenancy::TenantManager;

pub struct MultiTenantRAG {
    pipeline: Arc<RAGPipeline>,
    tenant_manager: Arc<TenantManager>,
}

impl MultiTenantRAG {
    pub async fn ingest_for_tenant(
        &self, 
        tenant_id: &str, 
        document: Document
    ) -> Result<String> {
        // Validate tenant
        let tenant = self.tenant_manager.get_tenant(tenant_id)?;
        
        // Check quotas
        if !tenant.can_ingest(document.size()) {
            return Err(anyhow!("Quota exceeded"));
        }
        
        // Add tenant scope to document
        let scoped_document = document
            .with_scope(StateScope::Custom(format!("tenant:{}", tenant_id)));
        
        // Ingest with tenant isolation
        let doc_id = self.pipeline.ingest(scoped_document).await?;
        
        // Update usage
        tenant.record_ingestion(document.size());
        
        Ok(doc_id)
    }
    
    pub async fn search_for_tenant(
        &self, 
        tenant_id: &str, 
        query: &str,
        limit: usize
    ) -> Result<Vec<SearchResult>> {
        // Create scoped query
        let scoped_query = VectorQuery::new(query, limit)
            .with_scope(StateScope::Custom(format!("tenant:{}", tenant_id)));
        
        // Search with tenant isolation
        let results = self.pipeline.search(scoped_query).await?;
        
        // Verify no cross-tenant leakage
        for result in &results {
            if !result.metadata.get("tenant_id")
                .and_then(|v| v.as_str())
                .map(|id| id == tenant_id)
                .unwrap_or(false) 
            {
                tracing::error!("Cross-tenant data leak detected!");
                return Err(anyhow!("Security violation"));
            }
        }
        
        Ok(results)
    }
}
```

---

## Testing Extension Components

### Tool Testing

```rust
#[tokio::test]
#[cfg_attr(test_category = "integration")]
#[cfg_attr(test_category = "tool")]
async fn test_tool_with_rag() {
    use llmspell_testing::tool_helpers::create_test_tool_input;
    use llmspell_testing::rag_helpers::create_test_rag_pipeline;
    
    let pipeline = create_test_rag_pipeline().await;
    let tool = RAGTool::new(pipeline);
    
    let input = create_test_tool_input(vec![
        ("operation", "search"),
        ("query", "test query"),
    ]);
    
    let result = tool.execute(input, Default::default()).await?;
    assert!(result.tool_calls[0].result.is_some());
}
```

### RAG Pipeline Testing

```rust
#[tokio::test]
#[cfg_attr(test_category = "integration")]
#[cfg_attr(test_category = "rag")]
async fn test_rag_pipeline_e2e() {
    let pipeline = RAGPipelineBuilder::new()
        .with_config(test_config())
        .with_storage(Arc::new(InMemoryVectorStorage::new()))
        .with_embedding_factory(Arc::new(MockEmbeddingFactory::new()))
        .with_embedding_cache(Arc::new(NoOpCache::new()))
        .build()
        .await?;
    
    // Ingest document
    let doc_id = pipeline.ingest(Document {
        content: "Test document content".to_string(),
        metadata: json!({ "source": "test" }),
    }).await?;
    
    // Search
    let results = pipeline.search("test query", 5).await?;
    assert!(!results.is_empty());
}
```

---

## PART 6: Storage Backend Extension (NEW - Phase 13b.16)

### Storage Architecture Overview

rs-llmspell uses a **3-tier storage architecture** with hot-swappable backends:

1. **Component APIs**: Domain-specific storage interfaces (Vector, Memory, Session, etc.)
2. **StorageBackend Trait**: Unified abstraction for key-value + vector operations
3. **Backend Implementations**: Memory, Sled (embedded DB), PostgreSQL (production)

**Supported Backends** (Phase 13b.16):
- **Memory**: In-memory HashMap (development/testing)
- **Sled**: Embedded database (single-node production)
- **PostgreSQL 18**: Production-grade with HNSW vector similarity (multi-tenant, ACID)

### StorageBackend Trait

**Location**: `llmspell-storage/src/backend.rs`

**Purpose**: Unified interface for all storage backends, enabling hot-swappable storage without code changes.

```rust
use async_trait::async_trait;
use llmspell_storage::{StorageResult, StorageError, StorageStats, BatchOperation, BatchResult};

#[async_trait]
pub trait StorageBackend: Send + Sync + std::fmt::Debug {
    /// Get value by key
    async fn get(&self, key: &str) -> StorageResult<Option<Vec<u8>>>;

    /// Set key-value pair
    async fn set(&self, key: &str, value: Vec<u8>) -> StorageResult<()>;

    /// Delete key
    async fn delete(&self, key: &str) -> StorageResult<()>;

    /// Check if key exists
    async fn exists(&self, key: &str) -> StorageResult<bool>;

    /// List keys with optional prefix filter
    async fn list_keys(&self, prefix: Option<&str>) -> StorageResult<Vec<String>>;

    /// Clear all data (USE WITH CAUTION)
    async fn clear(&self) -> StorageResult<()>;

    /// Get storage statistics
    async fn stats(&self) -> StorageResult<StorageStats> {
        Ok(StorageStats::default())
    }

    /// Batch operations (optional optimization)
    async fn batch(&self, ops: Vec<BatchOperation>) -> StorageResult<Vec<BatchResult>> {
        // Default serial implementation provided
        let mut results = Vec::new();
        for op in ops {
            let result = match op {
                BatchOperation::Set { key, value } => {
                    self.set(&key, value).await.map(|_| BatchResult::Success)
                }
                BatchOperation::Delete { key } => {
                    self.delete(&key).await.map(|_| BatchResult::Success)
                }
            };
            results.push(result.unwrap_or(BatchResult::Failed));
        }
        Ok(results)
    }
}
```

### Implementing a Custom Backend

**Step 1**: Create backend struct with connection/state

```rust
use llmspell_storage::{StorageBackend, StorageResult, StorageError};
use std::collections::HashMap;
use tokio::sync::RwLock;

#[derive(Debug)]
pub struct RedisBackend {
    client: redis::Client,
    connection_pool: redis::aio::ConnectionManager,
    stats: Arc<RwLock<BackendStats>>,
}

impl RedisBackend {
    pub async fn new(redis_url: &str) -> StorageResult<Self> {
        let client = redis::Client::open(redis_url)
            .map_err(|e| StorageError::Connection(format!("Redis connection failed: {}", e)))?;

        let connection_pool = redis::aio::ConnectionManager::new(client.clone())
            .await
            .map_err(|e| StorageError::Connection(format!("Pool creation failed: {}", e)))?;

        Ok(Self {
            client,
            connection_pool,
            stats: Arc::new(RwLock::new(BackendStats::default())),
        })
    }
}
```

**Step 2**: Implement StorageBackend trait

```rust
#[async_trait]
impl StorageBackend for RedisBackend {
    async fn get(&self, key: &str) -> StorageResult<Option<Vec<u8>>> {
        use redis::AsyncCommands;

        let mut conn = self.connection_pool.clone();

        let value: Option<Vec<u8>> = conn.get(key).await
            .map_err(|e| StorageError::Read(format!("Redis GET failed: {}", e)))?;

        // Update stats
        let mut stats = self.stats.write().await;
        stats.reads += 1;
        if value.is_some() {
            stats.hits += 1;
        } else {
            stats.misses += 1;
        }

        Ok(value)
    }

    async fn set(&self, key: &str, value: Vec<u8>) -> StorageResult<()> {
        use redis::AsyncCommands;

        let mut conn = self.connection_pool.clone();

        conn.set(key, value).await
            .map_err(|e| StorageError::Write(format!("Redis SET failed: {}", e)))?;

        // Update stats
        let mut stats = self.stats.write().await;
        stats.writes += 1;

        Ok(())
    }

    async fn delete(&self, key: &str) -> StorageResult<()> {
        use redis::AsyncCommands;

        let mut conn = self.connection_pool.clone();

        let _: () = conn.del(key).await
            .map_err(|e| StorageError::Delete(format!("Redis DEL failed: {}", e)))?;

        // Update stats
        let mut stats = self.stats.write().await;
        stats.deletes += 1;

        Ok(())
    }

    async fn exists(&self, key: &str) -> StorageResult<bool> {
        use redis::AsyncCommands;

        let mut conn = self.connection_pool.clone();

        let exists: bool = conn.exists(key).await
            .map_err(|e| StorageError::Read(format!("Redis EXISTS failed: {}", e)))?;

        Ok(exists)
    }

    async fn list_keys(&self, prefix: Option<&str>) -> StorageResult<Vec<String>> {
        use redis::AsyncCommands;

        let mut conn = self.connection_pool.clone();

        let pattern = prefix.map(|p| format!("{}*", p)).unwrap_or_else(|| "*".to_string());

        let keys: Vec<String> = conn.keys(pattern).await
            .map_err(|e| StorageError::Read(format!("Redis KEYS failed: {}", e)))?;

        Ok(keys)
    }

    async fn clear(&self) -> StorageResult<()> {
        use redis::AsyncCommands;

        let mut conn = self.connection_pool.clone();

        let _: () = conn.flushdb().await
            .map_err(|e| StorageError::Delete(format!("Redis FLUSHDB failed: {}", e)))?;

        Ok(())
    }

    async fn stats(&self) -> StorageResult<StorageStats> {
        let stats = self.stats.read().await;
        Ok(StorageStats {
            total_keys: stats.writes - stats.deletes,
            total_size_bytes: 0,  // Redis INFO command could provide this
            reads: stats.reads,
            writes: stats.writes,
            deletes: stats.deletes,
            hit_rate: if stats.reads > 0 {
                stats.hits as f64 / stats.reads as f64
            } else {
                0.0
            },
        })
    }

    // Optional: Implement optimized batch operations using Redis pipelining
    async fn batch(&self, ops: Vec<BatchOperation>) -> StorageResult<Vec<BatchResult>> {
        use redis::AsyncCommands;

        let mut conn = self.connection_pool.clone();
        let mut pipe = redis::pipe();

        // Build pipeline
        for op in &ops {
            match op {
                BatchOperation::Set { key, value } => {
                    pipe.set(key, value).ignore();
                }
                BatchOperation::Delete { key } => {
                    pipe.del(key).ignore();
                }
            }
        }

        // Execute pipeline
        pipe.query_async(&mut conn).await
            .map_err(|e| StorageError::Write(format!("Pipeline failed: {}", e)))?;

        // All operations succeeded
        Ok(vec![BatchResult::Success; ops.len()])
    }
}
```

**Step 3**: Handle backend-specific errors

```rust
#[derive(Debug, thiserror::Error)]
pub enum RedisBackendError {
    #[error("Redis connection failed: {0}")]
    Connection(String),

    #[error("Redis operation timeout: {0}")]
    Timeout(String),

    #[error("Redis cluster error: {0}")]
    Cluster(String),
}

// Implement conversion to StorageError
impl From<RedisBackendError> for StorageError {
    fn from(err: RedisBackendError) -> Self {
        match err {
            RedisBackendError::Connection(msg) => StorageError::Connection(msg),
            RedisBackendError::Timeout(msg) => StorageError::Timeout(msg),
            RedisBackendError::Cluster(msg) => StorageError::Backend(msg),
        }
    }
}
```

### PostgreSQL Backend Example (Phase 13b.16)

**Study**: `llmspell-storage/src/postgres/backend.rs` for production pattern

```rust
use llmspell_storage::postgres::PostgreSQLBackend;
use sqlx::PgPool;

#[derive(Debug)]
pub struct PostgreSQLBackend {
    pool: PgPool,
    tenant_id: Option<String>,
    enforce_rls: bool,
}

impl PostgreSQLBackend {
    /// Create from database URL
    pub async fn new(database_url: &str) -> StorageResult<Self> {
        let pool = PgPool::connect(database_url).await
            .map_err(|e| StorageError::Connection(format!("PostgreSQL connection failed: {}", e)))?;

        Ok(Self {
            pool,
            tenant_id: None,
            enforce_rls: false,
        })
    }

    /// Enable Row-Level Security with tenant_id
    pub fn with_tenant(mut self, tenant_id: String) -> Self {
        self.tenant_id = Some(tenant_id);
        self.enforce_rls = true;
        self
    }

    /// Run database migrations (V1-V15)
    pub async fn migrate(&self) -> StorageResult<()> {
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .map_err(|e| StorageError::Backend(format!("Migration failed: {}", e)))?;
        Ok(())
    }

    /// Execute query with automatic RLS tenant injection
    async fn execute_with_tenant<T, F>(&self, f: F) -> StorageResult<T>
    where
        F: FnOnce(&PgPool) -> BoxFuture<StorageResult<T>>,
    {
        if self.enforce_rls {
            let tenant_id = self.tenant_id.as_ref()
                .ok_or_else(|| StorageError::Config("Tenant ID required for RLS".into()))?;

            // Set session tenant_id for RLS
            sqlx::query("SELECT set_config('app.current_tenant_id', $1, true)")
                .bind(tenant_id)
                .execute(&self.pool)
                .await
                .map_err(|e| StorageError::Backend(format!("RLS setup failed: {}", e)))?;
        }

        f(&self.pool).await
    }
}

#[async_trait]
impl StorageBackend for PostgreSQLBackend {
    async fn get(&self, key: &str) -> StorageResult<Option<Vec<u8>>> {
        self.execute_with_tenant(|pool| {
            Box::pin(async move {
                let row: Option<(Vec<u8>,)> = sqlx::query_as(
                    "SELECT value FROM llmspell.key_value_store
                     WHERE key = $1"
                )
                .bind(key)
                .fetch_optional(pool)
                .await
                .map_err(|e| StorageError::Read(format!("PostgreSQL query failed: {}", e)))?;

                Ok(row.map(|(value,)| value))
            })
        }).await
    }

    async fn set(&self, key: &str, value: Vec<u8>) -> StorageResult<()> {
        self.execute_with_tenant(|pool| {
            Box::pin(async move {
                sqlx::query(
                    "INSERT INTO llmspell.key_value_store (key, value, updated_at)
                     VALUES ($1, $2, now())
                     ON CONFLICT (key) DO UPDATE
                     SET value = EXCLUDED.value, updated_at = now()"
                )
                .bind(key)
                .bind(value)
                .execute(pool)
                .await
                .map_err(|e| StorageError::Write(format!("PostgreSQL insert failed: {}", e)))?;

                Ok(())
            })
        }).await
    }

    // ... implement remaining trait methods with execute_with_tenant()
}
```

### Backend Registration in Infrastructure Module

**Location**: `llmspell-bridge/src/infrastructure.rs`

**Pattern**: Register backend in `create_state_manager()` factory function

```rust
async fn create_state_manager(config: &LLMSpellConfig) -> Result<Arc<StateManager>, LLMSpellError> {
    use llmspell_storage::{MemoryBackend, SledBackend, PostgreSQLBackend, StorageBackend};

    // Backend selection from config
    let backend: Arc<dyn StorageBackend> = match config.storage.backend.as_str() {
        "memory" => {
            info!("Using in-memory storage backend");
            Arc::new(MemoryBackend::new())
        }
        "sled" => {
            info!("Using Sled embedded database backend");
            Arc::new(SledBackend::new_with_path(&config.storage.sled.path)?)
        }
        "postgres" => {
            info!("Using PostgreSQL backend");
            let pg_backend = PostgreSQLBackend::new(&config.storage.postgres.url).await?;

            // Run migrations if enabled
            if config.storage.postgres.run_migrations {
                pg_backend.migrate().await?;
            }

            // Enable RLS if configured
            if config.storage.postgres.enforce_tenant_isolation {
                Arc::new(pg_backend.with_tenant(config.storage.postgres.default_tenant.clone()))
            } else {
                Arc::new(pg_backend)
            }
        }
        "redis" => {
            info!("Using Redis backend");
            Arc::new(RedisBackend::new(&config.storage.redis.url).await?)
        }
        backend_type => {
            return Err(LLMSpellError::Config(format!(
                "Unknown storage backend: {}. Supported: memory, sled, postgres, redis",
                backend_type
            )));
        }
    };

    Ok(Arc::new(StateManager::new(backend)))
}
```

### Configuration

**File**: `config.toml`

```toml
[storage]
# Backend selection: "memory", "sled", "postgres", "redis"
backend = "redis"

# Redis configuration
[storage.redis]
url = "redis://localhost:6379"
pool_size = 20
connection_timeout_ms = 5000
operation_timeout_ms = 2000

# Component-specific backend overrides
[storage.components.vector_embeddings]
backend = "postgres"  # Use PostgreSQL HNSW for vectors

[storage.components.session_data]
backend = "redis"     # Use Redis for fast session access

[storage.components.agent_state]
backend = "sled"      # Use Sled for persistent agent state
```

### Backend Testing Pattern

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_testing::storage_helpers::*;

    #[tokio::test]
    async fn test_redis_backend_basic_operations() {
        // Use testcontainers for isolated Redis instance
        let redis_container = testcontainers::clients::Cli::default()
            .run(testcontainers::images::redis::Redis::default());

        let redis_url = format!("redis://localhost:{}", redis_container.get_host_port_ipv4(6379));
        let backend = RedisBackend::new(&redis_url).await.unwrap();

        // Test SET
        backend.set("test_key", b"test_value".to_vec()).await.unwrap();

        // Test GET
        let value = backend.get("test_key").await.unwrap();
        assert_eq!(value, Some(b"test_value".to_vec()));

        // Test EXISTS
        assert!(backend.exists("test_key").await.unwrap());
        assert!(!backend.exists("nonexistent").await.unwrap());

        // Test DELETE
        backend.delete("test_key").await.unwrap();
        assert!(!backend.exists("test_key").await.unwrap());
    }

    #[tokio::test]
    async fn test_redis_backend_batch_operations() {
        let backend = create_test_redis_backend().await;

        let ops = vec![
            BatchOperation::Set { key: "key1".into(), value: b"value1".to_vec() },
            BatchOperation::Set { key: "key2".into(), value: b"value2".to_vec() },
            BatchOperation::Set { key: "key3".into(), value: b"value3".to_vec() },
        ];

        let results = backend.batch(ops).await.unwrap();

        assert_eq!(results.len(), 3);
        assert!(results.iter().all(|r| matches!(r, BatchResult::Success)));

        // Verify all keys exist
        assert!(backend.exists("key1").await.unwrap());
        assert!(backend.exists("key2").await.unwrap());
        assert!(backend.exists("key3").await.unwrap());
    }

    #[tokio::test]
    async fn test_redis_backend_list_keys_with_prefix() {
        let backend = create_test_redis_backend().await;

        // Insert keys with different prefixes
        backend.set("user:1:name", b"Alice".to_vec()).await.unwrap();
        backend.set("user:2:name", b"Bob".to_vec()).await.unwrap();
        backend.set("session:abc", b"active".to_vec()).await.unwrap();

        // List with prefix
        let user_keys = backend.list_keys(Some("user:")).await.unwrap();
        assert_eq!(user_keys.len(), 2);
        assert!(user_keys.contains(&"user:1:name".to_string()));
        assert!(user_keys.contains(&"user:2:name".to_string()));
    }

    #[tokio::test]
    async fn test_redis_backend_stats() {
        let backend = create_test_redis_backend().await;

        // Perform operations
        backend.set("key1", b"value1".to_vec()).await.unwrap();
        backend.get("key1").await.unwrap();
        backend.get("key2").await.unwrap(); // Miss
        backend.delete("key1").await.unwrap();

        // Check stats
        let stats = backend.stats().await.unwrap();
        assert_eq!(stats.writes, 1);
        assert_eq!(stats.reads, 2);
        assert_eq!(stats.deletes, 1);
        assert_eq!(stats.hit_rate, 0.5); // 1 hit out of 2 reads
    }
}
```

### Best Practices for Backend Implementation

1. **Connection Pooling**: Use connection pools for database backends to avoid connection overhead
2. **Error Handling**: Convert backend-specific errors to `StorageError` variants
3. **Tenant Isolation**: Support multi-tenancy with Row-Level Security or key prefixing
4. **Statistics**: Track operations for monitoring and optimization
5. **Batch Operations**: Implement optimized batch methods using backend-specific features (pipelining, transactions)
6. **Graceful Degradation**: Handle connection failures with retries and fallbacks
7. **Testing**: Use testcontainers for isolated backend instances in tests
8. **Performance**: Profile operations and optimize hot paths (caching, indexing)

### Backend Performance Targets

| Operation | Target Latency | Target Throughput | Notes |
|-----------|---------------|-------------------|-------|
| GET | <1ms (p50), <5ms (p99) | 10K+ ops/sec | Use connection pooling |
| SET | <2ms (p50), <10ms (p99) | 5K+ ops/sec | Batch when possible |
| DELETE | <1ms (p50), <5ms (p99) | 5K+ ops/sec | Async when acceptable |
| LIST | <10ms (p50), <50ms (p99) | 1K+ ops/sec | Use indexes, limit results |
| BATCH (100 ops) | <20ms (p50), <100ms (p99) | 500+ batches/sec | Use transactions |

### Related Documentation

- **Storage Architecture**: `/docs/technical/storage-architecture.md` (backend selection, component storage matrix)
- **PostgreSQL Setup**: `/docs/user-guide/storage/postgresql-setup.md` (production PostgreSQL backend setup)
- **Schema Reference**: `/docs/user-guide/storage/schema-reference.md` (PostgreSQL table schemas)
- **API Reference**: `/docs/user-guide/api/rust/llmspell-storage.md` (PostgreSQL backend APIs)

---

## Best Practices

### General Extension Guidelines

1. **Always use llmspell-utils** for parameters, errors, responses
2. **Implement BaseAgent** as foundation for all components
3. **Declare security levels** explicitly
4. **Use llmspell-testing** helpers for tests
5. **Follow existing patterns** from the 37 tools and 60+ examples

### Performance Targets

| Component | Target | Notes |
|-----------|--------|-------|
| Tool init | <10ms | Use lazy initialization |
| Agent creation | <50ms | Cache providers |
| Hook overhead | <2% | Use async where possible |
| Vector search | <8ms @ 100K | Tune HNSW parameters |
| Embedding cache | >80% hit rate | Size cache appropriately |

### Security Considerations

1. **File access**: Always use bridge-provided sandbox
2. **Network access**: Declare in security requirements
3. **Input validation**: Use llmspell-utils validators
4. **Multi-tenant**: Use StateScope for isolation
5. **Secrets**: Never log or store in state

---

## Summary

This guide covered ALL extension patterns in rs-llmspell:

✅ **Tool Development**: BaseAgent + Tool trait with llmspell-utils
✅ **Agent Development**: LLM integration with provider abstraction
✅ **Hook Development**: Cross-cutting concerns with priority
✅ **Workflow Development**: Four patterns + multi-agent coordination
✅ **RAG Extension**: Pipeline builder, custom providers, HNSW tuning
✅ **Storage Backend Extension** (NEW - Phase 13b.16): StorageBackend trait implementation, hot-swappable backends, PostgreSQL example

**Next Steps**:
1. Study the 60+ examples in `examples/`
2. Start with tools, progress to agents, then RAG, then storage backends
3. Use production patterns from existing implementations (Memory, Sled, PostgreSQL)
4. Test thoroughly with proper categorization and testcontainers for backend isolation

**Extension Patterns by Complexity**:
- **Beginner**: Tools (BaseAgent + Tool trait)
- **Intermediate**: Agents (LLM integration), Hooks (cross-cutting)
- **Advanced**: Workflows (multi-agent), RAG (pipeline extension)
- **Expert**: Storage Backends (distributed systems, multi-tenancy)

---

*This guide consolidates patterns from 5 separate guides, adds comprehensive Phase 8 RAG extension documentation, and Phase 13b.16 storage backend extension patterns.*