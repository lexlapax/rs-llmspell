//! Integration tests for debug protocol routing and hook functionality

use llmspell_bridge::debug_adapters::{ExecutionManagerAdapter, VariableInspectorAdapter};
use llmspell_bridge::debug_runtime::{DebugSession, DebugSessionState};
use llmspell_bridge::debug_state_cache::SharedDebugStateCache;
use llmspell_bridge::execution_bridge::ExecutionManager;
use llmspell_bridge::execution_context::SharedExecutionContext;
use llmspell_bridge::lua::variable_inspector_impl::LuaVariableInspector;
use llmspell_bridge::variable_inspector::VariableInspector;
use llmspell_bridge::{debug_runtime::*, ScriptRuntime};
use llmspell_config::LLMSpellConfig;
use llmspell_core::debug::{DebugCapability, DebugRequest, DebugResponse};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_debug_protocol_routing() {
    // Create debug runtime with test session and proper capabilities
    let config = LLMSpellConfig::default();
    let session = DebugSession {
        session_id: "test-session".to_string(),
        script_content: r#"
            local x = 1
            local y = 2
            local z = x + y
            print("Result: " .. z)
        "#
        .to_string(),
        state: DebugSessionState::Initialized,
        start_time: std::time::Instant::now(),
    };

    // Set up debug capabilities properly
    let debug_cache = Arc::new(SharedDebugStateCache::new());
    let execution_manager = Arc::new(ExecutionManager::new(debug_cache));
    let execution_adapter = Arc::new(ExecutionManagerAdapter::new(
        execution_manager,
        session.session_id.clone(),
    )) as Arc<dyn DebugCapability>;

    let mut capabilities_map = HashMap::new();
    capabilities_map.insert("execution_manager".to_string(), execution_adapter);
    let capabilities = Arc::new(RwLock::new(capabilities_map));

    // Create the debug runtime - don't measure this as it includes ScriptRuntime creation
    let mut debug_runtime = DebugRuntime::new(config, session, capabilities)
        .await
        .expect("Failed to create debug runtime");

    // Test that debug commands can be routed properly
    let start = Instant::now();
    let request = DebugRequest::GetDebugState;
    let response = debug_runtime
        .process_debug_command(request)
        .await
        .expect("Failed to process debug command");
    let command_time = start.elapsed();

    // Verify command routing works and is fast
    assert!(
        command_time.as_millis() < 10,
        "Debug command routing took {}ms, expected < 10ms",
        command_time.as_millis()
    );

    // Verify we got a valid response
    match response {
        DebugResponse::DebugStateInfo(_) => {}
        _ => panic!("Expected DebugStateInfo response"),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_debug_hook_integration() {
    // Create a custom debug hook to track execution
    struct TestDebugHook {
        lines_hit: Arc<RwLock<Vec<u32>>>,
        functions_entered: Arc<RwLock<Vec<String>>>,
    }

    #[async_trait::async_trait]
    impl DebugHook for TestDebugHook {
        async fn on_line(&self, line: u32, _source: &str) -> DebugControl {
            self.lines_hit.write().await.push(line);
            DebugControl::Continue
        }

        async fn on_function_enter(&self, name: &str, _args: Vec<String>) -> DebugControl {
            self.functions_entered.write().await.push(name.to_string());
            DebugControl::Continue
        }

        async fn on_function_exit(&self, _name: &str, _result: Option<String>) -> DebugControl {
            DebugControl::Continue
        }

        async fn on_exception(&self, _error: &str, _line: u32) -> DebugControl {
            DebugControl::Pause
        }

        fn as_any(&self) -> &dyn std::any::Any {
            self
        }
    }

    let config = LLMSpellConfig::default();
    let mut runtime = ScriptRuntime::new_with_lua(config)
        .await
        .expect("Failed to create runtime");

    let test_hook = Arc::new(TestDebugHook {
        lines_hit: Arc::new(RwLock::new(Vec::new())),
        functions_entered: Arc::new(RwLock::new(Vec::new())),
    });

    // Install debug hooks
    runtime
        .install_debug_hooks(test_hook.clone())
        .expect("Failed to install debug hooks");

    // Execute script with hooks
    let script = r"
        local function test()
            local x = 1
            return x * 2
        end
        test()
    ";

    let _result = runtime
        .execute_script(script)
        .await
        .expect("Failed to execute script");

    // Verify hooks were called
    let lines_empty = test_hook.lines_hit.read().await.is_empty();
    assert!(
        !lines_empty,
        "Debug hook should have tracked line execution"
    );

    let functions_empty = test_hook.functions_entered.read().await.is_empty();
    assert!(
        !functions_empty,
        "Debug hook should have tracked function calls"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_debug_command_performance() {
    let config = LLMSpellConfig::default();
    let session = DebugSession {
        session_id: "perf-test".to_string(),
        script_content: "print('test')".to_string(),
        state: DebugSessionState::Initialized,
        start_time: std::time::Instant::now(),
    };

    // Set up debug capabilities properly
    let debug_cache = Arc::new(SharedDebugStateCache::new());
    let execution_manager = Arc::new(ExecutionManager::new(debug_cache));
    let execution_adapter = Arc::new(ExecutionManagerAdapter::new(
        execution_manager,
        session.session_id.clone(),
    )) as Arc<dyn DebugCapability>;

    let mut capabilities_map = HashMap::new();
    capabilities_map.insert("execution_manager".to_string(), execution_adapter);
    let capabilities = Arc::new(RwLock::new(capabilities_map));

    let mut debug_runtime = DebugRuntime::new(config, session, capabilities)
        .await
        .expect("Failed to create debug runtime");

    // Test breakpoint setting performance
    let start = Instant::now();
    let request = DebugRequest::SetBreakpoints {
        source: "test.lua".to_string(),
        breakpoints: vec![(10, None), (20, None), (30, None)],
    };

    let _response = debug_runtime
        .process_debug_command(request)
        .await
        .expect("Failed to process debug command");

    let command_time = start.elapsed();

    // Verify command execution time < 1ms
    assert!(
        command_time.as_micros() < 1000,
        "Debug command took {}μs, expected < 1000μs (1ms)",
        command_time.as_micros()
    );

    // Test step command performance
    let start = Instant::now();
    let request = DebugRequest::Step {
        step_type: llmspell_core::debug::StepType::StepOver,
    };

    let _response = debug_runtime
        .process_debug_command(request)
        .await
        .expect("Failed to process step command");

    let step_time = start.elapsed();

    assert!(
        step_time.as_micros() < 1000,
        "Step command took {}μs, expected < 1000μs (1ms)",
        step_time.as_micros()
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn test_capability_registry() {
    // Test that capabilities are properly registered and routed
    let config = LLMSpellConfig::default();
    let session = DebugSession {
        session_id: "capability-test".to_string(),
        script_content: "print('test')".to_string(),
        state: DebugSessionState::Initialized,
        start_time: std::time::Instant::now(),
    };

    // Set up debug capabilities properly
    let debug_cache = Arc::new(SharedDebugStateCache::new());
    let execution_context = Arc::new(RwLock::new(SharedExecutionContext::new()));

    // Create execution manager capability
    let execution_manager = Arc::new(ExecutionManager::new(debug_cache.clone()));
    let execution_adapter = Arc::new(ExecutionManagerAdapter::new(
        execution_manager,
        session.session_id.clone(),
    )) as Arc<dyn DebugCapability>;

    // Create variable inspector capability
    let variable_inspector = Arc::new(LuaVariableInspector::new(debug_cache, execution_context))
        as Arc<dyn VariableInspector>;
    let variable_adapter =
        Arc::new(VariableInspectorAdapter::new(variable_inspector)) as Arc<dyn DebugCapability>;

    let mut capabilities_map = HashMap::new();
    capabilities_map.insert("execution_manager".to_string(), execution_adapter);
    capabilities_map.insert("variable_inspector".to_string(), variable_adapter);
    let capabilities = Arc::new(RwLock::new(capabilities_map));

    let mut debug_runtime = DebugRuntime::new(config, session, capabilities)
        .await
        .expect("Failed to create debug runtime");

    // Test that execution_manager capability handles breakpoint requests
    let request = DebugRequest::SetBreakpoints {
        source: "test.lua".to_string(),
        breakpoints: vec![(5, None)],
    };

    let response = debug_runtime
        .process_debug_command(request)
        .await
        .expect("Failed to process command");

    match response {
        DebugResponse::BreakpointsSet { breakpoints } => {
            assert_eq!(breakpoints.len(), 1, "Should have set 1 breakpoint");
        }
        _ => panic!("Expected BreakpointsSet response"),
    }

    // Test that variable_inspector capability handles variable inspection
    let request = DebugRequest::InspectVariables {
        names: vec!["test_var".to_string()],
        frame_id: None,
    };

    let response = debug_runtime
        .process_debug_command(request)
        .await
        .expect("Failed to process command");

    match response {
        DebugResponse::Variables(_) => {
            // Success - variable inspector handled the request
        }
        _ => panic!("Expected Variables response"),
    }
}
