//! Test state persistence through kernel

use llmspell_bridge::ScriptRuntime;
use llmspell_config::LLMSpellConfig;
use llmspell_state_persistence::manager::StateManager;
use std::sync::Arc;

#[tokio::test(flavor = "multi_thread")]
async fn test_state_persistence_basic() {
    // Using in-memory state storage for tests

    // Create StateManager (uses in-memory backend by default)
    let state_manager = Arc::new(
        StateManager::new()
            .await
            .expect("Failed to create StateManager"),
    );

    // Create runtime with external state manager
    let config = LLMSpellConfig::default();
    let runtime =
        ScriptRuntime::new_with_engine_and_state_manager("lua", config, state_manager.clone())
            .await
            .expect("Failed to create runtime with state manager");

    // Execute script that sets state
    let set_script = r#"
        -- Check that state global is available
        if not state then
            error("state global not found")
        end
        
        -- Set some values
        state.set("test_key", "test_value")
        state.set("number_key", 42)
        state.set("table_key", {foo = "bar", nested = {value = 123}})
        
        return "values set"
    "#;

    let result = runtime
        .execute_script(set_script)
        .await
        .expect("Script execution failed");
    assert_eq!(result.output.as_str().unwrap_or(""), "values set");

    // Execute script that reads state
    let get_script = r#"
        -- Read values back
        local val1 = state.get("test_key")
        local val2 = state.get("number_key")
        local val3 = state.get("table_key")
        
        if val1 ~= "test_value" then
            error("test_key value mismatch: " .. tostring(val1))
        end
        
        if val2 ~= 42 then
            error("number_key value mismatch: " .. tostring(val2))
        end
        
        if not val3 or val3.foo ~= "bar" or val3.nested.value ~= 123 then
            error("table_key value mismatch")
        end
        
        return "values verified"
    "#;

    let result = runtime
        .execute_script(get_script)
        .await
        .expect("Script execution failed");
    assert_eq!(result.output.as_str().unwrap_or(""), "values verified");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_state_persistence_across_runtimes() {
    // Using in-memory state storage for tests

    // Create first runtime and set state
    {
        let state_manager = Arc::new(
            StateManager::new()
                .await
                .expect("Failed to create StateManager"),
        );

        let config = LLMSpellConfig::default();
        let runtime =
            ScriptRuntime::new_with_engine_and_state_manager("lua", config, state_manager)
                .await
                .expect("Failed to create runtime");

        let script = r#"
            state.set("persistent_key", "persistent_value")
            state.set("counter", 100)
            return "state saved"
        "#;

        let result = runtime
            .execute_script(script)
            .await
            .expect("Script execution failed");
        assert_eq!(result.output.as_str().unwrap_or(""), "state saved");
    }

    // Create second runtime and read state
    {
        let state_manager = Arc::new(
            StateManager::new()
                .await
                .expect("Failed to create StateManager"),
        );

        let config = LLMSpellConfig::default();
        let runtime =
            ScriptRuntime::new_with_engine_and_state_manager("lua", config, state_manager)
                .await
                .expect("Failed to create runtime");

        let script = r#"
            local val = state.get("persistent_key")
            local counter = state.get("counter")
            
            if val ~= "persistent_value" then
                error("persistent_key not found or wrong value: " .. tostring(val))
            end
            
            if counter ~= 100 then
                error("counter not found or wrong value: " .. tostring(counter))
            end
            
            -- Increment counter
            state.set("counter", counter + 1)
            
            return "state persisted: " .. val .. ", counter: " .. tostring(counter + 1)
        "#;

        let result = runtime
            .execute_script(script)
            .await
            .expect("Script execution failed");
        assert_eq!(
            result.output.as_str().unwrap_or(""),
            "state persisted: persistent_value, counter: 101"
        );
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_state_deletion() {
    // Using in-memory state storage for tests

    let state_manager = Arc::new(
        StateManager::new()
            .await
            .expect("Failed to create StateManager"),
    );

    let config = LLMSpellConfig::default();
    let runtime = ScriptRuntime::new_with_engine_and_state_manager("lua", config, state_manager)
        .await
        .expect("Failed to create runtime");

    // Set and then delete state
    let script = r#"
        -- Set a value
        state.set("temp_key", "temp_value")
        
        -- Verify it exists
        local val = state.get("temp_key")
        if val ~= "temp_value" then
            error("Value not set correctly")
        end
        
        -- Delete it
        state.delete("temp_key")
        
        -- Verify it's gone
        local deleted_val = state.get("temp_key")
        if deleted_val ~= nil then
            error("Value not deleted: " .. tostring(deleted_val))
        end
        
        return "deletion successful"
    "#;

    let result = runtime
        .execute_script(script)
        .await
        .expect("Script execution failed");
    assert_eq!(result.output.as_str().unwrap_or(""), "deletion successful");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_state_list_keys() {
    // Using in-memory state storage for tests

    let state_manager = Arc::new(
        StateManager::new()
            .await
            .expect("Failed to create StateManager"),
    );

    let config = LLMSpellConfig::default();
    let runtime = ScriptRuntime::new_with_engine_and_state_manager("lua", config, state_manager)
        .await
        .expect("Failed to create runtime");

    // Set multiple keys and list them
    let script = r#"
        -- Set multiple keys
        state.set("key1", "value1")
        state.set("key2", "value2")
        state.set("key3", "value3")
        
        -- Get all keys
        local keys = state.keys()
        
        -- Sort for consistent comparison
        table.sort(keys)
        
        -- Verify we have all keys
        if #keys < 3 then
            error("Not enough keys: " .. #keys)
        end
        
        -- Check specific keys exist
        local has_key1, has_key2, has_key3 = false, false, false
        for _, k in ipairs(keys) do
            if k == "key1" then has_key1 = true end
            if k == "key2" then has_key2 = true end
            if k == "key3" then has_key3 = true end
        end
        
        if not (has_key1 and has_key2 and has_key3) then
            error("Missing expected keys")
        end
        
        return "keys listed successfully"
    "#;

    let result = runtime
        .execute_script(script)
        .await
        .expect("Script execution failed");
    assert_eq!(
        result.output.as_str().unwrap_or(""),
        "keys listed successfully"
    );
}
