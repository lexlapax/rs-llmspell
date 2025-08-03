//! ABOUTME: Tool registration and management for the bridge
//! ABOUTME: Initializes and provides access to all Phase 2 tools from llmspell-tools

use crate::discovery::BridgeDiscovery;
use crate::ComponentRegistry;
use llmspell_core::traits::tool::{ResourceLimits, SecurityRequirements};
use llmspell_core::Tool;
use llmspell_security::sandbox::{file_sandbox::FileSandbox, SandboxContext};
use llmspell_tools::{
    ApiTesterTool, ArchiveHandlerTool, AudioProcessorTool, Base64EncoderTool, CalculatorTool,
    CsvAnalyzerTool, DataValidationTool, DatabaseConnectorTool, DateTimeHandlerTool,
    DiffCalculatorTool, EmailSenderTool, EnvironmentReaderTool, FileConverterTool,
    FileOperationsTool, FileSearchTool, FileWatcherTool, GraphQLQueryTool, HashCalculatorTool,
    HttpRequestTool, ImageProcessorTool, JsonProcessorTool, ProcessExecutorTool,
    ServiceCheckerTool, SitemapCrawlerTool, SystemMonitorTool, TemplateEngineTool,
    TextManipulatorTool, UrlAnalyzerTool, UuidGeneratorTool, VideoProcessorTool, WebScraperTool,
    WebSearchTool, WebhookCallerTool, WebpageMonitorTool,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;

/// Initialize and register all Phase 2 tools with the bridge registry
///
/// # Errors
///
/// Returns an error if tool registration fails
#[allow(clippy::default_trait_access)]
pub fn register_all_tools(
    registry: Arc<ComponentRegistry>,
) -> Result<(), Box<dyn std::error::Error>> {
    // Create a shared file sandbox for file system tools
    let security_requirements = SecurityRequirements::default().with_file_access("/tmp");
    let sandbox_context = SandboxContext::new(
        "bridge-tools".to_string(),
        security_requirements,
        ResourceLimits::default(),
    );
    let file_sandbox = Arc::new(FileSandbox::new(sandbox_context)?);

    // Register different tool categories
    register_utility_tools(registry.clone())?;
    register_data_processing_tools(registry.clone())?;
    register_file_system_tools(registry.clone(), file_sandbox)?;
    register_system_tools(registry.clone())?;
    register_media_tools(registry.clone())?;
    register_search_tools(registry.clone())?;
    register_web_tools(registry.clone())?;
    register_communication_tools(registry)?;

    Ok(())
}

/// Register a single tool with the bridge registry
fn register_tool<T, F>(
    registry: Arc<ComponentRegistry>,
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
    registry: Arc<ComponentRegistry>,
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
    registry: Arc<ComponentRegistry>,
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
pub fn get_all_tool_names(registry: Arc<ComponentRegistry>) -> Vec<String> {
    registry.list_tools()
}

/// Get a tool by name from the registry
#[must_use]
pub fn get_tool_by_name(registry: Arc<ComponentRegistry>, name: &str) -> Option<Arc<dyn Tool>> {
    registry.get_tool(name)
}

/// Register utility tools
fn register_utility_tools(
    registry: Arc<ComponentRegistry>,
) -> Result<(), Box<dyn std::error::Error>> {
    register_tool(registry.clone(), "base64_encoder", Base64EncoderTool::new)?;
    register_tool(registry.clone(), "calculator", CalculatorTool::new)?;
    register_tool(registry.clone(), "data_validation", DataValidationTool::new)?;
    register_tool(
        registry.clone(),
        "date_time_handler",
        DateTimeHandlerTool::new,
    )?;
    register_tool(registry.clone(), "diff_calculator", DiffCalculatorTool::new)?;
    register_tool(registry.clone(), "hash_calculator", || {
        HashCalculatorTool::new(Default::default())
    })?;
    register_tool(registry.clone(), "template_engine", TemplateEngineTool::new)?;
    register_tool(registry.clone(), "text_manipulator", || {
        TextManipulatorTool::new(Default::default())
    })?;
    register_tool(registry, "uuid_generator", || {
        UuidGeneratorTool::new(Default::default())
    })?;
    Ok(())
}

/// Register data processing tools
fn register_data_processing_tools(
    registry: Arc<ComponentRegistry>,
) -> Result<(), Box<dyn std::error::Error>> {
    register_tool(registry.clone(), "csv_analyzer", || {
        CsvAnalyzerTool::new(Default::default())
    })?;
    register_tool(registry.clone(), "json_processor", || {
        JsonProcessorTool::new(Default::default())
    })?;
    register_tool_result(registry.clone(), "graphql_query", || {
        GraphQLQueryTool::new(Default::default())
    })?;
    register_tool_result(registry, "http_request", || {
        HttpRequestTool::new(Default::default())
    })?;
    Ok(())
}

/// Register file system tools
fn register_file_system_tools(
    registry: Arc<ComponentRegistry>,
    file_sandbox: Arc<FileSandbox>,
) -> Result<(), Box<dyn std::error::Error>> {
    register_tool(registry.clone(), "archive_handler", ArchiveHandlerTool::new)?;

    // File converter with sandbox
    let file_sandbox_converter = file_sandbox.clone();
    register_tool_with_sandbox(
        registry.clone(),
        "file_converter",
        file_sandbox_converter.clone(),
        move || FileConverterTool::new(Default::default(), file_sandbox_converter),
    )?;

    register_tool(registry.clone(), "file_operations", || {
        FileOperationsTool::new(Default::default())
    })?;

    // File search with sandbox
    let file_sandbox_search = file_sandbox.clone();
    register_tool_with_sandbox(
        registry.clone(),
        "file_search",
        file_sandbox_search.clone(),
        move || FileSearchTool::new(Default::default(), file_sandbox_search),
    )?;

    // File watcher with sandbox
    let file_sandbox_watcher = file_sandbox;
    register_tool_with_sandbox(
        registry,
        "file_watcher",
        file_sandbox_watcher.clone(),
        move || FileWatcherTool::new(Default::default(), file_sandbox_watcher),
    )?;
    Ok(())
}

/// Register system integration tools
fn register_system_tools(
    registry: Arc<ComponentRegistry>,
) -> Result<(), Box<dyn std::error::Error>> {
    register_tool(registry.clone(), "environment_reader", || {
        EnvironmentReaderTool::new(Default::default())
    })?;
    register_tool(registry.clone(), "process_executor", || {
        ProcessExecutorTool::new(Default::default())
    })?;
    register_tool(registry.clone(), "service_checker", || {
        ServiceCheckerTool::new(Default::default())
    })?;
    register_tool(registry, "system_monitor", || {
        SystemMonitorTool::new(Default::default())
    })?;
    Ok(())
}

/// Register media processing tools
fn register_media_tools(
    registry: Arc<ComponentRegistry>,
) -> Result<(), Box<dyn std::error::Error>> {
    register_tool(registry.clone(), "audio_processor", || {
        AudioProcessorTool::new(Default::default())
    })?;
    register_tool(registry.clone(), "image_processor", || {
        ImageProcessorTool::new(Default::default())
    })?;
    register_tool(registry, "video_processor", || {
        VideoProcessorTool::new(Default::default())
    })?;
    Ok(())
}

/// Register search tools
fn register_search_tools(
    registry: Arc<ComponentRegistry>,
) -> Result<(), Box<dyn std::error::Error>> {
    register_tool_result(registry, "web_search", || {
        WebSearchTool::new(Default::default())
    })?;
    Ok(())
}

/// Register web tools
fn register_web_tools(registry: Arc<ComponentRegistry>) -> Result<(), Box<dyn std::error::Error>> {
    register_tool(registry.clone(), "url-analyzer", UrlAnalyzerTool::new)?;
    register_tool(registry.clone(), "web-scraper", || {
        WebScraperTool::new(Default::default())
    })?;
    register_tool(registry.clone(), "api-tester", ApiTesterTool::new)?;
    register_tool(registry.clone(), "webhook-caller", WebhookCallerTool::new)?;
    register_tool(registry.clone(), "webpage-monitor", WebpageMonitorTool::new)?;
    register_tool(registry, "sitemap-crawler", SitemapCrawlerTool::new)?;
    Ok(())
}

/// Register communication tools
fn register_communication_tools(
    registry: Arc<ComponentRegistry>,
) -> Result<(), Box<dyn std::error::Error>> {
    register_tool_result(registry.clone(), "email-sender", || {
        EmailSenderTool::new(Default::default())
    })?;
    register_tool_result(registry, "database-connector", || {
        DatabaseConnectorTool::new(Default::default())
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
    pub category: String,
    /// Required parameters
    pub required_params: Vec<String>,
    /// Optional parameters
    pub optional_params: Vec<String>,
    /// Security requirements
    pub requires_file_access: bool,
    pub requires_network_access: bool,
    pub requires_process_spawn: bool,
}

/// Tool discovery service
pub struct ToolDiscovery {
    /// Component registry
    registry: Arc<ComponentRegistry>,
    /// Cached tool information
    tool_info_cache: HashMap<String, ToolInfo>,
}

impl ToolDiscovery {
    /// Create a new tool discovery service
    pub fn new(registry: Arc<ComponentRegistry>) -> Self {
        let mut tool_info_cache = HashMap::new();

        // Populate tool information
        // Utility tools
        tool_info_cache.insert(
            "base64_encoder".to_string(),
            ToolInfo {
                name: "base64_encoder".to_string(),
                description: "Encode and decode Base64 data".to_string(),
                category: "utility".to_string(),
                required_params: vec!["operation".to_string(), "input".to_string()],
                optional_params: vec!["encoding".to_string()],
                requires_file_access: false,
                requires_network_access: false,
                requires_process_spawn: false,
            },
        );

        tool_info_cache.insert(
            "calculator".to_string(),
            ToolInfo {
                name: "calculator".to_string(),
                description: "Perform mathematical calculations".to_string(),
                category: "utility".to_string(),
                required_params: vec!["expression".to_string()],
                optional_params: vec!["precision".to_string()],
                requires_file_access: false,
                requires_network_access: false,
                requires_process_spawn: false,
            },
        );

        // Data processing tools
        tool_info_cache.insert(
            "csv_analyzer".to_string(),
            ToolInfo {
                name: "csv_analyzer".to_string(),
                description: "Analyze and process CSV data".to_string(),
                category: "data_processing".to_string(),
                required_params: vec!["operation".to_string(), "input".to_string()],
                optional_params: vec!["delimiter".to_string(), "headers".to_string()],
                requires_file_access: false,
                requires_network_access: false,
                requires_process_spawn: false,
            },
        );

        tool_info_cache.insert(
            "json_processor".to_string(),
            ToolInfo {
                name: "json_processor".to_string(),
                description: "Process and transform JSON data".to_string(),
                category: "data_processing".to_string(),
                required_params: vec!["operation".to_string(), "input".to_string()],
                optional_params: vec!["expression".to_string(), "format".to_string()],
                requires_file_access: false,
                requires_network_access: false,
                requires_process_spawn: false,
            },
        );

        // File system tools
        tool_info_cache.insert(
            "file_operations".to_string(),
            ToolInfo {
                name: "file_operations".to_string(),
                description: "Perform file system operations".to_string(),
                category: "file_system".to_string(),
                required_params: vec!["operation".to_string(), "path".to_string()],
                optional_params: vec!["content".to_string(), "target_path".to_string()],
                requires_file_access: true,
                requires_network_access: false,
                requires_process_spawn: false,
            },
        );

        // Web tools
        tool_info_cache.insert(
            "web_search".to_string(),
            ToolInfo {
                name: "web_search".to_string(),
                description: "Search the web for information".to_string(),
                category: "web".to_string(),
                required_params: vec!["query".to_string()],
                optional_params: vec!["limit".to_string(), "language".to_string()],
                requires_file_access: false,
                requires_network_access: true,
                requires_process_spawn: false,
            },
        );

        Self {
            registry,
            tool_info_cache,
        }
    }

    /// Get information about a specific tool
    pub fn get_tool_info(&self, tool_name: &str) -> Option<ToolInfo> {
        self.tool_info_cache.get(tool_name).cloned()
    }

    /// List all available tool names
    pub fn list_tool_names(&self) -> Vec<String> {
        self.registry.list_tools()
    }

    /// Get tools by category
    pub fn get_tools_by_category(&self, category: &str) -> Vec<(String, ToolInfo)> {
        self.tool_info_cache
            .iter()
            .filter(|(_, info)| info.category == category)
            .map(|(name, info)| (name.clone(), info.clone()))
            .collect()
    }
}

/// Implementation of unified BridgeDiscovery trait for ToolDiscovery
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
        self.tool_info_cache
            .iter()
            .filter(|(name, info)| predicate(name, info))
            .map(|(name, info)| (name.clone(), info.clone()))
            .collect()
    }
}
