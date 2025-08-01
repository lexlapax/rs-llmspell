// ABOUTME: Performance benchmarks for file_utils module
// ABOUTME: Measures performance of path operations, file I/O, and directory management

#![cfg_attr(test_category = "benchmark")]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use llmspell_utils::file_utils::{
    copy_file, ensure_dir, expand_path, is_absolute_path, join_paths, normalize_path, parent_dir,
    read_file, write_file, write_file_atomic,
};
use std::env;
use std::path::Path;

fn bench_normalize_path(c: &mut Criterion) {
    c.bench_function("normalize_path", |b| {
        b.iter(|| normalize_path(black_box(Path::new("/home/user/../user/./docs/./file.txt"))));
    });
}

fn bench_expand_path(c: &mut Criterion) {
    env::set_var("BENCH_VAR", "/test/path");

    c.bench_function("expand_path", |b| {
        b.iter(|| expand_path(black_box("$BENCH_VAR/subdir/file.txt")).unwrap());
    });

    env::remove_var("BENCH_VAR");
}

fn bench_join_paths(c: &mut Criterion) {
    c.bench_function("join_paths", |b| {
        b.iter(|| {
            let paths: Vec<&Path> = vec![
                Path::new("/home"),
                Path::new("user"),
                Path::new("documents"),
                Path::new("project"),
            ];
            join_paths(black_box(&paths))
        });
    });
}

fn bench_is_absolute_path(c: &mut Criterion) {
    c.bench_function("is_absolute_path", |b| {
        b.iter(|| is_absolute_path(black_box(Path::new("/home/user/documents"))));
    });
}

fn bench_parent_dir(c: &mut Criterion) {
    c.bench_function("parent_dir", |b| {
        b.iter(|| parent_dir(black_box(Path::new("/home/user/documents/file.txt"))));
    });
}

fn bench_write_file(c: &mut Criterion) {
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("llmspell_bench_write");
    let data = vec![b'x'; 1024]; // 1KB of data

    c.bench_function("write_file_1kb", |b| {
        b.iter(|| write_file(black_box(&test_file), black_box(&data)).unwrap());
    });

    // Cleanup
    let _ = std::fs::remove_file(&test_file);
}

fn bench_write_file_atomic(c: &mut Criterion) {
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("llmspell_bench_atomic");
    let data = vec![b'x'; 1024]; // 1KB of data

    c.bench_function("write_file_atomic_1kb", |b| {
        b.iter(|| write_file_atomic(black_box(&test_file), black_box(&data)).unwrap());
    });

    // Cleanup
    let _ = std::fs::remove_file(&test_file);
}

fn bench_read_file(c: &mut Criterion) {
    let temp_dir = std::env::temp_dir();
    let test_file = temp_dir.join("llmspell_bench_read");
    let data = vec![b'x'; 1024]; // 1KB of data

    // Create file
    write_file(&test_file, &data).unwrap();

    c.bench_function("read_file_1kb", |b| {
        b.iter(|| read_file(black_box(&test_file)).unwrap());
    });

    // Cleanup
    let _ = std::fs::remove_file(&test_file);
}

fn bench_ensure_dir(c: &mut Criterion) {
    let temp_dir = std::env::temp_dir();
    let test_dir = temp_dir.join("llmspell_bench_dir");

    // Remove if exists
    let _ = std::fs::remove_dir_all(&test_dir);

    c.bench_function("ensure_dir_new", |b| {
        b.iter(|| {
            ensure_dir(black_box(&test_dir)).unwrap();
            // Remove for next iteration
            std::fs::remove_dir(&test_dir).unwrap();
        });
    });

    // Test with existing directory
    ensure_dir(&test_dir).unwrap();

    c.bench_function("ensure_dir_existing", |b| {
        b.iter(|| ensure_dir(black_box(&test_dir)).unwrap());
    });

    // Cleanup
    let _ = std::fs::remove_dir(&test_dir);
}

fn bench_copy_file(c: &mut Criterion) {
    let temp_dir = std::env::temp_dir();
    let source = temp_dir.join("llmspell_bench_src");
    let dest = temp_dir.join("llmspell_bench_dst");
    let data = vec![b'x'; 1024]; // 1KB of data

    // Create source file
    write_file(&source, &data).unwrap();

    c.bench_function("copy_file_1kb", |b| {
        b.iter(|| {
            copy_file(black_box(&source), black_box(&dest)).unwrap();
            // Remove dest for next iteration
            std::fs::remove_file(&dest).unwrap();
        });
    });

    // Cleanup
    let _ = std::fs::remove_file(&source);
}

criterion_group!(
    benches,
    bench_normalize_path,
    bench_expand_path,
    bench_join_paths,
    bench_is_absolute_path,
    bench_parent_dir,
    bench_write_file,
    bench_write_file_atomic,
    bench_read_file,
    bench_ensure_dir,
    bench_copy_file
);

criterion_main!(benches);
