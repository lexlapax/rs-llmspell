// ABOUTME: Unit tests for llmspell-core crate
// ABOUTME: Tests core traits, types, and utilities

//! Unit tests for llmspell-core functionality

#[cfg(test)]
mod tests {
    use llmspell_core::types::{AgentInput, AgentOutput};
    
    #[test]
    fn test_agent_input_creation() {
        let input = AgentInput::text("test input");
        assert_eq!(input.text, "test input");
    }
    
    #[test]
    fn test_agent_output_creation() {
        let output = AgentOutput::text("test output");
        assert_eq!(output.text, "test output");
    }
}