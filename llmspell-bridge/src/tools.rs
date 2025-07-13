//! ABOUTME: Tool registration and management for the bridge
//! ABOUTME: Initializes and provides access to all Phase 2 tools from llmspell-tools

use crate::ComponentRegistry;
use llmspell_core::traits::tool::{ResourceLimits, SecurityRequirements};
use llmspell_core::Tool;
use llmspell_security::sandbox::{file_sandbox::FileSandbox, SandboxContext};
use llmspell_tools::*;
use std::sync::Arc;

/// Initialize and register all Phase 2 tools with the bridge registry
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

    // Utility tools
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
    register_tool(registry.clone(), "uuid_generator", || {
        UuidGeneratorTool::new(Default::default())
    })?;

    // Data processing tools
    register_tool(registry.clone(), "csv_analyzer", || {
        CsvAnalyzerTool::new(Default::default())
    })?;
    register_tool(registry.clone(), "json_processor", || {
        JsonProcessorTool::new(Default::default())
    })?;
    register_tool_result(registry.clone(), "graphql_query", || {
        GraphQLQueryTool::new(Default::default())
    })?;
    register_tool_result(registry.clone(), "http_request", || {
        HttpRequestTool::new(Default::default())
    })?;

    // File system tools
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
    let file_sandbox_watcher = file_sandbox.clone();
    register_tool_with_sandbox(
        registry.clone(),
        "file_watcher",
        file_sandbox_watcher.clone(),
        move || FileWatcherTool::new(Default::default(), file_sandbox_watcher),
    )?;

    // System integration tools
    register_tool(registry.clone(), "environment_reader", || {
        EnvironmentReaderTool::new(Default::default())
    })?;
    register_tool(registry.clone(), "process_executor", || {
        ProcessExecutorTool::new(Default::default())
    })?;
    register_tool(registry.clone(), "service_checker", || {
        ServiceCheckerTool::new(Default::default())
    })?;
    register_tool(registry.clone(), "system_monitor", || {
        SystemMonitorTool::new(Default::default())
    })?;

    // Media processing tools
    register_tool(registry.clone(), "audio_processor", || {
        AudioProcessorTool::new(Default::default())
    })?;
    register_tool(registry.clone(), "image_processor", || {
        ImageProcessorTool::new(Default::default())
    })?;
    register_tool(registry.clone(), "video_processor", || {
        VideoProcessorTool::new(Default::default())
    })?;

    // Search tools
    register_tool_result(registry.clone(), "web_search", || {
        WebSearchTool::new(Default::default())
    })?;

    // Phase 3.1 Web tools
    register_tool(registry.clone(), "url-analyzer", UrlAnalyzerTool::new)?;
    register_tool(registry.clone(), "web-scraper", || {
        WebScraperTool::new(Default::default())
    })?;
    register_tool(registry.clone(), "api-tester", ApiTesterTool::new)?;
    register_tool(registry.clone(), "webhook-caller", WebhookCallerTool::new)?;
    register_tool(registry.clone(), "webpage-monitor", WebpageMonitorTool::new)?;
    register_tool(registry.clone(), "sitemap-crawler", SitemapCrawlerTool::new)?;

    // Phase 3.1 Communication tools
    register_tool_result(registry.clone(), "email-sender", || {
        EmailSenderTool::new(Default::default())
    })?;
    register_tool_result(registry.clone(), "database-connector", || {
        DatabaseConnectorTool::new(Default::default())
    })?;

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
pub fn get_all_tool_names(registry: Arc<ComponentRegistry>) -> Vec<String> {
    registry.list_tools()
}

/// Get a tool by name from the registry
pub fn get_tool_by_name(registry: Arc<ComponentRegistry>, name: &str) -> Option<Arc<dyn Tool>> {
    registry.get_tool(name)
}
