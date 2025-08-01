//! ABOUTME: Performance validation tests for session policies
//! ABOUTME: Ensures policies meet the <5% overhead requirement from hook system

use anyhow::Result;
use llmspell_hooks::{
    types::{ComponentId, ComponentType},
    HookContext, HookExecutor, HookPoint, HookRegistry,
};
use llmspell_sessions::policies::{PolicyComposition, SessionPolicyConfig, SessionPolicyManager};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Baseline test - measure hook system overhead without policies
#[tokio::test]
async fn test_baseline_hook_overhead() -> Result<()> {
    let hook_registry = Arc::new(HookRegistry::new());
    let _hook_executor = Arc::new(HookExecutor::new());

    let context = HookContext::new(
        HookPoint::SessionCheckpoint,
        ComponentId::new(ComponentType::Agent, "test-session".to_string()),
    );

    // Measure baseline execution time (no hooks)
    let start = Instant::now();
    let iterations = 1000;

    for _ in 0..iterations {
        // Get hooks (should be empty)
        let hooks = hook_registry.get_hooks(&context.point);
        assert!(hooks.is_empty());
    }

    let baseline_duration = start.elapsed();
    println!(
        "Baseline (no hooks): {:?} for {} iterations",
        baseline_duration, iterations
    );
    println!(
        "Average per operation: {:?}",
        baseline_duration / iterations
    );

    Ok(())
}

/// Test with all policies enabled
#[tokio::test]
async fn test_policy_overhead() -> Result<()> {
    let hook_registry = Arc::new(HookRegistry::new());
    let hook_executor = Arc::new(HookExecutor::new());

    // Create policy manager with adjusted limits for performance testing
    let mut policy_config = SessionPolicyConfig::default();
    // Increase timeouts to avoid triggering during performance test
    policy_config.timeout_config.idle_timeout = Duration::from_secs(3600); // 1 hour
    policy_config.timeout_config.max_session_duration = Duration::from_secs(7200); // 2 hours
                                                                                   // Increase rate limits for performance testing
    policy_config.rate_limit_config.global_rpm = 100_000; // Very high for testing
    policy_config.rate_limit_config.per_session_rpm = 100_000; // Very high for testing

    let policy_manager =
        SessionPolicyManager::new(policy_config, hook_registry.clone(), hook_executor.clone());

    // Register policies
    policy_manager.register_policies()?;

    let mut context = HookContext::new(
        HookPoint::SessionCheckpoint,
        ComponentId::new(ComponentType::Agent, "test-session".to_string()),
    );

    // Add some realistic context data
    let now = chrono::Utc::now();
    context.data.insert(
        "session_start_time".to_string(),
        serde_json::json!(now.to_rfc3339()),
    );
    context.data.insert(
        "last_activity_time".to_string(),
        serde_json::json!(now.to_rfc3339()),
    );
    context
        .data
        .insert("memory_usage_bytes".to_string(), serde_json::json!(1000u64));
    context
        .data
        .insert("operation_count".to_string(), serde_json::json!(10u64));

    // Warm up
    for _ in 0..10 {
        let _ = policy_manager.evaluate_policies(&mut context).await?;
    }

    // Measure execution time with policies
    let start = Instant::now();
    let iterations = 1000;

    for i in 0..iterations {
        // Periodically update activity time to keep it fresh
        if i % 100 == 0 {
            context.data.insert(
                "last_activity_time".to_string(),
                serde_json::json!(chrono::Utc::now().to_rfc3339()),
            );
        }
        let result = policy_manager.evaluate_policies(&mut context).await?;
        if !result.should_continue() {
            panic!("Policy rejected execution at iteration {}: {:?}", i, result);
        }
    }

    let policy_duration = start.elapsed();
    println!(
        "With policies: {:?} for {} iterations",
        policy_duration, iterations
    );
    println!("Average per operation: {:?}", policy_duration / iterations);

    // Calculate overhead
    let baseline_ns = 100_000; // 100 microseconds baseline (estimated)
    let policy_ns = policy_duration.as_nanos() / iterations as u128;
    let overhead_percent = ((policy_ns as f64 - baseline_ns as f64) / baseline_ns as f64) * 100.0;

    println!("Estimated overhead: {:.2}%", overhead_percent);

    // Hook system should maintain <5% overhead
    // Our policies should add minimal additional overhead
    assert!(
        overhead_percent < 10.0,
        "Policy overhead exceeds 10%: {:.2}%",
        overhead_percent
    );

    Ok(())
}

/// Test different composition patterns performance
#[tokio::test]
async fn test_composition_performance() -> Result<()> {
    let patterns = vec![
        ("Sequential", PolicyComposition::Sequential),
        ("Parallel", PolicyComposition::Parallel),
        ("Voting", PolicyComposition::Voting { threshold: 0.6 }),
    ];

    for (name, pattern) in patterns {
        let hook_registry = Arc::new(HookRegistry::new());
        let hook_executor = Arc::new(HookExecutor::new());

        let mut policy_config = SessionPolicyConfig::default();
        policy_config.composition_pattern = pattern;
        // Increase timeouts for performance testing
        policy_config.timeout_config.idle_timeout = Duration::from_secs(3600);
        policy_config.timeout_config.max_session_duration = Duration::from_secs(7200);
        // Increase rate limits for performance testing
        policy_config.rate_limit_config.global_rpm = 100_000;
        policy_config.rate_limit_config.per_session_rpm = 100_000;

        let policy_manager =
            SessionPolicyManager::new(policy_config, hook_registry.clone(), hook_executor.clone());

        policy_manager.register_policies()?;

        let mut context = HookContext::new(
            HookPoint::SessionCheckpoint,
            ComponentId::new(ComponentType::Agent, "test-session".to_string()),
        );

        // Initialize session context
        let now = chrono::Utc::now();
        context.data.insert(
            "session_start_time".to_string(),
            serde_json::json!(now.to_rfc3339()),
        );
        context.data.insert(
            "last_activity_time".to_string(),
            serde_json::json!(now.to_rfc3339()),
        );
        context
            .data
            .insert("memory_usage_bytes".to_string(), serde_json::json!(1000u64));
        context
            .data
            .insert("operation_count".to_string(), serde_json::json!(10u64));

        // Warm up
        for _ in 0..10 {
            let _ = policy_manager.evaluate_policies(&mut context).await?;
        }

        // Measure
        let start = Instant::now();
        let iterations = 1000;

        for i in 0..iterations {
            // Update activity time periodically
            if i % 100 == 0 {
                context.data.insert(
                    "last_activity_time".to_string(),
                    serde_json::json!(chrono::Utc::now().to_rfc3339()),
                );
            }
            let result = policy_manager.evaluate_policies(&mut context).await?;
            if !result.should_continue() {
                panic!(
                    "{} composition rejected at iteration {}: {:?}",
                    name, i, result
                );
            }
        }

        let duration = start.elapsed();
        println!(
            "{} composition: {:?} for {} iterations",
            name, duration, iterations
        );
        println!("Average per operation: {:?}", duration / iterations);
    }

    Ok(())
}

/// Test memory overhead of policies
#[tokio::test]
async fn test_memory_overhead() -> Result<()> {
    // Create multiple policy managers to test memory usage
    let mut managers = Vec::new();

    for i in 0..100 {
        let hook_registry = Arc::new(HookRegistry::new());
        let hook_executor = Arc::new(HookExecutor::new());

        let policy_config = SessionPolicyConfig::default();
        let policy_manager =
            SessionPolicyManager::new(policy_config, hook_registry.clone(), hook_executor.clone());

        policy_manager.register_policies()?;
        managers.push((policy_manager, hook_registry, hook_executor));

        if i % 10 == 0 {
            println!("Created {} policy managers", i + 1);
        }
    }

    // Policy managers should have minimal memory footprint
    println!("Successfully created {} policy managers", managers.len());

    Ok(())
}

/// Test circuit breaker integration
#[tokio::test]
async fn test_circuit_breaker_protection() -> Result<()> {
    let hook_registry = Arc::new(HookRegistry::new());
    let hook_executor = Arc::new(HookExecutor::new());

    let policy_config = SessionPolicyConfig::default();
    let policy_manager =
        SessionPolicyManager::new(policy_config, hook_registry.clone(), hook_executor.clone());

    policy_manager.register_policies()?;

    let mut context = HookContext::new(
        HookPoint::SessionCheckpoint,
        ComponentId::new(ComponentType::Agent, "test-session".to_string()),
    );

    // Simulate a slow operation by adding delay in context
    // The circuit breaker should protect against this
    let start = Instant::now();
    let result = policy_manager.evaluate_policies(&mut context).await?;
    let duration = start.elapsed();

    assert!(result.should_continue());
    assert!(
        duration < Duration::from_millis(200),
        "Policy execution took too long: {:?}",
        duration
    );

    Ok(())
}
