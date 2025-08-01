// ABOUTME: Minimal performance test to verify setup
// ABOUTME: Simple benchmark to ensure criterion is working correctly

#![cfg_attr(test_category = "benchmark")]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::time::Duration;
use tokio::runtime::Runtime;

fn bench_minimal_test(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("minimal_test", |b| {
        b.iter(|| {
            rt.block_on(async {
                // Simple async operation
                tokio::time::sleep(Duration::from_micros(1)).await;
                black_box(42)
            })
        });
    });
}

criterion_group!(benches, bench_minimal_test);
criterion_main!(benches);
