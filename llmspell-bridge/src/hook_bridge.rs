//! ABOUTME: Hook bridge for cross-language hook execution system
//! ABOUTME: Provides unified interface for scripts to register and execute hooks

#![allow(clippy::significant_drop_tightening)]

use crate::globals::types::GlobalContext;
use llmspell_core::error::LLMSpellError;
use llmspell_events::{
    universal_event::UniversalEventBuilder, Language as EventLanguage, UniversalEvent,
};
use llmspell_hooks::{
    Hook, HookAdapter, HookContext, HookExecutor, HookMetadata, HookPoint, HookRegistry,
    HookResult, Language, Priority,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::error;
use uuid::Uuid;

/// Wrapper for language-specific hooks
struct LanguageHook {
    id: String,
    language: Language,
    hook_point: HookPoint,
    priority: Priority,
    enabled: Arc<RwLock<bool>>,
    #[allow(dead_code)]
    callback: Arc<RwLock<Box<dyn std::any::Any + Send + Sync>>>,
    adapter:
        Arc<dyn HookAdapter<Context = Box<dyn std::any::Any>, Result = Box<dyn std::any::Any>>>,
}

impl std::fmt::Debug for LanguageHook {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LanguageHook")
            .field("id", &self.id)
            .field("language", &self.language)
            .field("hook_point", &self.hook_point)
            .field("priority", &self.priority)
            .field("enabled", &"<RwLock>")
            .field("callback", &"<dyn Any>")
            .field("adapter", &"<dyn HookAdapter>")
            .finish()
    }
}

/// Convert `LanguageHook` to a Hook trait implementation
#[derive(Debug)]
struct LanguageHookWrapper {
    inner: Arc<LanguageHook>,
}

#[async_trait::async_trait]
impl Hook for LanguageHookWrapper {
    async fn execute(&self, context: &mut HookContext) -> anyhow::Result<HookResult> {
        // Check if hook is enabled
        let enabled = *self.inner.enabled.read().await;
        if !enabled {
            return Ok(HookResult::Continue);
        }

        // Set the language in context
        context.language = self.inner.language;

        // Adapt the context to language-specific format
        let _lang_context = self.inner.adapter.adapt_context(context);

        // Execute the callback (this would be language-specific)
        // For now, we'll just return Continue
        // The actual execution will be handled by language-specific adapters
        Ok(HookResult::Continue)
    }

    fn metadata(&self) -> HookMetadata {
        HookMetadata {
            name: format!("{}_{}", self.inner.language, self.inner.id),
            description: Some(format!(
                "{} hook for {:?}",
                self.inner.language, self.inner.hook_point
            )),
            priority: self.inner.priority,
            language: self.inner.language,
            tags: vec![
                "cross-language".to_string(),
                self.inner.language.to_string(),
            ],
            version: "1.0.0".to_string(),
        }
    }

    fn should_execute(&self, context: &HookContext) -> bool {
        context.point == self.inner.hook_point
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

/// Hook registration handle for cleanup
#[derive(Clone)]
pub struct HookHandle {
    pub id: String,
    pub hook_point: HookPoint,
    pub language: Language,
}

/// Information about a registered hook
#[derive(Debug, Clone)]
pub struct HookInfo {
    pub id: String,
    pub hook_point: HookPoint,
    pub language: Language,
    pub priority: Priority,
    pub enabled: bool,
    pub metadata: HookMetadata,
}

/// Type alias for language adapter map
type AdapterMap = HashMap<
    Language,
    Arc<dyn HookAdapter<Context = Box<dyn std::any::Any>, Result = Box<dyn std::any::Any>>>,
>;

/// Bridge between scripts and the hook system
pub struct HookBridge {
    /// Hook executor from llmspell-hooks
    hook_executor: Arc<HookExecutor>,
    /// Hook registry from llmspell-hooks
    hook_registry: Arc<HookRegistry>,
    /// Language adapters
    adapters: Arc<RwLock<AdapterMap>>,
    /// Registered language hooks
    language_hooks: Arc<RwLock<HashMap<String, Arc<LanguageHook>>>>,
}

impl HookBridge {
    /// Create a new hook bridge
    ///
    /// # Errors
    ///
    /// Returns an error if hook executor or registry creation fails
    pub fn new(_context: Arc<GlobalContext>) -> Result<Self, LLMSpellError> {
        // Create hook executor and registry
        let hook_executor = Arc::new(HookExecutor::new());
        let hook_registry = Arc::new(HookRegistry::new());

        Ok(Self {
            hook_executor,
            hook_registry,
            adapters: Arc::new(RwLock::new(HashMap::new())),
            language_hooks: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Register a language adapter
    ///
    /// # Errors
    ///
    /// Returns an error if adapter registration fails
    pub async fn register_adapter(
        &self,
        language: Language,
        adapter: Arc<
            dyn HookAdapter<Context = Box<dyn std::any::Any>, Result = Box<dyn std::any::Any>>,
        >,
    ) -> Result<(), LLMSpellError> {
        let mut adapters = self.adapters.write().await;
        adapters.insert(language, adapter);
        Ok(())
    }

    /// Execute hooks for a given hook point
    ///
    /// # Errors
    ///
    /// Returns an error if hook execution fails for the given hook point
    pub async fn execute_hook(
        &self,
        hook_point: HookPoint,
        context: &mut HookContext,
    ) -> Result<HookResult, LLMSpellError> {
        // Get hooks for this point from the registry
        let hooks = self.hook_registry.get_hooks(&hook_point);

        // Execute hooks in sequence and aggregate results
        let results = self
            .hook_executor
            .execute_hooks(&hooks, context)
            .await
            .map_err(|e| LLMSpellError::Component {
                message: format!("Hook execution failed for {hook_point:?}: {e}"),
                source: None,
            })?;

        // Aggregate results - return the first non-Continue result, or Continue if all are Continue
        Ok(results
            .into_iter()
            .find(|r| !matches!(r, HookResult::Continue))
            .unwrap_or(HookResult::Continue))
    }

    /// Register a hook from a script language
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - No adapter is registered for the specified language
    /// - Hook registration fails
    pub async fn register_hook(
        &self,
        language: Language,
        hook_point: HookPoint,
        priority: Priority,
        callback: Box<dyn std::any::Any + Send + Sync>,
    ) -> Result<HookHandle, LLMSpellError> {
        // Get the adapter for this language
        let adapters = self.adapters.read().await;
        let adapter = adapters
            .get(&language)
            .ok_or_else(|| LLMSpellError::Configuration {
                message: format!("No adapter registered for language: {language}"),
                source: None,
            })?
            .clone();

        // Create a unique ID for this hook
        let id = Uuid::new_v4().to_string();

        // Create the language hook
        let language_hook = Arc::new(LanguageHook {
            id: id.clone(),
            language,
            hook_point: hook_point.clone(),
            priority,
            enabled: Arc::new(RwLock::new(true)),
            callback: Arc::new(RwLock::new(callback)),
            adapter,
        });

        // Store the language hook
        {
            let mut hooks = self.language_hooks.write().await;
            hooks.insert(id.clone(), language_hook.clone());
        }

        // Create the wrapper and register with the registry
        let wrapper = LanguageHookWrapper {
            inner: language_hook,
        };
        self.hook_registry
            .register(hook_point.clone(), wrapper)
            .map_err(|e| LLMSpellError::Configuration {
                message: format!("Failed to register hook: {e}"),
                source: None,
            })?;

        Ok(HookHandle {
            id,
            hook_point,
            language,
        })
    }

    /// Unregister a hook
    ///
    /// # Errors
    ///
    /// Returns an error if hook unregistration fails
    pub async fn unregister_hook(&self, handle: &HookHandle) -> Result<(), LLMSpellError> {
        // Remove from language hooks
        {
            let mut hooks = self.language_hooks.write().await;
            hooks.remove(&handle.id);
        }

        // Note: The HookRegistry doesn't currently support unregistration
        // This would need to be added to llmspell-hooks
        // For now, hooks will remain registered but won't execute without the language hook

        Ok(())
    }

    /// List all registered hooks for a hook point
    ///
    /// # Errors
    ///
    /// Returns an error if retrieving hook metadata fails
    pub async fn list_hooks(
        &self,
        hook_point: Option<HookPoint>,
    ) -> Result<Vec<HookMetadata>, LLMSpellError> {
        let language_hooks = self.language_hooks.read().await;
        let mut metadata = Vec::new();

        for (_, hook) in language_hooks.iter() {
            if hook_point.is_none() || hook_point.as_ref() == Some(&hook.hook_point) {
                let wrapper = LanguageHookWrapper {
                    inner: hook.clone(),
                };
                metadata.push(wrapper.metadata());
            }
        }

        // Sort by priority
        metadata.sort_by(|a, b| a.priority.cmp(&b.priority));

        Ok(metadata)
    }

    /// Get information about a specific hook
    ///
    /// # Errors
    ///
    /// Returns an error if retrieving hook information fails
    pub async fn get_hook_info(&self, hook_id: &str) -> Result<Option<HookInfo>, LLMSpellError> {
        let language_hooks = self.language_hooks.read().await;

        if let Some(hook) = language_hooks.get(hook_id) {
            let enabled = *hook.enabled.read().await;
            let wrapper = LanguageHookWrapper {
                inner: hook.clone(),
            };

            Ok(Some(HookInfo {
                id: hook.id.clone(),
                hook_point: hook.hook_point.clone(),
                language: hook.language,
                priority: hook.priority,
                enabled,
                metadata: wrapper.metadata(),
            }))
        } else {
            Ok(None)
        }
    }

    /// Enable a hook by ID
    ///
    /// # Errors
    ///
    /// Returns an error if enabling the hook fails
    pub async fn enable_hook(&self, hook_id: &str) -> Result<bool, LLMSpellError> {
        let language_hooks = self.language_hooks.read().await;

        if let Some(hook) = language_hooks.get(hook_id) {
            let mut enabled = hook.enabled.write().await;
            *enabled = true;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Disable a hook by ID
    ///
    /// # Errors
    ///
    /// Returns an error if hook state update fails
    pub async fn disable_hook(&self, hook_id: &str) -> Result<bool, LLMSpellError> {
        let language_hooks = self.language_hooks.read().await;

        if let Some(hook) = language_hooks.get(hook_id) {
            let mut enabled = hook.enabled.write().await;
            *enabled = false;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Execute hooks and publish integration events
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Hook execution fails
    /// - Event creation or publishing fails
    pub async fn execute_hook_with_events(
        &self,
        hook_point: HookPoint,
        context: &mut HookContext,
        event_bridge: Option<Arc<crate::event_bridge::EventBridge>>,
    ) -> Result<HookResult, LLMSpellError> {
        // Use the existing correlation ID from context
        let correlation_id = context.correlation_id;

        // Publish before hook event if event bridge is available
        if let Some(ref bridge) = event_bridge {
            let before_event =
                Self::create_hook_event(&hook_point, "before", correlation_id, context);
            if let Err(e) = bridge
                .publish_correlated_event(before_event, correlation_id)
                .await
            {
                // Log error but don't fail hook execution
                error!("Failed to publish before hook event: {e}");
            }
        }

        // Execute the hook
        let result = self.execute_hook(hook_point.clone(), context).await;

        // Publish after hook event
        if let Some(ref bridge) = event_bridge {
            let status = match &result {
                Ok(HookResult::Continue) => "success",
                Ok(HookResult::Modified(_)) => "modified",
                Ok(HookResult::Cancel(_)) => "cancelled",
                Ok(HookResult::Redirect(_)) => "redirected",
                Ok(HookResult::Replace(_)) => "replaced",
                Ok(HookResult::Retry { .. }) => "retry",
                Ok(HookResult::Fork { .. }) => "forked",
                Ok(HookResult::Cache { .. }) => "cached",
                Ok(HookResult::Skipped(_)) => "skipped",
                Err(_) => "error",
            };
            let after_event = Self::create_hook_event(&hook_point, status, correlation_id, context);
            if let Err(e) = bridge
                .publish_correlated_event(after_event, correlation_id)
                .await
            {
                error!("Failed to publish after hook event: {e}");
            }
        }

        result
    }

    /// Create a standardized hook event
    fn create_hook_event(
        hook_point: &HookPoint,
        status: &str,
        correlation_id: Uuid,
        context: &HookContext,
    ) -> UniversalEvent {
        let event_type = format!(
            "hook.{}.{}",
            hook_point.to_string().to_lowercase().replace(' ', "_"),
            status
        );

        // Create event data payload
        let event_data = serde_json::json!({
            "hook_point": hook_point.to_string(),
            "component_id": context.component_id.clone(),
            "language": context.language.to_string(),
            "status": status
        });

        UniversalEventBuilder::new(&event_type)
            .language(EventLanguage::Rust)
            .data(event_data)
            .source("hook_bridge")
            .correlation_id(correlation_id)
            .build()
    }

    /// Get metrics from the hook executor
    #[must_use]
    pub fn get_metrics(
        &self,
    ) -> std::collections::HashMap<String, llmspell_hooks::performance::PerformanceMetrics> {
        self.hook_executor.get_all_metrics()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use llmspell_config::providers::ProviderManagerConfig;
    #[tokio::test]
    async fn test_hook_bridge_creation() {
        let context = Arc::new(GlobalContext::new(
            Arc::new(crate::ComponentRegistry::new()),
            Arc::new(
                crate::ProviderManager::new(ProviderManagerConfig::default())
                    .await
                    .unwrap(),
            ),
        ));
        let bridge = HookBridge::new(context).unwrap();
        assert!(bridge.language_hooks.read().await.is_empty());
    }
    #[tokio::test]
    async fn test_hook_registration() {
        // Create a simple adapter for testing
        struct TestAdapter;
        impl HookAdapter for TestAdapter {
            type Context = Box<dyn std::any::Any>;
            type Result = Box<dyn std::any::Any>;

            fn adapt_context(&self, _ctx: &HookContext) -> Self::Context {
                Box::new(())
            }

            fn adapt_result(&self, _result: Self::Result) -> HookResult {
                HookResult::Continue
            }
        }

        let context = Arc::new(GlobalContext::new(
            Arc::new(crate::ComponentRegistry::new()),
            Arc::new(
                crate::ProviderManager::new(ProviderManagerConfig::default())
                    .await
                    .unwrap(),
            ),
        ));
        let bridge = HookBridge::new(context).unwrap();

        // Register the adapter
        bridge
            .register_adapter(Language::Lua, Arc::new(TestAdapter))
            .await
            .unwrap();

        // Register a hook
        let callback = Box::new("test_callback");
        let handle = bridge
            .register_hook(
                Language::Lua,
                HookPoint::BeforeToolExecution,
                Priority::NORMAL,
                callback,
            )
            .await
            .unwrap();

        assert_eq!(handle.language, Language::Lua);
        assert_eq!(handle.hook_point, HookPoint::BeforeToolExecution);

        // List hooks
        let hooks = bridge.list_hooks(None).await.unwrap();
        assert_eq!(hooks.len(), 1);
        assert!(hooks[0].name.starts_with("lua_"));
    }
}
