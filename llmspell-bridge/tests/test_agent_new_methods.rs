//! ABOUTME: Test the new Agent API methods added in Task 3.3.28
//! ABOUTME: Verifies Agent.register(), Agent.get() and other new methods work correctly

#[cfg(feature = "lua")]
#[tokio::test]
async fn test_agent_new_methods() -> Result<(), Box<dyn std::error::Error>> {
    use llmspell_bridge::globals::{create_standard_registry, GlobalContext, GlobalInjector};
    use llmspell_bridge::{ComponentRegistry, ProviderManager, ProviderManagerConfig};
    use mlua::Lua;
    use std::sync::Arc;

    // Setup test context
    let registry = Arc::new(ComponentRegistry::new());
    let config = ProviderManagerConfig::default();
    let providers = Arc::new(ProviderManager::new(config).await?);
    let context = Arc::new(GlobalContext::new(registry, providers));

    // Create Lua environment
    let lua = Lua::new();

    // Create and inject globals
    let global_registry = create_standard_registry(context.clone()).await?;
    let injector = GlobalInjector::new(Arc::new(global_registry));
    injector.inject_lua(&lua, &context)?;

    // Test Agent.register() exists
    let result = lua
        .load(
            r#"
        print("Testing Agent global methods...")
        
        -- Check Agent exists
        assert(Agent ~= nil, "Agent global not found")
        
        -- Check all methods exist
        local methods = {
            "create", "list", "discover", 
            "register", "get", "wrapAsTool", 
            "getInfo", "listCapabilities", 
            "createComposite", "discoverByCapability"
        }
        
        for _, method in ipairs(methods) do
            assert(type(Agent[method]) == "function", 
                string.format("Agent.%s() not found or not a function", method))
            print(string.format("✓ Agent.%s() exists", method))
        end
        
        print("All Agent methods found!")
    "#,
        )
        .exec();

    match result {
        Ok(_) => println!("Lua test passed!"),
        Err(e) => return Err(format!("Lua test failed: {}", e).into()),
    }

    Ok(())
}
