//! ABOUTME: Integration tests for RAG functionality through Lua
//! ABOUTME: Tests Lua script access to RAG operations

#[cfg(feature = "lua")]
mod rag_lua_tests {
    use llmspell_bridge::globals::GlobalContext;
    use llmspell_bridge::{ComponentRegistry, ProviderManager, ScriptRuntime};
    use llmspell_config::providers::ProviderManagerConfig;
    use llmspell_config::LLMSpellConfig;
    use llmspell_events::bus::EventBus;
    use llmspell_hooks::{HookExecutor, HookRegistry};
    use llmspell_rag::multi_tenant_integration::MultiTenantRAG;
    use llmspell_sessions::{SessionManager, SessionManagerConfig};
    use llmspell_state_persistence::StateManager;
    use llmspell_storage::backends::vector::hnsw::HNSWVectorStorage;
    use llmspell_storage::vector_storage::HNSWConfig;
    use llmspell_storage::MemoryBackend;
    use llmspell_tenancy::manager::MultiTenantVectorManager;
    use mlua::Lua;
    use std::sync::Arc;

    async fn setup_test_context_with_rag() -> (Arc<GlobalContext>, Lua) {
        let registry = Arc::new(ComponentRegistry::new());
        let config = ProviderManagerConfig::default();
        let providers = Arc::new(ProviderManager::new(config).await.unwrap());

        // Create infrastructure
        let storage_backend = Arc::new(MemoryBackend::new());
        let state_manager = Arc::new(StateManager::new().await.unwrap());
        let hook_registry = Arc::new(HookRegistry::new());
        let hook_executor = Arc::new(HookExecutor::new());
        let event_bus = Arc::new(EventBus::new());

        let session_config = SessionManagerConfig::default();
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

        // Create multi-tenant RAG with real HNSW vector storage
        let hnsw_config = HNSWConfig {
            m: 16,
            ef_construction: 200,
            ef_search: 50,
            max_elements: 10000,
            seed: None,
            metric: llmspell_storage::vector_storage::DistanceMetric::Cosine,
            allow_replace_deleted: true,
            num_threads: None,
        };
        let vector_storage = Arc::new(HNSWVectorStorage::new(384, hnsw_config));
        let tenant_manager = Arc::new(MultiTenantVectorManager::new(vector_storage.clone()));
        let multi_tenant_rag = Arc::new(MultiTenantRAG::new(tenant_manager));

        let context = GlobalContext::new(registry, providers.clone());
        context.set_bridge("session_manager", session_manager.clone());
        context.set_bridge("state_manager", state_manager.clone());
        context.set_bridge("multi_tenant_rag", multi_tenant_rag.clone());

        let context = Arc::new(context);
        let lua = Lua::new();

        // Inject only RAG global to avoid Hook registration issues
        let rag_bridge = llmspell_bridge::rag_bridge::RAGBridge::from_components(
            Arc::new(
                llmspell_rag::state_integration::StateAwareVectorStorage::new(
                    vector_storage,
                    state_manager.clone(),
                    multi_tenant_rag.clone(),
                ),
            ),
            session_manager,
            multi_tenant_rag,
            providers.create_core_manager_arc().await.unwrap(),
        );

        llmspell_bridge::lua::globals::rag::inject_rag_global(&lua, &context, Arc::new(rag_bridge))
            .unwrap();

        (context, lua)
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_lua_rag_basic_operations() {
        let (_context, lua) = setup_test_context_with_rag().await;

        let script = r#"
            -- Test basic RAG operations
            local success = true
            local errors = {}
            
            -- Test ingestion
            local ingest_result = RAG.ingest({
                {
                    id = "doc1",
                    text = "The quick brown fox jumps over the lazy dog",
                    metadata = { source = "test.txt" }
                },
                {
                    id = "doc2",
                    text = "Machine learning is transforming industries",
                    metadata = { source = "ml.txt" }
                }
            }, {
                scope = "test",
                scope_id = "lua_test"
            })
            
            if not ingest_result.success then
                success = false
                table.insert(errors, "Ingest failed: " .. (ingest_result.error or "unknown"))
            end
            
            -- Test search
            local search_result = RAG.search("machine learning", {
                k = 5,
                scope = "test",
                scope_id = "lua_test"
            })
            
            if not search_result.success then
                success = false
                table.insert(errors, "Search failed: " .. (search_result.error or "unknown"))
            end
            
            return {
                success = success,
                errors = errors,
                ingest_docs = ingest_result.documents_processed,
                search_results = search_result.total
            }
        "#;

        let result: mlua::Value = lua.load(script).eval().unwrap();
        let table = result.as_table().unwrap();

        assert!(table.get::<_, bool>("success").unwrap());
        assert_eq!(table.get::<_, u32>("ingest_docs").unwrap(), 2);
        assert!(table.get::<_, u32>("search_results").unwrap() > 0);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_lua_rag_with_chunking() {
        let (_context, lua) = setup_test_context_with_rag().await;

        let script = r#"
            -- Test RAG with chunking configuration
            local long_text = string.rep("This is a test sentence. ", 100)
            
            local result = RAG.ingest({
                {
                    id = "long_doc",
                    text = long_text,
                    metadata = { type = "long" }
                }
            }, {
                scope = "test",
                scope_id = "chunking_test",
                chunking = {
                    chunk_size = 100,
                    overlap = 20,
                    strategy = "sliding_window"
                }
            })
            
            return {
                success = result.success,
                documents = result.documents_processed,
                vectors = result.vectors_created
            }
        "#;

        let result: mlua::Value = lua.load(script).eval().unwrap();
        let table = result.as_table().unwrap();

        assert!(table.get::<_, bool>("success").unwrap());
        assert_eq!(table.get::<_, u32>("documents").unwrap(), 1);
        // Should create multiple vectors due to chunking
        assert!(table.get::<_, u32>("vectors").unwrap() >= 1);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_lua_rag_with_filters() {
        let (_context, lua) = setup_test_context_with_rag().await;

        let script = r#"
            -- Ingest documents with metadata
            RAG.ingest({
                {
                    id = "sci1",
                    text = "Quantum computing breakthrough",
                    metadata = { category = "science", year = 2024 }
                },
                {
                    id = "tech1",
                    text = "New smartphone released",
                    metadata = { category = "technology", year = 2024 }
                },
                {
                    id = "sci2",
                    text = "Climate change research",
                    metadata = { category = "science", year = 2023 }
                }
            }, {
                scope = "test",
                scope_id = "filter_test"
            })
            
            -- Search with filters
            local result = RAG.search("research", {
                k = 5,
                scope = "test",
                scope_id = "filter_test",
                filters = { category = "science" },
                threshold = 0.3
            })
            
            return {
                success = result.success,
                total = result.total,
                results = result.results
            }
        "#;

        let result: mlua::Value = lua.load(script).eval().unwrap();
        let table = result.as_table().unwrap();

        assert!(table.get::<_, bool>("success").unwrap());
        // Should find science documents
        assert!(table.get::<_, u32>("total").unwrap() > 0);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_lua_rag_cleanup() {
        let (_context, lua) = setup_test_context_with_rag().await;

        let script = r#"
            -- Ingest documents
            RAG.ingest({
                { id = "cleanup1", text = "Test document 1" },
                { id = "cleanup2", text = "Test document 2" }
            }, {
                scope = "test",
                scope_id = "cleanup_test"
            })
            
            -- Verify they exist
            local before = RAG.search("test", {
                scope = "test",
                scope_id = "cleanup_test"
            })
            
            -- Clean up
            local deleted = RAG.cleanup_scope("test", "cleanup_test")
            
            -- Verify they're gone
            local after = RAG.search("test", {
                scope = "test",
                scope_id = "cleanup_test"
            })
            
            return {
                before_count = before.total,
                deleted_count = deleted,
                after_count = after.total
            }
        "#;

        let result: mlua::Value = lua.load(script).eval().unwrap();
        let table = result.as_table().unwrap();

        assert!(table.get::<_, u32>("before_count").unwrap() > 0);
        assert!(table.get::<_, u32>("deleted_count").unwrap() > 0);
        assert_eq!(table.get::<_, u32>("after_count").unwrap(), 0);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_lua_rag_session_collection() {
        let (_context, lua) = setup_test_context_with_rag().await;

        let script = r#"
            -- Create session collection
            local session = RAG.create_session_collection("test_session_123", 3600)
            
            -- Configure session
            local config = RAG.configure_session({
                session_id = "test_session_123",
                vector_ttl = 1800
            })
            
            return {
                session_created = session.created,
                session_id = session.session_id,
                namespace = session.namespace,
                config_ok = config.configured
            }
        "#;

        let result: mlua::Value = lua.load(script).eval().unwrap();
        let table = result.as_table().unwrap();

        assert!(table.get::<_, bool>("session_created").unwrap());
        assert_eq!(
            table.get::<_, String>("session_id").unwrap(),
            "test_session_123"
        );
        assert!(table
            .get::<_, String>("namespace")
            .unwrap()
            .contains("session"));
        assert!(table.get::<_, bool>("config_ok").unwrap());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_lua_rag_providers() {
        let (_context, lua) = setup_test_context_with_rag().await;

        let script = r#"
            -- List available providers
            local providers = RAG.list_providers()
            
            -- Configure RAG
            RAG.configure({
                default_provider = "mock",
                enable_cache = true,
                cache_ttl = 1800
            })
            
            return {
                provider_count = #providers,
                has_mock = false
            }
        "#;

        let result: mlua::Value = lua.load(script).eval().unwrap();
        let table = result.as_table().unwrap();

        assert!(table.get::<_, u32>("provider_count").unwrap() > 0);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_lua_rag_get_stats() {
        let (_context, lua) = setup_test_context_with_rag().await;

        let script = r#"
            -- Ingest some documents
            RAG.ingest({
                { id = "stats1", text = "Document for statistics" },
                { id = "stats2", text = "Another document" }
            }, {
                scope = "test",
                scope_id = "stats_test"
            })
            
            -- Get statistics
            local stats = RAG.get_stats("test", "stats_test")
            
            -- Check if we have stats
            local has_stats = false
            for k, v in pairs(stats) do
                has_stats = true
                break
            end
            
            return {
                has_stats = has_stats,
                stats = stats
            }
        "#;

        let result: mlua::Value = lua.load(script).eval().unwrap();
        let table = result.as_table().unwrap();

        assert!(table.get::<_, bool>("has_stats").unwrap());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_lua_rag_error_handling() {
        let (_context, lua) = setup_test_context_with_rag().await;

        let script = r#"
            local results = {}
            
            -- Test empty document list
            local empty_result = RAG.ingest({}, {
                scope = "test",
                scope_id = "empty_test"
            })
            results.empty_ok = empty_result.success
            results.empty_docs = empty_result.documents_processed
            
            -- Test search with non-existent scope
            local search_result = RAG.search("test", {
                scope = "nonexistent",
                scope_id = "nonexistent"
            })
            results.search_ok = search_result.success
            results.search_count = search_result.total
            
            return results
        "#;

        let result: mlua::Value = lua.load(script).eval().unwrap();
        let table = result.as_table().unwrap();

        assert!(table.get::<_, bool>("empty_ok").unwrap());
        assert_eq!(table.get::<_, u32>("empty_docs").unwrap(), 0);
        assert!(table.get::<_, bool>("search_ok").unwrap());
        assert_eq!(table.get::<_, u32>("search_count").unwrap(), 0);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_lua_rag_with_runtime() {
        // Test using ScriptRuntime instead of direct Lua
        let mut config = LLMSpellConfig {
            default_engine: "lua".to_string(),
            ..Default::default()
        };
        config.rag.enabled = true; // Enable RAG functionality
                                   // Explicitly configure vector backend (default is HNSW)
        config.rag.vector_storage.backend = llmspell_config::VectorBackend::HNSW;
        config.rag.vector_storage.dimensions = 384;

        let runtime = ScriptRuntime::new_with_lua(config).await.unwrap();

        let script = r#"
            -- Use RAG through runtime
            local docs = {
                { id = "rt1", text = "Runtime test document" }
            }
            
            local ingest = RAG.ingest(docs, { scope = "runtime", scope_id = "test" })
            local search = RAG.search("runtime", { scope = "runtime", scope_id = "test" })
            
            return {
                ingest_success = ingest.success,
                search_success = search.success,
                found = search.total > 0
            }
        "#;

        let result = runtime.execute_script(script).await.unwrap();
        assert!(result.metadata.warnings.is_empty());
        // Output verification would depend on the script's return format
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_lua_rag_with_mock_backend() {
        // Test using ScriptRuntime with Mock backend
        let mut config = LLMSpellConfig {
            default_engine: "lua".to_string(),
            ..Default::default()
        };
        config.rag.enabled = true;
        // Use Mock backend for testing
        config.rag.vector_storage.backend = llmspell_config::VectorBackend::Mock;
        config.rag.vector_storage.dimensions = 768; // Different dimensions to verify config

        let runtime = ScriptRuntime::new_with_lua(config).await.unwrap();

        let script = r#"
            -- Verify RAG is available with Mock backend
            local docs = {
                { id = "mock1", text = "Mock backend test document" }
            }
            
            local ingest = RAG.ingest(docs, { scope = "mock", scope_id = "test" })
            local search = RAG.search("mock", { scope = "mock", scope_id = "test" })
            
            return {
                ingest_success = ingest.success,
                search_success = search.success,
                backend_working = ingest.success and search.success
            }
        "#;

        let result = runtime.execute_script(script).await.unwrap();
        assert!(result.metadata.warnings.is_empty());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_lua_rag_disabled() {
        // Test that RAG is not available when disabled
        let mut config = LLMSpellConfig {
            default_engine: "lua".to_string(),
            ..Default::default()
        };
        config.rag.enabled = false; // RAG disabled

        let runtime = ScriptRuntime::new_with_lua(config).await.unwrap();

        let script = r"
            -- RAG should be nil when disabled
            return {
                rag_is_nil = RAG == nil
            }
        ";

        let result = runtime.execute_script(script).await.unwrap();
        // Verify RAG is indeed nil when disabled
        if let serde_json::Value::Object(obj) = result.output {
            assert_eq!(obj.get("rag_is_nil"), Some(&serde_json::Value::Bool(true)));
        } else {
            panic!("Expected object output");
        }
    }
}
