// ABOUTME: Example demonstrating how to embed LLMSpell in a Rust application
// ABOUTME: Shows basic initialization, script execution, and tool registration

use anyhow::Result;
use llmspell_bridge::{lua::LuaEngine, ScriptEngine};
use llmspell_core::ToolRegistry;
use llmspell_tools::register_all_tools;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    println!("=== Embedding LLMSpell in Rust ===\n");

    // Step 1: Create a tool registry and register tools
    println!("1. Setting up tool registry...");
    let registry = ToolRegistry::new();
    register_all_tools(&registry).await?;
    let tool_count = registry.list_tools().await.len();
    println!("   ✅ Registered {} tools", tool_count);

    // Step 2: Create a Lua script engine
    println!("\n2. Creating Lua script engine...");
    let mut engine = LuaEngine::new()?;
    
    // Initialize the engine with global objects
    engine.initialize().await?;
    println!("   ✅ Engine initialized with global objects");

    // Step 3: Execute a simple Lua script
    println!("\n3. Executing inline Lua script...");
    let script = r#"
        print("   Hello from embedded Lua!")
        print("   Available globals:")
        
        -- Check which globals are available
        local globals = {"Tool", "Agent", "Workflow", "Provider", "State"}
        for _, name in ipairs(globals) do
            if _G[name] then
                print("   - " .. name .. ": available")
            else
                print("   - " .. name .. ": not available")
            end
        end
        
        -- Use a tool
        local result = Tool.invoke("uuid_generator", {
            operation = "generate",
            version = "v4"
        })
        
        if result and result.text then
            print("\n   Generated UUID: " .. result.text)
        end
        
        return "Script completed successfully"
    "#;

    let result = engine.execute(script).await?;
    if let Some(output) = result {
        println!("\n   Script returned: {}", output);
    }

    // Step 4: Execute a script from file
    println!("\n4. Executing script from file...");
    
    // Create a simple test script
    let test_script_path = "/tmp/test_embed.lua";
    std::fs::write(
        test_script_path,
        r#"
-- Test script for embedded LLMSpell
print("   Running from file: " .. _VERSION)

-- List available tools
local tools = Tool.list()
print("   Found " .. #tools .. " tools")

-- Show first 3 tools
for i = 1, math.min(3, #tools) do
    print("   - " .. tostring(tools[i]))
end

return {
    success = true,
    message = "File script executed",
    tool_count = #tools
}
"#,
    )?;

    let file_result = engine.execute_file(test_script_path).await?;
    if let Some(output) = file_result {
        println!("\n   File script returned: {}", output);
    }

    // Step 5: Interact with script state
    println!("\n5. Interacting with script state...");
    
    // Set a global variable
    engine.execute("my_counter = 0").await?;
    
    // Increment it multiple times
    for i in 1..=3 {
        let script = format!("my_counter = my_counter + 1; return my_counter");
        let result = engine.execute(&script).await?;
        if let Some(value) = result {
            println!("   Iteration {}: counter = {}", i, value);
        }
    }

    // Step 6: Error handling
    println!("\n6. Demonstrating error handling...");
    
    let error_script = r#"
        -- This will cause an error
        local result = 10 / 0
        return result
    "#;
    
    match engine.execute(error_script).await {
        Ok(Some(v)) => println!("   Result: {}", v),
        Ok(None) => println!("   No result returned"),
        Err(e) => println!("   ⚠️  Caught expected error: {}", e),
    }

    // Step 7: Custom tool registration (pseudo-code - would need actual implementation)
    println!("\n7. Advanced: Custom tool registration...");
    println!("   In a real application, you could:");
    println!("   - Register custom tools with the ToolRegistry");
    println!("   - Expose Rust functions to Lua scripts");
    println!("   - Handle async operations from scripts");
    println!("   - Integrate with your existing systems");

    println!("\n✅ Successfully embedded LLMSpell in Rust!");
    println!("\n💡 Key Concepts:");
    println!("   - LuaEngine provides script execution");
    println!("   - ToolRegistry manages available tools");
    println!("   - Scripts have access to global objects");
    println!("   - State persists between script executions");
    println!("   - Errors can be caught and handled");

    Ok(())
}

// Example: How to run this example
// From the example directory:
// cargo run
//
// Or from the project root:
// cargo run --example embed-llmspell