//! ABOUTME: Agent templates system for pre-configured agent patterns
//! ABOUTME: Provides schema, base traits, implementations, customization, and validation for agent templates

/// Template schema definitions
pub mod schema;

/// Base template trait and factory
pub mod base;

/// Tool agent template
pub mod tool_agent;

/// Orchestrator agent template  
pub mod orchestrator_agent;

/// Monitor agent template
pub mod monitor_agent;

/// Template customization API
pub mod customization;

/// Template validation utilities
pub mod validation;

// Re-export commonly used types
pub use base::{
    AgentTemplate, TemplateFactory, TemplateInstantiationParams, TemplateInstantiationResult,
};
pub use customization::{TemplateBuilder, TemplateCustomizer, TemplateMixin};
pub use monitor_agent::MonitorAgentTemplate;
pub use orchestrator_agent::OrchestratorAgentTemplate;
pub use schema::{
    CapabilityRequirement, ComplexityLevel, ParameterDefinition, ParameterType,
    ResourceRequirements, TemplateCategory, TemplateMetadata, TemplateSchema, ToolDependency,
};
pub use tool_agent::ToolAgentTemplate;
pub use validation::{TemplateValidator, ValidationError, ValidationResult, ValidationWarning};

/// Prelude for convenient imports
pub mod prelude {
    pub use super::base::{AgentTemplate, TemplateFactory, TemplateInstantiationParams};
    pub use super::customization::{TemplateCustomizer, TemplateMixin};
    pub use super::monitor_agent::MonitorAgentTemplate;
    pub use super::orchestrator_agent::OrchestratorAgentTemplate;
    pub use super::tool_agent::ToolAgentTemplate;
    pub use super::validation::TemplateValidator;
}

/// Create and register all built-in templates
///
/// # Panics
///
/// Panics if template registration fails (e.g., duplicate template IDs).
#[must_use]
pub fn create_builtin_templates() -> TemplateFactory {
    let mut factory = TemplateFactory::new();

    // Register tool agent templates
    factory
        .register_template(Box::new(ToolAgentTemplate::new()))
        .unwrap();
    factory
        .register_template(Box::new(ToolAgentTemplate::lightweight()))
        .unwrap();
    factory
        .register_template(Box::new(ToolAgentTemplate::batch_processor()))
        .unwrap();

    // Register orchestrator agent templates
    factory
        .register_template(Box::new(OrchestratorAgentTemplate::new()))
        .unwrap();
    factory
        .register_template(Box::new(OrchestratorAgentTemplate::simple()))
        .unwrap();
    factory
        .register_template(Box::new(OrchestratorAgentTemplate::enterprise()))
        .unwrap();
    factory
        .register_template(Box::new(OrchestratorAgentTemplate::event_driven()))
        .unwrap();

    // Register monitor agent templates
    factory
        .register_template(Box::new(MonitorAgentTemplate::new()))
        .unwrap();
    factory
        .register_template(Box::new(MonitorAgentTemplate::system_monitor()))
        .unwrap();
    factory
        .register_template(Box::new(MonitorAgentTemplate::application_monitor()))
        .unwrap();
    factory
        .register_template(Box::new(MonitorAgentTemplate::lightweight()))
        .unwrap();

    factory
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_builtin_templates() {
        let factory = create_builtin_templates();

        // Check that all templates are registered
        assert!(factory.template_count() >= 11); // At least 11 built-in templates

        // Check specific templates exist
        assert!(factory.has_template("tool_agent"));
        assert!(factory.has_template("orchestrator_agent"));
        assert!(factory.has_template("monitor_agent"));

        // Check categories have templates
        let tool_templates = factory.get_templates_by_category(&TemplateCategory::ToolExecution);
        assert!(!tool_templates.is_empty());

        let orchestrator_templates =
            factory.get_templates_by_category(&TemplateCategory::Orchestration);
        assert!(!orchestrator_templates.is_empty());

        let monitor_templates = factory.get_templates_by_category(&TemplateCategory::Monitoring);
        assert!(!monitor_templates.is_empty());
    }
    #[test]
    fn test_template_search() {
        let factory = create_builtin_templates();

        // Search for tool templates
        let found = factory.find_templates("tool");
        assert!(!found.is_empty());

        // Search for monitor templates
        let found = factory.find_templates("monitor");
        assert!(!found.is_empty());

        // Search for orchestrator templates
        let found = factory.find_templates("orchestr");
        assert!(!found.is_empty());
    }
}
