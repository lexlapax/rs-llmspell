// ABOUTME: Unit tests for llmspell-core crate
// ABOUTME: Tests core traits, types, and utilities

//! Unit tests for llmspell-core functionality

// TODO: Add test categorization when macro support is improved

#[cfg(test)]
mod core_type_tests {
    use llmspell_core::types::{AgentInput, AgentOutput};

    #[test]
    // TODO: Add category: unit
    fn test_agent_input_creation() {
        let input = AgentInput::text("test input");
        assert_eq!(input.text, "test input");
    }

    #[test]
    // TODO: Add category: unit
    fn test_agent_output_creation() {
        let output = AgentOutput::text("test output");
        assert_eq!(output.text, "test output");
    }

    #[test]
    // TODO: Add category: unit
    fn test_agent_input_with_parameters() {
        let input = AgentInput::text("test").with_parameter("key", serde_json::json!("value"));
        assert_eq!(input.text, "test");
        assert_eq!(input.parameters.get("key").unwrap(), "value");
    }
}
