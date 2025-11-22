//! ABOUTME: llmspell-tools implementation crate
//! ABOUTME: Built-in tools library with registry, security sandbox, and tool implementations
//!
//! # Builder Pattern for Tool Construction
//!
//! This crate demonstrates fluent builder patterns for constructing configurable tools.
//! The builder pattern provides a clean, type-safe way to create tools with optional
//! configuration while maintaining compile-time guarantees.
//!
//! ## Basic Builder Pattern
//!
//! Create a simple tool using builder pattern with method chaining:
//!
//! ```
//! use llmspell_core::{
//!     ComponentMetadata, ExecutionContext,
//!     types::{AgentInput, AgentOutput},
//!     traits::{base_agent::BaseAgent, tool::{Tool, ToolCategory, SecurityLevel, ToolSchema}},
//!     LLMSpellError
//! };
//! use async_trait::async_trait;
//! use llmspell_core::Result;
//!
//! // Tool configuration structure
//! #[derive(Debug, Clone)]
//! pub struct ToolConfig {
//!     pub timeout_ms: Option<u64>,
//!     pub cache_enabled: bool,
//! }
//!
//! impl Default for ToolConfig {
//!     fn default() -> Self {
//!         Self {
//!             timeout_ms: Some(5000),
//!             cache_enabled: true,
//!         }
//!     }
//! }
//!
//! // Configurable tool
//! #[derive(Debug)]
//! pub struct ConfigurableTool {
//!     metadata: ComponentMetadata,
//!     config: ToolConfig,
//! }
//!
//! // Builder for the tool
//! #[derive(Debug)]
//! pub struct ConfigurableToolBuilder {
//!     name: String,
//!     config: ToolConfig,
//! }
//!
//! impl ConfigurableToolBuilder {
//!     pub fn new(name: String) -> Self {
//!         Self {
//!             name,
//!             config: ToolConfig::default(),
//!         }
//!     }
//!
//!     pub fn timeout(mut self, ms: u64) -> Self {
//!         self.config.timeout_ms = Some(ms);
//!         self
//!     }
//!
//!     pub fn cache(mut self, enabled: bool) -> Self {
//!         self.config.cache_enabled = enabled;
//!         self
//!     }
//!
//!     pub fn build(self) -> std::result::Result<ConfigurableTool, String> {
//!         if self.name.is_empty() {
//!             return Err("Tool name cannot be empty".to_string());
//!         }
//!
//!         Ok(ConfigurableTool {
//!             metadata: ComponentMetadata::new(
//!                 self.name.clone(),
//!                 format!("Configurable tool '{}'", self.name),
//!             ),
//!             config: self.config,
//!         })
//!     }
//! }
//!
//! impl ConfigurableTool {
//!     pub fn builder(name: String) -> ConfigurableToolBuilder {
//!         ConfigurableToolBuilder::new(name)
//!     }
//! }
//!
//! #[async_trait]
//! impl BaseAgent for ConfigurableTool {
//!     fn metadata(&self) -> &ComponentMetadata {
//!         &self.metadata
//!     }
//!
//!     async fn execute_impl(
//!         &self,
//!         input: AgentInput,
//!         _context: ExecutionContext,
//!     ) -> Result<AgentOutput> {
//!         Ok(AgentOutput::text(format!("Processed: {}", input.text)))
//!     }
//!
//!     async fn validate_input(&self, _input: &AgentInput) -> Result<()> {
//!         Ok(())
//!     }
//!
//!     async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
//!         Err(error)
//!     }
//! }
//!
//! #[async_trait]
//! impl Tool for ConfigurableTool {
//!     fn category(&self) -> ToolCategory {
//!         ToolCategory::Utility
//!     }
//!
//!     fn security_level(&self) -> SecurityLevel {
//!         SecurityLevel::Safe
//!     }
//!
//!     fn schema(&self) -> ToolSchema {
//!         ToolSchema::new(
//!             self.metadata.name.clone(),
//!             self.metadata.description.clone(),
//!         )
//!     }
//! }
//!
//! // Usage: Build a tool with fluent API
//! let tool = ConfigurableTool::builder("my_tool".to_string())
//!     .timeout(3000)
//!     .cache(false)
//!     .build()
//!     .expect("Failed to build tool");
//!
//! assert_eq!(tool.metadata.name, "my_tool");
//! assert_eq!(tool.config.timeout_ms, Some(3000));
//! assert_eq!(tool.config.cache_enabled, false);
//! ```
//!
//! ## Builder Validation
//!
//! Builders can validate configuration during the build process:
//!
//! ```
//! # use llmspell_core::ComponentMetadata;
//! #
//! # #[derive(Debug, Clone)]
//! # pub struct ToolConfig {
//! #     pub timeout_ms: Option<u64>,
//! # }
//! #
//! # impl Default for ToolConfig {
//! #     fn default() -> Self {
//! #         Self { timeout_ms: Some(5000) }
//! #     }
//! # }
//! #
//! # #[derive(Debug)]
//! # pub struct ConfigurableTool {
//! #     metadata: ComponentMetadata,
//! #     config: ToolConfig,
//! # }
//! #
//! # #[derive(Debug)]
//! # pub struct ConfigurableToolBuilder {
//! #     name: String,
//! #     config: ToolConfig,
//! # }
//! #
//! # impl ConfigurableToolBuilder {
//! #     pub fn new(name: String) -> Self {
//! #         Self {
//! #             name,
//! #             config: ToolConfig::default(),
//! #         }
//! #     }
//! #
//! #     pub fn timeout(mut self, ms: u64) -> Self {
//! #         self.config.timeout_ms = Some(ms);
//! #         self
//! #     }
//! #
//! pub fn build(self) -> std::result::Result<ConfigurableTool, String> {
//!     // Validation: empty name
//!     if self.name.is_empty() {
//!         return Err("Tool name cannot be empty".to_string());
//!     }
//!
//!     // Validation: zero timeout
//!     if let Some(timeout) = self.config.timeout_ms {
//!         if timeout == 0 {
//!             return Err("Timeout must be greater than 0".to_string());
//!         }
//!     }
//!
//!     Ok(ConfigurableTool {
//!         metadata: ComponentMetadata::new(
//!             self.name.clone(),
//!             format!("Tool: {}", self.name),
//!         ),
//!         config: self.config,
//!     })
//! }
//! # }
//! #
//! # impl ConfigurableTool {
//! #     pub fn builder(name: String) -> ConfigurableToolBuilder {
//! #         ConfigurableToolBuilder::new(name)
//! #     }
//! # }
//!
//! // Test validation: empty name should fail
//! let result = ConfigurableTool::builder(String::new()).build();
//! assert!(result.is_err());
//! assert_eq!(result.unwrap_err(), "Tool name cannot be empty");
//!
//! // Test validation: zero timeout should fail
//! let result = ConfigurableTool::builder("test".to_string())
//!     .timeout(0)
//!     .build();
//! assert!(result.is_err());
//! assert_eq!(result.unwrap_err(), "Timeout must be greater than 0");
//! ```
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
