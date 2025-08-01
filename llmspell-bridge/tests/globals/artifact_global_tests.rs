//! ABOUTME: Integration tests for Artifact global in Lua environment
//! ABOUTME: Tests artifact storage, retrieval, and query functionality

use llmspell_bridge::lua::engine::LuaEngine;
use llmspell_bridge::runtime::GlobalRuntimeConfig;
use llmspell_bridge::ComponentRegistry;
use llmspell_core::test_utils::TestLogger;
use std::sync::Arc;
use tempfile::TempDir;
use tokio;

/// Test artifact storage and retrieval
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_artifact_storage_and_retrieval() {
    TestLogger::init();
    
    let mut runtime_config = GlobalRuntimeConfig::default();
    runtime_config.runtime.sessions.enabled = true;
    runtime_config.runtime.sessions.storage_backend = "memory".to_string();

    let registry = Arc::new(ComponentRegistry::new());
    let mut engine = LuaEngine::new(registry, runtime_config).await.unwrap();

    let lua_code = r#"
        -- Create session first
        local session_id = Session.create({
            name = "artifact_test_session"
        })
        
        -- Test storing text artifact
        local artifact_id = Artifact.store(
            session_id,
            "user_input",
            "test_document.txt",
            "This is test content for artifact storage",
            {
                author = "test_user",
                category = "documentation"
            }
        )
        
        -- Verify artifact ID structure
        assert(artifact_id ~= nil, "Artifact ID should not be nil")
        assert(type(artifact_id) == "table", "Artifact ID should be table")
        assert(artifact_id.content_hash ~= nil, "Should have content hash")
        assert(artifact_id.session_id == session_id, "Should match session ID")
        assert(type(artifact_id.sequence) == "number", "Should have sequence number")
        
        -- Test retrieving artifact
        local result = Artifact.get(session_id, artifact_id)
        assert(result ~= nil, "Should retrieve artifact")
        assert(result.content == "This is test content for artifact storage", "Content should match")
        assert(result.metadata.author == "test_user", "Metadata should match")
        
        return artifact_id
    "#;

    let result = engine.execute(lua_code).await;
    assert!(result.is_ok(), "Artifact storage test should succeed: {:?}", result.err());
}

/// Test artifact listing and querying
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_artifact_listing_and_query() {
    TestLogger::init();
    
    let mut runtime_config = GlobalRuntimeConfig::default();
    runtime_config.runtime.sessions.enabled = true;
    runtime_config.runtime.sessions.storage_backend = "memory".to_string();

    let registry = Arc::new(ComponentRegistry::new());
    let mut engine = LuaEngine::new(registry, runtime_config).await.unwrap();

    let lua_code = r#"
        -- Create session
        local session_id = Session.create({
            name = "query_test_session"
        })
        
        -- Store multiple artifacts
        local id1 = Artifact.store(
            session_id,
            "user_input",
            "doc1.txt",
            "Content 1",
            { tags = {"tag1", "common"} }
        )
        
        local id2 = Artifact.store(
            session_id,
            "tool_result",
            "result1.json",
            '{"data": "value"}',
            { tags = {"tag2", "common"} }
        )
        
        -- Test listing artifacts
        local artifacts = Artifact.list(session_id)
        assert(#artifacts >= 2, "Should have at least 2 artifacts")
        
        -- Test querying by type
        local query_result = Artifact.query({
            session_id = session_id,
            type = "user_input"
        })
        assert(#query_result >= 1, "Should find user_input artifacts")
        
        -- Test querying by tags
        local tag_result = Artifact.query({
            session_id = session_id,
            tags = {"common"}
        })
        assert(#tag_result >= 2, "Should find artifacts with common tag")
        
        -- Test querying with limit
        local limited_result = Artifact.query({
            session_id = session_id,
            limit = 1
        })
        assert(#limited_result == 1, "Should respect limit")
        
        return true
    "#;

    let result = engine.execute(lua_code).await;
    assert!(result.is_ok(), "Artifact query test should succeed: {:?}", result.err());
}

/// Test artifact file storage
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_artifact_file_storage() {
    TestLogger::init();
    
    let temp_dir = TempDir::new().unwrap();
    let test_file = temp_dir.path().join("test.txt");
    std::fs::write(&test_file, "Test file content").unwrap();

    let mut runtime_config = GlobalRuntimeConfig::default();
    runtime_config.runtime.sessions.enabled = true;
    runtime_config.runtime.sessions.storage_backend = "memory".to_string();

    let registry = Arc::new(ComponentRegistry::new());
    let mut engine = LuaEngine::new(registry, runtime_config).await.unwrap();

    let lua_code = format!(r#"
        -- Create session
        local session_id = Session.create({{
            name = "file_test_session"
        }})
        
        -- Store file as artifact
        local artifact_id = Artifact.storeFile(
            session_id,
            "{}",
            "user_input",
            {{
                source = "test_file",
                mime_type = "text/plain"
            }}
        )
        
        -- Verify file was stored
        assert(artifact_id ~= nil, "File artifact ID should not be nil")
        assert(type(artifact_id) == "table", "File artifact ID should be table")
        
        -- Retrieve and verify content
        local result = Artifact.get(session_id, artifact_id)
        assert(result.content == "Test file content", "File content should match")
        
        return true
    "#, test_file.to_string_lossy());

    let result = engine.execute(&lua_code).await;
    assert!(result.is_ok(), "Artifact file storage test should succeed: {:?}", result.err());
}

/// Test artifact deletion
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_artifact_deletion() {
    TestLogger::init();
    
    let mut runtime_config = GlobalRuntimeConfig::default();
    runtime_config.runtime.sessions.enabled = true;
    runtime_config.runtime.sessions.storage_backend = "memory".to_string();

    let registry = Arc::new(ComponentRegistry::new());
    let mut engine = LuaEngine::new(registry, runtime_config).await.unwrap();

    let lua_code = r#"
        -- Create session
        local session_id = Session.create({
            name = "delete_test_session"
        })
        
        -- Store artifact
        local artifact_id = Artifact.store(
            session_id,
            "user_input",
            "to_delete.txt",
            "This will be deleted"
        )
        
        -- Verify it exists
        local before_delete = Artifact.list(session_id)
        local found_before = false
        for i, artifact in ipairs(before_delete) do
            if artifact.name == "to_delete.txt" then
                found_before = true
                break
            end
        end
        assert(found_before, "Artifact should exist before deletion")
        
        -- Delete artifact
        Artifact.delete(session_id, artifact_id)
        
        -- Verify it's gone
        local after_delete = Artifact.list(session_id)
        local found_after = false
        for i, artifact in ipairs(after_delete) do
            if artifact.name == "to_delete.txt" then
                found_after = true
                break
            end
        end
        assert(not found_after, "Artifact should not exist after deletion")
        
        return true
    "#;

    let result = engine.execute(lua_code).await;
    assert!(result.is_ok(), "Artifact deletion test should succeed: {:?}", result.err());
}

/// Test artifact error conditions
#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_artifact_error_conditions() {
    TestLogger::init();
    
    let mut runtime_config = GlobalRuntimeConfig::default();
    runtime_config.runtime.sessions.enabled = true;
    runtime_config.runtime.sessions.storage_backend = "memory".to_string();

    let registry = Arc::new(ComponentRegistry::new());
    let mut engine = LuaEngine::new(registry, runtime_config).await.unwrap();

    let lua_code = r#"
        -- Create session
        local session_id = Session.create({
            name = "error_test_session"
        })
        
        -- Test invalid artifact ID
        local success, err = pcall(function()
            Artifact.get(session_id, {
                content_hash = "invalid",
                session_id = session_id,
                sequence = 999
            })
        end)
        assert(not success, "Should fail with invalid artifact ID")
        
        -- Test invalid artifact type
        local success2, err2 = pcall(function()
            Artifact.store(session_id, "invalid_type", "test.txt", "content")
        end)
        assert(not success2, "Should fail with invalid artifact type")
        
        return true
    "#;

    let result = engine.execute(lua_code).await;
    assert!(result.is_ok(), "Artifact error test should succeed: {:?}", result.err());
}