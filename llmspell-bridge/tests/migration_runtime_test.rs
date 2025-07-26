//! ABOUTME: Tests for migration API availability in runtime configuration
//! ABOUTME: Verifies that migration functionality is properly enabled via configuration

use llmspell_bridge::{RuntimeConfig, ScriptRuntime};

#[tokio::test(flavor = "multi_thread")]
async fn test_migration_api_available_when_configured() -> Result<(), Box<dyn std::error::Error>> {
    // Create config with state persistence and migration enabled
    let mut config = RuntimeConfig::default();
    config.runtime.state_persistence.enabled = true;
    config.runtime.state_persistence.backend_type = "memory".to_string();
    config.runtime.state_persistence.migration_enabled = true;

    // Create runtime
    let runtime = ScriptRuntime::new_with_lua(config).await?;

    // Test that migration APIs are available
    let script = r#"
        -- Check that State global has migration methods
        assert(State ~= nil, "State global should exist")
        assert(type(State.migrate_to_version) == "function", "migrate_to_version should be a function")
        assert(type(State.get_migration_status) == "function", "get_migration_status should be a function")
        assert(type(State.list_schema_versions) == "function", "list_schema_versions should be a function")
        
        -- Try to call the methods to ensure they work
        local versions = State.list_schema_versions()
        assert(type(versions) == "table", "list_schema_versions should return a table")
        
        local status = State.get_migration_status()
        assert(type(status) == "table", "get_migration_status should return a table")
        
        return { success = true, migration_available = true }
    "#;

    let output = runtime.execute_script(script).await?;

    // Verify the script succeeded
    let result = output.output.as_object().unwrap();
    assert_eq!(result.get("success").unwrap().as_bool().unwrap(), true);
    assert_eq!(
        result
            .get("migration_available")
            .unwrap()
            .as_bool()
            .unwrap(),
        true
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_migration_api_not_available_when_disabled() -> Result<(), Box<dyn std::error::Error>>
{
    // Create config with state persistence disabled
    let mut config = RuntimeConfig::default();
    config.runtime.state_persistence.enabled = false;

    // Create runtime
    let runtime = ScriptRuntime::new_with_lua(config).await?;

    // Test that migration APIs are NOT available
    let script = r#"
        -- Check that State global exists but migration methods don't
        assert(State ~= nil, "State global should exist")
        
        -- Migration methods should be nil
        local has_migrate = State.migrate_to_version ~= nil
        local has_status = State.get_migration_status ~= nil
        local has_versions = State.list_schema_versions ~= nil
        
        return { 
            success = true, 
            has_migration_api = has_migrate or has_status or has_versions
        }
    "#;

    let output = runtime.execute_script(script).await?;

    // Verify the script succeeded
    let result = output.output.as_object().unwrap();
    assert_eq!(result.get("success").unwrap().as_bool().unwrap(), true);
    assert_eq!(
        result.get("has_migration_api").unwrap().as_bool().unwrap(),
        false
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_state_persistence_without_migration() -> Result<(), Box<dyn std::error::Error>> {
    // Create config with state persistence enabled but migration disabled
    let mut config = RuntimeConfig::default();
    config.runtime.state_persistence.enabled = true;
    config.runtime.state_persistence.backend_type = "memory".to_string();
    config.runtime.state_persistence.migration_enabled = false;

    // Create runtime
    let runtime = ScriptRuntime::new_with_lua(config).await?;

    // Test that basic state operations work but migration APIs are not available
    let script = r#"
        -- Check that State global exists and basic operations work
        assert(State ~= nil, "State global should exist")
        
        -- Basic state operations should work
        State.set("test_key", "test_value")
        local value = State.get("test_key")
        assert(value == "test_value", "State.set/get should work")
        
        -- Migration methods should be nil
        local has_migrate = State.migrate_to_version ~= nil
        local has_status = State.get_migration_status ~= nil
        local has_versions = State.list_schema_versions ~= nil
        
        return { 
            success = true, 
            state_works = true,
            has_migration_api = has_migrate or has_status or has_versions
        }
    "#;

    let output = runtime.execute_script(script).await?;

    // Verify the script succeeded
    let result = output.output.as_object().unwrap();
    assert_eq!(result.get("success").unwrap().as_bool().unwrap(), true);
    assert_eq!(result.get("state_works").unwrap().as_bool().unwrap(), true);
    assert_eq!(
        result.get("has_migration_api").unwrap().as_bool().unwrap(),
        false
    );

    Ok(())
}
