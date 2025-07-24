//! ABOUTME: Integration tests for global object injection system
//! ABOUTME: Tests Agent, Tool, and Workflow globals in Lua environment

#[cfg(feature = "lua")]
mod lua_globals {
    use llmspell_bridge::globals::{create_standard_registry, GlobalContext, GlobalInjector};
    use llmspell_bridge::{ComponentRegistry, ProviderManager, ProviderManagerConfig};
    use llmspell_core::Result;
    use mlua::Lua;
    use std::sync::Arc;

    async fn setup_test_context() -> Arc<GlobalContext> {
        let registry = Arc::new(ComponentRegistry::new());
        // Create a default provider manager config for tests
        let config = ProviderManagerConfig::default();
        let providers = Arc::new(ProviderManager::new(config).await.unwrap());
        Arc::new(GlobalContext::new(registry, providers))
    }

    #[tokio::test]
    async fn test_global_registry_creation() -> Result<()> {
        let context = setup_test_context().await;
        let registry = create_standard_registry(context).await?;

        // Check that core globals are registered
        assert!(registry.get("Agent").is_some());
        assert!(registry.get("Tool").is_some());
        assert!(registry.get("Workflow").is_some());
        assert!(registry.get("JSON").is_some());
        assert!(registry.get("Streaming").is_some());
        assert!(registry.get("Logger").is_some());
        assert!(registry.get("Config").is_some());
        assert!(registry.get("Utils").is_some());

        Ok(())
    }

    #[tokio::test]
    async fn test_global_injection_lua() -> Result<()> {
        let lua = Lua::new();
        let context = setup_test_context().await;
        let registry = create_standard_registry(context.clone()).await?;
        let injector = GlobalInjector::new(Arc::new(registry));

        // Inject globals into Lua
        injector.inject_lua(&lua, &context)?;

        // Verify globals exist in Lua
        lua.load(
            r#"
            assert(Agent ~= nil, "Agent global not found")
            assert(Tool ~= nil, "Tool global not found")
            assert(Workflow ~= nil, "Workflow global not found")
            assert(JSON ~= nil, "JSON global not found")
            assert(Streaming ~= nil, "Streaming global not found")
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
        let context = setup_test_context().await;
        let registry = create_standard_registry(context.clone()).await?;
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
        let context = setup_test_context().await;

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

        let registry = create_standard_registry(context.clone()).await?;
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
        let context = setup_test_context().await;
        let registry = create_standard_registry(context.clone()).await?;
        let injector = GlobalInjector::new(Arc::new(registry));

        injector.inject_lua(&lua, &context)?;

        // Test Workflow global functions
        lua.load(
            r#"
            -- Test Workflow.sequential()
            local seq = Workflow.sequential({
                name = "test_seq",
                description = "A test sequential workflow",
                steps = {
                    {
                        name = "step1",
                        type = "tool",
                        tool = "dummy_tool",
                        input = { message = "test" }
                    }
                }
            })
            assert(seq ~= nil, "Sequential workflow creation failed")
            local info = seq:getInfo()
            assert(info.name == "test_seq", "Sequential workflow name mismatch")
            assert(info.type == "sequential", "Sequential workflow type mismatch")
            
            -- Test Workflow.conditional()
            local cond = Workflow.conditional({
                name = "test_cond",
                description = "A test conditional workflow",
                branches = {
                    {
                        name = "branch1",
                        condition = { type = "always" },
                        steps = {
                            {
                                name = "step1",
                                type = "tool",
                                tool = "dummy_tool",
                                input = {}
                            }
                        }
                    }
                }
            })
            assert(cond ~= nil, "Conditional workflow creation failed")
            info = cond:getInfo()
            assert(info.type == "conditional", "Conditional workflow type mismatch")
            
            -- Test Workflow.loop()
            local loop_wf = Workflow.loop({
                name = "test_loop",
                description = "A test loop workflow",
                iterator = {
                    range = {
                        start = 1,
                        ["end"] = 5,
                        step = 1
                    }
                },
                body = {
                    {
                        name = "loop_step",
                        type = "tool",
                        tool = "dummy_tool",
                        input = {}
                    }
                }
            })
            assert(loop_wf ~= nil, "Loop workflow creation failed")
            info = loop_wf:getInfo()
            assert(info.type == "loop", "Loop workflow type mismatch")
            
            -- Test Workflow.parallel()
            local par = Workflow.parallel({
                name = "test_parallel",
                description = "A test parallel workflow",
                branches = {
                    {
                        name = "branch1",
                        steps = {
                            {
                                name = "step1",
                                type = "tool",
                                tool = "dummy_tool",
                                input = {}
                            }
                        }
                    }
                }
            })
            assert(par ~= nil, "Parallel workflow creation failed")
            info = par:getInfo()
            assert(info.type == "parallel", "Parallel workflow type mismatch")
            
            -- Test Workflow types listing
            local types = Workflow.types()
            assert(#types >= 4, "Should have at least 4 workflow types")
        "#,
        )
        .exec()
        .map_err(|e| llmspell_core::LLMSpellError::Component {
            message: format!("Workflow Lua test failed: {}", e),
            source: None,
        })?;

        Ok(())
    }

    #[tokio::test]
    async fn test_global_injection_performance() -> Result<()> {
        use std::time::Instant;

        let lua = Lua::new();
        let context = setup_test_context().await;
        let registry = create_standard_registry(context.clone()).await?;
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

    #[tokio::test]
    async fn test_json_global_lua() -> Result<()> {
        let lua = Lua::new();
        let context = setup_test_context().await;
        let registry = create_standard_registry(context.clone()).await?;
        let injector = GlobalInjector::new(Arc::new(registry));

        injector.inject_lua(&lua, &context)?;

        // Test JSON global functions
        lua.load(
            r#"
            -- Test JSON.parse()
            local json_str = '{"name": "test", "value": 42, "active": true}'
            local obj = JSON.parse(json_str)
            assert(obj.name == "test", "JSON.parse() name field incorrect")
            assert(obj.value == 42, "JSON.parse() value field incorrect")
            assert(obj.active == true, "JSON.parse() active field incorrect")
            
            -- Test JSON.stringify()
            local data = {
                message = "hello",
                count = 10,
                nested = {
                    flag = false
                }
            }
            local str = JSON.stringify(data)
            assert(type(str) == "string", "JSON.stringify() should return a string")
            
            -- Test round-trip
            local parsed = JSON.parse(str)
            assert(parsed.message == "hello", "Round-trip message incorrect")
            assert(parsed.count == 10, "Round-trip count incorrect")
            assert(parsed.nested.flag == false, "Round-trip nested flag incorrect")
            
            -- Test error handling
            local success, err = pcall(JSON.parse, "invalid json")
            assert(not success, "JSON.parse() should fail on invalid JSON")
        "#,
        )
        .exec()
        .map_err(|e| llmspell_core::LLMSpellError::Component {
            message: format!("JSON Lua test failed: {}", e),
            source: None,
        })?;

        Ok(())
    }

    #[tokio::test]
    async fn test_hook_global_lua() -> Result<()> {
        let lua = Lua::new();
        let context = setup_test_context().await;
        let registry = create_standard_registry(context.clone()).await?;
        let injector = GlobalInjector::new(Arc::new(registry));

        injector.inject_lua(&lua, &context)?;

        // Test Hook global placeholder functions
        lua.load(
            r#"
            -- Test Hook.register() placeholder
            local result = Hook.register("test_hook", function() end)
            assert(type(result) == "string", "Hook.register() should return a string")
            assert(result:find("placeholder") ~= nil, "Hook.register() should indicate it's a placeholder")
            
            -- Test Hook.list()
            local hooks = Hook.list()
            assert(type(hooks) == "table", "Hook.list() should return a table")
        "#,
        )
        .exec()
        .map_err(|e| llmspell_core::LLMSpellError::Component {
            message: format!("Hook Lua test failed: {}", e),
            source: None,
        })?;

        Ok(())
    }

    #[tokio::test]
    async fn test_event_global_lua() -> Result<()> {
        let lua = Lua::new();
        let context = setup_test_context().await;
        let registry = create_standard_registry(context.clone()).await?;
        let injector = GlobalInjector::new(Arc::new(registry));

        injector.inject_lua(&lua, &context)?;

        // Test Event global placeholder functions
        lua.load(
            r#"
            -- Test Event.emit() placeholder
            local result = Event.emit("test_event", {data = "test"})
            assert(type(result) == "string", "Event.emit() should return a string")
            assert(result:find("placeholder") ~= nil, "Event.emit() should indicate it's a placeholder")
            
            -- Test Event.subscribe() placeholder
            result = Event.subscribe("test_event", function() end)
            assert(type(result) == "string", "Event.subscribe() should return a string")
            assert(result:find("placeholder") ~= nil, "Event.subscribe() should indicate it's a placeholder")
            
            -- Test Event.unsubscribe()
            local success = Event.unsubscribe("test_subscription")
            assert(success == true, "Event.unsubscribe() should return true")
        "#,
        )
        .exec()
        .map_err(|e| llmspell_core::LLMSpellError::Component {
            message: format!("Event Lua test failed: {}", e),
            source: None,
        })?;

        Ok(())
    }

    #[tokio::test]
    async fn test_state_global_lua() -> Result<()> {
        let lua = Lua::new();
        let context = setup_test_context().await;
        let registry = create_standard_registry(context.clone()).await?;
        let injector = GlobalInjector::new(Arc::new(registry));

        injector.inject_lua(&lua, &context)?;

        // Test State global functions (in-memory implementation)
        lua.load(
            r#"
            -- Test State.set() and State.get()
            State.set("test_key", "test_value")
            local value = State.get("test_key")
            assert(value == "test_value", "State.get() should return stored value")
            
            -- Test complex data
            State.set("complex", {
                name = "test",
                count = 42,
                nested = {
                    flag = true
                }
            })
            local complex = State.get("complex")
            assert(complex.name == "test", "Complex state name incorrect")
            assert(complex.count == 42, "Complex state count incorrect")
            assert(complex.nested.flag == true, "Complex state nested flag incorrect")
            
            -- Test State.list()
            local keys = State.list()
            assert(type(keys) == "table", "State.list() should return a table")
            local found_test_key = false
            local found_complex = false
            for _, key in ipairs(keys) do
                if key == "test_key" then found_test_key = true end
                if key == "complex" then found_complex = true end
            end
            assert(found_test_key, "State.list() should include test_key")
            assert(found_complex, "State.list() should include complex")
            
            -- Test State.delete()
            State.delete("test_key")
            value = State.get("test_key")
            assert(value == nil, "State.get() should return nil after delete")
            
            -- Test non-existent key
            value = State.get("non_existent")
            assert(value == nil, "State.get() should return nil for non-existent key")
        "#,
        )
        .exec()
        .map_err(|e| llmspell_core::LLMSpellError::Component {
            message: format!("State Lua test failed: {}", e),
            source: None,
        })?;

        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_streaming_global_lua() -> Result<()> {
        let lua = Lua::new();
        let context = setup_test_context().await;
        let registry = create_standard_registry(context.clone()).await?;
        let injector = GlobalInjector::new(Arc::new(registry));

        injector.inject_lua(&lua, &context)?;

        // Test Streaming global functions
        lua.load(
            r#"
            -- Test Streaming.create()
            local stream = Streaming.create(function()
                for i = 1, 3 do
                    coroutine.yield("chunk" .. i)
                end
            end)
            assert(stream ~= nil, "Streaming.create() should return a stream")
            assert(type(stream.next) == "function", "Stream should have next() method")
            assert(type(stream.isDone) == "function", "Stream should have isDone() method")
            assert(type(stream.collect) == "function", "Stream should have collect() method")
            
            -- Test stream iteration
            local chunks = {}
            while not stream:isDone() do
                local chunk = stream:next()
                if chunk ~= nil then
                    table.insert(chunks, chunk)
                end
            end
            assert(#chunks == 3, "Should have 3 chunks")
            assert(chunks[1] == "chunk1", "First chunk should be 'chunk1'")
            assert(chunks[2] == "chunk2", "Second chunk should be 'chunk2'")
            assert(chunks[3] == "chunk3", "Third chunk should be 'chunk3'")
            
            -- Test that stream is done
            assert(stream:isDone() == true, "Stream should be done after iteration")
            assert(stream:next() == nil, "next() should return nil when done")
            
            -- Test stream collection
            local new_stream = Streaming.create(function()
                coroutine.yield("a")
                coroutine.yield("b")
                coroutine.yield("c")
            end)
            local collected = new_stream:collect()
            assert(type(collected) == "table", "collect() should return a table")
            assert(#collected == 3, "collect() should return 3 items")
            assert(collected[1] == "a", "First collected item should be 'a'")
            assert(collected[2] == "b", "Second collected item should be 'b'")
            assert(collected[3] == "c", "Third collected item should be 'c'")
            
            -- Test Streaming.yield exists (even if it's a placeholder)
            assert(type(Streaming.yield) == "function", "Streaming.yield should be a function")
        "#,
        )
        .exec()
        .map_err(|e| llmspell_core::LLMSpellError::Component {
            message: format!("Streaming Lua test failed: {}", e),
            source: None,
        })?;

        Ok(())
    }
}
