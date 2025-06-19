// ABOUTME: Integration tests for rs-llmspell library functionality
// ABOUTME: Tests the public API and end-to-end behavior of LLM agents and workflows

use rs_llmspell::{Agent, Workflow, StepType};

#[test]
fn test_agent_integration() {
    let agent = Agent::new()
        .with_model("gpt-4")
        .with_system("You are helpful");
    let result = agent.run("Hello world").unwrap();
    assert!(result.contains("Hello world"));
}

#[test]
fn test_agent_with_tools() {
    let agent = Agent::new()
        .with_tools(vec!["calculator", "web_search"]);
    let result = agent.run("Calculate 2+2").unwrap();
    assert!(result.contains("Calculate 2+2"));
}

#[test]
fn test_workflow_integration() {
    let agent = Agent::new().with_model("gpt-4");
    let workflow = Workflow::sequential()
        .add_step("analyze", StepType::Agent(agent))
        .add_step("save", StepType::Tool("file_writer".to_string()));
    
    let result = workflow.run("test input").unwrap();
    assert_eq!(result, "Workflow completed");
}