# Phase 2: Built-in Tools Library - Design Document

**Version**: 2.0  
**Date**: July 2025  
**Status**: Implementation Ready  
**Phase**: 2 (Self-Contained Tools Library)  
**Timeline**: Weeks 5-8 (14 working days)  
**Priority**: CRITICAL (Core Functionality)

> **📋 Detailed Implementation Guide**: This document provides complete specifications for implementing Phase 2 built-in tools library and provider enhancements for rs-llmspell.

---

## Phase Overview

### Goal
Implement comprehensive self-contained tools library with 26+ essential tools across all categories, complete agent-tool integration, and enhance provider system with convenient model specification syntax. Focus on tools without external dependencies.

### Core Principles
- **Tool First Design**: Every tool must have clear schema and validation
- **Provider Enhancement**: Support intuitive `provider/model` syntax
- **Self-Contained First**: No external API dependencies in Phase 2
- **Streaming Ready**: All tools support streaming where applicable
- **Security by Default**: Tools run in sandboxed environments
- **Bridge Pattern**: Tools work consistently across all script engines
- **DRY Principle**: Common utilities in llmspell-utils, tool logic in llmspell-tools

### Success Criteria
- [ ] 26+ functional self-contained tools with complete implementations
- [ ] ModelSpecifier supports `provider/model` syntax parsing
- [ ] Base URL overrides work at agent creation time
- [ ] Tool registry with discovery and validation
- [ ] Security sandboxing for filesystem and system access
- [ ] All tools use llmspell-utils for common operations (DRY)
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

### 1.3 Self-Contained Tools Implementation

**Tool Categories and Implementations:**

#### 1.3.1 Utilities & Helpers Tools (7 tools)

```rust
// llmspell-tools/src/util/text_manipulator.rs
pub struct TextManipulatorTool {
    // Uses llmspell-utils text processing functions
}

// llmspell-tools/src/util/uuid_generator.rs
pub struct UuidGeneratorTool {
    // Uses llmspell-utils UUID generation
}

// llmspell-tools/src/util/hash_calculator.rs
pub struct HashCalculatorTool {
    // Uses llmspell-utils hash functions
}

// llmspell-tools/src/util/base64_encoder.rs
pub struct Base64EncoderTool {
    // Uses llmspell-utils encoding functions
}

// llmspell-tools/src/util/diff_calculator.rs
pub struct DiffCalculatorTool {
    diff_engine: DiffEngine,
}

// llmspell-tools/src/util/date_time_handler.rs
pub struct DateTimeHandlerTool {
    // Uses llmspell-utils time functions
}

// llmspell-tools/src/util/calculator.rs
pub struct CalculatorTool {
    // Changed from evalexpr to fasteval for built-in math functions
    // fasteval provides sin, cos, tan, log, etc. natively
    // We add sqrt, exp, ln as custom functions via callback namespace
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

// Removed XmlTransformerTool - moved to Phase 2.5 (external dependency)
```

#### 1.3.3 API Tools (2 tools) - Self-contained HTTP/GraphQL

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

#### 1.3.4 File System Tools (5 tools)

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

// llmspell-tools/src/fs/file_watcher.rs
pub struct FileWatcherTool {
    // Uses llmspell-utils file monitoring
}

// llmspell-tools/src/fs/file_converter.rs
pub struct FileConverterTool {
    // Uses llmspell-utils encoding detection
}

// llmspell-tools/src/fs/file_search.rs
pub struct FileSearchTool {
    // Self-contained content search
}
```

#### 1.3.5 System Integration Tools (4 tools)

```rust
// llmspell-tools/src/system/environment_reader.rs
pub struct EnvironmentReaderTool {
    // Uses llmspell-utils system queries
}

// llmspell-tools/src/system/process_executor.rs
pub struct ProcessExecutorTool {
    sandbox: ProcessSandbox,
}

// llmspell-tools/src/system/service_checker.rs
pub struct ServiceCheckerTool {
    // Uses llmspell-utils system monitoring
}

// llmspell-tools/src/system/system_monitor.rs
pub struct SystemMonitorTool {
    // Uses llmspell-utils resource monitoring
}
```

#### 1.3.6 Simple Media Tools (3 tools)

```rust
// llmspell-tools/src/media/audio_processor.rs
pub struct AudioProcessorTool {
    // Basic audio operations only
}

// llmspell-tools/src/media/video_processor.rs
pub struct VideoProcessorTool {
    // Basic video operations only
}

// llmspell-tools/src/media/image_processor.rs
pub struct ImageProcessorTool {
    // Basic image operations
}
```

#### 1.3.7 Utility Tools (2 tools)

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

### 1.4 Common Utilities Enhancement (llmspell-utils)

**DRY Principle Implementation:**

```rust
// llmspell-utils/src/text.rs
pub mod text {
    pub fn manipulate(text: &str, operation: TextOp) -> String { /* ... */ }
    pub fn regex_match(text: &str, pattern: &str) -> Vec<Match> { /* ... */ }
    pub fn format_template(template: &str, vars: &HashMap<String, String>) -> String { /* ... */ }
}

// llmspell-utils/src/encoding.rs
pub mod encoding {
    pub fn hash_data(data: &[u8], algorithm: HashAlgorithm) -> Vec<u8> { /* ... */ }
    pub fn base64_encode(data: &[u8]) -> String { /* ... */ }
    pub fn base64_decode(encoded: &str) -> Result<Vec<u8>> { /* ... */ }
    pub fn generate_uuid(version: UuidVersion) -> String { /* ... */ }
}

// llmspell-utils/src/file_monitor.rs
pub mod file_monitor {
    pub fn watch_path(path: &Path, callback: WatchCallback) -> WatchHandle { /* ... */ }
    pub fn detect_encoding(data: &[u8]) -> Encoding { /* ... */ }
    pub fn convert_encoding(data: &[u8], from: Encoding, to: Encoding) -> Vec<u8> { /* ... */ }
}

// llmspell-utils/src/system.rs
pub mod system {
    pub fn read_env_vars() -> HashMap<String, String> { /* ... */ }
    pub fn get_system_info() -> SystemInfo { /* ... */ }
    pub fn monitor_resources() -> ResourceStats { /* ... */ }
    pub fn check_port(port: u16) -> bool { /* ... */ }
}

// llmspell-utils/src/time.rs
pub mod time {
    pub fn parse_datetime(input: &str) -> Result<DateTime<Utc>> { /* ... */ }
    pub fn format_datetime(dt: DateTime<Utc>, format: &str) -> String { /* ... */ }
    pub fn convert_timezone(dt: DateTime<Utc>, tz: &str) -> DateTime<FixedOffset> { /* ... */ }
}
```

### 1.5 Tool Registry System

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

### 1.6.1 Async Bridge Architecture

**Problem**: Current synchronous Lua execution cannot properly handle async Rust tools (HTTP, GraphQL, etc.), resulting in "attempt to yield from outside a coroutine" errors.

**Solution**: Implement coroutine-based async execution at the tool level while keeping script execution synchronous. This avoids mlua AsyncThread Send trait issues while providing proper async support.

**Implementation Approach:**

After analyzing several options, we chose a **coroutine-based solution** that:
- Keeps the Lua engine synchronous (avoiding Send trait issues)
- Wraps async tool execution in Lua coroutines
- Provides helper functions for seamless async tool usage
- Maintains backward compatibility

**Architecture (No Changes Required):**

```rust
// llmspell-bridge/src/lua/engine.rs
pub struct LuaEngine {
    lua: Arc<parking_lot::Mutex<mlua::Lua>>, // Standard Lua with async features
    _config: LuaConfig,
    api_injected: bool,
    execution_context: ExecutionContext,
    // No dedicated runtime needed
}
```

**Tool Execution Bridge (Already Async-Enabled):**

```rust
// llmspell-bridge/src/lua/api/tool.rs
// Tool execution already uses create_async_function
tool_table.set(
    "execute",
    lua.create_async_function(move |lua, args: mlua::Table| {
        let tool_instance = tool_arc_for_execute.clone();
        async move {
            // Convert parameters and execute
            let result = tool_instance.execute(input, context).await;
            // Return result table
        }
    })?
)?;
```

**Lua Coroutine Helpers:**

```lua
-- Helper to execute tool functions within a coroutine
function Tool.executeAsync(tool_name, params)
    local tool = Tool.get(tool_name)
    if not tool then
        return {success = false, error = "Tool not found: " .. tool_name}
    end
    
    -- Create coroutine for async execution
    local co = coroutine.create(function()
        return tool.execute(params or {})
    end)
    
    -- Execute the coroutine
    local success, result = coroutine.resume(co)
    
    -- Handle async operations that yield
    while success and coroutine.status(co) ~= "dead" do
        success, result = coroutine.resume(co, result)
    end
    
    if not success then
        return {success = false, error = tostring(result)}
    end
    
    return result
end

-- Backward compatibility wrapper
function Tool.executeSync(tool_name, params)
    return Tool.executeAsync(tool_name, params)
end
```

**Usage Pattern:**

```lua
-- Old way (still works but may error with async tools)
local tool = Tool.get("http_request")
local result = tool.execute({...})  -- May fail with coroutine error

-- New way (works with all tools)
local result = Tool.executeAsync("http_request", {
    method = "GET",
    url = "https://api.example.com/data"
})

-- Test helpers updated to use new pattern
function TestHelpers.execute_tool(tool_name, params)
    return Tool.executeAsync(tool_name, params)
end
```

**Performance Impact:**

Benchmarking shows minimal overhead:
- Average overhead: 2.3% (well within <5% target)
- Synchronous tools: ~0.8ms additional per call
- Mixed tool execution: ~2.5% overhead
- Memory impact: Negligible

**Benefits:**
- ✅ Fixes "attempt to yield from outside a coroutine" errors
- ✅ No breaking changes to existing code
- ✅ Simpler implementation (no AsyncThread complexity)
- ✅ Better error handling with proper error propagation
- ✅ Maintains script execution simplicity
- ✅ Works with all existing examples

**Migration Guide:**

1. **For tool users**: Update direct tool.execute() calls to Tool.executeAsync()
2. **For test writers**: Use test-helpers.lua which is already updated
3. **For existing scripts**: No changes needed if using agent-based approach
4. **For new scripts**: Prefer Tool.executeAsync() for all tool execution

### 1.7 JSON API for Script Bridge

**Problem**: Tool outputs are returned as JSON strings, but Lua lacks built-in JSON parsing capabilities. This forces scripts to work with raw JSON strings rather than structured data, limiting the usefulness of tool outputs.

**Solution**: Implement a language-agnostic JSON API at the bridge level that provides consistent JSON parsing/stringifying across all scripting languages.

**Architecture:**

```rust
// llmspell-bridge/src/engine/types.rs
// Add to ApiSurface struct
pub struct ApiSurface {
    pub agent_api: AgentApiDefinition,
    pub tool_api: ToolApiDefinition,
    pub workflow_api: WorkflowApiDefinition,
    pub streaming_api: StreamingApiDefinition,
    pub json_api: JsonApiDefinition,  // NEW
}

// JSON API definition
#[derive(Debug, Clone)]
pub struct JsonApiDefinition {
    /// Global object name (e.g., "JSON" in Lua/JS)
    pub global_name: String,
    /// Function to parse JSON string to native value
    pub parse_function: String,
    /// Function to stringify native value to JSON
    pub stringify_function: String,
}

impl JsonApiDefinition {
    pub fn standard() -> Self {
        Self {
            global_name: "JSON".to_string(),
            parse_function: "parse".to_string(),
            stringify_function: "stringify".to_string(),
        }
    }
}
```

**Lua Implementation:**

```rust
// llmspell-bridge/src/lua/api/json.rs
pub fn inject_json_api(
    lua: &mlua::Lua,
    api_def: &JsonApiDefinition,
) -> Result<(), LLMSpellError> {
    let json_table = lua.create_table()?;
    
    // JSON.parse(string) -> table/value
    let parse_fn = lua.create_function(|lua, json_str: String| {
        let json_value = serde_json::from_str::<serde_json::Value>(&json_str)
            .map_err(|e| mlua::Error::RuntimeError(format!("JSON parse error: {}", e)))?;
        json_value_to_lua(lua, json_value)
    })?;
    
    // JSON.stringify(value) -> string
    let stringify_fn = lua.create_function(|lua, value: mlua::Value| {
        let json_value = lua_value_to_json(value)?;
        serde_json::to_string(&json_value)
            .map_err(|e| mlua::Error::RuntimeError(format!("JSON stringify error: {}", e)))
    })?;
    
    json_table.set(api_def.parse_function.as_str(), parse_fn)?;
    json_table.set(api_def.stringify_function.as_str(), stringify_fn)?;
    
    lua.globals().set(api_def.global_name.as_str(), json_table)?;
    Ok(())
}

// Reuse existing conversion functions from tool.rs
fn json_value_to_lua(lua: &mlua::Lua, value: serde_json::Value) -> mlua::Result<mlua::Value> {
    match value {
        serde_json::Value::Null => Ok(mlua::Value::Nil),
        serde_json::Value::Bool(b) => Ok(mlua::Value::Boolean(b)),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(mlua::Value::Integer(i))
            } else if let Some(f) = n.as_f64() {
                Ok(mlua::Value::Number(f))
            } else {
                Ok(mlua::Value::Nil)
            }
        }
        serde_json::Value::String(s) => Ok(mlua::Value::String(lua.create_string(&s)?)),
        serde_json::Value::Array(arr) => {
            let table = lua.create_table()?;
            for (i, val) in arr.into_iter().enumerate() {
                table.set(i + 1, json_value_to_lua(lua, val)?)?;
            }
            Ok(mlua::Value::Table(table))
        }
        serde_json::Value::Object(map) => {
            let table = lua.create_table()?;
            for (k, v) in map {
                table.set(k, json_value_to_lua(lua, v)?)?;
            }
            Ok(mlua::Value::Table(table))
        }
    }
}
```

**Usage Pattern:**

```lua
-- Parse tool output
local tool_result = Tool.executeAsync("uuid_generator", {operation = "generate"})
if tool_result.success and tool_result.output then
    -- Parse JSON string to Lua table
    local parsed = JSON.parse(tool_result.output)
    print("UUID:", parsed.result.uuid)
    print("Version:", parsed.result.version)
end

-- Stringify data for tool input
local data = {
    items = {
        {name = "item1", value = 42},
        {name = "item2", value = 84}
    }
}
local json_str = JSON.stringify(data)

-- Use with json_processor tool
local result = Tool.executeAsync("json_processor", {
    data = json_str,
    query = ".items[] | select(.value > 50)"
})
```

**JavaScript Implementation (Future):**

```javascript
// llmspell-bridge/src/javascript/api/json.rs
// JavaScript already has native JSON object, so we just ensure consistency
pub fn inject_json_api(
    ctx: &JavaScriptContext,
    api_def: &JsonApiDefinition,
) -> Result<(), LLMSpellError> {
    // JavaScript's native JSON object already provides parse/stringify
    // This ensures the API exists and matches our definition
    ctx.ensure_global_object(&api_def.global_name)?;
    Ok(())
}
```

**Benefits:**

1. **Language Agnostic**: Same API surface across Lua, JavaScript, Python
2. **Performance**: Native Rust serde_json performance vs pure-script implementations
3. **Type Safety**: Proper error handling and type conversions
4. **Consistency**: Follows established bridge API patterns
5. **Reusability**: Leverages existing conversion code in the bridge

**Implementation Impact:**

- All tool examples can properly parse structured output
- Scripts can work with native data structures instead of strings
- Enables complex data transformations between tools
- No external dependencies or embedded parsers needed

---

## 2. Technical Design Details

### 2.1 Tool Schema Validation

Each tool must define a complete JSON Schema for parameter validation:

```rust
impl Tool for TextManipulatorTool {
    fn schema(&self) -> ToolSchema {
        ToolSchema {
            name: "text_manipulator".to_string(),
            description: "Manipulate and transform text".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "text": {
                        "type": "string",
                        "description": "Input text to manipulate"
                    },
                    "operation": {
                        "type": "string",
                        "enum": ["uppercase", "lowercase", "reverse", "trim", "replace"],
                        "description": "Operation to perform"
                    },
                    "options": {
                        "type": "object",
                        "description": "Additional options for the operation"
                    }
                },
                "required": ["text", "operation"]
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

### 2.5 Calculator Tool Enhancement

**Migration from evalexpr to fasteval:**

The CalculatorTool has been enhanced to support mathematical functions by switching from `evalexpr` to `fasteval`:

**Built-in Functions (Native to fasteval)**:
- **Trigonometric**: sin, cos, tan, asin, acos, atan, sinh, cosh, tanh, asinh, acosh, atanh
- **Numeric**: abs, sign, ceil, floor, round, int, min, max
- **Logarithm**: log (with optional base, defaults to 10)
- **Constants**: pi(), e() functions return mathematical constants

**Custom Functions (Added via callback namespace)**:
- **sqrt(x)**: Square root, implemented as x^0.5
- **exp(x)**: Exponential function, implemented as e()^x  
- **ln(x)**: Natural logarithm, implemented as log(e(), x)

**Implementation Note**: Fasteval doesn't include sqrt, exp, or ln natively, so we implement them as custom functions using fasteval's callback namespace feature. This provides a seamless user experience while leveraging fasteval's performance and safety.

- **Performance**: Fasteval benchmarks show it's the fastest expression evaluation library
- **Safety**: Designed for safe evaluation of untrusted expressions
- **Variables**: Full support for variable substitution

**Example Usage:**
```lua
-- Mathematical expressions work seamlessly
local result = calculator:execute({
    expression = "sin(pi()/2) + sqrt(16) + log(10, 100)"  -- Returns 6.0
})

-- Complex expressions with variables
local result = calculator:execute({
    expression = "sqrt(x^2 + y^2) * exp(-t/2)",
    variables = {x = 3, y = 4, t = 1}
})

-- Natural logarithm
local result = calculator:execute({
    expression = "ln(e()) + ln(10)"  -- ln(e) = 1, ln(10) ≈ 2.303
})
```

---

## 3. Implementation Phases

### Phase 2.1: Provider Enhancement (Days 1-2)
- Implement ModelSpecifier
- Update ProviderManager
- Add base_url override support
- Update script APIs

### Phase 2.2: Core Tool Infrastructure (Day 3)
- Enhanced Tool trait
- Tool registry implementation
- Security sandbox setup
- Resource monitoring

### Phase 2.3: Utilities & Helpers Tools (Days 4-5)
- TextManipulatorTool, UuidGeneratorTool, HashCalculatorTool
- Base64EncoderTool, DiffCalculatorTool
- DateTimeHandlerTool, CalculatorTool

### Phase 2.4: Data Processing & File System Tools (Days 6-7)
- JsonProcessorTool, CsvAnalyzerTool
- FileOperationsTool, ArchiveHandlerTool
- FileWatcherTool, FileConverterTool, FileSearchTool

### Phase 2.5: System Integration Tools (Day 8)
- EnvironmentReaderTool, ProcessExecutorTool
- ServiceCheckerTool, SystemMonitorTool

### Phase 2.6: API & Simple Media Tools (Day 9)
- HttpRequestTool, GraphQLQueryTool
- AudioProcessorTool, VideoProcessorTool, ImageProcessorTool

### Phase 2.7: Common Utilities Enhancement (Day 10)
- Enhance llmspell-utils with common functions
- Refactor existing tools to use shared utilities
- Remove duplicate code across implementations

### Phase 2.8: Utility Tools & Integration (Days 11-12)
- TemplateEngineTool, DataValidationTool
- Script integration tests
- Performance optimization

### Phase 2.9: Testing & Documentation (Days 13-14)
- Comprehensive tool testing
- Security validation
- Documentation and examples
- Phase 3 handoff preparation

---

## 4. Success Metrics

### Functional Requirements
- ✅ All 26+ self-contained tools implemented and tested
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
1. **System Tool Security**: Enhanced sandbox testing for system integration
2. **Media Processing Performance**: Resource limits and optimization
3. **Security Vulnerabilities**: Comprehensive sandbox testing
4. **Cross-platform Issues**: Test on Linux/macOS/Windows

### Schedule Risks
1. **Tool Count Increase**: 26+ tools vs original 12 tools
2. **Utility Refactoring Time**: DRY principle implementation
3. **Security Testing Time**: Parallelize with development
4. **Documentation Overhead**: Write as we code

---

## 6. Dependencies

### External Crates
- `reqwest`: HTTP client for API tools
- `tokio`: Async runtime
- `serde_json`: JSON processing
- `jsonschema`: Schema validation
- `regex`: Pattern matching
- `csv`: CSV processing
- `notify`: File system watching
- `encoding_rs`: Encoding detection
- `sysinfo`: System information
- `sha2`, `md5`: Hash algorithms
- `base64`: Base64 encoding
- `uuid`: UUID generation
- `chrono`: Date/time handling
- `zip`, `tar`: Archive handling
- `fasteval`: Mathematical expression evaluation (replaced evalexpr)

### Internal Dependencies
- `llmspell-core`: Trait definitions
- `llmspell-utils`: Enhanced shared utilities (DRY principle)
- `llmspell-security`: Sandboxing (enhanced for system tools)
- `llmspell-providers`: Agent creation

---

## 7. Deliverables

### Code Deliverables
1. Enhanced provider system with ModelSpecifier
2. 26+ fully functional self-contained tools
3. Enhanced llmspell-utils with common utilities
4. Tool registry with discovery
5. Security sandbox implementation (enhanced for system tools)
6. Comprehensive test suite (26+ tools covered)

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

### Cross-Platform Assumptions

The Phase 2 tools implementation makes the following platform-specific assumptions:

1. **File System Operations**
   - Path separators handled by Rust's std::path
   - UTF-8 encoding assumed for text files
   - Symbolic links follow platform conventions
   - File permissions respect platform security models

2. **System Monitor Tool**
   - CPU usage calculation differs between platforms:
     - Linux: Uses /proc/stat for accurate CPU metrics
     - macOS/Windows: Falls back to load average approximation
   - Memory statistics use platform-specific APIs (libc::sysinfo)
   - Disk space uses statvfs on Unix, GetDiskFreeSpaceEx on Windows

3. **Process Executor Tool**
   - Command execution uses platform shell:
     - Unix: /bin/sh
     - Windows: cmd.exe
   - Signal handling is Unix-specific
   - Process limits may not be enforced on Windows

4. **Environment Reader Tool**
   - Environment variable names are case-sensitive on Unix
   - PATH separator is : on Unix, ; on Windows
   - System info relies on sysinfo crate for cross-platform data

5. **File Watcher Tool**
   - Uses notify crate which has platform-specific backends:
     - Linux: inotify
     - macOS: FSEvents
     - Windows: ReadDirectoryChangesW
   - Event granularity varies by platform

6. **Network Tools (HTTP, Service Checker)**
   - Assumes standard TCP/IP stack
   - DNS resolution uses platform resolver
   - Certificate validation follows platform trust stores

7. **Testing Assumptions**
   - Examples assume Unix-style shell for test runner
   - bc calculator optional (falls back to awk)
   - Timeout command availability varies
   - ANSI color codes may not work on all terminals