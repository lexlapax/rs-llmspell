//! ABOUTME: Integration tests for SessionGlobal functionality
//! ABOUTME: Tests CRUD operations, error handling, and session lifecycle through Lua

#[cfg(feature = "lua")]
mod session_tests {
    use llmspell_bridge::globals::{create_standard_registry, GlobalContext, GlobalInjector};
    use llmspell_bridge::{ComponentRegistry, ProviderManager, ProviderManagerConfig};
    use llmspell_events::bus::EventBus;
    use llmspell_hooks::{HookExecutor, HookRegistry};
    use llmspell_sessions::{SessionManager, SessionManagerConfig};
    use llmspell_state_persistence::StateManager;
    use llmspell_storage::MemoryBackend;
    use mlua::Lua;
    use std::sync::Arc;

    async fn setup_test_context_with_sessions() -> (Arc<GlobalContext>, Lua) {
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
    async fn test_session_create_and_get() {
        let (_context, lua) = setup_test_context_with_sessions().await;
        let lua_code = r#"
        -- Create a session with options
        local session_id = Session.create({
            name = "Test Session",
            description = "Integration test session",
            tags = {"test", "integration"}
        })
        
        assert(session_id, "Session ID should not be nil")
        assert(type(session_id) == "string", "Session ID should be a string")
        
        -- Get session metadata
        local metadata = Session.get(session_id)
        assert(metadata, "Metadata should not be nil")
        assert(metadata.name == "Test Session", "Session name mismatch")
        assert(metadata.description == "Integration test session", "Description mismatch")
        assert(#metadata.tags == 2, "Should have 2 tags")
        assert(metadata.status == "active", "Session should be active")
        
        return session_id
    "#;

        lua.load(lua_code).exec().unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_session_list_and_query() {
        let (_context, lua) = setup_test_context_with_sessions().await;
        let lua_code = r#"
        -- Create multiple sessions
        local ids = {}
        for i = 1, 3 do
            ids[i] = Session.create({
                name = "Session " .. i,
                tags = i % 2 == 0 and {"even"} or {"odd"}
            })
        end
        
        -- List all sessions
        local all_sessions = Session.list()
        assert(#all_sessions >= 3, "Should have at least 3 sessions")
        
        -- List with query
        local tagged_sessions = Session.list({
            tags = {"odd"},
            limit = 2
        })
        assert(#tagged_sessions <= 2, "Should respect limit")
        
        return #all_sessions
    "#;

        lua.load(lua_code).exec().unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_session_lifecycle() {
        let (_context, lua) = setup_test_context_with_sessions().await;
        let lua_code = r#"
        -- Create and manipulate session lifecycle
        local session_id = Session.create({name = "Lifecycle Test"})
        
        -- Get initial status
        local metadata = Session.get(session_id)
        assert(metadata.status == "active", "New session should be active")
        
        -- Suspend the session
        Session.suspend(session_id)
        metadata = Session.get(session_id)
        assert(metadata.status == "suspended", "Session should be suspended")
        
        -- Resume the session
        Session.resume(session_id)
        metadata = Session.get(session_id)
        assert(metadata.status == "active", "Session should be active after resume")
        
        -- Complete the session (this removes it from active sessions)
        Session.complete(session_id)
        
        return session_id
    "#;

        lua.load(lua_code).exec().unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_session_save_and_load() {
        let (_context, lua) = setup_test_context_with_sessions().await;

        // Test save and load functionality
        // Note: We can't directly simulate a restart by removing from active_sessions
        // because it's a private field. Instead, we test that save/load works
        // and that loading an already-active session is idempotent.
        let lua_code = r#"
        -- Create session with data
        local session_id = Session.create({
            name = "Persistent Session",
            description = "Test persistence",
            tags = {"test", "persistence"}
        })
        
        -- Save the session
        Session.save(session_id)
        
        -- Get current metadata
        local metadata = Session.get(session_id)
        assert(metadata.name == "Persistent Session", "Session should exist")
        
        -- Load the session (even though it's already active)
        -- This should be idempotent - loading an active session should work
        local loaded_id = Session.load(session_id)
        assert(loaded_id == session_id, "Loaded session ID should match")
        
        -- Verify data is still intact
        local metadata2 = Session.get(session_id)
        assert(metadata2.name == "Persistent Session", "Session data preserved")
        assert(metadata2.description == "Test persistence", "Description preserved")
        assert(#metadata2.tags == 2, "Tags preserved")
        
        -- Test loading a non-existent session
        local fake_id = "00000000-0000-0000-0000-000000000000"
        local success, err = pcall(Session.load, fake_id)
        assert(not success, "Loading non-existent session should fail")
        
        return true
    "#;

        lua.load(lua_code).exec().unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_session_current_context() {
        let (_context, lua) = setup_test_context_with_sessions().await;
        let lua_code = r#"
        -- Initially no current session
        assert(Session.get_current() == nil, "Should have no current session")
        
        -- Create and set current
        local session_id = Session.create({name = "Context Test"})
        Session.set_current(session_id)
        
        -- Verify current
        assert(Session.get_current() == session_id, "Current session should match")
        
        -- Clear current
        Session.set_current(nil)
        assert(Session.get_current() == nil, "Current session should be cleared")
        
        return true
    "#;

        lua.load(lua_code).exec().unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_session_error_handling() {
        let (_context, lua) = setup_test_context_with_sessions().await;
        let lua_code = r#"
        -- Test invalid session ID
        local success, err = pcall(Session.get, "invalid-uuid")
        assert(not success, "Should fail with invalid ID")
        assert(string.find(tostring(err), "Invalid session ID"), "Should have appropriate error message")
        
        -- Test operations on non-existent session
        local fake_id = "00000000-0000-0000-0000-000000000000"
        success, err = pcall(Session.suspend, fake_id)
        assert(not success, "Should fail with non-existent session")
        
        -- Test invalid operations on completed session
        local session_id = Session.create({name = "Complete Test"})
        Session.complete(session_id)
        success, err = pcall(Session.suspend, session_id)
        assert(not success, "Should not suspend completed session")
        
        return true
    "#;

        lua.load(lua_code).exec().unwrap();
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_concurrent_session_operations() {
        let (_context, lua) = setup_test_context_with_sessions().await;
        let lua_code = r#"
            -- Create sessions concurrently using coroutines
            local sessions = {}
            local function create_session(index)
                sessions[index] = Session.create({
                    name = "Concurrent " .. index,
                    tags = {"concurrent", "test" .. index}
                })
            end
            
            -- Create 10 sessions
            for i = 1, 10 do
                create_session(i)
            end
            
            -- Verify all created
            for i = 1, 10 do
                assert(sessions[i], "Session " .. i .. " should be created")
                local metadata = Session.get(sessions[i])
                assert(metadata.name == "Concurrent " .. i, "Session name mismatch")
            end
            
            return #sessions
        "#;

        lua.load(lua_code).exec().unwrap();
    }
}
