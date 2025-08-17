//! ABOUTME: Integration tests for Session global in Lua environment
//! ABOUTME: Tests session lifecycle, persistence, and replay functionality

use llmspell_bridge::lua::engine::LuaEngine;
use llmspell_bridge::runtime::Global;
use llmspell_config::LLMSpellConfig;
use llmspell_bridge::ComponentRegistry;
use llmspell_core::test_utils::TestLogger;
use llmspell_sessions::config::SessionManagerConfig;
use std::sync::Arc;
use tempfile::TempDir;
use tokio;

/// Test session creation and basic operations
#[tokio::test]
async fn test_session_creation_and_operations() {
    TestLogger::init();
    
    // Create temporary directory for storage
    let temp_dir = TempDir::new().unwrap();
    let storage_path = temp_dir.path().to_path_buf();

    // Create runtime config with sessions enabled
    let mut runtime_config = GlobalLLMSpellConfig::default();
    runtime_config.runtime.sessions.enabled = true;
    runtime_config.runtime.sessions.storage_backend = "memory".to_string();
    runtime_config.runtime.sessions.max_sessions = 10;

    // Create component registry
    let registry = Arc::new(ComponentRegistry::new());

    // Create Lua engine
    let mut engine = LuaEngine::new(registry, runtime_config).await.unwrap();

    // Test session creation
    let lua_code = r#"
        -- Test session creation
        local session_id = Session.create({
            name = "test_session",
            description = "Integration test session",
            tags = {"test", "integration"}
        })
        
        -- Verify session was created
        assert(session_id ~= nil, "Session ID should not be nil")
        assert(type(session_id) == "string", "Session ID should be string")
        
        -- Test getting session metadata
        local metadata = Session.get(session_id)
        assert(metadata ~= nil, "Session metadata should not be nil")
        assert(metadata.name == "test_session", "Session name should match")
        assert(metadata.description == "Integration test session", "Description should match")
        
        -- Test session context
        Session.set_current(session_id)
        local current = Session.get_current()
        assert(current == session_id, "Current session should match set session")
        
        -- Test session listing
        local sessions = Session.list()
        assert(#sessions > 0, "Should have at least one session")
        
        return session_id
    "#;

    let result = engine.execute(lua_code).await;
    assert!(result.is_ok(), "Session creation test should succeed: {:?}", result.err());
}

/// Test session persistence (save/load)
#[tokio::test]
async fn test_session_persistence() {
    TestLogger::init();
    
    // Create runtime config
    let mut runtime_config = GlobalLLMSpellConfig::default();
    runtime_config.runtime.sessions.enabled = true;
    runtime_config.runtime.sessions.storage_backend = "memory".to_string();

    let registry = Arc::new(ComponentRegistry::new());
    let mut engine = LuaEngine::new(registry, runtime_config).await.unwrap();

    let lua_code = r#"
        -- Create and save session
        local session_id = Session.create({
            name = "persistent_session",
            description = "Test persistence"
        })
        
        -- Save the session
        Session.save(session_id)
        
        -- Complete the session to remove from active list
        Session.complete(session_id)
        
        -- Load it back
        local loaded_id = Session.load(session_id)
        assert(loaded_id == session_id, "Loaded session ID should match")
        
        -- Verify metadata is preserved
        local metadata = Session.get(session_id)
        assert(metadata.name == "persistent_session", "Name should be preserved")
        
        return true
    "#;

    let result = engine.execute(lua_code).await;
    assert!(result.is_ok(), "Session persistence test should succeed: {:?}", result.err());
}

/// Test session lifecycle (suspend/resume/complete)
#[tokio::test]
async fn test_session_lifecycle() {
    TestLogger::init();
    
    let mut runtime_config = GlobalLLMSpellConfig::default();
    runtime_config.runtime.sessions.enabled = true;
    runtime_config.runtime.sessions.storage_backend = "memory".to_string();

    let registry = Arc::new(ComponentRegistry::new());
    let mut engine = LuaEngine::new(registry, runtime_config).await.unwrap();

    let lua_code = r#"
        -- Create session
        local session_id = Session.create({
            name = "lifecycle_session"
        })
        
        -- Test suspend
        Session.suspend(session_id)
        
        -- Test resume
        Session.resume(session_id)
        
        -- Test complete
        Session.complete(session_id)
        
        return true
    "#;

    let result = engine.execute(lua_code).await;
    assert!(result.is_ok(), "Session lifecycle test should succeed: {:?}", result.err());
}

/// Test session replay functionality
#[tokio::test]
async fn test_session_replay() {
    TestLogger::init();
    
    let mut runtime_config = GlobalLLMSpellConfig::default();
    runtime_config.runtime.sessions.enabled = true;
    runtime_config.runtime.sessions.storage_backend = "memory".to_string();

    let registry = Arc::new(ComponentRegistry::new());
    let mut engine = LuaEngine::new(registry, runtime_config).await.unwrap();

    let lua_code = r#"
        -- Create session
        local session_id = Session.create({
            name = "replay_session"
        })
        
        -- Test canReplay
        local can_replay = Session.canReplay(session_id)
        -- Note: New sessions might not have replay data yet
        assert(type(can_replay) == "boolean", "canReplay should return boolean")
        
        -- Test getReplayMetadata
        local metadata = Session.getReplayMetadata(session_id)
        assert(metadata ~= nil, "Replay metadata should not be nil")
        assert(type(metadata) == "table", "Replay metadata should be table")
        
        -- Test listReplayable
        local replayable = Session.listReplayable()
        assert(type(replayable) == "table", "listReplayable should return table")
        
        return true
    "#;

    let result = engine.execute(lua_code).await;
    assert!(result.is_ok(), "Session replay test should succeed: {:?}", result.err());
}

/// Test error conditions
#[tokio::test]
async fn test_session_error_conditions() {
    TestLogger::init();
    
    let mut runtime_config = GlobalLLMSpellConfig::default();
    runtime_config.runtime.sessions.enabled = true;
    runtime_config.runtime.sessions.storage_backend = "memory".to_string();

    let registry = Arc::new(ComponentRegistry::new());
    let mut engine = LuaEngine::new(registry, runtime_config).await.unwrap();

    let lua_code = r#"
        -- Test invalid session ID
        local success, err = pcall(function()
            Session.get("invalid-session-id")
        end)
        assert(not success, "Should fail with invalid session ID")
        
        -- Test operations on non-existent session
        local success2, err2 = pcall(function()
            Session.suspend("00000000-0000-0000-0000-000000000000")
        end)
        assert(not success2, "Should fail with non-existent session")
        
        return true
    "#;

    let result = engine.execute(lua_code).await;
    assert!(result.is_ok(), "Session error test should succeed: {:?}", result.err());
}