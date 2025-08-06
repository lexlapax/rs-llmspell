// ABOUTME: Performance benchmarks for state migration operations
// ABOUTME: Measures throughput, latency, and scalability of migration engine

// Benchmark file

use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use llmspell_state_persistence::{
    manager::SerializableState,
    migration::{DataTransformer, FieldTransform, StateTransformation},
};
use serde_json::json;
use std::collections::HashMap;
use std::time::SystemTime;

fn create_test_state(id: usize) -> SerializableState {
    #[allow(clippy::cast_precision_loss)]
    let price = id as f64 * 10.0;
    #[allow(clippy::cast_precision_loss)]
    let weight = id as f64 * 0.5;
    SerializableState {
        key: format!("item_{}", id),
        value: json!({
            "id": id,
            "name": format!("Item {}", id),
            "price": price,
            "quantity": id % 100,
            "metadata": {
                "created": "2024-01-01T00:00:00Z",
                "updated": "2024-01-01T00:00:00Z",
                "tags": vec![format!("tag{}", id % 10), format!("category{}", id % 5)],
                "attributes": {
                    "color": if id % 2 == 0 { "blue" } else { "red" },
                    "size": match id % 3 {
                        0 => "small",
                        1 => "medium",
                        _ => "large",
                    },
                    "weight": weight
                }
            }
        }),
        timestamp: SystemTime::now(),
        schema_version: 1,
    }
}

fn create_simple_transformation() -> StateTransformation {
    let mut transformation = StateTransformation::new(
        "simple_transform".to_string(),
        "Simple field rename and compute".to_string(),
        1,
        2,
    );

    transformation.add_transform(FieldTransform::Copy {
        from_field: "price".to_string(),
        to_field: "unit_price".to_string(),
    });

    transformation.add_transform(FieldTransform::Default {
        field: "total_value".to_string(),
        value: json!(0.0),
    });

    transformation
}

fn create_complex_transformation() -> StateTransformation {
    let mut transformation = StateTransformation::new(
        "complex_transform".to_string(),
        "Complex nested transformation".to_string(),
        1,
        2,
    );

    // Multiple field copies
    transformation.add_transform(FieldTransform::Custom {
        from_fields: vec!["metadata.attributes.color".to_string()],
        to_fields: vec!["display.color".to_string()],
        transformer: "copy_nested".to_string(),
        config: HashMap::new(),
    });

    transformation.add_transform(FieldTransform::Custom {
        from_fields: vec!["metadata.attributes.size".to_string()],
        to_fields: vec!["display.size".to_string()],
        transformer: "copy_nested".to_string(),
        config: HashMap::new(),
    });

    transformation.add_transform(FieldTransform::Custom {
        from_fields: vec!["metadata.attributes.weight".to_string()],
        to_fields: vec!["shipping.weight".to_string()],
        transformer: "copy_nested".to_string(),
        config: HashMap::new(),
    });

    // Copy and default
    transformation.add_transform(FieldTransform::Copy {
        from_field: "metadata.tags".to_string(),
        to_field: "categories".to_string(),
    });

    transformation.add_transform(FieldTransform::Default {
        field: "shipping.cost".to_string(),
        value: json!(10.0),
    });

    // Add defaults
    transformation.add_transform(FieldTransform::Default {
        field: "status".to_string(),
        value: json!("active"),
    });

    transformation.add_transform(FieldTransform::Default {
        field: "version".to_string(),
        value: json!(2),
    });

    transformation
}

fn benchmark_simple_transformation(c: &mut Criterion) {
    let mut group = c.benchmark_group("migration/simple_transformation");
    let transformer = DataTransformer::new();
    let transformation = create_simple_transformation();

    // Single item transformation
    group.bench_function("single_item", |b| {
        let state = create_test_state(1);
        b.iter(|| {
            let mut state_copy = state.clone();
            let result = transformer
                .transform_state(&mut state_copy, &transformation)
                .unwrap();
            black_box(result);
        });
    });

    // Batch sizes
    for size in [10, 100, 1000] {
        group.throughput(Throughput::Elements(size as u64));
        group.bench_function(format!("batch_{}", size), |b| {
            let mut states: Vec<_> = (0..size).map(create_test_state).collect();
            b.iter(|| {
                for state in &mut states {
                    let mut state_copy = state.clone();
                    let result = transformer
                        .transform_state(&mut state_copy, &transformation)
                        .unwrap();
                    black_box(result);
                }
            });
        });
    }

    group.finish();
}

fn benchmark_complex_transformation(c: &mut Criterion) {
    let mut group = c.benchmark_group("migration/complex_transformation");
    let transformer = DataTransformer::new();
    let transformation = create_complex_transformation();

    // Single item transformation
    group.bench_function("single_item", |b| {
        let state = create_test_state(1);
        b.iter(|| {
            let mut state_copy = state.clone();
            let result = transformer
                .transform_state(&mut state_copy, &transformation)
                .unwrap();
            black_box(result);
        });
    });

    // Batch sizes
    for size in [10, 100, 1000] {
        group.throughput(Throughput::Elements(size as u64));
        group.bench_function(format!("batch_{}", size), |b| {
            let mut states: Vec<_> = (0..size).map(create_test_state).collect();
            b.iter(|| {
                for state in &mut states {
                    let mut state_copy = state.clone();
                    let result = transformer
                        .transform_state(&mut state_copy, &transformation)
                        .unwrap();
                    black_box(result);
                }
            });
        });
    }

    group.finish();
}

fn benchmark_transformation_types(c: &mut Criterion) {
    let mut group = c.benchmark_group("migration/transformation_types");
    let transformer = DataTransformer::new();

    // Copy transformation
    group.bench_function("copy", |b| {
        let state = create_test_state(1);
        let mut transformation = StateTransformation::new("copy".to_string(), "".to_string(), 1, 2);
        transformation.add_transform(FieldTransform::Copy {
            from_field: "name".to_string(),
            to_field: "item_name".to_string(),
        });

        b.iter(|| {
            let mut state_copy = state.clone();
            let result = transformer
                .transform_state(&mut state_copy, &transformation)
                .unwrap();
            black_box(result);
        });
    });

    // Remove transformation
    group.bench_function("remove", |b| {
        let state = create_test_state(1);
        let mut transformation =
            StateTransformation::new("remove".to_string(), "".to_string(), 1, 2);
        transformation.add_transform(FieldTransform::Remove {
            field: "metadata".to_string(),
        });

        b.iter(|| {
            let mut state_copy = state.clone();
            let result = transformer
                .transform_state(&mut state_copy, &transformation)
                .unwrap();
            black_box(result);
        });
    });

    // Default transformation
    group.bench_function("default", |b| {
        let state = create_test_state(1);
        let mut transformation =
            StateTransformation::new("default".to_string(), "".to_string(), 1, 2);
        transformation.add_transform(FieldTransform::Default {
            field: "new_field".to_string(),
            value: json!("default_value"),
        });

        b.iter(|| {
            let mut state_copy = state.clone();
            let result = transformer
                .transform_state(&mut state_copy, &transformation)
                .unwrap();
            black_box(result);
        });
    });

    // Custom transformation
    group.bench_function("custom", |b| {
        let state = create_test_state(1);
        let mut transformation =
            StateTransformation::new("custom".to_string(), "".to_string(), 1, 2);
        transformation.add_transform(FieldTransform::Custom {
            from_fields: vec!["price".to_string(), "quantity".to_string()],
            to_fields: vec!["calculated".to_string()],
            transformer: "multiply".to_string(),
            config: HashMap::new(),
        });

        b.iter(|| {
            let mut state_copy = state.clone();
            let result = transformer
                .transform_state(&mut state_copy, &transformation)
                .unwrap();
            black_box(result);
        });
    });

    group.finish();
}

fn benchmark_large_dataset_migration(c: &mut Criterion) {
    let mut group = c.benchmark_group("migration/large_dataset");
    group.sample_size(10); // Reduce sample size for large datasets

    let transformer = DataTransformer::new();
    let transformation = create_simple_transformation();

    for size in [1000, 5000, 10000] {
        group.throughput(Throughput::Elements(size as u64));
        group.bench_function(format!("migrate_{}_items", size), |b| {
            let mut states: Vec<_> = (0..size).map(create_test_state).collect();

            b.iter(|| {
                let mut total_transformed = 0;
                for state in &mut states {
                    let mut state_copy = state.clone();
                    let result = transformer
                        .transform_state(&mut state_copy, &transformation)
                        .unwrap();
                    if result.success {
                        total_transformed += 1;
                    }
                    black_box(result);
                }
                assert_eq!(total_transformed, size);
            });
        });
    }

    group.finish();
}

fn benchmark_nested_data_depth(c: &mut Criterion) {
    let mut group = c.benchmark_group("migration/nested_depth");
    let transformer = DataTransformer::new();

    // Create states with varying nesting depths
    fn create_nested_state(depth: usize) -> SerializableState {
        let mut value = json!({"value": 42});
        for i in 0..depth {
            value = json!({
                format!("level_{}", i): value
            });
        }

        SerializableState {
            key: "nested".to_string(),
            value,
            timestamp: SystemTime::now(),
            schema_version: 1,
        }
    }

    for depth in [1, 5, 10, 20] {
        group.bench_function(format!("depth_{}", depth), |b| {
            let state = create_nested_state(depth);
            let mut transformation =
                StateTransformation::new("nested".to_string(), "".to_string(), 1, 2);

            // Create a move operation for the deepest field
            let mut path = String::new();
            for i in 0..depth {
                if !path.is_empty() {
                    path.push('.');
                }
                path.push_str(&format!("level_{}", i));
            }
            path.push_str(".value");

            transformation.add_transform(FieldTransform::Custom {
                from_fields: vec![path],
                to_fields: vec!["extracted_value".to_string()],
                transformer: "extract_nested".to_string(),
                config: HashMap::new(),
            });

            b.iter(|| {
                let mut state_copy = state.clone();
                let result = transformer
                    .transform_state(&mut state_copy, &transformation)
                    .unwrap();
                black_box(result);
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_simple_transformation,
    benchmark_complex_transformation,
    benchmark_transformation_types,
    benchmark_large_dataset_migration,
    benchmark_nested_data_depth
);
criterion_main!(benches);
