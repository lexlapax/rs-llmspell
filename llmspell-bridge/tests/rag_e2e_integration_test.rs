//! End-to-end RAG integration tests
//! Tests complete RAG functionality from CLI to storage

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
fn create_test_rag_config(_backend: &str, multi_tenant: bool) -> RAGConfig {
    RAGConfig {
        enabled: true,
        multi_tenant,
        vector_storage: VectorStorageConfig {
            dimensions: 384,
            backend: VectorBackend::HNSW,
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
                nb_layers: None,
                parallel_batch_size: Some(32),
                enable_mmap: false,
                mmap_sync_interval: Some(60),
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
        local ingest_result = RAG.ingest({
            content = "The quick brown fox jumps over the lazy dog",
            metadata = { source = "test", type = "example" }
        }, {})
        
        if not ingest_result or not ingest_result.success then
            error("Failed to ingest document")
        end
        
        print("Document ingested:", type(ingest_result))
        
        -- Test search
        local search_response = RAG.search("fox", { k = 5 })
        
        if not search_response or not search_response.success then
            error("Search failed")
        end
        
        print("Search results:", search_response.total)
        
        -- Verify we got results
        if search_response.total == 0 then
            error("No results found")
        end
        
        -- Test result structure
        local result = search_response.results[1]
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
        local ingest1 = RAG.ingest({
            content = "Tenant 1 private data",
            metadata = { tenant = "tenant1" }
        }, {
            tenant_id = "tenant1"
        })
        
        -- Ingest to tenant2
        local ingest2 = RAG.ingest({
            content = "Tenant 2 confidential information",
            metadata = { tenant = "tenant2" }
        }, {
            tenant_id = "tenant2"
        })
        
        -- Search as tenant1 - should only see tenant1 data
        local search1 = RAG.search("data", {
            tenant_id = "tenant1",
            k = 10
        })
        
        -- Verify isolation
        for _, result in ipairs(search1.results) do
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
            RAG.search("test content " .. tostring(i), {
                k = 10
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
                    RAG.ingest(nil, {})  -- Invalid: nil documents
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
                    RAG.search("", {
                        k = -1  -- Invalid parameter
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
                    RAG.search(nil, { k = 5 })  -- Missing query
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
            local result = RAG.ingest({
                content = "Persistent data that should survive restart",
                metadata = { persistent = true }
            })
            print("Data ingested:", result and result.success)
            print("Vectors created:", result and result.vectors_created)
            
            -- Save the vector storage to disk
            RAG.save()
            print("Data saved to disk")
            
            -- Verify data is searchable before saving
            local search_before = RAG.search("persistent data", { k = 1 })
            print("Search before restart - found:", search_before and search_before.total or 0)
        "#;

        runtime.execute_script(script).await.unwrap();

        // Drop the runtime to trigger save
        drop(runtime);

        // Give the save task a moment to complete
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

        // Check if persistence directory exists
        println!("Checking persistence path: {persistence_path:?}");
        if persistence_path.exists() {
            println!("Path exists, contents:");
            for entry in std::fs::read_dir(&persistence_path).unwrap() {
                let entry = entry.unwrap();
                println!("  - {:?}", entry.path());
            }
        } else {
            println!("Path does not exist!");
        }
    }

    // Second runtime - verify data persisted
    {
        println!("\n=== Creating second runtime to load persisted data ===");
        let runtime = Arc::new(
            ScriptRuntime::new_with_lua(config)
                .await
                .expect("Failed to create runtime"),
        );

        let script = r#"
            print("Searching for persisted data...")
            local search_response = RAG.search("persistent data", {
                k = 1
            })
            
            print("Search response:", search_response and "exists" or "nil")
            if search_response then
                print("Total results:", search_response.total)
                print("Success:", search_response.success)
                if search_response.results and #search_response.results > 0 then
                    print("First result:", search_response.results[1].content)
                end
            end
            
            if not search_response or search_response.total == 0 then
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
        local search_response = RAG.search("document", {
            k = 5
        })
        
        print("Search still works:", search_response and search_response.total > 0)
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
                RAG.search("concurrent", {
                    k = 3
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
        local search_response = RAG.search("cleanup test", {
            k = 10
        })
        
        -- After clear, we should have no results (or fresh state)
        print("Results after cleanup:", search_response and search_response.total or 0)
    "#;

    let result = runtime.execute_script(script).await;
    assert!(result.is_ok(), "Cleanup test failed");

    // Runtime should shut down cleanly
    drop(runtime);
}
