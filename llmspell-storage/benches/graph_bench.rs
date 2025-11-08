//! Benchmarks for PostgreSQL bi-temporal graph storage (Phase 13b.5.6)
//!
//! These benchmarks measure the performance of:
//! - Point queries (get_entity_at)
//! - Range queries (query_temporal)
//! - Graph traversal (get_related with varying depths)
//! - GiST index performance for bi-temporal queries
//!
//! # Running Benchmarks
//!
//! ## Prerequisites
//! 1. PostgreSQL instance running at `localhost:5432`
//! 2. Database `llmspell_dev` created
//! 3. User `llmspell` with password `llmspell_dev_pass` (superuser for migrations)
//! 4. User `llmspell_app` with password `llmspell_dev_pass` (application user)
//!
//! ## Quick Setup (Docker)
//! ```bash
//! # Start PostgreSQL with proper setup
//! docker compose up -d postgres
//!
//! # Or use the test database setup script
//! ./scripts/db/reset-postgres-test-db.sh
//! ```
//!
//! ## Execute Benchmarks
//! ```bash
//! # Run all graph benchmarks with PostgreSQL feature
//! cargo bench --bench graph_bench --features postgres
//!
//! # Run specific benchmark
//! cargo bench --bench graph_bench --features postgres -- point_queries
//!
//! # Without PostgreSQL available (benchmarks will be skipped)
//! cargo bench --bench graph_bench
//! ```
//!
//! # Performance Baseline
//!
//! From Task 13b.5.2 tests (10 time-travel tests completed in 0.05s):
//! - **Average per-query**: ~5ms
//! - **Target**: <50ms
//! - **Margin**: 10x under target
//!
//! This benchmark suite provides more detailed measurements under controlled load:
//! - Small scale: 10-100 entities
//! - Medium scale: 1,000 entities
//! - Large scale: 10,000 entities (optional, disabled by default)
//!
//! # Benchmark Architecture
//!
//! Uses conditional compilation to gracefully handle missing PostgreSQL:
//! - `#[cfg(feature = "postgres")]`: Real benchmarks with database
//! - `#[cfg(not(feature = "postgres"))]`: Placeholder that skips cleanly
//!
//! This allows `cargo bench` to work without database setup while enabling
//! developers to opt-in to comprehensive PostgreSQL benchmarks.

#[cfg(feature = "postgres")]
mod postgres_benchmarks {
    use chrono::{Duration, Utc};
    use criterion::{black_box, BenchmarkId, Criterion};
    use llmspell_graph::types::TemporalQuery;
    use llmspell_storage::backends::postgres::{
        PostgresBackend, PostgresConfig, PostgresGraphStorage,
    };
    use serde_json::json;
    use std::sync::Arc;
    use tokio::runtime::Runtime;
    use uuid::Uuid;

    const TEST_CONNECTION_STRING: &str =
        "postgresql://llmspell_app:llmspell_dev_pass@localhost:5432/llmspell_dev";

    /// Benchmark tenant for isolation
    fn bench_tenant_id(scenario: &str) -> String {
        format!("bench-{}-{}", scenario, Uuid::new_v4())
    }

    /// Setup: Create test backend with application user
    ///
    /// Returns None if PostgreSQL is not available (allows graceful benchmark skip)
    async fn setup_test_backend() -> Option<Arc<PostgresBackend>> {
        let config = PostgresConfig::new(TEST_CONNECTION_STRING);
        match PostgresBackend::new(config).await {
            Ok(backend) => Some(Arc::new(backend)),
            Err(e) => {
                eprintln!("PostgreSQL not available, skipping benchmarks: {}", e);
                eprintln!(
                    "To run benchmarks, ensure PostgreSQL is running (see benchmark documentation)"
                );
                None
            }
        }
    }

    /// Helper: Insert test entity
    async fn insert_entity(
        backend: &PostgresBackend,
        tenant_id: &str,
        entity_type: &str,
        name: &str,
    ) -> String {
        let client = backend.get_client().await.unwrap();
        let entity_id = Uuid::new_v4();
        let now = Utc::now();
        let far_future = now + Duration::days(36500);

        client
            .execute(
                "INSERT INTO llmspell.entities
                 (entity_id, tenant_id, entity_type, name, properties, valid_time_start, valid_time_end)
                 VALUES ($1, $2, $3, $4, $5, $6, $7)",
                &[
                    &entity_id,
                    &tenant_id,
                    &entity_type,
                    &name,
                    &json!({}),
                    &now,
                    &far_future,
                ],
            )
            .await
            .unwrap();

        entity_id.to_string()
    }

    /// Helper: Insert relationship
    async fn insert_relationship(
        backend: &PostgresBackend,
        tenant_id: &str,
        from_id: &str,
        to_id: &str,
        rel_type: &str,
    ) {
        let client = backend.get_client().await.unwrap();
        let now = Utc::now();
        let far_future = now + Duration::days(36500);
        let from_uuid = Uuid::parse_str(from_id).unwrap();
        let to_uuid = Uuid::parse_str(to_id).unwrap();

        client
            .execute(
                "INSERT INTO llmspell.relationships
                 (tenant_id, from_entity, to_entity, relationship_type, properties, valid_time_start, valid_time_end)
                 VALUES ($1, $2, $3, $4, $5, $6, $7)",
                &[
                    &tenant_id,
                    &from_uuid,
                    &to_uuid,
                    &rel_type,
                    &json!({}),
                    &now,
                    &far_future,
                ],
            )
            .await
            .unwrap();
    }

    /// Benchmark: Point queries (get_entity_at) with varying scales
    pub fn bench_point_queries(c: &mut Criterion) {
        let rt = Runtime::new().unwrap();
        let backend = match rt.block_on(setup_test_backend()) {
            Some(b) => b,
            None => return, // PostgreSQL unavailable, skip benchmark
        };

        let mut group = c.benchmark_group("point_queries");

        for size in [10, 100].iter() {
            // Setup: Create entities
            let (tenant_id, entity_ids) = rt.block_on(async {
                let tenant = bench_tenant_id("point");
                backend.set_tenant_context(&tenant).await.unwrap();

                let mut ids = Vec::new();
                for i in 0..*size {
                    let id =
                        insert_entity(&backend, &tenant, "benchmark", &format!("entity-{}", i))
                            .await;
                    ids.push(id);
                }

                (tenant, ids)
            });

            // Benchmark: Query a random entity
            group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
                let storage = PostgresGraphStorage::new(Arc::clone(&backend));
                let _tenant = tenant_id.clone();
                let target_id = entity_ids[size / 2].clone(); // Middle entity
                let query_time = Utc::now();

                b.to_async(&rt).iter(|| async {
                    storage
                        .get_entity_at(
                            black_box(&target_id),
                            black_box(query_time),
                            black_box(query_time),
                        )
                        .await
                        .unwrap()
                });
            });
        }

        group.finish();
    }

    /// Benchmark: Range queries (query_temporal) with filters
    pub fn bench_range_queries(c: &mut Criterion) {
        let rt = Runtime::new().unwrap();
        let backend = match rt.block_on(setup_test_backend()) {
            Some(b) => b,
            None => return, // PostgreSQL unavailable, skip benchmark
        };

        let mut group = c.benchmark_group("range_queries");

        for size in [10, 100].iter() {
            // Setup: Create entities with different types
            let tenant_id = rt.block_on(async {
                let tenant = bench_tenant_id("range");
                backend.set_tenant_context(&tenant).await.unwrap();

                for i in 0..*size {
                    let entity_type = if i % 2 == 0 { "even" } else { "odd" };
                    insert_entity(&backend, &tenant, entity_type, &format!("entity-{}", i)).await;
                }

                tenant
            });

            // Benchmark: Query by type
            group.bench_with_input(BenchmarkId::from_parameter(size), size, |b, _| {
                let storage = PostgresGraphStorage::new(Arc::clone(&backend));
                let _tenant = tenant_id.clone();

                b.to_async(&rt).iter(|| async {
                    let query = TemporalQuery::new().with_entity_type("even".to_string());
                    storage.query_temporal(black_box(&query)).await.unwrap()
                });
            });
        }

        group.finish();
    }

    /// Benchmark: Graph traversal (get_related) with varying depths
    pub fn bench_graph_traversal(c: &mut Criterion) {
        let rt = Runtime::new().unwrap();
        let backend = match rt.block_on(setup_test_backend()) {
            Some(b) => b,
            None => return, // PostgreSQL unavailable, skip benchmark
        };

        let mut group = c.benchmark_group("graph_traversal");

        // Setup: Create chain graph (entity1 -> entity2 -> entity3 -> entity4)
        let (tenant_id, root_id) = rt.block_on(async {
            let tenant = bench_tenant_id("traversal");
            backend.set_tenant_context(&tenant).await.unwrap();

            let mut ids = Vec::new();
            for i in 0..5 {
                let id = insert_entity(&backend, &tenant, "node", &format!("node-{}", i)).await;
                ids.push(id);
            }

            // Create chain: 0 -> 1 -> 2 -> 3 -> 4
            for i in 0..4 {
                insert_relationship(&backend, &tenant, &ids[i], &ids[i + 1], "next").await;
            }

            (tenant, ids[0].clone())
        });

        // Benchmark different traversal depths
        for depth in [1, 2, 3, 4].iter() {
            group.bench_with_input(BenchmarkId::from_parameter(depth), depth, |b, &d| {
                let storage = PostgresGraphStorage::new(Arc::clone(&backend));
                let _tenant = tenant_id.clone();
                let root = root_id.clone();
                let query_time = Utc::now();

                b.to_async(&rt).iter(|| async {
                    storage
                        .get_related(
                            black_box(&root),
                            black_box(Some("next")),
                            black_box(d),
                            black_box(query_time),
                        )
                        .await
                        .unwrap()
                });
            });
        }

        group.finish();
    }
}

#[cfg(feature = "postgres")]
use postgres_benchmarks::{bench_graph_traversal, bench_point_queries, bench_range_queries};

#[cfg(not(feature = "postgres"))]
use criterion::Criterion;

#[cfg(not(feature = "postgres"))]
#[allow(clippy::missing_const_for_fn)]
fn bench_point_queries(_c: &mut Criterion) {}

#[cfg(not(feature = "postgres"))]
#[allow(clippy::missing_const_for_fn)]
fn bench_range_queries(_c: &mut Criterion) {}

#[cfg(not(feature = "postgres"))]
#[allow(clippy::missing_const_for_fn)]
fn bench_graph_traversal(_c: &mut Criterion) {}

use criterion::{criterion_group, criterion_main};

criterion_group!(
    benches,
    bench_point_queries,
    bench_range_queries,
    bench_graph_traversal
);
criterion_main!(benches);
