// ABOUTME: Performance benchmarks for state persistence operations
// ABOUTME: Validates <5% overhead requirement for state save/load operations

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use llmspell_state_persistence::{
    agent_state::{
        AgentMetadata, AgentStateData, ExecutionState, PersistentAgentState, ToolUsageStats,
    },
    StateManager, StateScope,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::runtime::Runtime;

/// Benchmark basic state save operation
fn bench_state_save_basic(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("state_save_basic", |b| {
        b.iter(|| {
            rt.block_on(async {
                let state_manager = StateManager::new().await.unwrap();

                // Save some basic state
                let scope = StateScope::Agent("test-agent".to_string());
                let value = serde_json::json!({
                    "conversation": ["Hello", "Hi there!"],
                    "context": {"topic": "greeting"}
                });

                let _ = state_manager.set(scope, "state", value).await;

                black_box(state_manager)
            })
        });
    });
}

/// Benchmark state load operation
fn bench_state_load_basic(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("state_load_basic", |b| {
        b.iter(|| {
            rt.block_on(async {
                let state_manager = StateManager::new().await.unwrap();

                // Pre-save some state
                let scope = StateScope::Agent("test-agent".to_string());
                let value = serde_json::json!({
                    "conversation": ["Hello", "Hi there!"],
                    "context": {"topic": "greeting"}
                });
                state_manager
                    .set(scope.clone(), "state", value)
                    .await
                    .unwrap();

                // Load state
                let loaded = state_manager.get(scope, "state").await.unwrap();

                black_box(loaded)
            })
        });
    });
}

/// Benchmark state operations with large data
fn bench_state_large_data(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("state_save_large_data", |b| {
        b.iter(|| {
            rt.block_on(async {
                let state_manager = StateManager::new().await.unwrap();

                // Create large conversation history
                let mut conversation = Vec::new();
                for i in 0..100 {
                    conversation.push(serde_json::json!({
                        "role": if i % 2 == 0 { "user" } else { "assistant" },
                        "content": format!("This is message number {} with some content to make it realistic", i),
                        "timestamp": "2025-07-25T12:00:00Z"
                    }));
                }

                let scope = StateScope::Agent("test-agent".to_string());
                let value = serde_json::json!({
                    "conversation": conversation,
                    "metadata": {
                        "model": "gpt-4",
                        "total_tokens": 10000
                    }
                });

                // Save state
                let _ = state_manager.set(scope, "state", value).await;

                black_box(state_manager)
            })
        });
    });
}

/// Benchmark concurrent state operations
fn bench_state_concurrent_access(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("state_concurrent_access", |b| {
        b.iter(|| {
            rt.block_on(async {
                let state_manager = Arc::new(StateManager::new().await.unwrap());

                // Spawn multiple tasks accessing different agents
                let mut handles = vec![];

                for i in 0..10 {
                    let sm = state_manager.clone();
                    let handle = tokio::spawn(async move {
                        let scope = StateScope::Agent(format!("agent-{}", i));
                        let value = serde_json::json!({
                            "agent_id": format!("agent-{}", i),
                            "state": "active"
                        });

                        // Save and load state
                        sm.set(scope.clone(), "state", value).await.unwrap();
                        sm.get(scope, "state").await.unwrap()
                    });
                    handles.push(handle);
                }

                // Wait for all tasks
                for handle in handles {
                    let _ = handle.await;
                }

                black_box(state_manager)
            })
        });
    });
}

/// Benchmark state scope operations
fn bench_state_scope_isolation(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("state_scope_isolation", |b| {
        b.iter(|| {
            rt.block_on(async {
                let state_manager = StateManager::new().await.unwrap();

                // Test different scopes
                let scopes = [
                    StateScope::Global,
                    StateScope::Agent("agent-1".to_string()),
                    StateScope::Workflow("workflow-1".to_string()),
                    StateScope::Session("session-1".to_string()),
                ];

                for (i, scope) in scopes.iter().enumerate() {
                    let key = format!("key-{}", i);
                    let value = serde_json::json!({"data": i});

                    state_manager.set(scope.clone(), &key, value).await.unwrap();
                    let _ = state_manager.get(scope.clone(), &key).await.unwrap();
                }

                black_box(state_manager)
            })
        });
    });
}

/// Benchmark agent state persistence
fn bench_agent_state_persistence(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    c.bench_function("agent_state_save_load", |b| {
        b.iter(|| {
            rt.block_on(async {
                let state_manager = StateManager::new().await.unwrap();

                // Create a basic agent state
                let agent_state = PersistentAgentState {
                    agent_id: "test-agent".to_string(),
                    agent_type: "BasicAgent".to_string(),
                    state: AgentStateData {
                        conversation_history: vec![],
                        context_variables: HashMap::new(),
                        tool_usage_stats: ToolUsageStats::default(),
                        execution_state: ExecutionState::Idle,
                        custom_data: HashMap::new(),
                    },
                    metadata: AgentMetadata::default(),
                    creation_time: std::time::SystemTime::now(),
                    last_modified: std::time::SystemTime::now(),
                    schema_version: 1,
                    hook_registrations: vec![],
                    last_hook_execution: None,
                    correlation_context: None,
                };

                // Save and load agent state
                state_manager.save_agent_state(&agent_state).await.unwrap();
                let _ = state_manager
                    .load_agent_state(&agent_state.agent_id)
                    .await
                    .unwrap();

                black_box(state_manager)
            })
        });
    });
}

/// Calculate state persistence overhead
fn calculate_state_persistence_overhead(_c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    println!("\n=== State Persistence Overhead Analysis ===");

    rt.block_on(async {
        let state_manager = StateManager::new().await.unwrap();

        // Create test data
        let scope = StateScope::Agent("overhead-test".to_string());
        let test_data = serde_json::json!({
            "conversation": ["Hello", "Hi there!"],
            "context": {"topic": "greeting"}
        });

        // Baseline: Direct memory operations
        let start = tokio::time::Instant::now();
        for _ in 0..1000 {
            // Simulate direct memory write
            let _ = black_box(test_data.clone());
        }
        let baseline = start.elapsed();

        // With state persistence
        let start = tokio::time::Instant::now();
        for i in 0..1000 {
            let key = format!("key-{}", i);
            state_manager
                .set(scope.clone(), &key, test_data.clone())
                .await
                .unwrap();
        }
        let with_persistence = start.elapsed();

        let overhead_ns = with_persistence
            .as_nanos()
            .saturating_sub(baseline.as_nanos());
        let overhead_percent = (overhead_ns as f64 / baseline.as_nanos() as f64) * 100.0;

        println!("Baseline operation: {:?}", baseline);
        println!("With state persistence: {:?}", with_persistence);
        println!("Overhead: {:.2}%", overhead_percent);
        println!("Target: <5%");
        println!(
            "Status: {}",
            if overhead_percent < 5.0 {
                "PASS ✅"
            } else {
                "FAIL ❌"
            }
        );

        // Also test agent state persistence overhead
        println!("\n--- Agent State Persistence Overhead ---");

        let agent_state = PersistentAgentState {
            agent_id: "overhead-agent".to_string(),
            agent_type: "BasicAgent".to_string(),
            state: AgentStateData {
                conversation_history: vec![],
                context_variables: HashMap::new(),
                tool_usage_stats: ToolUsageStats::default(),
                execution_state: ExecutionState::Idle,
                custom_data: HashMap::new(),
            },
            metadata: AgentMetadata::default(),
            creation_time: std::time::SystemTime::now(),
            last_modified: std::time::SystemTime::now(),
            schema_version: 1,
            hook_registrations: vec![],
            last_hook_execution: None,
            correlation_context: None,
        };

        // Baseline for agent state
        let start = tokio::time::Instant::now();
        for _ in 0..100 {
            let _ = black_box(agent_state.clone());
        }
        let agent_baseline = start.elapsed();

        // With agent state persistence
        let start = tokio::time::Instant::now();
        for _ in 0..100 {
            state_manager.save_agent_state(&agent_state).await.unwrap();
        }
        let agent_with_persistence = start.elapsed();

        let agent_overhead_ns = agent_with_persistence
            .as_nanos()
            .saturating_sub(agent_baseline.as_nanos());
        let agent_overhead_percent =
            (agent_overhead_ns as f64 / agent_baseline.as_nanos() as f64) * 100.0;

        println!("Agent baseline operation: {:?}", agent_baseline);
        println!("With agent state persistence: {:?}", agent_with_persistence);
        println!("Agent state overhead: {:.2}%", agent_overhead_percent);
        println!(
            "Agent state status: {}",
            if agent_overhead_percent < 5.0 {
                "PASS ✅"
            } else {
                "FAIL ❌"
            }
        );
    });
}

criterion_group!(
    benches,
    bench_state_save_basic,
    bench_state_load_basic,
    bench_state_large_data,
    bench_state_concurrent_access,
    bench_state_scope_isolation,
    bench_agent_state_persistence,
    calculate_state_persistence_overhead
);
criterion_main!(benches);
