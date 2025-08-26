//! ABOUTME: Code generation agent example demonstrating automated code creation and validation
//! ABOUTME: Shows how agents can generate, test, and refine code based on specifications

use llmspell_agents::templates::{AgentTemplate, TemplateInstantiationParams, ToolAgentTemplate};
use llmspell_core::types::AgentInput;
use llmspell_core::ExecutionContext;
use std::time::Instant;
use tracing::{info, Level};

/// Example demonstrating a code generation agent that creates code,
/// validates it, runs tests, and iteratively improves based on feedback.
#[tokio::main]
#[allow(clippy::too_many_lines)] // Example code with comprehensive demonstration
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::fmt().with_max_level(Level::INFO).init();

    info!("Starting Code Generation Agent Example");

    // Create a code generation agent with development tools
    let codegen_template = ToolAgentTemplate::new();
    let codegen_params = TemplateInstantiationParams::new("codegen-001".to_string())
        .with_parameter("agent_name", "Code Generator".into())
        .with_parameter(
            "allowed_tools",
            vec![
                "file_operations",
                "process_executor",
                "text_manipulator",
                "template_engine",
                "diff_calculator",
                "json_processor",
                "data_validation",
            ]
            .into(),
        )
        .with_parameter("tool_selection_strategy", "task_based".into())
        .with_parameter("enable_iterative_refinement", true.into())
        .with_parameter("max_iterations", 3.into())
        .with_parameter("test_driven_development", true.into());

    let codegen_result = codegen_template.instantiate(codegen_params).await?;
    let codegen = codegen_result.agent;

    // Example 1: Function Generation
    println!("\n=== Example 1: Function Generation ===");

    let function_spec = AgentInput::text(
        "Generate a Rust function with the following specification:\n\
         - Name: calculate_fibonacci\n\
         - Input: n: u32\n\
         - Output: Vec<u32> containing first n Fibonacci numbers\n\
         - Include error handling for edge cases\n\
         - Add comprehensive documentation\n\
         - Write unit tests",
    );

    let start = Instant::now();
    let function_output = codegen
        .execute(function_spec, ExecutionContext::default())
        .await?;
    println!("Generated Code:\n{}", function_output.text);
    println!("Generation Time: {:?}", start.elapsed());

    // Example 2: API Endpoint Generation
    println!("\n=== Example 2: API Endpoint Generation ===");

    let api_spec = AgentInput::text(
        "Generate a REST API endpoint:\n\
         - Endpoint: POST /api/users\n\
         - Framework: Actix-web\n\
         - Request body: JSON with name, email, age\n\
         - Validation: Email format, age > 0\n\
         - Response: Created user with ID\n\
         - Include error responses\n\
         - Add integration tests",
    );

    let api_output = codegen
        .execute(api_spec, ExecutionContext::default())
        .await?;
    println!("API Endpoint Code:\n{}", api_output.text);

    // Example 3: Data Structure Generation
    println!("\n=== Example 3: Data Structure Generation ===");

    let struct_spec = AgentInput::text(
        "Generate Rust data structures for a task management system:\n\
         - Task: id, title, description, status, priority, due_date\n\
         - Status: enum (Todo, InProgress, Done)\n\
         - Priority: enum (Low, Medium, High, Critical)\n\
         - Implement Display, Serialize, Deserialize traits\n\
         - Add builder pattern\n\
         - Include validation methods",
    );

    let struct_output = codegen
        .execute(struct_spec, ExecutionContext::default())
        .await?;
    println!("Generated Data Structures:\n{}", struct_output.text);

    // Example 4: Test Generation
    println!("\n=== Example 4: Test Generation ===");

    let test_spec = AgentInput::text(
        "Generate comprehensive tests for existing code:\n\
         - Read the module at src/utils/validator.rs\n\
         - Analyze all public functions\n\
         - Generate unit tests with:\n\
           - Happy path cases\n\
           - Edge cases\n\
           - Error cases\n\
           - Property-based tests where applicable\n\
         - Achieve >90% code coverage",
    );

    let test_output = codegen
        .execute(test_spec, ExecutionContext::default())
        .await?;
    println!("Generated Tests:\n{}", test_output.text);

    // Example 5: Code Refactoring
    println!("\n=== Example 5: Code Refactoring ===");

    let refactor_spec = AgentInput::text(
        "Refactor the following code patterns:\n\
         - Find all instances of manual error handling\n\
         - Replace with ? operator where appropriate\n\
         - Extract common patterns into helper functions\n\
         - Improve variable naming for clarity\n\
         - Add missing documentation\n\
         - Ensure clippy warnings are resolved",
    );

    let refactor_output = codegen
        .execute(refactor_spec, ExecutionContext::default())
        .await?;
    println!("Refactored Code:\n{}", refactor_output.text);

    // Example 6: CLI Tool Generation
    println!("\n=== Example 6: CLI Tool Generation ===");

    let cli_spec = AgentInput::text(
        "Generate a CLI tool using clap:\n\
         - Name: file-organizer\n\
         - Commands:\n\
           - organize: Sort files by type into folders\n\
           - clean: Remove duplicate files\n\
           - archive: Compress old files\n\
         - Global flags: --dry-run, --verbose\n\
         - Include progress bars\n\
         - Add configuration file support",
    );

    let cli_output = codegen
        .execute(cli_spec, ExecutionContext::default())
        .await?;
    println!("CLI Tool Code:\n{}", cli_output.text);

    // Example 7: Documentation Generation
    println!("\n=== Example 7: Documentation Generation ===");

    let doc_spec = AgentInput::text(
        "Generate documentation for a codebase:\n\
         - Analyze all public APIs in the project\n\
         - Generate:\n\
           - API reference documentation\n\
           - Usage examples for each module\n\
           - Architecture overview\n\
           - Getting started guide\n\
         - Format as Markdown\n\
         - Include code snippets",
    );

    let doc_output = codegen
        .execute(doc_spec, ExecutionContext::default())
        .await?;
    println!("Generated Documentation:\n{}", doc_output.text);

    // Code Generation Patterns
    println!("\n=== Code Generation Patterns ===");
    println!("1. **Template-Based**: Use templates for common patterns");
    println!("2. **AST Manipulation**: Work with abstract syntax trees");
    println!("3. **Incremental Generation**: Build code step by step");
    println!("4. **Test-Driven**: Generate tests first, then implementation");
    println!("5. **Validation Loop**: Generate, validate, refine");

    // Best Practices
    println!("\n=== Code Generation Best Practices ===");
    println!("1. **Clear Specifications**: Detailed requirements produce better code");
    println!("2. **Iterative Refinement**: Multiple passes improve quality");
    println!("3. **Style Consistency**: Match existing codebase conventions");
    println!("4. **Error Handling**: Always include proper error handling");
    println!("5. **Documentation**: Generate docs alongside code");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_testing::environment_helpers::create_test_context;

    #[tokio::test]
    async fn test_codegen_agent_creation() {
        let template = ToolAgentTemplate::new();
        let params = TemplateInstantiationParams::new("test-codegen".to_string())
            .with_parameter("agent_name", "Test Code Generator".into())
            .with_parameter(
                "allowed_tools",
                vec!["file_operations", "text_manipulator"].into(),
            );

        let result = template.instantiate(params).await.unwrap();
        assert!(!result.agent.metadata().name.is_empty());
    }

    #[tokio::test]
    async fn test_function_generation() {
        let template = ToolAgentTemplate::new();
        let params = TemplateInstantiationParams::new("func-gen".to_string())
            .with_parameter("agent_name", "Function Generator".into())
            .with_parameter("allowed_tools", vec!["text_manipulator"].into());

        let result = template.instantiate(params).await.unwrap();
        let agent = result.agent;

        let input = AgentInput::text("Generate a simple add function");
        let output = agent.execute(input, create_test_context()).await.unwrap();

        assert!(!output.text.is_empty());
    }

    #[tokio::test]
    async fn test_test_generation() {
        let template = ToolAgentTemplate::new();
        let params = TemplateInstantiationParams::new("test-gen".to_string())
            .with_parameter("agent_name", "Test Generator".into())
            .with_parameter(
                "allowed_tools",
                vec!["file_operations", "text_manipulator"].into(),
            )
            .with_parameter("test_driven_development", true.into());

        let result = template.instantiate(params).await.unwrap();
        let agent = result.agent;

        let input = AgentInput::text("Generate unit tests for a calculator module");
        let output = agent.execute(input, create_test_context()).await.unwrap();

        assert!(!output.text.is_empty());
    }

    #[tokio::test]
    async fn test_refactoring() {
        let template = ToolAgentTemplate::new();
        let params = TemplateInstantiationParams::new("refactor-agent".to_string())
            .with_parameter("agent_name", "Code Refactorer".into())
            .with_parameter(
                "allowed_tools",
                vec!["file_operations", "diff_calculator"].into(),
            );

        let result = template.instantiate(params).await.unwrap();
        let agent = result.agent;

        let input = AgentInput::text("Refactor code to improve readability");
        let output = agent.execute(input, create_test_context()).await.unwrap();

        assert!(!output.text.is_empty());
    }

    #[tokio::test]
    async fn test_iterative_generation() {
        let template = ToolAgentTemplate::new();
        let params = TemplateInstantiationParams::new("iterative-gen".to_string())
            .with_parameter("agent_name", "Iterative Generator".into())
            .with_parameter(
                "allowed_tools",
                vec!["text_manipulator", "process_executor"].into(),
            )
            .with_parameter("enable_iterative_refinement", true.into())
            .with_parameter("max_iterations", 2.into());

        let result = template.instantiate(params).await.unwrap();
        let agent = result.agent;

        let input = AgentInput::text("Generate and refine a sorting algorithm");
        let output = agent.execute(input, create_test_context()).await.unwrap();

        assert!(!output.text.is_empty());
    }
}
