//! # Production-Ready AI Agent Templates - Phase 12
//!
//! This crate provides turn-key AI workflow templates that solve the "0-day retention problem"
//! by offering immediate value post-installation. Templates combine agents, tools, RAG, and
//! LocalLLM into executable solutions without requiring users to architect workflows from scratch.
//!
//! ## Architecture
//!
//! ```text
//! Templates (User-Facing)
//! ├── Research Assistant (4-phase: gather → ingest → synthesize → validate)
//! ├── Interactive Chat (session-based conversation)
//! ├── Data Analysis (stats + visualization)
//! ├── Code Generator (spec → impl → test)
//! ├── Document Processor (PDF/OCR + transformation)
//! └── Workflow Orchestrator (custom patterns)
//! ```
//!
//! ## Distinction from Agent Templates
//!
//! - **llmspell-agents/templates**: Internal infrastructure patterns (ToolAgentTemplate, OrchestratorAgentTemplate)
//! - **llmspell-templates** (this crate): End-user workflow templates (ResearchAssistantTemplate, InteractiveChatTemplate)
//!
//! ## Usage
//!
//! ### CLI (Direct Execution)
//!
//! ```bash
//! # List available templates
//! llmspell template list
//!
//! # Get template info
//! llmspell template info research-assistant
//!
//! # Execute template
//! llmspell template exec research-assistant \
//!     --param topic="Rust async runtime design" \
//!     --param max_sources=15 \
//!     --param model="ollama/llama3.2:3b"
//!
//! # Search templates
//! llmspell template search "research"
//!
//! # Get parameter schema
//! llmspell template schema research-assistant
//! ```
//!
//! ### Lua API
//!
//! ```lua
//! -- List templates
//! local templates = Template.list()
//!
//! -- Get info
//! local info = Template.info("research-assistant")
//!
//! -- Execute template
//! local result = Template.execute("research-assistant", {
//!     topic = "Rust async runtime design",
//!     max_sources = 15,
//!     model = "ollama/llama3.2:3b"
//! })
//!
//! -- Access results
//! print(result.artifacts[1].content) -- synthesis.md
//! print(result.metrics.execution_time_ms)
//! ```
//!
//! ### Rust API
//!
//! ```rust,ignore
//! use llmspell_templates::{TemplateRegistry, ExecutionContext};
//! use serde_json::json;
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Get template registry
//! let registry = TemplateRegistry::with_builtin_templates()?;
//!
//! // Get template
//! let template = registry.get("research-assistant")?;
//!
//! // Execute
//! let params = json!({
//!     "topic": "Rust async runtime design",
//!     "max_sources": 15,
//!     "model": "ollama/llama3.2:3b"
//! });
//! let context = ExecutionContext::new(/* ... */);
//! let result = template.execute(params.into(), context).await?;
//!
//! // Access artifacts
//! for artifact in result.artifacts {
//!     println!("{}: {}", artifact.filename, artifact.content);
//! }
//! # Ok(())
//! # }
//! ```
//!
//! ## Performance Targets
//!
//! - Template execution overhead: <100ms
//! - Template discovery: <10ms
//! - Parameter validation: <5ms
//! - Competitive advantage: 10-100x faster than Python frameworks
//!
//! ## Phase 13 Memory Synergy
//!
//! Templates are designed for zero-breaking-change memory enhancement with A-TKG:
//!
//! ```rust,ignore
//! // Phase 12: Works without memory
//! let result = template.execute(params, context).await?;
//!
//! // Phase 13: Same template with memory (opt-in)
//! let mut params_with_memory = params.clone();
//! params_with_memory.insert("enable_memory", json!(true));
//! let result = template.execute(params_with_memory, context).await?;
//! ```

pub mod artifacts;
pub mod builtin;
pub mod context;
pub mod core;
pub mod error;
pub mod registry;
pub mod validation;

// Re-exports for convenience
pub use artifacts::Artifact;
pub use context::ExecutionContext;
pub use core::{
    CostEstimate, Template, TemplateCategory, TemplateMetadata, TemplateOutput, TemplateParams,
};
pub use error::{Result, TemplateError, ValidationError};
pub use registry::TemplateRegistry;
pub use validation::ConfigSchema;
