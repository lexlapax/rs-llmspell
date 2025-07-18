//! ABOUTME: Core traits, types, and infrastructure for rs-llmspell
//! ABOUTME: Foundation layer providing BaseAgent, Agent, Tool, and Workflow traits
//!
//! # Overview
//!
//! `llmspell-core` is the foundational crate of the LLMSpell system, providing:
//!
//! - **Core Traits**: `BaseAgent`, `Agent`, `Tool`, and `Workflow` for component abstraction
//! - **Type System**: `ComponentId`, `Version`, and `ComponentMetadata` for identification
//! - **Error Handling**: Comprehensive error types with severity and retry logic
//! - **Logging**: Structured logging with JSON and human-readable formats
//!
//! # Architecture
//!
//! The crate follows a trait-based architecture where all components implement
//! `BaseAgent` as the foundation:
//!
//! ```text
//! BaseAgent (foundation trait)
//!     ├── Agent (LLM-powered components)
//!     ├── Tool (functional components)  
//!     └── Workflow (orchestration)
//! ```
//!
//! # Getting Started
//!
//! ## Creating a Simple Agent
//!
//! ```
//! use llmspell_core::{
//!     ComponentMetadata, Result,
//!     traits::{
//!         base_agent::BaseAgent,
//!         agent::{Agent, AgentConfig, ConversationMessage}
//!     },
//!     types::{AgentInput, AgentOutput, ExecutionContext}
//! };
//! use async_trait::async_trait;
//!
//! struct MyAgent {
//!     metadata: ComponentMetadata,
//!     config: AgentConfig,
//! }
//!
//! impl MyAgent {
//!     fn new(name: String) -> Self {
//!         Self {
//!             metadata: ComponentMetadata::new(name, "My custom agent".to_string()),
//!             config: AgentConfig::default(),
//!         }
//!     }
//! }
//!
//! #[async_trait]
//! impl BaseAgent for MyAgent {
//!     fn metadata(&self) -> &ComponentMetadata {
//!         &self.metadata
//!     }
//!     
//!     async fn execute(
//!         &self,
//!         input: AgentInput,
//!         context: ExecutionContext,
//!     ) -> Result<AgentOutput> {
//!         // Your agent logic here
//!         Ok(AgentOutput::text(format!("Processed: {}", input.text)))
//!     }
//!     
//!     async fn validate_input(&self, input: &AgentInput) -> Result<()> {
//!         Ok(())
//!     }
//!     
//!     async fn handle_error(&self, error: llmspell_core::LLMSpellError) -> Result<AgentOutput> {
//!         Ok(AgentOutput::text(format!("Error: {}", error)))
//!     }
//! }
//! ```
//!
//! ## Error Handling
//!
//! ```
//! use llmspell_core::{LLMSpellError, Result, component_error};
//!
//! fn process_data(data: &str) -> Result<String> {
//!     if data.is_empty() {
//!         return Err(LLMSpellError::Validation {
//!             message: "Data cannot be empty".to_string(),
//!             field: Some("data".to_string()),
//!         });
//!     }
//!     
//!     // Or use convenience macros
//!     if data.len() > 1000 {
//!         return Err(component_error!("Data too large"));
//!     }
//!     
//!     Ok(data.to_uppercase())
//! }
//! ```
//!
//! ## Logging
//!
//! ```no_run
//! use llmspell_core::logging::{init_from_env, info, debug};
//!
//! // Initialize logging
//! init_from_env().expect("Failed to init logging");
//!
//! // Use standard tracing macros
//! info!("Application started");
//! debug!(user = "alice", "Processing request");
//! ```

pub mod error;
pub mod logging;
pub mod types;

pub mod traits {
    pub mod agent;
    pub mod base_agent;
    pub mod tool;
    pub mod tool_capable;
    pub mod workflow;
}

// Re-export commonly used types
pub use error::{LLMSpellError, Result};
pub use traits::{
    agent::Agent, base_agent::BaseAgent, tool::Tool, tool_capable::ToolCapable, workflow::Workflow,
};
pub use types::{ComponentId, ComponentMetadata, Version};
