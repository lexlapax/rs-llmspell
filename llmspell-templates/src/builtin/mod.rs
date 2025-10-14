//! Built-in template implementations
//!
//! This module contains all 6 production templates:
//! 1. Research Assistant (4-phase: gather → ingest → synthesize → validate)
//! 2. Interactive Chat (session-based conversation)
//! 3. Data Analysis (stats + visualization)
//! 4. Code Generator (spec → impl → test)
//! 5. Document Processor (PDF/OCR + transformation)
//! 6. Workflow Orchestrator (custom patterns)

pub mod code_generator;
pub mod data_analysis;
pub mod document_processor;
pub mod interactive_chat;
pub mod research_assistant;
pub mod workflow_orchestrator;

// Re-exports
pub use code_generator::CodeGeneratorTemplate;
pub use data_analysis::DataAnalysisTemplate;
pub use document_processor::DocumentProcessorTemplate;
pub use interactive_chat::InteractiveChatTemplate;
pub use research_assistant::ResearchAssistantTemplate;
pub use workflow_orchestrator::WorkflowOrchestratorTemplate;

/// Register built-in templates with the registry
///
/// Registers all 6 production templates:
/// - Research Assistant (Phase 12.3) - Multi-source research with RAG and citations
/// - Interactive Chat (Phase 12.4.1) - Session-based conversation with tool integration
/// - Data Analysis (Phase 12.4.2) - Statistical analysis and visualization
/// - Code Generator (Phase 12.4.3) - Specification, implementation, and testing
/// - Document Processor (Phase 12.4.4) - PDF/OCR extraction and transformation
/// - Workflow Orchestrator (Phase 12.4.4) - Custom agent/tool composition patterns
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

    // Register Code Generator (Phase 12.4.3)
    registry.register(std::sync::Arc::new(CodeGeneratorTemplate::new()))?;

    // Register Document Processor (Phase 12.4.4)
    registry.register(std::sync::Arc::new(DocumentProcessorTemplate::new()))?;

    // Register Workflow Orchestrator (Phase 12.4.4)
    registry.register(std::sync::Arc::new(WorkflowOrchestratorTemplate::new()))?;

    tracing::info!("Registered 6 built-in templates");
    Ok(())
}
