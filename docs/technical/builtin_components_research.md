# Built-in Components Research for Rs-LLMSpell

## Overview

Comprehensive research on built-in tools and components for rs-llmspell, based on analysis of go-llms, LangChain, AutoGPT, and other LLM agent frameworks. This document outlines the essential 30-40 built-in tools that rs-llmspell should provide.

## Table of Contents

1. [Tool Categories and Organization](#tool-categories-and-organization)
2. [Core File System Tools](#core-file-system-tools)
3. [Web and HTTP Tools](#web-and-http-tools)
4. [Data Processing Tools](#data-processing-tools)
5. [Mathematical and Calculation Tools](#mathematical-and-calculation-tools)
6. [Text Processing Tools](#text-processing-tools)
7. [System and Utility Tools](#system-and-utility-tools)
8. [AI and Content Tools](#ai-and-content-tools)
9. [Tool Registry and Management](#tool-registry-and-management)
10. [Built-in Agent Patterns](#built-in-agent-patterns)
11. [Implementation Recommendations](#implementation-recommendations)

## Tool Categories and Organization

### Go-LLMs Inspired Structure

Based on go-llms analysis, tools should be organized into clear categories:

```
rs-llmspell-tools/
├── core/           # Essential utilities
├── file/           # File system operations
├── web/            # Web and HTTP operations
├── data/           # Data processing and manipulation
├── math/           # Mathematical operations
├── text/           # Text processing
├── system/         # System utilities
├── ai/             # AI-specific tools
└── registry.rs     # Tool registration and discovery
```

### Tool Registration Pattern

```rust
// Based on go-llms registry patterns
#[derive(Debug, Clone)]
pub struct ToolMetadata {
    pub name: String,
    pub version: String,
    pub category: String,
    pub description: String,
    pub usage_instructions: String,
    pub examples: Vec<String>,
    pub parameters_schema: serde_json::Value,
    pub output_schema: serde_json::Value,
    pub required_permissions: Vec<String>,
    pub resource_usage: ResourceUsage,
    pub execution_time_estimate: Option<Duration>,
    pub is_deterministic: bool,
    pub is_destructive: bool,
    pub requires_confirmation: bool,
    pub tags: Vec<String>,
    pub constraints: Vec<String>,
    pub error_guidance: String,
}

#[derive(Debug, Clone)]
pub enum ResourceUsage {
    Minimal,    // < 1MB memory, < 100ms CPU
    Low,        // < 10MB memory, < 1s CPU
    Medium,     // < 100MB memory, < 10s CPU
    High,       // < 1GB memory, < 60s CPU
    Intensive,  // > 1GB memory or > 60s CPU
}

pub trait ToolRegistry: Send + Sync {
    fn register_tool(&mut self, tool: Arc<dyn Tool>) -> Result<(), RegistryError>;
    fn get_tool(&self, name: &str) -> Option<Arc<dyn Tool>>;
    fn list_tools(&self) -> Vec<String>;
    fn list_by_category(&self, category: &str) -> Vec<String>;
    fn list_by_permission(&self, permission: &str) -> Vec<String>;
    fn list_by_resource_usage(&self, usage: ResourceUsage) -> Vec<String>;
    fn export_metadata(&self) -> Vec<ToolMetadata>;
    fn validate_tool_metadata(&self, metadata: &ToolMetadata) -> Result<(), ValidationError>;
}
```

## Core File System Tools

### Essential File Operations (6 tools)

**1. FileReadTool**
```rust
pub struct FileReadTool {
    max_file_size: usize, // Default: 10MB
    allowed_extensions: Option<Vec<String>>,
    restricted_paths: Vec<PathBuf>,
}

#[derive(Debug, serde::Deserialize)]
pub struct FileReadParams {
    pub path: String,
    pub start_line: Option<usize>,
    pub end_line: Option<usize>,
    pub encoding: Option<String>, // utf-8, ascii, etc.
}

#[derive(Debug, serde::Serialize)]
pub struct FileReadResult {
    pub content: String,
    pub metadata: FileMetadata,
    pub encoding_detected: String,
    pub line_count: usize,
    pub size_bytes: u64,
}

pub struct FileMetadata {
    pub absolute_path: PathBuf,
    pub file_size: u64,
    pub permissions: u32,
    pub modified_time: DateTime<Utc>,
    pub created_time: DateTime<Utc>,
    pub file_extension: Option<String>,
    pub is_binary: bool,
}
```

**2. FileWriteTool**
```rust
#[derive(Debug, serde::Deserialize)]
pub struct FileWriteParams {
    pub path: String,
    pub content: String,
    pub mode: WriteMode,
    pub encoding: Option<String>,
    pub create_dirs: bool,
    pub backup_existing: bool,
}

#[derive(Debug, serde::Deserialize)]
pub enum WriteMode {
    Overwrite,
    Append,
    CreateNew, // Fail if exists
    InsertAt { line: usize },
}
```

**3. FileListTool**
```rust
#[derive(Debug, serde::Deserialize)]
pub struct FileListParams {
    pub path: String,
    pub recursive: bool,
    pub include_hidden: bool,
    pub pattern: Option<String>, // glob pattern
    pub file_types: Option<Vec<FileType>>,
    pub max_depth: Option<usize>,
    pub max_files: Option<usize>,
}

#[derive(Debug, serde::Deserialize)]
pub enum FileType {
    File,
    Directory,
    Symlink,
}
```

**4. FileMoveTool**
```rust
#[derive(Debug, serde::Deserialize)]
pub struct FileMoveParams {
    pub source: String,
    pub destination: String,
    pub overwrite: bool,
    pub create_dirs: bool,
}
```

**5. FileDeleteTool**
```rust
#[derive(Debug, serde::Deserialize)]
pub struct FileDeleteParams {
    pub path: String,
    pub recursive: bool,
    pub force: bool, // Skip confirmation for destructive operations
}
```

**6. FileSearchTool**
```rust
#[derive(Debug, serde::Deserialize)]
pub struct FileSearchParams {
    pub path: String,
    pub pattern: String,
    pub search_type: SearchType,
    pub case_sensitive: bool,
    pub max_results: Option<usize>,
    pub include_binary: bool,
}

#[derive(Debug, serde::Deserialize)]
pub enum SearchType {
    Content,     // Search file contents
    Filename,    // Search filenames
    Both,
}
```

## Web and HTTP Tools

### Essential Web Operations (8 tools)

**1. HttpRequestTool**
```rust
#[derive(Debug, serde::Deserialize)]
pub struct HttpRequestParams {
    pub url: String,
    pub method: HttpMethod,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<String>,
    pub timeout: Option<u64>, // seconds
    pub follow_redirects: bool,
    pub max_redirects: usize,
    pub verify_ssl: bool,
}

#[derive(Debug, serde::Deserialize)]
pub enum HttpMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    HEAD,
    OPTIONS,
}

#[derive(Debug, serde::Serialize)]
pub struct HttpResponse {
    pub status_code: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
    pub content_type: Option<String>,
    pub response_time_ms: u64,
    pub final_url: String, // After redirects
}
```

**2. WebScrapeTool**
```rust
#[derive(Debug, serde::Deserialize)]
pub struct WebScrapeParams {
    pub url: String,
    pub selectors: Option<Vec<CssSelector>>,
    pub extract_links: bool,
    pub extract_images: bool,
    pub extract_text_only: bool,
    pub follow_links: bool,
    pub max_depth: usize,
    pub user_agent: Option<String>,
    pub wait_time: Option<u64>, // For dynamic content
}

#[derive(Debug, serde::Deserialize)]
pub struct CssSelector {
    pub selector: String,
    pub attribute: Option<String>, // Extract specific attribute
    pub multiple: bool, // Multiple matches vs first match
}

#[derive(Debug, serde::Serialize)]
pub struct WebScrapeResult {
    pub content: String,
    pub extracted_data: HashMap<String, serde_json::Value>,
    pub links: Vec<String>,
    pub images: Vec<String>,
    pub metadata: WebPageMetadata,
}
```

**3. WebSearchTool**
```rust
#[derive(Debug, serde::Deserialize)]
pub struct WebSearchParams {
    pub query: String,
    pub engine: SearchEngine,
    pub num_results: usize,
    pub language: Option<String>,
    pub region: Option<String>,
    pub time_filter: Option<TimeFilter>,
}

#[derive(Debug, serde::Deserialize)]
pub enum SearchEngine {
    DuckDuckGo,
    Brave,
    Bing, // Requires API key
    Google, // Requires API key
}

#[derive(Debug, serde::Serialize)]
pub struct SearchResult {
    pub title: String,
    pub url: String,
    pub snippet: String,
    pub rank: usize,
    pub domain: String,
}
```

**4. UrlParseTool**
```rust
#[derive(Debug, serde::Deserialize)]
pub struct UrlParseParams {
    pub url: String,
    pub operation: UrlOperation,
}

#[derive(Debug, serde::Deserialize)]
pub enum UrlOperation {
    Parse,
    Join { base: String },
    Encode,
    Decode,
    ExtractDomain,
    ExtractParams,
}
```

**5. ApiClientTool**
```rust
#[derive(Debug, serde::Deserialize)]
pub struct ApiClientParams {
    pub base_url: String,
    pub endpoint: String,
    pub method: HttpMethod,
    pub auth: Option<AuthConfig>,
    pub parameters: Option<serde_json::Value>,
    pub rate_limit: Option<RateLimit>,
}

#[derive(Debug, serde::Deserialize)]
pub enum AuthConfig {
    Bearer { token: String },
    ApiKey { key: String, header: String },
    Basic { username: String, password: String },
    OAuth2 { token: String },
}
```

**6. OpenApiTool** (Based on go-llms patterns)
```rust
#[derive(Debug, serde::Deserialize)]
pub struct OpenApiParams {
    pub spec_url: String,
    pub operation_id: String,
    pub parameters: serde_json::Value,
    pub auth: Option<AuthConfig>,
}
```

**7. GraphQLTool** (Based on go-llms patterns)
```rust
#[derive(Debug, serde::Deserialize)]
pub struct GraphQLParams {
    pub endpoint: String,
    pub query: String,
    pub variables: Option<serde_json::Value>,
    pub operation_name: Option<String>,
    pub auth: Option<AuthConfig>,
}
```

**8. WebhookTool**
```rust
#[derive(Debug, serde::Deserialize)]
pub struct WebhookParams {
    pub action: WebhookAction,
    pub url: Option<String>,
    pub payload: Option<serde_json::Value>,
}

#[derive(Debug, serde::Deserialize)]
pub enum WebhookAction {
    Create { port: u16, path: String },
    Send { url: String },
    Listen { timeout: u64 },
}
```

## Data Processing Tools

### Essential Data Operations (7 tools)

**1. JsonTool**
```rust
#[derive(Debug, serde::Deserialize)]
pub struct JsonParams {
    pub operation: JsonOperation,
    pub input: serde_json::Value,
    pub path: Option<String>, // JSONPath
}

#[derive(Debug, serde::Deserialize)]
pub enum JsonOperation {
    Parse { input: String },
    Stringify { pretty: bool },
    Query { path: String },
    Merge { other: serde_json::Value },
    Transform { jq_filter: String },
    Validate { schema: serde_json::Value },
    Diff { other: serde_json::Value },
}
```

**2. CsvReaderTool**
```rust
#[derive(Debug, serde::Deserialize)]
pub struct CsvReadParams {
    pub source: CsvSource,
    pub delimiter: Option<char>,
    pub has_headers: bool,
    pub max_rows: Option<usize>,
    pub columns: Option<Vec<String>>, // Select specific columns
}

#[derive(Debug, serde::Deserialize)]
pub enum CsvSource {
    File { path: String },
    Content { data: String },
    Url { url: String },
}
```

**3. CsvWriterTool**
```rust
#[derive(Debug, serde::Deserialize)]
pub struct CsvWriteParams {
    pub data: Vec<serde_json::Value>,
    pub output: CsvOutput,
    pub delimiter: Option<char>,
    pub include_headers: bool,
}

#[derive(Debug, serde::Deserialize)]
pub enum CsvOutput {
    File { path: String },
    String,
}
```

**4. DataFilterTool**
```rust
#[derive(Debug, serde::Deserialize)]
pub struct DataFilterParams {
    pub data: serde_json::Value,
    pub filters: Vec<FilterCondition>,
    pub operation: FilterOperation,
}

#[derive(Debug, serde::Deserialize)]
pub struct FilterCondition {
    pub field: String,
    pub operator: FilterOperator,
    pub value: serde_json::Value,
}

#[derive(Debug, serde::Deserialize)]
pub enum FilterOperator {
    Equals,
    NotEquals,
    GreaterThan,
    LessThan,
    Contains,
    StartsWith,
    EndsWith,
    Regex,
    In,
    NotIn,
}

#[derive(Debug, serde::Deserialize)]
pub enum FilterOperation {
    And,
    Or,
    Not,
}
```

**5. DataSortTool**
```rust
#[derive(Debug, serde::Deserialize)]
pub struct DataSortParams {
    pub data: serde_json::Value,
    pub sort_by: Vec<SortCriteria>,
}

#[derive(Debug, serde::Deserialize)]
pub struct SortCriteria {
    pub field: String,
    pub direction: SortDirection,
    pub data_type: Option<DataType>,
}

#[derive(Debug, serde::Deserialize)]
pub enum SortDirection {
    Ascending,
    Descending,
}
```

**6. DataGroupTool**
```rust
#[derive(Debug, serde::Deserialize)]
pub struct DataGroupParams {
    pub data: serde_json::Value,
    pub group_by: Vec<String>,
    pub aggregations: Vec<Aggregation>,
}

#[derive(Debug, serde::Deserialize)]
pub struct Aggregation {
    pub field: String,
    pub operation: AggregationOperation,
    pub alias: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub enum AggregationOperation {
    Count,
    Sum,
    Average,
    Min,
    Max,
    First,
    Last,
    Distinct,
}
```

**7. SqlQueryTool**
```rust
#[derive(Debug, serde::Deserialize)]
pub struct SqlQueryParams {
    pub connection: DatabaseConnection,
    pub query: String,
    pub parameters: Option<Vec<serde_json::Value>>,
    pub max_rows: Option<usize>,
    pub timeout: Option<u64>,
}

#[derive(Debug, serde::Deserialize)]
pub struct DatabaseConnection {
    pub database_type: DatabaseType,
    pub connection_string: String,
    pub pool_size: Option<usize>,
}

#[derive(Debug, serde::Deserialize)]
pub enum DatabaseType {
    SQLite,
    PostgreSQL,
    MySQL,
    SqlServer,
}
```

## Mathematical and Calculation Tools

### Essential Math Operations (4 tools)

**1. CalculatorTool** (Based on go-llms implementation)
```rust
#[derive(Debug, serde::Deserialize)]
pub struct CalculatorParams {
    pub operation: MathOperation,
    pub operands: Vec<f64>,
    pub precision: Option<usize>,
}

#[derive(Debug, serde::Deserialize)]
pub enum MathOperation {
    // Basic arithmetic
    Add,
    Subtract,
    Multiply,
    Divide,
    Power,
    Modulo,
    
    // Advanced operations
    Sqrt,
    Log,
    Log10,
    Ln,
    
    // Trigonometric
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    
    // Other
    Abs,
    Ceil,
    Floor,
    Round,
    Factorial,
    Gcd,
    Lcm,
    
    // Complex expressions
    Evaluate { expression: String },
}
```

**2. StatsTool**
```rust
#[derive(Debug, serde::Deserialize)]
pub struct StatsParams {
    pub data: Vec<f64>,
    pub operations: Vec<StatOperation>,
}

#[derive(Debug, serde::Deserialize)]
pub enum StatOperation {
    Mean,
    Median,
    Mode,
    StandardDeviation,
    Variance,
    Min,
    Max,
    Range,
    Percentile { percentile: f64 },
    Correlation { other: Vec<f64> },
}
```

**3. UnitConverterTool**
```rust
#[derive(Debug, serde::Deserialize)]
pub struct UnitConvertParams {
    pub value: f64,
    pub from_unit: String,
    pub to_unit: String,
    pub category: UnitCategory,
}

#[derive(Debug, serde::Deserialize)]
pub enum UnitCategory {
    Length,
    Weight,
    Temperature,
    Area,
    Volume,
    Speed,
    Energy,
    Power,
    Time,
    Currency,
}
```

**4. RandomTool**
```rust
#[derive(Debug, serde::Deserialize)]
pub struct RandomParams {
    pub operation: RandomOperation,
    pub seed: Option<u64>,
}

#[derive(Debug, serde::Deserialize)]
pub enum RandomOperation {
    Integer { min: i64, max: i64 },
    Float { min: f64, max: f64 },
    Boolean,
    Choice { options: Vec<serde_json::Value> },
    Shuffle { list: Vec<serde_json::Value> },
    UUID,
    Password { length: usize, include_symbols: bool },
}
```

## Text Processing Tools

### Essential Text Operations (6 tools)

**1. TextSearchTool**
```rust
#[derive(Debug, serde::Deserialize)]
pub struct TextSearchParams {
    pub text: String,
    pub pattern: String,
    pub search_type: TextSearchType,
    pub case_sensitive: bool,
    pub max_matches: Option<usize>,
}

#[derive(Debug, serde::Deserialize)]
pub enum TextSearchType {
    Literal,
    Regex,
    Fuzzy { threshold: f64 },
    Wildcard,
}
```

**2. TextReplaceTool**
```rust
#[derive(Debug, serde::Deserialize)]
pub struct TextReplaceParams {
    pub text: String,
    pub pattern: String,
    pub replacement: String,
    pub replace_type: ReplaceType,
    pub case_sensitive: bool,
    pub max_replacements: Option<usize>,
}

#[derive(Debug, serde::Deserialize)]
pub enum ReplaceType {
    All,
    First,
    Last,
    Regex,
}
```

**3. TextTransformTool**
```rust
#[derive(Debug, serde::Deserialize)]
pub struct TextTransformParams {
    pub text: String,
    pub transformations: Vec<TextTransformation>,
}

#[derive(Debug, serde::Deserialize)]
pub enum TextTransformation {
    ToUpperCase,
    ToLowerCase,
    ToTitleCase,
    TrimWhitespace,
    RemoveWhitespace,
    NormalizeWhitespace,
    RemoveHtml,
    RemoveMarkdown,
    ExtractEmails,
    ExtractUrls,
    ExtractPhoneNumbers,
    WordCount,
    CharacterCount,
    LineCount,
}
```

**4. TemplateTool**
```rust
#[derive(Debug, serde::Deserialize)]
pub struct TemplateParams {
    pub template: String,
    pub variables: serde_json::Value,
    pub engine: TemplateEngine,
}

#[derive(Debug, serde::Deserialize)]
pub enum TemplateEngine {
    Handlebars,
    Jinja2,
    Mustache,
    Simple, // {{variable}} replacement
}
```

**5. TextAnalysisTool**
```rust
#[derive(Debug, serde::Deserialize)]
pub struct TextAnalysisParams {
    pub text: String,
    pub analyses: Vec<TextAnalysisType>,
}

#[derive(Debug, serde::Deserialize)]
pub enum TextAnalysisType {
    Sentiment,
    LanguageDetection,
    ReadabilityScore,
    KeywordExtraction { max_keywords: usize },
    EntityExtraction,
    WordFrequency,
    TextComplexity,
}
```

**6. EncodingTool**
```rust
#[derive(Debug, serde::Deserialize)]
pub struct EncodingParams {
    pub input: String,
    pub operation: EncodingOperation,
}

#[derive(Debug, serde::Deserialize)]
pub enum EncodingOperation {
    Base64Encode,
    Base64Decode,
    UrlEncode,
    UrlDecode,
    HtmlEncode,
    HtmlDecode,
    Utf8Encode,
    Utf8Decode,
    HashMd5,
    HashSha1,
    HashSha256,
}
```

## System and Utility Tools

### Essential System Operations (5 tools)

**1. DateTimeTool**
```rust
#[derive(Debug, serde::Deserialize)]
pub struct DateTimeParams {
    pub operation: DateTimeOperation,
    pub input: Option<String>,
    pub format: Option<String>,
    pub timezone: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub enum DateTimeOperation {
    Now,
    Parse { input: String },
    Format { datetime: String, format: String },
    Add { datetime: String, duration: String },
    Subtract { datetime: String, duration: String },
    Diff { start: String, end: String },
    ToTimestamp,
    FromTimestamp { timestamp: i64 },
    ToUtc,
    ToTimezone { timezone: String },
}
```

**2. UuidTool**
```rust
#[derive(Debug, serde::Deserialize)]
pub struct UuidParams {
    pub version: UuidVersion,
    pub count: Option<usize>,
}

#[derive(Debug, serde::Deserialize)]
pub enum UuidVersion {
    V4, // Random
    V1, // Timestamp
    V5 { namespace: String, name: String },
}
```

**3. ValidationTool**
```rust
#[derive(Debug, serde::Deserialize)]
pub struct ValidationParams {
    pub input: String,
    pub validation_type: ValidationType,
}

#[derive(Debug, serde::Deserialize)]
pub enum ValidationType {
    Email,
    Url,
    Phone,
    CreditCard,
    Ip,
    Uuid,
    Json,
    Xml,
    Regex { pattern: String },
    Custom { schema: serde_json::Value },
}
```

**4. CompressionTool**
```rust
#[derive(Debug, serde::Deserialize)]
pub struct CompressionParams {
    pub operation: CompressionOperation,
    pub algorithm: CompressionAlgorithm,
}

#[derive(Debug, serde::Deserialize)]
pub enum CompressionOperation {
    Compress { input: String },
    Decompress { input: String },
    CompressFile { path: String },
    DecompressFile { path: String },
}

#[derive(Debug, serde::Deserialize)]
pub enum CompressionAlgorithm {
    Gzip,
    Zlib,
    Brotli,
    Lz4,
}
```

**5. NetworkTool**
```rust
#[derive(Debug, serde::Deserialize)]
pub struct NetworkParams {
    pub operation: NetworkOperation,
    pub target: String,
    pub timeout: Option<u64>,
}

#[derive(Debug, serde::Deserialize)]
pub enum NetworkOperation {
    Ping,
    Traceroute,
    DnsLookup,
    PortScan { ports: Vec<u16> },
    Whois,
    GetPublicIp,
}
```

## AI and Content Tools

### Essential AI Operations (4 tools)

**1. SummarizerTool**
```rust
#[derive(Debug, serde::Deserialize)]
pub struct SummarizerParams {
    pub text: String,
    pub max_length: Option<usize>,
    pub summary_type: SummaryType,
    pub preserve_structure: bool,
}

#[derive(Debug, serde::Deserialize)]
pub enum SummaryType {
    Extractive, // Select key sentences
    Abstractive, // Generate new summary
    Bullet { max_points: usize },
    Keywords { max_keywords: usize },
}
```

**2. TranslatorTool**
```rust
#[derive(Debug, serde::Deserialize)]
pub struct TranslatorParams {
    pub text: String,
    pub target_language: String,
    pub source_language: Option<String>, // Auto-detect if None
    pub preserve_formatting: bool,
}
```

**3. ContentGeneratorTool**
```rust
#[derive(Debug, serde::Deserialize)]
pub struct ContentGeneratorParams {
    pub content_type: ContentType,
    pub parameters: serde_json::Value,
    pub style: Option<String>,
    pub tone: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub enum ContentType {
    Email { subject: String, recipient: String },
    BlogPost { title: String, keywords: Vec<String> },
    SocialMedia { platform: String, hashtags: Vec<String> },
    Documentation { topic: String, audience: String },
    Code { language: String, description: String },
}
```

**4. ImageProcessingTool**
```rust
#[derive(Debug, serde::Deserialize)]
pub struct ImageProcessingParams {
    pub source: ImageSource,
    pub operations: Vec<ImageOperation>,
    pub output_format: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub enum ImageSource {
    File { path: String },
    Url { url: String },
    Base64 { data: String },
}

#[derive(Debug, serde::Deserialize)]
pub enum ImageOperation {
    Resize { width: u32, height: u32 },
    Crop { x: u32, y: u32, width: u32, height: u32 },
    Rotate { degrees: f32 },
    GetMetadata,
    ExtractText, // OCR
    Describe, // AI description
}
```

## Tool Registry and Management

### Registry Implementation

```rust
// Enhanced registry based on go-llms patterns
pub struct BuiltinToolRegistry {
    tools: Arc<RwLock<HashMap<String, Arc<dyn Tool>>>>,
    metadata: Arc<RwLock<HashMap<String, ToolMetadata>>>,
    categories: Arc<RwLock<HashMap<String, Vec<String>>>>,
    permissions: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl BuiltinToolRegistry {
    pub fn new() -> Self {
        let registry = Self {
            tools: Arc::new(RwLock::new(HashMap::new())),
            metadata: Arc::new(RwLock::new(HashMap::new())),
            categories: Arc::new(RwLock::new(HashMap::new())),
            permissions: Arc::new(RwLock::new(HashMap::new())),
        };
        
        registry.register_all_builtin_tools();
        registry
    }
    
    fn register_all_builtin_tools(&self) {
        // File tools
        self.register_tool_with_metadata(
            Arc::new(FileReadTool::new()),
            ToolMetadata {
                name: "file_read".to_string(),
                category: "file".to_string(),
                description: "Read file contents with encoding detection".to_string(),
                required_permissions: vec!["file_read".to_string()],
                resource_usage: ResourceUsage::Low,
                is_destructive: false,
                // ... other metadata
            }
        );
        
        // Web tools
        self.register_tool_with_metadata(
            Arc::new(HttpRequestTool::new()),
            ToolMetadata {
                name: "http_request".to_string(),
                category: "web".to_string(),
                description: "Make HTTP requests to web APIs".to_string(),
                required_permissions: vec!["network_access".to_string()],
                resource_usage: ResourceUsage::Medium,
                is_destructive: false,
                // ... other metadata
            }
        );
        
        // Math tools
        self.register_tool_with_metadata(
            Arc::new(CalculatorTool::new()),
            ToolMetadata {
                name: "calculator".to_string(),
                category: "math".to_string(),
                description: "Perform mathematical calculations".to_string(),
                required_permissions: vec![],
                resource_usage: ResourceUsage::Minimal,
                is_destructive: false,
                // ... other metadata
            }
        );
        
        // Continue for all 40+ tools...
    }
}
```

### Tool Discovery and Filtering

```rust
impl ToolRegistry for BuiltinToolRegistry {
    fn list_by_category(&self, category: &str) -> Vec<String> {
        let categories = self.categories.read().unwrap();
        categories.get(category).cloned().unwrap_or_default()
    }
    
    fn list_by_permission(&self, permission: &str) -> Vec<String> {
        let permissions = self.permissions.read().unwrap();
        permissions.get(permission).cloned().unwrap_or_default()
    }
    
    fn list_by_resource_usage(&self, usage: ResourceUsage) -> Vec<String> {
        let metadata = self.metadata.read().unwrap();
        metadata.iter()
            .filter(|(_, meta)| meta.resource_usage == usage)
            .map(|(name, _)| name.clone())
            .collect()
    }
    
    fn list_safe_tools(&self) -> Vec<String> {
        let metadata = self.metadata.read().unwrap();
        metadata.iter()
            .filter(|(_, meta)| !meta.is_destructive && !meta.requires_confirmation)
            .map(|(name, _)| name.clone())
            .collect()
    }
}
```

## Built-in Agent Patterns

### Common Agent Templates (6 agents)

**1. ResearchAgent**
```rust
pub struct ResearchAgentTemplate {
    pub tools: Vec<String>, // web_search, file_write, summarizer
    pub system_prompt: String,
    pub model_config: ModelConfig,
}
```

**2. DataAnalystAgent**
```rust
pub struct DataAnalystAgentTemplate {
    pub tools: Vec<String>, // csv_reader, data_filter, data_sort, stats, calculator
    pub system_prompt: String,
    pub model_config: ModelConfig,
}
```

**3. ContentCreatorAgent**
```rust
pub struct ContentCreatorAgentTemplate {
    pub tools: Vec<String>, // template, text_transform, translator, content_generator
    pub system_prompt: String,
    pub model_config: ModelConfig,
}
```

**4. FileManagerAgent**
```rust
pub struct FileManagerAgentTemplate {
    pub tools: Vec<String>, // file_read, file_write, file_list, file_search
    pub system_prompt: String,
    pub model_config: ModelConfig,
}
```

**5. WebAutomationAgent**
```rust
pub struct WebAutomationAgentTemplate {
    pub tools: Vec<String>, // http_request, web_scrape, api_client
    pub system_prompt: String,
    pub model_config: ModelConfig,
}
```

**6. GeneralAssistantAgent**
```rust
pub struct GeneralAssistantAgentTemplate {
    pub tools: Vec<String>, // calculator, datetime, uuid, text_search, validation
    pub system_prompt: String,
    pub model_config: ModelConfig,
}
```

## Implementation Recommendations

### Priority Implementation Order

**Phase 1: Core Foundation (12 tools)**
1. file_read
2. file_write
3. file_list
4. http_request
5. calculator
6. json
7. text_search
8. text_replace
9. datetime
10. uuid
11. validation
12. template

**Phase 2: Data and Web (15 tools)**
13. web_scrape
14. web_search
15. csv_reader
16. csv_writer
17. data_filter
18. data_sort
19. data_group
20. url_parse
21. api_client
22. file_move
23. file_delete
24. file_search
25. text_transform
26. encoding
27. stats

**Phase 3: Advanced and AI (15 tools)**
28. sql_query
29. openapi
30. graphql
31. webhook
32. unit_converter
33. random
34. text_analysis
35. compression
36. network
37. summarizer
38. translator
39. content_generator
40. image_processing

### Quality Standards

**All tools must include:**
1. Comprehensive input validation
2. Detailed error messages with context
3. Resource usage tracking
4. Permission checking
5. Async execution support
6. Extensive test coverage
7. Clear documentation and examples
8. JSON schema for parameters and outputs

**Performance Requirements:**
- Minimal tools: < 1MB memory, < 100ms execution
- Low resource tools: < 10MB memory, < 1s execution
- Medium resource tools: < 100MB memory, < 10s execution
- Graceful degradation and cancellation support

**Security Requirements:**
- Input sanitization and validation
- Path traversal protection (file tools)
- URL validation (web tools)
- Resource limits and timeouts
- Permission-based access control
- Safe execution environment

This comprehensive built-in tool library provides rs-llmspell with a solid foundation for LLM agent development while maintaining compatibility with existing agent frameworks and following established patterns from go-llms.