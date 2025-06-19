// ABOUTME: Main library module for rs-llmspell, a Rust library for scriptable LLM interactions
// ABOUTME: Provides core functionality for LLM agents, workflows, and automation

//! # rs-llmspell
//!
//! Cast scripting spells to animate LLM golems 🧙‍♂️✨
//! 
//! This library provides functionality for orchestrating LLM agents, tools, and workflows
//! with Rust's safety, performance, and reliability.
//!
//! ## Example
//!
//! ```rust
//! use rs_llmspell::Agent;
//!
//! let agent = Agent::new()
//!     .with_model("gpt-4")
//!     .with_system("You are a helpful assistant");
//! let response = agent.run("Explain quantum computing").unwrap();
//! println!("{}", response);
//! ```

use std::collections::HashMap;

/// Result type for rs-llmspell operations
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Configuration for LLM models
#[derive(Debug, Clone)]
pub struct ModelConfig {
    pub name: String,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

/// Main agent struct for LLM interactions
pub struct Agent {
    model: String,
    system_prompt: Option<String>,
    tools: Vec<String>,
    config: HashMap<String, String>,
}

impl Agent {
    /// Creates a new Agent instance
    pub fn new() -> Self {
        Self {
            model: "gpt-3.5-turbo".to_string(),
            system_prompt: None,
            tools: Vec::new(),
            config: HashMap::new(),
        }
    }

    /// Sets the model for this agent
    pub fn with_model(mut self, model: &str) -> Self {
        self.model = model.to_string();
        self
    }

    /// Sets the system prompt for this agent
    pub fn with_system(mut self, system: &str) -> Self {
        self.system_prompt = Some(system.to_string());
        self
    }

    /// Adds tools to this agent
    pub fn with_tools(mut self, tools: Vec<&str>) -> Self {
        self.tools = tools.into_iter().map(|s| s.to_string()).collect();
        self
    }

    /// Runs the agent with the given prompt
    pub fn run(&self, prompt: &str) -> Result<String> {
        // Implementation will be added here
        Ok(format!("Agent response to: {}", prompt))
    }
}

impl Default for Agent {
    fn default() -> Self {
        Self::new()
    }
}

/// Workflow orchestration for multiple agents and tools
pub struct Workflow {
    steps: Vec<WorkflowStep>,
}

/// A single step in a workflow
pub struct WorkflowStep {
    pub name: String,
    pub step_type: StepType,
}

/// Types of workflow steps
pub enum StepType {
    Agent(Agent),
    Tool(String),
}

impl Workflow {
    /// Creates a new empty workflow
    pub fn new() -> Self {
        Self { steps: Vec::new() }
    }

    /// Creates a sequential workflow
    pub fn sequential() -> Self {
        Self::new()
    }

    /// Adds a step to the workflow
    pub fn add_step(mut self, name: &str, step: StepType) -> Self {
        self.steps.push(WorkflowStep {
            name: name.to_string(),
            step_type: step,
        });
        self
    }

    /// Runs the workflow with the given input
    pub fn run(&self, _input: &str) -> Result<String> {
        // Implementation will be added here
        Ok("Workflow completed".to_string())
    }
}

impl Default for Workflow {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_creation() {
        let agent = Agent::new();
        assert_eq!(agent.model, "gpt-3.5-turbo");
    }

    #[test]
    fn test_agent_with_model() {
        let agent = Agent::new().with_model("gpt-4");
        assert_eq!(agent.model, "gpt-4");
    }

    #[test]
    fn test_agent_with_system() {
        let agent = Agent::new().with_system("You are helpful");
        assert_eq!(agent.system_prompt, Some("You are helpful".to_string()));
    }

    #[test]
    fn test_agent_run() {
        let agent = Agent::new();
        let result = agent.run("test prompt").unwrap();
        assert!(result.contains("test prompt"));
    }

    #[test]
    fn test_workflow_creation() {
        let workflow = Workflow::new();
        assert_eq!(workflow.steps.len(), 0);
    }

    #[test]
    fn test_workflow_add_step() {
        let agent = Agent::new();
        let workflow = Workflow::sequential()
            .add_step("test", StepType::Agent(agent));
        assert_eq!(workflow.steps.len(), 1);
        assert_eq!(workflow.steps[0].name, "test");
    }
}