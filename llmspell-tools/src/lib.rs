//! ABOUTME: llmspell-tools implementation crate
//! ABOUTME: Built-in tools library with registry, security sandbox, and tool implementations

pub mod api;
pub mod data;
pub mod fs;
pub mod media;
pub mod registry;
pub mod search;
pub mod system;
pub mod util;
pub mod web;

// Re-export main types
pub use registry::{CapabilityMatcher, RegistryStatistics, ToolInfo, ToolRegistry};

// Re-export tools
pub use api::{GraphQLQueryTool, HttpRequestTool};
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
