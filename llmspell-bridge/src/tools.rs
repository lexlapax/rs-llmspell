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
    ApiTesterTool, AudioProcessorTool, Base64EncoderTool, CalculatorTool,
    CitationFormatterTool, DataValidationTool, DateTimeHandlerTool, DiffCalculatorTool,
    EnvironmentReaderTool, FileConverterTool, FileOperationsTool, FileSearchTool,
    FileWatcherTool, GraphBuilderTool, GraphQLQueryTool, HashCalculatorTool, HttpRequestTool,
    ImageProcessorTool, ProcessExecutorTool, ServiceCheckerTool, SitemapCrawlerTool,
    SystemMonitorTool, TextManipulatorTool, UrlAnalyzerTool, UuidGeneratorTool,
    VideoProcessorTool, WebScraperTool, WebSearchTool, WebhookCallerTool, WebpageMonitorTool,
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
    register_file_system_tools(
        registry,
        file_sandbox.clone(),
        &tools_config.file_operations,
    )?;
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

/// Register a tool that requires a sandbox with the bridge registry
fn register_tool_with_sandbox<T, F>(
    registry: &Arc<ComponentRegistry>,
    name: &str,
    _sandbox: Arc<FileSandbox>,
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

/// Register a tool that returns a Result
fn register_tool_result<T, F>(
    registry: &Arc<ComponentRegistry>,
    name: &str,
    tool_factory: F,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: Tool + Send + Sync + 'static,
    F: FnOnce() -> Result<T, llmspell_core::error::LLMSpellError>,
{
    let tool = tool_factory()?;
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
    register_tool(registry, "data_validation", DataValidationTool::new)?;
    register_tool(registry, "date_time_handler", DateTimeHandlerTool::new)?;
    register_tool(registry, "diff_calculator", DiffCalculatorTool::new)?;
    register_tool(registry, "hash_calculator", || {
        HashCalculatorTool::new(HashCalculatorConfig::default())
    })?;
    #[cfg(feature = "templates")]
    register_tool(registry, "template_engine", TemplateEngineTool::new)?;
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
    #[cfg(feature = "csv-parquet")]
    register_tool(registry, "csv_analyzer", || {
        CsvAnalyzerTool::new(CsvAnalyzerConfig::default())
    })?;
    #[cfg(feature = "json-query")]
    register_tool(registry, "json_processor", || {
        JsonProcessorTool::new(JsonProcessorConfig::default())
    })?;
    register_tool_result(registry, "graphql_query", || {
        GraphQLQueryTool::new(GraphQLConfig::default())
    })?;
    // Use the provided configuration for HttpRequestTool
    let http_request_config = http_request_config.clone();
    register_tool_result(registry, "http_request", move || {
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
        HttpRequestTool::new(tool_config)
    })?;
    // Phase 7 tools
    #[cfg(feature = "pdf")]
    register_tool(registry, "pdf-processor", PdfProcessorTool::new)?;
    register_tool(registry, "graph-builder", GraphBuilderTool::new)?;
    Ok(())
}

/// Register file system tools
fn register_file_system_tools(
    registry: &Arc<ComponentRegistry>,
    file_sandbox: Arc<FileSandbox>,
    file_ops_config: &llmspell_config::tools::FileOperationsConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "archives")]
    register_tool(registry, "archive_handler", ArchiveHandlerTool::new)?;

    // File converter with sandbox
    let file_sandbox_converter = file_sandbox.clone();
    register_tool_with_sandbox(
        registry,
        "file_converter",
        file_sandbox_converter.clone(),
        move || FileConverterTool::new(FileConverterConfig::default(), file_sandbox_converter),
    )?;

    // Use the provided configuration for FileOperationsTool with sandbox
    let file_ops_config = file_ops_config.clone();
    let file_sandbox_ops = file_sandbox.clone();
    register_tool_with_sandbox(
        registry,
        "file_operations",
        file_sandbox_ops.clone(),
        move || {
            // Convert from llmspell_config FileOperationsConfig to llmspell_tools FileOperationsConfig
            let tool_config = FileOperationsConfig {
                allowed_paths: file_ops_config.allowed_paths.clone(),
                atomic_writes: file_ops_config.atomic_writes,
                max_file_size: file_ops_config.max_file_size,
                max_dir_entries: 1000,      // Default value
                allow_recursive: true,      // Default value
                default_permissions: 0o644, // Default permissions
            };
            FileOperationsTool::new(tool_config, file_sandbox_ops)
        },
    )?;

    // File search with sandbox
    let file_sandbox_search = file_sandbox.clone();
    register_tool_with_sandbox(
        registry,
        "file_search",
        file_sandbox_search.clone(),
        move || FileSearchTool::new(FileSearchConfig::default(), file_sandbox_search),
    )?;

    // File watcher with sandbox
    let file_sandbox_watcher = file_sandbox;
    register_tool_with_sandbox(
        registry,
        "file_watcher",
        file_sandbox_watcher.clone(),
        move || FileWatcherTool::new(FileWatcherConfig::default(), file_sandbox_watcher),
    )?;
    Ok(())
}

/// Register system integration tools
fn register_system_tools(
    registry: &Arc<ComponentRegistry>,
    file_sandbox: &Arc<FileSandbox>,
) -> Result<(), Box<dyn std::error::Error>> {
    register_tool(registry, "environment_reader", || {
        EnvironmentReaderTool::new(EnvironmentReaderConfig::default())
    })?;
    // ProcessExecutorTool needs sandbox for working directory validation
    let process_executor_sandbox = file_sandbox.clone();
    register_tool_with_sandbox(
        registry,
        "process_executor",
        process_executor_sandbox.clone(),
        move || {
            ProcessExecutorTool::new(ProcessExecutorConfig::default(), process_executor_sandbox)
        },
    )?;
    register_tool(registry, "service_checker", || {
        ServiceCheckerTool::new(ServiceCheckerConfig::default())
    })?;

    // SystemMonitorTool needs sandbox for /proc file access
    let system_monitor_sandbox = file_sandbox.clone();
    register_tool_with_sandbox(
        registry,
        "system_monitor",
        system_monitor_sandbox.clone(),
        move || SystemMonitorTool::new(SystemMonitorConfig::default(), system_monitor_sandbox),
    )?;
    Ok(())
}

/// Register media processing tools
fn register_media_tools(
    registry: &Arc<ComponentRegistry>,
    file_sandbox: &Arc<FileSandbox>,
) -> Result<(), Box<dyn std::error::Error>> {
    let audio_sandbox = file_sandbox.clone();
    register_tool_with_sandbox(
        registry,
        "audio_processor",
        audio_sandbox.clone(),
        move || AudioProcessorTool::new(AudioProcessorConfig::default(), audio_sandbox),
    )?;
    let image_sandbox = file_sandbox.clone();
    register_tool_with_sandbox(
        registry,
        "image_processor",
        image_sandbox.clone(),
        move || ImageProcessorTool::new(ImageProcessorConfig::default(), image_sandbox),
    )?;
    let video_sandbox = file_sandbox.clone();
    register_tool_with_sandbox(
        registry,
        "video_processor",
        video_sandbox.clone(),
        move || VideoProcessorTool::new(VideoProcessorConfig::default(), video_sandbox),
    )?;
    Ok(())
}

/// Register search tools
fn register_search_tools(
    registry: &Arc<ComponentRegistry>,
    web_search_config: &llmspell_config::tools::WebSearchConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    // Use the provided configuration for WebSearchTool
    let web_search_config = web_search_config.clone();
    register_tool_result(registry, "web_search", move || {
        // Convert from llmspell_config WebSearchConfig to llmspell_tools WebSearchConfig
        // Note: Config structures have different fields - using defaults for missing ones
        let tool_config = WebSearchConfig {
            default_provider: "duckduckgo".to_string(), // Default provider
            providers: HashMap::new(),                  // TODO: Add provider configuration
            max_results: web_search_config.max_results,
            safe_search: true, // Default to safe search
            language: None,    // Default language
            fallback_chain: vec!["duckduckgo".to_string()], // Default fallback
        };
        WebSearchTool::new(tool_config)
    })?;
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
#[allow(unused_variables)]  // registry is unused when no features are enabled
#[allow(clippy::unnecessary_wraps)]  // Result needed for consistency with other register functions
fn register_communication_tools(
    registry: &Arc<ComponentRegistry>,
) -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "email")]
    register_tool_result(registry, "email-sender", || {
        EmailSenderTool::new(EmailSenderConfig::default())
    })?;
    #[cfg(feature = "database")]
    register_tool_result(registry, "database-connector", || {
        DatabaseConnectorTool::new(DatabaseConnectorConfig::default())
    })?;
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
