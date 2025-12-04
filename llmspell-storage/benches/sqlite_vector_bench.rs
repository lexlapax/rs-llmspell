//! Benchmarks for SqliteVectorStorage with HNSW (Phase 13c.2.3a)
//!
//! Measures performance of:
//! - Vector insert operations (with SQLite + HNSW index)
//! - Vector search operations (HNSW approximate nearest neighbor)
//! - Scaling behavior at 1K, 10K, 100K vectors
//!
//! # Running Benchmarks
//!
//! ```bash
//! # Run all SQLite vector benchmarks
//! cargo bench --bench sqlite_vector_bench --features sqlite
//!
//! # Run specific benchmark
//! cargo bench --bench sqlite_vector_bench --features sqlite -- insert
//!
//! # Without SQLite feature (benchmarks will be skipped)
//! cargo bench --bench sqlite_vector_bench
//! ```
//!
//! # Performance Targets (Task 13c.2.3)
//!
//! - **Insert**: <1ms per vector (including SQLite write + HNSW index update)
//! - **Search**: <10ms for 10K vectors (HNSW approximate search)
//! - **Speedup**: 3-100x faster than sqlite-vec brute-force baseline
//!
//! # Baseline Comparison
//!
//! sqlite-vec brute-force search (cosine distance):
//! - 1K vectors: ~2-5ms
//! - 10K vectors: ~20-50ms
//! - 100K vectors: ~200-500ms
//!
//! Expected HNSW performance:
//! - 1K vectors: <1ms (3-5x speedup)
//! - 10K vectors: <10ms (3-5x speedup)
//! - 100K vectors: <50ms (4-10x speedup)

#[cfg(feature = "sqlite")]
mod sqlite_benchmarks {
    use criterion::{black_box, BenchmarkId, Criterion};
    use llmspell_core::state::StateScope;
    use llmspell_storage::backends::sqlite::{SqliteBackend, SqliteConfig, SqliteVectorStorage};
    use llmspell_storage::{VectorEntry, VectorQuery, VectorStorage};
    use std::sync::Arc;
    use tempfile::TempDir;
    use tokio::runtime::Runtime;

    /// Create test backend with tables for benchmarking
    async fn setup_test_backend(dimension: usize) -> (SqliteVectorStorage, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("bench.db");

        let config = SqliteConfig::new(&db_path).with_max_connections(20);
        let backend = Arc::new(SqliteBackend::new(config).await.unwrap());

        // Create tables (Migration V3 equivalent)
        let conn = backend.get_connection().await.unwrap();

        // Create vec_embeddings tables for all dimensions
        for dim in &[384, 768, 1536, 3072] {
            let create_sql = format!(
                "CREATE TABLE IF NOT EXISTS vec_embeddings_{} (rowid INTEGER PRIMARY KEY, embedding BLOB)",
                dim
            );
            conn.execute(&create_sql, ()).await.unwrap();
        }

        // Create vector_metadata table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS vector_metadata (
                rowid INTEGER PRIMARY KEY,
                id TEXT NOT NULL UNIQUE,
                tenant_id TEXT,
                scope TEXT NOT NULL,
                dimension INTEGER NOT NULL CHECK (dimension IN (384, 768, 1536, 3072)),
                metadata TEXT NOT NULL DEFAULT '{}',
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            (),
        )
        .await
        .unwrap();

        // Create indices
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_vector_metadata_tenant_scope ON vector_metadata(tenant_id, scope)",
            (),
        )
        .await
        .unwrap();

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_vector_metadata_id ON vector_metadata(id)",
            (),
        )
        .await
        .unwrap();

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_vector_metadata_dimension ON vector_metadata(dimension)",
            (),
        )
        .await
        .unwrap();

        let storage = SqliteVectorStorage::new(backend, dimension).await.unwrap();

        (storage, temp_dir)
    }

    /// Generate deterministic test vector
    fn create_test_vector(dimension: usize, seed: f32) -> Vec<f32> {
        (0..dimension).map(|i| seed + (i as f32 / 1000.0)).collect()
    }

    /// Benchmark: Insert operations at different scales
    pub fn bench_insert(c: &mut Criterion) {
        use std::sync::atomic::{AtomicUsize, Ordering};
        let rt = Runtime::new().unwrap();
        let mut group = c.benchmark_group("insert");

        for size in [100, 1000, 10_000].iter() {
            group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, &s| {
                let (storage, _temp) = rt.block_on(setup_test_backend(384));
                let counter = Arc::new(AtomicUsize::new(0));

                b.to_async(&rt).iter(|| {
                    let storage = &storage;
                    let counter = Arc::clone(&counter);
                    async move {
                        let idx = counter.fetch_add(1, Ordering::Relaxed);
                        // Insert single vector with unique ID
                        let entry = VectorEntry::new(
                            format!("vec-{}-{}", s, idx),
                            create_test_vector(384, s as f32),
                        )
                        .with_scope(StateScope::Global);

                        storage.insert(vec![black_box(entry)]).await.unwrap();
                    }
                });
            });
        }

        group.finish();
    }

    /// Benchmark: Search operations at different scales
    pub fn bench_search(c: &mut Criterion) {
        let rt = Runtime::new().unwrap();
        let mut group = c.benchmark_group("search");

        for size in [100, 1000, 10_000].iter() {
            // Setup: Pre-populate with vectors
            let (storage, _temp) = rt.block_on(async {
                let (storage, temp) = setup_test_backend(384).await;

                // Insert test vectors
                let mut entries = Vec::new();
                for i in 0..*size {
                    let entry =
                        VectorEntry::new(format!("vec-{}", i), create_test_vector(384, i as f32))
                            .with_scope(StateScope::Global);
                    entries.push(entry);
                }

                storage.insert(entries).await.unwrap();

                (storage, temp)
            });

            // Benchmark: Search for 10 nearest neighbors
            group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
                let query_vec = create_test_vector(384, (*size / 2) as f32);
                let query = VectorQuery::new(query_vec, 10);

                b.to_async(&rt).iter(|| async {
                    storage
                        .search_scoped(black_box(&query), black_box(&StateScope::Global))
                        .await
                        .unwrap()
                });
            });
        }

        group.finish();
    }

    /// Benchmark: Batch insert performance
    pub fn bench_batch_insert(c: &mut Criterion) {
        use std::sync::atomic::{AtomicUsize, Ordering};
        let rt = Runtime::new().unwrap();
        let mut group = c.benchmark_group("batch_insert");

        for batch_size in [10, 100, 1000].iter() {
            group.bench_with_input(
                BenchmarkId::from_parameter(batch_size),
                batch_size,
                |b, &s| {
                    let (storage, _temp) = rt.block_on(setup_test_backend(384));
                    let counter = Arc::new(AtomicUsize::new(0));

                    b.to_async(&rt).iter(|| {
                        let storage = &storage;
                        let counter = Arc::clone(&counter);
                        async move {
                            let batch_idx = counter.fetch_add(1, Ordering::Relaxed);
                            // Create batch of vectors with unique IDs
                            let mut entries = Vec::new();
                            for i in 0..s {
                                let entry = VectorEntry::new(
                                    format!("batch-vec-{}-{}", batch_idx, i),
                                    create_test_vector(384, i as f32),
                                )
                                .with_scope(StateScope::Global);
                                entries.push(entry);
                            }

                            storage.insert(black_box(entries)).await.unwrap();
                        }
                    });
                },
            );
        }

        group.finish();
    }
}

#[cfg(feature = "sqlite")]
use sqlite_benchmarks::{bench_batch_insert, bench_insert, bench_search};

#[cfg(not(feature = "sqlite"))]
use criterion::Criterion;

#[cfg(not(feature = "sqlite"))]
#[allow(clippy::missing_const_for_fn)]
fn bench_insert(_c: &mut Criterion) {}

#[cfg(not(feature = "sqlite"))]
#[allow(clippy::missing_const_for_fn)]
fn bench_search(_c: &mut Criterion) {}

#[cfg(not(feature = "sqlite"))]
#[allow(clippy::missing_const_for_fn)]
fn bench_batch_insert(_c: &mut Criterion) {}

use criterion::{criterion_group, criterion_main};

criterion_group!(benches, bench_insert, bench_search, bench_batch_insert);
criterion_main!(benches);
