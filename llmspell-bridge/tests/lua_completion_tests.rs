//! Comprehensive tests for Lua completion provider

mod test_helpers;

#[cfg(all(test, feature = "lua"))]
mod lua_completion_tests {
    use crate::test_helpers::create_test_infrastructure;
    use llmspell_bridge::engine::bridge::{CompletionContext, CompletionKind, ScriptEngineBridge};
    use llmspell_bridge::engine::factory::LuaConfig;
    use llmspell_bridge::lua::completion::LuaCompletionProvider;
    use llmspell_bridge::lua::engine::LuaEngine;
    use llmspell_bridge::ComponentRegistry;
    use llmspell_bridge::ProviderManager;
    use llmspell_config::providers::ProviderManagerConfig;
    use mlua::Lua;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    fn setup_lua_engine() -> LuaEngine {
        let config = LuaConfig::default();
        let mut engine = LuaEngine::new(&config).unwrap();
        let registry = Arc::new(ComponentRegistry::new());
        let provider_config = ProviderManagerConfig::default();
        let providers =
            Arc::new(futures::executor::block_on(ProviderManager::new(provider_config)).unwrap());
        let (tool_registry, agent_registry, workflow_factory) = create_test_infrastructure();
        let _ = engine.inject_apis(
            &registry,
            &providers,
            &tool_registry,
            &agent_registry,
            &workflow_factory,
            None,
        );
        engine
    }

    #[test]
    fn test_global_symbols_extraction() {
        let lua = Lua::new();
        let provider = LuaCompletionProvider::new();

        // Test getting all global symbols
        let symbols = provider.get_global_symbols(&lua, "");

        // Verify standard Lua globals are present
        assert!(symbols.iter().any(|s| s.text == "print"));
        assert!(symbols.iter().any(|s| s.text == "require"));
        assert!(symbols.iter().any(|s| s.text == "type"));
        assert!(symbols.iter().any(|s| s.text == "tostring"));
        assert!(symbols.iter().any(|s| s.text == "pairs"));
        assert!(symbols.iter().any(|s| s.text == "ipairs"));
        assert!(symbols.iter().any(|s| s.text == "math"));
        assert!(symbols.iter().any(|s| s.text == "string"));
        assert!(symbols.iter().any(|s| s.text == "table"));
        assert!(symbols.iter().any(|s| s.text == "io"));
        assert!(symbols.iter().any(|s| s.text == "os"));

        // Verify function signatures
        let print_sym = symbols.iter().find(|s| s.text == "print").unwrap();
        assert_eq!(print_sym.kind, CompletionKind::Function);
        assert_eq!(print_sym.signature, Some("print(...)".to_string()));
    }

    #[test]
    fn test_prefix_filtering() {
        let lua = Lua::new();
        let provider = LuaCompletionProvider::new();

        // Test filtering with "pr" prefix
        let symbols = provider.get_global_symbols(&lua, "pr");
        assert!(symbols.iter().all(|s| s.text.starts_with("pr")));
        assert!(symbols.iter().any(|s| s.text == "print"));

        // Test filtering with "to" prefix
        let symbols = provider.get_global_symbols(&lua, "to");
        assert!(symbols.iter().all(|s| s.text.starts_with("to")));
        assert!(symbols.iter().any(|s| s.text == "tostring"));
        assert!(symbols.iter().any(|s| s.text == "tonumber"));
    }

    #[test]
    fn test_table_member_completion() {
        let lua = Lua::new();
        let provider = LuaCompletionProvider::new();

        // Test math table members
        let math_members = provider.get_table_members(&lua, "math", "");
        assert!(math_members.iter().any(|m| m.text == "abs"));
        assert!(math_members.iter().any(|m| m.text == "floor"));
        assert!(math_members.iter().any(|m| m.text == "ceil"));
        assert!(math_members.iter().any(|m| m.text == "sin"));
        assert!(math_members.iter().any(|m| m.text == "cos"));
        assert!(math_members.iter().any(|m| m.text == "random"));

        // Verify signatures for math functions
        let abs_member = math_members.iter().find(|m| m.text == "abs").unwrap();
        assert_eq!(abs_member.kind, CompletionKind::Method);
        assert_eq!(abs_member.signature, Some("math.abs(x)".to_string()));

        // Test string table members
        let string_members = provider.get_table_members(&lua, "string", "");
        assert!(string_members.iter().any(|m| m.text == "sub"));
        assert!(string_members.iter().any(|m| m.text == "find"));
        assert!(string_members.iter().any(|m| m.text == "format"));
        assert!(string_members.iter().any(|m| m.text == "upper"));
        assert!(string_members.iter().any(|m| m.text == "lower"));

        // Test table module members
        let table_members = provider.get_table_members(&lua, "table", "");
        assert!(table_members.iter().any(|m| m.text == "insert"));
        assert!(table_members.iter().any(|m| m.text == "remove"));
        assert!(table_members.iter().any(|m| m.text == "sort"));
        assert!(table_members.iter().any(|m| m.text == "concat"));
    }

    #[test]
    fn test_table_member_filtering() {
        let lua = Lua::new();
        let provider = LuaCompletionProvider::new();

        // Test filtering math members with "fl" prefix
        let math_members = provider.get_table_members(&lua, "math", "fl");
        assert!(math_members.iter().all(|m| m.text.starts_with("fl")));
        assert!(math_members.iter().any(|m| m.text == "floor"));

        // Test filtering string members with "s" prefix
        let string_members = provider.get_table_members(&lua, "string", "s");
        assert!(string_members.iter().all(|m| m.text.starts_with('s')));
        assert!(string_members.iter().any(|m| m.text == "sub"));
    }

    #[test]
    fn test_completion_context_scenarios() {
        let engine = setup_lua_engine();

        // Test simple global completion
        let ctx = CompletionContext::new("pri", 3);
        let completions = engine.get_completion_candidates(&ctx);
        assert!(completions.iter().any(|c| c.text == "print"));

        // Test member access completion for math
        let ctx = CompletionContext::new("math.", 5);
        let completions = engine.get_completion_candidates(&ctx);
        assert!(completions.iter().any(|c| c.text == "abs"));
        assert!(completions.iter().any(|c| c.text == "floor"));

        // Test member access with partial text
        let ctx = CompletionContext::new("math.fl", 7);
        let completions = engine.get_completion_candidates(&ctx);
        assert!(completions.iter().any(|c| c.text == "floor"));
        assert!(!completions.iter().any(|c| c.text == "abs"));

        // Test method call syntax (colon)
        let ctx = CompletionContext::new("str:s", 5);
        let completions = engine.get_completion_candidates(&ctx);
        assert!(completions.iter().any(|c| c.text == "sub"));

        // Test after equals sign (should invalidate cache)
        let ctx = CompletionContext::new("local x = ", 10);
        let completions = engine.get_completion_candidates(&ctx);
        // Should return globals and keywords
        assert!(!completions.is_empty());
    }

    #[test]
    fn test_keyword_completions() {
        let engine = setup_lua_engine();

        // Test at beginning of line
        let ctx = CompletionContext::new("", 0);
        let completions = engine.get_completion_candidates(&ctx);

        // Should include keywords
        assert!(completions
            .iter()
            .any(|c| c.text == "if" && c.kind == CompletionKind::Keyword));
        assert!(completions
            .iter()
            .any(|c| c.text == "for" && c.kind == CompletionKind::Keyword));
        assert!(completions
            .iter()
            .any(|c| c.text == "while" && c.kind == CompletionKind::Keyword));
        assert!(completions
            .iter()
            .any(|c| c.text == "function" && c.kind == CompletionKind::Keyword));
        assert!(completions
            .iter()
            .any(|c| c.text == "local" && c.kind == CompletionKind::Keyword));

        // Test after 'local'
        let ctx = CompletionContext::new("local ", 6);
        let completions = engine.get_completion_candidates(&ctx);
        assert!(completions
            .iter()
            .any(|c| c.text == "function" && c.kind == CompletionKind::Keyword));
    }

    #[test]
    fn test_caching_behavior() {
        let lua = Lua::new();
        let provider = LuaCompletionProvider::new();

        // First call should populate cache
        let symbols1 = provider.get_global_symbols(&lua, "");
        let initial_count = symbols1.len();

        // Second call should use cache (within 5 seconds)
        let symbols2 = provider.get_global_symbols(&lua, "");
        assert_eq!(symbols2.len(), initial_count);

        // Invalidate cache
        provider.invalidate_cache();

        // After invalidation, should refresh
        let symbols3 = provider.get_global_symbols(&lua, "");
        assert_eq!(symbols3.len(), initial_count);
    }

    #[test]
    fn test_custom_globals() {
        let lua = Lua::new();
        let provider = LuaCompletionProvider::new();

        // Add custom global
        lua.globals()
            .set(
                "myCustomFunction",
                lua.create_function(|_, ()| Ok(())).unwrap(),
            )
            .unwrap();
        lua.globals()
            .set("myCustomTable", lua.create_table().unwrap())
            .unwrap();

        // Should find custom globals
        let symbols = provider.get_global_symbols(&lua, "my");
        assert!(symbols
            .iter()
            .any(|s| s.text == "myCustomFunction" && s.kind == CompletionKind::Function));
        assert!(symbols
            .iter()
            .any(|s| s.text == "myCustomTable" && s.kind == CompletionKind::Module));
    }

    #[test]
    fn test_private_symbol_filtering() {
        let lua = Lua::new();
        let provider = LuaCompletionProvider::new();

        // Add private globals
        lua.globals().set("_privateVar", 42).unwrap();
        lua.globals().set("__doublePrivate", "hidden").unwrap();

        // Should filter out private symbols except _G and _VERSION
        let symbols = provider.get_global_symbols(&lua, "");
        assert!(!symbols.iter().any(|s| s.text == "_privateVar"));
        assert!(!symbols.iter().any(|s| s.text == "__doublePrivate"));
        assert!(symbols.iter().any(|s| s.text == "_G"));
        assert!(symbols.iter().any(|s| s.text == "_VERSION"));
    }

    #[test]
    fn test_thread_safety() {
        let engine = Arc::new(setup_lua_engine());
        let mut handles = vec![];

        // Spawn multiple threads trying to get completions simultaneously
        for i in 0..10 {
            let engine_clone = Arc::clone(&engine);
            let handle = thread::spawn(move || {
                let ctx = CompletionContext::new(&format!("pr{}", i % 2), 3);
                let completions = engine_clone.get_completion_candidates(&ctx);

                // Should either get completions or empty (if busy)
                if !completions.is_empty() {
                    assert!(completions.iter().any(|c| c.text.starts_with("pr")));
                }
            });
            handles.push(handle);
        }

        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }
    }

    #[test]
    fn test_completion_with_busy_engine() {
        let engine = Arc::new(setup_lua_engine());

        // Simulate busy engine by holding a lock
        let engine_clone = Arc::clone(&engine);
        let handle = thread::spawn(move || {
            // Execute a script to hold the Lua lock
            let _ = futures::executor::block_on(
                engine_clone.execute_script("for i=1,1000000 do local x = i end return 'done'"),
            );
        });

        // Try to get completions while engine is busy
        thread::sleep(Duration::from_millis(5));
        let ctx = CompletionContext::new("print", 5);
        let completions = engine.get_completion_candidates(&ctx);

        // Should return empty when engine is busy (timeout after 10ms)
        // or completions if the other thread finished
        assert!(completions.is_empty() || completions.iter().any(|c| c.text == "print"));

        handle.join().unwrap();
    }

    #[test]
    fn test_io_and_os_completions() {
        let lua = Lua::new();
        let provider = LuaCompletionProvider::new();

        // Test io module members
        let io_members = provider.get_table_members(&lua, "io", "");
        assert!(io_members.iter().any(|m| m.text == "open"));
        assert!(io_members.iter().any(|m| m.text == "close"));
        assert!(io_members.iter().any(|m| m.text == "read"));
        assert!(io_members.iter().any(|m| m.text == "write"));

        let open_member = io_members.iter().find(|m| m.text == "open").unwrap();
        assert_eq!(
            open_member.signature,
            Some("io.open(filename, mode?)".to_string())
        );

        // Test os module members
        let os_members = provider.get_table_members(&lua, "os", "");
        assert!(os_members.iter().any(|m| m.text == "execute"));
        assert!(os_members.iter().any(|m| m.text == "getenv"));
        assert!(os_members.iter().any(|m| m.text == "time"));
        assert!(os_members.iter().any(|m| m.text == "date"));
    }

    #[test]
    fn test_string_method_completions() {
        let lua = Lua::new();
        let provider = LuaCompletionProvider::new();

        // Test string methods (accessed with colon)
        let methods = provider.get_object_methods(&lua, "someString", "");
        assert!(methods.iter().any(|m| m.text == "sub"));
        assert!(methods.iter().any(|m| m.text == "upper"));
        assert!(methods.iter().any(|m| m.text == "lower"));
        assert!(methods.iter().any(|m| m.text == "find"));
        assert!(methods.iter().any(|m| m.text == "format"));

        // Verify all are marked as methods
        assert!(methods.iter().all(|m| m.kind == CompletionKind::Method));

        // Verify signatures
        let sub_method = methods.iter().find(|m| m.text == "sub").unwrap();
        assert_eq!(
            sub_method.signature,
            Some("string.sub(s, i, j?)".to_string())
        );
    }

    #[test]
    fn test_empty_completions() {
        let engine = setup_lua_engine();

        // Test with non-existent table
        let ctx = CompletionContext::new("nonexistent.", 12);
        let completions = engine.get_completion_candidates(&ctx);
        assert!(completions.is_empty());

        // Test with partial input that should give completions
        let ctx = CompletionContext::new("p", 1);
        let completions = engine.get_completion_candidates(&ctx);
        // Should return completions starting with 'p' like print, pairs, pcall
        assert!(
            !completions.is_empty(),
            "Expected completions for 'p' prefix"
        );
        assert!(completions.iter().any(|c| c.text == "print"));
    }

    #[test]
    fn test_function_argument_completions() {
        let engine = setup_lua_engine();

        // Test completion inside function arguments
        let ctx = CompletionContext::new("print(", 6);
        assert!(ctx.is_inside_function_args());
        assert_eq!(ctx.get_function_context(), Some("print".to_string()));
        let completions = engine.get_completion_candidates(&ctx);
        // Should provide globals and keywords as potential arguments
        assert!(!completions.is_empty());
        assert!(completions
            .iter()
            .any(|c| c.text == "true" && c.kind == CompletionKind::Keyword));
        assert!(completions
            .iter()
            .any(|c| c.text == "false" && c.kind == CompletionKind::Keyword));
        assert!(completions
            .iter()
            .any(|c| c.text == "nil" && c.kind == CompletionKind::Keyword));

        // Test completion with partial text inside arguments
        let ctx = CompletionContext::new("print(pr", 8);
        assert!(ctx.is_inside_function_args());
        let completions = engine.get_completion_candidates(&ctx);
        // Should filter to items starting with "pr"
        assert!(completions.iter().any(|c| c.text == "print"));
        assert!(!completions.iter().any(|c| c.text == "true")); // Filtered out

        // Test nested function calls
        let ctx = CompletionContext::new("print(tostring(", 15);
        assert!(ctx.is_inside_function_args());
        assert_eq!(ctx.get_function_context(), Some("tostring".to_string()));
        let completions = engine.get_completion_candidates(&ctx);
        assert!(!completions.is_empty());

        // Test multiple arguments
        let ctx = CompletionContext::new("math.max(5, ", 12);
        assert!(ctx.is_inside_function_args());
        let completions = engine.get_completion_candidates(&ctx);
        assert!(!completions.is_empty());

        // Test closed parentheses (not inside)
        let ctx = CompletionContext::new("print() ", 8);
        assert!(!ctx.is_inside_function_args());
        let completions = engine.get_completion_candidates(&ctx);
        // Should get normal completions
        assert!(!completions.is_empty());
    }

    #[test]
    fn test_parentheses_detection() {
        // Test various parentheses scenarios
        let ctx = CompletionContext::new("(", 1);
        assert!(ctx.is_inside_function_args());

        let ctx = CompletionContext::new("()", 1);
        assert!(ctx.is_inside_function_args());

        let ctx = CompletionContext::new("()", 2);
        assert!(!ctx.is_inside_function_args());

        let ctx = CompletionContext::new("((", 2);
        assert!(ctx.is_inside_function_args());

        let ctx = CompletionContext::new("(())", 3);
        assert!(ctx.is_inside_function_args());

        let ctx = CompletionContext::new("(())", 4);
        assert!(!ctx.is_inside_function_args());

        // Test with actual function names
        let ctx = CompletionContext::new("func(", 5);
        assert!(ctx.is_inside_function_args());
        assert_eq!(ctx.get_function_context(), Some("func".to_string()));

        let ctx = CompletionContext::new("obj.method(", 11);
        assert!(ctx.is_inside_function_args());
        assert_eq!(ctx.get_function_context(), Some("obj.method".to_string()));

        let ctx = CompletionContext::new("a = func(", 9);
        assert!(ctx.is_inside_function_args());
        assert_eq!(ctx.get_function_context(), Some("func".to_string()));
    }
}
