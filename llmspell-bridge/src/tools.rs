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
use std::sync::Arc;

/// Initialize and register all Phase 2 tools with BOTH registries (dual-registration)
///
/// Phase 12.7.1.2: Registers tools in both `ComponentRegistry` (script access) and
/// `ToolRegistry` (template infrastructure). This enables both Lua/JS scripts and
/// template execution to access the same tool instances via Arc sharing.
///
/// # Errors
///
/// Returns an error if tool registration fails in either registry
pub async fn register_all_tools(
    component_registry: &Arc<ComponentRegistry>,
    tool_registry: &Arc<llmspell_tools::ToolRegistry>,
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
    // Phase 12.7.1.2: Pass both registries for dual-registration
    register_utility_tools(component_registry, tool_registry).await?;
    register_data_processing_tools(
        component_registry,
        tool_registry,
        &tools_config.http_request,
    )
    .await?;
    register_file_system_tools(
        component_registry,
        tool_registry,
        &file_sandbox,
        &tools_config.file_operations,
    )
    .await?;
    register_system_tools(component_registry, tool_registry, &file_sandbox).await?;
    register_media_tools(component_registry, tool_registry, &file_sandbox).await?;
    register_search_tools(component_registry, tool_registry, &tools_config.web_search).await?;
    register_web_tools(component_registry, tool_registry).await?;
    register_communication_tools(component_registry, tool_registry).await?;

    Ok(())
}

/// Register a single tool with BOTH registries (dual-registration pattern)
///
/// Phase 12.7.1.2: Registers tools in both:
/// 1. `ComponentRegistry` (`HashMap` for script access)
/// 2. `ToolRegistry` (infrastructure with hooks, caching, discovery)
///
/// Creates two separate tool instances (tools are stateless, so this is acceptable).
/// `ToolRegistry` requires ownership to wrap internally with `Arc` and trait objects.
async fn register_tool_dual<T, F>(
    component_registry: &Arc<ComponentRegistry>,
    tool_registry: &Arc<llmspell_tools::ToolRegistry>,
    name: &str,
    mut tool_factory: F,
) -> Result<(), Box<dyn std::error::Error>>
where
    T: Tool + Send + Sync + 'static,
    F: FnMut() -> T,
{
    // Create first instance for ComponentRegistry
    let tool_for_component = Arc::new(tool_factory());
    component_registry
        .register_tool(name.to_string(), tool_for_component)
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    // Create second instance for ToolRegistry (it needs owned T to wrap internally)
    let tool_for_infrastructure = tool_factory();
    tool_registry
        .register(name.to_string(), tool_for_infrastructure)
        .await
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

/// Register utility tools with dual-registration
async fn register_utility_tools(
    component_registry: &Arc<ComponentRegistry>,
    tool_registry: &Arc<llmspell_tools::ToolRegistry>,
) -> Result<(), Box<dyn std::error::Error>> {
    register_tool_dual(component_registry, tool_registry, "base64-encoder", || {
        Base64EncoderTool::new()
    })
    .await?;
    register_tool_dual(component_registry, tool_registry, "calculator", || {
        CalculatorTool::new()
    })
    .await?;

    // Data validator - manual dual-registration (create separate instances)
    component_registry.register_tool(
        "data-validator".to_string(),
        Arc::new(DataValidationTool::new()),
    )?;
    tool_registry
        .register("data-validator".to_string(), DataValidationTool::new())
        .await?;

    register_tool_dual(
        component_registry,
        tool_registry,
        "datetime-handler",
        DateTimeHandlerTool::new,
    )
    .await?;
    register_tool_dual(component_registry, tool_registry, "diff-calculator", || {
        DiffCalculatorTool::new()
    })
    .await?;
    register_tool_dual(component_registry, tool_registry, "hash-calculator", || {
        HashCalculatorTool::new(HashCalculatorConfig::default())
    })
    .await?;

    // Template creator - manual dual-registration (create separate instances)
    #[cfg(feature = "templates")]
    {
        component_registry.register_tool(
            "template-creator".to_string(),
            Arc::new(TemplateEngineTool::new()),
        )?;
        tool_registry
            .register("template-creator".to_string(), TemplateEngineTool::new())
            .await?;
    }

    register_tool_dual(
        component_registry,
        tool_registry,
        "text-manipulator",
        || TextManipulatorTool::new(TextManipulatorConfig::default()),
    )
    .await?;
    register_tool_dual(component_registry, tool_registry, "uuid-generator", || {
        UuidGeneratorTool::new(UuidGeneratorConfig::default())
    })
    .await?;
    // Phase 7 tools
    register_tool_dual(
        component_registry,
        tool_registry,
        "citation-formatter",
        CitationFormatterTool::new,
    )
    .await?;
    Ok(())
}

/// Register data processing tools with dual-registration
async fn register_data_processing_tools(
    component_registry: &Arc<ComponentRegistry>,
    tool_registry: &Arc<llmspell_tools::ToolRegistry>,
    http_request_config: &llmspell_config::tools::HttpRequestConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    // CSV analyzer - manual dual-registration (create separate instances)
    #[cfg(feature = "csv-parquet")]
    {
        component_registry.register_tool(
            "csv-analyzer".to_string(),
            Arc::new(CsvAnalyzerTool::new(CsvAnalyzerConfig::default())),
        )?;
        tool_registry
            .register(
                "csv-analyzer".to_string(),
                CsvAnalyzerTool::new(CsvAnalyzerConfig::default()),
            )
            .await?;
    }

    // JSON processor - manual dual-registration (create separate instances)
    #[cfg(feature = "json-query")]
    {
        component_registry.register_tool(
            "json-processor".to_string(),
            Arc::new(JsonProcessorTool::new(JsonProcessorConfig::default())),
        )?;
        tool_registry
            .register(
                "json-processor".to_string(),
                JsonProcessorTool::new(JsonProcessorConfig::default()),
            )
            .await?;
    }
    // GraphQL query - manual dual-registration (create separate instances)
    component_registry.register_tool(
        "graphql-query".to_string(),
        Arc::new(GraphQLQueryTool::new(GraphQLConfig::default())?),
    )?;
    tool_registry
        .register(
            "graphql-query".to_string(),
            GraphQLQueryTool::new(GraphQLConfig::default())?,
        )
        .await?;

    // HTTP requester: register with kebab-case primary name - manual dual-registration (create separate instances)
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
    component_registry.register_tool(
        "http-requester".to_string(),
        Arc::new(HttpRequestTool::new(tool_config.clone())?),
    )?;
    tool_registry
        .register(
            "http-requester".to_string(),
            HttpRequestTool::new(tool_config)?,
        )
        .await?;

    // Phase 7 tools
    #[cfg(feature = "pdf")]
    register_tool_dual(component_registry, tool_registry, "pdf-processor", || {
        PdfProcessorTool::new()
    })
    .await?;
    register_tool_dual(component_registry, tool_registry, "graph-builder", || {
        GraphBuilderTool::new()
    })
    .await?;
    Ok(())
}

/// Register file system tools with dual-registration
async fn register_file_system_tools(
    component_registry: &Arc<ComponentRegistry>,
    tool_registry: &Arc<llmspell_tools::ToolRegistry>,
    file_sandbox: &Arc<FileSandbox>,
    file_ops_config: &llmspell_config::tools::FileOperationsConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    // Archive handler - manual dual-registration (create separate instances)
    #[cfg(feature = "archives")]
    {
        component_registry.register_tool(
            "archive-handler".to_string(),
            Arc::new(ArchiveHandlerTool::new()),
        )?;
        tool_registry
            .register("archive-handler".to_string(), ArchiveHandlerTool::new())
            .await?;
    }

    // File converter - manual dual-registration (create separate instances)
    component_registry.register_tool(
        "file-converter".to_string(),
        Arc::new(FileConverterTool::new(
            FileConverterConfig::default(),
            file_sandbox.clone(),
        )),
    )?;
    tool_registry
        .register(
            "file-converter".to_string(),
            FileConverterTool::new(FileConverterConfig::default(), file_sandbox.clone()),
        )
        .await?;

    // File operations: register with kebab-case primary name - manual dual-registration (create separate instances)
    // Convert from llmspell_config FileOperationsConfig to llmspell_tools FileOperationsConfig
    let tool_config = FileOperationsConfig {
        allowed_paths: file_ops_config.allowed_paths.clone(),
        atomic_writes: file_ops_config.atomic_writes,
        max_file_size: file_ops_config.max_file_size,
        max_dir_entries: 1000,      // Default value
        allow_recursive: true,      // Default value
        default_permissions: 0o644, // Default permissions
    };
    component_registry.register_tool(
        "file-operations".to_string(),
        Arc::new(FileOperationsTool::new(
            tool_config.clone(),
            file_sandbox.clone(),
        )),
    )?;
    tool_registry
        .register(
            "file-operations".to_string(),
            FileOperationsTool::new(tool_config, file_sandbox.clone()),
        )
        .await?;

    // File search - manual dual-registration (create separate instances)
    component_registry.register_tool(
        "file-search".to_string(),
        Arc::new(FileSearchTool::new(
            FileSearchConfig::default(),
            file_sandbox.clone(),
        )),
    )?;
    tool_registry
        .register(
            "file-search".to_string(),
            FileSearchTool::new(FileSearchConfig::default(), file_sandbox.clone()),
        )
        .await?;

    // File watcher - manual dual-registration (create separate instances)
    component_registry.register_tool(
        "file-watcher".to_string(),
        Arc::new(FileWatcherTool::new(
            FileWatcherConfig::default(),
            file_sandbox.clone(),
        )),
    )?;
    tool_registry
        .register(
            "file-watcher".to_string(),
            FileWatcherTool::new(FileWatcherConfig::default(), file_sandbox.clone()),
        )
        .await?;
    Ok(())
}

/// Register system integration tools with dual-registration
async fn register_system_tools(
    component_registry: &Arc<ComponentRegistry>,
    tool_registry: &Arc<llmspell_tools::ToolRegistry>,
    file_sandbox: &Arc<FileSandbox>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Environment reader - manual dual-registration (create separate instances)
    component_registry.register_tool(
        "environment-reader".to_string(),
        Arc::new(EnvironmentReaderTool::new(
            EnvironmentReaderConfig::default(),
        )),
    )?;
    tool_registry
        .register(
            "environment-reader".to_string(),
            EnvironmentReaderTool::new(EnvironmentReaderConfig::default()),
        )
        .await?;

    // Process executor - manual dual-registration (create separate instances)
    component_registry.register_tool(
        "process-executor".to_string(),
        Arc::new(ProcessExecutorTool::new(
            ProcessExecutorConfig::default(),
            file_sandbox.clone(),
        )),
    )?;
    tool_registry
        .register(
            "process-executor".to_string(),
            ProcessExecutorTool::new(ProcessExecutorConfig::default(), file_sandbox.clone()),
        )
        .await?;

    // Service checker - manual dual-registration (create separate instances)
    component_registry.register_tool(
        "service-checker".to_string(),
        Arc::new(ServiceCheckerTool::new(ServiceCheckerConfig::default())),
    )?;
    tool_registry
        .register(
            "service-checker".to_string(),
            ServiceCheckerTool::new(ServiceCheckerConfig::default()),
        )
        .await?;

    // System monitor - manual dual-registration (create separate instances)
    component_registry.register_tool(
        "system-monitor".to_string(),
        Arc::new(SystemMonitorTool::new(
            SystemMonitorConfig::default(),
            file_sandbox.clone(),
        )),
    )?;
    tool_registry
        .register(
            "system-monitor".to_string(),
            SystemMonitorTool::new(SystemMonitorConfig::default(), file_sandbox.clone()),
        )
        .await?;
    Ok(())
}

/// Register media processing tools with dual-registration
async fn register_media_tools(
    component_registry: &Arc<ComponentRegistry>,
    tool_registry: &Arc<llmspell_tools::ToolRegistry>,
    file_sandbox: &Arc<FileSandbox>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Audio processor - manual dual-registration (create separate instances)
    component_registry.register_tool(
        "audio-processor".to_string(),
        Arc::new(AudioProcessorTool::new(
            AudioProcessorConfig::default(),
            file_sandbox.clone(),
        )),
    )?;
    tool_registry
        .register(
            "audio-processor".to_string(),
            AudioProcessorTool::new(AudioProcessorConfig::default(), file_sandbox.clone()),
        )
        .await?;

    // Image processor - manual dual-registration (create separate instances)
    component_registry.register_tool(
        "image-processor".to_string(),
        Arc::new(ImageProcessorTool::new(
            ImageProcessorConfig::default(),
            file_sandbox.clone(),
        )),
    )?;
    tool_registry
        .register(
            "image-processor".to_string(),
            ImageProcessorTool::new(ImageProcessorConfig::default(), file_sandbox.clone()),
        )
        .await?;

    // Video processor - manual dual-registration (create separate instances)
    component_registry.register_tool(
        "video-processor".to_string(),
        Arc::new(VideoProcessorTool::new(
            VideoProcessorConfig::default(),
            file_sandbox.clone(),
        )),
    )?;
    tool_registry
        .register(
            "video-processor".to_string(),
            VideoProcessorTool::new(VideoProcessorConfig::default(), file_sandbox.clone()),
        )
        .await?;

    Ok(())
}

/// Register search tools with dual-registration
async fn register_search_tools(
    component_registry: &Arc<ComponentRegistry>,
    tool_registry: &Arc<llmspell_tools::ToolRegistry>,
    web_search_config: &llmspell_config::tools::WebSearchConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    // Web searcher: register with kebab-case primary name - manual dual-registration (create separate instances)
    // Load config from environment to pick up API keys (BRAVE_API_KEY, SERPAPI_API_KEY, SERPERDEV_API_KEY, etc.)
    // This sets up the full fallback chain: duckduckgo → serperdev → brave → google → serpapi
    let mut tool_config = WebSearchConfig::from_env();

    // Override with user-specified max_results if provided
    tool_config.max_results = web_search_config.max_results;
    component_registry.register_tool(
        "web-searcher".to_string(),
        Arc::new(WebSearchTool::new(tool_config.clone())?),
    )?;
    tool_registry
        .register("web-searcher".to_string(), WebSearchTool::new(tool_config)?)
        .await?;
    Ok(())
}

/// Register web tools with dual-registration
async fn register_web_tools(
    component_registry: &Arc<ComponentRegistry>,
    tool_registry: &Arc<llmspell_tools::ToolRegistry>,
) -> Result<(), Box<dyn std::error::Error>> {
    register_tool_dual(component_registry, tool_registry, "url-analyzer", || {
        UrlAnalyzerTool::new()
    })
    .await?;
    register_tool_dual(component_registry, tool_registry, "web-scraper", || {
        WebScraperTool::new(WebScraperConfig::default())
    })
    .await?;
    register_tool_dual(component_registry, tool_registry, "api-tester", || {
        ApiTesterTool::new()
    })
    .await?;
    register_tool_dual(component_registry, tool_registry, "webhook-caller", || {
        WebhookCallerTool::new()
    })
    .await?;
    register_tool_dual(component_registry, tool_registry, "webpage-monitor", || {
        WebpageMonitorTool::new()
    })
    .await?;
    register_tool_dual(component_registry, tool_registry, "sitemap-crawler", || {
        SitemapCrawlerTool::new()
    })
    .await?;
    Ok(())
}

/// Register communication tools with dual-registration
#[allow(unused_variables)] // registries are unused when no features are enabled
#[allow(clippy::unnecessary_wraps)] // Result needed for consistency with other register functions
async fn register_communication_tools(
    component_registry: &Arc<ComponentRegistry>,
    tool_registry: &Arc<llmspell_tools::ToolRegistry>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Email sender - manual dual-registration (create separate instances)
    #[cfg(feature = "email")]
    {
        component_registry.register_tool(
            "email-sender".to_string(),
            Arc::new(EmailSenderTool::new(EmailSenderConfig::default())?),
        )?;
        tool_registry
            .register(
                "email-sender".to_string(),
                EmailSenderTool::new(EmailSenderConfig::default())?,
            )
            .await?;
    }

    // Database connector - manual dual-registration (create separate instances)
    #[cfg(feature = "database")]
    {
        component_registry.register_tool(
            "database-connector".to_string(),
            Arc::new(DatabaseConnectorTool::new(
                DatabaseConnectorConfig::default(),
            )?),
        )?;
        tool_registry
            .register(
                "database-connector".to_string(),
                DatabaseConnectorTool::new(DatabaseConnectorConfig::default())?,
            )
            .await?;
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
