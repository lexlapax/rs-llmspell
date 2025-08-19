//! ABOUTME: Tool configuration definitions for llmspell
//! ABOUTME: Manages tool-specific configurations including security settings

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

fn default_true() -> bool {
    true
}

/// Tools configuration - Single source of truth for ALL tool settings
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct ToolsConfig {
    /// Enable tool system globally
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub enabled: Option<bool>,

    /// Global rate limit for tools (requests per minute)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub rate_limit_per_minute: Option<u32>,

    // Core tool categories
    /// File operations tool configuration
    pub file_operations: FileOperationsConfig,
    /// Web search tool configuration  
    pub web_search: WebSearchConfig,
    /// HTTP request tool configuration
    pub http_request: HttpRequestConfig,
    /// Network settings for tools
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub network: Option<NetworkConfig>,

    // Additional tool categories
    /// Web tools configuration (scraping, monitoring, etc.)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub web_tools: Option<WebToolsConfig>,
    /// Media processing tools configuration
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub media: Option<MediaToolsConfig>,
    /// Database tools configuration
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub database: Option<DatabaseToolsConfig>,
    /// Email tools configuration
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub email: Option<EmailToolsConfig>,
    /// System tools configuration
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub system: Option<SystemToolsConfig>,
    /// Data processing tools configuration
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub data: Option<DataToolsConfig>,
    /// Academic tools configuration
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub academic: Option<AcademicToolsConfig>,
    /// Document processing tools configuration
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub document: Option<DocumentToolsConfig>,
    /// Search tools configuration
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub search: Option<SearchToolsConfig>,

    /// Additional tool configurations (for custom/external tools)
    #[serde(flatten)]
    pub custom: HashMap<String, serde_json::Value>,
}

impl ToolsConfig {
    /// Create a new builder for `ToolsConfig`
    #[must_use]
    pub fn builder() -> ToolsConfigBuilder {
        ToolsConfigBuilder::new()
    }

    /// Get a custom tool configuration
    #[must_use]
    pub fn get_custom_config(&self, tool_name: &str) -> Option<&serde_json::Value> {
        self.custom.get(tool_name)
    }

    /// Add a custom tool configuration
    pub fn add_custom_config(&mut self, tool_name: String, config: serde_json::Value) {
        self.custom.insert(tool_name, config);
    }
}

/// Builder for `ToolsConfig`
#[derive(Debug, Clone)]
pub struct ToolsConfigBuilder {
    config: ToolsConfig,
}

impl ToolsConfigBuilder {
    /// Create a new builder with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: ToolsConfig::default(),
        }
    }

    /// Set file operations configuration
    #[must_use]
    pub fn file_operations(mut self, config: FileOperationsConfig) -> Self {
        self.config.file_operations = config;
        self
    }

    /// Set web search configuration
    #[must_use]
    pub fn web_search(mut self, config: WebSearchConfig) -> Self {
        self.config.web_search = config;
        self
    }

    /// Set HTTP request configuration
    #[must_use]
    pub fn http_request(mut self, config: HttpRequestConfig) -> Self {
        self.config.http_request = config;
        self
    }

    /// Add custom tool configuration
    #[must_use]
    pub fn custom_tool(mut self, name: impl Into<String>, config: serde_json::Value) -> Self {
        self.config.custom.insert(name.into(), config);
        self
    }

    /// Build the configuration
    #[must_use]
    pub fn build(self) -> ToolsConfig {
        self.config
    }
}

impl Default for ToolsConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Network configuration for tools
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct NetworkConfig {
    /// Default timeout for network operations in seconds
    pub timeout_seconds: u64,
    /// Maximum retries for network failures
    pub max_retries: u32,
    /// Enable SSL verification
    pub verify_ssl: bool,
}

/// File operations tool configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct FileOperationsConfig {
    /// Whether file operations are enabled
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Allowed file paths for security
    pub allowed_paths: Vec<String>,
    /// Maximum file size in bytes
    pub max_file_size: usize,
    /// Enable atomic write operations
    pub atomic_writes: bool,
    /// Maximum directory depth for traversal
    pub max_depth: Option<usize>,
    /// Allowed file extensions (empty = all allowed)
    pub allowed_extensions: Vec<String>,
    /// Blocked file extensions
    pub blocked_extensions: Vec<String>,
    /// Enable file type validation
    pub validate_file_types: bool,
}

impl Default for FileOperationsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            allowed_paths: vec!["/tmp".to_string()],
            max_file_size: 50_000_000, // 50MB
            atomic_writes: true,
            max_depth: Some(10),
            allowed_extensions: Vec::new(), // Empty = all allowed
            blocked_extensions: vec![
                "exe".to_string(),
                "dll".to_string(),
                "so".to_string(),
                "dylib".to_string(),
            ],
            validate_file_types: true,
        }
    }
}

impl FileOperationsConfig {
    /// Create a new builder for `FileOperationsConfig`
    #[must_use]
    pub fn builder() -> FileOperationsConfigBuilder {
        FileOperationsConfigBuilder::new()
    }

    /// Check if a path is allowed
    #[must_use]
    pub fn is_path_allowed(&self, path: &str) -> bool {
        if self.allowed_paths.contains(&"*".to_string()) {
            return true;
        }

        for allowed_path in &self.allowed_paths {
            if path == *allowed_path || path.starts_with(allowed_path) {
                return true;
            }
        }

        false
    }

    /// Check if a file extension is allowed
    #[must_use]
    pub fn is_extension_allowed(&self, extension: &str) -> bool {
        // If extension is blocked, deny
        if self.blocked_extensions.contains(&extension.to_lowercase()) {
            return false;
        }

        // If allowed_extensions is empty, allow all (except blocked)
        if self.allowed_extensions.is_empty() {
            return true;
        }

        // Check if extension is in allowed list
        self.allowed_extensions.contains(&extension.to_lowercase())
    }
}

/// Builder for `FileOperationsConfig`
#[derive(Debug, Clone)]
pub struct FileOperationsConfigBuilder {
    config: FileOperationsConfig,
}

impl FileOperationsConfigBuilder {
    /// Create a new builder with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: FileOperationsConfig::default(),
        }
    }

    /// Set allowed paths
    #[must_use]
    pub fn allowed_paths(mut self, paths: Vec<String>) -> Self {
        self.config.allowed_paths = paths;
        self
    }

    /// Add an allowed path
    #[must_use]
    pub fn add_allowed_path(mut self, path: impl Into<String>) -> Self {
        self.config.allowed_paths.push(path.into());
        self
    }

    /// Set maximum file size
    #[must_use]
    pub const fn max_file_size(mut self, size: usize) -> Self {
        self.config.max_file_size = size;
        self
    }

    /// Enable or disable atomic writes
    #[must_use]
    pub const fn atomic_writes(mut self, enable: bool) -> Self {
        self.config.atomic_writes = enable;
        self
    }

    /// Set maximum directory depth
    #[must_use]
    pub const fn max_depth(mut self, depth: Option<usize>) -> Self {
        self.config.max_depth = depth;
        self
    }

    /// Set allowed file extensions
    #[must_use]
    pub fn allowed_extensions(mut self, extensions: Vec<String>) -> Self {
        self.config.allowed_extensions = extensions;
        self
    }

    /// Set blocked file extensions
    #[must_use]
    pub fn blocked_extensions(mut self, extensions: Vec<String>) -> Self {
        self.config.blocked_extensions = extensions;
        self
    }

    /// Enable or disable file type validation
    #[must_use]
    pub const fn validate_file_types(mut self, enable: bool) -> Self {
        self.config.validate_file_types = enable;
        self
    }

    /// Build the configuration
    #[must_use]
    pub fn build(self) -> FileOperationsConfig {
        self.config
    }
}

impl Default for FileOperationsConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Web search tool configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WebSearchConfig {
    /// Rate limit requests per minute
    pub rate_limit_per_minute: u32,
    /// Allowed domains (empty = all allowed)
    pub allowed_domains: Vec<String>,
    /// Blocked domains
    pub blocked_domains: Vec<String>,
    /// Maximum results per search
    pub max_results: usize,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
    /// User agent string
    pub user_agent: Option<String>,
}

impl Default for WebSearchConfig {
    fn default() -> Self {
        Self {
            rate_limit_per_minute: 30,
            allowed_domains: vec!["*".to_string()], // Allow all by default
            blocked_domains: Vec::new(),
            max_results: 10,
            timeout_seconds: 30,
            user_agent: Some("llmspell-web-search/1.0".to_string()),
        }
    }
}

impl WebSearchConfig {
    /// Create a new builder for `WebSearchConfig`
    #[must_use]
    pub fn builder() -> WebSearchConfigBuilder {
        WebSearchConfigBuilder::new()
    }

    /// Check if a domain is allowed
    #[must_use]
    pub fn is_domain_allowed(&self, domain: &str) -> bool {
        // Check if domain is blocked
        if self
            .blocked_domains
            .iter()
            .any(|blocked| domain.contains(blocked))
        {
            return false;
        }

        // If allowed_domains contains "*", allow all (except blocked)
        if self.allowed_domains.contains(&"*".to_string()) {
            return true;
        }

        // Check if domain matches any allowed domain
        self.allowed_domains
            .iter()
            .any(|allowed| domain.contains(allowed))
    }
}

/// Builder for `WebSearchConfig`
#[derive(Debug, Clone)]
pub struct WebSearchConfigBuilder {
    config: WebSearchConfig,
}

impl WebSearchConfigBuilder {
    /// Create a new builder with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: WebSearchConfig::default(),
        }
    }

    /// Set rate limit per minute
    #[must_use]
    pub const fn rate_limit_per_minute(mut self, limit: u32) -> Self {
        self.config.rate_limit_per_minute = limit;
        self
    }

    /// Set allowed domains
    #[must_use]
    pub fn allowed_domains(mut self, domains: Vec<String>) -> Self {
        self.config.allowed_domains = domains;
        self
    }

    /// Set blocked domains
    #[must_use]
    pub fn blocked_domains(mut self, domains: Vec<String>) -> Self {
        self.config.blocked_domains = domains;
        self
    }

    /// Set maximum results
    #[must_use]
    pub const fn max_results(mut self, max: usize) -> Self {
        self.config.max_results = max;
        self
    }

    /// Set timeout in seconds
    #[must_use]
    pub const fn timeout_seconds(mut self, timeout: u64) -> Self {
        self.config.timeout_seconds = timeout;
        self
    }

    /// Set user agent
    #[must_use]
    pub fn user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.config.user_agent = Some(user_agent.into());
        self
    }

    /// Build the configuration
    #[must_use]
    pub fn build(self) -> WebSearchConfig {
        self.config
    }
}

impl Default for WebSearchConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// HTTP request tool configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HttpRequestConfig {
    /// Allowed hosts (empty = all allowed)
    pub allowed_hosts: Vec<String>,
    /// Blocked hosts
    pub blocked_hosts: Vec<String>,
    /// Maximum request size in bytes
    pub max_request_size: usize,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
    /// Maximum redirects to follow
    pub max_redirects: u32,
    /// Default headers to include
    pub default_headers: HashMap<String, String>,
}

impl Default for HttpRequestConfig {
    fn default() -> Self {
        let mut default_headers = HashMap::new();
        default_headers.insert("User-Agent".to_string(), "llmspell-http/1.0".to_string());

        Self {
            allowed_hosts: vec!["*".to_string()], // Allow all by default
            blocked_hosts: vec![
                "localhost".to_string(),
                "127.0.0.1".to_string(),
                "0.0.0.0".to_string(),
            ],
            max_request_size: 10_000_000, // 10MB
            timeout_seconds: 30,
            max_redirects: 5,
            default_headers,
        }
    }
}

impl HttpRequestConfig {
    /// Create a new builder for `HttpRequestConfig`
    #[must_use]
    pub fn builder() -> HttpRequestConfigBuilder {
        HttpRequestConfigBuilder::new()
    }

    /// Check if a host is allowed
    #[must_use]
    pub fn is_host_allowed(&self, host: &str) -> bool {
        // Check if host is blocked
        if self
            .blocked_hosts
            .iter()
            .any(|blocked| host.contains(blocked))
        {
            return false;
        }

        // If allowed_hosts contains "*", allow all (except blocked)
        if self.allowed_hosts.contains(&"*".to_string()) {
            return true;
        }

        // Check if host matches any allowed host
        self.allowed_hosts
            .iter()
            .any(|allowed| host.contains(allowed))
    }
}

/// Builder for `HttpRequestConfig`
#[derive(Debug, Clone)]
pub struct HttpRequestConfigBuilder {
    config: HttpRequestConfig,
}

impl HttpRequestConfigBuilder {
    /// Create a new builder with default configuration
    #[must_use]
    pub fn new() -> Self {
        Self {
            config: HttpRequestConfig::default(),
        }
    }

    /// Set allowed hosts
    #[must_use]
    pub fn allowed_hosts(mut self, hosts: Vec<String>) -> Self {
        self.config.allowed_hosts = hosts;
        self
    }

    /// Set blocked hosts
    #[must_use]
    pub fn blocked_hosts(mut self, hosts: Vec<String>) -> Self {
        self.config.blocked_hosts = hosts;
        self
    }

    /// Set maximum request size
    #[must_use]
    pub const fn max_request_size(mut self, size: usize) -> Self {
        self.config.max_request_size = size;
        self
    }

    /// Set timeout in seconds
    #[must_use]
    pub const fn timeout_seconds(mut self, timeout: u64) -> Self {
        self.config.timeout_seconds = timeout;
        self
    }

    /// Set maximum redirects
    #[must_use]
    pub const fn max_redirects(mut self, max: u32) -> Self {
        self.config.max_redirects = max;
        self
    }

    /// Set default headers
    #[must_use]
    pub fn default_headers(mut self, headers: HashMap<String, String>) -> Self {
        self.config.default_headers = headers;
        self
    }

    /// Add a default header
    #[must_use]
    pub fn add_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.config.default_headers.insert(key.into(), value.into());
        self
    }

    /// Build the configuration
    #[must_use]
    pub fn build(self) -> HttpRequestConfig {
        self.config
    }
}

impl Default for HttpRequestConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Web tools configuration (scraping, monitoring, API testing)
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct WebToolsConfig {
    /// User agent string for web requests
    pub user_agent: Option<String>,
    /// Allowed domains (comma-separated or list)
    pub allowed_domains: Option<String>,
    /// Blocked domains (comma-separated or list)
    pub blocked_domains: Option<String>,
    /// Maximum redirects to follow
    pub max_redirects: Option<u32>,
    /// Scraping rate limit
    pub scraping_delay_ms: Option<u64>,
}

/// Media processing tools configuration
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct MediaToolsConfig {
    /// Maximum file size for media processing
    pub max_file_size: Option<usize>,
    /// Processing timeout in seconds
    pub processing_timeout_seconds: Option<u64>,
    /// Maximum image dimensions (e.g., "4096x4096")
    pub image_max_dimensions: Option<String>,
    /// Video processing quality (0-100)
    pub video_quality: Option<u8>,
    /// Audio processing sample rate
    pub audio_sample_rate: Option<u32>,
}

/// Database tools configuration
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct DatabaseToolsConfig {
    /// Connection timeout in seconds
    pub connection_timeout_seconds: Option<u64>,
    /// Maximum concurrent connections
    pub max_connections: Option<u32>,
    /// Allowed database hosts (comma-separated)
    pub allowed_hosts: Option<String>,
    /// Query timeout in seconds
    pub query_timeout_seconds: Option<u64>,
    /// Enable query logging
    pub enable_query_logging: Option<bool>,
}

/// Email tools configuration
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct EmailToolsConfig {
    /// SMTP server host
    pub smtp_host: Option<String>,
    /// SMTP server port
    pub smtp_port: Option<u16>,
    /// SMTP username
    pub smtp_username: Option<String>,
    /// SMTP password (should use env var)
    #[serde(skip_serializing)]
    pub smtp_password: Option<String>,
    /// Default from address
    pub from_address: Option<String>,
    /// Rate limit (emails per minute)
    pub rate_limit_per_minute: Option<u32>,
    /// Enable TLS/SSL
    pub enable_tls: Option<bool>,
}

/// System tools configuration
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SystemToolsConfig {
    /// Allow process execution
    pub allow_process_execution: Option<bool>,
    /// Allowed system commands (comma-separated)
    pub allowed_commands: Option<String>,
    /// Maximum output size in bytes
    pub max_output_size: Option<usize>,
    /// Command timeout in seconds
    pub command_timeout_seconds: Option<u64>,
    /// Environment variables to pass through
    pub allowed_env_vars: Option<String>,
}

/// Data processing tools configuration
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct DataToolsConfig {
    /// Maximum CSV file size
    pub max_csv_size: Option<usize>,
    /// Maximum JSON nesting depth
    pub max_json_depth: Option<u32>,
    /// Maximum XML file size
    pub max_xml_size: Option<usize>,
    /// Enable data validation
    pub enable_validation: Option<bool>,
    /// Processing timeout in seconds
    pub processing_timeout_seconds: Option<u64>,
}

/// Academic tools configuration
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct AcademicToolsConfig {
    /// API key for citation services
    #[serde(skip_serializing)]
    pub citation_api_key: Option<String>,
    /// Maximum references to process
    pub max_references: Option<u32>,
    /// Default citation style (APA, MLA, Chicago, etc.)
    pub default_citation_style: Option<String>,
    /// Enable DOI resolution
    pub enable_doi_resolution: Option<bool>,
}

/// Document processing tools configuration
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct DocumentToolsConfig {
    /// Maximum PDF file size
    pub max_pdf_size: Option<usize>,
    /// Extract images from documents
    pub extract_images: Option<bool>,
    /// OCR enable for scanned documents
    pub enable_ocr: Option<bool>,
    /// Maximum pages to process
    pub max_pages: Option<u32>,
    /// Extract metadata
    pub extract_metadata: Option<bool>,
}

/// Search tools configuration
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct SearchToolsConfig {
    /// Default search engine
    pub default_engine: Option<String>,
    /// Maximum results to return
    pub max_results: Option<u32>,
    /// Search API key
    #[serde(skip_serializing)]
    pub api_key: Option<String>,
    /// Safe search mode
    pub safe_search: Option<bool>,
    /// Search timeout in seconds
    pub timeout_seconds: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_operations_config_default() {
        let config = FileOperationsConfig::default();
        assert_eq!(config.allowed_paths, vec!["/tmp"]);
        assert_eq!(config.max_file_size, 50_000_000);
        assert!(config.atomic_writes);
        assert_eq!(config.max_depth, Some(10));
    }

    #[test]
    fn test_file_operations_config_path_validation() {
        let config = FileOperationsConfig::builder()
            .allowed_paths(vec!["/tmp".to_string(), "/home/user".to_string()])
            .build();

        assert!(config.is_path_allowed("/tmp/test.txt"));
        assert!(config.is_path_allowed("/home/user/document.pdf"));
        assert!(!config.is_path_allowed("/etc/passwd"));

        // Test wildcard
        let config = FileOperationsConfig::builder()
            .allowed_paths(vec!["*".to_string()])
            .build();

        assert!(config.is_path_allowed("/any/path"));
    }

    #[test]
    fn test_file_operations_config_extension_validation() {
        let config = FileOperationsConfig::builder()
            .allowed_extensions(vec!["txt".to_string(), "json".to_string()])
            .blocked_extensions(vec!["exe".to_string()])
            .build();

        assert!(config.is_extension_allowed("txt"));
        assert!(config.is_extension_allowed("json"));
        assert!(!config.is_extension_allowed("exe"));
        assert!(!config.is_extension_allowed("dll")); // Not in allowed list

        // Test default (empty allowed = all allowed except blocked)
        let config = FileOperationsConfig::default();
        assert!(config.is_extension_allowed("txt"));
        assert!(!config.is_extension_allowed("exe")); // Blocked by default
    }

    #[test]
    fn test_web_search_config_domain_validation() {
        let config = WebSearchConfig::builder()
            .allowed_domains(vec!["example.com".to_string(), "github.com".to_string()])
            .blocked_domains(vec!["malicious.com".to_string()])
            .build();

        assert!(config.is_domain_allowed("api.example.com"));
        assert!(config.is_domain_allowed("www.github.com"));
        assert!(!config.is_domain_allowed("malicious.com"));
        assert!(!config.is_domain_allowed("other.com")); // Not in allowed list

        // Test wildcard
        let config = WebSearchConfig::default(); // Uses "*" by default
        assert!(config.is_domain_allowed("any.domain.com"));
    }

    #[test]
    fn test_http_request_config_host_validation() {
        let config = HttpRequestConfig::builder()
            .allowed_hosts(vec!["api.example.com".to_string()])
            .blocked_hosts(vec!["localhost".to_string()])
            .build();

        assert!(config.is_host_allowed("api.example.com"));
        assert!(!config.is_host_allowed("localhost"));

        // Test default blocking
        let config = HttpRequestConfig::default();
        assert!(!config.is_host_allowed("localhost")); // Blocked by default
        assert!(!config.is_host_allowed("127.0.0.1")); // Blocked by default
    }

    #[test]
    fn test_tools_config_builder() {
        let file_config = FileOperationsConfig::builder()
            .add_allowed_path("/custom/path")
            .max_file_size(100_000_000)
            .build();

        let web_config = WebSearchConfig::builder().rate_limit_per_minute(60).build();

        let tools_config = ToolsConfig::builder()
            .file_operations(file_config)
            .web_search(web_config)
            .custom_tool("my_tool", serde_json::json!({"enabled": true}))
            .build();

        assert_eq!(tools_config.file_operations.max_file_size, 100_000_000);
        assert_eq!(tools_config.web_search.rate_limit_per_minute, 60);
        assert!(tools_config.get_custom_config("my_tool").is_some());
    }

    #[test]
    fn test_tools_config_serialization() {
        let config = ToolsConfig::builder()
            .file_operations(
                FileOperationsConfig::builder()
                    .allowed_paths(vec!["/tmp".to_string(), "/home/user".to_string()])
                    .max_file_size(25_000_000)
                    .build(),
            )
            .web_search(WebSearchConfig::builder().rate_limit_per_minute(20).build())
            .build();

        let serialized = serde_json::to_string(&config).expect("Serialization should work");
        let deserialized: ToolsConfig =
            serde_json::from_str(&serialized).expect("Deserialization should work");

        assert_eq!(deserialized.file_operations.max_file_size, 25_000_000);
        assert_eq!(deserialized.web_search.rate_limit_per_minute, 20);
        assert_eq!(deserialized.file_operations.allowed_paths.len(), 2);
    }

    #[test]
    fn test_custom_tool_configuration() {
        let mut config = ToolsConfig::default();

        let custom_config = serde_json::json!({
            "api_key": "test-key",
            "endpoint": "https://api.example.com",
            "rate_limit": 100
        });

        config.add_custom_config("custom_ai_tool".to_string(), custom_config);

        let retrieved = config.get_custom_config("custom_ai_tool");
        assert!(retrieved.is_some());

        let config_obj = retrieved.unwrap().as_object().unwrap();
        assert_eq!(config_obj["api_key"], "test-key");
        assert_eq!(config_obj["endpoint"], "https://api.example.com");
        assert_eq!(config_obj["rate_limit"], 100);
    }
}
