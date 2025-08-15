//! ABOUTME: llmspell-tools implementation crate
//! ABOUTME: Built-in tools library with registry, security sandbox, and tool implementations

pub mod api;
pub mod api_key_integration;
pub mod communication;
/// Data processing and transformation tools
pub mod data;
pub mod fs;
pub mod lifecycle;
pub mod media;
pub mod registry;
pub mod resource_limited;
pub mod search;
/// State management and persistence tools
pub mod state;
pub mod system;
pub mod util;
pub mod web;

// Re-export main types
pub use registry::{
    CapabilityMatcher, RegistryStatistics, ResourceUsageStats, ToolInfo, ToolRegistry,
};
pub use resource_limited::{ResourceLimitExt, ResourceLimited, ResourceLimitedTool};

// Re-export lifecycle components
pub use lifecycle::{
    ExecutionMetrics, HookableToolExecution, ToolExecutionState, ToolExecutor, ToolHookContext,
    ToolLifecycleConfig, ToolStateMachine,
};

// Re-export state persistence components
pub use state::{
    CachedResult, RegistryStatistics as ToolRegistryStatistics,
    ResourceUsageStats as ToolResourceUsageStats, ToolExecutionStats, ToolState,
    ToolStateManagerHolder, ToolStatePersistence, ToolStateRegistry,
};

// Re-export tools
pub use api::{GraphQLQueryTool, HttpRequestTool};
pub use communication::{DatabaseConnectorTool, EmailSenderTool};
pub use data::{CsvAnalyzerTool, JsonProcessorTool};
pub use fs::{
    ArchiveHandlerTool, FileConverterTool, FileOperationsTool, FileSearchTool, FileWatcherTool,
};
pub use media::{AudioProcessorTool, ImageProcessorTool, VideoProcessorTool};
pub use search::WebSearchTool;
pub use system::{
    EnvironmentReaderTool, ProcessExecutorTool, ServiceCheckerTool, SystemMonitorTool,
};
pub use util::{
    Base64EncoderTool, CalculatorTool, DataValidationTool, DateTimeHandlerTool, DiffCalculatorTool,
    HashCalculatorTool, TemplateEngineTool, TextManipulatorTool, UuidGeneratorTool,
};
pub use web::{
    ApiTesterTool, SitemapCrawlerTool, UrlAnalyzerTool, WebScraperTool, WebhookCallerTool,
    WebpageMonitorTool,
};
