//! Test script arguments passing through kernel to runtime

use llmspell_bridge::ScriptRuntime;
use llmspell_config::LLMSpellConfig;
use std::collections::HashMap;

#[tokio::test(flavor = "multi_thread")]
async fn test_script_args_injection_lua() {
    // Create runtime with Lua engine
    let config = LLMSpellConfig::default();
    let mut runtime = ScriptRuntime::new_with_engine_name("lua", config)
        .await
        .expect("Failed to create Lua runtime");

    // Set script arguments
    let mut args = HashMap::new();
    args.insert("0".to_string(), "test_script.lua".to_string());
    args.insert("1".to_string(), "hello".to_string());
    args.insert("2".to_string(), "world".to_string());

    runtime
        .set_script_args(args)
        .await
        .expect("Failed to set script args");

    // Execute script that uses args
    let script = r#"
        -- Check that ARGS global is available
        if not ARGS then
            error("ARGS global not found")
        end
        
        -- Return concatenated args (use numeric index since they're positional)
        return ARGS[1] .. " " .. ARGS[2]
    "#;

    let result = runtime
        .execute_script(script)
        .await
        .expect("Script execution failed");

    // Check output
    let output_str = result.output.as_str().unwrap_or("");
    assert_eq!(output_str, "hello world", "Script args not properly passed");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_script_args_empty() {
    // Create runtime with Lua engine
    let config = LLMSpellConfig::default();
    let runtime = ScriptRuntime::new_with_engine_name("lua", config)
        .await
        .expect("Failed to create Lua runtime");

    // Execute script without setting args
    let script = r#"
        -- ARGS should be nil or empty when not set
        if ARGS and next(ARGS) ~= nil then
            error("ARGS should be empty")
        end
        return "ok"
    "#;

    let result = runtime
        .execute_script(script)
        .await
        .expect("Script execution failed");

    // Check output
    let output_str = result.output.as_str().unwrap_or("");
    assert_eq!(output_str, "ok", "ARGS should be empty when not set");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_script_args_indexed() {
    // Create runtime with Lua engine
    let config = LLMSpellConfig::default();
    let mut runtime = ScriptRuntime::new_with_engine_name("lua", config)
        .await
        .expect("Failed to create Lua runtime");

    // Set indexed script arguments (like command line args)
    let mut args = HashMap::new();
    args.insert("0".to_string(), "script.lua".to_string());
    args.insert("1".to_string(), "first".to_string());
    args.insert("2".to_string(), "second".to_string());
    args.insert("3".to_string(), "third".to_string());

    runtime
        .set_script_args(args)
        .await
        .expect("Failed to set script args");

    // Execute script that iterates over indexed args
    let script = r#"
        local result = {}
        -- ARGS uses numeric indices for positional args
        for i = 1, 3 do
            if ARGS[i] then
                table.insert(result, ARGS[i])
            end
        end
        return table.concat(result, ",")
    "#;

    let result = runtime
        .execute_script(script)
        .await
        .expect("Script execution failed");

    // Check output
    let output_str = result.output.as_str().unwrap_or("");
    assert_eq!(
        output_str, "first,second,third",
        "Indexed args not properly passed"
    );
}
