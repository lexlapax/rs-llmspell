// ABOUTME: Language-specific adapters for event handling
// ABOUTME: Stub implementation - will be fully implemented in Phase 15

use crate::universal_event::{Language, UniversalEvent};
use anyhow::Result;
use async_trait::async_trait;

/// Trait for language-specific event adapters
#[async_trait]
pub trait LanguageAdapter: Send + Sync {
    /// Convert event to language-specific format
    async fn adapt_event(&self, event: UniversalEvent) -> Result<serde_json::Value>;

    /// Get the target language
    fn target_language(&self) -> Language;
}

/// Lua event adapter (stub)
pub struct LuaEventAdapter;

#[async_trait]
impl LanguageAdapter for LuaEventAdapter {
    async fn adapt_event(&self, event: UniversalEvent) -> Result<serde_json::Value> {
        // Stub - will be implemented in Phase 15
        Ok(serde_json::to_value(&event)?)
    }

    fn target_language(&self) -> Language {
        Language::Lua
    }
}

/// JavaScript event adapter (stub)
pub struct JavaScriptEventAdapter;

#[async_trait]
impl LanguageAdapter for JavaScriptEventAdapter {
    async fn adapt_event(&self, event: UniversalEvent) -> Result<serde_json::Value> {
        // Stub - will be implemented in Phase 15
        Ok(serde_json::to_value(&event)?)
    }

    fn target_language(&self) -> Language {
        Language::JavaScript
    }
}

/// Python event adapter (stub)
pub struct PythonEventAdapter;

#[async_trait]
impl LanguageAdapter for PythonEventAdapter {
    async fn adapt_event(&self, event: UniversalEvent) -> Result<serde_json::Value> {
        // Stub - will be implemented in Phase 15
        Ok(serde_json::to_value(&event)?)
    }

    fn target_language(&self) -> Language {
        Language::Python
    }
}
