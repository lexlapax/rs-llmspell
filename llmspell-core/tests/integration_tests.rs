//! Integration tests for llmspell-core
//! 
//! These tests verify that different components work together correctly

use llmspell_core::{
    ComponentMetadata, Version, LLMSpellError, Result,
    traits::{
        base_agent::{BaseAgent, AgentInput, AgentOutput, ExecutionContext},
        agent::{Agent, AgentConfig, ConversationMessage, MessageRole},
        tool::{Tool, ToolSchema, ToolCategory, SecurityLevel, ParameterDef, ParameterType},
    },
};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};

/// Test agent that maintains conversation state
struct TestAgent {
    metadata: ComponentMetadata,
    config: AgentConfig,
    conversation: Arc<Mutex<Vec<ConversationMessage>>>,
    execution_count: Arc<Mutex<usize>>,
}

impl TestAgent {
    fn new(name: &str) -> Self {
        Self {
            metadata: ComponentMetadata::new(
                name.to_string(),
                format!("Test agent: {}", name),
            ),
            config: AgentConfig::default(),
            conversation: Arc::new(Mutex::new(Vec::new())),
            execution_count: Arc::new(Mutex::new(0)),
        }
    }
}

#[async_trait]
impl BaseAgent for TestAgent {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }
    
    async fn execute(&self, input: AgentInput, _context: ExecutionContext) -> Result<AgentOutput> {
        // Validate input first
        self.validate_input(&input).await?;
        
        *self.execution_count.lock().unwrap() += 1;
        
        // Add to conversation
        let mut conv = self.conversation.lock().unwrap();
        conv.push(ConversationMessage::user(input.prompt.clone()));
        
        let response = format!("Processed: {}", input.prompt);
        conv.push(ConversationMessage::assistant(response.clone()));
        
        Ok(AgentOutput::new(response)
            .with_metadata("execution_count".to_string(), serde_json::json!(*self.execution_count.lock().unwrap())))
    }
    
    async fn validate_input(&self, input: &AgentInput) -> Result<()> {
        if input.prompt.is_empty() {
            return Err(LLMSpellError::Validation {
                message: "Prompt cannot be empty".to_string(),
                field: Some("prompt".to_string()),
            });
        }
        Ok(())
    }
    
    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
        Ok(AgentOutput::new(format!("Error handled: {}", error)))
    }
}

#[async_trait]
impl Agent for TestAgent {
    fn config(&self) -> &AgentConfig {
        &self.config
    }
    
    async fn get_conversation(&self) -> Result<Vec<ConversationMessage>> {
        Ok(self.conversation.lock().unwrap().clone())
    }
    
    async fn add_message(&mut self, message: ConversationMessage) -> Result<()> {
        self.conversation.lock().unwrap().push(message);
        Ok(())
    }
    
    async fn clear_conversation(&mut self) -> Result<()> {
        self.conversation.lock().unwrap().clear();
        Ok(())
    }
}

/// Test tool that performs string transformations
struct TestTool {
    metadata: ComponentMetadata,
    invocation_count: Arc<Mutex<usize>>,
}

impl TestTool {
    fn new(name: &str) -> Self {
        Self {
            metadata: ComponentMetadata::new(
                name.to_string(),
                format!("Test tool: {}", name),
            ),
            invocation_count: Arc::new(Mutex::new(0)),
        }
    }
}

#[async_trait]
impl BaseAgent for TestTool {
    fn metadata(&self) -> &ComponentMetadata {
        &self.metadata
    }
    
    async fn execute(&self, input: AgentInput, _context: ExecutionContext) -> Result<AgentOutput> {
        *self.invocation_count.lock().unwrap() += 1;
        
        // Parse parameters
        let params = input.get_context("params")
            .ok_or_else(|| LLMSpellError::Validation {
                message: "Missing params in context".to_string(),
                field: Some("params".to_string()),
            })?;
        
        let text = params.get("text")
            .and_then(|v| v.as_str())
            .ok_or_else(|| LLMSpellError::Validation {
                message: "Missing text parameter".to_string(),
                field: Some("text".to_string()),
            })?;
        
        let operation = params.get("operation")
            .and_then(|v| v.as_str())
            .unwrap_or("uppercase");
        
        let result = match operation {
            "uppercase" => text.to_uppercase(),
            "lowercase" => text.to_lowercase(),
            "reverse" => text.chars().rev().collect(),
            _ => text.to_string(),
        };
        
        Ok(AgentOutput::new(result))
    }
    
    async fn validate_input(&self, _input: &AgentInput) -> Result<()> {
        Ok(())
    }
    
    async fn handle_error(&self, error: LLMSpellError) -> Result<AgentOutput> {
        Ok(AgentOutput::new(format!("Tool error: {}", error)))
    }
}

#[async_trait]
impl Tool for TestTool {
    fn category(&self) -> ToolCategory {
        ToolCategory::Utility
    }
    
    fn security_level(&self) -> SecurityLevel {
        SecurityLevel::Safe
    }
    
    fn schema(&self) -> ToolSchema {
        ToolSchema::new(
            "string_transform".to_string(),
            "Transform strings in various ways".to_string(),
        )
        .with_parameter(ParameterDef {
            name: "text".to_string(),
            param_type: ParameterType::String,
            description: "Text to transform".to_string(),
            required: true,
            default: None,
        })
        .with_parameter(ParameterDef {
            name: "operation".to_string(),
            param_type: ParameterType::String,
            description: "Operation: uppercase, lowercase, reverse".to_string(),
            required: false,
            default: Some(serde_json::json!("uppercase")),
        })
    }
    
    async fn validate_parameters(&self, params: &serde_json::Value) -> Result<()> {
        if !params.is_object() {
            return Err(LLMSpellError::Validation {
                message: "Parameters must be an object".to_string(),
                field: None,
            });
        }
        
        if !params.get("text").is_some() {
            return Err(LLMSpellError::Validation {
                message: "Missing required parameter: text".to_string(),
                field: Some("text".to_string()),
            });
        }
        
        Ok(())
    }
}

#[tokio::test]
async fn test_agent_conversation_flow() {
    let mut agent = TestAgent::new("conversational-agent");
    
    // Test empty conversation
    let conv = agent.get_conversation().await.unwrap();
    assert_eq!(conv.len(), 0);
    
    // Execute with input
    let input = AgentInput::new("Hello, agent!".to_string());
    let context = ExecutionContext::new("test-session".to_string());
    
    let output = agent.execute(input, context).await.unwrap();
    assert_eq!(output.content, "Processed: Hello, agent!");
    
    // Check conversation was updated
    let conv = agent.get_conversation().await.unwrap();
    assert_eq!(conv.len(), 2);
    assert_eq!(conv[0].role, MessageRole::User);
    assert_eq!(conv[0].content, "Hello, agent!");
    assert_eq!(conv[1].role, MessageRole::Assistant);
    assert_eq!(conv[1].content, "Processed: Hello, agent!");
    
    // Check execution count in metadata
    let count = output.get_metadata("execution_count").unwrap();
    assert_eq!(count, 1);
    
    // Add system message
    agent.add_message(ConversationMessage::system("You are a helpful assistant".to_string())).await.unwrap();
    let conv = agent.get_conversation().await.unwrap();
    assert_eq!(conv.len(), 3);
    
    // Clear conversation
    agent.clear_conversation().await.unwrap();
    let conv = agent.get_conversation().await.unwrap();
    assert_eq!(conv.len(), 0);
}

#[tokio::test]
async fn test_tool_execution_and_validation() {
    let tool = TestTool::new("string-tool");
    
    // Test schema
    let schema = tool.schema();
    assert_eq!(schema.name, "string_transform");
    assert_eq!(schema.parameters.len(), 2);
    assert_eq!(schema.required_parameters(), vec!["text"]);
    
    // Test parameter validation
    let valid_params = serde_json::json!({
        "text": "hello world",
        "operation": "uppercase"
    });
    assert!(tool.validate_parameters(&valid_params).await.is_ok());
    
    let invalid_params = serde_json::json!({
        "operation": "uppercase"
    });
    let err = tool.validate_parameters(&invalid_params).await.unwrap_err();
    assert!(err.to_string().contains("Missing required parameter"));
    
    // Test execution
    let input = AgentInput::new("transform".to_string())
        .with_context("params".to_string(), valid_params);
    let context = ExecutionContext::new("test-session".to_string());
    
    let output = tool.execute(input, context).await.unwrap();
    assert_eq!(output.content, "HELLO WORLD");
    
    // Test different operations
    let params = serde_json::json!({
        "text": "HELLO",
        "operation": "lowercase"
    });
    let input = AgentInput::new("transform".to_string())
        .with_context("params".to_string(), params);
    let output = tool.execute(input, ExecutionContext::new("test".to_string())).await.unwrap();
    assert_eq!(output.content, "hello");
    
    // Test reverse
    let params = serde_json::json!({
        "text": "hello",
        "operation": "reverse"
    });
    let input = AgentInput::new("transform".to_string())
        .with_context("params".to_string(), params);
    let output = tool.execute(input, ExecutionContext::new("test".to_string())).await.unwrap();
    assert_eq!(output.content, "olleh");
}

#[tokio::test]
async fn test_error_handling_flow() {
    let agent = TestAgent::new("error-test-agent");
    
    // Test validation error
    let input = AgentInput::new("".to_string());
    let context = ExecutionContext::new("test".to_string());
    
    let result = agent.execute(input, context).await;
    assert!(result.is_err());
    
    let err = result.unwrap_err();
    match &err {
        LLMSpellError::Validation { field, .. } => {
            assert_eq!(*field, Some("prompt".to_string()));
        }
        _ => panic!("Expected validation error"),
    }
    
    // Test error handling
    let handled = agent.handle_error(err).await.unwrap();
    assert!(handled.content.contains("Error handled"));
}

#[tokio::test]
async fn test_component_metadata_updates() {
    let mut metadata = ComponentMetadata::new(
        "test-component".to_string(),
        "A test component".to_string(),
    );
    
    // Initial version
    assert_eq!(metadata.version, Version::new(0, 1, 0));
    
    // Update version
    metadata.update_version(Version::new(1, 0, 0));
    assert_eq!(metadata.version, Version::new(1, 0, 0));
    
    // Check timestamps
    assert!(metadata.updated_at >= metadata.created_at);
    
    // Test serialization
    let json = serde_json::to_string(&metadata).unwrap();
    let deserialized: ComponentMetadata = serde_json::from_str(&json).unwrap();
    assert_eq!(metadata.id, deserialized.id);
    assert_eq!(metadata.name, deserialized.name);
}

#[tokio::test]
async fn test_execution_context_environment() {
    let context = ExecutionContext::new("test-session".to_string())
        .with_user_id("user-123".to_string())
        .with_env("LOG_LEVEL".to_string(), "debug".to_string())
        .with_env("ENV".to_string(), "test".to_string());
    
    assert_eq!(context.session_id, "test-session");
    assert_eq!(context.user_id, Some("user-123".to_string()));
    assert_eq!(context.get_env("LOG_LEVEL"), Some(&"debug".to_string()));
    assert_eq!(context.get_env("ENV"), Some(&"test".to_string()));
    assert_eq!(context.get_env("MISSING"), None);
}

#[tokio::test]
async fn test_agent_input_context_manipulation() {
    let input = AgentInput::new("test prompt".to_string())
        .with_context("key1".to_string(), serde_json::json!("value1"))
        .with_context("key2".to_string(), serde_json::json!(42))
        .with_context("nested".to_string(), serde_json::json!({
            "inner": "value",
            "count": 10
        }));
    
    assert_eq!(input.prompt, "test prompt");
    assert_eq!(input.get_context("key1"), Some(&serde_json::json!("value1")));
    assert_eq!(input.get_context("key2"), Some(&serde_json::json!(42)));
    
    let nested = input.get_context("nested").unwrap();
    assert_eq!(nested.get("inner"), Some(&serde_json::json!("value")));
    assert_eq!(nested.get("count"), Some(&serde_json::json!(10)));
}

#[tokio::test]
async fn test_agent_output_metadata() {
    let output = AgentOutput::new("result".to_string())
        .with_metadata("confidence".to_string(), serde_json::json!(0.95))
        .with_metadata("tokens".to_string(), serde_json::json!(100))
        .with_metadata("model".to_string(), serde_json::json!("gpt-4"));
    
    assert_eq!(output.content, "result");
    assert_eq!(output.get_metadata("confidence"), Some(&serde_json::json!(0.95)));
    assert_eq!(output.get_metadata("tokens"), Some(&serde_json::json!(100)));
    assert_eq!(output.get_metadata("model"), Some(&serde_json::json!("gpt-4")));
    assert_eq!(output.get_metadata("missing"), None);
}