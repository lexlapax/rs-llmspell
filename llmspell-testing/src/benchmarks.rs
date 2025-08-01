//! ABOUTME: Benchmark helpers and utilities for criterion performance testing
//! ABOUTME: Provides common benchmark scenarios and measurement helpers

//! Performance benchmarking utilities.
//!
//! This module provides helpers for creating criterion benchmarks
//! to measure the performance of LLMSpell components.
//!
//! # Examples
//!
//! ```rust,ignore
//! use criterion::{criterion_group, criterion_main, Criterion};
//! use llmspell_testing::benchmarks::{bench_trait_dispatch, bench_serialization};
//!
//! fn my_benchmarks(c: &mut Criterion) {
//!     bench_trait_dispatch(c);
//!     bench_serialization(c);
//! }
//!
//! criterion_group!(benches, my_benchmarks);
//! criterion_main!(benches);
//! ```

use criterion::{black_box, BenchmarkId, Criterion};
use llmspell_core::{types::AgentInput, ComponentId, ComponentMetadata, Version};
use std::time::Duration;

/// Benchmark trait method dispatch overhead
pub fn bench_trait_dispatch(c: &mut Criterion) {
    let mut group = c.benchmark_group("trait_dispatch");

    // Direct function call baseline
    group.bench_function("direct_call", |b| {
        b.iter(|| {
            let result = direct_function_call(black_box("test"));
            black_box(result)
        });
    });

    // TODO: Add trait dispatch benchmark when implementations are available

    group.finish();
}

/// Benchmark ComponentId generation
pub fn bench_component_id_generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("component_id");

    group.bench_function("new_uuid_v4", |b| {
        b.iter(|| {
            let id = ComponentId::new();
            black_box(id)
        });
    });

    group.bench_function("from_name", |b| {
        let name = "test-component-name";
        b.iter(|| {
            let id = ComponentId::from_name(black_box(name));
            black_box(id)
        });
    });

    group.finish();
}

/// Benchmark serialization performance
pub fn bench_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("serialization");

    // ComponentMetadata serialization
    let metadata = ComponentMetadata::new(
        "test-component".to_string(),
        "A test component for benchmarking".to_string(),
    );

    group.bench_function("metadata_to_json", |b| {
        b.iter(|| {
            let json = serde_json::to_string(black_box(&metadata)).unwrap();
            black_box(json)
        });
    });

    let json = serde_json::to_string(&metadata).unwrap();
    group.bench_function("metadata_from_json", |b| {
        b.iter(|| {
            let parsed: ComponentMetadata = serde_json::from_str(black_box(&json)).unwrap();
            black_box(parsed)
        });
    });

    // AgentInput serialization with varying sizes
    for size in [10, 100, 1000].iter() {
        let input = AgentInput::text("x".repeat(*size));

        group.bench_with_input(
            BenchmarkId::new("agent_input_to_json", size),
            size,
            |b, _| {
                b.iter(|| {
                    let json = serde_json::to_string(black_box(&input)).unwrap();
                    black_box(json)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark error creation and handling
pub fn bench_error_handling(c: &mut Criterion) {
    use llmspell_core::LLMSpellError;

    let mut group = c.benchmark_group("error_handling");

    group.bench_function("error_creation", |b| {
        b.iter(|| {
            let error = LLMSpellError::Component {
                message: "Test error".to_string(),
                source: None,
            };
            black_box(error)
        });
    });

    group.bench_function("error_with_source", |b| {
        b.iter(|| {
            let error = LLMSpellError::Storage {
                message: "Storage error".to_string(),
                operation: Some("read".to_string()),
                source: None,
            };
            black_box(error)
        });
    });

    group.finish();
}

/// Benchmark Version comparison operations
pub fn bench_version_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("version_operations");

    let v1 = Version {
        major: 1,
        minor: 2,
        patch: 3,
    };
    let v2 = Version {
        major: 1,
        minor: 2,
        patch: 4,
    };
    let _v3 = Version {
        major: 2,
        minor: 0,
        patch: 0,
    };

    group.bench_function("version_comparison", |b| {
        b.iter(|| {
            let result = black_box(&v1) < black_box(&v2);
            black_box(result)
        });
    });

    group.bench_function("version_is_compatible", |b| {
        b.iter(|| {
            let result = black_box(&v1).is_compatible_with(black_box(&v2));
            black_box(result)
        });
    });

    group.finish();
}

/// Helper function for measuring async operations
pub async fn measure_async_operation<F, Fut, T>(operation: F) -> (T, Duration)
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = T>,
{
    let start = std::time::Instant::now();
    let result = operation().await;
    let duration = start.elapsed();
    (result, duration)
}

/// Create a standard benchmark configuration
pub fn standard_benchmark_config() -> Criterion {
    Criterion::default()
        .sample_size(100)
        .measurement_time(Duration::from_secs(10))
        .warm_up_time(Duration::from_secs(3))
}

// Helper function for baseline comparison
fn direct_function_call(input: &str) -> String {
    format!("Processed: {}", input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_component_id_benchmark_data() {
        // Ensure benchmark data generation works
        let id = ComponentId::new();
        let _ = id; // Just verify it was created

        let id_from_name = ComponentId::from_name("test");
        let id_from_name2 = ComponentId::from_name("test");
        assert_eq!(id_from_name, id_from_name2); // Same name produces same ID
    }

    #[cfg_attr(test_category = "unit")]
    #[test]
    fn test_serialization_benchmark_data() {
        let metadata = ComponentMetadata::new("test".to_string(), "description".to_string());

        let json = serde_json::to_string(&metadata).unwrap();
        assert!(!json.is_empty());

        let parsed: ComponentMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "test");
    }

    #[cfg_attr(test_category = "unit")]
    #[tokio::test]
    async fn test_async_measurement() {
        let (result, duration) = measure_async_operation(|| async {
            tokio::time::sleep(Duration::from_millis(10)).await;
            42
        })
        .await;

        assert_eq!(result, 42);
        assert!(duration >= Duration::from_millis(10));
    }
}
