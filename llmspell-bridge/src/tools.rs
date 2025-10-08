//! ABOUTME: Tool registration and management for the bridge
//! ABOUTME: Initializes and provides access to all Phase 2 tools from llmspell-tools

use crate::discovery::BridgeDiscovery;
use crate::ComponentRegistry;
use llmspell_config::tools::ToolsConfig;
use llmspell_core::traits::tool::{
    ResourceLimits, SecurityLevel, SecurityRequirements, ToolCategory, ToolSchema,
};
use llmspell_core::Tool;
use llmspell_security::sandbox::{file_sandbox::FileSandbox, SandboxContext};
// Import tools conditionally based on features
use llmspell_tools::{
    ApiTesterTool, AudioProcessorTool, Base64EncoderTool, CalculatorTool, CitationFormatterTool,
    DataValidationTool, DateTimeHandlerTool, DiffCalculatorTool, EnvironmentReaderTool,
    FileConverterTool, FileOperationsTool, FileSearchTool, FileWatcherTool, GraphBuilderTool,
    GraphQLQueryTool, HashCalculatorTool, HttpRequestTool, ImageProcessorTool, ProcessExecutorTool,
    ServiceCheckerTool, SitemapCrawlerTool, SystemMonitorTool, TextManipulatorTool,
    UrlAnalyzerTool, UuidGeneratorTool, VideoProcessorTool, WebScraperTool, WebSearchTool,
    WebhookCallerTool, WebpageMonitorTool,
};

#[cfg(feature = "archives")]
use llmspell_tools::ArchiveHandlerTool;
#[cfg(feature = "csv-parquet")]
use llmspell_tools::CsvAnalyzerTool;
#[cfg(feature = "database")]
use llmspell_tools::DatabaseConnectorTool;
#[cfg(feature = "email")]
use llmspell_tools::EmailSenderTool;
#[cfg(feature = "json-query")]
use llmspell_tools::JsonProcessorTool;
#[cfg(feature = "pdf")]
use llmspell_tools::PdfProcessorTool;
#[cfg(feature = "templates")]
use llmspell_tools::TemplateEngineTool;

// Import Config types from submodules
use llmspell_tools::api::graphql_query::GraphQLConfig;
use llmspell_tools::api::http_request::{HttpRequestConfig, RetryConfig};
#[cfg(feature = "database")]
use llmspell_tools::communication::database_connector::DatabaseConnectorConfig;
#[cfg(feature = "email")]
use llmspell_tools::communication::email_sender::EmailSenderConfig;
#[cfg(feature = "csv-parquet")]
use llmspell_tools::data::csv_analyzer::CsvAnalyzerConfig;
#[cfg(feature = "json-query")]
use llmspell_tools::data::json_processor::JsonProcessorConfig;
use llmspell_tools::fs::{
    FileConverterConfig, FileOperationsConfig, FileSearchConfig, FileWatcherConfig,
};
use llmspell_tools::media::{AudioProcessorConfig, ImageProcessorConfig, VideoProcessorConfig};
use llmspell_tools::search::WebSearchConfig;
use llmspell_tools::system::{
    EnvironmentReaderConfig, ProcessExecutorConfig, ServiceCheckerConfig, SystemMonitorConfig,
};
use llmspell_tools::util::{HashCalculatorConfig, TextManipulatorConfig, UuidGeneratorConfig};
use llmspell_tools::web::web_scraper::WebScraperConfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Initialize and register all Phase 2 tools with the bridge registry
///
/// # Errors
///
/// Returns an error if tool registration fails
pub fn register_all_tools(
    registry: &Arc<ComponentRegistry>,
    tools_config: &ToolsConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create a shared file sandbox for file system tools using configured allowed paths
    let mut security_requirements = SecurityRequirements::default();
    for path in &tools_config.file_operations.allowed_paths {
        security_requirements = security_requirements.with_file_access(path);
    }
    let sandbox_context = SandboxContext::new(
        "bridge-tools".to_string(),
        security_requirements,
        ResourceLimits::default(),
    );
    let file_sandbox = Arc::new(FileSandbox::new(sandbox_context)?);

    // Register different tool categories with their specific configurations
    register_utility_tools(registry)?;
    register_data_processing_tools(registry, &tools_config.http_request)?;
    register_file_system_tools(registry, &file_sandbox, &tools_config.file_operations)?;
    register_system_tools(registry, &file_sandbox)?;
    register_media_tools(registry, &file_sandbox)?;
    register_search_tools(registry, &tools_config.web_search)?;
    register_web_tools(registry)?;
    register_communication_tools(registry)?;

    Ok(())
}

/// Register a single tool with the bridge registry
fn register_tool<T, F>(
    registry: &Arc<ComponentRegistry>,
    name: &str,
    tool_factory: F,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: Tool + Send + Sync + 'static,
    F: FnOnce() -> T,
{
    let tool = tool_factory();
    registry
        .register_tool(name.to_string(), Arc::new(tool))
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;
    Ok(())
}

/// Get all registered tool names
#[must_use]
pub fn get_all_tool_names(registry: &Arc<ComponentRegistry>) -> Vec<String> {
    registry.list_tools()
}

/// Get a tool by name from the registry
#[must_use]
pub fn get_tool_by_name(registry: &Arc<ComponentRegistry>, name: &str) -> Option<Arc<dyn Tool>> {
    registry.get_tool(name)
}

/// Register utility tools
fn register_utility_tools(
    registry: &Arc<ComponentRegistry>,
) -> Result<(), Box<dyn std::error::Error>> {
    register_tool(registry, "base64_encoder", Base64EncoderTool::new)?;
    register_tool(registry, "calculator", CalculatorTool::new)?;

    // Data validator: register with kebab-case primary name
    let data_validator_tool = Arc::new(DataValidationTool::new());
    registry.register_tool("data-validator".to_string(), data_validator_tool.clone())?;
    // Register with snake_case alias for backward compatibility
    registry.register_tool("data_validation".to_string(), data_validator_tool.clone())?;
    // Register with old -tool suffix alias for backward compatibility
    registry.register_tool("data-validation-tool".to_string(), data_validator_tool)?;

    register_tool(registry, "date_time_handler", DateTimeHandlerTool::new)?;
    register_tool(registry, "diff_calculator", DiffCalculatorTool::new)?;
    register_tool(registry, "hash_calculator", || {
        HashCalculatorTool::new(HashCalculatorConfig::default())
    })?;

    // Template creator: register with kebab-case primary name
    #[cfg(feature = "templates")]
    {
        let template_tool = Arc::new(TemplateEngineTool::new());
        registry.register_tool("template-creator".to_string(), template_tool.clone())?;
        // Register with snake_case alias for backward compatibility
        registry.register_tool("template_engine".to_string(), template_tool.clone())?;
        // Register with old -tool suffix alias for backward compatibility
        registry.register_tool("template-engine-tool".to_string(), template_tool)?;
    }

    register_tool(registry, "text_manipulator", || {
        TextManipulatorTool::new(TextManipulatorConfig::default())
    })?;
    register_tool(registry, "uuid_generator", || {
        UuidGeneratorTool::new(UuidGeneratorConfig::default())
    })?;
    // Phase 7 tools
    register_tool(registry, "citation-formatter", CitationFormatterTool::new)?;
    Ok(())
}

/// Register data processing tools
fn register_data_processing_tools(
    registry: &Arc<ComponentRegistry>,
    http_request_config: &llmspell_config::tools::HttpRequestConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    // CSV analyzer: register with kebab-case primary name
    #[cfg(feature = "csv-parquet")]
    {
        let csv_tool = Arc::new(CsvAnalyzerTool::new(CsvAnalyzerConfig::default()));
        registry.register_tool("csv-analyzer".to_string(), csv_tool.clone())?;
        // Register with snake_case alias for backward compatibility
        registry.register_tool("csv_analyzer".to_string(), csv_tool.clone())?;
        // Register with old -tool suffix alias for backward compatibility
        registry.register_tool("csv-analyzer-tool".to_string(), csv_tool)?;
    }

    // JSON processor: register with kebab-case primary name
    #[cfg(feature = "json-query")]
    {
        let json_tool = Arc::new(JsonProcessorTool::new(JsonProcessorConfig::default()));
        registry.register_tool("json-processor".to_string(), json_tool.clone())?;
        // Register with snake_case alias for backward compatibility
        registry.register_tool("json_processor".to_string(), json_tool.clone())?;
        // Register with old -tool suffix alias for backward compatibility
        registry.register_tool("json-processor-tool".to_string(), json_tool)?;
    }
    // GraphQL query: register with kebab-case primary name
    let graphql_tool = Arc::new(GraphQLQueryTool::new(GraphQLConfig::default())?);
    registry.register_tool("graphql-query".to_string(), graphql_tool.clone())?;
    // Register with snake_case alias for backward compatibility
    registry.register_tool("graphql_query".to_string(), graphql_tool.clone())?;
    // Register with old -tool suffix alias for backward compatibility
    registry.register_tool("graphql-query-tool".to_string(), graphql_tool)?;

    // HTTP requester: register with kebab-case primary name
    // Convert from llmspell_config HttpRequestConfig to llmspell_tools HttpRequestConfig
    // Note: Some fields in llmspell_config don't exist in tool config yet
    let tool_config = HttpRequestConfig {
        timeout_seconds: http_request_config.timeout_seconds,
        follow_redirects: true, // Default to following redirects
        max_redirects: http_request_config.max_redirects as usize,
        retry_config: RetryConfig::default(), // TODO: Add retry config to llmspell_config
        rate_limit_per_minute: None,          // TODO: Add rate limiting to llmspell_config
        user_agent: http_request_config
            .default_headers
            .get("User-Agent")
            .cloned()
            .unwrap_or_else(|| "llmspell-http/1.0".to_string()),
    };
    let http_tool = Arc::new(HttpRequestTool::new(tool_config)?);
    registry.register_tool("http-requester".to_string(), http_tool.clone())?;
    // Register with snake_case alias for backward compatibility
    registry.register_tool("http_request".to_string(), http_tool.clone())?;
    // Register with old -tool suffix alias for backward compatibility
    registry.register_tool("http-request-tool".to_string(), http_tool)?;
    // Phase 7 tools
    #[cfg(feature = "pdf")]
    register_tool(registry, "pdf-processor", PdfProcessorTool::new)?;
    register_tool(registry, "graph-builder", GraphBuilderTool::new)?;
    Ok(())
}

/// Register file system tools
fn register_file_system_tools(
    registry: &Arc<ComponentRegistry>,
    file_sandbox: &Arc<FileSandbox>,
    file_ops_config: &llmspell_config::tools::FileOperationsConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    // Archive handler: register with kebab-case primary name
    #[cfg(feature = "archives")]
    {
        let archive_tool = Arc::new(ArchiveHandlerTool::new());
        registry.register_tool("archive-handler".to_string(), archive_tool.clone())?;
        // Register with snake_case alias for backward compatibility
        registry.register_tool("archive_handler".to_string(), archive_tool.clone())?;
        // Register with old -tool suffix alias for backward compatibility
        registry.register_tool("archive-handler-tool".to_string(), archive_tool)?;
    }

    // File converter: register with kebab-case primary name
    let file_converter_tool = Arc::new(FileConverterTool::new(
        FileConverterConfig::default(),
        file_sandbox.clone(),
    ));
    registry.register_tool("file-converter".to_string(), file_converter_tool.clone())?;
    // Register with snake_case alias for backward compatibility
    registry.register_tool("file_converter".to_string(), file_converter_tool)?;

    // File operations: register with kebab-case primary name
    // Convert from llmspell_config FileOperationsConfig to llmspell_tools FileOperationsConfig
    let tool_config = FileOperationsConfig {
        allowed_paths: file_ops_config.allowed_paths.clone(),
        atomic_writes: file_ops_config.atomic_writes,
        max_file_size: file_ops_config.max_file_size,
        max_dir_entries: 1000,      // Default value
        allow_recursive: true,      // Default value
        default_permissions: 0o644, // Default permissions
    };
    let file_ops_tool = Arc::new(FileOperationsTool::new(tool_config, file_sandbox.clone()));
    registry.register_tool("file-operations".to_string(), file_ops_tool.clone())?;
    // Register with snake_case alias for backward compatibility
    registry.register_tool("file_operations".to_string(), file_ops_tool.clone())?;
    // Register with old -tool suffix alias for backward compatibility
    registry.register_tool("file-operations-tool".to_string(), file_ops_tool)?;

    // File search: register with kebab-case primary name
    let file_search_tool = Arc::new(FileSearchTool::new(
        FileSearchConfig::default(),
        file_sandbox.clone(),
    ));
    registry.register_tool("file-search".to_string(), file_search_tool.clone())?;
    // Register with snake_case alias for backward compatibility
    registry.register_tool("file_search".to_string(), file_search_tool)?;

    // File watcher: register with kebab-case primary name
    let file_watcher_tool = Arc::new(FileWatcherTool::new(
        FileWatcherConfig::default(),
        file_sandbox.clone(),
    ));
    registry.register_tool("file-watcher".to_string(), file_watcher_tool.clone())?;
    // Register with snake_case alias for backward compatibility
    registry.register_tool("file_watcher".to_string(), file_watcher_tool)?;
    Ok(())
}

/// Register system integration tools
fn register_system_tools(
    registry: &Arc<ComponentRegistry>,
    file_sandbox: &Arc<FileSandbox>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Environment reader: register with kebab-case primary name
    let env_tool = Arc::new(EnvironmentReaderTool::new(
        EnvironmentReaderConfig::default(),
    ));
    registry.register_tool("environment-reader".to_string(), env_tool.clone())?;
    // Register with snake_case alias for backward compatibility
    registry.register_tool("environment_reader".to_string(), env_tool)?;

    // Process executor: register with kebab-case primary name
    let process_tool = Arc::new(ProcessExecutorTool::new(
        ProcessExecutorConfig::default(),
        file_sandbox.clone(),
    ));
    registry.register_tool("process-executor".to_string(), process_tool.clone())?;
    // Register with snake_case alias for backward compatibility
    registry.register_tool("process_executor".to_string(), process_tool)?;

    // Service checker: register with kebab-case primary name
    let service_tool = Arc::new(ServiceCheckerTool::new(ServiceCheckerConfig::default()));
    registry.register_tool("service-checker".to_string(), service_tool.clone())?;
    // Register with snake_case alias for backward compatibility
    registry.register_tool("service_checker".to_string(), service_tool)?;

    // System monitor: register with kebab-case primary name
    let monitor_tool = Arc::new(SystemMonitorTool::new(
        SystemMonitorConfig::default(),
        file_sandbox.clone(),
    ));
    registry.register_tool("system-monitor".to_string(), monitor_tool.clone())?;
    // Register with snake_case alias for backward compatibility
    registry.register_tool("system_monitor".to_string(), monitor_tool)?;
    Ok(())
}

/// Register media processing tools
fn register_media_tools(
    registry: &Arc<ComponentRegistry>,
    file_sandbox: &Arc<FileSandbox>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Audio processor: register with kebab-case primary name
    let audio_tool = Arc::new(AudioProcessorTool::new(
        AudioProcessorConfig::default(),
        file_sandbox.clone(),
    ));
    registry.register_tool("audio-processor".to_string(), audio_tool.clone())?;
    // Register with snake_case alias for backward compatibility
    registry.register_tool("audio_processor".to_string(), audio_tool)?;

    // Image processor: register with kebab-case primary name
    let image_tool = Arc::new(ImageProcessorTool::new(
        ImageProcessorConfig::default(),
        file_sandbox.clone(),
    ));
    registry.register_tool("image-processor".to_string(), image_tool.clone())?;
    // Register with snake_case alias for backward compatibility
    registry.register_tool("image_processor".to_string(), image_tool)?;

    // Video processor: register with kebab-case primary name
    let video_tool = Arc::new(VideoProcessorTool::new(
        VideoProcessorConfig::default(),
        file_sandbox.clone(),
    ));
    registry.register_tool("video-processor".to_string(), video_tool.clone())?;
    // Register with snake_case alias for backward compatibility
    registry.register_tool("video_processor".to_string(), video_tool)?;

    Ok(())
}

/// Register search tools
fn register_search_tools(
    registry: &Arc<ComponentRegistry>,
    web_search_config: &llmspell_config::tools::WebSearchConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    // Web searcher: register with kebab-case primary name
    // Convert from llmspell_config WebSearchConfig to llmspell_tools WebSearchConfig
    // Note: Config structures have different fields - using defaults for missing ones
    let tool_config = WebSearchConfig {
        default_provider: "duckduckgo".to_string(), // Default provider
        providers: HashMap::new(),                  // TODO: Add provider configuration
        max_results: web_search_config.max_results,
        safe_search: true,                              // Default to safe search
        language: None,                                 // Default language
        fallback_chain: vec!["duckduckgo".to_string()], // Default fallback
    };
    let web_search_tool = Arc::new(WebSearchTool::new(tool_config)?);
    registry.register_tool("web-searcher".to_string(), web_search_tool.clone())?;
    // Register with snake_case alias for backward compatibility
    registry.register_tool("web_search".to_string(), web_search_tool.clone())?;
    // Register with old -tool suffix alias for backward compatibility
    registry.register_tool("web-search-tool".to_string(), web_search_tool)?;
    Ok(())
}

/// Register web tools
fn register_web_tools(registry: &Arc<ComponentRegistry>) -> Result<(), Box<dyn std::error::Error>> {
    register_tool(registry, "url-analyzer", UrlAnalyzerTool::new)?;
    register_tool(registry, "web-scraper", || {
        WebScraperTool::new(WebScraperConfig::default())
    })?;
    register_tool(registry, "api-tester", ApiTesterTool::new)?;
    register_tool(registry, "webhook-caller", WebhookCallerTool::new)?;
    register_tool(registry, "webpage-monitor", WebpageMonitorTool::new)?;
    register_tool(registry, "sitemap-crawler", SitemapCrawlerTool::new)?;
    Ok(())
}

/// Register communication tools
#[allow(unused_variables)] // registry is unused when no features are enabled
#[allow(clippy::unnecessary_wraps)] // Result needed for consistency with other register functions
fn register_communication_tools(
    registry: &Arc<ComponentRegistry>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Email sender: register with kebab-case primary name
    #[cfg(feature = "email")]
    {
        let email_tool = Arc::new(EmailSenderTool::new(EmailSenderConfig::default())?);
        registry.register_tool("email-sender".to_string(), email_tool.clone())?;
        // Register with snake_case alias for backward compatibility
        registry.register_tool("email_sender".to_string(), email_tool)?;
    }

    // Database connector: register with kebab-case primary name
    #[cfg(feature = "database")]
    {
        let db_tool = Arc::new(DatabaseConnectorTool::new(
            DatabaseConnectorConfig::default(),
        )?);
        registry.register_tool("database-connector".to_string(), db_tool.clone())?;
        // Register with snake_case alias for backward compatibility
        registry.register_tool("database_connector".to_string(), db_tool)?;
    }
    Ok(())
}

/// Information about a tool type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolInfo {
    /// Tool name
    pub name: String,
    /// Tool description
    pub description: String,
    /// Tool category
    pub category: ToolCategory,
    /// Security level
    pub security_level: SecurityLevel,
    /// Tool schema with parameters
    pub schema: ToolSchema,
    /// Security requirements
    pub security_requirements: SecurityRequirements,
    /// Resource limits
    pub resource_limits: ResourceLimits,
}

/// Tool discovery service
pub struct ToolDiscovery {
    /// Component registry
    registry: Arc<ComponentRegistry>,
}

impl ToolDiscovery {
    /// Create a new tool discovery service
    #[must_use]
    pub const fn new(registry: Arc<ComponentRegistry>) -> Self {
        Self { registry }
    }

    /// Get information about a specific tool
    #[must_use]
    pub fn get_tool_info(&self, tool_name: &str) -> Option<ToolInfo> {
        let tool = self.registry.get_tool(tool_name)?;
        let metadata = tool.metadata();
        let schema = tool.schema();
        let category = tool.category();
        let security_level = tool.security_level();
        let security_requirements = tool.security_requirements();
        let resource_limits = tool.resource_limits();

        Some(ToolInfo {
            name: tool_name.to_string(),
            description: metadata.description.clone(),
            category,
            security_level,
            schema,
            security_requirements,
            resource_limits,
        })
    }

    /// List all available tool names
    #[must_use]
    pub fn list_tool_names(&self) -> Vec<String> {
        self.registry.list_tools()
    }

    /// Get tools by category
    #[must_use]
    pub fn get_tools_by_category(&self, category: &str) -> Vec<(String, ToolInfo)> {
        let tool_names = self.registry.list_tools();
        tool_names
            .into_iter()
            .filter_map(|name| {
                let tool = self.registry.get_tool(&name)?;
                let tool_category = tool.category();
                if tool_category.to_string() == category {
                    self.get_tool_info(&name).map(|info| (name, info))
                } else {
                    None
                }
            })
            .collect()
    }
}

/// Implementation of unified `BridgeDiscovery` trait for `ToolDiscovery`
#[async_trait::async_trait]
impl BridgeDiscovery<ToolInfo> for ToolDiscovery {
    async fn discover_types(&self) -> Vec<(String, ToolInfo)> {
        self.list_tool_names()
            .into_iter()
            .filter_map(|name| self.get_tool_info(&name).map(|info| (name, info)))
            .collect()
    }

    async fn get_type_info(&self, type_name: &str) -> Option<ToolInfo> {
        self.get_tool_info(type_name)
    }

    async fn has_type(&self, type_name: &str) -> bool {
        self.registry.get_tool(type_name).is_some()
    }

    async fn list_types(&self) -> Vec<String> {
        self.list_tool_names()
    }

    async fn filter_types<F>(&self, predicate: F) -> Vec<(String, ToolInfo)>
    where
        F: Fn(&str, &ToolInfo) -> bool + Send,
    {
        let tool_names = self.registry.list_tools();
        tool_names
            .into_iter()
            .filter_map(|name| {
                self.get_tool_info(&name)
                    .filter(|info| predicate(&name, info))
                    .map(|info| (name, info))
            })
            .collect()
    }
}
