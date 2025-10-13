//! Built-in template implementations
//!
//! This module contains all 6 production templates:
//! 1. Research Assistant (4-phase: gather → ingest → synthesize → validate)
//! 2. Interactive Chat (session-based conversation)
//! 3. Data Analysis (stats + visualization)
//! 4. Code Generator (spec → impl → test)
//! 5. Document Processor (PDF/OCR + transformation)
//! 6. Workflow Orchestrator (custom patterns)

pub mod research_assistant;
pub mod interactive_chat;
pub mod data_analysis;

// Modules to be implemented:
// pub mod code_generator;
// pub mod document_processor;
// pub mod workflow_orchestrator;

// Re-exports
pub use research_assistant::ResearchAssistantTemplate;
pub use interactive_chat::InteractiveChatTemplate;
pub use data_analysis::DataAnalysisTemplate;

// Re-exports to be added:
// pub use code_generator::CodeGeneratorTemplate;
// pub use document_processor::DocumentProcessorTemplate;
// pub use workflow_orchestrator::WorkflowOrchestratorTemplate;

/// Register built-in templates with the registry
///
/// Currently registers:
/// - Research Assistant (Phase 12.3)
/// - Interactive Chat (Phase 12.4.1)
/// - Data Analysis (Phase 12.4.2)
///
/// TODO: Add remaining templates in Phase 12.4
pub fn register_builtin_templates(
    registry: &crate::registry::TemplateRegistry,
) -> crate::error::Result<()> {
    tracing::info!("Registering built-in templates");

    // Register Research Assistant (Phase 12.3.1)
    registry.register(std::sync::Arc::new(ResearchAssistantTemplate::new()))?;

    // Register Interactive Chat (Phase 12.4.1)
    registry.register(std::sync::Arc::new(InteractiveChatTemplate::new()))?;

    // Register Data Analysis (Phase 12.4.2)
    registry.register(std::sync::Arc::new(DataAnalysisTemplate::new()))?;

    tracing::info!("Registered 3 built-in templates");
    Ok(())
}
