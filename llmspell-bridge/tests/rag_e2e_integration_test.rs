//! End-to-end RAG integration tests
//! Tests complete RAG functionality from CLI to storage

#![cfg(feature = "integration-tests")]

use llmspell_bridge::runtime::ScriptRuntime;
use llmspell_config::{
    rag::{
        ChunkingConfig, ChunkingStrategy, DistanceMetric, EmbeddingConfig, HNSWConfig, RAGConfig,
        VectorBackend, VectorStorageConfig,
    },
    LLMSpellConfig,
};
use std::sync::Arc;
use tempfile::TempDir;
use tokio::time::{timeout, Duration};

/// Test helper to create a test RAG configuration
fn create_test_rag_config(backend: &str, multi_tenant: bool) -> RAGConfig {
    RAGConfig {
        enabled: true,
        multi_tenant,
        vector_storage: VectorStorageConfig {
            dimensions: 384,
            backend: match backend {
                "hnsw" => VectorBackend::HNSW,
                _ => VectorBackend::Mock,
            },
            persistence_path: None,
            max_memory_mb: Some(512),
            hnsw: HNSWConfig {
                m: 8,
                ef_construction: 50,
                ef_search: 25,
                max_elements: 10000,
                metric: DistanceMetric::Cosine,
                allow_replace_deleted: true,
                num_threads: Some(2),
                seed: None,
            },
        },
        embedding: EmbeddingConfig {
            default_provider: "mock".to_string(),
            cache_enabled: true,
            cache_size: 100,
            cache_ttl_seconds: 60,
            batch_size: 4,
            timeout_seconds: 10,
            max_retries: 1,
        },
        chunking: ChunkingConfig {
            strategy: ChunkingStrategy::SlidingWindow,
            chunk_size: 256,
            overlap: 32,
            max_chunk_size: 1024,
            min_chunk_size: 50,
        },
        ..Default::default()
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_rag_cli_to_storage_flow() {
    let _temp_dir = TempDir::new().unwrap();

    // Create a test configuration
    let config = LLMSpellConfig {
        rag: create_test_rag_config("hnsw", false),
        ..Default::default()
    };

    // Create runtime with config
    let runtime = Arc::new(
        ScriptRuntime::new_with_lua(config.clone())
            .await
            .expect("Failed to create runtime"),
    );

    // Test script that uses RAG
    let test_script = r#"
        -- Test RAG functionality
        if not RAG then
            error("RAG not available")
        end
        
        -- Test ingestion
        local doc_id = RAG.ingest({
            content = "The quick brown fox jumps over the lazy dog",
            metadata = { source = "test", type = "example" }
        })
        
        if not doc_id then
            error("Failed to ingest document")
        end
        
        print("Document ingested:", doc_id)
        
        -- Test search
        local results = RAG.search({
            query = "fox",
            top_k = 5
        })
        
        if not results then
            error("Search failed")
        end
        
        print("Search results:", #results)
        
        -- Verify we got results
        if #results == 0 then
            error("No results found")
        end
        
        -- Test result structure
        local result = results[1]
        if not result.score or not result.content then
            error("Invalid result structure")
        end
        
        print("Test completed successfully")
    "#;

    // Execute script with timeout
    let result = timeout(Duration::from_secs(30), runtime.execute_script(test_script)).await;

    assert!(result.is_ok(), "Script execution timed out");
    assert!(result.unwrap().is_ok(), "Script execution failed");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_rag_multi_tenant_isolation() {
    let config = LLMSpellConfig {
        rag: create_test_rag_config("hnsw", true),
        ..Default::default()
    };

    let runtime = Arc::new(
        ScriptRuntime::new_with_lua(config)
            .await
            .expect("Failed to create runtime"),
    );

    let test_script = r#"
        -- Test multi-tenant isolation
        if not RAG then
            error("RAG not available")
        end
        
        -- Ingest to tenant1
        local doc1 = RAG.ingest({
            content = "Tenant 1 private data",
            metadata = { tenant = "tenant1" },
            tenant_id = "tenant1"
        })
        
        -- Ingest to tenant2
        local doc2 = RAG.ingest({
            content = "Tenant 2 confidential information",
            metadata = { tenant = "tenant2" },
            tenant_id = "tenant2"
        })
        
        -- Search as tenant1 - should only see tenant1 data
        local results1 = RAG.search({
            query = "data",
            tenant_id = "tenant1",
            top_k = 10
        })
        
        -- Verify isolation
        for _, result in ipairs(results1) do
            if result.content:find("Tenant 2") then
                error("Tenant isolation violated!")
            end
        end
        
        print("Multi-tenant isolation verified")
    "#;

    let result = runtime.execute_script(test_script).await;
    assert!(result.is_ok(), "Multi-tenant test failed");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_rag_configuration_loading() {
    // Test different configuration scenarios
    let configs = vec![
        ("default", create_test_rag_config("hnsw", false)),
        ("multi-tenant", create_test_rag_config("hnsw", true)),
        ("mock-backend", create_test_rag_config("mock", false)),
    ];

    for (name, rag_config) in configs {
        let config = LLMSpellConfig {
            rag: rag_config,
            ..Default::default()
        };

        let runtime = Arc::new(
            ScriptRuntime::new_with_lua(config)
                .await
                .expect("Failed to create runtime"),
        );

        let test_script = r#"
            if not RAG then
                error("RAG not initialized")
            end
            print("Configuration loaded successfully")
        "#;

        let result = runtime.execute_script(test_script).await;
        assert!(result.is_ok(), "Failed to load configuration: {name}");
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_rag_performance_benchmarks() {
    use std::time::Instant;

    let config = LLMSpellConfig {
        rag: create_test_rag_config("hnsw", false),
        ..Default::default()
    };

    let runtime = Arc::new(
        ScriptRuntime::new_with_lua(config)
            .await
            .expect("Failed to create runtime"),
    );

    // Measure startup time
    let start = Instant::now();
    let init_script = r#"
        if not RAG then
            error("RAG not available")
        end
    "#;
    runtime.execute_script(init_script).await.unwrap();
    let startup_time = start.elapsed();

    println!("RAG startup time: {startup_time:?}");
    assert!(startup_time < Duration::from_secs(5), "Startup too slow");

    // Measure ingestion performance
    let ingest_script = r#"
        local start = os.clock()
        for i = 1, 100 do
            RAG.ingest({
                content = "Document " .. i .. " with some test content",
                metadata = { index = i }
            })
        end
        local elapsed = os.clock() - start
        print("Ingestion time:", elapsed)
        return elapsed
    "#;

    let result = runtime.execute_script(ingest_script).await;
    assert!(result.is_ok(), "Ingestion benchmark failed");

    // Measure search performance
    let search_script = r#"
        local start = os.clock()
        for i = 1, 50 do
            RAG.search({
                query = "test content " .. i,
                top_k = 10
            })
        end
        local elapsed = os.clock() - start
        print("Search time:", elapsed)
        return elapsed
    "#;

    let result = runtime.execute_script(search_script).await;
    assert!(result.is_ok(), "Search benchmark failed");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_rag_error_handling() {
    let config = LLMSpellConfig {
        rag: create_test_rag_config("hnsw", false),
        ..Default::default()
    };

    let runtime = Arc::new(
        ScriptRuntime::new_with_lua(config)
            .await
            .expect("Failed to create runtime"),
    );

    // Test various error scenarios
    let error_tests = vec![
        (
            "invalid_ingest",
            r#"
                local ok, err = pcall(function()
                    RAG.ingest({})  -- Missing required fields
                end)
                if ok then
                    error("Should have failed on invalid input")
                end
                print("Error handled:", err)
            "#,
        ),
        (
            "invalid_search",
            r#"
                local ok, err = pcall(function()
                    RAG.search({
                        top_k = -1  -- Invalid parameter
                    })
                end)
                if ok then
                    error("Should have failed on invalid top_k")
                end
                print("Error handled:", err)
            "#,
        ),
        (
            "missing_query",
            r#"
                local ok, err = pcall(function()
                    RAG.search({ top_k = 5 })  -- Missing query
                end)
                if ok then
                    error("Should have failed on missing query")
                end
                print("Error handled:", err)
            "#,
        ),
    ];

    for (name, script) in error_tests {
        let result = runtime.execute_script(script).await;
        assert!(result.is_ok(), "Error handling test failed: {name}");
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_rag_persistence() {
    let temp_dir = TempDir::new().unwrap();
    let persistence_path = temp_dir.path().join("vectors");

    let mut rag_config = create_test_rag_config("hnsw", false);
    rag_config.vector_storage.persistence_path = Some(persistence_path.clone());
    let config = LLMSpellConfig {
        rag: rag_config,
        ..Default::default()
    };

    // First runtime - ingest data
    {
        let runtime = Arc::new(
            ScriptRuntime::new_with_lua(config.clone())
                .await
                .expect("Failed to create runtime"),
        );

        let script = r#"
            RAG.ingest({
                content = "Persistent data that should survive restart",
                metadata = { persistent = true }
            })
            print("Data ingested")
            
            -- Save the vector storage to disk (currently no-op)
            RAG.save()
            print("Data saved - note: save is currently no-op")
        "#;

        runtime.execute_script(script).await.unwrap();

        // Drop the runtime to trigger save
        drop(runtime);

        // Give the save task a moment to complete
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    // Second runtime - verify data persisted
    {
        let runtime = Arc::new(
            ScriptRuntime::new_with_lua(config)
                .await
                .expect("Failed to create runtime"),
        );

        let script = r#"
            local results = RAG.search({
                query = "persistent data",
                top_k = 1
            })
            
            if #results == 0 then
                error("Persisted data not found!")
            end
            
            print("Persistence verified")
        "#;

        runtime.execute_script(script).await.unwrap();
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_rag_memory_limits() {
    let mut rag_config = create_test_rag_config("hnsw", false);
    rag_config.vector_storage.max_memory_mb = Some(10); // Very low limit
    let config = LLMSpellConfig {
        rag: rag_config,
        ..Default::default()
    };

    let runtime = Arc::new(
        ScriptRuntime::new_with_lua(config)
            .await
            .expect("Failed to create runtime"),
    );

    let script = r#"
        -- Try to exceed memory limit
        local count = 0
        local ok = true
        
        while ok and count < 10000 do
            ok = pcall(function()
                RAG.ingest({
                    content = string.rep("Large document content ", 1000),
                    metadata = { index = count }
                })
            end)
            count = count + 1
        end
        
        print("Ingested documents before limit:", count - 1)
        
        -- System should still be functional
        local results = RAG.search({
            query = "document",
            top_k = 5
        })
        
        print("Search still works:", #results > 0)
    "#;

    let result = runtime.execute_script(script).await;
    assert!(result.is_ok(), "Memory limit test failed");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_rag_concurrent_operations() {
    let config = LLMSpellConfig {
        rag: create_test_rag_config("hnsw", false),
        ..Default::default()
    };

    let runtime = Arc::new(
        ScriptRuntime::new_with_lua(config)
            .await
            .expect("Failed to create runtime"),
    );

    let script = r#"
        -- Simulate concurrent operations
        local operations = {}
        
        -- Queue up operations
        for i = 1, 10 do
            table.insert(operations, function()
                RAG.ingest({
                    content = "Concurrent document " .. i,
                    metadata = { thread = i }
                })
            end)
        end
        
        for i = 1, 5 do
            table.insert(operations, function()
                RAG.search({
                    query = "concurrent",
                    top_k = 3
                })
            end)
        end
        
        -- Execute all operations (simulated concurrency)
        local errors = 0
        for _, op in ipairs(operations) do
            local ok, err = pcall(op)
            if not ok then
                errors = errors + 1
                print("Operation failed:", err)
            end
        end
        
        if errors > 0 then
            error("Concurrent operations had " .. errors .. " failures")
        end
        
        print("All concurrent operations succeeded")
    "#;

    let result = runtime.execute_script(script).await;
    assert!(result.is_ok(), "Concurrent operations test failed");
}

#[tokio::test(flavor = "multi_thread")]
async fn test_rag_cleanup_and_shutdown() {
    let config = LLMSpellConfig {
        rag: create_test_rag_config("hnsw", false),
        ..Default::default()
    };

    let runtime = Arc::new(
        ScriptRuntime::new_with_lua(config)
            .await
            .expect("Failed to create runtime"),
    );

    let script = r#"
        -- Ingest some data
        for i = 1, 10 do
            RAG.ingest({
                content = "Document for cleanup test " .. i,
                metadata = { test = "cleanup" }
            })
        end
        
        -- Clear/cleanup operations (if available)
        if RAG.clear then
            RAG.clear()
            print("RAG cleared")
        end
        
        -- Verify cleanup
        local results = RAG.search({
            query = "cleanup test",
            top_k = 10
        })
        
        -- After clear, we should have no results (or fresh state)
        print("Results after cleanup:", #results)
    "#;

    let result = runtime.execute_script(script).await;
    assert!(result.is_ok(), "Cleanup test failed");

    // Runtime should shut down cleanly
    drop(runtime);
}
