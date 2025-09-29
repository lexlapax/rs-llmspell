//! ABOUTME: llmspell-tools implementation crate
//! ABOUTME: Built-in tools library with registry, security sandbox, and tool implementations
#![allow(clippy::too_long_first_doc_paragraph)]

/// Academic and research tools
pub mod academic;
pub mod api;
pub mod api_key_integration;
/// Communication tools (conditional modules inside)
pub mod communication;
/// Data processing and transformation tools
pub mod data;
/// Document processing tools (conditional - PDF only)
#[cfg(feature = "pdf")]
pub mod document;
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

// Re-export tools (conditionally based on features)
pub use academic::CitationFormatterTool;
pub use api::{GraphQLQueryTool, HttpRequestTool};

// Communication tools (conditional)
#[cfg(feature = "database")]
pub use communication::DatabaseConnectorTool;
#[cfg(feature = "email")]
pub use communication::EmailSenderTool;

// Data tools (conditional)
#[cfg(feature = "csv-parquet")]
pub use data::CsvAnalyzerTool;
pub use data::GraphBuilderTool;
#[cfg(feature = "json-query")]
pub use data::JsonProcessorTool;

// Document tools (conditional)
#[cfg(feature = "pdf")]
pub use document::PdfProcessorTool;

// File system tools (conditional)
#[cfg(feature = "archives")]
pub use fs::ArchiveHandlerTool;
pub use fs::{FileConverterTool, FileOperationsTool, FileSearchTool, FileWatcherTool};

// Media tools (always available)
pub use media::{AudioProcessorTool, ImageProcessorTool, VideoProcessorTool};
pub use search::WebSearchTool;

// System tools (always available)
pub use system::{
    EnvironmentReaderTool, ProcessExecutorTool, ServiceCheckerTool, SystemMonitorTool,
};

// Utility tools (conditional)
#[cfg(feature = "templates")]
pub use util::TemplateEngineTool;
pub use util::{
    Base64EncoderTool, CalculatorTool, DataValidationTool, DateTimeHandlerTool, DiffCalculatorTool,
    HashCalculatorTool, TextManipulatorTool, UuidGeneratorTool,
};

// Web tools (always available)
pub use web::{
    ApiTesterTool, SitemapCrawlerTool, UrlAnalyzerTool, WebScraperTool, WebhookCallerTool,
    WebpageMonitorTool,
};
