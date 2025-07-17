//! ABOUTME: Resource exhaustion and DoS attack tests
//! ABOUTME: Tests for memory, CPU, disk, and network resource exhaustion

use super::test_framework::*;
use serde_json::json;

/// Memory exhaustion test vectors
pub fn memory_exhaustion_tests() -> Vec<SecurityTestCase> {
    vec![
        SecurityTestCase {
            name: "MEM_LARGE_ALLOCATION".to_string(),
            description: "Large memory allocation attempt".to_string(),
            payload: json!({
                "input": "A".repeat(100_000_000), // 100MB string
                "operation": "process"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::ResourceExhaustion],
        },
        SecurityTestCase {
            name: "MEM_EXPONENTIAL_GROWTH".to_string(),
            description: "Exponential memory growth".to_string(),
            payload: json!({
                "input": json!({
                    "data": vec![json!({"nested": vec![0; 1000]}); 1000]
                }),
                "operation": "expand"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::ResourceExhaustion],
        },
        SecurityTestCase {
            name: "MEM_RECURSIVE_STRUCTURE".to_string(),
            description: "Deeply recursive data structure".to_string(),
            payload: json!({
                "input": create_deep_json(1000),
                "operation": "parse"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::ResourceExhaustion],
        },
        SecurityTestCase {
            name: "MEM_STRING_CONCATENATION".to_string(),
            description: "Excessive string concatenation".to_string(),
            payload: json!({
                "input": "A",
                "operation": "repeat",
                "count": 100_000_000
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Low,
            categories: vec![TestCategory::ResourceExhaustion],
        },
    ]
}

/// CPU exhaustion test vectors
pub fn cpu_exhaustion_tests() -> Vec<SecurityTestCase> {
    vec![
        SecurityTestCase {
            name: "CPU_REGEX_CATASTROPHIC".to_string(),
            description: "Catastrophic regex backtracking".to_string(),
            payload: json!({
                "input": "a".repeat(50) + "b",
                "pattern": "^(a+)+$",
                "operation": "match"
            }),
            expected_behavior: ExpectedBehavior::Timeout,
            severity: Severity::High,
            categories: vec![TestCategory::ResourceExhaustion],
        },
        SecurityTestCase {
            name: "CPU_NESTED_LOOPS".to_string(),
            description: "Nested loop computation".to_string(),
            payload: json!({
                "expression": "sum([sum([i*j for j in range(1000)]) for i in range(1000)])",
                "operation": "calculate"
            }),
            expected_behavior: ExpectedBehavior::Timeout,
            severity: Severity::Medium,
            categories: vec![TestCategory::ResourceExhaustion],
        },
        SecurityTestCase {
            name: "CPU_COMPLEX_MATH".to_string(),
            description: "Complex mathematical expression".to_string(),
            payload: json!({
                "input": "2^2^2^2^2^2^2^2^2^2",
                "operation": "calculate"
            }),
            expected_behavior: ExpectedBehavior::Timeout,
            severity: Severity::High,
            categories: vec![TestCategory::ResourceExhaustion],
        },
        SecurityTestCase {
            name: "CPU_HASH_COLLISION".to_string(),
            description: "Hash collision attack".to_string(),
            payload: json!({
                "input": create_hash_collision_data(),
                "operation": "process"
            }),
            expected_behavior: ExpectedBehavior::Timeout,
            severity: Severity::Medium,
            categories: vec![TestCategory::ResourceExhaustion],
        },
        SecurityTestCase {
            name: "CPU_COMPRESSION_BOMB".to_string(),
            description: "Compression ratio attack".to_string(),
            payload: json!({
                "input": "0".repeat(1000),
                "operation": "compress",
                "iterations": 1000
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::ResourceExhaustion],
        },
    ]
}

/// Disk exhaustion test vectors
pub fn disk_exhaustion_tests() -> Vec<SecurityTestCase> {
    vec![
        SecurityTestCase {
            name: "DISK_LARGE_FILE_WRITE".to_string(),
            description: "Large file write attempt".to_string(),
            payload: json!({
                "path": "/tmp/large_file",
                "input": "A".repeat(1_000_000),
                "operation": "write",
                "repeat": 1000
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::ResourceExhaustion],
        },
        SecurityTestCase {
            name: "DISK_ZIP_BOMB".to_string(),
            description: "Zip bomb extraction".to_string(),
            payload: json!({
                "path": "test_files/zipbomb.zip",
                "operation": "extract",
                "target_path": "/tmp/extract"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::ResourceExhaustion],
        },
        SecurityTestCase {
            name: "DISK_RECURSIVE_SYMLINK".to_string(),
            description: "Recursive symlink creation".to_string(),
            payload: json!({
                "source_path": "/tmp/link1",
                "target_path": "/tmp/link2",
                "operation": "symlink",
                "recursive": true
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Low,
            categories: vec![TestCategory::ResourceExhaustion],
        },
        SecurityTestCase {
            name: "DISK_MANY_SMALL_FILES".to_string(),
            description: "Many small files creation".to_string(),
            payload: json!({
                "path_pattern": "/tmp/file_{}.txt",
                "count": 100000,
                "operation": "create_many"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::ResourceExhaustion],
        },
    ]
}

/// Network exhaustion test vectors
pub fn network_exhaustion_tests() -> Vec<SecurityTestCase> {
    vec![
        SecurityTestCase {
            name: "NET_RAPID_REQUESTS".to_string(),
            description: "Rapid API requests".to_string(),
            payload: json!({
                "input": "https://example.com/api",
                "count": 1000,
                "parallel": true
            }),
            expected_behavior: ExpectedBehavior::RateLimit,
            severity: Severity::Medium,
            categories: vec![TestCategory::ResourceExhaustion],
        },
        SecurityTestCase {
            name: "NET_LARGE_DOWNLOAD".to_string(),
            description: "Large file download".to_string(),
            payload: json!({
                "input": "https://example.com/1gb_file.bin",
                "operation": "download"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::ResourceExhaustion],
        },
        SecurityTestCase {
            name: "NET_SLOW_LORIS".to_string(),
            description: "Slowloris-style attack".to_string(),
            payload: json!({
                "input": "https://example.com",
                "headers": create_slow_headers(),
                "delay_ms": 10000
            }),
            expected_behavior: ExpectedBehavior::Timeout,
            severity: Severity::Low,
            categories: vec![TestCategory::ResourceExhaustion],
        },
        SecurityTestCase {
            name: "NET_CONNECTION_POOL".to_string(),
            description: "Connection pool exhaustion".to_string(),
            payload: json!({
                "connections": 1000,
                "keep_alive": true,
                "operation": "connect_many"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::ResourceExhaustion],
        },
    ]
}

/// Thread/process exhaustion test vectors
pub fn thread_exhaustion_tests() -> Vec<SecurityTestCase> {
    vec![
        SecurityTestCase {
            name: "THREAD_FORK_BOMB".to_string(),
            description: "Fork bomb attempt".to_string(),
            payload: json!({
                "command": ":(){ :|:& };:",
                "operation": "execute"
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Critical,
            categories: vec![TestCategory::ResourceExhaustion],
        },
        SecurityTestCase {
            name: "THREAD_MANY_ASYNC".to_string(),
            description: "Many async operations".to_string(),
            payload: json!({
                "operations": vec!["sleep 10"; 1000],
                "parallel": true
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::Medium,
            categories: vec![TestCategory::ResourceExhaustion],
        },
        SecurityTestCase {
            name: "THREAD_RECURSIVE_SPAWN".to_string(),
            description: "Recursive thread spawning".to_string(),
            payload: json!({
                "operation": "spawn_recursive",
                "depth": 1000
            }),
            expected_behavior: ExpectedBehavior::Reject,
            severity: Severity::High,
            categories: vec![TestCategory::ResourceExhaustion],
        },
    ]
}

/// Time-based exhaustion test vectors
pub fn time_exhaustion_tests() -> Vec<SecurityTestCase> {
    vec![
        SecurityTestCase {
            name: "TIME_LONG_SLEEP".to_string(),
            description: "Long sleep command".to_string(),
            payload: json!({
                "command": "sleep 3600",
                "operation": "execute"
            }),
            expected_behavior: ExpectedBehavior::Timeout,
            severity: Severity::Low,
            categories: vec![TestCategory::ResourceExhaustion],
        },
        SecurityTestCase {
            name: "TIME_INFINITE_LOOP".to_string(),
            description: "Infinite loop".to_string(),
            payload: json!({
                "input": "while(true){}",
                "operation": "evaluate"
            }),
            expected_behavior: ExpectedBehavior::Timeout,
            severity: Severity::Medium,
            categories: vec![TestCategory::ResourceExhaustion],
        },
        SecurityTestCase {
            name: "TIME_BLOCKING_IO".to_string(),
            description: "Blocking I/O operation".to_string(),
            payload: json!({
                "input": "/dev/random",
                "bytes": 1_000_000_000,
                "operation": "read"
            }),
            expected_behavior: ExpectedBehavior::Timeout,
            severity: Severity::Low,
            categories: vec![TestCategory::ResourceExhaustion],
        },
    ]
}

/// Algorithm complexity attack vectors
pub fn algorithmic_complexity_tests() -> Vec<SecurityTestCase> {
    vec![
        SecurityTestCase {
            name: "ALGO_QUADRATIC_JSON".to_string(),
            description: "Quadratic JSON parsing".to_string(),
            payload: json!({
                "input": create_nested_json_array(100),
                "operation": "parse"
            }),
            expected_behavior: ExpectedBehavior::Timeout,
            severity: Severity::Medium,
            categories: vec![TestCategory::ResourceExhaustion],
        },
        SecurityTestCase {
            name: "ALGO_WORST_CASE_SORT".to_string(),
            description: "Worst-case sorting input".to_string(),
            payload: json!({
                "input": create_worst_case_sort_data(10000),
                "operation": "sort"
            }),
            expected_behavior: ExpectedBehavior::Timeout,
            severity: Severity::Low,
            categories: vec![TestCategory::ResourceExhaustion],
        },
        SecurityTestCase {
            name: "ALGO_PATHOLOGICAL_TREE".to_string(),
            description: "Pathological tree structure".to_string(),
            payload: json!({
                "input": create_unbalanced_tree(1000),
                "operation": "traverse"
            }),
            expected_behavior: ExpectedBehavior::Timeout,
            severity: Severity::Medium,
            categories: vec![TestCategory::ResourceExhaustion],
        },
    ]
}

// Helper functions

fn create_deep_json(depth: usize) -> serde_json::Value {
    let mut json = json!({"value": "deep"});
    for _ in 0..depth {
        json = json!({"nested": json});
    }
    json
}

fn create_hash_collision_data() -> serde_json::Value {
    // Create data that causes hash collisions in common hash functions
    let mut data = serde_json::Map::new();
    for i in 0..1000 {
        // These strings are designed to collide in simple hash functions
        let key = format!("a{:0>5}", i * 31);
        data.insert(key, json!(i));
    }
    json!(data)
}

fn create_slow_headers() -> serde_json::Value {
    let mut headers = serde_json::Map::new();
    for i in 0..100 {
        headers.insert(format!("X-Slow-Header-{}", i), json!("a".repeat(1000)));
    }
    json!(headers)
}

fn create_nested_json_array(depth: usize) -> serde_json::Value {
    let mut arr = vec![];
    for i in 0..depth {
        arr.push(json!(vec![0; i]));
    }
    json!(arr)
}

fn create_worst_case_sort_data(size: usize) -> serde_json::Value {
    // Create data that triggers worst-case behavior in quicksort
    let data: Vec<i32> = (0..size as i32).rev().collect();
    json!(data)
}

fn create_unbalanced_tree(depth: usize) -> serde_json::Value {
    let mut tree = json!({"value": 0, "left": null, "right": null});
    for i in 1..depth {
        tree = json!({
            "value": i,
            "left": tree,
            "right": null
        });
    }
    tree
}

/// Create all resource exhaustion test cases
pub fn all_resource_exhaustion_tests() -> Vec<SecurityTestCase> {
    let mut tests = Vec::new();
    tests.extend(memory_exhaustion_tests());
    tests.extend(cpu_exhaustion_tests());
    tests.extend(disk_exhaustion_tests());
    tests.extend(network_exhaustion_tests());
    tests.extend(thread_exhaustion_tests());
    tests.extend(time_exhaustion_tests());
    tests.extend(algorithmic_complexity_tests());
    tests
}