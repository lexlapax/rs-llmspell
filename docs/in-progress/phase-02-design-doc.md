# Phase 2: Built-in Tools Library - Design Document

**Version**: 1.0  
**Date**: June 2025  
**Status**: Implementation Ready  
**Phase**: 2 (Built-in Tools Library)  
**Timeline**: Weeks 5-6 (10 working days)  
**Priority**: CRITICAL (Core Functionality)

> **📋 Detailed Implementation Guide**: This document provides complete specifications for implementing Phase 2 built-in tools library and provider enhancements for rs-llmspell.

---

## Phase Overview

### Goal
Implement comprehensive built-in tools library with 12+ essential tools, complete agent-tool integration, and enhance provider system with convenient model specification syntax.

### Core Principles
- **Tool First Design**: Every tool must have clear schema and validation
- **Provider Enhancement**: Support intuitive `provider/model` syntax
- **Streaming Ready**: All tools support streaming where applicable
- **Security by Default**: Tools run in sandboxed environments
- **Bridge Pattern**: Tools work consistently across all script engines

### Success Criteria
- [ ] 12+ functional built-in tools with complete implementations
- [ ] ModelSpecifier supports `provider/model` syntax parsing
- [ ] Base URL overrides work at agent creation time
- [ ] Tool registry with discovery and validation
- [ ] Security sandboxing for filesystem and network access
- [ ] All tools have comprehensive tests and documentation
- [ ] Agent-tool integration works seamlessly in scripts
- [ ] Performance benchmarks show <10ms tool initialization

---

## 1. Implementation Specifications

### 1.1 Provider Enhancement (Rolled from Phase 1)

**ModelSpecifier Implementation:**

```rust
// llmspell-providers/src/model_specifier.rs
#[derive(Debug, Clone, PartialEq)]
pub struct ModelSpecifier {
    pub provider: Option<String>,
    pub model: String,
    pub base_url: Option<String>,
}

impl ModelSpecifier {
    /// Parse "provider/model" or "model" format
    pub fn parse(spec: &str) -> Self {
        if let Some(slash_pos) = spec.rfind('/') {
            let provider = spec[..slash_pos].to_string();
            let model = spec[slash_pos + 1..].to_string();
            Self {
                provider: Some(provider),
                model,
                base_url: None,
            }
        } else {
            Self {
                provider: None,
                model: spec.to_string(),
                base_url: None,
            }
        }
    }
    
    /// Parse with base_url override
    pub fn parse_with_base_url(spec: &str, base_url: Option<String>) -> Self {
        let mut parsed = Self::parse(spec);
        parsed.base_url = base_url;
        parsed
    }
}
```

**ProviderManager Updates:**

```rust
// llmspell-providers/src/manager.rs
impl ProviderManager {
    /// Create agent with convenient syntax
    pub async fn create_agent_from_spec(
        &self,
        name: &str,
        spec: &str,
        base_url: Option<String>,
        config: AgentConfig,
    ) -> Result<Box<dyn BaseAgent>> {
        let model_spec = ModelSpecifier::parse_with_base_url(spec, base_url);
        
        // Determine provider
        let provider_name = model_spec.provider
            .or_else(|| self.default_provider.clone())
            .ok_or_else(|| Error::NoProviderSpecified)?;
            
        // Get or create provider with base_url override
        let provider = self.get_or_create_provider(
            &provider_name,
            model_spec.base_url.as_deref()
        )?;
        
        // Create agent with model
        provider.create_agent(name, &model_spec.model, config).await
    }
}
```

### 1.2 Core Tool Architecture

**Tool Trait Enhancements:**

```rust
// llmspell-core/src/traits/tool.rs
#[async_trait]
pub trait Tool: Send + Sync {
    // Existing methods...
    
    /// Stream execution support
    async fn stream_execute(
        &self,
        params: Value,
        context: ExecutionContext,
    ) -> Result<ToolStream> {
        // Default: convert regular execution to single-item stream
        let result = self.execute(params, context).await?;
        Ok(Box::pin(futures::stream::once(async { Ok(result) })))
    }
    
    /// Security requirements for this tool
    fn security_requirements(&self) -> SecurityRequirements {
        SecurityRequirements::default()
    }
    
    /// Resource limits for execution
    fn resource_limits(&self) -> ResourceLimits {
        ResourceLimits::default()
    }
}
```

### 1.3 Built-in Tools Implementation

**Tool Categories and Implementations:**

#### 1.3.1 Search Tools (3 tools)

```rust
// llmspell-tools/src/search/web_search.rs
pub struct WebSearchTool {
    client: reqwest::Client,
    api_key: Option<String>,
    provider: SearchProvider,
}

// llmspell-tools/src/search/semantic_search.rs
pub struct SemanticSearchTool {
    embedding_model: Box<dyn EmbeddingModel>,
    vector_store: Box<dyn VectorStore>,
}

// llmspell-tools/src/search/code_search.rs
pub struct CodeSearchTool {
    index_path: PathBuf,
    language_parsers: HashMap<String, Box<dyn LanguageParser>>,
}
```

#### 1.3.2 Data Processing Tools (3 tools)

```rust
// llmspell-tools/src/data/json_processor.rs
pub struct JsonProcessorTool {
    jq_engine: JqEngine,
    schema_validator: Option<JsonSchema>,
}

// llmspell-tools/src/data/csv_analyzer.rs
pub struct CsvAnalyzerTool {
    max_file_size: usize,
    encoding_detector: EncodingDetector,
}

// llmspell-tools/src/data/xml_transformer.rs
pub struct XmlTransformerTool {
    xslt_processor: XsltProcessor,
    xpath_engine: XPathEngine,
}
```

#### 1.3.3 External API Tools (2 tools)

```rust
// llmspell-tools/src/api/http_request.rs
pub struct HttpRequestTool {
    client: reqwest::Client,
    rate_limiter: RateLimiter,
    retry_policy: RetryPolicy,
}

// llmspell-tools/src/api/graphql_query.rs
pub struct GraphQLQueryTool {
    client: GraphQLClient,
    schema_cache: HashMap<String, GraphQLSchema>,
}
```

#### 1.3.4 File System Tools (2 tools)

```rust
// llmspell-tools/src/fs/file_operations.rs
pub struct FileOperationsTool {
    sandbox: FileSandbox,
    allowed_paths: Vec<PathBuf>,
}

// llmspell-tools/src/fs/archive_handler.rs
pub struct ArchiveHandlerTool {
    supported_formats: Vec<ArchiveFormat>,
    extraction_limits: ExtractionLimits,
}
```

#### 1.3.5 Utility Tools (2 tools)

```rust
// llmspell-tools/src/util/template_engine.rs
pub struct TemplateEngineTool {
    engine: TemplateEngine,
    custom_filters: HashMap<String, Box<dyn TemplateFilter>>,
}

// llmspell-tools/src/util/data_validation.rs
pub struct DataValidationTool {
    validators: HashMap<String, Box<dyn Validator>>,
    custom_rules: Vec<ValidationRule>,
}
```

### 1.4 Tool Registry System

```rust
// llmspell-tools/src/registry.rs
pub struct ToolRegistry {
    tools: HashMap<String, Arc<dyn Tool>>,
    categories: HashMap<String, Vec<String>>,
    metadata: HashMap<String, ToolMetadata>,
}

impl ToolRegistry {
    /// Register a new tool
    pub fn register<T: Tool + 'static>(&mut self, tool: T) -> Result<()> {
        let metadata = tool.metadata();
        let id = metadata.id.clone();
        
        // Validate tool schema
        self.validate_tool(&tool)?;
        
        // Check security requirements
        self.check_security_requirements(&tool)?;
        
        // Register tool
        self.tools.insert(id.to_string(), Arc::new(tool));
        self.metadata.insert(id.to_string(), metadata);
        
        Ok(())
    }
    
    /// Discover tools by capability
    pub fn discover_by_capability(&self, capability: &str) -> Vec<&ComponentId> {
        self.metadata
            .iter()
            .filter(|(_, meta)| meta.capabilities.contains(capability))
            .map(|(id, _)| &self.tools[id].metadata().id)
            .collect()
    }
}
```

### 1.5 Security Sandboxing

```rust
// llmspell-security/src/sandbox.rs
pub struct ToolSandbox {
    fs_sandbox: FileSandbox,
    network_sandbox: NetworkSandbox,
    resource_monitor: ResourceMonitor,
}

impl ToolSandbox {
    /// Execute tool in sandbox
    pub async fn execute_tool<T: Tool>(
        &self,
        tool: &T,
        params: Value,
        context: ExecutionContext,
    ) -> Result<ToolOutput> {
        // Check security requirements
        let requirements = tool.security_requirements();
        self.validate_requirements(&requirements)?;
        
        // Set up resource limits
        let limits = tool.resource_limits();
        let _guard = self.resource_monitor.acquire_resources(&limits)?;
        
        // Execute with monitoring
        let result = tokio::time::timeout(
            limits.max_execution_time,
            tool.execute(params, context)
        ).await??;
        
        Ok(result)
    }
}
```

### 1.6 Script Integration

**Enhanced Lua API:**

```lua
-- Create agent with convenience syntax
local agent = Agent.create({
    name = "researcher",
    model = "openai/gpt-4",  -- Provider inferred
    base_url = "https://custom.api.com"  -- Optional override
})

-- Alternative syntaxes
local agent2 = Agent.create({
    name = "assistant", 
    model = "gpt-4"  -- Uses default provider
})

local agent3 = Agent.create({
    name = "local",
    provider = "ollama",  -- Explicit provider
    model = "llama3"
})

-- Tool usage
local search = Tool.get("web_search")
local results = search:execute({
    query = "rust async programming",
    max_results = 10
})

-- Streaming tool execution
local processor = Tool.get("json_processor")
local stream = processor:stream_execute({
    data = large_dataset,
    query = ".items[] | select(.active)"
})

for chunk in stream do
    print("Processed:", chunk)
end
```

---

## 2. Technical Design Details

### 2.1 Tool Schema Validation

Each tool must define a complete JSON Schema for parameter validation:

```rust
impl Tool for WebSearchTool {
    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: "web_search".to_string(),
            description: "Search the web for information".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query"
                    },
                    "max_results": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 100,
                        "default": 10
                    },
                    "search_type": {
                        "type": "string",
                        "enum": ["web", "news", "images", "videos"],
                        "default": "web"
                    }
                },
                "required": ["query"]
            }),
        }
    }
}
```

### 2.2 Error Handling

Comprehensive error types for tools:

```rust
#[derive(Debug, thiserror::Error)]
pub enum ToolError {
    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),
    
    #[error("Security violation: {0}")]
    SecurityViolation(String),
    
    #[error("Resource limit exceeded: {0}")]
    ResourceLimitExceeded(String),
    
    #[error("External service error: {0}")]
    ExternalServiceError(String),
    
    #[error("Tool not found: {0}")]
    ToolNotFound(String),
}
```

### 2.3 Performance Optimization

- Tool initialization: Lazy loading with <10ms startup
- Parameter validation: Cached JSON Schema compilation
- Resource pooling: Connection pools for HTTP/database tools
- Streaming: Efficient chunk processing without buffering

### 2.4 Testing Strategy

Each tool requires:
- Unit tests for core functionality
- Integration tests with mocked services
- Property-based tests for parameter validation
- Performance benchmarks
- Security sandbox tests

---

## 3. Implementation Phases

### Phase 2.1: Provider Enhancement (Days 1-2)
- Implement ModelSpecifier
- Update ProviderManager
- Add base_url override support
- Update script APIs

### Phase 2.2: Core Tool Infrastructure (Days 3-4)
- Enhanced Tool trait
- Tool registry implementation
- Security sandbox setup
- Resource monitoring

### Phase 2.3: Search Tools (Days 4-5)
- WebSearchTool
- SemanticSearchTool
- CodeSearchTool

### Phase 2.4: Data & API Tools (Days 6-7)
- JsonProcessorTool
- CsvAnalyzerTool
- HttpRequestTool
- GraphQLQueryTool

### Phase 2.5: File & Utility Tools (Days 8)
- FileOperationsTool
- ArchiveHandlerTool
- TemplateEngineTool
- DataValidationTool

### Phase 2.6: Integration & Testing (Days 9-10)
- Script integration tests
- Performance optimization
- Documentation
- Security validation

---

## 4. Success Metrics

### Functional Requirements
- ✅ All 12 tools implemented and tested
- ✅ ModelSpecifier parses all syntax variants
- ✅ Tool registry discovers by capability
- ✅ Security sandbox prevents violations
- ✅ Streaming works for applicable tools

### Performance Requirements
- ✅ Tool initialization <10ms
- ✅ Parameter validation <1ms
- ✅ Registry lookup <100μs
- ✅ Memory overhead <5MB per tool

### Quality Requirements
- ✅ >90% test coverage
- ✅ Zero security vulnerabilities
- ✅ All tools documented
- ✅ Examples for each tool

---

## 5. Risk Mitigation

### Technical Risks
1. **External API Dependencies**: Mock services for testing
2. **Security Vulnerabilities**: Comprehensive sandbox testing
3. **Performance Degradation**: Continuous benchmarking
4. **Cross-platform Issues**: Test on Linux/macOS/Windows

### Schedule Risks
1. **Complex Tool Implementation**: Start with simpler tools
2. **Security Testing Time**: Parallelize with development
3. **Documentation Overhead**: Write as we code

---

## 6. Dependencies

### External Crates
- `reqwest`: HTTP client for API tools
- `tokio`: Async runtime
- `serde_json`: JSON processing
- `jsonschema`: Schema validation
- `regex`: Pattern matching
- `csv`: CSV processing
- `quick-xml`: XML handling

### Internal Dependencies
- `llmspell-core`: Trait definitions
- `llmspell-utils`: Shared utilities
- `llmspell-security`: Sandboxing
- `llmspell-providers`: Agent creation

---

## 7. Deliverables

### Code Deliverables
1. Enhanced provider system with ModelSpecifier
2. 12+ fully functional built-in tools
3. Tool registry with discovery
4. Security sandbox implementation
5. Comprehensive test suite

### Documentation Deliverables
1. Tool usage guide
2. Security best practices
3. Tool development guide
4. API reference
5. Migration guide for model syntax

### Phase Handoff
1. Working tools demonstration
2. Performance benchmarks
3. Security audit results
4. Phase 3 preparation notes