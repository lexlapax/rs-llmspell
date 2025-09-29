//! Large-scale tests for HNSW vector storage
//! These tests verify memory usage and load time with 100K vectors

use llmspell_core::state::StateScope;
use llmspell_storage::{
    backends::vector::hnsw::HNSWVectorStorage,
    vector_storage::{HNSWConfig, VectorEntry, VectorQuery, VectorStorage},
};
use std::time::Instant;
use tempfile::TempDir;

/// Helper to get current process memory usage in bytes
fn get_process_memory_bytes() -> usize {
    #[cfg(target_os = "macos")]
    {
        use std::process::Command;
        let output = Command::new("ps")
            .args(["-o", "rss=", "-p", &std::process::id().to_string()])
            .output()
            .expect("Failed to get memory info");
        let rss_kb = String::from_utf8_lossy(&output.stdout)
            .trim()
            .parse::<usize>()
            .unwrap_or(0);
        rss_kb * 1024 // Convert KB to bytes
    }
    #[cfg(not(target_os = "macos"))]
    {
        // For Linux, read from /proc/self/status
        use std::fs::read_to_string;
        let status = read_to_string("/proc/self/status").unwrap_or_default();
        for line in status.lines() {
            if let Some(rss_line) = line.strip_prefix("VmRSS:") {
                let kb = rss_line
                    .split_whitespace()
                    .next()
                    .and_then(|s| s.parse::<usize>().ok())
                    .unwrap_or(0);
                return kb * 1024;
            }
        }
        0
    }
}

/// Generate random vectors for testing
fn generate_random_vectors(count: usize, dimensions: usize) -> Vec<VectorEntry> {
    use rand::{thread_rng, Rng};
    let mut rng = thread_rng();

    (0..count)
        .map(|i| {
            let vector: Vec<f32> = (0..dimensions).map(|_| rng.gen_range(-1.0..1.0)).collect();
            VectorEntry::new(format!("vec_{}", i), vector).with_scope(StateScope::Global)
        })
        .collect()
}

#[tokio::test]
#[ignore] // Expensive test - run with --ignored
async fn test_hnsw_memory_usage_100k_vectors() {
    const VECTOR_COUNT: usize = 100_000;
    const DIMENSIONS: usize = 384;
    const MAX_MEMORY_GB: f64 = 1.0;

    println!(
        "Testing memory usage with {} vectors of {} dimensions",
        VECTOR_COUNT, DIMENSIONS
    );

    // Measure baseline memory
    let baseline_memory = get_process_memory_bytes();
    println!(
        "Baseline memory: {:.2} MB",
        baseline_memory as f64 / 1_048_576.0
    );

    // Create HNSW storage with configuration for large dataset
    let config = HNSWConfig {
        m: 32,
        ef_construction: 400,
        ef_search: 200,
        max_elements: VECTOR_COUNT + 10_000, // Add some buffer
        parallel_batch_size: Some(256),
        num_threads: Some(4),
        ..Default::default()
    };

    let storage = HNSWVectorStorage::new(DIMENSIONS, config);

    // Generate and insert vectors in batches
    println!("Generating and inserting {} vectors...", VECTOR_COUNT);
    let batch_size = 10_000;
    let start = Instant::now();

    for batch_idx in 0..(VECTOR_COUNT / batch_size) {
        let batch_start = batch_idx * batch_size;
        let batch_end = ((batch_idx + 1) * batch_size).min(VECTOR_COUNT);
        let batch = generate_random_vectors(batch_end - batch_start, DIMENSIONS);

        storage.insert(batch).await.expect("Failed to insert batch");

        if (batch_idx + 1).is_multiple_of(10) {
            let current_memory = get_process_memory_bytes();
            let memory_used = current_memory.saturating_sub(baseline_memory);
            println!(
                "Inserted {} vectors, memory used: {:.2} MB",
                batch_end,
                memory_used as f64 / 1_048_576.0
            );
        }
    }

    let insertion_time = start.elapsed();
    println!(
        "Insertion completed in {:.2}s",
        insertion_time.as_secs_f64()
    );

    // Measure peak memory usage
    let peak_memory = get_process_memory_bytes();
    let memory_used = peak_memory.saturating_sub(baseline_memory);
    let memory_used_mb = memory_used as f64 / 1_048_576.0;
    let memory_per_vector = memory_used / VECTOR_COUNT;

    println!("\n=== Memory Usage Results ===");
    println!("Total memory used: {:.2} MB", memory_used_mb);
    println!("Memory per vector: {} bytes", memory_per_vector);
    println!("Expected (2KB/vector): {} MB", VECTOR_COUNT * 2 / 1024);

    // Verify memory usage is under 1GB
    assert!(
        memory_used_mb < MAX_MEMORY_GB * 1024.0,
        "Memory usage ({:.2} MB) exceeds limit ({:.2} MB)",
        memory_used_mb,
        MAX_MEMORY_GB * 1024.0
    );

    // Test search performance with large index
    println!("\n=== Testing search performance ===");
    let query_vector = generate_random_vectors(1, DIMENSIONS)
        .into_iter()
        .next()
        .unwrap();
    let query = VectorQuery::new(query_vector.embedding, 100);

    let search_start = Instant::now();
    let results = storage.search(&query).await.expect("Search failed");
    let search_time = search_start.elapsed();

    println!(
        "Search returned {} results in {:.3}ms",
        results.len(),
        search_time.as_secs_f64() * 1000.0
    );
    assert!(!results.is_empty(), "Search should return results");
}

#[tokio::test]
#[ignore] // Expensive test - run with --ignored
async fn test_hnsw_load_time_100k_vectors() {
    const VECTOR_COUNT: usize = 100_000;
    const DIMENSIONS: usize = 384;
    const MAX_LOAD_TIME_SECS: u64 = 5;

    println!("Testing load time with {} vectors", VECTOR_COUNT);

    // Create a temporary directory for persistence
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let persistence_path = temp_dir.path().to_path_buf();

    // Phase 1: Create and persist 100K vectors
    println!(
        "Phase 1: Creating and persisting {} vectors...",
        VECTOR_COUNT
    );
    {
        let config = HNSWConfig {
            m: 32,
            ef_construction: 400,
            ef_search: 200,
            max_elements: VECTOR_COUNT + 10_000,
            parallel_batch_size: Some(256),
            num_threads: Some(4),
            ..Default::default()
        };

        let storage =
            HNSWVectorStorage::new(DIMENSIONS, config).with_persistence(persistence_path.clone());

        // Insert vectors in batches
        let batch_size = 10_000;
        for _batch_idx in 0..(VECTOR_COUNT / batch_size) {
            let batch = generate_random_vectors(batch_size, DIMENSIONS);
            storage.insert(batch).await.expect("Failed to insert batch");
        }

        // Explicitly save to disk
        storage.save().await.expect("Failed to save index");
        println!("Index saved to disk");
    }

    // Phase 2: Load the persisted index and measure time
    println!("\nPhase 2: Loading persisted index...");
    let load_start = Instant::now();

    let config = HNSWConfig {
        m: 32,
        ef_construction: 400,
        ef_search: 200,
        max_elements: VECTOR_COUNT + 10_000,
        parallel_batch_size: Some(256),
        num_threads: Some(4),
        ..Default::default()
    };

    let loaded_storage = HNSWVectorStorage::from_path(&persistence_path, DIMENSIONS, config)
        .await
        .expect("Failed to load index");

    let load_time = load_start.elapsed();

    println!("\n=== Load Time Results ===");
    println!("Load time: {:.2}s", load_time.as_secs_f64());
    println!(
        "Vectors per second: {:.0}",
        VECTOR_COUNT as f64 / load_time.as_secs_f64()
    );

    // Verify load time is under 5 seconds
    assert!(
        load_time.as_secs() < MAX_LOAD_TIME_SECS,
        "Load time ({:.2}s) exceeds limit ({}s)",
        load_time.as_secs_f64(),
        MAX_LOAD_TIME_SECS
    );

    // Verify the loaded index works correctly
    let stats = loaded_storage.stats().await.expect("Failed to get stats");
    assert_eq!(
        stats.total_vectors, VECTOR_COUNT,
        "Loaded vector count mismatch"
    );

    // Test that search works on loaded index
    let query_vector = generate_random_vectors(1, DIMENSIONS)
        .into_iter()
        .next()
        .unwrap();
    let query = VectorQuery::new(query_vector.embedding, 10);
    let results = loaded_storage.search(&query).await.expect("Search failed");
    assert!(!results.is_empty(), "Loaded index should be searchable");

    println!("Load test completed successfully");
}

#[tokio::test]
async fn test_hnsw_error_handling() {
    // Test various error conditions
    let config = HNSWConfig::default();
    let storage = HNSWVectorStorage::new(384, config);

    // Test dimension mismatch
    let wrong_dim_vector = VectorEntry::new("wrong".to_string(), vec![0.1; 100]); // Wrong dimension
    let result = storage.insert(vec![wrong_dim_vector]).await;
    assert!(result.is_err(), "Should fail on dimension mismatch");
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("dimension mismatch"));

    // Test search with wrong dimensions
    let wrong_query = VectorQuery::new(vec![0.1; 100], 10);
    let result = storage.search(&wrong_query).await;
    assert!(result.is_err(), "Should fail on query dimension mismatch");

    // Test loading from inaccessible path (permission denied)
    #[cfg(unix)]
    {
        let bad_path = std::path::Path::new("/root/restricted");
        let result = HNSWVectorStorage::from_path(bad_path, 384, HNSWConfig::default()).await;
        // This may or may not fail depending on permissions, so just verify it doesn't panic
        let _ = result;
    }

    // Test that from_path with non-existent directory creates empty storage (this is correct behavior)
    let nonexistent_path = std::path::Path::new("/tmp/nonexistent_test_dir");
    let result = HNSWVectorStorage::from_path(nonexistent_path, 384, HNSWConfig::default()).await;
    assert!(
        result.is_ok(),
        "from_path should succeed even with non-existent directory"
    );
}
