# Phase 3: Tool Enhancement & Agent Infrastructure - Design Document

**Version**: 1.0  
**Date**: July 2025  
**Status**: Implementation Ready  
**Phase**: 3 (Tool Enhancement & Agent Infrastructure)  
**Timeline**: Weeks 9-16 (8 weeks)  
**Priority**: HIGH (MVP Completion)

> **📋 Comprehensive Implementation Guide**: This document provides complete specifications for implementing Phase 3, which spans 8 weeks and encompasses critical tool fixes, external integrations, security hardening, and agent infrastructure.

---

## Phase Overview

### Goal
Transform the existing 26 self-contained tools into a standardized, secure, and extensible library of 33+ tools, then implement comprehensive agent infrastructure patterns that leverage the full tool ecosystem.

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
4. **Phase 3.3 (Weeks 15-16)**: Agent Infrastructure - Factory, Registry, and Templates

### Success Metrics
- **Tool Consistency**: 95% parameter standardization (from 60%)
- **DRY Compliance**: 95% shared utility usage (from 80%)
- **Security Coverage**: Comprehensive vulnerability mitigation
- **Tool Count**: 33+ production-ready tools
- **Agent Support**: Factory, Registry, and Templates operational with full tool library

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
pub fn parse_rate_limit_headers(headers: &HeaderMap) -> RateLimitInfo; // Now in llmspell-utils
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
        
        // Rate limiting (using llmspell-utils::ProviderRateLimiter)
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
- [x] Create provider selection and rate limiting system (completed as llmspell-utils utility)
- [x] Implement circuit breaker pattern for fault tolerance (completed as llmspell-utils utility)
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

## Phase 3.3: Agent Infrastructure (Weeks 15-16)

### Goal
Implement comprehensive agent infrastructure including factory patterns, registry system, lifecycle management, and pre-configured templates that leverage all 41+ standardized and secured tools.

### 1. Agent Factory Pattern

```rust
// llmspell-agents/src/factory.rs
pub struct AgentFactory {
    builders: HashMap<String, Box<dyn AgentBuilder>>,
    tool_registry: Arc<ToolRegistry>,
    llm_providers: Arc<LlmProviderRegistry>,
    default_config: AgentConfig,
}

impl AgentFactory {
    pub fn new(tool_registry: Arc<ToolRegistry>) -> Self {
        Self {
            builders: HashMap::new(),
            tool_registry,
            llm_providers: Arc::new(LlmProviderRegistry::default()),
            default_config: AgentConfig::default(),
        }
    }
    
    pub fn register_builder<B: AgentBuilder + 'static>(&mut self, name: &str, builder: B) {
        self.builders.insert(name.to_string(), Box::new(builder));
    }
    
    pub async fn create_agent(&self, spec: &AgentSpec) -> Result<Box<dyn Agent>> {
        let builder = self.builders.get(&spec.agent_type)
            .ok_or_else(|| AgentError::UnknownType(spec.agent_type.clone()))?;
        
        let mut agent = builder.build(spec)?;
        
        // Configure with tools
        for tool_name in &spec.tools {
            let tool = self.tool_registry.get(tool_name)?;
            agent.add_tool(tool_name, tool)?;
        }
        
        // Configure LLM provider
        if let Some(provider_spec) = &spec.llm_provider {
            let provider = self.llm_providers.create(provider_spec)?;
            agent.set_llm_provider(provider)?;
        }
        
        // Apply configuration
        agent.configure(spec.config.as_ref().unwrap_or(&self.default_config))?;
        
        Ok(agent)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentSpec {
    pub name: String,
    pub agent_type: String,
    pub description: Option<String>,
    pub tools: Vec<String>,
    pub llm_provider: Option<LlmProviderSpec>,
    pub config: Option<AgentConfig>,
    pub metadata: HashMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentConfig {
    pub max_iterations: usize,
    pub timeout: Duration,
    pub temperature: f32,
    pub max_tokens: usize,
    pub retry_policy: RetryPolicy,
    pub memory_config: MemoryConfig,
    pub security_config: SecurityConfig,
}

pub trait AgentBuilder: Send + Sync {
    fn build(&self, spec: &AgentSpec) -> Result<Box<dyn Agent>>;
    fn supported_features(&self) -> Vec<AgentFeature>;
    fn validate_spec(&self, spec: &AgentSpec) -> Result<()>;
}
```

### 2. Agent Registry with Discovery

#### 2.1 Registry Architecture

```rust
// llmspell-agents/src/registry.rs
pub struct AgentRegistry {
    agents: Arc<RwLock<HashMap<String, AgentEntry>>>,
    factory: Arc<AgentFactory>,
    discovery: Arc<DiscoveryService>,
    persistence: Arc<dyn RegistryPersistence>,
}

#[derive(Debug, Clone)]
pub struct AgentEntry {
    pub id: String,
    pub spec: AgentSpec,
    pub instance: Option<Arc<Mutex<Box<dyn Agent>>>>,
    pub status: AgentStatus,
    pub metadata: RegistryMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryMetadata {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub last_accessed: DateTime<Utc>,
    pub access_count: u64,
    pub tags: Vec<String>,
    pub capabilities: Vec<AgentCapability>,
}

impl AgentRegistry {
    pub async fn new(factory: Arc<AgentFactory>, config: RegistryConfig) -> Result<Self> {
        let persistence = Self::create_persistence(&config)?;
        let discovery = Arc::new(DiscoveryService::new(config.discovery));
        
        let mut registry = Self {
            agents: Arc::new(RwLock::new(HashMap::new())),
            factory,
            discovery,
            persistence,
        };
        
        // Load persisted agents
        registry.load_from_persistence().await?;
        
        // Start discovery service
        registry.start_discovery().await?;
        
        Ok(registry)
    }
    
    pub async fn register(&self, spec: AgentSpec) -> Result<String> {
        let id = Uuid::new_v4().to_string();
        let entry = AgentEntry {
            id: id.clone(),
            spec: spec.clone(),
            instance: None,
            status: AgentStatus::Registered,
            metadata: RegistryMetadata::new(&spec),
        };
        
        self.agents.write().await.insert(id.clone(), entry);
        self.persistence.save_agent(&id, &spec).await?;
        
        Ok(id)
    }
    
    pub async fn instantiate(&self, id: &str) -> Result<Arc<Mutex<Box<dyn Agent>>>> {
        let mut agents = self.agents.write().await;
        let entry = agents.get_mut(id)
            .ok_or_else(|| RegistryError::AgentNotFound(id.to_string()))?;
        
        if let Some(instance) = &entry.instance {
            entry.metadata.last_accessed = Utc::now();
            entry.metadata.access_count += 1;
            return Ok(instance.clone());
        }
        
        // Create new instance
        let agent = self.factory.create_agent(&entry.spec).await?;
        let instance = Arc::new(Mutex::new(agent));
        
        entry.instance = Some(instance.clone());
        entry.status = AgentStatus::Active;
        entry.metadata.last_accessed = Utc::now();
        entry.metadata.access_count += 1;
        
        Ok(instance)
    }
    
    pub async fn discover(&self, query: &DiscoveryQuery) -> Result<Vec<AgentEntry>> {
        let agents = self.agents.read().await;
        let mut results = Vec::new();
        
        for entry in agents.values() {
            if self.matches_query(entry, query) {
                results.push(entry.clone());
            }
        }
        
        // Sort by relevance
        results.sort_by(|a, b| {
            self.calculate_relevance(a, query)
                .partial_cmp(&self.calculate_relevance(b, query))
                .unwrap_or(Ordering::Equal)
                .reverse()
        });
        
        Ok(results)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryQuery {
    pub capabilities: Vec<AgentCapability>,
    pub tags: Vec<String>,
    pub tools: Vec<String>,
    pub text_search: Option<String>,
    pub filters: HashMap<String, Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentCapability {
    Research,
    CodeGeneration,
    DataAnalysis,
    Conversation,
    TaskAutomation,
    ApiIntegration,
    FileProcessing,
    WebScraping,
    Custom(String),
}
```

#### 2.2 Discovery Service

```rust
pub struct DiscoveryService {
    providers: Vec<Box<dyn DiscoveryProvider>>,
    cache: Arc<Mutex<DiscoveryCache>>,
    config: DiscoveryConfig,
}

#[async_trait]
pub trait DiscoveryProvider: Send + Sync {
    async fn discover(&self, query: &DiscoveryQuery) -> Result<Vec<DiscoveredAgent>>;
    fn provider_name(&self) -> &str;
    fn supports_capability(&self, capability: &AgentCapability) -> bool;
}

// Local file-based discovery
pub struct FileDiscoveryProvider {
    search_paths: Vec<PathBuf>,
    file_patterns: Vec<String>,
}

#[async_trait]
impl DiscoveryProvider for FileDiscoveryProvider {
    async fn discover(&self, query: &DiscoveryQuery) -> Result<Vec<DiscoveredAgent>> {
        let mut results = Vec::new();
        
        for path in &self.search_paths {
            for entry in WalkDir::new(path) {
                if let Ok(entry) = entry {
                    if self.matches_pattern(&entry) {
                        if let Ok(spec) = self.load_agent_spec(&entry.path()).await {
                            if self.matches_query(&spec, query) {
                                results.push(DiscoveredAgent {
                                    spec,
                                    source: DiscoverySource::File(entry.path().to_path_buf()),
                                    confidence: 1.0,
                                });
                            }
                        }
                    }
                }
            }
        }
        
        Ok(results)
    }
}

// Network-based discovery
pub struct NetworkDiscoveryProvider {
    registry_urls: Vec<String>,
    client: reqwest::Client,
}

#[async_trait]
impl DiscoveryProvider for NetworkDiscoveryProvider {
    async fn discover(&self, query: &DiscoveryQuery) -> Result<Vec<DiscoveredAgent>> {
        let mut results = Vec::new();
        
        for url in &self.registry_urls {
            let response = self.client
                .post(format!("{}/discover", url))
                .json(query)
                .send()
                .await?;
                
            let agents: Vec<DiscoveredAgent> = response.json().await?;
            results.extend(agents);
        }
        
        Ok(results)
    }
}
```

### 3. Agent Lifecycle Management

```rust
// llmspell-agents/src/lifecycle.rs
pub struct AgentLifecycleManager {
    registry: Arc<AgentRegistry>,
    state_store: Arc<dyn StateStore>,
    monitor: Arc<AgentMonitor>,
    scheduler: Arc<AgentScheduler>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentState {
    Created,
    Initializing,
    Ready,
    Running,
    Paused,
    Stopping,
    Stopped,
    Failed(AgentError),
}

impl AgentLifecycleManager {
    pub async fn transition(&self, agent_id: &str, new_state: AgentState) -> Result<()> {
        let current_state = self.state_store.get_state(agent_id).await?;
        
        // Validate transition
        if !self.is_valid_transition(&current_state, &new_state) {
            return Err(LifecycleError::InvalidTransition(
                current_state,
                new_state
            ));
        }
        
        // Execute transition
        match new_state {
            AgentState::Initializing => self.initialize_agent(agent_id).await?,
            AgentState::Running => self.start_agent(agent_id).await?,
            AgentState::Paused => self.pause_agent(agent_id).await?,
            AgentState::Stopping => self.stop_agent(agent_id).await?,
            _ => {}
        }
        
        // Update state
        self.state_store.set_state(agent_id, new_state).await?;
        
        // Notify monitors
        self.monitor.notify_state_change(agent_id, current_state, new_state).await;
        
        Ok(())
    }
    
    fn is_valid_transition(&self, from: &AgentState, to: &AgentState) -> bool {
        match (from, to) {
            (AgentState::Created, AgentState::Initializing) => true,
            (AgentState::Initializing, AgentState::Ready) => true,
            (AgentState::Ready, AgentState::Running) => true,
            (AgentState::Running, AgentState::Paused) => true,
            (AgentState::Running, AgentState::Stopping) => true,
            (AgentState::Paused, AgentState::Running) => true,
            (AgentState::Paused, AgentState::Stopping) => true,
            (AgentState::Stopping, AgentState::Stopped) => true,
            (_, AgentState::Failed(_)) => true,
            _ => false,
        }
    }
}

// State machine for agent lifecycle
pub struct AgentStateMachine {
    states: HashMap<AgentState, StateHandler>,
    transitions: HashMap<(AgentState, AgentState), TransitionHandler>,
}

type StateHandler = Box<dyn Fn(&Agent) -> Result<()> + Send + Sync>;
type TransitionHandler = Box<dyn Fn(&Agent, &AgentState, &AgentState) -> Result<()> + Send + Sync>;

impl AgentStateMachine {
    pub fn new() -> Self {
        let mut sm = Self {
            states: HashMap::new(),
            transitions: HashMap::new(),
        };
        
        // Register state handlers
        sm.register_state(AgentState::Initializing, Box::new(|agent| {
            agent.load_tools()?;
            agent.connect_llm()?;
            agent.initialize_memory()?;
            Ok(())
        }));
        
        sm.register_state(AgentState::Running, Box::new(|agent| {
            agent.start_message_loop()?;
            agent.enable_monitoring()?;
            Ok(())
        }));
        
        sm.register_state(AgentState::Stopping, Box::new(|agent| {
            agent.stop_message_loop()?;
            agent.save_state()?;
            agent.cleanup_resources()?;
            Ok(())
        }));
        
        sm
    }
}
```

### 4. Agent Templates

```rust
// llmspell-agents/src/templates/mod.rs
pub trait AgentTemplate {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn build_spec(&self, params: &TemplateParams) -> Result<AgentSpec>;
    fn default_tools(&self) -> Vec<&'static str>;
    fn required_capabilities(&self) -> Vec<AgentCapability>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateParams {
    pub name: String,
    pub custom_tools: Vec<String>,
    pub llm_config: Option<LlmConfig>,
    pub memory_size: Option<usize>,
    pub custom_prompts: HashMap<String, String>,
}

// Research Agent Template
pub struct ResearchAgentTemplate;

impl AgentTemplate for ResearchAgentTemplate {
    fn name(&self) -> &str { "research_agent" }
    
    fn description(&self) -> &str {
        "Agent optimized for research tasks with web search, content analysis, and summarization"
    }
    
    fn default_tools(&self) -> Vec<&'static str> {
        vec![
            "web_search",
            "web_scraper",
            "text_summarizer",
            "file_writer",
            "json_processor",
            "template_engine",
        ]
    }
    
    fn build_spec(&self, params: &TemplateParams) -> Result<AgentSpec> {
        let mut tools = self.default_tools().iter().map(|s| s.to_string()).collect::<Vec<_>>();
        tools.extend(params.custom_tools.clone());
        
        Ok(AgentSpec {
            name: params.name.clone(),
            agent_type: "research".to_string(),
            description: Some(self.description().to_string()),
            tools,
            llm_provider: Some(LlmProviderSpec {
                provider: "openai".to_string(),
                model: "gpt-4".to_string(),
                config: params.llm_config.clone(),
            }),
            config: Some(AgentConfig {
                max_iterations: 20,
                timeout: Duration::from_secs(300),
                temperature: 0.7,
                max_tokens: 2000,
                retry_policy: RetryPolicy::exponential(3),
                memory_config: MemoryConfig::conversational(10),
                security_config: SecurityConfig::standard(),
            }),
            metadata: hashmap!{
                "template".to_string() => json!(self.name()),
                "version".to_string() => json!("1.0"),
            },
        })
    }
}

// Code Generation Agent Template
pub struct CodeAgentTemplate;

impl AgentTemplate for CodeAgentTemplate {
    fn name(&self) -> &str { "code_agent" }
    
    fn default_tools(&self) -> Vec<&'static str> {
        vec![
            "file_operations",
            "file_search",
            "process_executor",
            "git_operations",
            "code_analyzer",
            "test_runner",
            "linter",
        ]
    }
    
    fn build_spec(&self, params: &TemplateParams) -> Result<AgentSpec> {
        // Implementation similar to research agent
        // but with code-specific configuration
        Ok(AgentSpec {
            name: params.name.clone(),
            agent_type: "code".to_string(),
            tools: self.default_tools().iter().map(|s| s.to_string()).collect(),
            config: Some(AgentConfig {
                temperature: 0.2, // Lower temperature for code
                security_config: SecurityConfig::restricted(), // Sandbox file access
                ..Default::default()
            }),
            ..Default::default()
        })
    }
}
```

#### 4.1 Additional Agent Templates

Pre-configured templates for common agent patterns.

```rust
// Chat Agent Template
pub struct ChatAgentTemplate;

impl AgentTemplate for ChatAgentTemplate {
    fn name(&self) -> &str { "chat_agent" }
    
    fn default_tools(&self) -> Vec<&'static str> {
        vec![
            "text_manipulator",
            "template_engine",
            "datetime_handler",
            "calculator",
        ]
    }
    
    fn build_spec(&self, params: &TemplateParams) -> Result<AgentSpec> {
        Ok(AgentSpec {
            name: params.name.clone(),
            agent_type: "chat".to_string(),
            config: Some(AgentConfig {
                temperature: 0.9,
                memory_config: MemoryConfig::conversational(50),
                ..Default::default()
            }),
            ..Default::default()
        })
    }
}

// API Integration Agent Template
pub struct ApiAgentTemplate;

impl AgentTemplate for ApiAgentTemplate {
    fn name(&self) -> &str { "api_agent" }
    
    fn default_tools(&self) -> Vec<&'static str> {
        vec![
            "http_request",
            "graphql_query",
            "json_processor",
            "data_validator",
            "webhook_caller",
            "api_tester",
        ]
    }
}

// Data Processing Agent Template
pub struct DataAgentTemplate;

impl AgentTemplate for DataAgentTemplate {
    fn name(&self) -> &str { "data_agent" }
    
    fn default_tools(&self) -> Vec<&'static str> {
        vec![
            "csv_analyzer",
            "json_processor",
            "database_connector",
            "data_validator",
            "file_converter",
            "archive_handler",
        ]
    }
}
```

### 5. Enhanced ExecutionContext

```rust
// llmspell-agents/src/context.rs
pub struct ExecutionContext {
    pub agent_id: String,
    pub session_id: String,
    pub services: ServiceBundle,
    pub state: Arc<RwLock<ContextState>>,
    pub metrics: Arc<Mutex<ContextMetrics>>,
}

#[derive(Clone)]
pub struct ServiceBundle {
    pub tool_registry: Arc<ToolRegistry>,
    pub llm_provider: Arc<dyn LlmProvider>,
    pub memory_store: Arc<dyn MemoryStore>,
    pub event_bus: Arc<EventBus>,
    pub logger: Arc<Logger>,
    pub cache: Arc<dyn CacheProvider>,
}

impl ExecutionContext {
    pub fn new(agent_id: String, services: ServiceBundle) -> Self {
        Self {
            agent_id: agent_id.clone(),
            session_id: Uuid::new_v4().to_string(),
            services,
            state: Arc::new(RwLock::new(ContextState::new())),
            metrics: Arc::new(Mutex::new(ContextMetrics::new())),
        }
    }
    
    pub async fn with_tool<F, R>(&self, tool_name: &str, f: F) -> Result<R>
    where
        F: FnOnce(&dyn Tool) -> Fut,
        Fut: Future<Output = Result<R>>,
    {
        let tool = self.services.tool_registry.get(tool_name)?;
        self.metrics.lock().await.record_tool_use(tool_name);
        f(tool.as_ref()).await
    }
    
    pub async fn store_memory(&self, key: &str, value: Value) -> Result<()> {
        self.services.memory_store.set(key, value).await
    }
    
    pub async fn recall_memory(&self, key: &str) -> Result<Option<Value>> {
        self.services.memory_store.get(key).await
    }
}
```

### 6. Agent Communication Protocols

```rust
// llmspell-agents/src/communication.rs
pub trait AgentCommunication: Send + Sync {
    async fn send_message(&self, to: &str, message: AgentMessage) -> Result<()>;
    async fn receive_message(&self) -> Result<Option<AgentMessage>>;
    async fn subscribe(&self, topic: &str) -> Result<MessageStream>;
    async fn publish(&self, topic: &str, message: AgentMessage) -> Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentMessage {
    pub id: String,
    pub from: String,
    pub to: Option<String>,
    pub topic: Option<String>,
    pub message_type: MessageType,
    pub payload: Value,
    pub timestamp: DateTime<Utc>,
    pub correlation_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    Request,
    Response,
    Event,
    Command,
    Query,
    Notification,
}

// Direct agent-to-agent communication
pub struct DirectCommunication {
    agents: Arc<RwLock<HashMap<String, MessageQueue>>>,
}

impl DirectCommunication {
    pub async fn send(&self, from: &str, to: &str, message: AgentMessage) -> Result<()> {
        let mut agents = self.agents.write().await;
        let queue = agents.entry(to.to_string())
            .or_insert_with(|| MessageQueue::new(1000));
        queue.push(message)?;
        Ok(())
    }
}

// Event-based communication
pub struct EventBusCommunication {
    bus: Arc<EventBus>,
    subscriptions: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl EventBusCommunication {
    pub async fn publish(&self, topic: &str, message: AgentMessage) -> Result<()> {
        self.bus.publish(topic, message).await
    }
    
    pub async fn subscribe(&self, agent_id: &str, topic: &str) -> Result<MessageStream> {
        let mut subs = self.subscriptions.write().await;
        subs.entry(agent_id.to_string())
            .or_insert_with(Vec::new)
            .push(topic.to_string());
        
        self.bus.subscribe(topic).await
    }
}
```

### 7. Agent Composition Patterns

```rust
// llmspell-agents/src/composition.rs
pub trait AgentComposition {
    fn compose(agents: Vec<Box<dyn Agent>>) -> Result<Box<dyn Agent>>;
}

// Pipeline Composition: Agents process in sequence
pub struct PipelineComposition {
    agents: Vec<Box<dyn Agent>>,
}

impl Agent for PipelineComposition {
    async fn process(&self, input: AgentInput) -> Result<AgentOutput> {
        let mut current = input;
        
        for agent in &self.agents {
            let output = agent.process(current).await?;
            current = AgentInput::from_output(output)?;
        }
        
        Ok(current.into_output())
    }
}

// Ensemble Composition: Multiple agents vote on result
pub struct EnsembleComposition {
    agents: Vec<Box<dyn Agent>>,
    voting_strategy: VotingStrategy,
}

impl Agent for EnsembleComposition {
    async fn process(&self, input: AgentInput) -> Result<AgentOutput> {
        let mut results = Vec::new();
        
        // Process in parallel
        let futures: Vec<_> = self.agents.iter()
            .map(|agent| agent.process(input.clone()))
            .collect();
        
        let outputs = futures::future::join_all(futures).await;
        
        for output in outputs {
            if let Ok(result) = output {
                results.push(result);
            }
        }
        
        // Apply voting strategy
        self.voting_strategy.select(results)
    }
}

// Hierarchical Composition: Parent agent delegates to children
pub struct HierarchicalComposition {
    parent: Box<dyn Agent>,
    children: HashMap<String, Box<dyn Agent>>,
    delegation_strategy: DelegationStrategy,
}

impl Agent for HierarchicalComposition {
    async fn process(&self, input: AgentInput) -> Result<AgentOutput> {
        // Parent analyzes and decides delegation
        let analysis = self.parent.process(input.clone()).await?;
        
        // Determine which child agents to use
        let delegations = self.delegation_strategy.decide(&analysis)?;
        
        // Execute delegated tasks
        let mut results = HashMap::new();
        for (task, agent_name) in delegations {
            if let Some(agent) = self.children.get(&agent_name) {
                let result = agent.process(task).await?;
                results.insert(agent_name, result);
            }
        }
        
        // Parent aggregates results
        self.parent.aggregate(results).await
}
```

### 8. Agent Examples

#### 8.1 Research Agent Configuration

```rust
let research_agent_spec = AgentSpec {
    name: "research_assistant".to_string(),
    agent_type: "research".to_string(),
    description: Some("Agent for conducting web research and analysis".to_string()),
    tools: vec![
        "web_search".to_string(),
        "web_scraper".to_string(),
        "text_summarizer".to_string(),
        "sentiment_analyzer".to_string(),
        "file_writer".to_string(),
        "json_processor".to_string(),
    ],
    llm_provider: Some(LlmProviderSpec {
        provider: "openai".to_string(),
        model: "gpt-4".to_string(),
        config: Some(json!({
            "temperature": 0.7,
            "max_tokens": 2000,
        })),
    }),
    config: Some(AgentConfig {
        max_iterations: 20,
        timeout: Duration::from_secs(300),
        temperature: 0.7,
        max_tokens: 2000,
        retry_policy: RetryPolicy::exponential(3),
        memory_config: MemoryConfig::conversational(10),
        security_config: SecurityConfig::standard(),
    }),
    metadata: hashmap!{
        "capabilities".to_string() => json!(["research", "analysis", "summarization"]),
        "version".to_string() => json!("1.0"),
    },
};

// Create agent using factory
let agent = factory.create_agent(&research_agent_spec).await?;

// Use the agent
let result = agent.process(AgentInput {
    message: "Research the latest developments in quantum computing".to_string(),
    context: Default::default(),
}).await?;
```

#### 8.2 Code Generation Agent with Composition

```rust
// Create a hierarchical code agent with specialized sub-agents
let code_agent = HierarchicalComposition {
    parent: factory.create_agent(&AgentSpec {
        name: "code_orchestrator".to_string(),
        agent_type: "orchestrator".to_string(),
        tools: vec!["code_analyzer".to_string()],
        ..Default::default()
    }).await?,
    
    children: hashmap!{
        "generator".to_string() => factory.create_agent(&AgentSpec {
            name: "code_generator".to_string(),
            agent_type: "code".to_string(),
            tools: vec!["file_operations", "template_engine"],
            config: Some(AgentConfig {
                temperature: 0.2,
                ..Default::default()
            }),
            ..Default::default()
        }).await?,
        
        "tester".to_string() => factory.create_agent(&AgentSpec {
            name: "test_generator".to_string(),
            agent_type: "code".to_string(),
            tools: vec!["test_runner", "file_operations"],
            ..Default::default()
        }).await?,
        
        "documenter".to_string() => factory.create_agent(&AgentSpec {
            name: "doc_generator".to_string(),
            agent_type: "documentation".to_string(),
            tools: vec!["markdown_processor", "template_engine"],
            ..Default::default()
        }).await?,
    },
    
    delegation_strategy: DelegationStrategy::TaskBased,
};
```

#### 8.3 Data Processing Pipeline

```rust
// Create a pipeline of agents for data processing
let data_pipeline = PipelineComposition {
    agents: vec![
        // Stage 1: Data extraction
        factory.create_agent(&AgentSpec {
            name: "data_extractor".to_string(),
            agent_type: "data".to_string(),
            tools: vec!["database_connector", "csv_analyzer"],
            ..Default::default()
        }).await?,
        
        // Stage 2: Data validation
        factory.create_agent(&AgentSpec {
            name: "data_validator".to_string(),
            agent_type: "validation".to_string(),
            tools: vec!["data_validator", "json_processor"],
            ..Default::default()
        }).await?,
        
        // Stage 3: Data transformation
        factory.create_agent(&AgentSpec {
            name: "data_transformer".to_string(),
            agent_type: "transform".to_string(),
            tools: vec!["json_processor", "template_engine"],
            ..Default::default()
        }).await?,
        
        // Stage 4: Data storage
        factory.create_agent(&AgentSpec {
            name: "data_writer".to_string(),
            agent_type: "storage".to_string(),
            tools: vec!["file_writer", "database_connector"],
            ..Default::default()
        }).await?,
    ],
};
```

### 9. Implementation Checklist

**Week 15 Tasks**:
- [ ] Implement Agent Factory pattern with builders
- [ ] Create Agent Registry with persistence
- [ ] Build Discovery Service with providers
- [ ] Implement Agent Lifecycle Management
- [ ] Create Agent State Machine
- [ ] Design Agent Templates (Research, Chat, Code, API, Data)

**Week 16 Tasks**:
- [ ] Implement Enhanced ExecutionContext
- [ ] Build Agent Communication protocols
- [ ] Create Agent Composition patterns
- [ ] Develop example agent configurations
- [ ] Integration testing with all 41+ tools
- [ ] Performance benchmarking

---

## Testing Strategy

### 1. Unit Tests
- Each tool standardization change
- Security vulnerability fixes
- Agent infrastructure components
- State management operations

### 2. Integration Tests
- Tool chain interactions
- Agent-to-agent communication
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
- Agent lifecycle timing
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
- Updated schemas for all 41+ tools
- Usage examples with new interfaces
- Performance characteristics
- Security considerations

### 3. Agent Documentation
- Factory pattern explanations
- Registry API reference
- Template usage examples
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
- **Risk**: 41+ tools with different patterns
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
- Robust agent infrastructure
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
- [ ] All agent infrastructure patterns implemented
- [ ] 41+ tools integrated
- [ ] Example agent configurations functional
- [ ] Full test suite passing

---

This comprehensive design document provides the detailed specifications needed to implement Phase 3's ambitious 8-week plan, transforming the tool library into a standardized, secure, and agent-ready ecosystem.
