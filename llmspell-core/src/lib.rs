//! ABOUTME: Core traits, types, and infrastructure for rs-llmspell
//! ABOUTME: Foundation layer providing BaseAgent, Agent, Tool, and Workflow traits

pub mod error;
pub mod logging;
pub mod types;

pub mod traits {
    pub mod base_agent;
    pub mod agent;
    pub mod tool;
    pub mod workflow;
}

// Re-export commonly used types
pub use error::{LLMSpellError, Result};
pub use types::{ComponentId, ComponentMetadata, Version};
pub use traits::{
    agent::Agent,
    base_agent::BaseAgent, 
    tool::Tool,
    workflow::Workflow,
};