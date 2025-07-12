# Phase 3: Tool Enhancement & Workflow Orchestration - Design Document

**Version**: 1.0  
**Date**: July 2025  
**Status**: Implementation Ready  
**Phase**: 3 (Tool Enhancement & Workflow Orchestration)  
**Timeline**: Weeks 9-16 (8 weeks)  
**Priority**: HIGH (MVP Completion)

> **📋 Comprehensive Implementation Guide**: This document provides complete specifications for implementing Phase 3, which spans 8 weeks and encompasses critical tool fixes, external integrations, security hardening, and workflow orchestration.

---

## Phase Overview

### Goal
Transform the existing 26 self-contained tools into a standardized, secure, and extensible library of 33+ tools, then implement comprehensive workflow orchestration patterns that leverage the full tool ecosystem.

### Core Principles
- **Standardization First**: Fix existing tools before adding new ones
- **Clean Break Strategy**: Direct upgrade with clear documentation (pre-1.0 freedom)
- **Security by Design**: Address all known vulnerabilities
- **Progressive Enhancement**: Each sub-phase builds on the previous
- **Documentation Driven**: Update docs before implementation

### Phase Structure
Phase 3 is divided into four 2-week sub-phases:

1. **Phase 3.0 (Weeks 9-10)**: Critical Tool Fixes - Standardization, DRY, and Initial Security
2. **Phase 3.1 (Weeks 11-12)**: External Integration Tools - 8 new tools
3. **Phase 3.2 (Weeks 13-14)**: Advanced Security & Performance - Optimization for all 33 tools
4. **Phase 3.3 (Weeks 15-16)**: Workflow Orchestration - Patterns and engine

### Success Metrics
- **Tool Consistency**: 95% parameter standardization (from 60%)
- **DRY Compliance**: 95% shared utility usage (from 80%)
- **Security Coverage**: Comprehensive vulnerability mitigation
- **Tool Count**: 33+ production-ready tools
- **Workflow Support**: All patterns functional with full tool library

---

## Pre-1.0 Clean Break Benefits

### Why No Migration Tools?

1. **Development Velocity**: Save ~1 week of development time that can be invested in better features
2. **Code Cleanliness**: No legacy compatibility code cluttering the codebase
3. **Better Architecture**: Freedom to make optimal design decisions without compatibility constraints
4. **Industry Standard**: Pre-1.0 breaking changes are expected and accepted
5. **No Production Users**: Fresh from Phase 2 with minimal adoption risk

### What We Provide Instead

1. **Comprehensive Documentation**: Every breaking change documented with examples
2. **Clear Upgrade Path**: Step-by-step instructions for updating scripts
3. **Example Conversions**: Before/after code samples for common patterns
4. **Tool Reference**: Complete parameter documentation for all 33 tools

---

## Phase 3.0: Critical Tool Fixes (Weeks 9-10)

### Goal
Standardize all 26 existing tools to use consistent interfaces, parameter names, and response formats, while implementing critical security fixes.

### 1. Tool Signature Standardization

#### 1.1 Parameter Naming Convention

**Primary Data Parameter**:
```rust
// BEFORE: Inconsistent naming
- text: String         // TextManipulator
- content: String      // FileWriter  
- input: Value        // JsonProcessor
- data: String        // DataValidator
- query: String       // Calculator
- expression: String  // Calculator alternate
- template: String    // TemplateEngine

// AFTER: Standardized
- input: String | Value  // Universal primary data parameter
```

**File Path Parameters**:
```rust
// BEFORE: Inconsistent
- file_path: String
- input_path: String / output_path: String  
- path: String
- paths: Vec<String>
- archive_path: String
- file: String
- url: String

// AFTER: Standardized
- path: PathBuf              // Single file operations
- source_path: PathBuf       // Transform operations (with target_path)
- target_path: PathBuf       // Transform operations (with source_path)
- paths: Vec<PathBuf>        // Multi-file operations
```

**Operation Parameters**:
```rust
// All multi-function tools MUST use:
- operation: String  // Required for tools with multiple operations
```

#### 1.2 Detailed Parameter Mapping (26 Tools)

##### File Operations Tools (5 tools)
| Tool | Old Parameters | New Parameters | Notes |
|------|---------------|----------------|-------|
| **FileOperationsTool** | `operation: String`<br>`path: String`<br>`content: Option<String>` | `operation: String`<br>`path: PathBuf`<br>`input: Option<String>` | `content` → `input`<br>String → PathBuf |
| **FileSearchTool** | `pattern: String`<br>`path: String`<br>`search_type: String` | `operation: String`<br>`input: String`<br>`path: PathBuf`<br>`search_type: String` | Add `operation`<br>`pattern` → `input` |
| **FileConverterTool** | `input_path: String`<br>`output_path: String`<br>`format: String` | `operation: String`<br>`source_path: PathBuf`<br>`target_path: PathBuf`<br>`format: String` | Add `operation`<br>Rename paths |
| **FileWatcherTool** | `path: String`<br>`pattern: String`<br>`events: Vec<String>` | `operation: String`<br>`path: PathBuf`<br>`input: String`<br>`events: Vec<String>` | Add `operation`<br>`pattern` → `input` |
| **ArchiveHandlerTool** | `operation: String`<br>`archive_path: String`<br>`files: Vec<String>` | `operation: String`<br>`path: PathBuf`<br>`files: Vec<PathBuf>` | `archive_path` → `path` |

##### Data Processing Tools (2 tools)
| Tool | Old Parameters | New Parameters | Notes |
|------|---------------|----------------|-------|
| **CsvAnalyzerTool** | `operation: String`<br>`file_path: Option<String>`<br>`content: Option<String>` | `operation: String`<br>`input: String`<br>`path: Option<PathBuf>` | Consolidate to `input`<br>`file_path` → `path` |
| **JsonProcessorTool** | `operation: String`<br>`input: Option<Value>`<br>`query: Option<String>`<br>`content: Option<String>` | `operation: String`<br>`input: Value`<br>`query: Option<String>` | Remove duplicate `content` |

##### Media Processing Tools (3 tools)
| Tool | Old Parameters | New Parameters | Notes |
|------|---------------|----------------|-------|
| **ImageProcessorTool** | `operation: String`<br>`file_path: Option<String>`<br>`input_path: Option<String>`<br>`output_path: Option<String>` | `operation: String`<br>`source_path: PathBuf`<br>`target_path: Option<PathBuf>` | Consolidate path params |
| **AudioProcessorTool** | `operation: String`<br>`file_path: Option<String>`<br>`input_path: Option<String>`<br>`output_path: Option<String>` | `operation: String`<br>`source_path: PathBuf`<br>`target_path: Option<PathBuf>` | Consolidate path params |
| **VideoProcessorTool** | `operation: String`<br>`file_path: String`<br>`output_path: Option<String>` | `operation: String`<br>`source_path: PathBuf`<br>`target_path: Option<PathBuf>` | Standardize paths |

##### Utility Tools (9 tools)
| Tool | Old Parameters | New Parameters | Notes |
|------|---------------|----------------|-------|
| **Base64EncoderTool** | `operation: String`<br>`input: String` | `operation: String`<br>`input: String` | Already compliant ✓ |
| **CalculatorTool** | `expression: String`<br>`precision: Option<u8>` | `operation: String`<br>`input: String`<br>`precision: Option<u8>` | Add `operation`<br>`expression` → `input` |
| **TextManipulatorTool** | `text: String`<br>`operation: String` | `operation: String`<br>`input: String` | `text` → `input`<br>Reorder params |
| **HashCalculatorTool** | `algorithm: String`<br>`data: Option<String>`<br>`file_path: Option<String>` | `operation: String`<br>`input: Option<String>`<br>`path: Option<PathBuf>`<br>`algorithm: String` | Add `operation`<br>`data` → `input` |
| **DateTimeHandlerTool** | `operation: String`<br>`input: Option<String>` | `operation: String`<br>`input: Option<String>` | Already compliant ✓ |
| **UuidGeneratorTool** | `version: Option<u8>`<br>`namespace: Option<String>`<br>`name: Option<String>` | `operation: String`<br>`input: Option<String>`<br>`version: Option<u8>` | Add `operation`<br>Restructure |
| **DiffCalculatorTool** | `source: String`<br>`target: String` | `operation: String`<br>`source_path: PathBuf`<br>`target_path: PathBuf` | Add `operation`<br>Use path pattern |
| **TemplateEngineTool** | `template: String`<br>`context: Value` | `operation: String`<br>`input: String`<br>`context: Value` | Add `operation`<br>`template` → `input` |
| **DataValidationTool** | `data: Value`<br>`schema: Option<Value>` | `operation: String`<br>`input: Value`<br>`schema: Option<Value>` | Add `operation`<br>`data` → `input` |

##### System Integration Tools (4 tools)
| Tool | Old Parameters | New Parameters | Notes |
|------|---------------|----------------|-------|
| **ProcessExecutorTool** | `executable: String`<br>`arguments: Vec<String>` | `operation: String`<br>`input: String`<br>`arguments: Vec<String>` | Add `operation`<br>`executable` → `input` |
| **SystemMonitorTool** | `metrics: Vec<String>` | `operation: String`<br>`input: Vec<String>` | Add `operation`<br>`metrics` → `input` |
| **ServiceCheckerTool** | `service: String`<br>`host: String`<br>`port: u16` | `operation: String`<br>`input: String`<br>`host: String`<br>`port: u16` | Add `operation`<br>`service` → `input` |
| **EnvironmentReaderTool** | `operation: String`<br>`pattern: Option<String>` | `operation: String`<br>`input: Option<String>` | `pattern` → `input` |

##### API/Web Tools (3 tools)
| Tool | Old Parameters | New Parameters | Notes |
|------|---------------|----------------|-------|
| **HttpRequestTool** | `method: String`<br>`url: String` | `operation: String`<br>`input: String`<br>`method: String` | Add `operation`<br>`url` → `input` |
| **GraphQLQueryTool** | `endpoint: String`<br>`query: String` | `operation: String`<br>`input: String`<br>`endpoint: String` | Add `operation`<br>`query` → `input` |
| **WebSearchTool** | `query: String` | `operation: String`<br>`input: String` | Add `operation`<br>`query` → `input` |

**Summary**: 26 tools total - Only 2 already compliant (Base64EncoderTool, DateTimeHandlerTool)

#### 1.3 ResponseBuilder Pattern

**Standard Response Format**:
```rust
use llmspell_utils::response::ResponseBuilder;

#[derive(Debug, Serialize, Deserialize)]
pub struct StandardResponse {
    pub operation: String,
    pub success: bool,
    pub message: Option<String>,
    pub result: serde_json::Value,
    pub error: Option<ErrorDetails>,
}

impl ResponseBuilder {
    pub fn new(operation: &str) -> Self {
        Self {
            operation: operation.to_string(),
            success: true,
            message: None,
            result: Value::Null,
            error: None,
        }
    }
    
    pub fn with_result(mut self, result: impl Serialize) -> Self {
        self.result = serde_json::to_value(result).unwrap_or(Value::Null);
        self
    }
    
    pub fn with_error(mut self, error: &str) -> Self {
        self.success = false;
        self.error = Some(ErrorDetails::new(error));
        self
    }
    
    pub fn build(self) -> String {
        serde_json::to_string_pretty(&self).unwrap()
    }
}
```

**Validation Response Standard**:
```rust
// All validation operations MUST return:
{
    "operation": "validate",
    "success": true,
    "result": {
        "valid": bool,
        "errors": Option<Vec<ValidationError>>
    }
}
```

### 2. DRY Principle Enforcement

#### 2.1 Shared Validators (llmspell-utils)

**Move to llmspell_utils::validators**:
```rust
// From DataValidationTool -> llmspell_utils
pub fn validate_email(email: &str) -> Result<()>;
pub fn validate_url(url: &str) -> Result<()>;
pub fn validate_json_schema(data: &Value, schema: &Value) -> Result<()>;
pub fn validate_regex_pattern(pattern: &str) -> Result<()>;
pub fn validate_date_format(date: &str, format: &str) -> Result<()>;
```

#### 2.2 Shared HTTP Utilities

**Move to llmspell_utils::http**:
```rust
// From HttpRequestTool -> llmspell_utils
pub async fn retry_async<F, T>(
    operation: F,
    config: RetryConfig,
) -> Result<T>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T>>;

pub fn create_http_client(config: HttpClientConfig) -> Result<Client>;
pub fn parse_rate_limit_headers(headers: &HeaderMap) -> RateLimitInfo;
```

#### 2.3 Shared JSON Operations

**Move to llmspell_utils::serialization**:
```rust
// Standardize all JSON operations
pub fn to_json_pretty<T: Serialize>(value: &T) -> Result<String>;
pub fn from_json_str<T: DeserializeOwned>(s: &str) -> Result<T>;
pub fn merge_json_objects(base: Value, patch: Value) -> Value;
```

### 3. Breaking Changes Documentation

#### 3.1 Pre-1.0 Clean Break Approach

**Philosophy**: As a pre-1.0 project (version 0.1.0), we're making clean breaking changes without migration tools to achieve the best possible architecture before stability commitments.

#### 3.2 Comprehensive Change Documentation

**Create CHANGELOG_v0.3.0.md**:
```markdown
# Version 0.3.0 Breaking Changes

## Parameter Standardization
| Tool | Old Parameter | New Parameter | Example |
|------|---------------|---------------|---------|
| TextManipulator | text | input | `{"input": "hello"}` |
| JsonProcessor | input/content | input | `{"input": {...}}` |
| FileOperations | file_path | path | `{"path": "/tmp/file"}` |
| Calculator | query/expression | input | `{"input": "2+2"}` |
| DataValidator | data | input | `{"input": {...}}` |
| TemplateEngine | template | input | `{"input": "Hello {{name}}"}` |

## Response Format Changes
- All tools now use ResponseBuilder pattern
- Consistent success/error structure
- Standardized field names

## Upgrade Instructions
1. Update all tool parameter names in your scripts
2. Adjust response parsing for new format
3. Review security settings for hardened defaults
```

#### 3.3 Example Conversions Guide

```rust
// Before (v0.2.0)
let params = json!({
    "text": "hello world",
    "operation": "uppercase"
});

// After (v0.3.0)
let params = json!({
    "input": "hello world",
    "operation": "uppercase"
});

// Response parsing before
let result = response["result"].as_str();

// Response parsing after
let response = serde_json::from_str::<StandardResponse>(&output)?;
let result = response.result["text"].as_str();
```

### 4. Security Hardening (Moved from Phase 3.2)

#### 4.1 Critical Security Fixes

**Calculator DoS Protection**:
```rust
// Implement in Phase 3.0 for immediate security
pub struct CalculatorLimits {
    max_expression_length: usize,  // 1000 chars
    max_complexity_score: u32,     // Prevent exponential expressions
    max_eval_time_ms: u64,         // 100ms timeout
    max_memory_bytes: usize,       // 10MB limit
}
```

**Path Security for File Tools**:
```rust
// Extract to llmspell_utils::security::paths
pub fn validate_safe_path(path: &Path, jail_dir: &Path) -> Result<()> {
    // Prevent symlink attacks
    let canonical = path.canonicalize()?;
    if !canonical.starts_with(jail_dir) {
        return Err(SecurityError::PathEscape);
    }
    // Additional checks...
}
```

### 5. Implementation Checklist

**Week 9 Tasks**:
- [ ] Create ResponseBuilder in llmspell-utils
- [ ] Extract shared validators to llmspell-utils
- [ ] Extract HTTP utilities to llmspell-utils
- [ ] Implement critical security fixes (Calculator DoS, Path security)
- [ ] Update first 12 tools to new standards
- [ ] Create comprehensive change documentation

**Week 10 Tasks**:
- [ ] Update remaining 13 tools to new standards
- [ ] Complete security hardening for all file tools
- [ ] Write example conversion guide
- [ ] Update all tool documentation
- [ ] Performance optimization for standardized tools
- [ ] Run full regression and security test suite

---

## Phase 3.1: External Integration Tools (Weeks 11-12)

### Goal
Add 8 external integration tools following the standardized patterns established in Phase 3.0.

### 1. Web & Network Tools (7 tools)

#### 1.1 WebSearchTool Enhancement

**Current State**: Basic implementation exists from Phase 2  
**Enhancement Required**: Add real search provider APIs

```rust
// llmspell-tools/src/web/web_search.rs
pub struct WebSearchTool {
    providers: HashMap<String, Box<dyn SearchProvider>>,
    default_provider: String,
    rate_limiter: RateLimiter,
}

#[async_trait]
impl Tool for WebSearchTool {
    fn schema(&self) -> ToolSchema {
        ToolSchema::new("web_search", "Enhanced web search with multiple providers")
            .with_parameter("input", ParameterType::String, "Search query", true)
            .with_parameter("provider", ParameterType::String, "google|brave|duckduckgo|serpapi|serperdev", false)
            .with_parameter("max_results", ParameterType::Integer, "1-100", false)
            .with_parameter("search_type", ParameterType::String, "web|news|images", false)
    }
    
    async fn execute(&self, input: ToolInput) -> Result<ToolOutput> {
        let query = input.get_required_string("input")?;
        let provider = input.get_optional_string("provider")
            .unwrap_or(&self.default_provider);
        
        // Rate limiting
        self.rate_limiter.check_and_wait(provider).await?;
        
        // Provider execution
        let results = self.providers.get(provider)
            .ok_or_else(|| ToolError::InvalidProvider(provider.to_string()))?
            .search(&query, &input).await?;
        
        // Standard response
        ResponseBuilder::new("search")
            .with_result(results)
            .build()
    }
}
```

**Search Providers**:
```rust
// DuckDuckGo (no API key required)
pub struct DuckDuckGoProvider {
    client: Client,
}

// Google Custom Search (API key required)
pub struct GoogleSearchProvider {
    api_key: String,
    search_engine_id: String,
    client: Client,
}

// Brave Search (API key required)
pub struct BraveSearchProvider {
    api_key: String,
    client: Client,
}

// SerpApi.com (API key required)
pub struct SerpApiProvider {
    api_key: String,
    client: Client,
}

// Serper.dev (API key required)
pub struct SerperDevProvider {
    api_key: String,
    client: Client,
}
```

**Rate Limiting Considerations**:
Each provider has different rate limits that must be respected:
- DuckDuckGo: No official API, web scraping limits apply
- Google Custom Search: 100 queries/day (free tier), 10,000/day (paid)
- Brave: 2,000 queries/month (free tier), higher tiers available
- SerpApi: Varies by plan (100-5,000 searches/month)
- SerperDev: 2,500 queries/month (free tier), 50k+ (paid)

**Provider Selection Strategy**:
```rust
pub struct ProviderSelector {
    providers: Vec<(String, Box<dyn SearchProvider>)>,
    rate_limiters: HashMap<String, RateLimiter>,
    priority_order: Vec<String>, // Prefer free/high-limit providers
}
```

**Configuration Management**:
With 5 providers requiring different API keys and settings:
```rust
// Environment variables for API keys
WEBSEARCH_GOOGLE_API_KEY=...
WEBSEARCH_GOOGLE_SEARCH_ENGINE_ID=...
WEBSEARCH_BRAVE_API_KEY=...
WEBSEARCH_SERPAPI_API_KEY=...
WEBSEARCH_SERPERDEV_API_KEY=...

// Configuration structure
pub struct WebSearchConfig {
    providers: HashMap<String, ProviderConfig>,
    default_provider: String,
    fallback_chain: Vec<String>, // e.g., ["duckduckgo", "serperdev", "google"]
}
```

#### 1.2 Additional Web Tools

**WebScraperTool**:
```rust
pub struct WebScraperTool {
    client: Client,
    js_renderer: Option<JsRenderer>, // For JavaScript-heavy sites
}

impl Tool for WebScraperTool {
    fn schema(&self) -> ToolSchema {
        ToolSchema::new("web_scraper", "Extract content from web pages")
            .with_parameter("input", ParameterType::String, "URL to scrape", true)
            .with_parameter("selectors", ParameterType::Object, "CSS selectors", false)
            .with_parameter("wait_for_js", ParameterType::Boolean, "Wait for JS", false)
            .with_parameter("timeout", ParameterType::Integer, "Timeout in ms", false)
    }
}
```

**Other Web Tools**:
- `UrlAnalyzerTool`: URL validation, metadata extraction
- `ApiTesterTool`: REST API testing with assertions
- `WebhookCallerTool`: Webhook invocation with retries
- `WebpageMonitorTool`: Change detection and alerting
- `SitemapCrawlerTool`: Sitemap parsing and analysis

### 2. Communication Tools (1 tool)

#### 2.1 EmailSenderTool

```rust
pub struct EmailSenderTool {
    providers: HashMap<String, Box<dyn EmailProvider>>,
    default_provider: String,
}

impl Tool for EmailSenderTool {
    fn schema(&self) -> ToolSchema {
        ToolSchema::new("email_sender", "Send emails via multiple providers")
            .with_parameter("input", ParameterType::Object, "Email content", true)
            .with_parameter("provider", ParameterType::String, "smtp|sendgrid|ses", false)
    }
}

// Email input structure
#[derive(Deserialize)]
struct EmailInput {
    to: Vec<String>,
    subject: String,
    body: String,
    html: Option<String>,
    attachments: Option<Vec<Attachment>>,
}
```

#### 2.2 DatabaseConnectorTool

```rust
pub struct DatabaseConnectorTool {
    connections: HashMap<String, DatabaseConnection>,
    query_timeout: Duration,
}

impl Tool for DatabaseConnectorTool {
    fn schema(&self) -> ToolSchema {
        ToolSchema::new("database_connector", "Execute database queries")
            .with_parameter("input", ParameterType::String, "SQL query", true)
            .with_parameter("connection", ParameterType::String, "Connection name", true)
            .with_parameter("parameters", ParameterType::Array, "Query parameters", false)
    }
}
```

**Note**: SlackIntegrationTool and GitHubIntegrationTool have been deferred to Phase 19 (Additional Optional Enhancements).

### 4. Implementation Checklist

**Week 11 Tasks**:
- [ ] Implement WebSearchTool providers (Google, Brave, DuckDuckGo, SerpApi, SerperDev)
- [ ] Create provider selection and rate limiting system
- [ ] Create WebScraperTool with JS rendering
- [ ] Implement remaining web tools (5 tools)
- [ ] Create EmailSenderTool with providers
- [ ] Test all web/network tools with rate limit simulation

**Week 12 Tasks**:
- [ ] Implement DatabaseConnectorTool
- [ ] Complete EmailSenderTool testing
- [ ] Integration testing with Phase 3.0 standards
- [ ] Documentation for all 8 new tools

---

## Phase 3.2: Security & Performance (Weeks 13-14)

### Goal
Harden security across all 33 tools and optimize performance while maintaining the 52,600x speed advantage.

### 1. Security Hardening

#### 1.1 Calculator DoS Protection

**Current Vulnerability**: Calculator accepts arbitrarily complex expressions

```rust
// llmspell-tools/src/math/calculator.rs
impl CalculatorTool {
    fn validate_expression_complexity(&self, expr: &str) -> Result<()> {
        // Complexity metrics
        let depth = self.calculate_nesting_depth(expr)?;
        let operations = self.count_operations(expr)?;
        let length = expr.len();
        
        // Limits
        const MAX_DEPTH: usize = 10;
        const MAX_OPERATIONS: usize = 100;
        const MAX_LENGTH: usize = 1000;
        
        if depth > MAX_DEPTH {
            return Err(ToolError::ComplexityLimit("Nesting too deep"));
        }
        if operations > MAX_OPERATIONS {
            return Err(ToolError::ComplexityLimit("Too many operations"));
        }
        if length > MAX_LENGTH {
            return Err(ToolError::ComplexityLimit("Expression too long"));
        }
        
        Ok(())
    }
    
    async fn execute(&self, input: ToolInput) -> Result<ToolOutput> {
        let expr = input.get_required_string("input")?;
        
        // Validate before evaluation
        self.validate_expression_complexity(&expr)?;
        
        // Timeout protection
        let result = timeout(Duration::from_secs(1), async {
            self.evaluate_expression(&expr)
        }).await??;
        
        ResponseBuilder::new("calculate")
            .with_result(result)
            .build()
    }
}
```

#### 1.2 Symlink Escape Prevention

**Current Vulnerability**: File operations may follow symlinks outside sandbox

```rust
// llmspell-security/src/sandbox/file_sandbox.rs
impl FileSandbox {
    pub fn resolve_path_safely(&self, path: &Path) -> Result<PathBuf> {
        // Canonicalize without following symlinks
        let mut resolved = self.sandbox_root.clone();
        
        for component in path.components() {
            match component {
                Component::ParentDir => {
                    if !resolved.pop() {
                        return Err(SecurityError::PathEscape);
                    }
                }
                Component::Normal(name) => {
                    resolved.push(name);
                    
                    // Check if it's a symlink
                    if resolved.is_symlink() {
                        let target = fs::read_link(&resolved)?;
                        let resolved_target = self.resolve_symlink_target(&resolved, &target)?;
                        
                        // Ensure target is within sandbox
                        if !resolved_target.starts_with(&self.sandbox_root) {
                            return Err(SecurityError::SymlinkEscape(resolved_target));
                        }
                        
                        resolved = resolved_target;
                    }
                }
                _ => {}
            }
        }
        
        // Final check
        if !resolved.starts_with(&self.sandbox_root) {
            return Err(SecurityError::PathEscape);
        }
        
        Ok(resolved)
    }
}
```

#### 1.3 Resource Limit Enforcement

```rust
// llmspell-security/src/limits.rs
#[derive(Clone, Debug)]
pub struct ResourceLimits {
    pub max_file_size: u64,         // Maximum file size for operations
    pub max_memory_usage: u64,      // Maximum memory per tool
    pub max_execution_time: Duration, // Maximum execution time
    pub max_open_files: usize,      // Maximum open file handles
    pub max_network_connections: usize, // Maximum concurrent connections
}

impl Default for ResourceLimits {
    fn default() -> Self {
        Self {
            max_file_size: 100 * 1024 * 1024,        // 100MB
            max_memory_usage: 512 * 1024 * 1024,     // 512MB
            max_execution_time: Duration::from_secs(30),
            max_open_files: 100,
            max_network_connections: 10,
        }
    }
}

// Enforcement middleware
pub struct ResourceLimitGuard {
    limits: ResourceLimits,
    current_usage: Arc<Mutex<ResourceUsage>>,
}

impl ResourceLimitGuard {
    pub async fn check_file_size(&self, size: u64) -> Result<()> {
        if size > self.limits.max_file_size {
            return Err(SecurityError::FileSizeLimit(size, self.limits.max_file_size));
        }
        Ok(())
    }
    
    pub async fn track_memory<F, R>(&self, f: F) -> Result<R>
    where
        F: FnOnce() -> R,
    {
        let start_memory = self.get_current_memory();
        let result = f();
        let end_memory = self.get_current_memory();
        
        let used = end_memory.saturating_sub(start_memory);
        if used > self.limits.max_memory_usage {
            return Err(SecurityError::MemoryLimit(used, self.limits.max_memory_usage));
        }
        
        Ok(result)
    }
}
```

### 2. Performance Optimization

#### 2.1 Shared Resource Pools

```rust
// llmspell-utils/src/pools.rs
lazy_static! {
    // Shared HTTP client pool
    static ref HTTP_CLIENT_POOL: ClientPool = ClientPool::new(
        ClientConfig {
            pool_size: 10,
            timeout: Duration::from_secs(30),
            keep_alive: true,
        }
    );
    
    // Shared regex cache
    static ref REGEX_CACHE: Mutex<LruCache<String, Regex>> = 
        Mutex::new(LruCache::new(NonZeroUsize::new(100).unwrap()));
    
    // Shared template engine
    static ref TEMPLATE_ENGINE: RwLock<Tera> = RwLock::new({
        let mut tera = Tera::default();
        tera.autoescape_on(vec![".html", ".xml"]);
        tera
    });
}

// Usage in tools
impl HttpRequestTool {
    pub fn new() -> Self {
        Self {
            client: HTTP_CLIENT_POOL.get(), // Reuse shared client
        }
    }
}
```

#### 2.2 Caching Strategies

```rust
// llmspell-utils/src/cache.rs
pub struct ToolCache {
    memory_cache: Arc<Mutex<LruCache<String, CachedResult>>>,
    disk_cache: Option<DiskCache>,
}

impl ToolCache {
    pub async fn get_or_compute<F, Fut, T>(&self, key: &str, f: F) -> Result<T>
    where
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T>>,
        T: Serialize + DeserializeOwned,
    {
        // Check memory cache
        if let Some(cached) = self.memory_cache.lock().await.get(key) {
            if !cached.is_expired() {
                return Ok(serde_json::from_value(cached.value.clone())?);
            }
        }
        
        // Compute result
        let result = f().await?;
        
        // Cache result
        let cached = CachedResult {
            value: serde_json::to_value(&result)?,
            timestamp: Instant::now(),
            ttl: Duration::from_secs(300),
        };
        
        self.memory_cache.lock().await.put(key.to_string(), cached);
        
        Ok(result)
    }
}
```

### 3. Implementation Checklist

**Week 13 Tasks**:
- [ ] Implement calculator complexity validation
- [ ] Create symlink escape prevention
- [ ] Build resource limit framework
- [ ] Add security middleware to all tools
- [ ] Security audit first 20 tools

**Week 14 Tasks**:
- [ ] Security audit remaining 21 tools
- [ ] Implement shared resource pools
- [ ] Add caching to appropriate tools
- [ ] Performance benchmarking
- [ ] Create security test suite

---

## Phase 3.3: Workflow Orchestration (Weeks 15-16)

### Goal
Implement comprehensive workflow orchestration patterns that leverage all 41+ standardized and secured tools.

### 1. Workflow Trait System

```rust
// llmspell-workflows/src/traits.rs
#[async_trait]
pub trait Workflow: Send + Sync {
    /// Unique identifier for the workflow
    fn id(&self) -> &str;
    
    /// Human-readable name
    fn name(&self) -> &str;
    
    /// Workflow metadata
    fn metadata(&self) -> &WorkflowMetadata;
    
    /// Execute the workflow
    async fn execute(&self, input: WorkflowInput, context: ExecutionContext) -> Result<WorkflowOutput>;
    
    /// Validate workflow configuration
    fn validate(&self) -> Result<()>;
    
    /// Get workflow schema for validation
    fn schema(&self) -> WorkflowSchema;
}

#[derive(Debug, Clone)]
pub struct WorkflowMetadata {
    pub version: String,
    pub description: String,
    pub author: Option<String>,
    pub tags: Vec<String>,
    pub required_tools: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct WorkflowInput {
    pub initial_data: Value,
    pub parameters: HashMap<String, Value>,
    pub starting_step: Option<String>,
}

#[derive(Debug, Clone)]
pub struct WorkflowOutput {
    pub final_result: Value,
    pub step_results: HashMap<String, StepResult>,
    pub execution_path: Vec<String>,
    pub metrics: WorkflowMetrics,
}
```

### 2. Workflow Patterns

#### 2.1 Sequential Workflow

```rust
pub struct SequentialWorkflow {
    id: String,
    name: String,
    steps: Vec<WorkflowStep>,
    error_handling: ErrorStrategy,
}

impl SequentialWorkflow {
    pub fn builder(name: &str) -> SequentialWorkflowBuilder {
        SequentialWorkflowBuilder::new(name)
    }
}

#[async_trait]
impl Workflow for SequentialWorkflow {
    async fn execute(&self, input: WorkflowInput, context: ExecutionContext) -> Result<WorkflowOutput> {
        let mut state = WorkflowState::new(input.initial_data);
        let mut step_results = HashMap::new();
        let mut execution_path = Vec::new();
        
        for step in &self.steps {
            execution_path.push(step.id.clone());
            
            // Execute step
            let step_input = self.prepare_step_input(&step, &state)?;
            let result = match self.execute_step(&step, step_input, &context).await {
                Ok(r) => r,
                Err(e) => {
                    match self.error_handling {
                        ErrorStrategy::Fail => return Err(e),
                        ErrorStrategy::Continue => {
                            step_results.insert(step.id.clone(), StepResult::Failed(e.to_string()));
                            continue;
                        }
                        ErrorStrategy::Retry(attempts) => {
                            self.retry_step(&step, step_input, &context, attempts).await?
                        }
                    }
                }
            };
            
            // Update state
            state.update(&step.id, &result)?;
            step_results.insert(step.id.clone(), StepResult::Success(result));
        }
        
        Ok(WorkflowOutput {
            final_result: state.get_final_result()?,
            step_results,
            execution_path,
            metrics: state.get_metrics(),
        })
    }
}
```

#### 2.2 Conditional Workflow

```rust
pub struct ConditionalWorkflow {
    id: String,
    name: String,
    initial_step: String,
    steps: HashMap<String, ConditionalStep>,
    conditions: HashMap<String, Condition>,
}

#[derive(Debug, Clone)]
pub struct ConditionalStep {
    pub id: String,
    pub tool: String,
    pub parameters: Value,
    pub branches: Vec<Branch>,
}

#[derive(Debug, Clone)]
pub struct Branch {
    pub condition: String,
    pub next_step: Option<String>,
}

impl ConditionalWorkflow {
    async fn execute_step(&self, step_id: &str, state: &WorkflowState, context: &ExecutionContext) -> Result<(StepResult, Option<String>)> {
        let step = self.steps.get(step_id)
            .ok_or_else(|| WorkflowError::StepNotFound(step_id.to_string()))?;
        
        // Execute the tool
        let tool_result = self.execute_tool(&step.tool, &step.parameters, state, context).await?;
        
        // Evaluate branches
        for branch in &step.branches {
            let condition = self.conditions.get(&branch.condition)
                .ok_or_else(|| WorkflowError::ConditionNotFound(branch.condition.clone()))?;
            
            if condition.evaluate(&tool_result, state)? {
                return Ok((StepResult::Success(tool_result), branch.next_step.clone()));
            }
        }
        
        // No matching branch
        Ok((StepResult::Success(tool_result), None))
    }
}
```

#### 2.3 Loop Workflow

```rust
pub struct LoopWorkflow {
    id: String,
    name: String,
    iterator: Iterator,
    body: Box<dyn Workflow>,
    max_iterations: Option<usize>,
    break_condition: Option<Condition>,
}

#[derive(Debug, Clone)]
pub enum Iterator {
    Collection(String),      // Iterate over collection in state
    Range(usize, usize),    // Iterate over numeric range
    WhileCondition(Condition), // While condition is true
}

impl LoopWorkflow {
    async fn execute(&self, input: WorkflowInput, context: ExecutionContext) -> Result<WorkflowOutput> {
        let mut state = WorkflowState::new(input.initial_data);
        let mut iterations = 0;
        let mut all_results = Vec::new();
        
        loop {
            // Check max iterations
            if let Some(max) = self.max_iterations {
                if iterations >= max {
                    break;
                }
            }
            
            // Check break condition
            if let Some(condition) = &self.break_condition {
                if condition.evaluate(&Value::Null, &state)? {
                    break;
                }
            }
            
            // Get next item
            let item = match &self.iterator {
                Iterator::Collection(path) => {
                    state.get_collection_item(path, iterations)?
                }
                Iterator::Range(start, end) => {
                    if start + iterations >= *end {
                        break;
                    }
                    Value::Number((start + iterations).into())
                }
                Iterator::WhileCondition(condition) => {
                    if !condition.evaluate(&Value::Null, &state)? {
                        break;
                    }
                    Value::Null
                }
            };
            
            // Execute body with item
            let body_input = WorkflowInput {
                initial_data: item,
                parameters: state.get_loop_parameters(),
                starting_step: None,
            };
            
            let result = self.body.execute(body_input, context.clone()).await?;
            all_results.push(result.final_result);
            
            iterations += 1;
        }
        
        Ok(WorkflowOutput {
            final_result: Value::Array(all_results),
            step_results: HashMap::new(),
            execution_path: vec![format!("loop_{}_iterations", iterations)],
            metrics: WorkflowMetrics::default(),
        })
    }
}
```

#### 2.4 Streaming Workflow

```rust
pub struct StreamingWorkflow {
    id: String,
    name: String,
    source: StreamSource,
    pipeline: Vec<StreamProcessor>,
    sink: StreamSink,
    backpressure_strategy: BackpressureStrategy,
}

#[derive(Debug, Clone)]
pub enum StreamSource {
    Tool(String, Value),           // Tool that produces stream
    File(PathBuf),                // File to stream
    Network(String),              // Network endpoint
    Collection(Vec<Value>),       // In-memory collection
}

impl StreamingWorkflow {
    async fn execute(&self, input: WorkflowInput, context: ExecutionContext) -> Result<WorkflowOutput> {
        // Create stream from source
        let stream = self.create_stream(&self.source, &context).await?;
        
        // Apply processors
        let processed = self.pipeline.iter().fold(stream, |s, processor| {
            processor.apply(s, &context)
        });
        
        // Handle backpressure
        let controlled = match self.backpressure_strategy {
            BackpressureStrategy::Buffer(size) => processed.buffer_unordered(size),
            BackpressureStrategy::Drop => processed.drop_on_overflow(),
            BackpressureStrategy::Pause => processed.pause_on_pressure(),
        };
        
        // Collect to sink
        let results = self.sink.collect(controlled).await?;
        
        Ok(WorkflowOutput {
            final_result: results,
            step_results: HashMap::new(),
            execution_path: vec!["streaming".to_string()],
            metrics: WorkflowMetrics::default(),
        })
    }
}
```

### 3. Workflow State Management

```rust
pub struct WorkflowState {
    data: HashMap<String, Value>,
    history: Vec<StateChange>,
    variables: HashMap<String, Variable>,
}

impl WorkflowState {
    pub fn new(initial_data: Value) -> Self {
        let mut data = HashMap::new();
        data.insert("$input".to_string(), initial_data);
        
        Self {
            data,
            history: Vec::new(),
            variables: HashMap::new(),
        }
    }
    
    pub fn get(&self, path: &str) -> Result<&Value> {
        // JSONPath-like access
        jsonpath::select(&self.data, path)?
            .first()
            .ok_or_else(|| StateError::PathNotFound(path.to_string()))
    }
    
    pub fn set(&mut self, path: &str, value: Value) -> Result<()> {
        // Record change
        self.history.push(StateChange {
            timestamp: Instant::now(),
            path: path.to_string(),
            old_value: self.get(path).ok().cloned(),
            new_value: value.clone(),
        });
        
        // Update data
        jsonpath::set(&mut self.data, path, value)?;
        Ok(())
    }
    
    pub fn merge(&mut self, other: &WorkflowState) -> Result<()> {
        for (key, value) in &other.data {
            if key != "$input" {
                self.set(key, value.clone())?;
            }
        }
        Ok(())
    }
}
```

### 4. Workflow Examples

#### 4.1 Research Workflow

```rust
let research_workflow = SequentialWorkflow::builder("research_assistant")
    .add_step("search", "web_search", json!({
        "input": "{{query}}",
        "max_results": 10
    }))
    .add_step("extract", "web_scraper", json!({
        "input": "{{search.results[0].url}}",
        "selectors": {
            "title": "h1",
            "content": ".article-body"
        }
    }))
    .add_step("summarize", "text_summarizer", json!({
        "input": "{{extract.result.content}}",
        "max_length": 500
    }))
    .add_step("analyze", "sentiment_analyzer", json!({
        "input": "{{summarize.result}}",
        "operations": ["sentiment", "entities", "keywords"]
    }))
    .with_error_handling(ErrorStrategy::Retry(3))
    .build()?;
```

#### 4.2 Data Processing Pipeline

```rust
let etl_workflow = StreamingWorkflow::builder("etl_pipeline")
    .source(StreamSource::Tool("database_connector", json!({
        "query": "SELECT * FROM users WHERE created_at > ?",
        "parameters": ["2025-01-01"]
    })))
    .add_processor(StreamProcessor::Transform("data_transformer", json!({
        "operation": "map",
        "mapping": {
            "id": "user_id",
            "email": "contact_email"
        }
    })))
    .add_processor(StreamProcessor::Filter("data_validator", json!({
        "rules": {
            "contact_email": {"type": "email"}
        }
    })))
    .add_processor(StreamProcessor::Batch(100))
    .sink(StreamSink::Tool("file_writer", json!({
        "path": "/output/users.jsonl",
        "format": "jsonl"
    })))
    .with_backpressure(BackpressureStrategy::Buffer(1000))
    .build()?;
```

### 5. Implementation Checklist

**Week 15 Tasks**:
- [ ] Implement Workflow trait system
- [ ] Create SequentialWorkflow
- [ ] Create ConditionalWorkflow
- [ ] Implement workflow state management
- [ ] Create workflow builder patterns

**Week 16 Tasks**:
- [ ] Implement LoopWorkflow
- [ ] Create StreamingWorkflow
- [ ] Build workflow examples
- [ ] Integration testing with all 33 tools
- [ ] Performance benchmarking

---

## Testing Strategy

### 1. Unit Tests
- Each tool standardization change
- Security vulnerability fixes
- Workflow pattern implementations
- State management operations

### 2. Integration Tests
- Tool chain workflows
- Cross-tool data flow
- Security boundary testing
- Performance regression tests

### 3. Upgrade Validation Tests
- Parameter standardization verification
- Response format consistency checks
- Breaking change documentation accuracy
- Example code validation

### 4. Security Tests
- Penetration testing for each vulnerability
- Resource exhaustion tests
- Sandbox escape attempts
- Permission boundary validation

### 5. Performance Tests
- Tool initialization benchmarks
- Workflow execution timing
- Memory usage profiling
- Concurrent execution stress tests

---

## Documentation Requirements

### 1. Breaking Changes Guide
- Complete parameter mapping table
- Response format changes documentation
- Before/after code examples
- Manual upgrade instructions

### 2. Tool Documentation
- Updated schemas for all 33 tools
- Usage examples with new interfaces
- Performance characteristics
- Security considerations

### 3. Workflow Documentation
- Pattern explanations
- Builder API reference
- Example workflows
- Best practices guide

### 4. Security Documentation
- Vulnerability descriptions
- Mitigation strategies
- Security test results
- Ongoing security practices

---

## Risk Mitigation

### 1. Breaking Changes
- **Risk**: Existing scripts break with new interfaces
- **Mitigation**: Clear documentation and example conversions
- **Validation**: Test upgraded example scripts

### 2. Performance Regression
- **Risk**: Security checks slow down tools
- **Mitigation**: Optimize hot paths, use caching
- **Validation**: Continuous benchmarking

### 3. Integration Complexity
- **Risk**: 33 tools with different patterns
- **Mitigation**: Strict adherence to standards
- **Validation**: Integration test suite

### 4. Security Gaps
- **Risk**: New vulnerabilities introduced
- **Mitigation**: Security review checklist
- **Validation**: Penetration testing

---

## Success Metrics

### Quantitative Metrics
- Tool parameter consistency: 95%
- DRY code compliance: 95%
- Security test coverage: 100%
- Performance maintenance: <10ms initialization
- Documentation completeness: 100%

### Qualitative Metrics
- Developer experience improvement
- Clear documentation and examples
- Clean, maintainable codebase
- Robust workflow patterns
- Comprehensive security posture

---

## Handoff Criteria

### Phase 3.0 Handoff
- [ ] All 26 tools use standard interfaces
- [ ] Breaking changes documented
- [ ] Security hardening implemented
- [ ] All tests passing

### Phase 3.1 Handoff
- [ ] 16 external tools implemented
- [ ] Following Phase 3.0 standards
- [ ] Integration tests passing
- [ ] Documentation complete

### Phase 3.2 Handoff
- [ ] All security vulnerabilities addressed
- [ ] Performance benchmarks maintained
- [ ] Security test suite complete
- [ ] Resource limits enforced

### Phase 3.3 Handoff
- [ ] All workflow patterns implemented
- [ ] 41+ tools integrated
- [ ] Example workflows functional
- [ ] Full test suite passing

---

This comprehensive design document provides the detailed specifications needed to implement Phase 3's ambitious 8-week plan, transforming the tool library into a standardized, secure, and workflow-ready ecosystem.