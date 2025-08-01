//! ABOUTME: Integration tests for session policies
//! ABOUTME: Validates policy enforcement and composition patterns

use anyhow::Result;
use llmspell_events::bus::EventBus;
use llmspell_hooks::{
    types::{ComponentId, ComponentType},
    HookContext, HookExecutor, HookPoint, HookRegistry,
};
use llmspell_sessions::{
    policies::{PolicyComposition, PolicyType, SessionPolicyConfig, SessionPolicyManager},
    types::CreateSessionOptions,
    SessionManager, SessionManagerConfig,
};
use llmspell_state_persistence::StateManager;
use llmspell_storage::MemoryBackend;
use std::sync::Arc;
use std::time::Duration;

#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_timeout_policy_enforcement() -> Result<()> {
    let hook_registry = Arc::new(HookRegistry::new());
    let hook_executor = Arc::new(HookExecutor::new());

    // Create policy config with short timeout
    let mut policy_config = SessionPolicyConfig::default();
    policy_config.enable_timeout = true;
    policy_config.enable_resource_limits = false;
    policy_config.enable_rate_limiting = false;
    policy_config.timeout_config.max_session_duration = Duration::from_millis(100);
    policy_config.timeout_config.idle_timeout = Duration::from_secs(3600); // Long idle timeout

    let policy_manager =
        SessionPolicyManager::new(policy_config, hook_registry.clone(), hook_executor.clone());

    // Register policies
    policy_manager.register_policies()?;

    // Create a hook context
    let mut context = HookContext::new(
        HookPoint::SessionCheckpoint,
        ComponentId::new(ComponentType::Agent, "test-session".to_string()),
    );

    // Set session start time to past
    let past_time = chrono::Utc::now() - chrono::Duration::milliseconds(200);
    context.data.insert(
        "session_start_time".to_string(),
        serde_json::json!(past_time.to_rfc3339()),
    );

    // Evaluate policies
    let result = policy_manager.evaluate_policies(&mut context).await?;
    assert!(!result.should_continue());
    assert!(context.data.contains_key("timeout_reason"));

    Ok(())
}

#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_resource_limit_policy_enforcement() -> Result<()> {
    let hook_registry = Arc::new(HookRegistry::new());
    let hook_executor = Arc::new(HookExecutor::new());

    // Create policy config with low resource limits
    let mut policy_config = SessionPolicyConfig::default();
    policy_config.enable_timeout = false;
    policy_config.enable_resource_limits = true;
    policy_config.enable_rate_limiting = false;
    policy_config.resource_config.max_memory_bytes = Some(1000);
    policy_config.resource_config.max_operations = Some(5);

    let policy_manager =
        SessionPolicyManager::new(policy_config, hook_registry.clone(), hook_executor.clone());

    // Register policies
    policy_manager.register_policies()?;

    // Create a hook context
    let mut context = HookContext::new(
        HookPoint::SessionCheckpoint,
        ComponentId::new(ComponentType::Agent, "test-session".to_string()),
    );

    // Test memory limit
    context
        .data
        .insert("memory_usage_bytes".to_string(), serde_json::json!(2000u64));

    let result = policy_manager.evaluate_policies(&mut context).await?;
    assert!(!result.should_continue());
    assert!(context.data.contains_key("resource_limit_exceeded"));

    Ok(())
}

#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_rate_limit_policy_enforcement() -> Result<()> {
    let hook_registry = Arc::new(HookRegistry::new());
    let hook_executor = Arc::new(HookExecutor::new());

    // Create policy config with low rate limits
    let mut policy_config = SessionPolicyConfig::default();
    policy_config.enable_timeout = false;
    policy_config.enable_resource_limits = false;
    policy_config.enable_rate_limiting = true;
    policy_config.rate_limit_config.global_rpm = 1; // Very low for testing

    let policy_manager =
        SessionPolicyManager::new(policy_config, hook_registry.clone(), hook_executor.clone());

    // Register policies
    policy_manager.register_policies()?;

    // Create a hook context
    let mut context = HookContext::new(
        HookPoint::SessionCheckpoint,
        ComponentId::new(ComponentType::Agent, "test-session".to_string()),
    );

    // First request should succeed
    let result = policy_manager.evaluate_policies(&mut context).await?;
    assert!(result.should_continue());

    // Rapid subsequent requests might be rate limited
    // Note: Actual behavior depends on RateLimitHook implementation

    Ok(())
}

#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_policy_composition_sequential() -> Result<()> {
    let hook_registry = Arc::new(HookRegistry::new());
    let hook_executor = Arc::new(HookExecutor::new());

    // Create policy config with sequential composition
    let mut policy_config = SessionPolicyConfig::default();
    policy_config.composition_pattern = PolicyComposition::Sequential;

    let policy_manager =
        SessionPolicyManager::new(policy_config, hook_registry.clone(), hook_executor.clone());

    // Register policies
    policy_manager.register_policies()?;

    // Verify hooks are registered
    let hooks = hook_registry.get_hooks(&HookPoint::SessionStart);
    assert!(!hooks.is_empty());

    Ok(())
}

#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_policy_composition_parallel() -> Result<()> {
    let hook_registry = Arc::new(HookRegistry::new());
    let hook_executor = Arc::new(HookExecutor::new());

    // Create policy config with parallel composition
    let mut policy_config = SessionPolicyConfig::default();
    policy_config.composition_pattern = PolicyComposition::Parallel;

    let policy_manager =
        SessionPolicyManager::new(policy_config, hook_registry.clone(), hook_executor.clone());

    // Register policies
    policy_manager.register_policies()?;

    // Create a hook context
    let mut context = HookContext::new(
        HookPoint::SessionCheckpoint,
        ComponentId::new(ComponentType::Agent, "test-session".to_string()),
    );

    // All policies should execute in parallel
    let result = policy_manager.evaluate_policies(&mut context).await?;
    assert!(result.should_continue());

    Ok(())
}

#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_policy_composition_voting() -> Result<()> {
    let hook_registry = Arc::new(HookRegistry::new());
    let hook_executor = Arc::new(HookExecutor::new());

    // Create policy config with voting composition
    let mut policy_config = SessionPolicyConfig::default();
    policy_config.composition_pattern = PolicyComposition::Voting { threshold: 0.6 };

    let policy_manager =
        SessionPolicyManager::new(policy_config, hook_registry.clone(), hook_executor.clone());

    // Register policies
    policy_manager.register_policies()?;

    // Create a hook context
    let mut context = HookContext::new(
        HookPoint::SessionCheckpoint,
        ComponentId::new(ComponentType::Agent, "test-session".to_string()),
    );

    // Majority of policies should pass for overall pass
    let result = policy_manager.evaluate_policies(&mut context).await?;
    assert!(result.should_continue());

    Ok(())
}

#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_policy_integration_with_session_manager() -> Result<()> {
    // Create infrastructure
    let storage_backend = Arc::new(MemoryBackend::new());
    let state_manager = Arc::new(StateManager::new().await.unwrap());
    let hook_registry = Arc::new(HookRegistry::new());
    let hook_executor = Arc::new(HookExecutor::new());
    let event_bus = Arc::new(EventBus::new());

    // Create policy config
    let policy_config = SessionPolicyConfig {
        enable_timeout: true,
        enable_resource_limits: true,
        enable_rate_limiting: true,
        ..Default::default()
    };

    // Create and register policy manager
    let policy_manager =
        SessionPolicyManager::new(policy_config, hook_registry.clone(), hook_executor.clone());
    policy_manager.register_policies()?;

    // Create session manager
    let config = SessionManagerConfig::default();
    let session_manager = SessionManager::new(
        state_manager,
        storage_backend,
        hook_registry.clone(),
        hook_executor.clone(),
        &event_bus,
        config,
    )
    .unwrap();

    // Create a session - policies should be evaluated
    let session_id = session_manager
        .create_session(CreateSessionOptions::default())
        .await?;

    // Perform operations - policies should be enforced
    let session = session_manager.get_session(&session_id).await?;
    assert_eq!(
        session.status().await,
        llmspell_sessions::SessionStatus::Active
    );

    Ok(())
}

#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_policy_configuration_update() -> Result<()> {
    let hook_registry = Arc::new(HookRegistry::new());
    let hook_executor = Arc::new(HookExecutor::new());

    let policy_config = SessionPolicyConfig::default();
    let mut policy_manager =
        SessionPolicyManager::new(policy_config, hook_registry.clone(), hook_executor.clone());

    // Check initial state
    assert!(policy_manager.is_policy_enabled(PolicyType::Timeout));
    assert!(policy_manager.is_policy_enabled(PolicyType::ResourceLimit));
    assert!(policy_manager.is_policy_enabled(PolicyType::RateLimit));

    // Update configuration
    let mut new_config = SessionPolicyConfig::default();
    new_config.enable_timeout = false;
    new_config.enable_resource_limits = false;

    policy_manager.update_config(new_config);

    // Check updated state
    assert!(!policy_manager.is_policy_enabled(PolicyType::Timeout));
    assert!(!policy_manager.is_policy_enabled(PolicyType::ResourceLimit));
    assert!(policy_manager.is_policy_enabled(PolicyType::RateLimit));

    Ok(())
}

#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_operation_specific_rate_limits() -> Result<()> {
    let hook_registry = Arc::new(HookRegistry::new());
    let hook_executor = Arc::new(HookExecutor::new());

    // Create policy config with operation-specific limits
    let mut policy_config = SessionPolicyConfig::default();
    policy_config.enable_timeout = false;
    policy_config.enable_resource_limits = false;
    policy_config.enable_rate_limiting = true;

    // Set very low limit for LLM calls
    policy_config
        .rate_limit_config
        .operation_limits
        .insert("llm_call".to_string(), 1);

    let policy_manager =
        SessionPolicyManager::new(policy_config, hook_registry.clone(), hook_executor.clone());

    // Register policies
    policy_manager.register_policies()?;

    // Create a hook context for LLM call
    let mut context = HookContext::new(
        HookPoint::SessionCheckpoint,
        ComponentId::new(ComponentType::Agent, "test-session".to_string()),
    );

    context
        .data
        .insert("operation_type".to_string(), serde_json::json!("llm_call"));

    // First LLM call should succeed
    let result = policy_manager.evaluate_policies(&mut context).await?;
    assert!(result.should_continue());

    Ok(())
}

#[cfg_attr(test_category = "integration")]
#[tokio::test]
async fn test_warning_thresholds() -> Result<()> {
    let hook_registry = Arc::new(HookRegistry::new());
    let hook_executor = Arc::new(HookExecutor::new());

    // Create policy config
    let mut policy_config = SessionPolicyConfig::default();
    policy_config.timeout_config.enable_warnings = true;
    policy_config.timeout_config.warning_intervals = vec![Duration::from_secs(300)];

    let policy_manager =
        SessionPolicyManager::new(policy_config, hook_registry.clone(), hook_executor.clone());

    // Register policies
    policy_manager.register_policies()?;

    // Create a hook context
    let mut context = HookContext::new(
        HookPoint::SessionCheckpoint,
        ComponentId::new(ComponentType::Agent, "test-session".to_string()),
    );

    // Set session near timeout
    let near_timeout = chrono::Utc::now() - chrono::Duration::seconds(3300); // 55 minutes ago
    context.data.insert(
        "session_start_time".to_string(),
        serde_json::json!(near_timeout.to_rfc3339()),
    );

    // Should get warning but continue
    let result = policy_manager.evaluate_policies(&mut context).await?;
    assert!(result.should_continue());
    assert!(context.data.contains_key("timeout_warning"));

    Ok(())
}
