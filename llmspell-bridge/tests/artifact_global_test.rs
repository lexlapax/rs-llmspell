//! ABOUTME: Integration tests for `ArtifactGlobal` functionality
//! ABOUTME: Tests artifact CRUD, binary data, metadata, and large file handling

#[cfg(feature = "lua")]
mod artifact_tests {
    use llmspell_bridge::globals::{create_standard_registry, GlobalContext, GlobalInjector};
    use llmspell_bridge::{ComponentRegistry, ProviderManager, ProviderManagerConfig};
    use llmspell_events::bus::EventBus;
    use llmspell_hooks::{HookExecutor, HookRegistry};
    use llmspell_sessions::{SessionManager, SessionManagerConfig};
    use llmspell_state_persistence::StateManager;
    use llmspell_storage::MemoryBackend;
    use mlua::Lua;
    use std::fs;
    use std::sync::Arc;

    async fn setup_test_context_with_artifacts() -> (Arc<GlobalContext>, Lua) {
        let registry = Arc::new(ComponentRegistry::new());
        let config = ProviderManagerConfig::default();
        let providers = Arc::new(ProviderManager::new(config).await.unwrap());

        // Create infrastructure for session manager
        let storage_backend = Arc::new(MemoryBackend::new());
        let state_manager = Arc::new(StateManager::new().await.unwrap());
        let hook_registry = Arc::new(HookRegistry::new());
        let hook_executor = Arc::new(HookExecutor::new());
        let event_bus = Arc::new(EventBus::new());

        let session_config = SessionManagerConfig::default();

        // Create session manager with proper constructor
        let session_manager = Arc::new(
            SessionManager::new(
                state_manager.clone(),
                storage_backend,
                hook_registry,
                hook_executor,
                &event_bus,
                session_config,
            )
            .unwrap(),
        );

        let context = GlobalContext::new(registry, providers);
        context.set_bridge("session_manager", session_manager);
        context.set_bridge("state_manager", state_manager);

        let context = Arc::new(context);
        let lua = Lua::new();

        // Create and inject globals
        let registry = create_standard_registry(context.clone()).await.unwrap();
        let injector = GlobalInjector::new(Arc::new(registry));
        injector.inject_lua(&lua, &context).unwrap();

        (context, lua)
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_artifact_store_and_get() {
        let (_context, lua) = setup_test_context_with_artifacts().await;
        let lua_code = r#"
        -- Create a session first
        local session_id = Session.create({name = "Artifact Test"})
        Session.setCurrent(session_id)
        
        -- Store a text artifact
        local artifact_id = Artifact.store(
            session_id,
            "tool_result",
            "test_output.txt",
            "Hello, artifact world!",
            {
                mime_type = "text/plain",
                tags = {"test", "text"}
            }
        )
        
        assert(artifact_id, "Artifact ID should not be nil")
        assert(artifact_id.content_hash, "Should have content hash")
        assert(artifact_id.session_id == session_id, "Session ID should match")
        assert(artifact_id.sequence, "Should have sequence number")
        
        -- Get the artifact back
        local artifact = Artifact.get(session_id, artifact_id)
        assert(artifact, "Should retrieve artifact")
        assert(artifact.content == "Hello, artifact world!", "Content should match")
        assert(artifact.metadata.name == "test_output.txt", "Name should match")
        assert(artifact.metadata.mime_type == "text/plain", "MIME type should match")
        
        return artifact_id
    "#;

        lua.load(lua_code).exec().unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_artifact_binary_data() {
        let (_context, lua) = setup_test_context_with_artifacts().await;
        let lua_code = r#"
        -- Create session
        local session_id = Session.create({name = "Binary Test"})
        
        -- Create binary data (simulate image data)
        local binary_data = string.char(0xFF, 0xD8, 0xFF, 0xE0) .. "JFIF" .. string.rep("\0", 100)
        
        -- Store binary artifact
        local artifact_id = Artifact.store(
            session_id,
            "agent_output",
            "test_image.jpg",
            binary_data,
            {
                mime_type = "image/jpeg",
                size = #binary_data
            }
        )
        
        -- Retrieve and verify
        local artifact = Artifact.get(session_id, artifact_id)
        assert(artifact.metadata.mime_type == "image/jpeg", "MIME type should be preserved")
        assert(type(artifact.content) == "string", "Binary content should be returned")
        assert(#artifact.content == #binary_data, "Binary size should match")
        
        -- Verify binary content integrity
        assert(artifact.content:byte(1) == 0xFF, "First byte should match")
        assert(artifact.content:byte(2) == 0xD8, "Second byte should match")
        
        return true
    "#;

        lua.load(lua_code).exec().unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_artifact_metadata_preservation() {
        let (_context, lua) = setup_test_context_with_artifacts().await;
        let lua_code = r#"
        -- Create session
        local session_id = Session.create({name = "Metadata Test"})
        
        -- Store artifact with rich metadata
        local metadata = {
            mime_type = "application/json",
            tags = {"important", "processed", "v2"},
            custom_field = "custom_value",
            nested = {
                level = 2,
                data = {1, 2, 3}
            },
            timestamp = os.time()
        }
        
        local artifact_id = Artifact.store(
            session_id,
            "tool_result",
            "data.json",
            '{"key": "value"}',
            metadata
        )
        
        -- Retrieve and verify metadata
        local artifact = Artifact.get(session_id, artifact_id)
        assert(artifact.metadata.mime_type == "application/json", "MIME type preserved")
        assert(#artifact.metadata.tags == 3, "Tags preserved")
        assert(artifact.metadata.custom.custom_field == "custom_value", "Custom field preserved")
        assert(artifact.metadata.custom.nested.level == 2, "Nested data preserved")
        assert(artifact.metadata.custom.timestamp == metadata.timestamp, "Timestamp preserved")
        
        return true
    "#;

        lua.load(lua_code).exec().unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_artifact_list_operations() {
        let (_context, lua) = setup_test_context_with_artifacts().await;
        let lua_code = r#"
        -- Create session
        local session_id = Session.create({name = "List Test"})
        
        -- Store multiple artifacts
        local ids = {}
        for i = 1, 5 do
            ids[i] = Artifact.store(
                session_id,
                i % 2 == 0 and "tool_result" or "agent_output",
                "artifact_" .. i .. ".txt",
                "Content " .. i,
                {tags = {i % 2 == 0 and "even" or "odd"}}
            )
        end
        
        -- List all artifacts
        local artifacts = Artifact.list(session_id)
        assert(#artifacts == 5, "Should have 5 artifacts")
        
        -- Verify artifacts are returned in order
        for i, artifact in ipairs(artifacts) do
            assert(artifact.name == "artifact_" .. i .. ".txt", "Artifact order preserved")
        end
        
        -- Test list with empty session ID (uses current)
        Session.setCurrent(session_id)
        local current_artifacts = Artifact.list("")
        assert(#current_artifacts == 5, "Should list current session artifacts")
        
        return #artifacts
    "#;

        lua.load(lua_code).exec().unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_artifact_store_file() {
        let (_context, lua) = setup_test_context_with_artifacts().await;

        // Create a temporary test file
        let temp_dir = std::env::temp_dir();
        let file_name = format!("test_file_{}.txt", std::process::id());
        let test_file = temp_dir.join(&file_name);
        fs::write(&test_file, b"File content for testing").unwrap();

        let lua_code = format!(
            r#"
        -- Create session
        local session_id = Session.create({{name = "File Test"}})
        
        -- Store file as artifact
        local artifact_id = Artifact.storeFile(
            session_id,
            "{}",
            "tool_result",
            {{
                source = "filesystem",
                original_path = "{}"
            }}
        )
        
        -- Retrieve and verify
        local artifact = Artifact.get(session_id, artifact_id)
        assert(artifact.content == "File content for testing", "File content should match")
        assert(artifact.metadata.name == "{}", "File name should be preserved")
        assert(artifact.metadata.custom.source == "filesystem", "Metadata should be preserved")
        
        return artifact_id
    "#,
            test_file.display(),
            test_file.display(),
            file_name
        );

        lua.load(&lua_code).exec().unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_artifact_delete() {
        let (_context, lua) = setup_test_context_with_artifacts().await;
        let lua_code = r#"
        -- Create session and artifact
        local session_id = Session.create({name = "Delete Test"})
        local artifact_id = Artifact.store(
            session_id,
            "user_input",
            "to_delete.txt",
            "Delete me"
        )
        
        -- Verify it exists
        local artifact = Artifact.get(session_id, artifact_id)
        assert(artifact, "Artifact should exist")
        
        -- Delete it
        Artifact.delete(session_id, artifact_id)
        
        -- Verify it's gone
        local success, err = pcall(Artifact.get, session_id, artifact_id)
        assert(not success, "Should fail to get deleted artifact")
        assert(string.find(tostring(err), "not found"), "Should have not found error")
        
        return true
    "#;

        lua.load(lua_code).exec().unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_large_artifact_handling() {
        let (_context, lua) = setup_test_context_with_artifacts().await;
        let lua_code = r#"
        -- Create session
        local session_id = Session.create({name = "Large Artifact Test"})
        
        -- Create large content (1MB)
        local large_content = string.rep("x", 1024 * 1024)
        
        -- Store large artifact
        local start_time = os.clock()
        local artifact_id = Artifact.store(
            session_id,
            "system_generated",
            "large_file.txt",
            large_content,
            {
                mime_type = "text/plain",
                size = #large_content
            }
        )
        local store_time = os.clock() - start_time
        
        -- Retrieve large artifact
        start_time = os.clock()
        local artifact = Artifact.get(session_id, artifact_id)
        local get_time = os.clock() - start_time
        
        -- Verify content
        assert(#artifact.content == 1024 * 1024, "Large content size should match")
        -- Note: Compression happens at storage layer, not exposed to Lua layer
        -- The artifact content is automatically decompressed when retrieved
        
        -- Performance check (should be fast due to compression)
        assert(store_time < 1.0, "Store should complete within 1 second")
        assert(get_time < 0.5, "Get should complete within 0.5 seconds")
        
        return true
    "#;

        lua.load(lua_code).exec().unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_artifact_error_handling() {
        let (_context, lua) = setup_test_context_with_artifacts().await;
        let lua_code = r#"
        -- Test invalid session ID
        local success, err = pcall(Artifact.store, "invalid-id", "tool_result", "name", "content")
        assert(not success, "Should fail with invalid session ID")
        
        -- Test empty artifact type (this should fail, unlike custom types)
        local session_id = Session.create({name = "Error Test"})
        success, err = pcall(Artifact.store, session_id, "", "name", "content")
        assert(not success, "Should fail with empty artifact type")
        
        -- Test non-existent artifact
        local fake_id = {
            content_hash = "fakehash",
            session_id = session_id,
            sequence = 999
        }
        success, err = pcall(Artifact.get, session_id, fake_id)
        assert(not success, "Should fail with non-existent artifact")
        
        -- Test delete on non-existent
        success, err = pcall(Artifact.delete, session_id, fake_id)
        assert(not success, "Should fail to delete non-existent artifact")
        
        return true
    "#;

        lua.load(lua_code).exec().unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_artifact_performance() {
        let (_context, lua) = setup_test_context_with_artifacts().await;
        let lua_code = r#"
            local session_id = Session.create({name = "Performance Test"})
            
            -- Measure artifact creation performance
            local start_time = os.clock()
            local ids = {}
            
            -- Create 100 artifacts
            for i = 1, 100 do
                ids[i] = Artifact.store(
                    session_id,
                    "tool_result",
                    "perf_" .. i .. ".txt",
                    "Content " .. i .. string.rep("x", i * 100),
                    {index = i}
                )
            end
            
            local create_time = os.clock() - start_time
            assert(create_time < 5.0, "Should create 100 artifacts in under 5 seconds")
            
            -- Measure list performance
            start_time = os.clock()
            local artifacts = Artifact.list(session_id)
            local list_time = os.clock() - start_time
            
            assert(#artifacts == 100, "Should have all 100 artifacts")
            assert(list_time < 0.1, "Listing should be fast")
            
            return {
                create_time = create_time,
                list_time = list_time,
                count = #artifacts
            }
        "#;

        lua.load(lua_code).exec().unwrap();
    }
}
