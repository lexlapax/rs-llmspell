// ABOUTME: Example demonstrating how to create custom agents in Rust
// ABOUTME: Shows agent implementation, capabilities, and integration with LLMSpell

use anyhow::{Context, Result};
use async_trait::async_trait;
use llmspell_agents::{
    Agent, AgentBuilder, AgentCapabilities, AgentFactory, BaseAgent, Message, Role,
};
use llmspell_bridge::{lua::LuaEngine, ScriptEngine};
use llmspell_core::{ComponentId, ToolInput, ToolOutput, ToolRegistry};
use serde_json::{json, Value};
use std::sync::Arc;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
use uuid::Uuid;

// Step 1: Define a custom agent that answers questions with a specific personality
#[derive(Debug, Clone)]
struct PersonalityAgent {
    id: ComponentId,
    name: String,
    personality: String,
    enthusiasm_level: u8, // 1-10
}

impl PersonalityAgent {
    fn new(name: String, personality: String, enthusiasm_level: u8) -> Self {
        Self {
            id: ComponentId::new(),
            name,
            personality,
            enthusiasm_level: enthusiasm_level.min(10).max(1),
        }
    }

    fn format_response(&self, base_response: &str) -> String {
        // Add personality flair to responses
        let exclamation_marks = "!".repeat(self.enthusiasm_level as usize / 3);
        let prefix = match self.personality.as_str() {
            "pirate" => format!("Arrr, matey{} ", exclamation_marks),
            "robot" => "BEEP BOOP. PROCESSING... ",
            "wizard" => format!("*waves wand mystically* "),
            "cowboy" => format!("Well, howdy partner{} ", exclamation_marks),
            _ => "",
        };

        let suffix = match self.personality.as_str() {
            "pirate" => " Yo ho ho!",
            "robot" => " END TRANSMISSION.",
            "wizard" => " *sparkles appear*",
            "cowboy" => " Yeehaw!",
            _ => "",
        };

        format!("{}{}{}", prefix, base_response, suffix)
    }
}

#[async_trait]
impl BaseAgent for PersonalityAgent {
    fn id(&self) -> &ComponentId {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn capabilities(&self) -> AgentCapabilities {
        AgentCapabilities {
            can_use_tools: false,
            supports_streaming: false,
            max_context_length: Some(1000),
            supports_function_calling: false,
        }
    }

    async fn invoke(&self, input: ToolInput) -> Result<ToolOutput> {
        let text = input
            .get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("Hello!");

        // Simple response logic based on input
        let base_response = if text.to_lowercase().contains("hello") {
            "Greetings to you!"
        } else if text.to_lowercase().contains("how are you") {
            "I'm doing wonderfully, thank you for asking!"
        } else if text.to_lowercase().contains("help") {
            "I'm here to assist you with anything you need!"
        } else {
            "That's an interesting thought!"
        };

        let formatted_response = self.format_response(base_response);

        Ok(ToolOutput::from_json(json!({
            "text": formatted_response,
            "personality": self.personality,
            "enthusiasm": self.enthusiasm_level,
            "success": true
        })))
    }

    async fn chat(&self, messages: Vec<Message>) -> Result<Message> {
        // For simplicity, just respond to the last message
        let last_message = messages
            .last()
            .map(|m| m.content.as_str())
            .unwrap_or("Hello");

        let input = json!({ "text": last_message });
        let output = self.invoke(input).await?;
        let response_text = output
            .to_json()
            .get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("I'm speechless!")
            .to_string();

        Ok(Message {
            role: Role::Assistant,
            content: response_text,
        })
    }
}

// Step 2: Create a more complex agent that can use tools
#[derive(Debug, Clone)]
struct ToolUsingAgent {
    id: ComponentId,
    name: String,
    tool_registry: Arc<ToolRegistry>,
}

impl ToolUsingAgent {
    fn new(name: String, tool_registry: Arc<ToolRegistry>) -> Self {
        Self {
            id: ComponentId::new(),
            name,
            tool_registry,
        }
    }
}

#[async_trait]
impl BaseAgent for ToolUsingAgent {
    fn id(&self) -> &ComponentId {
        &self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn capabilities(&self) -> AgentCapabilities {
        AgentCapabilities {
            can_use_tools: true,
            supports_streaming: false,
            max_context_length: Some(2000),
            supports_function_calling: true,
        }
    }

    async fn invoke(&self, input: ToolInput) -> Result<ToolOutput> {
        let text = input
            .get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        // Simple tool selection based on keywords
        if text.contains("uuid") || text.contains("id") {
            // Use UUID generator tool
            if let Ok(tool) = self.tool_registry.get("uuid_generator").await {
                let tool_input = json!({
                    "operation": "generate",
                    "version": "v4"
                });
                let result = tool.invoke(tool_input).await?;
                return Ok(ToolOutput::from_json(json!({
                    "text": format!("I generated a UUID for you: {}", 
                        result.to_json().get("uuid")
                            .and_then(|v| v.as_str())
                            .unwrap_or("error")),
                    "tool_used": "uuid_generator",
                    "success": true
                })));
            }
        } else if text.contains("time") || text.contains("date") {
            // Use date/time tool
            if let Ok(tool) = self.tool_registry.get("date_time_handler").await {
                let tool_input = json!({
                    "operation": "now"
                });
                let result = tool.invoke(tool_input).await?;
                return Ok(ToolOutput::from_json(json!({
                    "text": format!("The current time is: {}", 
                        result.to_json().get("text")
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")),
                    "tool_used": "date_time_handler",
                    "success": true
                })));
            }
        }

        // Default response
        Ok(ToolOutput::from_json(json!({
            "text": "I can help you with generating UUIDs or checking the time. Just ask!",
            "tool_used": "none",
            "success": true
        })))
    }

    async fn chat(&self, messages: Vec<Message>) -> Result<Message> {
        let last_message = messages
            .last()
            .map(|m| m.content.as_str())
            .unwrap_or("");

        let input = json!({ "text": last_message });
        let output = self.invoke(input).await?;
        let response_text = output
            .to_json()
            .get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("Error processing request")
            .to_string();

        Ok(Message {
            role: Role::Assistant,
            content: response_text,
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    println!("=== Creating Custom Agents ===\n");

    // Step 3: Create personality agents
    println!("1. Creating personality agents...");
    
    let pirate_agent = Arc::new(PersonalityAgent::new(
        "Captain Blackbeard".to_string(),
        "pirate".to_string(),
        8,
    )) as Arc<dyn BaseAgent>;

    let robot_agent = Arc::new(PersonalityAgent::new(
        "Unit-7734".to_string(),
        "robot".to_string(),
        3,
    )) as Arc<dyn BaseAgent>;

    let wizard_agent = Arc::new(PersonalityAgent::new(
        "Merlin".to_string(),
        "wizard".to_string(),
        6,
    )) as Arc<dyn BaseAgent>;

    println!("   ✅ Created pirate agent: {}", pirate_agent.name());
    println!("   ✅ Created robot agent: {}", robot_agent.name());
    println!("   ✅ Created wizard agent: {}", wizard_agent.name());

    // Step 4: Test personality agents
    println!("\n2. Testing personality agents...");
    
    let test_messages = vec![
        "Hello there!",
        "How are you today?",
        "Can you help me?",
    ];

    for agent in &[&pirate_agent, &robot_agent, &wizard_agent] {
        println!("\n   Agent: {}", agent.name());
        println!("   " + &"-".repeat(40));
        
        for msg in &test_messages {
            let input = json!({ "text": msg });
            let response = agent.invoke(input).await?;
            let text = response
                .to_json()
                .get("text")
                .and_then(|v| v.as_str())
                .unwrap_or("No response");
            
            println!("   You: {}", msg);
            println!("   {}: {}", agent.name(), text);
        }
    }

    // Step 5: Create tool-using agent
    println!("\n3. Creating tool-using agent...");
    
    let registry = Arc::new(ToolRegistry::new());
    llmspell_tools::register_all_tools(&*registry).await?;
    
    let tool_agent = Arc::new(ToolUsingAgent::new(
        "Assistant".to_string(),
        registry.clone(),
    )) as Arc<dyn BaseAgent>;
    
    println!("   ✅ Created tool-using agent with access to {} tools", 
             registry.list_tools().await.len());

    // Step 6: Test tool-using agent
    println!("\n4. Testing tool-using agent...");
    
    let tool_tests = vec![
        "Can you generate a UUID for me?",
        "What time is it?",
        "Tell me a joke",
    ];

    println!("   Agent: {}", tool_agent.name());
    println!("   " + &"-".repeat(40));
    
    for msg in &tool_tests {
        let input = json!({ "text": msg });
        let response = tool_agent.invoke(input).await?;
        let response_json = response.to_json();
        let text = response_json
            .get("text")
            .and_then(|v| v.as_str())
            .unwrap_or("No response");
        let tool_used = response_json
            .get("tool_used")
            .and_then(|v| v.as_str())
            .unwrap_or("none");
        
        println!("   You: {}", msg);
        println!("   Assistant: {}", text);
        if tool_used != "none" {
            println!("   [Used tool: {}]", tool_used);
        }
    }

    // Step 7: Using agents in Lua scripts
    println!("\n5. Integrating custom agents with Lua...");
    
    let mut engine = LuaEngine::new()?;
    engine.initialize().await?;
    
    // In a real implementation, you would register the custom agents
    // with the engine so they're available in scripts
    
    println!("   ✅ Custom agents can be registered and used in scripts");

    // Step 8: Advanced agent patterns
    println!("\n6. Advanced agent patterns...");
    println!("   - Chain multiple agents for complex tasks");
    println!("   - Implement memory/context management");
    println!("   - Add streaming support for real-time responses");
    println!("   - Integrate with external AI providers");
    println!("   - Implement agent collaboration patterns");

    println!("\n✅ Successfully created custom agents!");
    println!("\n💡 Key Concepts:");
    println!("   - Implement BaseAgent trait for custom behavior");
    println!("   - Define capabilities to indicate agent features");
    println!("   - Handle both invoke() and chat() methods");
    println!("   - Agents can use tools through ToolRegistry");
    println!("   - Custom agents integrate with LLMSpell scripts");

    Ok(())
}