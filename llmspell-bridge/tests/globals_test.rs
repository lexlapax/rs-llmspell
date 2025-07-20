//! ABOUTME: Integration tests for global object injection system
//! ABOUTME: Tests Agent, Tool, and Workflow globals in Lua environment

#[cfg(feature = "lua")]
mod lua_globals {
    use llmspell_bridge::globals::{create_standard_registry, GlobalContext, GlobalInjector};
    use llmspell_bridge::ComponentRegistry;
    use llmspell_core::Result;
    use llmspell_providers::ProviderManager;
    use mlua::Lua;
    use std::sync::Arc;

    fn setup_test_context() -> Arc<GlobalContext> {
        let registry = Arc::new(ComponentRegistry::new());
        let providers = Arc::new(ProviderManager::new());
        Arc::new(GlobalContext::new(registry, providers))
    }

    #[test]
    fn test_global_registry_creation() -> Result<()> {
        let context = setup_test_context();
        let registry = create_standard_registry(context)?;

        // Check that core globals are registered
        assert!(registry.get("Agent").is_some());
        assert!(registry.get("Tool").is_some());
        assert!(registry.get("Workflow").is_some());
        assert!(registry.get("Logger").is_some());
        assert!(registry.get("Config").is_some());
        assert!(registry.get("Utils").is_some());

        Ok(())
    }

    #[test]
    fn test_global_injection_lua() -> Result<()> {
        let lua = Lua::new();
        let context = setup_test_context();
        let registry = create_standard_registry(context.clone())?;
        let injector = GlobalInjector::new(Arc::new(registry));

        // Inject globals into Lua
        injector.inject_lua(&lua, &context)?;

        // Verify globals exist in Lua
        lua.load(
            r#"
            assert(Agent ~= nil, "Agent global not found")
            assert(Tool ~= nil, "Tool global not found")
            assert(Workflow ~= nil, "Workflow global not found")
            assert(Logger ~= nil, "Logger global not found")
            assert(Config ~= nil, "Config global not found")
            assert(Utils ~= nil, "Utils global not found")
        "#,
        )
        .exec()
        .map_err(|e| llmspell_core::LLMSpellError::Component {
            message: format!("Lua test failed: {}", e),
            source: None,
        })?;

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_agent_global_lua() -> Result<()> {
        let lua = Lua::new();
        let context = setup_test_context();
        let registry = create_standard_registry(context.clone())?;
        let injector = GlobalInjector::new(Arc::new(registry));

        injector.inject_lua(&lua, &context)?;

        // Test Agent global functions
        lua.load(
            r#"
            -- Test Agent.list()
            local agents = Agent.list()
            assert(type(agents) == "table", "Agent.list() should return a table")
            
            -- Test Agent.discover()
            local types = Agent.discover()
            assert(type(types) == "table", "Agent.discover() should return a table")
            
            -- Test Agent.create (would fail without provider config)
            -- local agent = Agent.create({model = "test", system_prompt = "test"})
        "#,
        )
        .exec()
        .map_err(|e| llmspell_core::LLMSpellError::Component {
            message: format!("Agent Lua test failed: {}", e),
            source: None,
        })?;

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_tool_global_lua() -> Result<()> {
        let lua = Lua::new();
        let context = setup_test_context();

        // Register a test tool
        use async_trait::async_trait;
        use llmspell_core::traits::tool::{SecurityLevel, ToolCategory, ToolSchema};
        use llmspell_core::{BaseAgent, Tool};
        use llmspell_core::{ComponentMetadata, ExecutionContext};

        #[derive(Clone)]
        struct TestTool;

        #[async_trait]
        impl BaseAgent for TestTool {
            fn metadata(&self) -> &ComponentMetadata {
                // Create a static metadata instance
                static METADATA: std::sync::OnceLock<ComponentMetadata> =
                    std::sync::OnceLock::new();
                METADATA.get_or_init(|| {
                    ComponentMetadata::new("test_tool".to_string(), "A test tool".to_string())
                })
            }

            async fn execute(
                &self,
                _input: llmspell_core::types::AgentInput,
                _context: ExecutionContext,
            ) -> Result<llmspell_core::types::AgentOutput> {
                Ok(llmspell_core::types::AgentOutputBuilder::default()
                    .text("Test output".to_string())
                    .build())
            }

            async fn validate_input(
                &self,
                _input: &llmspell_core::types::AgentInput,
            ) -> Result<()> {
                Ok(())
            }

            async fn handle_error(
                &self,
                error: llmspell_core::LLMSpellError,
            ) -> Result<llmspell_core::types::AgentOutput> {
                Err(error)
            }
        }

        impl Tool for TestTool {
            fn category(&self) -> ToolCategory {
                ToolCategory::Utility
            }

            fn security_level(&self) -> SecurityLevel {
                SecurityLevel::Safe
            }

            fn schema(&self) -> ToolSchema {
                ToolSchema::new("test_tool".to_string(), "A test tool".to_string())
            }
        }

        context
            .registry
            .register_tool("test_tool".to_string(), Arc::new(TestTool))?;

        let registry = create_standard_registry(context.clone())?;
        let injector = GlobalInjector::new(Arc::new(registry));

        injector.inject_lua(&lua, &context)?;

        // Test Tool global functions
        lua.load(
            r#"
            -- Test Tool.list()
            local tools = Tool.list()
            assert(type(tools) == "table", "Tool.list() should return a table")
            assert(#tools > 0, "Should have at least one tool")
            
            -- Find our test tool
            local found = false
            for _, tool in ipairs(tools) do
                if tool.name == "test_tool" then
                    found = true
                    assert(tool.description == "A test tool", "Tool description mismatch")
                end
            end
            assert(found, "test_tool not found in list")
            
            -- Test Tool.get()
            local tool = Tool.get("test_tool")
            assert(tool ~= nil, "Tool.get() should return the tool")
            assert(tool.name == "test_tool", "Tool name mismatch")
            assert(tool.schema ~= nil, "Tool should have schema")
            
            -- Test Tool.exists()
            assert(Tool.exists("test_tool") == true, "test_tool should exist")
            assert(Tool.exists("nonexistent") == false, "nonexistent tool should not exist")
            
            -- Test Tool.categories()
            local categories = Tool.categories()
            assert(type(categories) == "table", "Tool.categories() should return a table")
        "#,
        )
        .exec()
        .map_err(|e| llmspell_core::LLMSpellError::Component {
            message: format!("Tool Lua test failed: {}", e),
            source: None,
        })?;

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_workflow_global_lua() -> Result<()> {
        let lua = Lua::new();
        let context = setup_test_context();
        let registry = create_standard_registry(context.clone())?;
        let injector = GlobalInjector::new(Arc::new(registry));

        injector.inject_lua(&lua, &context)?;

        // Test Workflow global functions
        lua.load(
            r#"
            -- Test Workflow.sequential()
            local seq = Workflow.sequential("test_seq", "A test sequential workflow")
            assert(seq ~= nil, "Sequential workflow creation failed")
            local info = seq:getInfo()
            assert(info.name == "test_seq", "Sequential workflow name mismatch")
            assert(info.type == "sequential", "Sequential workflow type mismatch")
            
            -- Test Workflow.conditional()
            local cond = Workflow.conditional("test_cond", "A test conditional workflow")
            assert(cond ~= nil, "Conditional workflow creation failed")
            info = cond:getInfo()
            assert(info.type == "conditional", "Conditional workflow type mismatch")
            
            -- Test Workflow.loop()
            -- Note: Loop workflow would need additional configuration to work properly
            -- Skip for now as it requires iterator configuration
            
            -- Test Workflow.parallel()
            -- Note: Parallel workflow would need branches to work properly
            -- Skip for now as it requires branch configuration
            
            -- Test Workflow.create()
            local wf = Workflow.create({
                type = "sequential",
                name = "test_create",
                description = "Created with create()"
            })
            assert(wf ~= nil, "Workflow.create() failed")
            info = wf:getInfo()
            assert(info.name == "test_create", "Created workflow name mismatch")
            
            -- Test that loop workflow would fail without proper configuration
            local success, err = pcall(function()
                return Workflow.create({
                    type = "loop",
                    name = "test_loop_fail",
                    description = "Should fail without iterator"
                })
            end)
            assert(not success, "Loop workflow should fail without iterator configuration")
            
            -- Test that parallel workflow would also fail without branches
            success, err = pcall(function()
                return Workflow.create({
                    type = "parallel",
                    name = "test_parallel_fail",
                    description = "Should fail without branches"
                })
            end)
            assert(not success, "Parallel workflow should fail without branch configuration")
        "#,
        )
        .exec()
        .map_err(|e| llmspell_core::LLMSpellError::Component {
            message: format!("Workflow Lua test failed: {}", e),
            source: None,
        })?;

        Ok(())
    }

    #[test]
    fn test_global_injection_performance() -> Result<()> {
        use std::time::Instant;

        let lua = Lua::new();
        let context = setup_test_context();
        let registry = create_standard_registry(context.clone())?;
        let injector = GlobalInjector::new(Arc::new(registry));

        // Measure injection time
        let start = Instant::now();
        injector.inject_lua(&lua, &context)?;
        let duration = start.elapsed();

        // Should complete within 5ms as per requirement
        assert!(
            duration.as_millis() < 5,
            "Global injection took {}ms, should be <5ms",
            duration.as_millis()
        );

        Ok(())
    }
}
