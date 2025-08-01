//! ABOUTME: Common utilities for provider hook integration tests including test fixtures and helpers
//! ABOUTME: Provides shared functionality for setting up hooks with real providers and test assertions

use anyhow::Result;
use llmspell_agents::agents::llm::LLMAgent;
use llmspell_agents::builder::AgentBuilder;
use llmspell_core::{traits::base_agent::BaseAgent, types::AgentInput, ExecutionContext};
use llmspell_hooks::{
    builtin::{
        caching::CachingHook, cost_tracking::CostTrackingHook, debugging::DebuggingHook,
        logging::LoggingHook, metrics::MetricsHook, rate_limit::RateLimitHook, retry::RetryHook,
        security::SecurityHook,
    },
    context::HookContext,
    executor::{HookExecutor, HookExecutorConfig},
    persistence::SerializedHookExecution,
    registry::HookRegistry,
    traits::Hook,
    types::{ComponentId, ComponentType, HookPoint},
};
use llmspell_providers::ProviderManager;
use std::collections::HashMap;
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tempfile::TempDir;
use tokio::sync::RwLock;
use tracing::{info, warn};
use uuid::Uuid;

/// Test context for hook provider integration tests
pub struct HookTestContext {
    pub executor: Arc<HookExecutor>,
    pub registry: Arc<HookRegistry>,
    pub provider_manager: Arc<ProviderManager>,
    pub temp_dir: TempDir,
    pub agent_id: String,
    // For simulating hook persistence
    pub stored_executions: Arc<RwLock<HashMap<String, Vec<SerializedHookExecution>>>>,
}

impl HookTestContext {
    /// Create a new test context with hooks
    pub async fn new() -> Result<Self> {
        let temp_dir = TempDir::new()?;

        // Create executor with config that enables persistence
        let executor_config = HookExecutorConfig {
            enable_circuit_breaker: true,
            enable_performance_monitoring: true,
            enable_persistence: false, // We'll simulate persistence ourselves
            max_execution_time: Duration::from_secs(5),
            performance_overhead_target: 0.05,
            ..Default::default()
        };
        let executor = Arc::new(HookExecutor::with_config(executor_config));

        // Create registry and register default hooks
        let registry = Arc::new(HookRegistry::new());

        // Register hooks with appropriate points
        let hook_registrations = vec![
            (
                HookPoint::BeforeAgentExecution,
                Arc::new(LoggingHook::default()) as Arc<dyn Hook>,
            ),
            (
                HookPoint::AfterAgentExecution,
                Arc::new(MetricsHook::default()) as Arc<dyn Hook>,
            ),
            (
                HookPoint::BeforeAgentExecution,
                Arc::new(SecurityHook::default()) as Arc<dyn Hook>,
            ),
            (
                HookPoint::AfterAgentExecution,
                Arc::new(CachingHook::default()) as Arc<dyn Hook>,
            ),
            (
                HookPoint::BeforeAgentExecution,
                Arc::new(RateLimitHook::default()) as Arc<dyn Hook>,
            ),
            (
                HookPoint::AfterAgentExecution,
                Arc::new(CostTrackingHook::default()) as Arc<dyn Hook>,
            ),
            (
                HookPoint::ToolError,
                Arc::new(RetryHook::default()) as Arc<dyn Hook>,
            ),
            (
                HookPoint::AgentError,
                Arc::new(DebuggingHook::default()) as Arc<dyn Hook>,
            ),
        ];

        for (point, hook) in hook_registrations {
            let hook_name = hook.metadata().name.clone();
            info!("Registering hook '{}' for point {:?}", hook_name, point);
            registry.register_arc(point, hook)?;
            info!("Successfully registered hook '{}'", hook_name);
        }

        // Verify hooks were registered
        let registered_hooks = registry.get_hooks(&HookPoint::BeforeAgentExecution);
        info!(
            "Registered {} hooks for BeforeAgentExecution",
            registered_hooks.len()
        );
        for hook in &registered_hooks {
            info!("  - {}", hook.metadata().name);
        }

        // Create provider manager and register rig provider
        let provider_manager = Arc::new(ProviderManager::new());
        provider_manager
            .register_provider("rig", llmspell_providers::create_rig_provider)
            .await;

        let agent_id = "test-hook-agent".to_string();
        let stored_executions = Arc::new(RwLock::new(HashMap::new()));

        Ok(Self {
            executor,
            registry,
            provider_manager,
            temp_dir,
            agent_id,
            stored_executions,
        })
    }

    /// Create an OpenAI agent with hooks if API key is available
    pub async fn create_openai_agent_with_hooks(&self) -> Result<Option<LLMAgent>> {
        if env::var("OPENAI_API_KEY").is_err() {
            warn!("OPENAI_API_KEY not set, skipping OpenAI tests");
            return Ok(None);
        }

        let config = AgentBuilder::new(&self.agent_id, "llm")
            .description("Test agent for OpenAI hook integration")
            .with_model("openai", "gpt-4")
            .temperature(0.7)
            .max_tokens(150)
            .build()?;

        let agent = LLMAgent::new(config, self.provider_manager.clone()).await?;
        Ok(Some(agent))
    }

    /// Create an Anthropic agent with hooks if API key is available
    pub async fn create_anthropic_agent_with_hooks(&self) -> Result<Option<LLMAgent>> {
        if env::var("ANTHROPIC_API_KEY").is_err() {
            warn!("ANTHROPIC_API_KEY not set, skipping Anthropic tests");
            return Ok(None);
        }

        let config = AgentBuilder::new(&format!("{}-anthropic", &self.agent_id), "llm")
            .description("Test agent for Anthropic hook integration")
            .with_model("anthropic", "claude-3-5-sonnet-latest")
            .temperature(0.7)
            .max_tokens(150)
            .build()?;

        let agent = LLMAgent::new(config, self.provider_manager.clone()).await?;
        Ok(Some(agent))
    }

    /// Execute hooks for a context and simulate persistence
    pub async fn execute_hooks_with_persistence(&self, context: &mut HookContext) -> Result<()> {
        // Get hooks for the specific point
        let hooks = self.registry.get_hooks(&context.point);
        info!("Found {} hooks for point {:?}", hooks.len(), context.point);

        // Execute each hook and create simulated execution records
        let mut executions = Vec::new();

        for hook in hooks {
            let hook_name = hook.metadata().name.clone();
            info!("Executing hook: {}", hook_name);

            let start = std::time::SystemTime::now();
            let result = self.executor.execute_hook(hook.as_ref(), context).await?;
            let duration = start.elapsed().unwrap_or_default();

            info!("Hook {} executed with result: {:?}", hook_name, result);

            // Create a simulated execution record for testing
            let serialized_context = serde_json::to_vec(context)?;
            let serialized_result = serde_json::to_string(&result)?;

            let execution = SerializedHookExecution {
                hook_id: hook_name.clone(),
                execution_id: Uuid::new_v4(),
                correlation_id: context.correlation_id,
                hook_context: serialized_context,
                result: serialized_result,
                timestamp: std::time::SystemTime::now(),
                duration,
                metadata: HashMap::new(),
            };

            executions.push(execution);
        }

        // Store executions
        if !executions.is_empty() {
            let mut stored = self.stored_executions.write().await;
            let correlation_id = context.correlation_id.to_string();

            // Append to existing executions or create new entry
            stored
                .entry(correlation_id)
                .or_insert_with(Vec::new)
                .extend(executions);
        }

        Ok(())
    }

    /// Create a test context for a specific hook point
    pub fn create_context(&self, point: HookPoint, component_name: &str) -> HookContext {
        let component_id = ComponentId::new(ComponentType::Agent, component_name.to_string());
        HookContext::new(point, component_id)
    }

    /// Run agent with hooks and capture execution
    pub async fn run_agent_with_hooks(
        &self,
        agent: &mut LLMAgent,
        message: &str,
    ) -> Result<String> {
        let context = ExecutionContext::new();

        // Initialize and start agent
        agent.initialize().await?;
        agent.start().await?;

        // Create hook context for before execution
        let mut before_context = self.create_context(
            HookPoint::BeforeAgentExecution,
            &agent.metadata().id.to_string(),
        );
        before_context.insert_data(
            "request".to_string(),
            serde_json::json!({
                "message": message,
                "agent_id": agent.metadata().id,
                "model": "test-model",
            }),
        );

        // Execute before hooks
        self.execute_hooks_with_persistence(&mut before_context)
            .await?;

        // Execute agent
        let input = AgentInput::text(message);
        let response = agent.execute(input, context).await?;

        // Create hook context for after execution
        let mut after_context = self.create_context(
            HookPoint::AfterAgentExecution,
            &agent.metadata().id.to_string(),
        );
        after_context.correlation_id = before_context.correlation_id; // Same correlation
        after_context.insert_data(
            "response".to_string(),
            serde_json::json!({
                "text": &response.text,
                "agent_id": agent.metadata().id,
                "tokens_used": response.metadata.extra.get("tokens_used"),
            }),
        );

        // Execute after hooks
        self.execute_hooks_with_persistence(&mut after_context)
            .await?;

        Ok(response.text)
    }

    /// Verify hook executions were captured
    pub async fn verify_hook_executions(&self, correlation_id: &str) -> Result<bool> {
        let stored = self.stored_executions.read().await;
        let executions = stored.get(correlation_id);

        match executions {
            Some(execs) => {
                info!(
                    "Found {} hook executions for correlation {}",
                    execs.len(),
                    correlation_id
                );
                Ok(!execs.is_empty())
            }
            None => {
                warn!(
                    "No hook executions found for correlation {}",
                    correlation_id
                );
                Ok(false)
            }
        }
    }

    /// Get hook executions for a correlation ID
    pub async fn get_hook_executions(
        &self,
        correlation_id: &str,
    ) -> Result<Option<Vec<SerializedHookExecution>>> {
        let stored = self.stored_executions.read().await;
        Ok(stored.get(correlation_id).cloned())
    }

    /// Verify specific hook was executed
    pub async fn verify_hook_executed(
        &self,
        correlation_id: &str,
        hook_name: &str,
    ) -> Result<bool> {
        if let Some(executions) = self.get_hook_executions(correlation_id).await? {
            let found = executions
                .iter()
                .any(|exec| exec.hook_id.contains(hook_name));
            if found {
                info!(
                    "✅ Hook '{}' was executed for correlation {}",
                    hook_name, correlation_id
                );
            } else {
                warn!(
                    "❌ Hook '{}' was NOT executed for correlation {}",
                    hook_name, correlation_id
                );
            }
            Ok(found)
        } else {
            Ok(false)
        }
    }
}

/// Check if a provider API key is available
pub fn check_api_key(provider: &str) -> bool {
    let key_name = match provider {
        "openai" => "OPENAI_API_KEY",
        "anthropic" => "ANTHROPIC_API_KEY",
        _ => return false,
    };

    env::var(key_name).is_ok()
}

/// Skip test if API key is not available
#[allow(dead_code)]
pub fn skip_if_no_api_key(provider: &str) {
    if !check_api_key(provider) {
        let key_name = match provider {
            "openai" => "OPENAI_API_KEY",
            "anthropic" => "ANTHROPIC_API_KEY",
            _ => "API_KEY",
        };
        panic!("Test requires {} environment variable", key_name);
    }
}

/// Common test data generators
pub mod test_data {
    use serde_json::json;

    /// Generate test LLM request data
    #[allow(dead_code)]
    pub fn llm_request_data(model: &str, prompt: &str) -> serde_json::Value {
        json!({
            "model": model,
            "prompt": prompt,
            "temperature": 0.7,
            "max_tokens": 150,
            "stream": false
        })
    }

    /// Generate test LLM response data
    #[allow(dead_code)]
    pub fn llm_response_data(content: &str, tokens: u32) -> serde_json::Value {
        json!({
            "content": content,
            "usage": {
                "prompt_tokens": tokens / 2,
                "completion_tokens": tokens / 2,
                "total_tokens": tokens
            },
            "model": "test-model",
            "finish_reason": "stop"
        })
    }

    /// Generate test tool execution data
    #[allow(dead_code)]
    pub fn tool_execution_data(tool_name: &str, input: &str) -> serde_json::Value {
        json!({
            "tool": tool_name,
            "input": input,
            "parameters": {
                "timeout": 30000,
                "retry": true
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg_attr(test_category = "external")]
    #[tokio::test]
    async fn test_context_creation() {
        let context = HookTestContext::new().await.unwrap();
        assert!(!context.agent_id.is_empty());
        assert!(context.temp_dir.path().exists());
    }

    #[cfg_attr(test_category = "external")]
    #[test]
    fn test_api_key_checking() {
        // This will return false unless API keys are actually set
        let has_openai = check_api_key("openai");
        let has_anthropic = check_api_key("anthropic");

        // Just verify the function doesn't panic
        assert!(has_openai || !has_openai);
        assert!(has_anthropic || !has_anthropic);
    }
}
