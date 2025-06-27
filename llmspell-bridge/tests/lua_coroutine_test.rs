//! ABOUTME: Tests for advanced Lua coroutine streaming functionality
//! ABOUTME: Validates true async streaming with Lua coroutines and yield

#[cfg(feature = "lua")]
mod tests {
    use llmspell_bridge::{
        engine::factory::{LuaConfig, EngineFactory},
        registry::ComponentRegistry,
        providers::{ProviderManager, ProviderManagerConfig},
    };
    use std::sync::Arc;
    use futures::StreamExt;

    #[tokio::test]
    async fn test_lua_coroutine_iteration() {
        let config = LuaConfig::default();
        let mut engine = EngineFactory::create_lua_engine(&config).unwrap();
        
        // Create mock registry and provider manager
        let registry = Arc::new(ComponentRegistry::new());
        let provider_config = ProviderManagerConfig::default();
        let providers = Arc::new(ProviderManager::new(provider_config).unwrap());
        
        // Inject APIs
        engine.inject_apis(&registry, &providers).unwrap();
        
        // Test streaming with coroutine iteration
        let script = r#"
            -- Create a streaming generator
            local stream = Streaming.create(function()
                for i = 1, 5 do
                    coroutine.yield("chunk_" .. i)
                end
            end)
            
            -- Iterate through the stream
            local chunks = {}
            while not stream:isDone() do
                local chunk = stream:next()
                if chunk then
                    table.insert(chunks, chunk)
                end
            end
            
            return {
                count = #chunks,
                first = chunks[1],
                last = chunks[#chunks],
                all = chunks
            }
        "#;
        
        let output = engine.execute_script(script).await;
        
        match output {
            Ok(result) => {
                let obj = result.output.as_object().expect("Expected object result");
                assert_eq!(obj.get("count").and_then(|v| v.as_i64()), Some(5));
                assert_eq!(obj.get("first").and_then(|v| v.as_str()), Some("chunk_1"));
                assert_eq!(obj.get("last").and_then(|v| v.as_str()), Some("chunk_5"));
                
                let all = obj.get("all").and_then(|v| v.as_array()).expect("Expected array");
                assert_eq!(all.len(), 5);
                for (i, chunk) in all.iter().enumerate() {
                    let expected = format!("chunk_{}", i + 1);
                    assert_eq!(chunk.as_str(), Some(&expected[..]));
                }
            }
            Err(e) => panic!("Script execution failed: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_lua_coroutine_collect() {
        let config = LuaConfig::default();
        let mut engine = EngineFactory::create_lua_engine(&config).unwrap();
        
        // Create mock registry and provider manager
        let registry = Arc::new(ComponentRegistry::new());
        let provider_config = ProviderManagerConfig::default();
        let providers = Arc::new(ProviderManager::new(provider_config).unwrap());
        
        // Inject APIs
        engine.inject_apis(&registry, &providers).unwrap();
        
        // Test collect method
        let script = r#"
            -- Create a streaming generator
            local stream = Streaming.create(function()
                coroutine.yield("hello")
                coroutine.yield("world")
                coroutine.yield("from")
                coroutine.yield("lua")
            end)
            
            -- Collect all chunks at once
            local all = stream:collect()
            
            return {
                count = #all,
                joined = table.concat(all, " ")
            }
        "#;
        
        let output = engine.execute_script(script).await;
        
        match output {
            Ok(result) => {
                let obj = result.output.as_object().expect("Expected object result");
                assert_eq!(obj.get("count").and_then(|v| v.as_i64()), Some(4));
                assert_eq!(obj.get("joined").and_then(|v| v.as_str()), Some("hello world from lua"));
            }
            Err(e) => panic!("Script execution failed: {:?}", e),
        }
    }

    #[tokio::test]
    async fn test_lua_streaming_execution_with_chunks() {
        let config = LuaConfig::default();
        let mut engine = EngineFactory::create_lua_engine(&config).unwrap();
        
        // Create mock registry and provider manager
        let registry = Arc::new(ComponentRegistry::new());
        let provider_config = ProviderManagerConfig::default();
        let providers = Arc::new(ProviderManager::new(provider_config).unwrap());
        
        // Inject APIs
        engine.inject_apis(&registry, &providers).unwrap();
        
        // Script that would produce streaming output
        let script = r#"
            -- Simulate a streaming LLM response
            local function simulateLLMStream()
                return Streaming.create(function()
                    local tokens = {"The", " ", "quick", " ", "brown", " ", "fox"}
                    for _, token in ipairs(tokens) do
                        coroutine.yield(token)
                    end
                end)
            end
            
            -- Get the stream and convert to result
            local stream = simulateLLMStream()
            local result = stream:collect()
            return table.concat(result, "")
        "#;
        
        // Execute with streaming
        let stream_result = engine.execute_script_streaming(script).await.unwrap();
        
        // Collect chunks from the stream
        let mut chunks = Vec::new();
        let mut stream = stream_result.stream;
        
        while let Some(chunk_result) = stream.next().await {
            match chunk_result {
                Ok(chunk) => {
                    if let llmspell_core::types::ChunkContent::Text(text) = &chunk.content {
                        chunks.push(text.clone());
                    }
                }
                Err(e) => panic!("Stream error: {:?}", e),
            }
        }
        
        // Should have at least one chunk (our current implementation)
        assert!(!chunks.is_empty(), "Expected at least one chunk");
        
        // The result should contain the expected output
        let combined = chunks.join("");
        assert!(combined.contains("The quick brown fox") || combined.contains("\"The quick brown fox\""), 
                "Expected result to contain 'The quick brown fox', got: {}", combined);
    }

    #[tokio::test]
    async fn test_lua_coroutine_error_handling() {
        let config = LuaConfig::default();
        let mut engine = EngineFactory::create_lua_engine(&config).unwrap();
        
        // Create mock registry and provider manager
        let registry = Arc::new(ComponentRegistry::new());
        let provider_config = ProviderManagerConfig::default();
        let providers = Arc::new(ProviderManager::new(provider_config).unwrap());
        
        // Inject APIs
        engine.inject_apis(&registry, &providers).unwrap();
        
        // Test error in coroutine
        let script = r#"
            local stream = Streaming.create(function()
                coroutine.yield("chunk1")
                error("Stream error!")
                coroutine.yield("chunk2") -- Never reached
            end)
            
            local chunks = {}
            local success, err = pcall(function()
                while not stream:isDone() do
                    local chunk = stream:next()
                    if chunk then
                        table.insert(chunks, chunk)
                    end
                end
            end)
            
            return {
                success = success,
                chunks_before_error = chunks,
                error_occurred = err ~= nil
            }
        "#;
        
        let output = engine.execute_script(script).await;
        
        match output {
            Ok(result) => {
                let obj = result.output.as_object().expect("Expected object result");
                assert_eq!(obj.get("success").and_then(|v| v.as_bool()), Some(false));
                
                let chunks = obj.get("chunks_before_error").and_then(|v| v.as_array()).expect("Expected array");
                assert_eq!(chunks.len(), 1);
                assert_eq!(chunks[0].as_str(), Some("chunk1"));
                
                assert_eq!(obj.get("error_occurred").and_then(|v| v.as_bool()), Some(true));
            }
            Err(e) => panic!("Script execution failed: {:?}", e),
        }
    }
}